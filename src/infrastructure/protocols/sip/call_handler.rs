//! Call handling (INVITE, ACK, BYE)

use super::auth::SipAuthenticator;
use super::builder::ResponseBuilder;
use super::call_router::CallRouter;
use super::handler::SipHandler;
use super::hold_manager::SdpHoldHelper;
use super::message::{SipError, SipMethod, SipRequest, SipResponse};
use super::registrar::Registrar;
use super::sdp::SdpSession;
use crate::domain::cdr::CdrRepository;
use crate::infrastructure::media::{CodecNegotiator, MediaBridge, MediaStream, StreamDirection};
use async_trait::async_trait;
use rsip::Header;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Active call session
pub struct CallSession {
    pub call_id: String,
    pub from_uri: String,
    pub to_uri: String,
    pub state: CallSessionState,
    pub media_bridge: Option<Arc<MediaBridge>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallSessionState {
    Inviting,
    Ringing,
    Answered,
    Terminated,
}

/// INVITE handler
pub struct InviteHandler {
    registrar: Arc<Registrar>,
    pub active_calls: Arc<RwLock<HashMap<String, CallSession>>>,
    local_ip: IpAddr,
    auth: Option<Arc<dyn SipAuthenticator>>,
    codec_negotiator: CodecNegotiator,
    next_rtp_port: Arc<RwLock<u16>>,
    call_router: Arc<CallRouter>,
    /// Enable auto-answer mode (for testing/simple PBX)
    auto_answer: bool,
}

impl InviteHandler {
    pub fn new(registrar: Arc<Registrar>, local_ip: IpAddr) -> Self {
        let call_router = Arc::new(CallRouter::new(registrar.clone()));
        Self {
            registrar: registrar.clone(),
            active_calls: Arc::new(RwLock::new(HashMap::new())),
            local_ip,
            auth: None,
            codec_negotiator: CodecNegotiator::new(),
            next_rtp_port: Arc::new(RwLock::new(10000)),
            call_router,
            auto_answer: true, // Default to auto-answer for backward compatibility
        }
    }

    /// Create handler with authentication
    pub fn with_auth(registrar: Arc<Registrar>, local_ip: IpAddr, auth: Arc<dyn SipAuthenticator>) -> Self {
        let call_router = Arc::new(CallRouter::new(registrar.clone()));
        Self {
            registrar: registrar.clone(),
            active_calls: Arc::new(RwLock::new(HashMap::new())),
            local_ip,
            auth: Some(auth),
            codec_negotiator: CodecNegotiator::new(),
            next_rtp_port: Arc::new(RwLock::new(10000)),
            call_router,
            auto_answer: true,
        }
    }

    /// Enable or disable auto-answer mode
    pub fn set_auto_answer(&mut self, auto_answer: bool) {
        self.auto_answer = auto_answer;
    }

    /// Set authentication (for existing handler)
    pub fn set_auth(&mut self, auth: Arc<dyn SipAuthenticator>) {
        self.auth = Some(auth);
    }

    /// Set CDR repository (for existing handler)
    pub fn with_cdr_repository(mut self, cdr_repository: Arc<dyn CdrRepository>) -> Self {
        // Replace call_router with one that has CDR repository
        let new_router = CallRouter::new(self.registrar.clone())
            .with_cdr_repository(cdr_repository);
        self.call_router = Arc::new(new_router);
        self
    }

    /// Get call router reference
    pub fn call_router(&self) -> Arc<CallRouter> {
        self.call_router.clone()
    }

    /// Allocate RTP port pair (RTP + RTCP)
    async fn allocate_rtp_port(&self) -> u16 {
        let mut port = self.next_rtp_port.write().await;
        let allocated = *port;
        *port = port.wrapping_add(2); // Allocate RTP and RTCP
        if *port < 10000 {
            *port = 10000; // Wrap around
        }
        allocated
    }

    async fn handle_invite(&self, request: &SipRequest) -> Result<SipResponse, SipError> {
        info!("Handling INVITE request");

        // Check authentication if enabled
        if let Some(auth) = &self.auth {
            // Check if Authorization header is present
            let has_auth = request.headers().iter().any(|h| {
                matches!(h, Header::Authorization(_) | Header::ProxyAuthorization(_))
            });

            if !has_auth {
                // Send 407 Proxy Authentication Required with challenge
                warn!("INVITE without authentication - sending challenge");
                let challenge = auth.create_challenge().await;

                return ResponseBuilder::new(407)
                    .header(Header::Other(
                        "Proxy-Authenticate".to_string(),
                        challenge.to_header_value(),
                    ))
                    .build_for_request(request);
            }

            // Verify authentication
            match auth.verify_request(request, "INVITE").await {
                Ok(username) => {
                    info!("INVITE authenticated for user: {}", username);
                }
                Err(e) => {
                    warn!("Authentication failed: {:?}", e);
                    // Send 407 with new challenge
                    let challenge = auth.create_challenge().await;

                    return ResponseBuilder::new(407)
                        .header(Header::Other(
                            "Proxy-Authenticate".to_string(),
                            challenge.to_header_value(),
                        ))
                        .build_for_request(request);
                }
            }
        }

        // Extract call information
        let call_id = request.call_id().unwrap_or_else(|| "unknown".to_string());
        let from_uri = self.extract_from_uri(request);
        let to_uri = self.extract_to_uri(request);

        debug!("Call: {} -> {}", from_uri, to_uri);

        // Check if this is a re-INVITE (call already exists)
        if let Some(_call_state) = self.call_router.get_call_state(&call_id).await {
            info!("Detected re-INVITE for existing call {}", call_id);
            return self.handle_reinvite(request, &call_id).await;
        }

        // Check if callee is registered
        let callee_available = self.call_router.is_callee_available(&to_uri).await;

        if !callee_available {
            warn!("Callee {} not found or not registered", to_uri);
            return ResponseBuilder::new(404)
                .build_for_request(request);
        }

        // Create call in router
        if let Err(e) = self.call_router.create_call(
            call_id.clone(),
            from_uri.clone(),
            to_uri.clone(),
        ).await {
            warn!("Failed to create call: {}", e);
            return ResponseBuilder::new(500)
                .build_for_request(request);
        }

        // Send 100 Trying immediately
        // Note: In a real implementation, we'd send this as a separate response
        // For now, we'll just log it
        debug!("Would send 100 Trying for call {}", call_id);

        // If not in auto-answer mode, send 180 Ringing and wait for actual answer
        if !self.auto_answer {
            info!("Call {} ringing (forward mode)", call_id);
            // In a real implementation, forward INVITE to callee here
            // and return 180 Ringing
            return self.call_router.send_ringing(&call_id, request).await;
        }

        // Parse SDP offer from request body
        let sdp_offer = {
            let body = request.body();
            if !body.is_empty() {
                let body_str = String::from_utf8_lossy(body);
                SdpSession::parse(&body_str)
            } else {
                None
            }
        };

        // Negotiate codecs if we have an SDP offer
        let (chosen_codec, local_port) = if let Some(offer) = sdp_offer {
            let offered_codecs = offer.audio_codecs();
            info!("Offered codecs: {:?}", offered_codecs);

            let negotiated = self.codec_negotiator.negotiate(&offered_codecs);
            if negotiated.is_empty() {
                warn!("No common codecs found");
                return ResponseBuilder::new(488) // Not Acceptable Here
                    .build_for_request(request);
            }

            let chosen = negotiated[0].clone();
            info!("Chosen codec: {} (PT {})", chosen.name, chosen.payload_type);

            // Allocate RTP port for this call
            let port = self.allocate_rtp_port().await;
            (Some(chosen), port)
        } else {
            // No SDP offer, use default
            let port = self.allocate_rtp_port().await;
            (None, port)
        };

        // Create media streams (simplified - both legs using same local stream for auto-answer)
        // In real implementation, you would create separate streams for caller and callee
        let media_stream = match MediaStream::new(
            local_port,
            chosen_codec.as_ref().map(|c| c.payload_type).unwrap_or(0),
            8000,
        ).await {
            Ok(stream) => Arc::new(stream),
            Err(e) => {
                warn!("Failed to create media stream: {}", e);
                return ResponseBuilder::new(500)
                    .build_for_request(request);
            }
        };

        // Start media stream
        if let Err(e) = media_stream.start().await {
            warn!("Failed to start media stream: {}", e);
        }

        // Set stream direction
        media_stream.set_direction(StreamDirection::SendRecv).await;

        // For auto-answer mode, create a simple bridge (in real implementation, you'd connect two different streams)
        let media_bridge = Arc::new(MediaBridge::new(media_stream.clone(), media_stream.clone()));

        // Create call session
        let session = CallSession {
            call_id: call_id.clone(),
            from_uri: from_uri.clone(),
            to_uri: to_uri.clone(),
            state: CallSessionState::Inviting,
            media_bridge: Some(media_bridge.clone()),
        };

        {
            let mut calls = self.active_calls.write().await;
            calls.insert(call_id.clone(), session);
        }

        // Auto-answer mode
        info!("Auto-answering call {}", call_id);

        // Answer call in router
        if let Err(e) = self.call_router.answer_call(&call_id).await {
            warn!("Failed to answer call in router: {}", e);
        }

        // Update legacy call state
        {
            let mut calls = self.active_calls.write().await;
            if let Some(call) = calls.get_mut(&call_id) {
                call.state = CallSessionState::Answered;
            }
        }

        // Create SDP answer with negotiated codec
        let sdp = SdpSession::create_audio_session(self.local_ip, local_port);
        let sdp_body = sdp.to_string();

        // Build 200 OK response with SDP
        let response = ResponseBuilder::ok()
            .body(sdp_body.into_bytes())
            .build_for_request(request)?;

        info!("Sent 200 OK for call {}", call_id);

        Ok(response)
    }

    /// Handle re-INVITE for session modification (hold/resume)
    async fn handle_reinvite(&self, request: &SipRequest, call_id: &str) -> Result<SipResponse, SipError> {
        info!("Handling re-INVITE for call {}", call_id);

        // Parse SDP from request body
        let sdp_offer = {
            let body = request.body();
            if !body.is_empty() {
                let body_str = String::from_utf8_lossy(body);
                SdpSession::parse(&body_str)
            } else {
                None
            }
        };

        if let Some(offer) = sdp_offer {
            let sdp_str = String::from_utf8_lossy(request.body());

            // Detect hold state from SDP
            let hold_state = SdpHoldHelper::detect_hold_state(&sdp_str);

            info!("Detected hold state: {:?} for call {}", hold_state, call_id);

            // Update hold state in call router
            use super::hold_manager::HoldState;
            match hold_state {
                HoldState::Active => {
                    // Remote party is resuming from hold
                    if let Err(e) = self.call_router.remote_resume(call_id).await {
                        warn!("Failed to resume call {}: {}", call_id, e);
                    }
                }
                HoldState::RemoteHold | HoldState::LocalHold => {
                    // Remote party is placing us on hold (sendonly from their perspective)
                    if let Err(e) = self.call_router.remote_hold(call_id).await {
                        warn!("Failed to hold call {}: {}", call_id, e);
                    }
                }
                HoldState::BothHold => {
                    // Both parties on hold (inactive)
                    if let Err(e) = self.call_router.remote_hold(call_id).await {
                        warn!("Failed to hold call {}: {}", call_id, e);
                    }
                }
            }

            // Create SDP answer
            // For now, we'll mirror the hold state back
            // In a real implementation, you'd use the actual local media parameters
            let media_port = offer.audio_media().map(|m| m.port).unwrap_or(10000);
            let sdp = SdpSession::create_audio_session(
                self.local_ip,
                media_port,
            );
            let mut sdp_body = sdp.to_string();

            // Apply hold state to answer SDP
            match hold_state {
                HoldState::Active => {
                    // Remote is active, we're active (sendrecv)
                    sdp_body = SdpHoldHelper::create_resume_sdp(&sdp_body);
                }
                HoldState::RemoteHold | HoldState::LocalHold => {
                    // Remote is on hold, we send recvonly
                    // (we're receiving only, they're sending music on hold)
                    // Note: This is the opposite of their sendonly
                    sdp_body = sdp_body.replace("a=sendrecv", "a=recvonly");
                    if !sdp_body.contains("a=recvonly") {
                        sdp_body.push_str("a=recvonly\r\n");
                    }
                }
                HoldState::BothHold => {
                    // Both on hold
                    sdp_body = SdpHoldHelper::create_hold_sdp(&sdp_body, true);
                }
            }

            // Build 200 OK response with SDP
            let response = ResponseBuilder::ok()
                .body(sdp_body.into_bytes())
                .build_for_request(request)?;

            info!("Sent 200 OK for re-INVITE (call {})", call_id);
            Ok(response)
        } else {
            // No SDP in re-INVITE, just return 200 OK
            warn!("re-INVITE without SDP for call {}", call_id);
            ResponseBuilder::ok().build_for_request(request)
        }
    }

    fn extract_from_uri(&self, request: &SipRequest) -> String {
        request
            .headers()
            .iter()
            .find_map(|h| match h {
                Header::From(from) => from.uri().ok().map(|u| u.to_string()),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn extract_to_uri(&self, request: &SipRequest) -> String {
        request
            .headers()
            .iter()
            .find_map(|h| match h {
                Header::To(to) => to.uri().ok().map(|u| u.to_string()),
                _ => None,
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl SipHandler for InviteHandler {
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError> {
        self.handle_invite(&request).await
    }

    fn can_handle(&self, method: SipMethod) -> bool {
        matches!(method, SipMethod::Invite)
    }
}

/// ACK handler
pub struct AckHandler {
    active_calls: Arc<RwLock<HashMap<String, CallSession>>>,
}

impl AckHandler {
    pub fn new(active_calls: Arc<RwLock<HashMap<String, CallSession>>>) -> Self {
        Self { active_calls }
    }
}

#[async_trait]
impl SipHandler for AckHandler {
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError> {
        let call_id = request.call_id().unwrap_or_else(|| "unknown".to_string());
        info!("Received ACK for call {}", call_id);

        // ACK doesn't need a response (it's a response itself)
        // Just log it
        let calls = self.active_calls.read().await;
        if let Some(call) = calls.get(&call_id) {
            info!("Call {} confirmed: {} -> {}", call_id, call.from_uri, call.to_uri);
        }

        // Return a dummy response (won't be sent)
        ResponseBuilder::ok().build_for_request(&request)
    }

    fn can_handle(&self, method: SipMethod) -> bool {
        matches!(method, SipMethod::Ack)
    }
}

/// CANCEL handler
pub struct CancelHandler {
    active_calls: Arc<RwLock<HashMap<String, CallSession>>>,
    call_router: Arc<CallRouter>,
}

impl CancelHandler {
    pub fn new(
        active_calls: Arc<RwLock<HashMap<String, CallSession>>>,
        call_router: Arc<CallRouter>,
    ) -> Self {
        Self {
            active_calls,
            call_router,
        }
    }
}

#[async_trait]
impl SipHandler for CancelHandler {
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError> {
        let call_id = request.call_id().unwrap_or_else(|| "unknown".to_string());
        info!("Received CANCEL for call {}", call_id);

        // Try to cancel the call in the router
        match self.call_router.cancel_call(&call_id).await {
            Ok(true) => {
                info!("Call {} cancelled successfully", call_id);

                // Remove call from active calls
                {
                    let mut calls = self.active_calls.write().await;
                    if let Some(call) = calls.remove(&call_id) {
                        // Stop media bridge if it exists
                        if let Some(bridge) = call.media_bridge {
                            bridge.stop().await;
                            debug!("Media bridge stopped for cancelled call {}", call_id);
                        }
                    }
                }

                // Return 200 OK to CANCEL request
                ResponseBuilder::ok().build_for_request(&request)
            }
            Ok(false) => {
                // Call cannot be cancelled (already established or terminated)
                warn!("Call {} cannot be cancelled", call_id);
                // Return 481 Call/Transaction Does Not Exist
                ResponseBuilder::new(481).build_for_request(&request)
            }
            Err(e) => {
                warn!("Failed to cancel call {}: {}", call_id, e);
                // Return 481 Call/Transaction Does Not Exist
                ResponseBuilder::new(481).build_for_request(&request)
            }
        }
    }

    fn can_handle(&self, method: SipMethod) -> bool {
        matches!(method, SipMethod::Cancel)
    }
}

/// BYE handler
pub struct ByeHandler {
    active_calls: Arc<RwLock<HashMap<String, CallSession>>>,
    call_router: Option<Arc<CallRouter>>,
}

impl ByeHandler {
    pub fn new(active_calls: Arc<RwLock<HashMap<String, CallSession>>>) -> Self {
        Self {
            active_calls,
            call_router: None,
        }
    }

    /// Create BYE handler with call router
    pub fn with_router(active_calls: Arc<RwLock<HashMap<String, CallSession>>>, call_router: Arc<CallRouter>) -> Self {
        Self {
            active_calls,
            call_router: Some(call_router),
        }
    }
}

#[async_trait]
impl SipHandler for ByeHandler {
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError> {
        let call_id = request.call_id().unwrap_or_else(|| "unknown".to_string());
        info!("Received BYE for call {}", call_id);

        // Terminate call in router
        if let Some(router) = &self.call_router {
            if let Err(e) = router.terminate_call(&call_id).await {
                warn!("Failed to terminate call in router: {}", e);
            }
        }

        // Remove call from active calls and stop media
        {
            let mut calls = self.active_calls.write().await;
            if let Some(call) = calls.remove(&call_id) {
                info!("Call {} terminated: {} -> {}", call_id, call.from_uri, call.to_uri);

                // Stop media bridge
                if let Some(bridge) = call.media_bridge {
                    bridge.stop().await;
                    info!("Media bridge stopped for call {}", call_id);
                }
            }
        }

        // Return 200 OK
        ResponseBuilder::ok().build_for_request(&request)
    }

    fn can_handle(&self, method: SipMethod) -> bool {
        matches!(method, SipMethod::Bye)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_call_session_state() {
        let state = CallSessionState::Inviting;
        assert_eq!(state, CallSessionState::Inviting);

        let state2 = CallSessionState::Answered;
        assert_ne!(state, state2);
    }

    #[tokio::test]
    async fn test_call_forwarding_integration() {
        // Setup
        let registrar = Arc::new(Registrar::new());
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Register caller and callee
        registrar.add_binding(
            "sip:alice@example.com".to_string(),
            "127.0.0.1:5060".to_string(),
            3600,
        ).await.unwrap();

        registrar.add_binding(
            "sip:bob@example.com".to_string(),
            "127.0.0.1:5061".to_string(),
            3600,
        ).await.unwrap();

        // Create InviteHandler with CallRouter
        let invite_handler = InviteHandler::new(registrar.clone(), local_ip);
        let call_router = invite_handler.call_router();

        // Create INVITE request from alice to bob
        let invite_request = "INVITE sip:bob@example.com SIP/2.0\r\n\
            Via: SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK776asdhds\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: a84b4c76e66710\r\n\
            CSeq: 314159 INVITE\r\n\
            Contact: <sip:alice@127.0.0.1:5060>\r\n\
            Content-Type: application/sdp\r\n\
            Content-Length: 142\r\n\
            \r\n\
            v=0\r\n\
            o=alice 2890844526 2890844526 IN IP4 127.0.0.1\r\n\
            s=-\r\n\
            c=IN IP4 127.0.0.1\r\n\
            t=0 0\r\n\
            m=audio 49170 RTP/AVP 0 8\r\n\
            a=rtpmap:0 PCMU/8000\r\n\
            a=rtpmap:8 PCMA/8000\r\n";

        let request = SipRequest::parse(invite_request.as_bytes()).unwrap();

        // Handle INVITE
        let response = invite_handler.handle_request(request.clone()).await.unwrap();

        // Verify response is 200 OK (auto-answer mode)
        assert_eq!(response.status_code(), 200);

        // Verify call was created in router
        let call_count = call_router.active_call_count().await;
        assert_eq!(call_count, 1);

        // Verify call state
        let call_state = call_router.get_call_state("a84b4c76e66710").await;
        assert_eq!(call_state, Some(super::super::call_state::CallState::Established));

        // Verify call session exists
        let active_calls = invite_handler.active_calls.read().await;
        let call_session = active_calls.get("a84b4c76e66710");
        assert!(call_session.is_some());
        assert_eq!(call_session.unwrap().state, CallSessionState::Answered);
        drop(active_calls);

        // Create BYE handler
        let bye_handler = ByeHandler::with_router(
            invite_handler.active_calls.clone(),
            call_router.clone(),
        );

        // Create BYE request
        let bye_request = "BYE sip:bob@example.com SIP/2.0\r\n\
            Via: SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK776asdhds\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: a84b4c76e66710\r\n\
            CSeq: 314160 BYE\r\n\
            \r\n";

        let bye_req = SipRequest::parse(bye_request.as_bytes()).unwrap();

        // Handle BYE
        let bye_response = bye_handler.handle_request(bye_req).await.unwrap();

        // Verify response is 200 OK
        assert_eq!(bye_response.status_code(), 200);

        // Verify call was removed
        let call_count_after = call_router.active_call_count().await;
        assert_eq!(call_count_after, 0);
    }

    #[tokio::test]
    async fn test_call_forwarding_not_found() {
        // Setup
        let registrar = Arc::new(Registrar::new());
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Register only caller (bob not registered)
        registrar.add_binding(
            "sip:alice@example.com".to_string(),
            "127.0.0.1:5060".to_string(),
            3600,
        ).await.unwrap();

        // Create InviteHandler
        let invite_handler = InviteHandler::new(registrar.clone(), local_ip);

        // Create INVITE request to unregistered user
        let invite_request = "INVITE sip:bob@example.com SIP/2.0\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-not-found\r\n\
            CSeq: 1 INVITE\r\n\
            \r\n";

        let request = SipRequest::parse(invite_request.as_bytes()).unwrap();

        // Handle INVITE
        let response = invite_handler.handle_request(request).await.unwrap();

        // Verify response is 404 Not Found
        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    async fn test_call_forwarding_with_ringing() {
        // Setup
        let registrar = Arc::new(Registrar::new());
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Register caller and callee
        registrar.add_binding(
            "sip:alice@example.com".to_string(),
            "127.0.0.1:5060".to_string(),
            3600,
        ).await.unwrap();

        registrar.add_binding(
            "sip:bob@example.com".to_string(),
            "127.0.0.1:5061".to_string(),
            3600,
        ).await.unwrap();

        // Create InviteHandler with auto-answer disabled
        let mut invite_handler = InviteHandler::new(registrar.clone(), local_ip);
        invite_handler.set_auto_answer(false);

        // Create INVITE request
        let invite_request = "INVITE sip:bob@example.com SIP/2.0\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-ringing\r\n\
            CSeq: 1 INVITE\r\n\
            \r\n";

        let request = SipRequest::parse(invite_request.as_bytes()).unwrap();

        // Handle INVITE
        let response = invite_handler.handle_request(request).await.unwrap();

        // Verify response is 180 Ringing
        assert_eq!(response.status_code(), 180);

        // Verify call state is Ringing
        let call_router = invite_handler.call_router();
        let call_state = call_router.get_call_state("test-ringing").await;
        assert_eq!(call_state, Some(super::super::call_state::CallState::Ringing));
    }

    #[tokio::test]
    async fn test_cancel_ringing_call() {
        // Setup
        let registrar = Arc::new(Registrar::new());
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Register caller and callee
        registrar.add_binding(
            "sip:alice@example.com".to_string(),
            "127.0.0.1:5060".to_string(),
            3600,
        ).await.unwrap();

        registrar.add_binding(
            "sip:bob@example.com".to_string(),
            "127.0.0.1:5061".to_string(),
            3600,
        ).await.unwrap();

        // Create InviteHandler with auto-answer disabled
        let mut invite_handler = InviteHandler::new(registrar.clone(), local_ip);
        invite_handler.set_auto_answer(false);

        // Create INVITE request
        let invite_request = "INVITE sip:bob@example.com SIP/2.0\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-cancel\r\n\
            CSeq: 1 INVITE\r\n\
            \r\n";

        let request = SipRequest::parse(invite_request.as_bytes()).unwrap();

        // Handle INVITE
        let response = invite_handler.handle_request(request).await.unwrap();
        assert_eq!(response.status_code(), 180);

        // Create CancelHandler
        let cancel_handler = CancelHandler::new(
            invite_handler.active_calls.clone(),
            invite_handler.call_router(),
        );

        // Create CANCEL request
        let cancel_request = "CANCEL sip:bob@example.com SIP/2.0\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-cancel\r\n\
            CSeq: 1 CANCEL\r\n\
            \r\n";

        let cancel_req = SipRequest::parse(cancel_request.as_bytes()).unwrap();

        // Handle CANCEL
        let cancel_response = cancel_handler.handle_request(cancel_req).await.unwrap();

        // Verify response is 200 OK
        assert_eq!(cancel_response.status_code(), 200);

        // Verify call state is Failed
        let call_router = invite_handler.call_router();
        let call_state = call_router.get_call_state("test-cancel").await;
        assert_eq!(call_state, Some(super::super::call_state::CallState::Failed));

        // Verify call was removed from active calls
        let active_calls = invite_handler.active_calls.read().await;
        assert!(active_calls.get("test-cancel").is_none());
    }

    #[tokio::test]
    async fn test_cancel_established_call_fails() {
        // Setup
        let registrar = Arc::new(Registrar::new());
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Register caller and callee
        registrar.add_binding(
            "sip:alice@example.com".to_string(),
            "127.0.0.1:5060".to_string(),
            3600,
        ).await.unwrap();

        registrar.add_binding(
            "sip:bob@example.com".to_string(),
            "127.0.0.1:5061".to_string(),
            3600,
        ).await.unwrap();

        // Create InviteHandler with auto-answer enabled
        let invite_handler = InviteHandler::new(registrar.clone(), local_ip);

        // Create INVITE request with SDP (using same format as successful test)
        let invite_request = "INVITE sip:bob@example.com SIP/2.0\r\n\
            Via: SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK776asdhds\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-cancel-established\r\n\
            CSeq: 1 INVITE\r\n\
            Contact: <sip:alice@127.0.0.1:5060>\r\n\
            Content-Type: application/sdp\r\n\
            Content-Length: 142\r\n\
            \r\n\
            v=0\r\n\
            o=alice 2890844526 2890844526 IN IP4 127.0.0.1\r\n\
            s=-\r\n\
            c=IN IP4 127.0.0.1\r\n\
            t=0 0\r\n\
            m=audio 49170 RTP/AVP 0 8\r\n\
            a=rtpmap:0 PCMU/8000\r\n\
            a=rtpmap:8 PCMA/8000\r\n";

        let request = SipRequest::parse(invite_request.as_bytes()).unwrap();

        // Handle INVITE (auto-answer)
        let response = invite_handler.handle_request(request).await.unwrap();
        assert_eq!(response.status_code(), 200); // Auto-answered

        // Create CancelHandler
        let cancel_handler = CancelHandler::new(
            invite_handler.active_calls.clone(),
            invite_handler.call_router(),
        );

        // Create CANCEL request
        let cancel_request = "CANCEL sip:bob@example.com SIP/2.0\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-cancel-established\r\n\
            CSeq: 1 CANCEL\r\n\
            \r\n";

        let cancel_req = SipRequest::parse(cancel_request.as_bytes()).unwrap();

        // Handle CANCEL (should fail because call is established)
        let cancel_response = cancel_handler.handle_request(cancel_req).await.unwrap();

        // Verify response is 481 Call/Transaction Does Not Exist
        assert_eq!(cancel_response.status_code(), 481);

        // Verify call state is still Established
        let call_router = invite_handler.call_router();
        let call_state = call_router.get_call_state("test-cancel-established").await;
        assert_eq!(call_state, Some(super::super::call_state::CallState::Established));
    }
}

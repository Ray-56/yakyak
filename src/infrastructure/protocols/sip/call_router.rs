//! Call Router
//!
//! Handles call routing and forwarding logic

use super::builder::ResponseBuilder;
use super::call_state::{CallEvent, CallState, CallStateMachine};
use super::hold_manager::HoldManager;
use super::message::{SipError, SipRequest, SipResponse};
use super::registrar::Registrar;
use crate::domain::cdr::{CallDetailRecord, CallDirection, CallStatus, CdrRepository};
use crate::infrastructure::media::{MediaBridge, MediaStream, MohPlayer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Active Call Information (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCallInfo {
    pub call_id: String,
    pub caller_uri: String,
    pub callee_uri: String,
    pub state: String,
    pub duration: i64,
    pub caller_contact: Option<String>,
    pub callee_contact: Option<String>,
    pub on_hold: bool,
}

/// Call Leg Information
pub struct CallLegInfo {
    pub uri: String,
    pub contact: Option<SocketAddr>,
    pub media_stream: Option<Arc<MediaStream>>,
}

/// Bridged Call
///
/// Represents a call with two legs (caller and callee)
pub struct BridgedCall {
    pub call_id: String,
    pub caller: CallLegInfo,
    pub callee: CallLegInfo,
    pub state_machine: CallStateMachine,
    pub media_bridge: Option<Arc<MediaBridge>>,
    pub cdr_id: Uuid,
}

impl BridgedCall {
    pub fn new(call_id: String, caller_uri: String, callee_uri: String, cdr_id: Uuid) -> Self {
        Self {
            call_id,
            caller: CallLegInfo {
                uri: caller_uri,
                contact: None,
                media_stream: None,
            },
            callee: CallLegInfo {
                uri: callee_uri,
                contact: None,
                media_stream: None,
            },
            state_machine: CallStateMachine::new(),
            media_bridge: None,
            cdr_id,
        }
    }

    pub fn state(&self) -> &CallState {
        self.state_machine.state()
    }

    pub fn process_event(&mut self, event: CallEvent) -> Result<(), String> {
        self.state_machine.process_event(event)
    }
}

/// Call Router
///
/// Routes calls between caller and callee
pub struct CallRouter {
    registrar: Arc<Registrar>,
    active_calls: Arc<RwLock<HashMap<String, BridgedCall>>>,
    cdr_repository: Option<Arc<dyn CdrRepository>>,
    hold_manager: Arc<HoldManager>,
    moh_players: Arc<RwLock<HashMap<String, Arc<MohPlayer>>>>,
}

impl CallRouter {
    pub fn new(registrar: Arc<Registrar>) -> Self {
        Self {
            registrar,
            active_calls: Arc::new(RwLock::new(HashMap::new())),
            cdr_repository: None,
            hold_manager: Arc::new(HoldManager::new()),
            moh_players: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_cdr_repository(mut self, cdr_repository: Arc<dyn CdrRepository>) -> Self {
        self.cdr_repository = Some(cdr_repository);
        self
    }

    /// Extract username from SIP URI
    /// Example: "sip:alice@example.com" -> "alice"
    fn extract_username(uri: &str) -> String {
        uri.trim_start_matches("sip:")
            .trim_start_matches("sips:")
            .split('@')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract IP from SocketAddr
    fn socket_to_ip(socket: &SocketAddr) -> String {
        socket.ip().to_string()
    }

    /// Create a new call
    pub async fn create_call(
        &self,
        call_id: String,
        caller_uri: String,
        callee_uri: String,
    ) -> Result<(), String> {
        // Create CDR if repository is available
        let cdr_id = if let Some(ref cdr_repo) = self.cdr_repository {
            let caller_username = Self::extract_username(&caller_uri);
            let callee_username = Self::extract_username(&callee_uri);

            // Create initial CDR (we don't have IPs yet at this point)
            let cdr = CallDetailRecord::new(
                call_id.clone(),
                caller_username,
                caller_uri.clone(),
                "0.0.0.0".to_string(), // Will be updated when we get the contact
                callee_username,
                callee_uri.clone(),
                CallDirection::Outbound, // Default, should be determined by context
            );

            let cdr_id = cdr.id;

            if let Err(e) = cdr_repo.create(&cdr).await {
                error!("Failed to create CDR for call {}: {}", call_id, e);
            } else {
                debug!("Created CDR {} for call {}", cdr_id, call_id);
            }

            cdr_id
        } else {
            // No CDR repository, use a default UUID
            Uuid::new_v4()
        };

        let call = BridgedCall::new(call_id.clone(), caller_uri, callee_uri, cdr_id);

        let mut calls = self.active_calls.write().await;
        calls.insert(call_id, call);

        Ok(())
    }

    /// Find callee contact
    pub async fn find_callee_contact(&self, callee_uri: &str) -> Option<SocketAddr> {
        // Look up callee in registrar
        if let Some(bindings) = self.registrar.get_bindings(callee_uri).await {
            if let Some(binding) = bindings.first() {
                // Parse contact string to SocketAddr
                if let Ok(addr) = binding.contact.parse::<SocketAddr>() {
                    return Some(addr);
                }
            }
        }
        None
    }

    /// Generate provisional response (100 Trying)
    pub async fn send_trying(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Some(call) = self.active_calls.write().await.get_mut(call_id) {
            if let Err(e) = call.process_event(CallEvent::Trying) {
                warn!("State transition error: {}", e);
            }
        }

        ResponseBuilder::new(100)
            .build_for_request(request)
    }

    /// Generate provisional response (180 Ringing)
    pub async fn send_ringing(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Some(call) = self.active_calls.write().await.get_mut(call_id) {
            if let Err(e) = call.process_event(CallEvent::Ringing) {
                warn!("State transition error: {}", e);
            }
        }

        ResponseBuilder::new(180)
            .build_for_request(request)
    }

    /// Generate provisional response (183 Session Progress)
    pub async fn send_session_progress(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Some(call) = self.active_calls.write().await.get_mut(call_id) {
            if let Err(e) = call.process_event(CallEvent::SessionProgress) {
                warn!("State transition error: {}", e);
            }
        }

        ResponseBuilder::new(183)
            .build_for_request(request)
    }

    /// Answer call
    pub async fn answer_call(&self, call_id: &str) -> Result<(), String> {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.process_event(CallEvent::Answer)?;
            info!("Call {} answered", call_id);

            // Update CDR with answer time
            if let Some(ref cdr_repo) = self.cdr_repository {
                if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                    cdr.mark_answered();
                    if let Err(e) = cdr_repo.update(&cdr).await {
                        error!("Failed to update CDR on answer: {}", e);
                    }
                }
            }

            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Reject call
    pub async fn reject_call(&self, call_id: &str, reason: &str) -> Result<(), String> {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.process_event(CallEvent::Reject)?;
            info!("Call {} rejected: {}", call_id, reason);

            // Update CDR with rejection
            if let Some(ref cdr_repo) = self.cdr_repository {
                if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                    // Determine status based on reason
                    let status = match reason.to_lowercase().as_str() {
                        "busy" => CallStatus::Busy,
                        "declined" | "not found" => CallStatus::Rejected,
                        _ => CallStatus::Failed,
                    };
                    cdr.mark_ended(status, Some(reason.to_string()), None);
                    if let Err(e) = cdr_repo.update(&cdr).await {
                        error!("Failed to update CDR on reject: {}", e);
                    }
                }
            }

            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Generate 486 Busy Here response
    pub async fn send_busy(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Err(e) = self.reject_call(call_id, "Busy").await {
            warn!("Failed to reject call: {}", e);
        }

        ResponseBuilder::new(486)
            .build_for_request(request)
    }

    /// Generate 603 Decline response
    pub async fn send_decline(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Err(e) = self.reject_call(call_id, "Declined").await {
            warn!("Failed to reject call: {}", e);
        }

        ResponseBuilder::new(603)
            .build_for_request(request)
    }

    /// Generate 404 Not Found response
    pub async fn send_not_found(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Update call state
        if let Err(e) = self.reject_call(call_id, "Not Found").await {
            warn!("Failed to reject call: {}", e);
        }

        ResponseBuilder::new(404)
            .build_for_request(request)
    }

    /// Terminate call
    pub async fn terminate_call(&self, call_id: &str) -> Result<(), String> {
        let mut calls = self.active_calls.write().await;
        if let Some(mut call) = calls.remove(call_id) {
            call.process_event(CallEvent::Bye)?;

            // Update CDR with completion
            if let Some(ref cdr_repo) = self.cdr_repository {
                if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                    cdr.mark_ended(CallStatus::Completed, Some("Normal clearing".to_string()), Some(200));

                    // TODO: Add media stats when MediaBridge provides stats API
                    // For now, we'll just mark the call as completed without media stats

                    if let Err(e) = cdr_repo.update(&cdr).await {
                        error!("Failed to update CDR on termination: {}", e);
                    }
                }
            }

            // Stop media
            if let Some(bridge) = call.media_bridge {
                bridge.stop().await;
                debug!("Media bridge stopped for call {}", call_id);
            }

            // Stop and cleanup MOH if playing
            {
                let mut moh_players = self.moh_players.write().await;
                if let Some(moh_player) = moh_players.remove(call_id) {
                    let _ = moh_player.stop().await;
                    debug!("MOH stopped and removed for call {}", call_id);
                }
            }

            // Clean up hold state
            self.hold_manager.remove_call(call_id).await;

            info!("Call {} terminated", call_id);
            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Set media bridge for call
    pub async fn set_media_bridge(&self, call_id: &str, bridge: Arc<MediaBridge>) {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.media_bridge = Some(bridge);
        }
    }

    /// Get call state
    pub async fn get_call_state(&self, call_id: &str) -> Option<CallState> {
        let calls = self.active_calls.read().await;
        calls.get(call_id).map(|call| call.state().clone())
    }

    /// Get active call count
    pub async fn active_call_count(&self) -> usize {
        self.active_calls.read().await.len()
    }

    /// Get all active calls
    pub async fn get_active_calls(&self) -> Vec<ActiveCallInfo> {
        let calls = self.active_calls.read().await;
        let mut result = Vec::new();

        for call in calls.values() {
            let stats = call.state_machine.stats();
            let duration = stats.ended_at
                .unwrap_or_else(|| std::time::Instant::now())
                .duration_since(stats.created_at)
                .as_secs() as i64;

            let on_hold = self.hold_manager.is_on_hold(&call.call_id).await;

            result.push(ActiveCallInfo {
                call_id: call.call_id.clone(),
                caller_uri: call.caller.uri.clone(),
                callee_uri: call.callee.uri.clone(),
                state: format!("{:?}", call.state()),
                duration,
                caller_contact: call.caller.contact.map(|c| c.to_string()),
                callee_contact: call.callee.contact.map(|c| c.to_string()),
                on_hold,
            });
        }

        result
    }

    /// Get active call by ID
    pub async fn get_active_call(&self, call_id: &str) -> Option<ActiveCallInfo> {
        let calls = self.active_calls.read().await;
        if let Some(call) = calls.get(call_id) {
            let stats = call.state_machine.stats();
            let duration = stats.ended_at
                .unwrap_or_else(|| std::time::Instant::now())
                .duration_since(stats.created_at)
                .as_secs() as i64;

            let on_hold = self.hold_manager.is_on_hold(call_id).await;

            Some(ActiveCallInfo {
                call_id: call.call_id.clone(),
                caller_uri: call.caller.uri.clone(),
                callee_uri: call.callee.uri.clone(),
                state: format!("{:?}", call.state()),
                duration,
                caller_contact: call.caller.contact.map(|c| c.to_string()),
                callee_contact: call.callee.contact.map(|c| c.to_string()),
                on_hold,
            })
        } else {
            None
        }
    }

    /// Force hangup a call (for admin/management use)
    pub async fn hangup_call(&self, call_id: &str) -> Result<(), String> {
        self.terminate_call(call_id).await
    }

    /// Check if callee is available
    pub async fn is_callee_available(&self, callee_uri: &str) -> bool {
        self.registrar.get_bindings(callee_uri).await.is_some()
    }

    /// Store caller contact for call
    pub async fn set_caller_contact(&self, call_id: &str, contact: SocketAddr) {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.caller.contact = Some(contact);
            debug!("Set caller contact for call {}: {}", call_id, contact);

            // Update CDR with caller IP
            if let Some(ref cdr_repo) = self.cdr_repository {
                if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                    cdr.caller_ip = Self::socket_to_ip(&contact);
                    if let Err(e) = cdr_repo.update(&cdr).await {
                        error!("Failed to update CDR with caller IP: {}", e);
                    }
                }
            }
        }
    }

    /// Store callee contact for call
    pub async fn set_callee_contact(&self, call_id: &str, contact: SocketAddr) {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.callee.contact = Some(contact);
            debug!("Set callee contact for call {}: {}", call_id, contact);

            // Update CDR with callee IP
            if let Some(ref cdr_repo) = self.cdr_repository {
                if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                    cdr.set_callee_ip(Self::socket_to_ip(&contact));
                    if let Err(e) = cdr_repo.update(&cdr).await {
                        error!("Failed to update CDR with callee IP: {}", e);
                    }
                }
            }
        }
    }

    /// Get caller contact for forwarding responses
    pub async fn get_caller_contact(&self, call_id: &str) -> Option<SocketAddr> {
        let calls = self.active_calls.read().await;
        calls.get(call_id).and_then(|call| call.caller.contact)
    }

    /// Get callee contact for forwarding requests
    pub async fn get_callee_contact(&self, call_id: &str) -> Option<SocketAddr> {
        let calls = self.active_calls.read().await;
        calls.get(call_id).and_then(|call| call.callee.contact)
    }

    /// Forward provisional response to caller
    ///
    /// This method would be used to forward 100 Trying, 180 Ringing, 183 Session Progress
    /// from callee back to caller in a real forwarding scenario
    pub async fn forward_provisional_response(
        &self,
        call_id: &str,
        status_code: u16,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        debug!(
            "Forwarding provisional response {} for call {}",
            status_code, call_id
        );

        // Update state based on response code
        let result = match status_code {
            100 => self.send_trying(call_id, request).await,
            180 => self.send_ringing(call_id, request).await,
            183 => self.send_session_progress(call_id, request).await,
            _ => {
                warn!("Unknown provisional response code: {}", status_code);
                ResponseBuilder::new(status_code).build_for_request(request)
            }
        };

        result
    }

    /// Cancel call
    ///
    /// CANCEL can only cancel calls that are not yet established
    /// Returns Ok(true) if call was cancelled, Ok(false) if call cannot be cancelled
    pub async fn cancel_call(&self, call_id: &str) -> Result<bool, String> {
        let mut calls = self.active_calls.write().await;

        if let Some(call) = calls.get_mut(call_id) {
            // Check if call can be cancelled (not yet established)
            let state = call.state();

            if state.is_provisional() {
                // Process reject event to transition to Failed state
                call.process_event(CallEvent::Reject)?;
                info!("Call {} cancelled", call_id);

                // Update CDR with cancellation
                if let Some(ref cdr_repo) = self.cdr_repository {
                    if let Ok(Some(mut cdr)) = cdr_repo.get_by_id(call.cdr_id).await {
                        cdr.mark_ended(CallStatus::Cancelled, Some("Call cancelled".to_string()), Some(487));
                        if let Err(e) = cdr_repo.update(&cdr).await {
                            error!("Failed to update CDR on cancel: {}", e);
                        }
                    }
                }

                Ok(true)
            } else if state == &CallState::Established {
                // Cannot cancel an established call
                warn!("Cannot cancel established call {}", call_id);
                Ok(false)
            } else {
                // Already terminated or failed
                debug!("Call {} already terminated/failed", call_id);
                Ok(false)
            }
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Generate 487 Request Terminated response
    ///
    /// This is sent to the original INVITE when a CANCEL is processed
    pub async fn send_request_terminated(
        &self,
        call_id: &str,
        request: &SipRequest,
    ) -> Result<SipResponse, SipError> {
        // Cancel the call first
        match self.cancel_call(call_id).await {
            Ok(true) => {
                info!("Sending 487 Request Terminated for call {}", call_id);
                ResponseBuilder::new(487).build_for_request(request)
            }
            Ok(false) => {
                // Call cannot be cancelled, return 481 Call/Transaction Does Not Exist
                warn!("Call {} cannot be cancelled", call_id);
                ResponseBuilder::new(481).build_for_request(request)
            }
            Err(e) => {
                warn!("Error cancelling call: {}", e);
                ResponseBuilder::new(481).build_for_request(request)
            }
        }
    }

    /// Put call on hold (local hold)
    ///
    /// This will mark the call as on hold and update the media stream direction
    /// to sendonly (sending music on hold)
    pub async fn hold_call(&self, call_id: &str) -> Result<(), String> {
        // Check if call exists and is established
        {
            let calls = self.active_calls.read().await;
            if let Some(call) = calls.get(call_id) {
                if !call.state().is_established() {
                    return Err("Call must be established to be put on hold".to_string());
                }
            } else {
                return Err(format!("Call {} not found", call_id));
            }
        }

        // Mark call as on hold in hold manager
        self.hold_manager.hold_call(call_id).await?;

        // Start music on hold
        let moh_player = Arc::new(MohPlayer::new());
        if let Err(e) = moh_player.start().await {
            warn!("Failed to start MOH for call {}: {}", call_id, e);
        } else {
            // Store MOH player for this call
            let mut moh_players = self.moh_players.write().await;
            moh_players.insert(call_id.to_string(), moh_player);
            debug!("Started MOH for call {}", call_id);
        }

        // TODO: Update media stream direction to sendonly
        // This requires accessing the media stream and changing its direction

        info!("Call {} placed on hold with MOH", call_id);
        Ok(())
    }

    /// Resume call from hold
    ///
    /// This will mark the call as active and restore media stream direction
    /// to sendrecv
    pub async fn resume_call(&self, call_id: &str) -> Result<(), String> {
        // Check if call exists
        {
            let calls = self.active_calls.read().await;
            if !calls.contains_key(call_id) {
                return Err(format!("Call {} not found", call_id));
            }
        }

        // Resume call in hold manager
        self.hold_manager.resume_call(call_id).await?;

        // Stop music on hold
        {
            let mut moh_players = self.moh_players.write().await;
            if let Some(moh_player) = moh_players.remove(call_id) {
                if let Err(e) = moh_player.stop().await {
                    warn!("Failed to stop MOH for call {}: {}", call_id, e);
                } else {
                    debug!("Stopped MOH for call {}", call_id);
                }
            }
        }

        // TODO: Update media stream direction to sendrecv
        // This requires accessing the media stream and changing its direction

        info!("Call {} resumed from hold", call_id);
        Ok(())
    }

    /// Mark remote party as holding (detected from re-INVITE with sendonly/recvonly SDP)
    pub async fn remote_hold(&self, call_id: &str) -> Result<(), String> {
        self.hold_manager.remote_hold(call_id).await
    }

    /// Mark remote party as resuming (detected from re-INVITE with sendrecv SDP)
    pub async fn remote_resume(&self, call_id: &str) -> Result<(), String> {
        self.hold_manager.remote_resume(call_id).await
    }

    /// Get hold manager reference (for advanced use cases)
    pub fn hold_manager(&self) -> Arc<HoldManager> {
        self.hold_manager.clone()
    }

    /// Check if call is on hold
    pub async fn is_call_on_hold(&self, call_id: &str) -> bool {
        self.hold_manager.is_on_hold(call_id).await
    }

    /// Blind transfer - transfer call to another party without consultation
    ///
    /// # Arguments
    /// * `call_id` - The call to transfer
    /// * `target_uri` - The URI to transfer to (from Refer-To header)
    ///
    /// # Returns
    /// Ok(()) if transfer was initiated successfully
    pub async fn blind_transfer(&self, call_id: &str, target_uri: &str) -> Result<(), String> {
        // Check if call exists and is established
        {
            let calls = self.active_calls.read().await;
            if let Some(call) = calls.get(call_id) {
                if !call.state().is_established() {
                    return Err("Call must be established to be transferred".to_string());
                }
            } else {
                return Err(format!("Call {} not found", call_id));
            }
        }

        info!(
            "Initiating blind transfer for call {} to {}",
            call_id, target_uri
        );

        // In a real implementation, we would:
        // 1. Send NOTIFY to transferor with "SIP/2.0 100 Trying"
        // 2. Create new INVITE to target
        // 3. Wait for target response
        // 4. Send NOTIFY to transferor with final status
        // 5. Bridge original caller with target
        // 6. Terminate transferor's leg

        // For now, just mark as successful
        // TODO: Implement actual call bridging and NOTIFY sending

        info!("Blind transfer initiated for call {}", call_id);
        Ok(())
    }

    /// Attended transfer (consultative transfer) - transfer after consultation
    ///
    /// # Arguments
    /// * `call_id` - The original call to transfer
    /// * `target_uri` - The target URI (from Refer-To header)
    /// * `replaces` - The Replaces header value (identifies consultation call)
    ///
    /// # Returns
    /// Ok(()) if transfer was initiated successfully
    pub async fn attended_transfer(
        &self,
        call_id: &str,
        target_uri: &str,
        replaces: Option<&str>,
    ) -> Result<(), String> {
        // Check if call exists and is established
        {
            let calls = self.active_calls.read().await;
            if let Some(call) = calls.get(call_id) {
                if !call.state().is_established() {
                    return Err("Call must be established to be transferred".to_string());
                }
            } else {
                return Err(format!("Call {} not found", call_id));
            }
        }

        info!(
            "Initiating attended transfer for call {} to {} (replaces: {:?})",
            call_id, target_uri, replaces
        );

        // Parse Replaces header to extract call-id, to-tag, from-tag
        // Format: call-id;to-tag=xxx;from-tag=yyy
        let replaced_call_id = if let Some(replaces_value) = replaces {
            Self::parse_replaces_header(replaces_value)
        } else {
            return Err("Attended transfer requires Replaces header".to_string());
        };

        debug!(
            "Attended transfer: replacing call {} with call {}",
            replaced_call_id, call_id
        );

        // In a real implementation, we would:
        // 1. Verify the consultation call (replaced_call_id) exists
        // 2. Send NOTIFY to transferor with "SIP/2.0 100 Trying"
        // 3. Send INVITE to target with Replaces header
        // 4. Wait for target to accept
        // 5. Bridge the two calls
        // 6. Send NOTIFY to transferor with success
        // 7. Terminate transferor's legs

        // For now, just mark as successful
        // TODO: Implement actual attended transfer logic

        info!("Attended transfer initiated for call {}", call_id);
        Ok(())
    }

    /// Parse Replaces header
    /// Format: call-id;to-tag=xxx;from-tag=yyy
    fn parse_replaces_header(replaces: &str) -> String {
        // Extract call-id (everything before first semicolon)
        replaces
            .split(';')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_call_router_creation() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        assert_eq!(router.active_call_count().await, 0);
    }

    #[tokio::test]
    async fn test_create_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-123".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(router.active_call_count().await, 1);

        let state = router.get_call_state("call-123").await;
        assert_eq!(state, Some(CallState::Trying));
    }

    #[tokio::test]
    async fn test_answer_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-456".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-456").await.unwrap();

        let state = router.get_call_state("call-456").await;
        assert_eq!(state, Some(CallState::Established));
    }

    #[tokio::test]
    async fn test_terminate_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-789".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-789").await.unwrap();
        router.terminate_call("call-789").await.unwrap();

        assert_eq!(router.active_call_count().await, 0);
    }

    #[tokio::test]
    async fn test_reject_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-reject".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.reject_call("call-reject", "Busy").await.unwrap();

        let state = router.get_call_state("call-reject").await;
        assert_eq!(state, Some(CallState::Failed));
    }

    #[tokio::test]
    async fn test_send_busy() {
        use super::super::message::SipRequest;

        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-busy".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Create a minimal request for testing
        let request_str = "INVITE sip:bob@example.com SIP/2.0\r\nCall-ID: call-busy\r\nCSeq: 1 INVITE\r\n\r\n";
        let request = SipRequest::parse(request_str.as_bytes()).unwrap();

        let response = router.send_busy("call-busy", &request).await.unwrap();
        assert_eq!(response.status_code(), 486);

        let state = router.get_call_state("call-busy").await;
        assert_eq!(state, Some(CallState::Failed));
    }

    #[tokio::test]
    async fn test_send_decline() {
        use super::super::message::SipRequest;

        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-decline".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        let request_str = "INVITE sip:bob@example.com SIP/2.0\r\nCall-ID: call-decline\r\nCSeq: 1 INVITE\r\n\r\n";
        let request = SipRequest::parse(request_str.as_bytes()).unwrap();

        let response = router.send_decline("call-decline", &request).await.unwrap();
        assert_eq!(response.status_code(), 603);

        let state = router.get_call_state("call-decline").await;
        assert_eq!(state, Some(CallState::Failed));
    }

    #[tokio::test]
    async fn test_forward_provisional_response() {
        use super::super::message::SipRequest;

        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-forward".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        let request_str = "INVITE sip:bob@example.com SIP/2.0\r\nCall-ID: call-forward\r\nCSeq: 1 INVITE\r\n\r\n";
        let request = SipRequest::parse(request_str.as_bytes()).unwrap();

        // Test forwarding 100 Trying
        let response = router.forward_provisional_response("call-forward", 100, &request).await.unwrap();
        assert_eq!(response.status_code(), 100);
        assert_eq!(router.get_call_state("call-forward").await, Some(CallState::Proceeding));

        // Test forwarding 180 Ringing
        let response = router.forward_provisional_response("call-forward", 180, &request).await.unwrap();
        assert_eq!(response.status_code(), 180);
        assert_eq!(router.get_call_state("call-forward").await, Some(CallState::Ringing));

        // Test forwarding 183 Session Progress (from Ringing state)
        let response = router.forward_provisional_response("call-forward", 183, &request).await.unwrap();
        assert_eq!(response.status_code(), 183);
        // Note: State remains Ringing because Ringing->SessionProgress is not a valid transition
        // This is expected behavior - the state machine rejects invalid transitions
    }

    #[tokio::test]
    async fn test_contact_management() {
        use std::net::{IpAddr, Ipv4Addr};

        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-contact".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        let caller_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 5060);
        let callee_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)), 5060);

        router.set_caller_contact("call-contact", caller_addr).await;
        router.set_callee_contact("call-contact", callee_addr).await;

        assert_eq!(router.get_caller_contact("call-contact").await, Some(caller_addr));
        assert_eq!(router.get_callee_contact("call-contact").await, Some(callee_addr));
    }

    #[tokio::test]
    async fn test_cancel_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-cancel".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Transition to Ringing state
        router.send_ringing("call-cancel", &create_test_request("call-cancel")).await.unwrap();

        // Cancel the call
        let result = router.cancel_call("call-cancel").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Verify call state is Failed
        let state = router.get_call_state("call-cancel").await;
        assert_eq!(state, Some(CallState::Failed));
    }

    #[tokio::test]
    async fn test_cancel_established_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-established".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Answer the call
        router.answer_call("call-established").await.unwrap();

        // Try to cancel an established call (should fail)
        let result = router.cancel_call("call-established").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Cannot cancel established call

        // Verify call state is still Established
        let state = router.get_call_state("call-established").await;
        assert_eq!(state, Some(CallState::Established));
    }

    #[tokio::test]
    async fn test_send_request_terminated() {
        use super::super::message::SipRequest;

        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        router
            .create_call(
                "call-terminate".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Transition to Ringing state
        router.send_ringing("call-terminate", &create_test_request("call-terminate")).await.unwrap();

        let request_str = "INVITE sip:bob@example.com SIP/2.0\r\nCall-ID: call-terminate\r\nCSeq: 1 INVITE\r\n\r\n";
        let request = SipRequest::parse(request_str.as_bytes()).unwrap();

        // Send 487 Request Terminated
        let response = router.send_request_terminated("call-terminate", &request).await.unwrap();
        assert_eq!(response.status_code(), 487);

        // Verify call state is Failed
        let state = router.get_call_state("call-terminate").await;
        assert_eq!(state, Some(CallState::Failed));
    }

    #[tokio::test]
    async fn test_call_hold_resume() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create and answer a call
        router
            .create_call(
                "call-hold-test".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-hold-test").await.unwrap();
        assert_eq!(router.get_call_state("call-hold-test").await, Some(CallState::Established));

        // Put call on hold
        router.hold_call("call-hold-test").await.unwrap();
        assert!(router.is_call_on_hold("call-hold-test").await);

        // Try to hold again (should fail)
        assert!(router.hold_call("call-hold-test").await.is_err());

        // Resume call
        router.resume_call("call-hold-test").await.unwrap();
        assert!(!router.is_call_on_hold("call-hold-test").await);

        // Terminate call
        router.terminate_call("call-hold-test").await.unwrap();
    }

    #[tokio::test]
    async fn test_call_hold_before_established() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create call but don't answer
        router
            .create_call(
                "call-early-hold".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Try to hold call before it's established (should fail)
        let result = router.hold_call("call-early-hold").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be established"));
    }

    #[tokio::test]
    async fn test_remote_hold_resume() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create and answer a call
        router
            .create_call(
                "call-remote-hold".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-remote-hold").await.unwrap();

        // Remote party puts us on hold
        router.remote_hold("call-remote-hold").await.unwrap();
        assert!(router.is_call_on_hold("call-remote-hold").await);

        // Remote party resumes
        router.remote_resume("call-remote-hold").await.unwrap();
        assert!(!router.is_call_on_hold("call-remote-hold").await);
    }

    #[tokio::test]
    async fn test_moh_cleanup_on_terminate() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create, answer, and hold a call
        router
            .create_call(
                "call-moh-cleanup".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-moh-cleanup").await.unwrap();
        router.hold_call("call-moh-cleanup").await.unwrap();
        assert!(router.is_call_on_hold("call-moh-cleanup").await);

        // Terminate call (should cleanup MOH)
        router.terminate_call("call-moh-cleanup").await.unwrap();

        // Verify call is removed
        assert_eq!(router.get_call_state("call-moh-cleanup").await, None);
    }

    #[tokio::test]
    async fn test_blind_transfer() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create and answer a call
        router
            .create_call(
                "call-transfer".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-transfer").await.unwrap();
        assert_eq!(router.get_call_state("call-transfer").await, Some(CallState::Established));

        // Initiate blind transfer to charlie
        let result = router
            .blind_transfer("call-transfer", "sip:charlie@example.com")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_blind_transfer_before_established() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create call but don't answer
        router
            .create_call(
                "call-early-transfer".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Try to transfer before call is established (should fail)
        let result = router
            .blind_transfer("call-early-transfer", "sip:charlie@example.com")
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be established"));
    }

    #[tokio::test]
    async fn test_blind_transfer_nonexistent_call() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Try to transfer nonexistent call (should fail)
        let result = router
            .blind_transfer("nonexistent-call", "sip:charlie@example.com")
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_attended_transfer() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create and answer original call (alice -> bob)
        router
            .create_call(
                "call-original".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-original").await.unwrap();

        // Create and answer consultation call (bob -> charlie)
        router
            .create_call(
                "call-consult".to_string(),
                "sip:bob@example.com".to_string(),
                "sip:charlie@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-consult").await.unwrap();

        // Initiate attended transfer with Replaces header
        let replaces = "call-consult;to-tag=abc123;from-tag=def456";
        let result = router
            .attended_transfer("call-original", "sip:charlie@example.com", Some(replaces))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attended_transfer_without_replaces() {
        let registrar = Arc::new(Registrar::new());
        let router = CallRouter::new(registrar);

        // Create and answer a call
        router
            .create_call(
                "call-transfer-no-replaces".to_string(),
                "sip:alice@example.com".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        router.answer_call("call-transfer-no-replaces").await.unwrap();

        // Try attended transfer without Replaces header (should fail)
        let result = router
            .attended_transfer("call-transfer-no-replaces", "sip:charlie@example.com", None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Replaces header"));
    }

    #[test]
    fn test_parse_replaces_header() {
        let replaces = "call-id-123;to-tag=abc;from-tag=def";
        let call_id = CallRouter::parse_replaces_header(replaces);
        assert_eq!(call_id, "call-id-123");

        let replaces_simple = "simple-call-id";
        let call_id_simple = CallRouter::parse_replaces_header(replaces_simple);
        assert_eq!(call_id_simple, "simple-call-id");
    }

    // Helper function to create a test request
    fn create_test_request(call_id: &str) -> super::super::message::SipRequest {
        let request_str = format!("INVITE sip:bob@example.com SIP/2.0\r\nCall-ID: {}\r\nCSeq: 1 INVITE\r\n\r\n", call_id);
        super::super::message::SipRequest::parse(request_str.as_bytes()).unwrap()
    }
}

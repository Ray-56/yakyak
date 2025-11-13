/// WebRTC to SIP Bridge
///
/// Allows WebRTC clients to communicate with SIP endpoints by translating
/// between WebRTC signaling and SIP signaling protocols.

use crate::infrastructure::protocols::sip::{CallRouter, SipRequest, SipResponse};
use crate::infrastructure::protocols::webrtc::session_manager::{WebRtcSessionManager, SessionState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Bridge session mapping WebRTC to SIP
#[derive(Debug, Clone)]
pub struct BridgeSession {
    pub bridge_id: Uuid,
    pub webrtc_session_id: Uuid,
    pub webrtc_peer_id: String,
    pub sip_call_id: Option<String>,
    pub sip_to_uri: Option<String>,
    pub state: BridgeState,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Bridge session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeState {
    /// Initial state
    New,
    /// WebRTC offer received, waiting to initiate SIP call
    WebRtcOfferReceived,
    /// SIP INVITE sent
    SipInviteSent,
    /// SIP call established, WebRTC answer pending
    SipEstablished,
    /// Both sides connected
    Connected,
    /// Call ended
    Ended,
    /// Error occurred
    Failed,
}

impl BridgeSession {
    pub fn new(webrtc_session_id: Uuid, webrtc_peer_id: String) -> Self {
        Self {
            bridge_id: Uuid::new_v4(),
            webrtc_session_id,
            webrtc_peer_id,
            sip_call_id: None,
            sip_to_uri: None,
            state: BridgeState::New,
            created_at: chrono::Utc::now(),
        }
    }
}

/// WebRTC to SIP Bridge
pub struct WebRtcSipBridge {
    /// WebRTC session manager
    webrtc_sessions: Arc<WebRtcSessionManager>,
    /// SIP call router
    sip_router: Option<Arc<CallRouter>>,
    /// Active bridge sessions (bridge_id -> session)
    bridge_sessions: Arc<RwLock<HashMap<Uuid, BridgeSession>>>,
    /// WebRTC to bridge mapping (webrtc_session_id -> bridge_id)
    webrtc_to_bridge: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    /// SIP to bridge mapping (sip_call_id -> bridge_id)
    sip_to_bridge: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl WebRtcSipBridge {
    /// Create new bridge
    pub fn new(webrtc_sessions: Arc<WebRtcSessionManager>, sip_router: Option<Arc<CallRouter>>) -> Self {
        Self {
            webrtc_sessions,
            sip_router,
            bridge_sessions: Arc::new(RwLock::new(HashMap::new())),
            webrtc_to_bridge: Arc::new(RwLock::new(HashMap::new())),
            sip_to_bridge: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create bridge session for WebRTC to SIP call
    pub async fn create_bridge_session(
        &self,
        webrtc_session_id: Uuid,
        webrtc_peer_id: String,
        sip_to_uri: String,
    ) -> Result<Uuid, String> {
        info!("Creating bridge session: WebRTC {} -> SIP {}", webrtc_peer_id, sip_to_uri);

        let mut session = BridgeSession::new(webrtc_session_id, webrtc_peer_id);
        session.sip_to_uri = Some(sip_to_uri);
        let bridge_id = session.bridge_id;

        let mut sessions = self.bridge_sessions.write().await;
        let mut webrtc_map = self.webrtc_to_bridge.write().await;

        sessions.insert(bridge_id, session);
        webrtc_map.insert(webrtc_session_id, bridge_id);

        Ok(bridge_id)
    }

    /// Handle WebRTC offer (initiate SIP call)
    pub async fn handle_webrtc_offer(
        &self,
        bridge_id: Uuid,
        sdp: String,
    ) -> Result<(), String> {
        info!("Handling WebRTC offer for bridge {}", bridge_id);

        let mut sessions = self.bridge_sessions.write().await;
        let session = sessions
            .get_mut(&bridge_id)
            .ok_or_else(|| "Bridge session not found".to_string())?;

        session.state = BridgeState::WebRtcOfferReceived;

        // Set offer in WebRTC session
        self.webrtc_sessions
            .set_remote_offer(session.webrtc_session_id, sdp.clone())
            .await?;

        // In production, would:
        // 1. Convert WebRTC SDP to SIP SDP
        // 2. Initiate SIP INVITE with converted SDP
        // 3. Wait for SIP 200 OK with answer SDP
        // 4. Convert SIP answer SDP to WebRTC format
        // 5. Send answer back to WebRTC client

        info!("WebRTC offer processed, SIP call would be initiated");
        session.state = BridgeState::SipInviteSent;

        Ok(())
    }

    /// Handle SIP answer (forward to WebRTC)
    pub async fn handle_sip_answer(
        &self,
        bridge_id: Uuid,
        sip_call_id: String,
        sdp: String,
    ) -> Result<String, String> {
        info!("Handling SIP answer for bridge {}", bridge_id);

        // Extract webrtc_session_id before dropping sessions
        let webrtc_session_id = {
            let mut sessions = self.bridge_sessions.write().await;
            let session = sessions
                .get_mut(&bridge_id)
                .ok_or_else(|| "Bridge session not found".to_string())?;

            session.sip_call_id = Some(sip_call_id.clone());
            session.state = BridgeState::SipEstablished;
            session.webrtc_session_id
        };

        // Update mapping
        let mut sip_map = self.sip_to_bridge.write().await;
        sip_map.insert(sip_call_id, bridge_id);
        drop(sip_map);

        // Convert SIP SDP to WebRTC format and create answer
        // In production, would perform actual SDP translation
        let webrtc_answer = self.webrtc_sessions
            .create_answer(webrtc_session_id)
            .await?;

        // Update state to connected
        let mut sessions = self.bridge_sessions.write().await;
        if let Some(session) = sessions.get_mut(&bridge_id) {
            session.state = BridgeState::Connected;
        }

        info!("Bridge session {} connected", bridge_id);
        Ok(webrtc_answer)
    }

    /// Handle call hangup
    pub async fn hangup(&self, bridge_id: Uuid) -> Result<(), String> {
        info!("Hanging up bridge session {}", bridge_id);

        let mut sessions = self.bridge_sessions.write().await;
        if let Some(session) = sessions.get_mut(&bridge_id) {
            session.state = BridgeState::Ended;

            // Close WebRTC session
            let _ = self.webrtc_sessions
                .close_session(session.webrtc_session_id)
                .await;

            // Hangup SIP call
            if let (Some(ref sip_router), Some(ref call_id)) = (&self.sip_router, &session.sip_call_id) {
                let _ = sip_router.terminate_call(call_id).await;
            }

            // Clean up mappings
            let webrtc_id = session.webrtc_session_id;
            let sip_id = session.sip_call_id.clone();

            drop(sessions);

            let mut webrtc_map = self.webrtc_to_bridge.write().await;
            webrtc_map.remove(&webrtc_id);

            if let Some(sip_id) = sip_id {
                let mut sip_map = self.sip_to_bridge.write().await;
                sip_map.remove(&sip_id);
            }

            info!("Bridge session {} ended", bridge_id);
            Ok(())
        } else {
            Err("Bridge session not found".to_string())
        }
    }

    /// Get bridge session by ID
    pub async fn get_session(&self, bridge_id: Uuid) -> Option<BridgeSession> {
        let sessions = self.bridge_sessions.read().await;
        sessions.get(&bridge_id).cloned()
    }

    /// Get bridge ID by WebRTC session ID
    pub async fn get_bridge_by_webrtc(&self, webrtc_session_id: Uuid) -> Option<Uuid> {
        let mapping = self.webrtc_to_bridge.read().await;
        mapping.get(&webrtc_session_id).copied()
    }

    /// Get bridge ID by SIP call ID
    pub async fn get_bridge_by_sip(&self, sip_call_id: &str) -> Option<Uuid> {
        let mapping = self.sip_to_bridge.read().await;
        mapping.get(sip_call_id).copied()
    }

    /// Get active bridge count
    pub async fn active_bridge_count(&self) -> usize {
        let sessions = self.bridge_sessions.read().await;
        sessions
            .values()
            .filter(|s| s.state == BridgeState::Connected)
            .count()
    }

    /// List all bridge sessions
    pub async fn list_sessions(&self) -> Vec<BridgeSession> {
        let sessions = self.bridge_sessions.read().await;
        sessions.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_session_creation() {
        let session = BridgeSession::new(
            Uuid::new_v4(),
            "webrtc_peer_1".to_string(),
        );

        assert_eq!(session.state, BridgeState::New);
        assert_eq!(session.webrtc_peer_id, "webrtc_peer_1");
    }

    #[tokio::test]
    async fn test_bridge_creation() {
        let webrtc_mgr = Arc::new(WebRtcSessionManager::new());
        let bridge = WebRtcSipBridge::new(webrtc_mgr, None);

        let webrtc_session_id = Uuid::new_v4();
        let bridge_id = bridge
            .create_bridge_session(
                webrtc_session_id,
                "peer1".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        let session = bridge.get_session(bridge_id).await;
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.webrtc_peer_id, "peer1");
        assert_eq!(session.sip_to_uri, Some("sip:bob@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_bridge_mappings() {
        let webrtc_mgr = Arc::new(WebRtcSessionManager::new());
        let bridge = WebRtcSipBridge::new(webrtc_mgr, None);

        let webrtc_session_id = Uuid::new_v4();
        let bridge_id = bridge
            .create_bridge_session(
                webrtc_session_id,
                "peer1".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        // Check WebRTC mapping
        let found_bridge = bridge.get_bridge_by_webrtc(webrtc_session_id).await;
        assert_eq!(found_bridge, Some(bridge_id));
    }

    #[tokio::test]
    async fn test_bridge_hangup() {
        let webrtc_mgr = Arc::new(WebRtcSessionManager::new());
        let bridge = WebRtcSipBridge::new(webrtc_mgr, None);

        let webrtc_session_id = Uuid::new_v4();
        let bridge_id = bridge
            .create_bridge_session(
                webrtc_session_id,
                "peer1".to_string(),
                "sip:bob@example.com".to_string(),
            )
            .await
            .unwrap();

        bridge.hangup(bridge_id).await.unwrap();

        let session = bridge.get_session(bridge_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().state, BridgeState::Ended);
    }
}

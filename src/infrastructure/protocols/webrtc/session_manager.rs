/// WebRTC Session Manager
///
/// Manages WebRTC peer connections and sessions, coordinating SDP offer/answer
/// exchanges and integrating with ICE/DTLS.

use crate::infrastructure::protocols::ice::{IceAgent, IceConfig};
use crate::infrastructure::protocols::webrtc::sdp::{
    WebRtcSdp, SdpType, MediaDescription, MediaType, RtpCodec,
    DtlsFingerprint, DtlsSetup, create_audio_offer,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// WebRTC session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    New,
    Offering,
    OfferReceived,
    Answering,
    AnswerReceived,
    Connected,
    Failed,
    Closed,
}

/// WebRTC session
pub struct WebRtcSession {
    pub session_id: Uuid,
    pub peer_id: String,
    pub state: SessionState,
    pub local_sdp: Option<WebRtcSdp>,
    pub remote_sdp: Option<WebRtcSdp>,
    pub ice_agent: Option<Arc<IceAgent>>,
    pub dtls_fingerprint: Option<DtlsFingerprint>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl WebRtcSession {
    /// Create new session
    pub fn new(peer_id: String) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            peer_id,
            state: SessionState::New,
            local_sdp: None,
            remote_sdp: None,
            ice_agent: None,
            dtls_fingerprint: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create local offer
    pub async fn create_offer(&mut self) -> Result<String, String> {
        if self.state != SessionState::New && self.state != SessionState::Closed {
            return Err("Cannot create offer in current state".to_string());
        }

        // Initialize ICE agent
        let ice_config = IceConfig::default();
        let ice_agent = Arc::new(IceAgent::new(ice_config));
        ice_agent.gather_candidates().await?;

        // Generate ICE credentials
        let ice_ufrag = format!("yakyak{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        let ice_pwd = Uuid::new_v4().to_string().replace("-", "");

        // Create SDP offer
        let mut offer = create_audio_offer(ice_ufrag, ice_pwd);

        // Add ICE candidates to SDP
        let candidates = ice_agent.get_local_candidates().await;
        if let Some(media) = offer.media_descriptions.first_mut() {
            for candidate in candidates {
                media.add_ice_candidate(candidate);
            }

            // Add DTLS fingerprint (would be generated from cert in production)
            let fingerprint = DtlsFingerprint::sha256(
                "AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99".to_string()
            );
            media.dtls_fingerprint = Some(fingerprint.clone());
            media.dtls_setup = Some(DtlsSetup::Actpass);

            self.dtls_fingerprint = Some(fingerprint);
        }

        self.local_sdp = Some(offer.clone());
        self.ice_agent = Some(ice_agent);
        self.state = SessionState::Offering;

        let sdp_string = offer.to_sdp_string();
        info!("Created WebRTC offer for session {}", self.session_id);
        Ok(sdp_string)
    }

    /// Set remote offer
    pub async fn set_remote_offer(&mut self, sdp: String) -> Result<(), String> {
        if self.state != SessionState::New {
            return Err("Cannot set remote offer in current state".to_string());
        }

        let offer = WebRtcSdp::from_sdp_string(&sdp, SdpType::Offer)?;
        self.remote_sdp = Some(offer);
        self.state = SessionState::OfferReceived;

        info!("Set remote offer for session {}", self.session_id);
        Ok(())
    }

    /// Create answer to remote offer
    pub async fn create_answer(&mut self) -> Result<String, String> {
        if self.state != SessionState::OfferReceived {
            return Err("No offer to answer".to_string());
        }

        // Initialize ICE agent
        let ice_config = IceConfig::default();
        let ice_agent = Arc::new(IceAgent::new(ice_config));
        ice_agent.gather_candidates().await?;

        // Generate ICE credentials
        let ice_ufrag = format!("yakyak{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        let ice_pwd = Uuid::new_v4().to_string().replace("-", "");

        // Create SDP answer
        let mut answer = WebRtcSdp::new(SdpType::Answer);

        // Match media from offer
        if let Some(ref remote) = self.remote_sdp {
            for remote_media in &remote.media_descriptions {
                let mut media = MediaDescription::new(remote_media.media_type, 9);
                media.mid = remote_media.mid.clone();
                media.set_ice_credentials(ice_ufrag.clone(), ice_pwd.clone());

                // Match codecs (simple codec matching)
                for codec in &remote_media.codecs {
                    media.add_codec(codec.clone());
                }

                // Add ICE candidates
                let candidates = ice_agent.get_local_candidates().await;
                for candidate in candidates {
                    media.add_ice_candidate(candidate);
                }

                // Add DTLS
                let fingerprint = DtlsFingerprint::sha256(
                    "AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99".to_string()
                );
                media.dtls_fingerprint = Some(fingerprint.clone());
                media.dtls_setup = Some(DtlsSetup::Active);

                self.dtls_fingerprint = Some(fingerprint);

                answer.add_media(media);
            }
        }

        answer.enable_bundle();
        self.local_sdp = Some(answer.clone());
        self.ice_agent = Some(ice_agent);
        self.state = SessionState::Answering;

        let sdp_string = answer.to_sdp_string();
        info!("Created WebRTC answer for session {}", self.session_id);
        Ok(sdp_string)
    }

    /// Set remote answer
    pub async fn set_remote_answer(&mut self, sdp: String) -> Result<(), String> {
        if self.state != SessionState::Offering {
            return Err("Not waiting for answer".to_string());
        }

        let answer = WebRtcSdp::from_sdp_string(&sdp, SdpType::Answer)?;
        self.remote_sdp = Some(answer);
        self.state = SessionState::AnswerReceived;

        // Start ICE connectivity checks
        if let Some(ref ice_agent) = self.ice_agent {
            // In production, would extract remote candidates from answer
            // and add them to ICE agent
            ice_agent.start_checks().await?;
        }

        info!("Set remote answer for session {}", self.session_id);
        self.state = SessionState::Connected;
        Ok(())
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&mut self, candidate: String) -> Result<(), String> {
        debug!("Adding ICE candidate to session {}: {}", self.session_id, candidate);
        // In production, would parse and add to ICE agent
        Ok(())
    }

    /// Close session
    pub fn close(&mut self) {
        self.state = SessionState::Closed;
        info!("Closed WebRTC session {}", self.session_id);
    }
}

/// WebRTC Session Manager
pub struct WebRtcSessionManager {
    /// Active sessions (session_id -> session)
    sessions: Arc<RwLock<HashMap<Uuid, WebRtcSession>>>,
    /// Peer to session mapping (peer_id -> session_id)
    peer_sessions: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl WebRtcSessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            peer_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new session for peer
    pub async fn create_session(&self, peer_id: String) -> Result<Uuid, String> {
        let session = WebRtcSession::new(peer_id.clone());
        let session_id = session.session_id;

        let mut sessions = self.sessions.write().await;
        let mut peer_sessions = self.peer_sessions.write().await;

        sessions.insert(session_id, session);
        peer_sessions.insert(peer_id, session_id);

        info!("Created WebRTC session: {}", session_id);
        Ok(session_id)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Option<WebRtcSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).map(|s| WebRtcSession {
            session_id: s.session_id,
            peer_id: s.peer_id.clone(),
            state: s.state,
            local_sdp: None, // Don't clone SDP
            remote_sdp: None,
            ice_agent: None,
            dtls_fingerprint: s.dtls_fingerprint.clone(),
            created_at: s.created_at,
        })
    }

    /// Get session by peer ID
    pub async fn get_session_by_peer(&self, peer_id: &str) -> Option<Uuid> {
        let peer_sessions = self.peer_sessions.read().await;
        peer_sessions.get(peer_id).copied()
    }

    /// Create offer for session
    pub async fn create_offer(&self, session_id: Uuid) -> Result<String, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.create_offer().await
    }

    /// Set remote offer
    pub async fn set_remote_offer(&self, session_id: Uuid, sdp: String) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.set_remote_offer(sdp).await
    }

    /// Create answer
    pub async fn create_answer(&self, session_id: Uuid) -> Result<String, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.create_answer().await
    }

    /// Set remote answer
    pub async fn set_remote_answer(&self, session_id: Uuid, sdp: String) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.set_remote_answer(sdp).await
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, session_id: Uuid, candidate: String) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.add_ice_candidate(candidate).await
    }

    /// Close session
    pub async fn close_session(&self, session_id: Uuid) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let mut peer_sessions = self.peer_sessions.write().await;

        if let Some(mut session) = sessions.remove(&session_id) {
            peer_sessions.remove(&session.peer_id);
            session.close();
            info!("Removed WebRTC session: {}", session_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Get active session count
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<(Uuid, String, SessionState)> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|s| (s.session_id, s.peer_id.clone(), s.state))
            .collect()
    }
}

impl Default for WebRtcSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let mut session = WebRtcSession::new("peer1".to_string());
        assert_eq!(session.state, SessionState::New);
        assert_eq!(session.peer_id, "peer1");
    }

    #[tokio::test]
    async fn test_create_offer() {
        let mut session = WebRtcSession::new("peer1".to_string());
        let sdp = session.create_offer().await.unwrap();

        assert!(sdp.contains("v=0"));
        assert!(sdp.contains("m=audio"));
        assert_eq!(session.state, SessionState::Offering);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = WebRtcSessionManager::new();

        let session_id = manager.create_session("peer1".to_string()).await.unwrap();
        assert_eq!(manager.active_session_count().await, 1);

        let session = manager.get_session(session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().peer_id, "peer1");

        manager.close_session(session_id).await.unwrap();
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_offer_answer_flow() {
        let manager = WebRtcSessionManager::new();

        // Peer 1 creates offer
        let session1_id = manager.create_session("peer1".to_string()).await.unwrap();
        let offer = manager.create_offer(session1_id).await.unwrap();

        assert!(offer.contains("m=audio"));
        assert!(offer.contains("a=ice-ufrag:"));

        // Peer 2 receives offer and creates answer
        let session2_id = manager.create_session("peer2".to_string()).await.unwrap();
        manager.set_remote_offer(session2_id, offer).await.unwrap();
        let answer = manager.create_answer(session2_id).await.unwrap();

        assert!(answer.contains("m=audio"));

        // Peer 1 receives answer
        manager.set_remote_answer(session1_id, answer).await.unwrap();

        let session1 = manager.get_session(session1_id).await.unwrap();
        assert_eq!(session1.state, SessionState::Connected);
    }
}

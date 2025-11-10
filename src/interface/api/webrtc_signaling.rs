/// WebRTC signaling server for browser-based clients
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalingMessage {
    /// Register a WebRTC peer
    Register {
        peer_id: String,
        username: String,
    },
    /// Unregister a WebRTC peer
    Unregister {
        peer_id: String,
    },
    /// SDP offer
    Offer {
        from: String,
        to: String,
        sdp: String,
    },
    /// SDP answer
    Answer {
        from: String,
        to: String,
        sdp: String,
    },
    /// ICE candidate
    IceCandidate {
        from: String,
        to: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    },
    /// Call initiation
    Call {
        from: String,
        to: String,
    },
    /// Call acceptance
    Accept {
        from: String,
        to: String,
    },
    /// Call rejection
    Reject {
        from: String,
        to: String,
        reason: Option<String>,
    },
    /// Call hangup
    Hangup {
        from: String,
        to: String,
    },
    /// Peer status update
    PeerStatus {
        peer_id: String,
        online: bool,
    },
    /// Error message
    Error {
        code: String,
        message: String,
    },
    /// Success acknowledgment
    Success {
        message: String,
    },
}

/// WebRTC peer information
#[derive(Debug, Clone)]
pub struct WebRtcPeer {
    pub peer_id: String,
    pub username: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// WebRTC signaling state
pub struct SignalingState {
    /// Active peers (peer_id -> peer info)
    peers: Arc<RwLock<HashMap<String, WebRtcPeer>>>,
    /// Broadcast channel for signaling messages
    tx: broadcast::Sender<(String, SignalingMessage)>,
}

impl SignalingState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    /// Register a new peer
    pub async fn register_peer(&self, peer_id: String, username: String) -> Result<(), String> {
        let mut peers = self.peers.write().await;

        if peers.contains_key(&peer_id) {
            return Err("Peer already registered".to_string());
        }

        let peer = WebRtcPeer {
            peer_id: peer_id.clone(),
            username,
            connected_at: chrono::Utc::now(),
        };

        peers.insert(peer_id.clone(), peer);

        // Broadcast peer status
        let _ = self.tx.send((
            peer_id.clone(),
            SignalingMessage::PeerStatus {
                peer_id,
                online: true,
            },
        ));

        Ok(())
    }

    /// Unregister a peer
    pub async fn unregister_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;

        if peers.remove(peer_id).is_some() {
            // Broadcast peer status
            let _ = self.tx.send((
                peer_id.to_string(),
                SignalingMessage::PeerStatus {
                    peer_id: peer_id.to_string(),
                    online: false,
                },
            ));
        }
    }

    /// Get peer by ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<WebRtcPeer> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// List all online peers
    pub async fn list_peers(&self) -> Vec<WebRtcPeer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Send message to specific peer
    pub fn send_to_peer(&self, to: String, message: SignalingMessage) {
        let _ = self.tx.send((to, message));
    }

    /// Broadcast message to all peers
    pub fn broadcast(&self, message: SignalingMessage) {
        let _ = self.tx.send(("*".to_string(), message));
    }

    /// Subscribe to messages
    pub fn subscribe(&self) -> broadcast::Receiver<(String, SignalingMessage)> {
        self.tx.subscribe()
    }
}

/// Create WebRTC signaling router
pub fn webrtc_signaling_router(state: Arc<SignalingState>) -> Router {
    Router::new()
        .route("/webrtc/signaling/:peer_id", get(websocket_handler))
        .with_state(state)
}

/// WebSocket handler for signaling
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(peer_id): Path<String>,
    State(state): State<Arc<SignalingState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, peer_id, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, peer_id: String, state: Arc<SignalingState>) {
    info!("WebRTC signaling connection established for peer: {}", peer_id);

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to messages
    let mut rx = state.subscribe();

    // Task to send messages to this peer
    let peer_id_clone = peer_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok((to, message)) = rx.recv().await {
            // Only send if message is for this peer or broadcast
            if to == peer_id_clone || to == "*" {
                let json = match serde_json::to_string(&message) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task to receive messages from this peer
    let state_clone = state.clone();
    let peer_id_clone = peer_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    debug!("Received signaling message from {}: {}", peer_id_clone, text);

                    let message: SignalingMessage = match serde_json::from_str(&text) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Failed to parse signaling message: {}", e);
                            let error_msg = SignalingMessage::Error {
                                code: "PARSE_ERROR".to_string(),
                                message: format!("Invalid message format: {}", e),
                            };
                            state_clone.send_to_peer(peer_id_clone.clone(), error_msg);
                            continue;
                        }
                    };

                    handle_signaling_message(&state_clone, &peer_id_clone, message).await;
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed for peer: {}", peer_id_clone);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Respond to ping (axum handles this automatically)
                    debug!("Received ping from {}", peer_id_clone);
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from {}", peer_id_clone);
                }
                Ok(_) => {
                    warn!("Received unexpected message type from {}", peer_id_clone);
                }
                Err(e) => {
                    error!("WebSocket error for {}: {}", peer_id_clone, e);
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {
            debug!("Send task finished for {}", peer_id);
        }
        _ = recv_task => {
            debug!("Receive task finished for {}", peer_id);
        }
    }

    // Cleanup
    state.unregister_peer(&peer_id).await;
    info!("WebRTC signaling connection closed for peer: {}", peer_id);
}

/// Handle individual signaling messages
async fn handle_signaling_message(
    state: &Arc<SignalingState>,
    peer_id: &str,
    message: SignalingMessage,
) {
    match message {
        SignalingMessage::Register { peer_id: reg_peer_id, username } => {
            // Ensure the peer_id matches
            if reg_peer_id != peer_id {
                let error_msg = SignalingMessage::Error {
                    code: "PEER_ID_MISMATCH".to_string(),
                    message: "Peer ID in message does not match connection".to_string(),
                };
                state.send_to_peer(peer_id.to_string(), error_msg);
                return;
            }

            match state.register_peer(reg_peer_id.clone(), username.clone()).await {
                Ok(_) => {
                    info!("Registered WebRTC peer: {} ({})", reg_peer_id, username);
                    let success_msg = SignalingMessage::Success {
                        message: format!("Registered as {}", username),
                    };
                    state.send_to_peer(reg_peer_id, success_msg);
                }
                Err(e) => {
                    error!("Failed to register peer {}: {}", peer_id, e);
                    let error_msg = SignalingMessage::Error {
                        code: "REGISTRATION_FAILED".to_string(),
                        message: e,
                    };
                    state.send_to_peer(peer_id.to_string(), error_msg);
                }
            }
        }

        SignalingMessage::Offer { from, to, sdp } => {
            info!("Forwarding SDP offer from {} to {}", from, to);

            // Verify sender
            if from != peer_id {
                let error_msg = SignalingMessage::Error {
                    code: "SENDER_MISMATCH".to_string(),
                    message: "Sender ID does not match connection".to_string(),
                };
                state.send_to_peer(peer_id.to_string(), error_msg);
                return;
            }

            // Check if recipient exists
            if state.get_peer(&to).await.is_none() {
                let error_msg = SignalingMessage::Error {
                    code: "PEER_NOT_FOUND".to_string(),
                    message: format!("Peer {} not found", to),
                };
                state.send_to_peer(from, error_msg);
                return;
            }

            // Forward the offer
            state.send_to_peer(
                to,
                SignalingMessage::Offer { from, to: to.clone(), sdp },
            );
        }

        SignalingMessage::Answer { from, to, sdp } => {
            info!("Forwarding SDP answer from {} to {}", from, to);

            // Verify sender
            if from != peer_id {
                let error_msg = SignalingMessage::Error {
                    code: "SENDER_MISMATCH".to_string(),
                    message: "Sender ID does not match connection".to_string(),
                };
                state.send_to_peer(peer_id.to_string(), error_msg);
                return;
            }

            // Forward the answer
            state.send_to_peer(
                to,
                SignalingMessage::Answer { from, to: to.clone(), sdp },
            );
        }

        SignalingMessage::IceCandidate {
            from,
            to,
            candidate,
            sdp_mid,
            sdp_m_line_index,
        } => {
            debug!("Forwarding ICE candidate from {} to {}", from, to);

            // Verify sender
            if from != peer_id {
                return;
            }

            // Forward the ICE candidate
            state.send_to_peer(
                to,
                SignalingMessage::IceCandidate {
                    from,
                    to: to.clone(),
                    candidate,
                    sdp_mid,
                    sdp_m_line_index,
                },
            );
        }

        SignalingMessage::Call { from, to } => {
            info!("Call initiation from {} to {}", from, to);

            if from != peer_id {
                return;
            }

            // Forward call signal
            state.send_to_peer(to, SignalingMessage::Call { from, to: to.clone() });
        }

        SignalingMessage::Accept { from, to } => {
            info!("Call accepted: {} -> {}", from, to);

            if from != peer_id {
                return;
            }

            state.send_to_peer(to, SignalingMessage::Accept { from, to: to.clone() });
        }

        SignalingMessage::Reject { from, to, reason } => {
            info!("Call rejected: {} -> {} (reason: {:?})", from, to, reason);

            if from != peer_id {
                return;
            }

            state.send_to_peer(
                to,
                SignalingMessage::Reject {
                    from,
                    to: to.clone(),
                    reason,
                },
            );
        }

        SignalingMessage::Hangup { from, to } => {
            info!("Call hangup: {} -> {}", from, to);

            if from != peer_id {
                return;
            }

            state.send_to_peer(to, SignalingMessage::Hangup { from, to: to.clone() });
        }

        _ => {
            warn!("Unexpected signaling message type from {}", peer_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_peer() {
        let state = SignalingState::new();

        let result = state.register_peer("peer1".to_string(), "alice".to_string()).await;
        assert!(result.is_ok());

        let peer = state.get_peer("peer1").await;
        assert!(peer.is_some());
        assert_eq!(peer.unwrap().username, "alice");
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let state = SignalingState::new();

        state.register_peer("peer1".to_string(), "alice".to_string()).await.unwrap();
        let result = state.register_peer("peer1".to_string(), "bob".to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_peer() {
        let state = SignalingState::new();

        state.register_peer("peer1".to_string(), "alice".to_string()).await.unwrap();
        state.unregister_peer("peer1").await;

        let peer = state.get_peer("peer1").await;
        assert!(peer.is_none());
    }

    #[tokio::test]
    async fn test_list_peers() {
        let state = SignalingState::new();

        state.register_peer("peer1".to_string(), "alice".to_string()).await.unwrap();
        state.register_peer("peer2".to_string(), "bob".to_string()).await.unwrap();

        let peers = state.list_peers().await;
        assert_eq!(peers.len(), 2);
    }
}

/// WebSocket event streaming for real-time monitoring
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// System event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemEvent {
    /// Call event
    Call {
        event: CallEvent,
        call_id: String,
        timestamp: i64,
    },
    /// Registration event
    Registration {
        event: RegistrationEvent,
        username: String,
        timestamp: i64,
    },
    /// Authentication event
    Authentication {
        event: AuthEvent,
        username: Option<String>,
        ip: String,
        timestamp: i64,
    },
    /// System health update
    HealthUpdate {
        status: String,
        message: String,
        timestamp: i64,
    },
    /// Conference event
    Conference {
        event: ConferenceEvent,
        room_id: Uuid,
        timestamp: i64,
    },
    /// Custom event
    Custom {
        name: String,
        data: serde_json::Value,
        timestamp: i64,
    },
}

/// Call event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallEvent {
    Created { caller: String, callee: String },
    Ringing { caller: String, callee: String },
    Answered { caller: String, callee: String },
    Terminated { caller: String, callee: String, reason: String },
    Failed { caller: String, callee: String, error: String },
    Hold { party: String },
    Resume { party: String },
    Transfer { from: String, to: String },
}

/// Registration event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationEvent {
    Registered { contact: String, expires: u32 },
    Unregistered { contact: String },
    Expired { contact: String },
}

/// Authentication event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthEvent {
    Success { method: String },
    Failure { method: String, reason: String },
    Lockout { reason: String },
    RateLimited,
}

/// Conference event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConferenceEvent {
    Created { name: String },
    Started { name: String },
    Ended { name: String },
    ParticipantJoined { participant_id: Uuid, name: String },
    ParticipantLeft { participant_id: Uuid, name: String },
    ParticipantMuted { participant_id: Uuid },
    ParticipantUnmuted { participant_id: Uuid },
}

/// Event broadcaster
pub struct EventBroadcaster {
    tx: broadcast::Sender<SystemEvent>,
}

impl EventBroadcaster {
    /// Create new event broadcaster with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Create with default capacity (1000 events)
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }

    /// Broadcast an event
    pub fn broadcast(&self, event: SystemEvent) {
        if let Err(e) = self.tx.send(event) {
            warn!("Failed to broadcast event: {}", e);
        }
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Broadcast call created event
    pub fn call_created(&self, call_id: String, caller: String, callee: String) {
        self.broadcast(SystemEvent::Call {
            event: CallEvent::Created { caller, callee },
            call_id,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast call answered event
    pub fn call_answered(&self, call_id: String, caller: String, callee: String) {
        self.broadcast(SystemEvent::Call {
            event: CallEvent::Answered { caller, callee },
            call_id,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast call terminated event
    pub fn call_terminated(&self, call_id: String, caller: String, callee: String, reason: String) {
        self.broadcast(SystemEvent::Call {
            event: CallEvent::Terminated { caller, callee, reason },
            call_id,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast registration event
    pub fn registration(
        &self,
        event: RegistrationEvent,
        username: String,
    ) {
        self.broadcast(SystemEvent::Registration {
            event,
            username,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast authentication success
    pub fn auth_success(&self, username: String, ip: String, method: String) {
        self.broadcast(SystemEvent::Authentication {
            event: AuthEvent::Success { method },
            username: Some(username),
            ip,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast authentication failure
    pub fn auth_failure(&self, username: Option<String>, ip: String, method: String, reason: String) {
        self.broadcast(SystemEvent::Authentication {
            event: AuthEvent::Failure { method, reason },
            username,
            ip,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast conference event
    pub fn conference_event(&self, room_id: Uuid, event: ConferenceEvent) {
        self.broadcast(SystemEvent::Conference {
            event,
            room_id,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Broadcast health update
    pub fn health_update(&self, status: String, message: String) {
        self.broadcast(SystemEvent::HealthUpdate {
            status,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }
}

/// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<Arc<EventBroadcaster>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, broadcaster))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, broadcaster: Arc<EventBroadcaster>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcaster.subscribe();

    info!("WebSocket client connected");

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "message": "Connected to YakYak event stream",
        "timestamp": chrono::Utc::now().timestamp(),
    });

    if let Ok(msg) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(msg)).await.is_err() {
            error!("Failed to send welcome message");
            return;
        }
    }

    // Spawn task to receive messages from client (for ping/pong)
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json)).await.is_err() {
                    debug!("Client disconnected");
                    break;
                }
            }
        }
    });

    // Handle incoming messages (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("WebSocket client requested close");
                    break;
                }
                Message::Ping(data) => {
                    debug!("Received ping");
                    // Pong is automatically handled by axum
                }
                Message::Pong(_) => {
                    debug!("Received pong");
                }
                Message::Text(text) => {
                    debug!("Received text message: {}", text);
                    // Could handle client commands here
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("WebSocket client disconnected");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_broadcaster_creation() {
        let broadcaster = EventBroadcaster::new(100);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn test_subscribe() {
        let broadcaster = EventBroadcaster::new(100);
        let _rx = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_broadcast_event() {
        let broadcaster = EventBroadcaster::new(100);
        let mut rx = broadcaster.subscribe();

        broadcaster.call_created(
            "call-123".to_string(),
            "alice".to_string(),
            "bob".to_string(),
        );

        let event = rx.recv().await.unwrap();
        match event {
            SystemEvent::Call { event, call_id, .. } => {
                assert_eq!(call_id, "call-123");
                match event {
                    CallEvent::Created { caller, callee } => {
                        assert_eq!(caller, "alice");
                        assert_eq!(callee, "bob");
                    }
                    _ => panic!("Wrong event type"),
                }
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new(100);
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 2);

        broadcaster.health_update("healthy".to_string(), "All systems operational".to_string());

        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        match (&event1, &event2) {
            (
                SystemEvent::HealthUpdate { status: s1, message: m1, .. },
                SystemEvent::HealthUpdate { status: s2, message: m2, .. },
            ) => {
                assert_eq!(s1, "healthy");
                assert_eq!(s2, "healthy");
                assert_eq!(m1, "All systems operational");
                assert_eq!(m2, "All systems operational");
            }
            _ => panic!("Wrong event types"),
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = SystemEvent::Call {
            event: CallEvent::Answered {
                caller: "alice".to_string(),
                callee: "bob".to_string(),
            },
            call_id: "call-123".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"call\""));
        assert!(json.contains("\"call_id\":\"call-123\""));
    }

    #[tokio::test]
    async fn test_auth_events() {
        let broadcaster = EventBroadcaster::new(100);
        let mut rx = broadcaster.subscribe();

        broadcaster.auth_success(
            "alice".to_string(),
            "192.168.1.100".to_string(),
            "REGISTER".to_string(),
        );

        let event = rx.recv().await.unwrap();
        match event {
            SystemEvent::Authentication { event, username, ip, .. } => {
                assert_eq!(username, Some("alice".to_string()));
                assert_eq!(ip, "192.168.1.100");
                match event {
                    AuthEvent::Success { method } => {
                        assert_eq!(method, "REGISTER");
                    }
                    _ => panic!("Wrong auth event type"),
                }
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_conference_events() {
        let broadcaster = EventBroadcaster::new(100);
        let mut rx = broadcaster.subscribe();

        let room_id = Uuid::new_v4();
        broadcaster.conference_event(
            room_id,
            ConferenceEvent::Created { name: "Team Meeting".to_string() },
        );

        let event = rx.recv().await.unwrap();
        match event {
            SystemEvent::Conference { event, room_id: rid, .. } => {
                assert_eq!(rid, room_id);
                match event {
                    ConferenceEvent::Created { name } => {
                        assert_eq!(name, "Team Meeting");
                    }
                    _ => panic!("Wrong conference event type"),
                }
            }
            _ => panic!("Wrong event type"),
        }
    }
}

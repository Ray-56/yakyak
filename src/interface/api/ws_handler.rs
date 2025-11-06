//! WebSocket event streaming handler

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// Event types that can be broadcast to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    /// Call initiated event
    CallInitiated {
        call_id: String,
        caller_uri: String,
        callee_uri: String,
    },
    /// Call state changed event
    CallStateChanged {
        call_id: String,
        old_state: String,
        new_state: String,
    },
    /// Call ended event
    CallEnded {
        call_id: String,
        duration: i64,
        reason: String,
    },
    /// User registered event
    UserRegistered {
        aor: String,
        contact: String,
        expires: u32,
    },
    /// User unregistered event
    UserUnregistered { aor: String },
    /// Active calls count updated
    ActiveCallsUpdated { count: usize },
    /// Registered users count updated
    RegisteredUsersUpdated { count: usize },
}

/// Event broadcaster
#[derive(Clone)]
pub struct EventBroadcaster {
    tx: broadcast::Sender<Event>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self { tx }
    }

    /// Publish an event
    pub fn publish(&self, event: Event) {
        // Ignore send errors (no receivers)
        let _ = self.tx.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket handler
pub async fn ws_handler(
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

    // Spawn a task to send events to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        debug!("Failed to send event to WebSocket client");
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize event: {}", e);
                }
            }
        }
    });

    // Spawn a task to receive messages from the client (for heartbeat/ping)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received text message: {}", text);
                }
                Message::Ping(_) => {
                    debug!("Received ping");
                    // Axum automatically responds to pings
                }
                Message::Pong(_) => {
                    debug!("Received pong");
                }
                Message::Close(_) => {
                    debug!("Received close message");
                    break;
                }
                Message::Binary(_) => {
                    debug!("Received binary message (ignored)");
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("WebSocket client disconnected");
}

// Need to import futures StreamExt for split() and SinkExt for send()
use futures::{SinkExt, StreamExt};

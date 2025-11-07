/// Voicemail management REST API
use crate::domain::voicemail::{VoicemailMailbox, VoicemailMessage, VoicemailRepository, VoicemailStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Voicemail API state
pub struct VoicemailApiState {
    pub repository: Arc<dyn VoicemailRepository>,
}

/// Create voicemail router
pub fn voicemail_router(state: Arc<VoicemailApiState>) -> Router {
    Router::new()
        // Mailbox management
        .route("/voicemail/mailboxes/:mailbox_id", get(get_mailbox))
        .route("/voicemail/mailboxes/:mailbox_id", put(update_mailbox))

        // Message management
        .route("/voicemail/mailboxes/:mailbox_id/messages", get(list_messages))
        .route("/voicemail/mailboxes/:mailbox_id/messages", post(create_message))
        .route("/voicemail/messages/:id", get(get_message))
        .route("/voicemail/messages/:id", delete(delete_message))
        .route("/voicemail/messages/:id/status", put(update_message_status))
        .route("/voicemail/messages/:id/mark-read", post(mark_as_read))
        .route("/voicemail/messages/:id/mark-saved", post(mark_as_saved))

        // Statistics
        .route("/voicemail/mailboxes/:mailbox_id/count", get(get_message_count))
        .with_state(state)
}

/// Request to create/update a mailbox
#[derive(Debug, Deserialize)]
struct MailboxRequest {
    user_id: Option<i32>,
    pin: Option<String>,
    greeting_file: Option<String>,
    max_message_duration: Option<u32>,
    max_messages: Option<u32>,
    email_notification: Option<bool>,
    email_address: Option<String>,
}

/// Response for mailbox operations
#[derive(Debug, Serialize)]
struct MailboxResponse {
    mailbox_id: String,
    user_id: i32,
    pin: Option<String>,
    greeting_file: Option<String>,
    max_message_duration: u32,
    max_messages: u32,
    email_notification: bool,
    email_address: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<VoicemailMailbox> for MailboxResponse {
    fn from(mailbox: VoicemailMailbox) -> Self {
        Self {
            mailbox_id: mailbox.mailbox_id,
            user_id: mailbox.user_id,
            pin: mailbox.pin,
            greeting_file: mailbox.greeting_file,
            max_message_duration: mailbox.max_message_duration,
            max_messages: mailbox.max_messages,
            email_notification: mailbox.email_notification,
            email_address: mailbox.email_address,
            created_at: mailbox.created_at.to_rfc3339(),
            updated_at: mailbox.updated_at.to_rfc3339(),
        }
    }
}

/// Request to create a message
#[derive(Debug, Deserialize)]
struct CreateMessageRequest {
    caller: String,
    caller_name: Option<String>,
    duration_seconds: u32,
    audio_file_path: String,
    audio_format: String,
}

/// Response for message operations
#[derive(Debug, Serialize)]
struct MessageResponse {
    id: Uuid,
    mailbox_id: String,
    caller: String,
    caller_name: Option<String>,
    duration_seconds: u32,
    audio_file_path: String,
    audio_format: String,
    status: String,
    created_at: String,
    read_at: Option<String>,
    saved_at: Option<String>,
}

impl From<VoicemailMessage> for MessageResponse {
    fn from(message: VoicemailMessage) -> Self {
        Self {
            id: message.id,
            mailbox_id: message.mailbox_id,
            caller: message.caller,
            caller_name: message.caller_name,
            duration_seconds: message.duration_seconds,
            audio_file_path: message.audio_file_path,
            audio_format: message.audio_format,
            status: format!("{:?}", message.status),
            created_at: message.created_at.to_rfc3339(),
            read_at: message.read_at.map(|dt| dt.to_rfc3339()),
            saved_at: message.saved_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Query parameters for listing messages
#[derive(Debug, Deserialize)]
struct ListMessagesQuery {
    status: Option<String>,
}

/// Request to update message status
#[derive(Debug, Deserialize)]
struct UpdateStatusRequest {
    status: String,
}

/// Message count response
#[derive(Debug, Serialize)]
struct MessageCountResponse {
    total: u32,
    new: u32,
    read: u32,
    saved: u32,
}

/// Get mailbox by ID
async fn get_mailbox(
    State(state): State<Arc<VoicemailApiState>>,
    Path(mailbox_id): Path<String>,
) -> Response {
    match state.repository.get_mailbox(&mailbox_id).await {
        Ok(Some(mailbox)) => {
            let response = MailboxResponse::from(mailbox);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Mailbox not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update mailbox configuration
async fn update_mailbox(
    State(state): State<Arc<VoicemailApiState>>,
    Path(mailbox_id): Path<String>,
    Json(req): Json<MailboxRequest>,
) -> Response {
    // Get existing mailbox or create new one
    let mut mailbox = match state.repository.get_mailbox(&mailbox_id).await {
        Ok(Some(mailbox)) => mailbox,
        Ok(None) => {
            let user_id = req.user_id.unwrap_or(0);
            VoicemailMailbox::new(mailbox_id.clone(), user_id)
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    // Update fields
    if let Some(pin) = req.pin {
        mailbox.pin = Some(pin);
    }
    if let Some(greeting) = req.greeting_file {
        mailbox.greeting_file = Some(greeting);
    }
    if let Some(max_duration) = req.max_message_duration {
        mailbox.max_message_duration = max_duration;
    }
    if let Some(max_messages) = req.max_messages {
        mailbox.max_messages = max_messages;
    }
    if let Some(email_notif) = req.email_notification {
        mailbox.email_notification = email_notif;
    }
    if let Some(email) = req.email_address {
        mailbox.email_address = Some(email);
    }

    mailbox.updated_at = chrono::Utc::now();

    match state.repository.save_mailbox(mailbox).await {
        Ok(mailbox) => {
            let response = MailboxResponse::from(mailbox);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List messages for a mailbox
async fn list_messages(
    State(state): State<Arc<VoicemailApiState>>,
    Path(mailbox_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Response {
    let status_filter = query.status.and_then(|s| match s.as_str() {
        "new" => Some(VoicemailStatus::New),
        "read" => Some(VoicemailStatus::Read),
        "saved" => Some(VoicemailStatus::Saved),
        "deleted" => Some(VoicemailStatus::Deleted),
        _ => None,
    });

    match state.repository.list_messages(&mailbox_id, status_filter).await {
        Ok(messages) => {
            let response: Vec<MessageResponse> = messages
                .into_iter()
                .map(MessageResponse::from)
                .collect();
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Create a new voicemail message
async fn create_message(
    State(state): State<Arc<VoicemailApiState>>,
    Path(mailbox_id): Path<String>,
    Json(req): Json<CreateMessageRequest>,
) -> Response {
    let message = VoicemailMessage::new(
        mailbox_id,
        req.caller,
        req.caller_name,
        req.duration_seconds,
        req.audio_file_path,
        req.audio_format,
    );

    match state.repository.create_message(message).await {
        Ok(message) => {
            let response = MessageResponse::from(message);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get a specific message
async fn get_message(
    State(state): State<Arc<VoicemailApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_message(id).await {
        Ok(Some(message)) => {
            let response = MessageResponse::from(message);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Message not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Delete a message
async fn delete_message(
    State(state): State<Arc<VoicemailApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.delete_message(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update message status
async fn update_message_status(
    State(state): State<Arc<VoicemailApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> Response {
    let status = match req.status.as_str() {
        "new" => VoicemailStatus::New,
        "read" => VoicemailStatus::Read,
        "saved" => VoicemailStatus::Saved,
        "deleted" => VoicemailStatus::Deleted,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid status" })),
            )
                .into_response()
        }
    };

    match state.repository.update_message_status(id, status).await {
        Ok(_) => {
            // Return updated message
            match state.repository.get_message(id).await {
                Ok(Some(message)) => {
                    let response = MessageResponse::from(message);
                    Json(response).into_response()
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Message not found" })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Mark message as read
async fn mark_as_read(
    State(state): State<Arc<VoicemailApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.update_message_status(id, VoicemailStatus::Read).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Mark message as saved
async fn mark_as_saved(
    State(state): State<Arc<VoicemailApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.update_message_status(id, VoicemailStatus::Saved).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get message count statistics
async fn get_message_count(
    State(state): State<Arc<VoicemailApiState>>,
    Path(mailbox_id): Path<String>,
) -> Response {
    // Count total messages
    let total = match state.repository.count_messages(&mailbox_id, None).await {
        Ok(count) => count,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    // Count new messages
    let new = match state
        .repository
        .count_messages(&mailbox_id, Some(VoicemailStatus::New))
        .await
    {
        Ok(count) => count,
        Err(_) => 0,
    };

    // Count read messages
    let read = match state
        .repository
        .count_messages(&mailbox_id, Some(VoicemailStatus::Read))
        .await
    {
        Ok(count) => count,
        Err(_) => 0,
    };

    // Count saved messages
    let saved = match state
        .repository
        .count_messages(&mailbox_id, Some(VoicemailStatus::Saved))
        .await
    {
        Ok(count) => count,
        Err(_) => 0,
    };

    let response = MessageCountResponse {
        total,
        new,
        read,
        saved,
    };

    Json(response).into_response()
}

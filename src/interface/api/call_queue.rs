/// Call Queue management REST API
use crate::domain::call_queue::{
    AgentStatus, CallQueue, CallQueueRepository, OverflowAction, QueueMember, QueueStrategy,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Call Queue API state
pub struct CallQueueApiState {
    pub repository: Arc<dyn CallQueueRepository>,
}

/// Create call queue router
pub fn call_queue_router(state: Arc<CallQueueApiState>) -> Router {
    Router::new()
        .route("/queues", post(create_queue))
        .route("/queues", get(list_queues))
        .route("/queues/:id", get(get_queue))
        .route("/queues/:id", put(update_queue))
        .route("/queues/:id", delete(delete_queue))
        .route("/queues/extension/:extension", get(get_queue_by_extension))
        .route("/queues/:id/members", post(add_member))
        .route("/queues/:id/members", get(list_members))
        .route("/queues/:id/members/:member_id", delete(remove_member))
        .route(
            "/queues/:id/members/:member_id",
            put(update_member_status),
        )
        .route(
            "/queues/:id/members/:member_id/pause",
            post(pause_member),
        )
        .route(
            "/queues/:id/members/:member_id/unpause",
            post(unpause_member),
        )
        .with_state(state)
}

/// Request to create a call queue
#[derive(Debug, Deserialize)]
struct CreateQueueRequest {
    name: String,
    extension: String,
    strategy: String, // QueueStrategy
    max_wait_time_secs: Option<u64>,
    max_queue_size: Option<usize>,
    ring_timeout_secs: Option<u64>,
    announce_position: Option<bool>,
    music_on_hold: Option<String>,
}

/// Request to update a call queue
#[derive(Debug, Deserialize)]
struct UpdateQueueRequest {
    name: Option<String>,
    strategy: Option<String>,
    max_wait_time_secs: Option<u64>,
    max_queue_size: Option<usize>,
    ring_timeout_secs: Option<u64>,
    announce_position: Option<bool>,
    music_on_hold: Option<String>,
}

/// Response for call queue operations
#[derive(Debug, Serialize)]
struct QueueResponse {
    id: String,
    name: String,
    extension: String,
    strategy: String,
    max_wait_time_secs: u64,
    max_queue_size: usize,
    ring_timeout_secs: u64,
    announce_position: bool,
    music_on_hold: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<CallQueue> for QueueResponse {
    fn from(queue: CallQueue) -> Self {
        Self {
            id: queue.id.to_string(),
            name: queue.name,
            extension: queue.extension,
            strategy: format!("{:?}", queue.strategy),
            max_wait_time_secs: queue.max_wait_time.as_secs(),
            max_queue_size: queue.max_queue_size,
            ring_timeout_secs: queue.ring_timeout.as_secs(),
            announce_position: queue.announce_position,
            music_on_hold: queue.music_on_hold,
            created_at: queue.created_at.to_rfc3339(),
            updated_at: queue.updated_at.to_rfc3339(),
        }
    }
}

/// Request to add a member to a queue
#[derive(Debug, Deserialize)]
struct AddMemberRequest {
    user_id: i32,
    username: String,
    extension: String,
}

/// Request to pause a member
#[derive(Debug, Deserialize)]
struct PauseMemberRequest {
    reason: Option<String>,
}

/// Request to update member status
#[derive(Debug, Deserialize)]
struct UpdateMemberStatusRequest {
    status: String, // AgentStatus
}

/// Response for queue member operations
#[derive(Debug, Serialize)]
struct MemberResponse {
    id: String,
    user_id: i32,
    username: String,
    extension: String,
    status: String,
    paused: bool,
    paused_reason: Option<String>,
    total_calls: u64,
    answered_calls: u64,
    missed_calls: u64,
    joined_at: String,
}

impl From<QueueMember> for MemberResponse {
    fn from(member: QueueMember) -> Self {
        Self {
            id: member.id.to_string(),
            user_id: member.user_id,
            username: member.username,
            extension: member.extension,
            status: format!("{:?}", member.status),
            paused: member.paused,
            paused_reason: member.paused_reason,
            total_calls: member.total_calls,
            answered_calls: member.answered_calls,
            missed_calls: member.missed_calls,
            joined_at: member.joined_at.to_rfc3339(),
        }
    }
}

/// Parse QueueStrategy from string
fn parse_strategy(s: &str) -> Result<QueueStrategy, String> {
    match s {
        "RingAll" => Ok(QueueStrategy::RingAll),
        "Linear" => Ok(QueueStrategy::Linear),
        "LeastRecent" => Ok(QueueStrategy::LeastRecent),
        "FewestCalls" => Ok(QueueStrategy::FewestCalls),
        "LeastTalkTime" => Ok(QueueStrategy::LeastTalkTime),
        "Random" => Ok(QueueStrategy::Random),
        "RoundRobin" => Ok(QueueStrategy::RoundRobin),
        _ => Err(format!("Invalid strategy: {}", s)),
    }
}

/// Parse AgentStatus from string
fn parse_agent_status(s: &str) -> Result<AgentStatus, String> {
    match s {
        "Available" => Ok(AgentStatus::Available),
        "Busy" => Ok(AgentStatus::Busy),
        "AfterCallWork" => Ok(AgentStatus::AfterCallWork),
        "Paused" => Ok(AgentStatus::Paused),
        "LoggedOut" => Ok(AgentStatus::LoggedOut),
        _ => Err(format!("Invalid status: {}", s)),
    }
}

/// Create a new call queue
async fn create_queue(
    State(state): State<Arc<CallQueueApiState>>,
    Json(req): Json<CreateQueueRequest>,
) -> Response {
    let strategy = match parse_strategy(&req.strategy) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let mut queue = CallQueue::new(req.name, req.extension, strategy);

    if let Some(max_wait_time) = req.max_wait_time_secs {
        queue.max_wait_time = Duration::from_secs(max_wait_time);
    }
    if let Some(max_queue_size) = req.max_queue_size {
        queue.max_queue_size = max_queue_size;
    }
    if let Some(ring_timeout) = req.ring_timeout_secs {
        queue.ring_timeout = Duration::from_secs(ring_timeout);
    }
    if let Some(announce_position) = req.announce_position {
        queue.announce_position = announce_position;
    }
    if let Some(music_on_hold) = req.music_on_hold {
        queue.music_on_hold = Some(music_on_hold);
    }

    match state.repository.create_queue(queue).await {
        Ok(queue) => {
            let response = QueueResponse::from(queue);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get queue by ID
async fn get_queue(
    State(state): State<Arc<CallQueueApiState>>,
    Path(id): Path<String>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_queue(queue_id).await {
        Ok(Some(queue)) => {
            let response = QueueResponse::from(queue);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Queue not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get queue by extension
async fn get_queue_by_extension(
    State(state): State<Arc<CallQueueApiState>>,
    Path(extension): Path<String>,
) -> Response {
    match state.repository.get_queue_by_extension(&extension).await {
        Ok(Some(queue)) => {
            let response = QueueResponse::from(queue);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Queue not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List all queues
async fn list_queues(State(state): State<Arc<CallQueueApiState>>) -> Response {
    match state.repository.list_queues().await {
        Ok(queues) => {
            let responses: Vec<QueueResponse> = queues.into_iter().map(|q| q.into()).collect();
            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update a queue
async fn update_queue(
    State(state): State<Arc<CallQueueApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateQueueRequest>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut queue = match state.repository.get_queue(queue_id).await {
        Ok(Some(queue)) => queue,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Queue not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    if let Some(name) = req.name {
        queue.name = name;
    }
    if let Some(strategy_str) = req.strategy {
        match parse_strategy(&strategy_str) {
            Ok(strategy) => queue.strategy = strategy,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response()
            }
        }
    }
    if let Some(max_wait_time) = req.max_wait_time_secs {
        queue.max_wait_time = Duration::from_secs(max_wait_time);
    }
    if let Some(max_queue_size) = req.max_queue_size {
        queue.max_queue_size = max_queue_size;
    }
    if let Some(ring_timeout) = req.ring_timeout_secs {
        queue.ring_timeout = Duration::from_secs(ring_timeout);
    }
    if let Some(announce_position) = req.announce_position {
        queue.announce_position = announce_position;
    }
    if let Some(music_on_hold) = req.music_on_hold {
        queue.music_on_hold = Some(music_on_hold);
    }

    queue.updated_at = chrono::Utc::now();

    match state.repository.update_queue(&queue).await {
        Ok(_) => {
            let response = QueueResponse::from(queue);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Delete a queue
async fn delete_queue(
    State(state): State<Arc<CallQueueApiState>>,
    Path(id): Path<String>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.delete_queue(queue_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Add a member to a queue
async fn add_member(
    State(state): State<Arc<CallQueueApiState>>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let member = QueueMember::new(req.user_id, req.username, req.extension);

    match state.repository.add_member(queue_id, member.clone()).await {
        Ok(_) => {
            let response = MemberResponse::from(member);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List members of a queue
async fn list_members(
    State(state): State<Arc<CallQueueApiState>>,
    Path(id): Path<String>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_members(queue_id).await {
        Ok(members) => {
            let responses: Vec<MemberResponse> = members.into_iter().map(|m| m.into()).collect();
            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Remove a member from a queue
async fn remove_member(
    State(state): State<Arc<CallQueueApiState>>,
    Path((id, member_id)): Path<(String, String)>,
) -> Response {
    let queue_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid queue UUID" })),
            )
                .into_response()
        }
    };

    let member_uuid = match Uuid::parse_str(&member_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid member UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.remove_member(queue_id, member_uuid).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update member status
async fn update_member_status(
    State(state): State<Arc<CallQueueApiState>>,
    Path((_id, member_id)): Path<(String, String)>,
    Json(req): Json<UpdateMemberStatusRequest>,
) -> Response {
    let member_uuid = match Uuid::parse_str(&member_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid member UUID" })),
            )
                .into_response()
        }
    };

    let mut member = match state.repository.get_member(member_uuid).await {
        Ok(Some(member)) => member,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Member not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    match parse_agent_status(&req.status) {
        Ok(status) => member.status = status,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }

    match state.repository.update_member(&member).await {
        Ok(_) => {
            let response = MemberResponse::from(member);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Pause a member
async fn pause_member(
    State(state): State<Arc<CallQueueApiState>>,
    Path((_id, member_id)): Path<(String, String)>,
    Json(req): Json<PauseMemberRequest>,
) -> Response {
    let member_uuid = match Uuid::parse_str(&member_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid member UUID" })),
            )
                .into_response()
        }
    };

    let mut member = match state.repository.get_member(member_uuid).await {
        Ok(Some(member)) => member,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Member not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    member.pause(req.reason);

    match state.repository.update_member(&member).await {
        Ok(_) => {
            let response = MemberResponse::from(member);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Unpause a member
async fn unpause_member(
    State(state): State<Arc<CallQueueApiState>>,
    Path((_id, member_id)): Path<(String, String)>,
) -> Response {
    let member_uuid = match Uuid::parse_str(&member_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid member UUID" })),
            )
                .into_response()
        }
    };

    let mut member = match state.repository.get_member(member_uuid).await {
        Ok(Some(member)) => member,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Member not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    member.unpause();

    match state.repository.update_member(&member).await {
        Ok(_) => {
            let response = MemberResponse::from(member);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

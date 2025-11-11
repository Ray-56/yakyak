/// Conference management REST API handlers
use super::user_handler::AppState;
use crate::domain::conference::ParticipantRole;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

/// Request to create a conference
#[derive(Debug, Deserialize)]
pub struct CreateConferenceRequest {
    pub name: String,
    pub pin: Option<String>,
    pub max_participants: Option<usize>,
}

/// Conference response
#[derive(Debug, Serialize)]
pub struct ConferenceResponse {
    pub id: String,
    pub name: String,
    pub has_pin: bool,
    pub max_participants: usize,
    pub participant_count: usize,
}

/// Request to join a conference
#[derive(Debug, Deserialize)]
pub struct JoinConferenceRequest {
    pub call_id: String,
    pub name: String,
    pub pin: Option<String>,
    pub role: Option<String>,
}

/// Join response
#[derive(Debug, Serialize)]
pub struct JoinConferenceResponse {
    pub room_id: String,
    pub participant_id: String,
    pub success: bool,
}

/// Mute request
#[derive(Debug, Deserialize)]
pub struct MuteRequest {
    pub call_id: String,
}

/// Create a new conference room
pub async fn create_conference_room(
    State(state): State<AppState>,
    Json(req): Json<CreateConferenceRequest>,
) -> impl IntoResponse {
    info!("Creating conference room: {}", req.name);

    let Some(ref manager) = state.conference_manager else {
        warn!("Conference manager not available");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Conference service not available" })),
        )
            .into_response();
    };

    let max_participants = req.max_participants.unwrap_or(50);

    match manager
        .create_room(req.name.clone(), req.pin.clone(), max_participants)
        .await
    {
        Ok(room_id) => {
            let response = ConferenceResponse {
                id: room_id.to_string(),
                name: req.name,
                has_pin: req.pin.is_some(),
                max_participants,
                participant_count: 0,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            warn!("Failed to create conference: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Join a conference room
pub async fn join_conference_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(req): Json<JoinConferenceRequest>,
) -> impl IntoResponse {
    info!("Join conference request for room {}", room_id);

    let Some(ref manager) = state.conference_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Conference service not available" })),
        )
            .into_response();
    };

    let room_uuid = match Uuid::parse_str(&room_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid room ID format" })),
            )
                .into_response();
        }
    };

    let role = match req.role.as_deref() {
        Some("moderator") => ParticipantRole::Moderator,
        Some("presenter") => ParticipantRole::Presenter,
        Some("listener") => ParticipantRole::Listener,
        _ => ParticipantRole::Attendee,
    };

    match manager
        .join_conference(room_uuid, req.call_id, req.name, role, req.pin)
        .await
    {
        Ok(participant_id) => {
            let response = JoinConferenceResponse {
                room_id: room_id.clone(),
                participant_id: participant_id.to_string(),
                success: true,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            warn!("Failed to join conference: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Leave a conference room
pub async fn leave_conference_room(
    State(state): State<AppState>,
    Json(req): Json<MuteRequest>,
) -> impl IntoResponse {
    info!("Leave conference request for call {}", req.call_id);

    let Some(ref manager) = state.conference_manager else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    match manager.leave_conference(&req.call_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!("Failed to leave conference: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Mute participant in conference
pub async fn mute_conference_participant(
    State(state): State<AppState>,
    Json(req): Json<MuteRequest>,
) -> impl IntoResponse {
    info!("Mute participant request for call {}", req.call_id);

    let Some(ref manager) = state.conference_manager else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    match manager.mute_participant(&req.call_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!("Failed to mute participant: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Unmute participant in conference
pub async fn unmute_conference_participant(
    State(state): State<AppState>,
    Json(req): Json<MuteRequest>,
) -> impl IntoResponse {
    info!("Unmute participant request for call {}", req.call_id);

    let Some(ref manager) = state.conference_manager else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    match manager.unmute_participant(&req.call_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!("Failed to unmute participant: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// End a conference
pub async fn end_conference(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    info!("End conference request for room {}", room_id);

    let Some(ref manager) = state.conference_manager else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    let room_uuid = match Uuid::parse_str(&room_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid room ID format" })),
            )
                .into_response();
        }
    };

    match manager.end_conference(room_uuid).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            warn!("Failed to end conference: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// Get conference details
pub async fn get_conference_details(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    let Some(ref manager) = state.conference_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Conference service not available" })),
        )
            .into_response();
    };

    let room_uuid = match Uuid::parse_str(&room_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid room ID format" })),
            )
                .into_response();
        }
    };

    match manager.get_room(room_uuid).await {
        Ok(room) => {
            let response = ConferenceResponse {
                id: room.id.to_string(),
                name: room.name.clone(),
                has_pin: room.pin.is_some(),
                max_participants: room.max_participants,
                participant_count: room.participant_count(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            warn!("Failed to get conference: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

/// List active conferences
pub async fn list_active_conferences(State(state): State<AppState>) -> impl IntoResponse {
    let Some(ref manager) = state.conference_manager else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Conference service not available" })),
        )
            .into_response();
    };

    let rooms = manager.list_active_conferences().await;
    let responses: Vec<ConferenceResponse> = rooms
        .into_iter()
        .map(|room| ConferenceResponse {
            id: room.id.to_string(),
            name: room.name.clone(),
            has_pin: room.pin.is_some(),
            max_participants: room.max_participants,
            participant_count: room.participant_count(),
        })
        .collect();

    (StatusCode::OK, Json(responses)).into_response()
}

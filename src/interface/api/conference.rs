/// Conference management REST API
use crate::domain::conference::{
    ConferenceRepository, ConferenceRoom, ConferenceState, Participant, ParticipantRole,
};
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

/// Conference API state
pub struct ConferenceApiState {
    pub repository: Arc<dyn ConferenceRepository>,
}

/// Create conference router
pub fn conference_router(state: Arc<ConferenceApiState>) -> Router {
    Router::new()
        .route("/conferences", post(create_conference))
        .route("/conferences", get(list_conferences))
        .route("/conferences/:id", get(get_conference))
        .route("/conferences/:id", put(update_conference))
        .route("/conferences/:id", delete(delete_conference))
        .route("/conferences/:id/start", post(start_conference))
        .route("/conferences/:id/end", post(end_conference))
        .route("/conferences/:id/lock", post(lock_conference))
        .route("/conferences/:id/unlock", post(unlock_conference))
        .route("/conferences/:id/participants", post(add_participant))
        .route("/conferences/:id/participants", get(list_participants))
        .route(
            "/conferences/:id/participants/:participant_id",
            delete(remove_participant),
        )
        .route(
            "/conferences/:id/participants/:participant_id/mute",
            post(mute_participant),
        )
        .route(
            "/conferences/:id/participants/:participant_id/unmute",
            post(unmute_participant),
        )
        .route(
            "/conferences/:id/participants/:participant_id/role",
            put(update_participant_role),
        )
        .with_state(state)
}

/// Request to create a conference
#[derive(Debug, Deserialize)]
struct CreateConferenceRequest {
    name: String,
    pin: Option<String>,
    max_participants: Option<usize>,
    recording_enabled: Option<bool>,
}

/// Response for conference operations
#[derive(Debug, Serialize)]
struct ConferenceResponse {
    id: Uuid,
    name: String,
    pin: Option<String>,
    max_participants: usize,
    state: String,
    moderator_id: Option<Uuid>,
    recording_enabled: bool,
    recording_file: Option<String>,
    participant_count: usize,
    created_at: String,
    started_at: Option<String>,
    ended_at: Option<String>,
}

impl From<ConferenceRoom> for ConferenceResponse {
    fn from(room: ConferenceRoom) -> Self {
        Self {
            id: room.id,
            name: room.name,
            pin: room.pin,
            max_participants: room.max_participants,
            state: format!("{:?}", room.state),
            moderator_id: room.moderator_id,
            recording_enabled: room.recording_enabled,
            recording_file: room.recording_file,
            participant_count: room.participants.len(),
            created_at: room.created_at.to_rfc3339(),
            started_at: room.started_at.map(|dt| dt.to_rfc3339()),
            ended_at: room.ended_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Participant response
#[derive(Debug, Serialize)]
struct ParticipantResponse {
    id: Uuid,
    name: String,
    call_id: String,
    role: String,
    state: String,
    is_muted: bool,
    volume: f32,
    joined_at: String,
    left_at: Option<String>,
}

impl From<Participant> for ParticipantResponse {
    fn from(participant: Participant) -> Self {
        Self {
            id: participant.id,
            name: participant.name,
            call_id: participant.call_id,
            role: format!("{:?}", participant.role),
            state: format!("{:?}", participant.state),
            is_muted: participant.is_muted,
            volume: participant.volume,
            joined_at: participant.joined_at.to_rfc3339(),
            left_at: participant.left_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Query parameters for listing conferences
#[derive(Debug, Deserialize)]
struct ListConferencesQuery {
    state: Option<String>,
}

/// Add participant request
#[derive(Debug, Deserialize)]
struct AddParticipantRequest {
    name: String,
    call_id: String,
    role: Option<String>,
}

/// Update role request
#[derive(Debug, Deserialize)]
struct UpdateRoleRequest {
    role: String,
}

/// Create a new conference
async fn create_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Json(req): Json<CreateConferenceRequest>,
) -> Response {
    let max_participants = req.max_participants.unwrap_or(50);
    let mut room = ConferenceRoom::new(req.name, req.pin, max_participants);

    if let Some(recording) = req.recording_enabled {
        room.recording_enabled = recording;
    }

    match state.repository.create_room(room).await {
        Ok(room) => {
            let response = ConferenceResponse::from(room);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get conference by ID
async fn get_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(room)) => {
            let response = ConferenceResponse::from(room);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List conferences
async fn list_conferences(
    State(state): State<Arc<ConferenceApiState>>,
    Query(query): Query<ListConferencesQuery>,
) -> Response {
    let state_filter = query.state.and_then(|s| match s.as_str() {
        "waiting" => Some(ConferenceState::Waiting),
        "active" => Some(ConferenceState::Active),
        "locked" => Some(ConferenceState::Locked),
        "ended" => Some(ConferenceState::Ended),
        _ => None,
    });

    match state.repository.list_rooms(state_filter).await {
        Ok(rooms) => {
            let response: Vec<ConferenceResponse> =
                rooms.into_iter().map(ConferenceResponse::from).collect();
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update conference
async fn update_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateConferenceRequest>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(mut room)) => {
            room.name = req.name;
            room.pin = req.pin;
            if let Some(max) = req.max_participants {
                room.max_participants = max;
            }
            if let Some(recording) = req.recording_enabled {
                room.recording_enabled = recording;
            }

            match state.repository.update_room(&room).await {
                Ok(_) => {
                    let response = ConferenceResponse::from(room);
                    Json(response).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Delete conference
async fn delete_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.delete_room(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Start conference
async fn start_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(mut room)) => {
            if let Err(e) = room.start() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response();
            }

            match state.repository.update_room(&room).await {
                Ok(_) => {
                    let response = ConferenceResponse::from(room);
                    Json(response).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// End conference
async fn end_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(mut room)) => {
            room.end();

            match state.repository.update_room(&room).await {
                Ok(_) => {
                    let response = ConferenceResponse::from(room);
                    Json(response).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Lock conference
async fn lock_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(mut room)) => {
            if let Err(e) = room.lock() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response();
            }

            match state.repository.update_room(&room).await {
                Ok(_) => {
                    let response = ConferenceResponse::from(room);
                    Json(response).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Unlock conference
async fn unlock_conference(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_room(id).await {
        Ok(Some(mut room)) => {
            if let Err(e) = room.unlock() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response();
            }

            match state.repository.update_room(&room).await {
                Ok(_) => {
                    let response = ConferenceResponse::from(room);
                    Json(response).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Conference not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Add participant to conference
async fn add_participant(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddParticipantRequest>,
) -> Response {
    let role = match req.role.as_deref() {
        Some("moderator") => ParticipantRole::Moderator,
        Some("presenter") => ParticipantRole::Presenter,
        Some("listener") => ParticipantRole::Listener,
        _ => ParticipantRole::Attendee,
    };

    let participant = Participant::new(req.name, req.call_id, role);

    match state.repository.add_participant(id, participant.clone()).await {
        Ok(_) => {
            let response = ParticipantResponse::from(participant);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List participants in conference
async fn list_participants(
    State(state): State<Arc<ConferenceApiState>>,
    Path(id): Path<Uuid>,
) -> Response {
    match state.repository.get_participants(id).await {
        Ok(participants) => {
            let response: Vec<ParticipantResponse> = participants
                .into_iter()
                .map(ParticipantResponse::from)
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

/// Remove participant from conference
async fn remove_participant(
    State(state): State<Arc<ConferenceApiState>>,
    Path((room_id, participant_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match state
        .repository
        .remove_participant(room_id, participant_id)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Mute participant
async fn mute_participant(
    State(state): State<Arc<ConferenceApiState>>,
    Path((room_id, participant_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match state.repository.get_participants(room_id).await {
        Ok(participants) => {
            if let Some(mut participant) = participants.into_iter().find(|p| p.id == participant_id)
            {
                participant.mute();

                match state
                    .repository
                    .update_participant(room_id, participant_id, &participant)
                    .await
                {
                    Ok(_) => {
                        let response = ParticipantResponse::from(participant);
                        Json(response).into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Participant not found" })),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Unmute participant
async fn unmute_participant(
    State(state): State<Arc<ConferenceApiState>>,
    Path((room_id, participant_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match state.repository.get_participants(room_id).await {
        Ok(participants) => {
            if let Some(mut participant) = participants.into_iter().find(|p| p.id == participant_id)
            {
                participant.unmute();

                match state
                    .repository
                    .update_participant(room_id, participant_id, &participant)
                    .await
                {
                    Ok(_) => {
                        let response = ParticipantResponse::from(participant);
                        Json(response).into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Participant not found" })),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update participant role
async fn update_participant_role(
    State(state): State<Arc<ConferenceApiState>>,
    Path((room_id, participant_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Response {
    let role = match req.role.as_str() {
        "moderator" => ParticipantRole::Moderator,
        "presenter" => ParticipantRole::Presenter,
        "listener" => ParticipantRole::Listener,
        _ => ParticipantRole::Attendee,
    };

    match state.repository.get_participants(room_id).await {
        Ok(participants) => {
            if let Some(mut participant) = participants.into_iter().find(|p| p.id == participant_id)
            {
                participant.role = role;

                match state
                    .repository
                    .update_participant(room_id, participant_id, &participant)
                    .await
                {
                    Ok(_) => {
                        let response = ParticipantResponse::from(participant);
                        Json(response).into_response()
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e })),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Participant not found" })),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

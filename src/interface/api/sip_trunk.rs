/// SIP Trunk management REST API
use crate::domain::sip_trunk::{SipTrunk, SipTrunkRepository, TrunkDirection, TrunkType};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// SIP Trunk API state
pub struct SipTrunkApiState {
    pub repository: Arc<dyn SipTrunkRepository>,
}

/// Create SIP trunk router
pub fn sip_trunk_router(state: Arc<SipTrunkApiState>) -> Router {
    Router::new()
        .route("/trunks", post(create_trunk))
        .route("/trunks", get(list_trunks))
        .route("/trunks/:id", get(get_trunk))
        .route("/trunks/:id", put(update_trunk))
        .route("/trunks/:id", delete(delete_trunk))
        .route("/trunks/name/:name", get(get_trunk_by_name))
        .route("/trunks/:id/register", post(trigger_registration))
        .route("/trunks/:id/statistics", get(get_trunk_statistics))
        .with_state(state)
}

/// Request to create a SIP trunk
#[derive(Debug, Deserialize)]
struct CreateTrunkRequest {
    name: String,
    provider_name: String,
    trunk_type: String,
    sip_server: String,
    sip_port: Option<u16>,
    direction: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

/// Request to update a SIP trunk
#[derive(Debug, Deserialize)]
struct UpdateTrunkRequest {
    provider_name: Option<String>,
    sip_server: Option<String>,
    sip_port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    enabled: Option<bool>,
}

/// Response for SIP trunk operations
#[derive(Debug, Serialize)]
struct TrunkResponse {
    id: String,
    name: String,
    provider_name: String,
    trunk_type: String,
    sip_server: String,
    sip_port: u16,
    direction: String,
    register_enabled: bool,
    registered: bool,
    enabled: bool,
    created_at: String,
    updated_at: String,
}

impl From<SipTrunk> for TrunkResponse {
    fn from(trunk: SipTrunk) -> Self {
        Self {
            id: trunk.id.to_string(),
            name: trunk.name,
            provider_name: trunk.provider_name,
            trunk_type: format!("{:?}", trunk.trunk_type),
            sip_server: trunk.sip_server,
            sip_port: trunk.sip_port,
            direction: format!("{:?}", trunk.direction),
            register_enabled: trunk.register_enabled,
            registered: trunk.registered,
            enabled: trunk.enabled,
            created_at: trunk.created_at.to_rfc3339(),
            updated_at: trunk.updated_at.to_rfc3339(),
        }
    }
}

/// Parse TrunkType from string
fn parse_trunk_type(s: &str) -> Result<TrunkType, String> {
    match s {
        "Register" => Ok(TrunkType::Register),
        "IpBased" => Ok(TrunkType::IpBased),
        "Peer" => Ok(TrunkType::Peer),
        _ => Err(format!("Invalid trunk type: {}", s)),
    }
}

/// Parse TrunkDirection from string
fn parse_direction(s: &str) -> Result<TrunkDirection, String> {
    match s {
        "Inbound" => Ok(TrunkDirection::Inbound),
        "Outbound" => Ok(TrunkDirection::Outbound),
        "Bidirectional" => Ok(TrunkDirection::Bidirectional),
        _ => Err(format!("Invalid direction: {}", s)),
    }
}

/// Create a new SIP trunk
async fn create_trunk(
    State(state): State<Arc<SipTrunkApiState>>,
    Json(req): Json<CreateTrunkRequest>,
) -> Response {
    let trunk_type = match parse_trunk_type(&req.trunk_type) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let mut trunk = SipTrunk::new(req.name, req.provider_name, trunk_type)
        .with_server(req.sip_server, req.sip_port.unwrap_or(5060));

    if let Some(dir_str) = req.direction {
        match parse_direction(&dir_str) {
            Ok(dir) => trunk.direction = dir,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response()
            }
        }
    }

    if let Some(username) = req.username {
        if let Some(password) = req.password {
            trunk = trunk.with_credentials(username, password);
        }
    }

    match state.repository.create_trunk(trunk).await {
        Ok(trunk) => {
            let response = TrunkResponse::from(trunk);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get trunk by ID
async fn get_trunk(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(id): Path<String>,
) -> Response {
    let trunk_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_trunk(trunk_id).await {
        Ok(Some(trunk)) => {
            let response = TrunkResponse::from(trunk);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Trunk not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get trunk by name
async fn get_trunk_by_name(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(name): Path<String>,
) -> Response {
    match state.repository.get_trunk_by_name(&name).await {
        Ok(Some(trunk)) => {
            let response = TrunkResponse::from(trunk);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Trunk not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List all trunks
async fn list_trunks(State(state): State<Arc<SipTrunkApiState>>) -> Response {
    match state.repository.list_trunks(false).await {
        Ok(trunks) => {
            let responses: Vec<TrunkResponse> = trunks.into_iter().map(|t| t.into()).collect();
            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update a trunk
async fn update_trunk(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTrunkRequest>,
) -> Response {
    let trunk_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut trunk = match state.repository.get_trunk(trunk_id).await {
        Ok(Some(trunk)) => trunk,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Trunk not found" })),
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

    if let Some(provider_name) = req.provider_name {
        trunk.provider_name = provider_name;
    }
    if let Some(sip_server) = req.sip_server {
        trunk.sip_server = sip_server;
    }
    if let Some(sip_port) = req.sip_port {
        trunk.sip_port = sip_port;
    }
    if let Some(username) = req.username {
        trunk.username = Some(username);
    }
    if let Some(password) = req.password {
        trunk.password = Some(password);
    }
    if let Some(enabled) = req.enabled {
        trunk.enabled = enabled;
    }

    trunk.updated_at = Utc::now();

    match state.repository.update_trunk(&trunk).await {
        Ok(_) => {
            let response = TrunkResponse::from(trunk);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Delete a trunk
async fn delete_trunk(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(id): Path<String>,
) -> Response {
    let trunk_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.delete_trunk(trunk_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Trigger registration for a trunk
async fn trigger_registration(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(id): Path<String>,
) -> Response {
    let trunk_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut trunk = match state.repository.get_trunk(trunk_id).await {
        Ok(Some(trunk)) => trunk,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Trunk not found" })),
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

    trunk.mark_registered();

    match state.repository.update_trunk(&trunk).await {
        Ok(_) => Json(serde_json::json!({ "message": "Registration triggered" })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get trunk statistics
async fn get_trunk_statistics(
    State(state): State<Arc<SipTrunkApiState>>,
    Path(id): Path<String>,
) -> Response {
    let trunk_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_statistics(trunk_id).await {
        Ok(Some(stats)) => Json(serde_json::json!({
            "trunk_id": stats.trunk_id.to_string(),
            "current_calls": stats.current_calls,
            "total_calls": stats.total_calls,
            "successful_calls": stats.successful_calls,
            "failed_calls": stats.failed_calls,
            "success_rate": stats.success_rate(),
            "average_call_duration": stats.average_call_duration,
            "total_minutes": stats.total_minutes,
        }))
        .into_response(),
        Ok(None) => Json(serde_json::json!({
            "trunk_id": id,
            "current_calls": 0,
            "total_calls": 0,
            "successful_calls": 0,
            "failed_calls": 0,
            "success_rate": 0.0,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

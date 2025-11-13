//! User API handlers

use super::user_dto::{
    ApiResponse, ChangePasswordRequest, CreateUserRequest, DeleteResponse, UpdateUserRequest,
    UserListResponse, UserResponse,
};
use crate::domain::user::UserRepository;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

use crate::domain::cdr::CdrRepository;
use crate::infrastructure::protocols::sip::{CallRouter, Registrar};
use super::ws_handler::EventBroadcaster;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub user_repository: Arc<dyn UserRepository>,
    pub cdr_repository: Option<Arc<dyn CdrRepository>>,
    pub call_router: Option<Arc<CallRouter>>,
    pub registrar: Option<Arc<Registrar>>,
    pub event_broadcaster: Option<Arc<EventBroadcaster>>,
    pub conference_repository: Option<Arc<dyn crate::domain::conference::ConferenceRepository>>,
    pub conference_manager: Option<Arc<crate::domain::conference_manager::ConferenceManager>>,
}

/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub realm: Option<String>,
}

fn default_limit() -> i64 {
    50
}

/// Create a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserResponse>>), StatusCode> {
    info!("API: Creating user {}", req.username);

    let create_data = req.into();

    match state.user_repository.create(create_data).await {
        Ok(user) => {
            info!("API: Created user {} (ID: {})", user.username, user.id);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success(user.into())),
            ))
        }
        Err(e) => {
            error!("API: Failed to create user: {}", e);
            Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(e.to_string())),
            ))
        }
    }
}

/// Get user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    info!("API: Getting user ID: {}", id);

    match state.user_repository.find_by_id(id).await {
        Ok(Some(user)) => Ok(Json(ApiResponse::success(user.into()))),
        Ok(None) => Ok(Json(ApiResponse::error(format!("User {} not found", id)))),
        Err(e) => {
            error!("API: Failed to get user: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get user by username
pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    info!("API: Getting user by username: {}", username);

    match state.user_repository.find_by_username(&username).await {
        Ok(Some(user)) => Ok(Json(ApiResponse::success(user.into()))),
        Ok(None) => Ok(Json(ApiResponse::error(format!(
            "User {} not found",
            username
        )))),
        Err(e) => {
            error!("API: Failed to get user: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List users
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<UserListResponse>>, StatusCode> {
    info!(
        "API: Listing users (limit: {}, offset: {}, realm: {:?})",
        query.limit, query.offset, query.realm
    );

    // Get users
    let users_result = if let Some(realm) = &query.realm {
        state
            .user_repository
            .list_by_realm(realm, query.limit, query.offset)
            .await
    } else {
        state
            .user_repository
            .list(query.limit, query.offset)
            .await
    };

    // Get total count
    let count_result = if let Some(realm) = &query.realm {
        state.user_repository.count_by_realm(realm).await
    } else {
        state.user_repository.count().await
    };

    match (users_result, count_result) {
        (Ok(users), Ok(total)) => {
            let response = UserListResponse {
                users: users.into_iter().map(|u| u.into()).collect(),
                total,
                limit: query.limit,
                offset: query.offset,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("API: Failed to list users: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Update user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    info!("API: Updating user ID: {}", id);

    let update_data = req.into();

    match state.user_repository.update(id, update_data).await {
        Ok(user) => {
            info!("API: Updated user {} (ID: {})", user.username, user.id);
            Ok(Json(ApiResponse::success(user.into())))
        }
        Err(e) => {
            error!("API: Failed to update user: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete user
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<DeleteResponse>>, StatusCode> {
    info!("API: Deleting user ID: {}", id);

    match state.user_repository.delete(id).await {
        Ok(()) => {
            info!("API: Deleted user ID: {}", id);
            Ok(Json(ApiResponse::success(DeleteResponse {
                id,
                deleted: true,
            })))
        }
        Err(e) => {
            error!("API: Failed to delete user: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Change user password
pub async fn change_password(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    info!("API: Changing password for user ID: {}", id);

    let change_data = req.into();

    match state.user_repository.change_password(id, change_data).await {
        Ok(()) => {
            info!("API: Changed password for user ID: {}", id);
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("API: Failed to change password: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Enable/disable user
pub async fn set_enabled(
    State(state): State<AppState>,
    Path((id, enabled)): Path<(i32, bool)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    info!("API: Setting enabled={} for user ID: {}", enabled, id);

    match state.user_repository.set_enabled(id, enabled).await {
        Ok(()) => {
            info!("API: Set enabled={} for user ID: {}", enabled, id);
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("API: Failed to set enabled: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("OK"))
}

/// Get online users (registered users)
pub async fn get_online_users(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<RegistrationInfo>>>, StatusCode> {
    info!("API: Getting online users");

    let registrar = match &state.registrar {
        Some(reg) => reg,
        None => {
            error!("Registrar not available");
            return Ok(Json(ApiResponse::error(
                "Registrar not available".to_string(),
            )));
        }
    };

    let registrations = registrar.get_all_registrations().await;
    let registration_info: Vec<RegistrationInfo> = registrations
        .into_iter()
        .map(|reg| RegistrationInfo {
            aor: reg.aor.clone(),
            bindings: reg
                .bindings
                .into_iter()
                .map(|b| BindingInfo {
                    contact: b.contact,
                    expires_at: b.expires_at.to_rfc3339(),
                    user_agent: b.user_agent,
                })
                .collect(),
        })
        .collect();

    Ok(Json(ApiResponse::success(registration_info)))
}

/// Get user registration status by username
pub async fn get_user_registration_status(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ApiResponse<UserRegistrationStatus>>, StatusCode> {
    info!("API: Getting registration status for user: {}", username);

    let registrar = match &state.registrar {
        Some(reg) => reg,
        None => {
            error!("Registrar not available");
            return Ok(Json(ApiResponse::error(
                "Registrar not available".to_string(),
            )));
        }
    };

    // Construct AoR from username (assuming default domain)
    // In production, you might want to query the user's actual realm
    let aor = format!("sip:{}@example.com", username);

    let is_registered = registrar.is_registered(&aor).await;
    let bindings = if is_registered {
        registrar
            .get_bindings(&aor)
            .await
            .map(|bindings| {
                bindings
                    .into_iter()
                    .map(|b| BindingInfo {
                        contact: b.contact,
                        expires_at: b.expires_at.to_rfc3339(),
                        user_agent: b.user_agent,
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let status = UserRegistrationStatus {
        username,
        is_online: is_registered,
        bindings,
    };

    Ok(Json(ApiResponse::success(status)))
}

/// Get online user count
pub async fn get_online_count(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<OnlineCountResponse>>, StatusCode> {
    info!("API: Getting online user count");

    let registrar = match &state.registrar {
        Some(reg) => reg,
        None => {
            error!("Registrar not available");
            return Ok(Json(ApiResponse::error(
                "Registrar not available".to_string(),
            )));
        }
    };

    let count = registrar.get_registration_count().await;

    Ok(Json(ApiResponse::success(OnlineCountResponse { count })))
}

/// Registration info for API response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistrationInfo {
    pub aor: String,
    pub bindings: Vec<BindingInfo>,
}

/// Binding info for API response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BindingInfo {
    pub contact: String,
    pub expires_at: String,
    pub user_agent: Option<String>,
}

/// User registration status
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserRegistrationStatus {
    pub username: String,
    pub is_online: bool,
    pub bindings: Vec<BindingInfo>,
}

/// Online count response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OnlineCountResponse {
    pub count: usize,
}

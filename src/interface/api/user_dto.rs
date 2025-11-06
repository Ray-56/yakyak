//! User API DTOs (Data Transfer Objects)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub realm: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub realm: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
}

/// Change password request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// Generic API response
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Delete response
#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub id: i32,
    pub deleted: bool,
}

/// Convert domain User to UserResponse
impl From<crate::domain::user::User> for UserResponse {
    fn from(user: crate::domain::user::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            realm: user.realm,
            display_name: user.display_name,
            email: user.email,
            enabled: user.enabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Convert CreateUserRequest to domain CreateUser
impl From<CreateUserRequest> for crate::domain::user::CreateUser {
    fn from(req: CreateUserRequest) -> Self {
        Self {
            username: req.username,
            password: req.password,
            realm: req.realm,
            display_name: req.display_name,
            email: req.email,
        }
    }
}

/// Convert UpdateUserRequest to domain UpdateUser
impl From<UpdateUserRequest> for crate::domain::user::UpdateUser {
    fn from(req: UpdateUserRequest) -> Self {
        Self {
            display_name: req.display_name,
            email: req.email,
            enabled: req.enabled,
        }
    }
}

/// Convert ChangePasswordRequest to domain ChangePassword
impl From<ChangePasswordRequest> for crate::domain::user::ChangePassword {
    fn from(req: ChangePasswordRequest) -> Self {
        Self {
            old_password: req.old_password,
            new_password: req.new_password,
        }
    }
}

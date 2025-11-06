//! User entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(sqlx::FromRow))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub sip_ha1: Option<String>, // MD5(username:realm:password) for SIP Digest Auth
    pub realm: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub enabled: bool,
    pub role_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User creation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String, // Plain text password (will be hashed)
    pub realm: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub role_id: Option<Uuid>,
}

/// User update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub role_id: Option<Uuid>,
}

/// Change password data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

impl User {
    /// Get SIP URI
    pub fn sip_uri(&self) -> String {
        format!("sip:{}@{}", self.username, self.realm)
    }

    /// Check if user is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

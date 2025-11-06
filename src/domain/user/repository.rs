//! User repository interface

use super::entity::{ChangePassword, CreateUser, UpdateUser, User};
use crate::domain::shared::error::Result;
use async_trait::async_trait;

/// User repository trait
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create a new user
    async fn create(&self, data: CreateUser) -> Result<User>;

    /// Find user by ID
    async fn find_by_id(&self, id: i32) -> Result<Option<User>>;

    /// Find user by username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Find user by username and realm
    async fn find_by_username_and_realm(&self, username: &str, realm: &str)
        -> Result<Option<User>>;

    /// List all users
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>>;

    /// List users by realm
    async fn list_by_realm(&self, realm: &str, limit: i64, offset: i64) -> Result<Vec<User>>;

    /// Update user
    async fn update(&self, id: i32, data: UpdateUser) -> Result<User>;

    /// Change password
    async fn change_password(&self, id: i32, data: ChangePassword) -> Result<()>;

    /// Delete user
    async fn delete(&self, id: i32) -> Result<()>;

    /// Enable/disable user
    async fn set_enabled(&self, id: i32, enabled: bool) -> Result<()>;

    /// Count total users
    async fn count(&self) -> Result<i64>;

    /// Count users by realm
    async fn count_by_realm(&self, realm: &str) -> Result<i64>;

    /// Verify user credentials
    async fn verify_credentials(&self, username: &str, password: &str) -> Result<Option<User>>;
}

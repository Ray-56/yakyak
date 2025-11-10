use async_trait::async_trait;
use std::collections::HashSet;
use uuid::Uuid;

use super::role::{Permission, Role};

/// Repository for role management
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Create a new role
    async fn create(&self, role: &Role) -> Result<Role, String>;

    /// Get role by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, String>;

    /// Get role by name
    async fn get_by_name(&self, name: &str) -> Result<Option<Role>, String>;

    /// List all roles
    async fn list(&self) -> Result<Vec<Role>, String>;

    /// Update role
    async fn update(&self, id: Uuid, name: Option<String>, description: Option<String>, permissions: Option<HashSet<Permission>>) -> Result<Role, String>;

    /// Delete role (only non-system roles)
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Check if role exists
    async fn exists(&self, id: Uuid) -> Result<bool, String>;

    /// Count roles
    async fn count(&self) -> Result<i64, String>;
}

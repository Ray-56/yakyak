#[cfg(feature = "postgres")]
use async_trait::async_trait;
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "postgres")]
use std::collections::HashSet;
#[cfg(feature = "postgres")]
use uuid::Uuid;

#[cfg(feature = "postgres")]
use crate::domain::user::role_repository::RoleRepository;
#[cfg(feature = "postgres")]
use crate::domain::user::role::{Permission, Role};

#[cfg(feature = "postgres")]
#[derive(Debug, Clone, sqlx::FromRow)]
struct RoleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    permissions: Vec<String>,
    is_system: bool,
}

#[cfg(feature = "postgres")]
impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        let permissions: HashSet<Permission> = row
            .permissions
            .iter()
            .filter_map(|s| Permission::from_str(s))
            .collect();

        Role {
            id: row.id,
            name: row.name,
            description: row.description,
            permissions,
            is_system: row.is_system,
        }
    }
}

#[cfg(feature = "postgres")]
pub struct PgRoleRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn permissions_to_vec(permissions: &HashSet<Permission>) -> Vec<String> {
        permissions.iter().map(|p| p.as_str().to_string()).collect()
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn create(&self, role: &Role) -> Result<Role, String> {
        let permissions = Self::permissions_to_vec(&role.permissions);

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            INSERT INTO roles (id, name, description, permissions, is_system)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, permissions, is_system
            "#,
        )
        .bind(role.id)
        .bind(&role.name)
        .bind(&role.description)
        .bind(&permissions)
        .bind(role.is_system)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create role: {}", e))?;

        Ok(row.into())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, String> {
        let result = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, permissions, is_system
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get role: {}", e))?;

        Ok(result.map(|row| row.into()))
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Role>, String> {
        let result = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, permissions, is_system
            FROM roles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get role by name: {}", e))?;

        Ok(result.map(|row| row.into()))
    }

    async fn list(&self) -> Result<Vec<Role>, String> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, permissions, is_system
            FROM roles
            ORDER BY is_system DESC, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list roles: {}", e))?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        permissions: Option<HashSet<Permission>>,
    ) -> Result<Role, String> {
        // Check if role exists and is not a system role
        let existing = self.get_by_id(id).await?
            .ok_or_else(|| "Role not found".to_string())?;

        if existing.is_system {
            return Err("Cannot update system role".to_string());
        }

        let name = name.unwrap_or(existing.name);
        let description = description.or(existing.description);
        let permissions = Self::permissions_to_vec(&permissions.unwrap_or(existing.permissions));

        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            UPDATE roles
            SET name = $1, description = $2, permissions = $3
            WHERE id = $4
            RETURNING id, name, description, permissions, is_system
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(&permissions)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update role: {}", e))?;

        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        // Check if role is a system role
        let role = self.get_by_id(id).await?
            .ok_or_else(|| "Role not found".to_string())?;

        if role.is_system {
            return Err("Cannot delete system role".to_string());
        }

        sqlx::query(
            r#"
            DELETE FROM roles
            WHERE id = $1 AND is_system = FALSE
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete role: {}", e))?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check role existence: {}", e))?;

        Ok(result)
    }

    async fn count(&self) -> Result<i64, String> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM roles
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count roles: {}", e))?;

        Ok(count)
    }
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use std::collections::HashSet;

    async fn get_test_pool() -> PgPool {
        // Use the same database as the application
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://yakyak:yakyak@localhost/yakyak".to_string());

        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_create_and_get_role() {
        let pool = get_test_pool().await;
        let repo = PgRoleRepository::new(pool);

        let permissions = HashSet::from([Permission::UserRead, Permission::CallCreate]);
        let role = Role::new(
            format!("test_role_{}", Uuid::new_v4()),
            Some("Test role".to_string()),
            permissions.clone(),
        );

        // Create role
        let created = repo.create(&role).await.unwrap();
        assert_eq!(created.name, role.name);
        assert_eq!(created.permissions.len(), 2);

        // Get role by ID
        let retrieved = repo.get_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);

        // Get role by name
        let by_name = repo.get_by_name(&created.name).await.unwrap().unwrap();
        assert_eq!(by_name.id, created.id);

        // Clean up
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_list_roles() {
        let pool = get_test_pool().await;
        let repo = PgRoleRepository::new(pool);

        let roles = repo.list().await.unwrap();
        // Should at least have the 3 system roles
        assert!(roles.len() >= 3);

        // Check for system roles
        let admin = roles.iter().find(|r| r.name == "administrator");
        assert!(admin.is_some());
        assert!(admin.unwrap().is_system);
    }

    #[tokio::test]
    async fn test_update_role() {
        let pool = get_test_pool().await;
        let repo = PgRoleRepository::new(pool);

        let permissions = HashSet::from([Permission::UserRead]);
        let role = Role::new(
            format!("test_role_{}", Uuid::new_v4()),
            Some("Test role".to_string()),
            permissions.clone(),
        );

        let created = repo.create(&role).await.unwrap();

        // Update role
        let new_permissions = HashSet::from([Permission::UserRead, Permission::CallCreate]);
        let updated = repo
            .update(
                created.id,
                Some("updated_role".to_string()),
                Some("Updated description".to_string()),
                Some(new_permissions),
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "updated_role");
        assert_eq!(updated.permissions.len(), 2);

        // Clean up
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_cannot_delete_system_role() {
        let pool = get_test_pool().await;
        let repo = PgRoleRepository::new(pool);

        // Try to delete administrator role
        let admin = repo.get_by_name("administrator").await.unwrap().unwrap();
        let result = repo.delete(admin.id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete system role"));
    }

    #[tokio::test]
    async fn test_role_count() {
        let pool = get_test_pool().await;
        let repo = PgRoleRepository::new(pool);

        let count = repo.count().await.unwrap();
        assert!(count >= 3); // At least 3 system roles
    }
}

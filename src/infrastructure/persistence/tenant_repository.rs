/// PostgreSQL implementation of TenantRepository
use crate::domain::tenant::{
    SubscriptionPlan, Tenant, TenantQuota, TenantRepository, TenantStatus, TenantUsage,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{debug, error};
use uuid::Uuid;

pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, String> {
        let status_str = format!("{:?}", tenant.status);
        let plan_str = format!("{:?}", tenant.plan);
        let features_str = tenant.quota.advanced_features.join(",");

        let result = sqlx::query(
            r#"
            INSERT INTO tenants
            (id, name, slug, status, plan, realm, contact_email, contact_name, contact_phone,
             company_name, billing_email, billing_address, custom_domain, timezone, language,
             logo_url, primary_color, max_users, max_concurrent_calls, max_conference_participants,
             storage_quota_gb, monthly_call_minutes, advanced_features, trial_ends_at,
             suspended_reason, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
            "#,
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(&status_str)
        .bind(&plan_str)
        .bind(&tenant.realm)
        .bind(&tenant.contact_email)
        .bind(tenant.contact_name.as_ref())
        .bind(tenant.contact_phone.as_ref())
        .bind(tenant.company_name.as_ref())
        .bind(tenant.billing_email.as_ref())
        .bind(tenant.billing_address.as_ref())
        .bind(tenant.custom_domain.as_ref())
        .bind(&tenant.timezone)
        .bind(&tenant.language)
        .bind(tenant.logo_url.as_ref())
        .bind(tenant.primary_color.as_ref())
        .bind(tenant.quota.max_users as i32)
        .bind(tenant.quota.max_concurrent_calls as i32)
        .bind(tenant.quota.max_conference_participants as i32)
        .bind(tenant.quota.storage_quota_gb as i32)
        .bind(tenant.quota.monthly_call_minutes as i32)
        .bind(&features_str)
        .bind(tenant.trial_ends_at)
        .bind(tenant.suspended_reason.as_ref())
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created tenant: {}", tenant.id);
                Ok(tenant)
            }
            Err(e) => {
                error!("Failed to create tenant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, slug, status, plan, realm, contact_email, contact_name, contact_phone,
                   company_name, billing_email, billing_address, custom_domain, timezone, language,
                   logo_url, primary_color, max_users, max_concurrent_calls, max_conference_participants,
                   storage_quota_gb, monthly_call_minutes, advanced_features, trial_ends_at,
                   suspended_reason, created_at, updated_at, metadata
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(row_to_tenant(row))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get tenant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, slug, status, plan, realm, contact_email, contact_name, contact_phone,
                   company_name, billing_email, billing_address, custom_domain, timezone, language,
                   logo_url, primary_color, max_users, max_concurrent_calls, max_conference_participants,
                   storage_quota_gb, monthly_call_minutes, advanced_features, trial_ends_at,
                   suspended_reason, created_at, updated_at, metadata
            FROM tenants
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(row_to_tenant(row))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get tenant by slug: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_tenant(&self, tenant: &Tenant) -> Result<(), String> {
        let status_str = format!("{:?}", tenant.status);
        let plan_str = format!("{:?}", tenant.plan);
        let features_str = tenant.quota.advanced_features.join(",");
        let metadata_json = serde_json::to_value(&tenant.metadata).unwrap_or(serde_json::json!({}));

        let result = sqlx::query(
            r#"
            UPDATE tenants
            SET name = $2, status = $3, plan = $4, contact_email = $5, contact_name = $6,
                contact_phone = $7, company_name = $8, billing_email = $9, billing_address = $10,
                custom_domain = $11, timezone = $12, language = $13, logo_url = $14,
                primary_color = $15, max_users = $16, max_concurrent_calls = $17,
                max_conference_participants = $18, storage_quota_gb = $19, monthly_call_minutes = $20,
                advanced_features = $21, trial_ends_at = $22, suspended_reason = $23,
                updated_at = $24, metadata = $25
            WHERE id = $1
            "#,
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&status_str)
        .bind(&plan_str)
        .bind(&tenant.contact_email)
        .bind(tenant.contact_name.as_ref())
        .bind(tenant.contact_phone.as_ref())
        .bind(tenant.company_name.as_ref())
        .bind(tenant.billing_email.as_ref())
        .bind(tenant.billing_address.as_ref())
        .bind(tenant.custom_domain.as_ref())
        .bind(&tenant.timezone)
        .bind(&tenant.language)
        .bind(tenant.logo_url.as_ref())
        .bind(tenant.primary_color.as_ref())
        .bind(tenant.quota.max_users as i32)
        .bind(tenant.quota.max_concurrent_calls as i32)
        .bind(tenant.quota.max_conference_participants as i32)
        .bind(tenant.quota.storage_quota_gb as i32)
        .bind(tenant.quota.monthly_call_minutes as i32)
        .bind(&features_str)
        .bind(tenant.trial_ends_at)
        .bind(tenant.suspended_reason.as_ref())
        .bind(tenant.updated_at)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated tenant: {}", tenant.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update tenant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted tenant: {}", tenant_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete tenant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn list_tenants(&self, status: Option<TenantStatus>) -> Result<Vec<Tenant>, String> {
        let result = if let Some(status) = status {
            let status_str = format!("{:?}", status);
            sqlx::query(
                r#"
                SELECT id, name, slug, status, plan, realm, contact_email, contact_name, contact_phone,
                       company_name, billing_email, billing_address, custom_domain, timezone, language,
                       logo_url, primary_color, max_users, max_concurrent_calls, max_conference_participants,
                       storage_quota_gb, monthly_call_minutes, advanced_features, trial_ends_at,
                       suspended_reason, created_at, updated_at, metadata
                FROM tenants
                WHERE status = $1
                ORDER BY name
                "#,
            )
            .bind(&status_str)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, name, slug, status, plan, realm, contact_email, contact_name, contact_phone,
                       company_name, billing_email, billing_address, custom_domain, timezone, language,
                       logo_url, primary_color, max_users, max_concurrent_calls, max_conference_participants,
                       storage_quota_gb, monthly_call_minutes, advanced_features, trial_ends_at,
                       suspended_reason, created_at, updated_at, metadata
                FROM tenants
                ORDER BY name
                "#,
            )
            .fetch_all(&self.pool)
            .await
        };

        match result {
            Ok(rows) => {
                let tenants: Vec<Tenant> = rows.iter().map(|row| row_to_tenant(row.clone())).collect();
                Ok(tenants)
            }
            Err(e) => {
                error!("Failed to list tenants: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_usage(&self, tenant_id: Uuid) -> Result<Option<TenantUsage>, String> {
        let result = sqlx::query(
            r#"
            SELECT tenant_id, current_users, current_calls, minutes_used_this_month,
                   storage_used_gb, last_activity_at
            FROM tenant_usage
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let usage = TenantUsage {
                    tenant_id: row.get("tenant_id"),
                    current_users: row.get::<i32, _>("current_users") as u32,
                    current_calls: row.get::<i32, _>("current_calls") as u32,
                    minutes_used_this_month: row.get("minutes_used_this_month"),
                    storage_used_gb: row.get("storage_used_gb"),
                    last_activity_at: row.get("last_activity_at"),
                };
                Ok(Some(usage))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get tenant usage: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_usage(&self, usage: &TenantUsage) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            INSERT INTO tenant_usage
            (tenant_id, current_users, current_calls, minutes_used_this_month, storage_used_gb, last_activity_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id)
            DO UPDATE SET
                current_users = $2,
                current_calls = $3,
                minutes_used_this_month = $4,
                storage_used_gb = $5,
                last_activity_at = $6
            "#,
        )
        .bind(usage.tenant_id)
        .bind(usage.current_users as i32)
        .bind(usage.current_calls as i32)
        .bind(usage.minutes_used_this_month)
        .bind(usage.storage_used_gb)
        .bind(usage.last_activity_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated usage for tenant: {}", usage.tenant_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update tenant usage: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }
}

fn row_to_tenant(row: sqlx::postgres::PgRow) -> Tenant {
    let status_str: String = row.get("status");
    let status = match status_str.as_str() {
        "Active" => TenantStatus::Active,
        "Suspended" => TenantStatus::Suspended,
        "Trial" => TenantStatus::Trial,
        "Deactivated" => TenantStatus::Deactivated,
        _ => TenantStatus::Active,
    };

    let plan_str: String = row.get("plan");
    let plan = match plan_str.as_str() {
        "Free" => SubscriptionPlan::Free,
        "Starter" => SubscriptionPlan::Starter,
        "Professional" => SubscriptionPlan::Professional,
        "Enterprise" => SubscriptionPlan::Enterprise,
        custom => SubscriptionPlan::Custom(custom.to_string()),
    };

    let features_str: String = row.get("advanced_features");
    let advanced_features: Vec<String> = features_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let quota = TenantQuota {
        max_users: row.get::<i32, _>("max_users") as u32,
        max_concurrent_calls: row.get::<i32, _>("max_concurrent_calls") as u32,
        max_conference_participants: row.get::<i32, _>("max_conference_participants") as u32,
        storage_quota_gb: row.get::<i32, _>("storage_quota_gb") as u32,
        monthly_call_minutes: row.get::<i32, _>("monthly_call_minutes") as u32,
        advanced_features,
    };

    let metadata_json: Option<serde_json::Value> = row.get("metadata");
    let metadata = metadata_json
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Tenant {
        id: row.get("id"),
        name: row.get("name"),
        slug: row.get("slug"),
        status,
        plan,
        quota,
        realm: row.get("realm"),
        contact_email: row.get("contact_email"),
        contact_name: row.get("contact_name"),
        contact_phone: row.get("contact_phone"),
        company_name: row.get("company_name"),
        billing_email: row.get("billing_email"),
        billing_address: row.get("billing_address"),
        custom_domain: row.get("custom_domain"),
        timezone: row.get("timezone"),
        language: row.get("language"),
        logo_url: row.get("logo_url"),
        primary_color: row.get("primary_color"),
        metadata,
        trial_ends_at: row.get("trial_ends_at"),
        suspended_reason: row.get("suspended_reason"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_get_tenant() {
        // Test implementation would go here
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_update_tenant() {
        // Test implementation would go here
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_usage_tracking() {
        // Test implementation would go here
    }
}

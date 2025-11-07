/// Tenant management REST API
use crate::domain::tenant::{SubscriptionPlan, Tenant, TenantRepository, TenantStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Tenant API state
pub struct TenantApiState {
    pub repository: Arc<dyn TenantRepository>,
}

/// Create tenant router
pub fn tenant_router(state: Arc<TenantApiState>) -> Router {
    Router::new()
        .route("/tenants", post(create_tenant))
        .route("/tenants", get(list_tenants))
        .route("/tenants/:id", get(get_tenant))
        .route("/tenants/:id", put(update_tenant))
        .route("/tenants/:id", delete(delete_tenant))
        .route("/tenants/slug/:slug", get(get_tenant_by_slug))
        .route("/tenants/:id/suspend", post(suspend_tenant))
        .route("/tenants/:id/reactivate", post(reactivate_tenant))
        .route("/tenants/:id/upgrade", post(upgrade_plan))
        .route("/tenants/:id/usage", get(get_usage))
        .with_state(state)
}

/// Request to create a tenant
#[derive(Debug, Deserialize)]
struct CreateTenantRequest {
    name: String,
    slug: String,
    contact_email: String,
    contact_name: String,
    plan: Option<String>,
}

/// Request to update a tenant
#[derive(Debug, Deserialize)]
struct UpdateTenantRequest {
    name: Option<String>,
    contact_email: Option<String>,
    contact_name: Option<String>,
    contact_phone: Option<String>,
    company_name: Option<String>,
}

/// Request to suspend a tenant
#[derive(Debug, Deserialize)]
struct SuspendTenantRequest {
    reason: String,
}

/// Request to upgrade plan
#[derive(Debug, Deserialize)]
struct UpgradePlanRequest {
    plan: String,
}

/// Query parameters for listing tenants
#[derive(Debug, Deserialize)]
struct ListTenantsQuery {
    status: Option<String>,
}

/// Response for tenant operations
#[derive(Debug, Serialize)]
struct TenantResponse {
    id: String,
    name: String,
    slug: String,
    status: String,
    plan: String,
    realm: String,
    contact_email: String,
    contact_name: Option<String>,
    max_users: u32,
    max_concurrent_calls: u32,
    storage_quota_gb: u32,
    monthly_call_minutes: u32,
    created_at: String,
    updated_at: String,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id.to_string(),
            name: tenant.name,
            slug: tenant.slug,
            status: format!("{:?}", tenant.status),
            plan: format!("{:?}", tenant.plan),
            realm: tenant.realm,
            contact_email: tenant.contact_email,
            contact_name: tenant.contact_name,
            max_users: tenant.quota.max_users,
            max_concurrent_calls: tenant.quota.max_concurrent_calls,
            storage_quota_gb: tenant.quota.storage_quota_gb,
            monthly_call_minutes: tenant.quota.monthly_call_minutes,
            created_at: tenant.created_at.to_rfc3339(),
            updated_at: tenant.updated_at.to_rfc3339(),
        }
    }
}

/// Parse SubscriptionPlan from string
fn parse_plan(s: &str) -> Result<SubscriptionPlan, String> {
    match s {
        "Free" => Ok(SubscriptionPlan::Free),
        "Starter" => Ok(SubscriptionPlan::Starter),
        "Professional" => Ok(SubscriptionPlan::Professional),
        "Enterprise" => Ok(SubscriptionPlan::Enterprise),
        custom => Ok(SubscriptionPlan::Custom(custom.to_string())),
    }
}

/// Parse TenantStatus from string
fn parse_status(s: &str) -> Result<TenantStatus, String> {
    match s {
        "Active" => Ok(TenantStatus::Active),
        "Suspended" => Ok(TenantStatus::Suspended),
        "Trial" => Ok(TenantStatus::Trial),
        "Deactivated" => Ok(TenantStatus::Deactivated),
        _ => Err(format!("Invalid status: {}", s)),
    }
}

/// Create a new tenant
async fn create_tenant(
    State(state): State<Arc<TenantApiState>>,
    Json(req): Json<CreateTenantRequest>,
) -> Response {
    let mut tenant = Tenant::new(req.name, req.slug, req.contact_email, req.contact_name);

    if let Some(plan_str) = req.plan {
        match parse_plan(&plan_str) {
            Ok(plan) => tenant.upgrade_plan(plan),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response()
            }
        }
    }

    match state.repository.create_tenant(tenant).await {
        Ok(tenant) => {
            let response = TenantResponse::from(tenant);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get tenant by ID
async fn get_tenant(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get tenant by slug
async fn get_tenant_by_slug(
    State(state): State<Arc<TenantApiState>>,
    Path(slug): Path<String>,
) -> Response {
    match state.repository.get_tenant_by_slug(&slug).await {
        Ok(Some(tenant)) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// List all tenants
async fn list_tenants(
    State(state): State<Arc<TenantApiState>>,
    Query(query): Query<ListTenantsQuery>,
) -> Response {
    let status_filter = if let Some(status_str) = query.status {
        match parse_status(&status_str) {
            Ok(status) => Some(status),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response()
            }
        }
    } else {
        None
    };

    match state.repository.list_tenants(status_filter).await {
        Ok(tenants) => {
            let responses: Vec<TenantResponse> = tenants.into_iter().map(|t| t.into()).collect();
            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Update a tenant
async fn update_tenant(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut tenant = match state.repository.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Tenant not found" })),
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

    if let Some(name) = req.name {
        tenant.name = name;
    }
    if let Some(contact_email) = req.contact_email {
        tenant.contact_email = contact_email;
    }
    if let Some(contact_name) = req.contact_name {
        tenant.contact_name = Some(contact_name);
    }
    if let Some(contact_phone) = req.contact_phone {
        tenant.contact_phone = Some(contact_phone);
    }
    if let Some(company_name) = req.company_name {
        tenant.company_name = Some(company_name);
    }

    tenant.updated_at = Utc::now();

    match state.repository.update_tenant(&tenant).await {
        Ok(_) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Delete a tenant
async fn delete_tenant(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.delete_tenant(tenant_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Suspend a tenant
async fn suspend_tenant(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
    Json(req): Json<SuspendTenantRequest>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut tenant = match state.repository.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Tenant not found" })),
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

    tenant.suspend(Some(req.reason));

    match state.repository.update_tenant(&tenant).await {
        Ok(_) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Reactivate a suspended tenant
async fn reactivate_tenant(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut tenant = match state.repository.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Tenant not found" })),
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

    tenant.reactivate();

    match state.repository.update_tenant(&tenant).await {
        Ok(_) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Upgrade tenant plan
async fn upgrade_plan(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpgradePlanRequest>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    let mut tenant = match state.repository.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Tenant not found" })),
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

    let plan = match parse_plan(&req.plan) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    tenant.upgrade_plan(plan);

    match state.repository.update_tenant(&tenant).await {
        Ok(_) => {
            let response = TenantResponse::from(tenant);
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Get tenant usage
async fn get_usage(
    State(state): State<Arc<TenantApiState>>,
    Path(id): Path<String>,
) -> Response {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid UUID" })),
            )
                .into_response()
        }
    };

    match state.repository.get_usage(tenant_id).await {
        Ok(Some(usage)) => Json(serde_json::json!({
            "tenant_id": usage.tenant_id.to_string(),
            "current_users": usage.current_users,
            "current_calls": usage.current_calls,
            "minutes_used_this_month": usage.minutes_used_this_month,
            "storage_used_gb": usage.storage_used_gb,
            "last_activity_at": usage.last_activity_at.to_rfc3339(),
        }))
        .into_response(),
        Ok(None) => Json(serde_json::json!({
            "tenant_id": id,
            "current_users": 0,
            "current_calls": 0,
            "minutes_used_this_month": 0.0,
            "storage_used_gb": 0.0,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

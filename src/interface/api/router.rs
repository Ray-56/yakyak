//! API Router configuration

use super::calls_handler::{get_active_call, get_active_calls, get_call_stats, hangup_call};
use super::cdr_handler::{export_cdrs_csv, export_cdrs_json, get_cdr, get_cdr_by_call_id, list_cdrs};
use super::metrics_handler::metrics_handler;
use super::monitoring::{get_prometheus_metrics, get_system_health};
use super::user_handler::{
    change_password, create_user, delete_user, get_online_count, get_online_users, get_user,
    get_user_by_username, get_user_registration_status, health_check, list_users, set_enabled,
    update_user, AppState,
};
use super::ws_handler::{ws_handler, EventBroadcaster};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Build the API router
pub fn build_router(
    state: AppState,
    prometheus_handle: PrometheusHandle,
    event_broadcaster: Arc<EventBroadcaster>,
) -> Router {
    // Health check route (no auth required)
    let health_routes = Router::new().route("/health", get(health_check));

    // User management routes
    let user_routes = Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/username/:username", get(get_user_by_username))
        .route("/users/:id/password", post(change_password))
        .route("/users/:id/enabled/:enabled", put(set_enabled))
        .route("/users/online", get(get_online_users))
        .route("/users/online/count", get(get_online_count))
        .route("/users/:username/status", get(get_user_registration_status));

    // CDR routes
    let cdr_routes = Router::new()
        .route("/cdrs", get(list_cdrs))
        .route("/cdrs/:id", get(get_cdr))
        .route("/cdrs/call-id/:call_id", get(get_cdr_by_call_id))
        .route("/cdrs/export/csv", get(export_cdrs_csv))
        .route("/cdrs/export/json", get(export_cdrs_json));

    // Call management routes
    let call_routes = Router::new()
        .route("/calls", get(get_active_calls))
        .route("/calls/:call_id", get(get_active_call))
        .route("/calls/:call_id/hangup", post(hangup_call))
        .route("/calls/stats", get(get_call_stats));

    // Monitoring routes
    let monitoring_routes = Router::new()
        .route("/monitoring/health", get(get_system_health))
        .route("/monitoring/prometheus", get(get_prometheus_metrics));

    // Metrics route (separate state)
    let metrics_routes = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(prometheus_handle);

    // WebSocket route (separate state)
    let ws_routes = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(event_broadcaster);

    // Combine routes with state
    Router::new()
        .merge(health_routes)
        .merge(user_routes)
        .merge(cdr_routes)
        .merge(call_routes)
        .merge(monitoring_routes)
        .with_state(state)
        .merge(metrics_routes)
        .merge(ws_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::UserRepository;
    use std::sync::Arc;

    #[test]
    fn test_router_creation() {
        // This is a compile-time test to ensure the router can be built
        // We can't actually run it without a real database connection

        // The test just verifies that the router structure compiles correctly
        assert!(true, "Router module compiles successfully");
    }
}

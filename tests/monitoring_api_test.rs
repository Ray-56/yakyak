//! Monitoring API Integration Tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt; // For `oneshot`
use yakyak::infrastructure::persistence::{
    create_pool, run_migrations, DatabaseConfig, PgCdrRepository, PgUserRepository,
};
use yakyak::interface::api::{build_router, init_metrics, EventBroadcaster};
use yakyak::interface::api::user_handler::AppState;

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_call_stats() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/calls/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["active_calls"].is_number());
    assert!(json["data"]["total_calls_today"].is_number());

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_active_calls() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/calls")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["calls"].is_array());
    assert!(json["data"]["count"].is_number());

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_online_users() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/online")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_online_count() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/online/count")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["count"].is_number());

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_user_registration_status() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/alice/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["username"], "alice");
    assert!(json["data"]["is_online"].is_boolean());
    assert!(json["data"]["bindings"].is_array());

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_metrics() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(body.to_vec()).unwrap();

    // Verify Prometheus metrics format
    assert!(metrics_text.contains("# HELP"));
    assert!(metrics_text.contains("# TYPE"));

    cleanup_monitoring_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_health_check() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_monitoring_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"], "OK");

    cleanup_monitoring_test(pool).await;
}

// Helper functions

async fn setup_monitoring_test() -> (PgPool, AppState, metrics_exporter_prometheus::PrometheusHandle, Arc<EventBroadcaster>) {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/yakyak_test".to_string());

    let config = DatabaseConfig {
        url: db_url,
        max_connections: 5,
        min_connections: 1,
        connect_timeout: std::time::Duration::from_secs(10),
        idle_timeout: std::time::Duration::from_secs(60),
        max_lifetime: std::time::Duration::from_secs(300),
    };

    let pool = create_pool(&config).await.expect("Failed to create pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let cdr_repo = Arc::new(PgCdrRepository::new(pool.clone()));

    // Initialize metrics and event broadcaster
    let prometheus_handle = init_metrics();
    let event_broadcaster = Arc::new(EventBroadcaster::new());

    let state = AppState {
        user_repository: user_repo,
        cdr_repository: Some(cdr_repo),
        call_router: None,
        registrar: None,
        event_broadcaster: Some(event_broadcaster.clone()),
    };

    (pool, state, prometheus_handle, event_broadcaster)
}

async fn cleanup_monitoring_test(pool: PgPool) {
    // Clean up test data
    pool.close().await;
}

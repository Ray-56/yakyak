//! CDR API Integration Tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt; // For `oneshot` and `ready`
use yakyak::domain::cdr::{CallDetailRecord, CallDirection, CdrRepository};
use yakyak::infrastructure::persistence::{create_pool, run_migrations, DatabaseConfig, PgCdrRepository, PgUserRepository};
use yakyak::interface::api::{build_router, init_metrics, EventBroadcaster};
use yakyak::interface::api::user_handler::AppState;

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_cdr() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create a test CDR
    let cdr_repo = PgCdrRepository::new(pool.clone());
    let cdr = CallDetailRecord::new(
        "test-api-get-123".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    let cdr_id = cdr.id;
    cdr_repo.create(&cdr).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/cdrs/{}", cdr_id))
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
    assert_eq!(json["data"]["call_id"], "test-api-get-123");
    assert_eq!(json["data"]["caller_username"], "alice");
    assert_eq!(json["data"]["callee_username"], "bob");

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_cdr_by_call_id() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create a test CDR
    let cdr_repo = PgCdrRepository::new(pool.clone());
    let cdr = CallDetailRecord::new(
        "test-api-call-id-456".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "charlie".to_string(),
        "sip:charlie@example.com".to_string(),
        CallDirection::Outbound,
    );
    cdr_repo.create(&cdr).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs/call-id/test-api-call-id-456")
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
    assert_eq!(json["data"]["call_id"], "test-api-call-id-456");
    assert_eq!(json["data"]["caller_username"], "alice");
    assert_eq!(json["data"]["callee_username"], "charlie");

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_get_cdr_not_found() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request with non-existent ID
    let non_existent_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/cdrs/{}", non_existent_id))
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

    assert_eq!(json["success"], false);
    assert!(json["error"].as_str().unwrap().contains("not found"));

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_list_cdrs() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create multiple test CDRs
    let cdr_repo = PgCdrRepository::new(pool.clone());
    for i in 0..3 {
        let cdr = CallDetailRecord::new(
            format!("test-api-list-{}", i),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );
        cdr_repo.create(&cdr).await.expect("Failed to create CDR");
    }

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs?limit=10&offset=0")
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
    assert!(json["data"]["cdrs"].as_array().unwrap().len() >= 3);
    assert!(json["data"]["total"].as_i64().unwrap() >= 3);
    assert_eq!(json["data"]["limit"], 10);
    assert_eq!(json["data"]["offset"], 0);

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_list_cdrs_with_filters() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create CDRs with different callers
    let cdr_repo = PgCdrRepository::new(pool.clone());

    let cdr1 = CallDetailRecord::new(
        "test-api-filter-alice".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    cdr_repo.create(&cdr1).await.expect("Failed to create CDR");

    let cdr2 = CallDetailRecord::new(
        "test-api-filter-charlie".to_string(),
        "charlie".to_string(),
        "sip:charlie@example.com".to_string(),
        "192.168.1.101".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    cdr_repo.create(&cdr2).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request with filter
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs?caller_username=alice")
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

    // Check that all returned CDRs have caller_username = "alice"
    let cdrs = json["data"]["cdrs"].as_array().unwrap();
    for cdr in cdrs {
        if cdr["call_id"].as_str().unwrap().starts_with("test-api-filter") {
            assert_eq!(cdr["caller_username"], "alice");
        }
    }

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_list_cdrs_pagination() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create multiple test CDRs
    let cdr_repo = PgCdrRepository::new(pool.clone());
    for i in 0..5 {
        let cdr = CallDetailRecord::new(
            format!("test-api-page-{}", i),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );
        cdr_repo.create(&cdr).await.expect("Failed to create CDR");
    }

    // Build router
    let app = build_router(state.clone(), prometheus_handle, event_broadcaster);

    // First page
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs?limit=2&offset=0")
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
    assert_eq!(json["data"]["limit"], 2);
    assert_eq!(json["data"]["offset"], 0);

    // Second page - need new metrics/broadcaster instances
    let prometheus_handle2 = init_metrics();
    let event_broadcaster2 = Arc::new(EventBroadcaster::new());
    let app2 = build_router(state, prometheus_handle2, event_broadcaster2);
    let response2 = app2
        .oneshot(
            Request::builder()
                .uri("/cdrs?limit=2&offset=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json2: Value = serde_json::from_slice(&body2).unwrap();

    assert_eq!(json2["success"], true);
    assert_eq!(json2["data"]["limit"], 2);
    assert_eq!(json2["data"]["offset"], 2);

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_export_cdrs_csv() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create test CDR
    let cdr_repo = PgCdrRepository::new(pool.clone());
    let cdr = CallDetailRecord::new(
        "test-api-export-csv".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    cdr_repo.create(&cdr).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs/export/csv")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "text/csv");
    assert!(headers
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("cdrs.csv"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let csv_content = String::from_utf8(body.to_vec()).unwrap();

    // Verify CSV structure
    assert!(csv_content.starts_with("id,call_id,caller_username"));
    assert!(csv_content.contains("test-api-export-csv"));
    assert!(csv_content.contains("alice"));
    assert!(csv_content.contains("bob"));

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_export_cdrs_json() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create test CDR
    let cdr_repo = PgCdrRepository::new(pool.clone());
    let cdr = CallDetailRecord::new(
        "test-api-export-json".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    cdr_repo.create(&cdr).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs/export/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    assert!(headers
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("cdrs.json"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify JSON structure
    assert!(json.is_array());
    let cdrs = json.as_array().unwrap();
    assert!(!cdrs.is_empty());

    // Find our test CDR
    let test_cdr = cdrs
        .iter()
        .find(|c| c["call_id"] == "test-api-export-json");
    assert!(test_cdr.is_some());
    let test_cdr = test_cdr.unwrap();
    assert_eq!(test_cdr["caller_username"], "alice");
    assert_eq!(test_cdr["callee_username"], "bob");

    cleanup_api_test(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_api_export_csv_with_special_characters() {
    let (pool, state, prometheus_handle, event_broadcaster) = setup_api_test().await;

    // Create CDR with special characters that need CSV escaping
    let cdr_repo = PgCdrRepository::new(pool.clone());
    let mut cdr = CallDetailRecord::new(
        "test-csv-escape".to_string(),
        "alice,test".to_string(), // Contains comma
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );

    // Add end reason with special characters
    cdr.mark_answered();
    cdr.mark_ended(
        yakyak::domain::cdr::CallStatus::Completed,
        Some("Call ended, \"normal\" clearing".to_string()), // Contains comma and quotes
        Some(200),
    );

    cdr_repo.create(&cdr).await.expect("Failed to create CDR");

    // Build router
    let app = build_router(state, prometheus_handle, event_broadcaster);

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cdrs/export/csv")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let csv_content = String::from_utf8(body.to_vec()).unwrap();

    // Verify CSV escaping - fields with commas or quotes should be quoted
    assert!(csv_content.contains("\"alice,test\""));
    assert!(csv_content.contains("\"Call ended, \"\"normal\"\" clearing\""));

    cleanup_api_test(pool).await;
}

// Helper functions

async fn setup_api_test() -> (PgPool, AppState, metrics_exporter_prometheus::PrometheusHandle, Arc<EventBroadcaster>) {
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

async fn cleanup_api_test(pool: PgPool) {
    // Clean up test data
    sqlx::query("DELETE FROM call_records WHERE call_id LIKE 'test-api-%' OR call_id LIKE 'test-csv-%'")
        .execute(&pool)
        .await
        .ok();
    pool.close().await;
}

//! CDR Repository Integration Tests

use sqlx::PgPool;
use yakyak::domain::cdr::{CallDetailRecord, CallDirection, CallStatus, CdrFilters, CdrRepository};
use yakyak::infrastructure::persistence::{create_pool, run_migrations, PgCdrRepository, DatabaseConfig};

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_create_and_get() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    // Create a CDR
    let cdr = CallDetailRecord::new(
        "test-call-123".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );

    let cdr_id = cdr.id;

    // Insert CDR
    repo.create(&cdr).await.expect("Failed to create CDR");

    // Retrieve by ID
    let retrieved = repo.get_by_id(cdr_id).await.expect("Failed to get CDR");
    assert!(retrieved.is_some());
    let retrieved_cdr = retrieved.unwrap();

    assert_eq!(retrieved_cdr.call_id, "test-call-123");
    assert_eq!(retrieved_cdr.caller_username, "alice");
    assert_eq!(retrieved_cdr.callee_username, "bob");
    assert_eq!(retrieved_cdr.direction, CallDirection::Internal);
    assert_eq!(retrieved_cdr.status, CallStatus::Active);

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_get_by_call_id() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    let cdr = CallDetailRecord::new(
        "unique-call-456".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "charlie".to_string(),
        "sip:charlie@example.com".to_string(),
        CallDirection::Outbound,
    );

    repo.create(&cdr).await.expect("Failed to create CDR");

    // Retrieve by Call-ID
    let retrieved = repo.get_by_call_id("unique-call-456").await.expect("Failed to get CDR");
    assert!(retrieved.is_some());
    let retrieved_cdr = retrieved.unwrap();

    assert_eq!(retrieved_cdr.call_id, "unique-call-456");
    assert_eq!(retrieved_cdr.caller_username, "alice");
    assert_eq!(retrieved_cdr.callee_username, "charlie");

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_update() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    let mut cdr = CallDetailRecord::new(
        "test-update-789".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );

    repo.create(&cdr).await.expect("Failed to create CDR");

    // Simulate answering the call
    std::thread::sleep(std::time::Duration::from_millis(10));
    cdr.mark_answered();

    // Update CDR
    repo.update(&cdr).await.expect("Failed to update CDR");

    // Retrieve and verify
    let retrieved = repo.get_by_id(cdr.id).await.expect("Failed to get CDR");
    assert!(retrieved.is_some());
    let retrieved_cdr = retrieved.unwrap();

    assert!(retrieved_cdr.answer_time.is_some());
    assert!(retrieved_cdr.setup_duration.is_some());
    assert!(retrieved_cdr.setup_duration.unwrap() >= 0);

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_list_and_count() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    // Create multiple CDRs
    for i in 0..5 {
        let cdr = CallDetailRecord::new(
            format!("call-{}", i),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );
        repo.create(&cdr).await.expect("Failed to create CDR");
    }

    // List all
    let filters = CdrFilters::default();
    let cdrs = repo.list(filters.clone(), 10, 0).await.expect("Failed to list CDRs");
    assert!(cdrs.len() >= 5);

    // Count all
    let count = repo.count(filters).await.expect("Failed to count CDRs");
    assert!(count >= 5);

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_filter_by_caller() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    // Create CDRs with different callers
    let cdr1 = CallDetailRecord::new(
        "call-filter-1".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    repo.create(&cdr1).await.expect("Failed to create CDR");

    let cdr2 = CallDetailRecord::new(
        "call-filter-2".to_string(),
        "charlie".to_string(),
        "sip:charlie@example.com".to_string(),
        "192.168.1.101".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    repo.create(&cdr2).await.expect("Failed to create CDR");

    // Filter by caller
    let mut filters = CdrFilters::default();
    filters.caller_username = Some("alice".to_string());

    let cdrs = repo.list(filters.clone(), 10, 0).await.expect("Failed to list CDRs");

    // Should have at least the alice call
    let alice_cdrs: Vec<_> = cdrs.iter().filter(|c| c.caller_username == "alice").collect();
    assert!(!alice_cdrs.is_empty());

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_complete_lifecycle() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    // Create CDR
    let mut cdr = CallDetailRecord::new(
        "lifecycle-call".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );

    repo.create(&cdr).await.expect("Failed to create CDR");
    assert_eq!(cdr.status, CallStatus::Active);

    // Answer the call
    std::thread::sleep(std::time::Duration::from_millis(10));
    cdr.mark_answered();
    repo.update(&cdr).await.expect("Failed to update CDR");

    // End the call
    std::thread::sleep(std::time::Duration::from_millis(20));
    cdr.mark_ended(CallStatus::Completed, Some("Normal clearing".to_string()), Some(200));
    repo.update(&cdr).await.expect("Failed to update CDR");

    // Verify final state
    let retrieved = repo.get_by_id(cdr.id).await.expect("Failed to get CDR");
    assert!(retrieved.is_some());
    let final_cdr = retrieved.unwrap();

    assert_eq!(final_cdr.status, CallStatus::Completed);
    assert!(final_cdr.answer_time.is_some());
    assert!(final_cdr.end_time.is_some());
    assert!(final_cdr.setup_duration.is_some());
    assert!(final_cdr.call_duration.is_some());
    assert!(final_cdr.total_duration.is_some());
    assert_eq!(final_cdr.end_reason, Some("Normal clearing".to_string()));
    assert_eq!(final_cdr.sip_response_code, Some(200));

    cleanup_database(pool).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cdr_delete_older_than() {
    let pool = setup_database().await;
    let repo = PgCdrRepository::new(pool.clone());

    // Create a CDR
    let cdr = CallDetailRecord::new(
        "old-call".to_string(),
        "alice".to_string(),
        "sip:alice@example.com".to_string(),
        "192.168.1.100".to_string(),
        "bob".to_string(),
        "sip:bob@example.com".to_string(),
        CallDirection::Internal,
    );
    repo.create(&cdr).await.expect("Failed to create CDR");

    // Try to delete CDRs older than 1 day (should not delete the just-created one)
    let _deleted = repo.delete_older_than(1).await.expect("Failed to delete old CDRs");

    // The just-created CDR should still exist
    let retrieved = repo.get_by_id(cdr.id).await.expect("Failed to get CDR");
    assert!(retrieved.is_some());

    cleanup_database(pool).await;
}

// Helper functions

async fn setup_database() -> PgPool {
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
    pool
}

async fn cleanup_database(pool: PgPool) {
    // Clean up test data
    sqlx::query("DELETE FROM call_records WHERE call_id LIKE 'test-%' OR call_id LIKE 'call-%' OR call_id LIKE 'unique-%' OR call_id LIKE 'lifecycle-%' OR call_id LIKE 'old-%'")
        .execute(&pool)
        .await
        .ok();
    pool.close().await;
}

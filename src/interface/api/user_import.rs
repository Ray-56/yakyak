/// User bulk import API handler
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::domain::user::{CreateUser, UserRepository};
use crate::interface::api::router::AppState;

/// Bulk import result
#[derive(Debug, Serialize)]
pub struct BulkImportResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<ImportError>,
}

/// Import error record
#[derive(Debug, Serialize)]
pub struct ImportError {
    pub line: usize,
    pub username: String,
    pub error: String,
}

/// CSV user record
#[derive(Debug, Deserialize)]
struct CsvUserRecord {
    username: String,
    password: String,
    realm: String,
    display_name: Option<String>,
    email: Option<String>,
}

/// Handle bulk user import from CSV
pub async fn import_users_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("Handling bulk user import from CSV");

    // Get CSV file from multipart form
    let mut csv_content = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            match field.text().await {
                Ok(content) => csv_content = content,
                Err(e) => {
                    error!("Failed to read CSV file: {}", e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "Failed to read CSV file"
                        })),
                    )
                        .into_response();
                }
            }
        }
    }

    if csv_content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "No CSV file provided"
            })),
        )
            .into_response();
    }

    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    let mut total = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for (line_num, result) in reader.deserialize::<CsvUserRecord>().enumerate() {
        total += 1;
        let line = line_num + 2; // +2 because CSV is 1-indexed and first line is header

        match result {
            Ok(record) => {
                // Create user
                let create_user = CreateUser {
                    username: record.username.clone(),
                    password: record.password,
                    realm: record.realm,
                    display_name: record.display_name,
                    email: record.email,
                    role_id: None, // Default role
                };

                match state.user_repository.create(create_user).await {
                    Ok(user) => {
                        successful += 1;
                        info!("Imported user: {} (line {})", user.username, line);
                    }
                    Err(e) => {
                        failed += 1;
                        let error_msg = format!("{}", e);
                        warn!("Failed to import user {} (line {}): {}", record.username, line, error_msg);
                        errors.push(ImportError {
                            line,
                            username: record.username,
                            error: error_msg,
                        });
                    }
                }
            }
            Err(e) => {
                failed += 1;
                let error_msg = format!("CSV parse error: {}", e);
                warn!("Failed to parse line {}: {}", line, error_msg);
                errors.push(ImportError {
                    line,
                    username: "unknown".to_string(),
                    error: error_msg,
                });
            }
        }
    }

    let result = BulkImportResult {
        total,
        successful,
        failed,
        errors,
    };

    info!(
        "Bulk import complete: {} total, {} successful, {} failed",
        result.total, result.successful, result.failed
    );

    (StatusCode::OK, Json(result)).into_response()
}

/// Handle bulk user import from JSON
pub async fn import_users_json(
    State(state): State<AppState>,
    Json(users): Json<Vec<CreateUser>>,
) -> impl IntoResponse {
    info!("Handling bulk user import from JSON ({} users)", users.len());

    let mut total = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for (index, create_user) in users.into_iter().enumerate() {
        total += 1;
        let line = index + 1;
        let username = create_user.username.clone();

        match state.user_repository.create(create_user).await {
            Ok(user) => {
                successful += 1;
                info!("Imported user: {} (item {})", user.username, line);
            }
            Err(e) => {
                failed += 1;
                let error_msg = format!("{}", e);
                warn!("Failed to import user {} (item {}): {}", username, line, error_msg);
                errors.push(ImportError {
                    line,
                    username,
                    error: error_msg,
                });
            }
        }
    }

    let result = BulkImportResult {
        total,
        successful,
        failed,
        errors,
    };

    info!(
        "Bulk import complete: {} total, {} successful, {} failed",
        result.total, result.successful, result.failed
    );

    (StatusCode::OK, Json(result)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing() {
        let csv_data = "\
username,password,realm,display_name,email
alice,secret123,example.com,Alice Smith,alice@example.com
bob,secret456,example.com,Bob Jones,bob@example.com
";

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());

        let mut count = 0;
        for result in reader.deserialize::<CsvUserRecord>() {
            assert!(result.is_ok());
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn test_csv_parsing_optional_fields() {
        let csv_data = "\
username,password,realm,display_name,email
alice,secret123,example.com,,
bob,secret456,example.com,Bob Jones,
";

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());

        let records: Vec<CsvUserRecord> = reader
            .deserialize()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].display_name, None);
        assert_eq!(records[0].email, None);
        assert_eq!(records[1].display_name, Some("Bob Jones".to_string()));
        assert_eq!(records[1].email, None);
    }
}

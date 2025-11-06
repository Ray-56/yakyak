//! CDR API handlers

use super::cdr_dto::{ApiResponse, CdrListResponse, CdrResponse};
use super::user_handler::AppState;
use crate::domain::cdr::{CdrFilters, CallDirection, CallStatus};
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

/// Query parameters for listing CDRs
#[derive(Debug, Deserialize)]
pub struct ListCdrsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub caller_username: Option<String>,
    pub callee_username: Option<String>,
    pub direction: Option<String>,
    pub status: Option<String>,
    pub start_time_from: Option<DateTime<Utc>>,
    pub start_time_to: Option<DateTime<Utc>>,
    pub min_duration: Option<i32>,
}

fn default_limit() -> i64 {
    100
}

/// Get CDR by ID
pub async fn get_cdr(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<CdrResponse>>, StatusCode> {
    info!("API: Getting CDR ID: {}", id);

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Ok(Json(ApiResponse::error("CDR repository not available".to_string())));
        }
    };

    match cdr_repo.get_by_id(id).await {
        Ok(Some(cdr)) => Ok(Json(ApiResponse::success(cdr.into()))),
        Ok(None) => Ok(Json(ApiResponse::error(format!("CDR {} not found", id)))),
        Err(e) => {
            error!("API: Failed to get CDR: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get CDR by Call-ID
pub async fn get_cdr_by_call_id(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<ApiResponse<CdrResponse>>, StatusCode> {
    info!("API: Getting CDR by Call-ID: {}", call_id);

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Ok(Json(ApiResponse::error("CDR repository not available".to_string())));
        }
    };

    match cdr_repo.get_by_call_id(&call_id).await {
        Ok(Some(cdr)) => Ok(Json(ApiResponse::success(cdr.into()))),
        Ok(None) => Ok(Json(ApiResponse::error(format!(
            "CDR for Call-ID {} not found",
            call_id
        )))),
        Err(e) => {
            error!("API: Failed to get CDR: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List CDRs
pub async fn list_cdrs(
    State(state): State<AppState>,
    Query(query): Query<ListCdrsQuery>,
) -> Result<Json<ApiResponse<CdrListResponse>>, StatusCode> {
    info!(
        "API: Listing CDRs (limit: {}, offset: {})",
        query.limit, query.offset
    );

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Ok(Json(ApiResponse::error("CDR repository not available".to_string())));
        }
    };

    // Build filters
    let mut filters = CdrFilters::default();
    filters.caller_username = query.caller_username;
    filters.callee_username = query.callee_username;
    filters.start_time_from = query.start_time_from;
    filters.start_time_to = query.start_time_to;
    filters.min_duration = query.min_duration;

    // Parse direction
    if let Some(ref dir_str) = query.direction {
        filters.direction = match dir_str.as_str() {
            "inbound" => Some(CallDirection::Inbound),
            "outbound" => Some(CallDirection::Outbound),
            "internal" => Some(CallDirection::Internal),
            _ => None,
        };
    }

    // Parse status
    if let Some(ref status_str) = query.status {
        filters.status = CallStatus::from_str(status_str);
    }

    // Get CDRs
    let cdrs_result = cdr_repo.list(filters.clone(), query.limit, query.offset).await;

    // Get total count
    let count_result = cdr_repo.count(filters).await;

    match (cdrs_result, count_result) {
        (Ok(cdrs), Ok(total)) => {
            let response = CdrListResponse {
                cdrs: cdrs.into_iter().map(|c| c.into()).collect(),
                total,
                limit: query.limit,
                offset: query.offset,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("API: Failed to list CDRs: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Export CDRs as CSV
pub async fn export_cdrs_csv(
    State(state): State<AppState>,
    Query(query): Query<ListCdrsQuery>,
) -> Result<Response, StatusCode> {
    info!("API: Exporting CDRs as CSV");

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // Build filters
    let mut filters = CdrFilters::default();
    filters.caller_username = query.caller_username;
    filters.callee_username = query.callee_username;
    filters.start_time_from = query.start_time_from;
    filters.start_time_to = query.start_time_to;
    filters.min_duration = query.min_duration;

    // Parse direction
    if let Some(ref dir_str) = query.direction {
        filters.direction = match dir_str.as_str() {
            "inbound" => Some(CallDirection::Inbound),
            "outbound" => Some(CallDirection::Outbound),
            "internal" => Some(CallDirection::Internal),
            _ => None,
        };
    }

    // Parse status
    if let Some(ref status_str) = query.status {
        filters.status = CallStatus::from_str(status_str);
    }

    // Get CDRs (export all matching records, not paginated)
    let cdrs = match cdr_repo.list(filters, 10000, 0).await {
        Ok(cdrs) => cdrs,
        Err(e) => {
            error!("API: Failed to export CDRs: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Build CSV
    let mut csv_content = String::new();

    // CSV Header
    csv_content.push_str("id,call_id,caller_username,caller_uri,caller_ip,callee_username,callee_uri,callee_ip,direction,start_time,answer_time,end_time,setup_duration,call_duration,total_duration,status,end_reason,sip_response_code,codec,rtp_packets_sent,rtp_packets_received,rtp_bytes_sent,rtp_bytes_received,created_at,updated_at\n");

    // CSV Rows
    for cdr in cdrs {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            cdr.id,
            escape_csv(&cdr.call_id),
            escape_csv(&cdr.caller_username),
            escape_csv(&cdr.caller_uri),
            escape_csv(&cdr.caller_ip),
            escape_csv(&cdr.callee_username),
            escape_csv(&cdr.callee_uri),
            cdr.callee_ip.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
            cdr.direction.as_str(),
            cdr.start_time.to_rfc3339(),
            cdr.answer_time.as_ref().map(|t| t.to_rfc3339()).unwrap_or_default(),
            cdr.end_time.as_ref().map(|t| t.to_rfc3339()).unwrap_or_default(),
            cdr.setup_duration.map(|d| d.to_string()).unwrap_or_default(),
            cdr.call_duration.map(|d| d.to_string()).unwrap_or_default(),
            cdr.total_duration.map(|d| d.to_string()).unwrap_or_default(),
            cdr.status.as_str(),
            cdr.end_reason.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
            cdr.sip_response_code.map(|c| c.to_string()).unwrap_or_default(),
            cdr.codec.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
            cdr.rtp_packets_sent.map(|p| p.to_string()).unwrap_or_default(),
            cdr.rtp_packets_received.map(|p| p.to_string()).unwrap_or_default(),
            cdr.rtp_bytes_sent.map(|b| b.to_string()).unwrap_or_default(),
            cdr.rtp_bytes_received.map(|b| b.to_string()).unwrap_or_default(),
            cdr.created_at.to_rfc3339(),
            cdr.updated_at.to_rfc3339(),
        ));
    }

    // Return CSV response
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"cdrs.csv\""),
        ],
        csv_content,
    ).into_response())
}

/// Export CDRs as JSON
pub async fn export_cdrs_json(
    State(state): State<AppState>,
    Query(query): Query<ListCdrsQuery>,
) -> Result<Response, StatusCode> {
    info!("API: Exporting CDRs as JSON");

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // Build filters
    let mut filters = CdrFilters::default();
    filters.caller_username = query.caller_username;
    filters.callee_username = query.callee_username;
    filters.start_time_from = query.start_time_from;
    filters.start_time_to = query.start_time_to;
    filters.min_duration = query.min_duration;

    // Parse direction
    if let Some(ref dir_str) = query.direction {
        filters.direction = match dir_str.as_str() {
            "inbound" => Some(CallDirection::Inbound),
            "outbound" => Some(CallDirection::Outbound),
            "internal" => Some(CallDirection::Internal),
            _ => None,
        };
    }

    // Parse status
    if let Some(ref status_str) = query.status {
        filters.status = CallStatus::from_str(status_str);
    }

    // Get CDRs (export all matching records, not paginated)
    let cdrs = match cdr_repo.list(filters, 10000, 0).await {
        Ok(cdrs) => cdrs,
        Err(e) => {
            error!("API: Failed to export CDRs: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert to response DTOs
    let cdr_responses: Vec<CdrResponse> = cdrs.into_iter().map(|c| c.into()).collect();

    // Serialize to JSON
    let json_content = match serde_json::to_string_pretty(&cdr_responses) {
        Ok(json) => json,
        Err(e) => {
            error!("API: Failed to serialize CDRs to JSON: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Return JSON response
    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"cdrs.json\""),
        ],
        json_content,
    ).into_response())
}

/// Escape CSV field
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

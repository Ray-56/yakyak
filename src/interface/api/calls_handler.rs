//! Call Management API handlers

use super::cdr_dto::ApiResponse;
use super::user_handler::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Call statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct CallStatsResponse {
    pub total_active_calls: usize,
    pub total_calls_today: i64,
    pub total_completed_calls: i64,
    pub total_failed_calls: i64,
    pub average_call_duration: i32,
}

/// Active calls list response
#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveCallsResponse {
    pub calls: Vec<crate::infrastructure::protocols::sip::ActiveCallInfo>,
    pub total: usize,
}

/// Get active calls
pub async fn get_active_calls(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ActiveCallsResponse>>, StatusCode> {
    info!("API: Getting active calls");

    let call_router = match &state.call_router {
        Some(router) => router,
        None => {
            error!("Call router not available");
            return Ok(Json(ApiResponse::error(
                "Call router not available".to_string(),
            )));
        }
    };

    let calls = call_router.get_active_calls().await;
    let total = calls.len();

    let response = ActiveCallsResponse { calls, total };

    Ok(Json(ApiResponse::success(response)))
}

/// Get active call by ID
pub async fn get_active_call(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<ApiResponse<crate::infrastructure::protocols::sip::ActiveCallInfo>>, StatusCode> {
    info!("API: Getting active call ID: {}", call_id);

    let call_router = match &state.call_router {
        Some(router) => router,
        None => {
            error!("Call router not available");
            return Ok(Json(ApiResponse::error(
                "Call router not available".to_string(),
            )));
        }
    };

    match call_router.get_active_call(&call_id).await {
        Some(call) => Ok(Json(ApiResponse::success(call))),
        None => Ok(Json(ApiResponse::error(format!(
            "Call {} not found",
            call_id
        )))),
    }
}

/// Hangup call
pub async fn hangup_call(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("API: Hanging up call ID: {}", call_id);

    let call_router = match &state.call_router {
        Some(router) => router,
        None => {
            error!("Call router not available");
            return Ok(Json(ApiResponse::error(
                "Call router not available".to_string(),
            )));
        }
    };

    match call_router.hangup_call(&call_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(format!(
            "Call {} hung up successfully",
            call_id
        )))),
        Err(e) => {
            error!("API: Failed to hangup call: {}", e);
            Ok(Json(ApiResponse::error(e)))
        }
    }
}

/// Get call statistics
pub async fn get_call_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<CallStatsResponse>>, StatusCode> {
    info!("API: Getting call statistics");

    let call_router = match &state.call_router {
        Some(router) => router,
        None => {
            error!("Call router not available");
            return Ok(Json(ApiResponse::error(
                "Call router not available".to_string(),
            )));
        }
    };

    let cdr_repo = match &state.cdr_repository {
        Some(repo) => repo,
        None => {
            error!("CDR repository not available");
            return Ok(Json(ApiResponse::error(
                "CDR repository not available".to_string(),
            )));
        }
    };

    // Get active calls count
    let total_active_calls = call_router.active_call_count().await;

    // Get today's date range
    let today_start = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    // Build filter for today's calls
    let mut filters = crate::domain::cdr::CdrFilters::default();
    filters.start_time_from = Some(today_start);

    // Get total calls today
    let total_calls_today = match cdr_repo.count(filters.clone()).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count CDRs: {}", e);
            0
        }
    };

    // Get completed calls today
    filters.status = Some(crate::domain::cdr::CallStatus::Completed);
    let total_completed_calls = match cdr_repo.count(filters.clone()).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count completed CDRs: {}", e);
            0
        }
    };

    // Get failed calls today
    filters.status = Some(crate::domain::cdr::CallStatus::Failed);
    let total_failed_calls = match cdr_repo.count(filters).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count failed CDRs: {}", e);
            0
        }
    };

    // Calculate average call duration
    // For simplicity, we'll query completed calls and calculate average
    let mut avg_filters = crate::domain::cdr::CdrFilters::default();
    avg_filters.start_time_from = Some(today_start);
    avg_filters.status = Some(crate::domain::cdr::CallStatus::Completed);

    let avg_duration = match cdr_repo.list(avg_filters, 1000, 0).await {
        Ok(cdrs) => {
            if cdrs.is_empty() {
                0
            } else {
                let total: i32 = cdrs
                    .iter()
                    .filter_map(|cdr| cdr.call_duration)
                    .sum();
                total / cdrs.len() as i32
            }
        }
        Err(e) => {
            error!("Failed to calculate average duration: {}", e);
            0
        }
    };

    let stats = CallStatsResponse {
        total_active_calls,
        total_calls_today,
        total_completed_calls,
        total_failed_calls,
        average_call_duration: avg_duration,
    };

    Ok(Json(ApiResponse::success(stats)))
}

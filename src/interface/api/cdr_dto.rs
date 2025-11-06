//! CDR API DTOs

use crate::domain::cdr::CallDetailRecord;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CDR response
#[derive(Debug, Serialize, Deserialize)]
pub struct CdrResponse {
    pub id: Uuid,
    pub call_id: String,
    pub caller_username: String,
    pub caller_uri: String,
    pub caller_ip: String,
    pub callee_username: String,
    pub callee_uri: String,
    pub callee_ip: Option<String>,
    pub direction: String,
    pub start_time: DateTime<Utc>,
    pub answer_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub setup_duration: Option<i32>,
    pub call_duration: Option<i32>,
    pub total_duration: Option<i32>,
    pub status: String,
    pub end_reason: Option<String>,
    pub sip_response_code: Option<u16>,
    pub codec: Option<String>,
    pub rtp_packets_sent: Option<i64>,
    pub rtp_packets_received: Option<i64>,
    pub rtp_bytes_sent: Option<i64>,
    pub rtp_bytes_received: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CallDetailRecord> for CdrResponse {
    fn from(cdr: CallDetailRecord) -> Self {
        CdrResponse {
            id: cdr.id,
            call_id: cdr.call_id,
            caller_username: cdr.caller_username,
            caller_uri: cdr.caller_uri,
            caller_ip: cdr.caller_ip,
            callee_username: cdr.callee_username,
            callee_uri: cdr.callee_uri,
            callee_ip: cdr.callee_ip,
            direction: cdr.direction.as_str().to_string(),
            start_time: cdr.start_time,
            answer_time: cdr.answer_time,
            end_time: cdr.end_time,
            setup_duration: cdr.setup_duration,
            call_duration: cdr.call_duration,
            total_duration: cdr.total_duration,
            status: cdr.status.as_str().to_string(),
            end_reason: cdr.end_reason,
            sip_response_code: cdr.sip_response_code,
            codec: cdr.codec,
            rtp_packets_sent: cdr.rtp_packets_sent,
            rtp_packets_received: cdr.rtp_packets_received,
            rtp_bytes_sent: cdr.rtp_bytes_sent,
            rtp_bytes_received: cdr.rtp_bytes_received,
            created_at: cdr.created_at,
            updated_at: cdr.updated_at,
        }
    }
}

/// CDR list response
#[derive(Debug, Serialize, Deserialize)]
pub struct CdrListResponse {
    pub cdrs: Vec<CdrResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Generic API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

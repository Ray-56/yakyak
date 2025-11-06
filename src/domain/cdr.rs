//! Call Detail Record (CDR) domain model
//!
//! CDR captures information about each call for billing, auditing, and analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Call Detail Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallDetailRecord {
    /// Unique CDR ID
    pub id: Uuid,

    /// SIP Call-ID
    pub call_id: String,

    /// Caller information
    pub caller_username: String,
    pub caller_uri: String,
    pub caller_ip: String,

    /// Callee information
    pub callee_username: String,
    pub callee_uri: String,
    pub callee_ip: Option<String>,

    /// Call direction
    pub direction: CallDirection,

    /// Time information
    pub start_time: DateTime<Utc>,
    pub answer_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,

    /// Duration in seconds
    pub setup_duration: Option<i32>,
    pub call_duration: Option<i32>,
    pub total_duration: Option<i32>,

    /// Call status and result
    pub status: CallStatus,
    pub end_reason: Option<String>,
    pub sip_response_code: Option<u16>,

    /// Media information
    pub codec: Option<String>,
    pub rtp_packets_sent: Option<i64>,
    pub rtp_packets_received: Option<i64>,
    pub rtp_bytes_sent: Option<i64>,
    pub rtp_bytes_received: Option<i64>,

    /// Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Call direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallDirection {
    /// Inbound call from external
    Inbound,
    /// Outbound call to external
    Outbound,
    /// Internal call between registered users
    Internal,
}

impl CallDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            CallDirection::Inbound => "inbound",
            CallDirection::Outbound => "outbound",
            CallDirection::Internal => "internal",
        }
    }
}

/// Call status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    /// Call in progress
    Active,
    /// Call completed successfully
    Completed,
    /// Call failed (connection issues, etc.)
    Failed,
    /// Callee was busy
    Busy,
    /// Call was not answered
    NoAnswer,
    /// Call was cancelled
    Cancelled,
    /// Call was rejected/declined
    Rejected,
}

impl CallStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CallStatus::Active => "active",
            CallStatus::Completed => "completed",
            CallStatus::Failed => "failed",
            CallStatus::Busy => "busy",
            CallStatus::NoAnswer => "no_answer",
            CallStatus::Cancelled => "cancelled",
            CallStatus::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(CallStatus::Active),
            "completed" => Some(CallStatus::Completed),
            "failed" => Some(CallStatus::Failed),
            "busy" => Some(CallStatus::Busy),
            "no_answer" => Some(CallStatus::NoAnswer),
            "cancelled" => Some(CallStatus::Cancelled),
            "rejected" => Some(CallStatus::Rejected),
            _ => None,
        }
    }
}

impl CallDetailRecord {
    /// Create a new CDR for an initiated call
    pub fn new(
        call_id: String,
        caller_username: String,
        caller_uri: String,
        caller_ip: String,
        callee_username: String,
        callee_uri: String,
        direction: CallDirection,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            call_id,
            caller_username,
            caller_uri,
            caller_ip,
            callee_username,
            callee_uri,
            callee_ip: None,
            direction,
            start_time: now,
            answer_time: None,
            end_time: None,
            setup_duration: None,
            call_duration: None,
            total_duration: None,
            status: CallStatus::Active,
            end_reason: None,
            sip_response_code: None,
            codec: None,
            rtp_packets_sent: None,
            rtp_packets_received: None,
            rtp_bytes_sent: None,
            rtp_bytes_received: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark the call as answered
    pub fn mark_answered(&mut self) {
        let now = Utc::now();
        self.answer_time = Some(now);
        self.setup_duration = Some((now - self.start_time).num_seconds() as i32);
        self.updated_at = now;
    }

    /// Mark the call as ended
    pub fn mark_ended(&mut self, status: CallStatus, reason: Option<String>, response_code: Option<u16>) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.status = status;
        self.end_reason = reason;
        self.sip_response_code = response_code;
        self.total_duration = Some((now - self.start_time).num_seconds() as i32);

        if let Some(answer_time) = self.answer_time {
            self.call_duration = Some((now - answer_time).num_seconds() as i32);
        }

        self.updated_at = now;
    }

    /// Set media information
    pub fn set_media_info(
        &mut self,
        codec: Option<String>,
        packets_sent: Option<i64>,
        packets_received: Option<i64>,
        bytes_sent: Option<i64>,
        bytes_received: Option<i64>,
    ) {
        self.codec = codec;
        self.rtp_packets_sent = packets_sent;
        self.rtp_packets_received = packets_received;
        self.rtp_bytes_sent = bytes_sent;
        self.rtp_bytes_received = bytes_received;
        self.updated_at = Utc::now();
    }

    /// Set callee IP address
    pub fn set_callee_ip(&mut self, ip: String) {
        self.callee_ip = Some(ip);
        self.updated_at = Utc::now();
    }
}

/// CDR Repository trait
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait CdrRepository: Send + Sync {
    /// Create a new CDR
    async fn create(&self, cdr: &CallDetailRecord) -> Result<(), String>;

    /// Update an existing CDR
    async fn update(&self, cdr: &CallDetailRecord) -> Result<(), String>;

    /// Get CDR by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<CallDetailRecord>, String>;

    /// Get CDR by Call-ID
    async fn get_by_call_id(&self, call_id: &str) -> Result<Option<CallDetailRecord>, String>;

    /// List CDRs with filters
    async fn list(
        &self,
        filters: CdrFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CallDetailRecord>, String>;

    /// Count CDRs with filters
    async fn count(&self, filters: CdrFilters) -> Result<i64, String>;

    /// Delete old CDRs (for cleanup)
    async fn delete_older_than(&self, days: i32) -> Result<i64, String>;
}

/// Filters for CDR queries
#[derive(Debug, Clone, Default)]
pub struct CdrFilters {
    pub caller_username: Option<String>,
    pub callee_username: Option<String>,
    pub direction: Option<CallDirection>,
    pub status: Option<CallStatus>,
    pub start_time_from: Option<DateTime<Utc>>,
    pub start_time_to: Option<DateTime<Utc>>,
    pub min_duration: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdr_creation() {
        let cdr = CallDetailRecord::new(
            "test-call-123".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        assert_eq!(cdr.call_id, "test-call-123");
        assert_eq!(cdr.caller_username, "alice");
        assert_eq!(cdr.callee_username, "bob");
        assert_eq!(cdr.direction, CallDirection::Internal);
        assert_eq!(cdr.status, CallStatus::Active);
        assert!(cdr.answer_time.is_none());
        assert!(cdr.end_time.is_none());
    }

    #[test]
    fn test_cdr_answered() {
        let mut cdr = CallDetailRecord::new(
            "test-call-123".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        std::thread::sleep(std::time::Duration::from_millis(10));
        cdr.mark_answered();

        assert!(cdr.answer_time.is_some());
        assert!(cdr.setup_duration.is_some());
        assert!(cdr.setup_duration.unwrap() >= 0);
    }

    #[test]
    fn test_cdr_ended() {
        let mut cdr = CallDetailRecord::new(
            "test-call-123".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        std::thread::sleep(std::time::Duration::from_millis(10));
        cdr.mark_answered();

        std::thread::sleep(std::time::Duration::from_millis(10));
        cdr.mark_ended(CallStatus::Completed, Some("Normal clearing".to_string()), Some(200));

        assert_eq!(cdr.status, CallStatus::Completed);
        assert_eq!(cdr.end_reason, Some("Normal clearing".to_string()));
        assert_eq!(cdr.sip_response_code, Some(200));
        assert!(cdr.end_time.is_some());
        assert!(cdr.total_duration.is_some());
        assert!(cdr.call_duration.is_some());
    }

    #[test]
    fn test_call_status_conversion() {
        assert_eq!(CallStatus::Completed.as_str(), "completed");
        assert_eq!(CallStatus::from_str("completed"), Some(CallStatus::Completed));
        assert_eq!(CallStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_call_direction() {
        assert_eq!(CallDirection::Internal.as_str(), "internal");
        assert_eq!(CallDirection::Inbound.as_str(), "inbound");
        assert_eq!(CallDirection::Outbound.as_str(), "outbound");
    }

    #[test]
    fn test_cdr_set_media_info() {
        let mut cdr = CallDetailRecord::new(
            "test-media".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        cdr.set_media_info(
            Some("PCMU".to_string()),
            Some(1000),
            Some(950),
            Some(160000),
            Some(152000),
        );

        assert_eq!(cdr.codec, Some("PCMU".to_string()));
        assert_eq!(cdr.rtp_packets_sent, Some(1000));
        assert_eq!(cdr.rtp_packets_received, Some(950));
        assert_eq!(cdr.rtp_bytes_sent, Some(160000));
        assert_eq!(cdr.rtp_bytes_received, Some(152000));
    }

    #[test]
    fn test_cdr_set_callee_ip() {
        let mut cdr = CallDetailRecord::new(
            "test-callee-ip".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        assert!(cdr.callee_ip.is_none());

        cdr.set_callee_ip("192.168.1.101".to_string());

        assert_eq!(cdr.callee_ip, Some("192.168.1.101".to_string()));
    }

    #[test]
    fn test_cdr_rejected_status() {
        let mut cdr = CallDetailRecord::new(
            "test-reject".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Outbound,
        );

        cdr.mark_ended(CallStatus::Rejected, Some("User declined".to_string()), Some(603));

        assert_eq!(cdr.status, CallStatus::Rejected);
        assert_eq!(cdr.end_reason, Some("User declined".to_string()));
        assert_eq!(cdr.sip_response_code, Some(603));
        assert!(cdr.end_time.is_some());
        assert!(cdr.total_duration.is_some());
        // Call was not answered, so call_duration should be None
        assert!(cdr.call_duration.is_none());
    }

    #[test]
    fn test_cdr_busy_status() {
        let mut cdr = CallDetailRecord::new(
            "test-busy".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Outbound,
        );

        cdr.mark_ended(CallStatus::Busy, Some("User busy".to_string()), Some(486));

        assert_eq!(cdr.status, CallStatus::Busy);
        assert_eq!(cdr.end_reason, Some("User busy".to_string()));
        assert_eq!(cdr.sip_response_code, Some(486));
    }

    #[test]
    fn test_cdr_cancelled_status() {
        let mut cdr = CallDetailRecord::new(
            "test-cancel".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Outbound,
        );

        cdr.mark_ended(CallStatus::Cancelled, Some("Call cancelled".to_string()), Some(487));

        assert_eq!(cdr.status, CallStatus::Cancelled);
        assert_eq!(cdr.end_reason, Some("Call cancelled".to_string()));
        assert_eq!(cdr.sip_response_code, Some(487));
    }

    #[test]
    fn test_cdr_no_answer_status() {
        let mut cdr = CallDetailRecord::new(
            "test-no-answer".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Outbound,
        );

        cdr.mark_ended(CallStatus::NoAnswer, Some("No answer".to_string()), Some(408));

        assert_eq!(cdr.status, CallStatus::NoAnswer);
        assert_eq!(cdr.end_reason, Some("No answer".to_string()));
        assert_eq!(cdr.sip_response_code, Some(408));
    }

    #[test]
    fn test_cdr_duration_calculation() {
        let mut cdr = CallDetailRecord::new(
            "test-duration".to_string(),
            "alice".to_string(),
            "sip:alice@example.com".to_string(),
            "192.168.1.100".to_string(),
            "bob".to_string(),
            "sip:bob@example.com".to_string(),
            CallDirection::Internal,
        );

        std::thread::sleep(std::time::Duration::from_millis(20));
        cdr.mark_answered();

        std::thread::sleep(std::time::Duration::from_millis(30));
        cdr.mark_ended(CallStatus::Completed, Some("Normal".to_string()), Some(200));

        assert!(cdr.setup_duration.is_some());
        assert!(cdr.call_duration.is_some());
        assert!(cdr.total_duration.is_some());

        // Total should be >= setup + call
        let total = cdr.total_duration.unwrap();
        let setup = cdr.setup_duration.unwrap();
        let call = cdr.call_duration.unwrap();

        assert!(total >= setup);
        assert!(total >= call);
    }

    #[test]
    fn test_cdr_filters_default() {
        let filters = CdrFilters::default();

        assert!(filters.caller_username.is_none());
        assert!(filters.callee_username.is_none());
        assert!(filters.direction.is_none());
        assert!(filters.status.is_none());
        assert!(filters.start_time_from.is_none());
        assert!(filters.start_time_to.is_none());
        assert!(filters.min_duration.is_none());
    }

    #[test]
    fn test_call_status_all_variants() {
        assert_eq!(CallStatus::Active.as_str(), "active");
        assert_eq!(CallStatus::Completed.as_str(), "completed");
        assert_eq!(CallStatus::Failed.as_str(), "failed");
        assert_eq!(CallStatus::Busy.as_str(), "busy");
        assert_eq!(CallStatus::NoAnswer.as_str(), "no_answer");
        assert_eq!(CallStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(CallStatus::Rejected.as_str(), "rejected");

        // Test from_str for all variants
        assert_eq!(CallStatus::from_str("active"), Some(CallStatus::Active));
        assert_eq!(CallStatus::from_str("completed"), Some(CallStatus::Completed));
        assert_eq!(CallStatus::from_str("failed"), Some(CallStatus::Failed));
        assert_eq!(CallStatus::from_str("busy"), Some(CallStatus::Busy));
        assert_eq!(CallStatus::from_str("no_answer"), Some(CallStatus::NoAnswer));
        assert_eq!(CallStatus::from_str("cancelled"), Some(CallStatus::Cancelled));
        assert_eq!(CallStatus::from_str("rejected"), Some(CallStatus::Rejected));
        assert_eq!(CallStatus::from_str("unknown"), None);
    }
}

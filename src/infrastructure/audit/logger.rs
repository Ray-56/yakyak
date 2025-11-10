/// Audit logging for security events and compliance
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Critical,
}

/// Types of auditable events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEventType {
    /// Authentication events
    AuthenticationSuccess { username: String, method: String },
    AuthenticationFailure { username: Option<String>, method: String, reason: String },
    AuthenticationLockout { username: String, ip: String },

    /// User management events
    UserCreated { username: String, created_by: String },
    UserUpdated { username: String, updated_by: String, fields: Vec<String> },
    UserDeleted { username: String, deleted_by: String },
    UserPasswordChanged { username: String, changed_by: String },

    /// Role and permission events
    RoleCreated { role_name: String, created_by: String },
    RoleUpdated { role_name: String, updated_by: String },
    RoleDeleted { role_name: String, deleted_by: String },
    RoleAssigned { username: String, role_name: String, assigned_by: String },

    /// Call events
    CallInitiated { caller: String, callee: String, call_id: String },
    CallAnswered { caller: String, callee: String, call_id: String },
    CallTerminated { caller: String, callee: String, call_id: String, duration: u64 },
    CallFailed { caller: String, callee: String, reason: String },

    /// Conference events
    ConferenceCreated { name: String, created_by: String },
    ConferenceJoined { conference_id: Uuid, username: String },
    ConferenceLeft { conference_id: Uuid, username: String },
    ParticipantMuted { conference_id: Uuid, participant: String, muted_by: String },

    /// System configuration events
    ConfigurationChanged { setting: String, changed_by: String, old_value: String, new_value: String },

    /// Security events
    UnauthorizedAccess { resource: String, username: Option<String>, ip: String },
    RateLimitExceeded { ip: String, endpoint: String },
    SuspiciousActivity { description: String, username: Option<String>, ip: String },

    /// Data access events
    DataExported { data_type: String, username: String, record_count: usize },
    DataDeleted { data_type: String, username: String, record_count: usize },

    /// Custom event
    Custom { event_name: String, details: HashMap<String, String> },
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: AuditLevel,
    pub event_type: AuditEventType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl AuditEvent {
    pub fn new(level: AuditLevel, event_type: AuditEventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            event_type,
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Audit logger backend trait
#[async_trait::async_trait]
pub trait AuditBackend: Send + Sync {
    async fn log(&self, event: &AuditEvent) -> Result<(), String>;
    async fn query(&self, filters: AuditQuery) -> Result<Vec<AuditEvent>, String>;
}

/// Query filters for audit log searches
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub level: Option<AuditLevel>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub limit: Option<usize>,
}

/// In-memory audit backend
pub struct MemoryAuditBackend {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
}

impl MemoryAuditBackend {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }
}

#[async_trait::async_trait]
impl AuditBackend for MemoryAuditBackend {
    async fn log(&self, event: &AuditEvent) -> Result<(), String> {
        let mut events = self.events.write().await;

        // Add new event
        events.push(event.clone());

        // Maintain max size (FIFO)
        while events.len() > self.max_events {
            events.remove(0);
        }

        Ok(())
    }

    async fn query(&self, filters: AuditQuery) -> Result<Vec<AuditEvent>, String> {
        let events = self.events.read().await;
        let mut results: Vec<AuditEvent> = events
            .iter()
            .filter(|event| {
                // Filter by time range
                if let Some(start) = filters.start_time {
                    if event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = filters.end_time {
                    if event.timestamp > end {
                        return false;
                    }
                }

                // Filter by level
                if let Some(level) = filters.level {
                    if event.level != level {
                        return false;
                    }
                }

                // Filter by IP address
                if let Some(ref ip) = filters.ip_address {
                    if event.ip_address.as_ref() != Some(ip) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = filters.limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}

/// Main audit logger
pub struct AuditLogger {
    backend: Arc<dyn AuditBackend>,
}

impl AuditLogger {
    pub fn new(backend: Arc<dyn AuditBackend>) -> Self {
        Self { backend }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        // Log to tracing as well
        match event.level {
            AuditLevel::Info => info!(
                "AUDIT: {:?} [{}]",
                event.event_type,
                event.id
            ),
            AuditLevel::Warning => warn!(
                "AUDIT: {:?} [{}]",
                event.event_type,
                event.id
            ),
            AuditLevel::Critical => warn!(
                "CRITICAL AUDIT: {:?} [{}]",
                event.event_type,
                event.id
            ),
        }

        // Store in backend
        if let Err(e) = self.backend.log(&event).await {
            warn!("Failed to log audit event: {}", e);
        }
    }

    /// Query audit logs
    pub async fn query(&self, filters: AuditQuery) -> Result<Vec<AuditEvent>, String> {
        self.backend.query(filters).await
    }

    /// Convenience methods for common events
    pub async fn log_auth_success(&self, username: String, method: String, ip: String) {
        let event = AuditEvent::new(
            AuditLevel::Info,
            AuditEventType::AuthenticationSuccess { username, method },
        )
        .with_ip(ip);
        self.log(event).await;
    }

    pub async fn log_auth_failure(
        &self,
        username: Option<String>,
        method: String,
        reason: String,
        ip: String,
    ) {
        let event = AuditEvent::new(
            AuditLevel::Warning,
            AuditEventType::AuthenticationFailure {
                username,
                method,
                reason,
            },
        )
        .with_ip(ip);
        self.log(event).await;
    }

    pub async fn log_auth_lockout(&self, username: String, ip: String) {
        let event = AuditEvent::new(
            AuditLevel::Critical,
            AuditEventType::AuthenticationLockout {
                username: username.clone(),
                ip: ip.clone(),
            },
        )
        .with_ip(ip);
        self.log(event).await;
    }

    pub async fn log_user_created(&self, username: String, created_by: String, ip: String) {
        let event = AuditEvent::new(
            AuditLevel::Info,
            AuditEventType::UserCreated {
                username,
                created_by,
            },
        )
        .with_ip(ip);
        self.log(event).await;
    }

    pub async fn log_unauthorized_access(
        &self,
        resource: String,
        username: Option<String>,
        ip: String,
    ) {
        let event = AuditEvent::new(
            AuditLevel::Critical,
            AuditEventType::UnauthorizedAccess {
                resource,
                username,
                ip: ip.clone(),
            },
        )
        .with_ip(ip);
        self.log(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditLevel::Info,
            AuditEventType::AuthenticationSuccess {
                username: "alice".to_string(),
                method: "REGISTER".to_string(),
            },
        )
        .with_ip("192.168.1.100".to_string());

        assert_eq!(event.level, AuditLevel::Info);
        assert_eq!(event.ip_address, Some("192.168.1.100".to_string()));
    }

    #[tokio::test]
    async fn test_memory_audit_backend() {
        let backend = Arc::new(MemoryAuditBackend::new(100));
        let logger = AuditLogger::new(backend.clone());

        logger
            .log_auth_success(
                "alice".to_string(),
                "REGISTER".to_string(),
                "192.168.1.100".to_string(),
            )
            .await;

        let query = AuditQuery {
            ip_address: Some("192.168.1.100".to_string()),
            ..Default::default()
        };

        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_query_filtering() {
        let backend = Arc::new(MemoryAuditBackend::new(100));
        let logger = AuditLogger::new(backend.clone());

        // Log multiple events
        logger
            .log_auth_success(
                "alice".to_string(),
                "REGISTER".to_string(),
                "192.168.1.100".to_string(),
            )
            .await;

        logger
            .log_auth_failure(
                Some("bob".to_string()),
                "REGISTER".to_string(),
                "Invalid credentials".to_string(),
                "192.168.1.101".to_string(),
            )
            .await;

        logger
            .log_auth_success(
                "charlie".to_string(),
                "REGISTER".to_string(),
                "192.168.1.102".to_string(),
            )
            .await;

        // Query by level
        let query = AuditQuery {
            level: Some(AuditLevel::Warning),
            ..Default::default()
        };

        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 1);

        // Query all
        let query = AuditQuery::default();
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_audit_max_events() {
        let backend = Arc::new(MemoryAuditBackend::new(2)); // Max 2 events
        let logger = AuditLogger::new(backend.clone());

        // Log 3 events
        for i in 0..3 {
            logger
                .log_auth_success(
                    format!("user{}", i),
                    "REGISTER".to_string(),
                    "192.168.1.100".to_string(),
                )
                .await;
        }

        // Should only have 2 events (oldest dropped)
        let query = AuditQuery::default();
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}

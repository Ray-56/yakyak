/// Audit logging system for security and compliance
pub mod logger;

pub use logger::{AuditLogger, AuditEvent, AuditEventType, AuditLevel};

//! Domain events infrastructure

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    /// Returns the event type name
    fn event_type(&self) -> &'static str;

    /// Returns when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub event_type: String,
}

impl EventMetadata {
    pub fn new(event_type: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            event_type,
        }
    }
}

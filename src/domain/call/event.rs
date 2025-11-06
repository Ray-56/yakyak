//! Call domain events

use crate::domain::call::entity::Participant;
use crate::domain::call::value_object::{CallDirection, EndReason};
use crate::domain::shared::events::{DomainEvent, EventMetadata};
use crate::domain::shared::value_objects::{CallId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base struct for all call events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEventBase {
    pub metadata: EventMetadata,
    pub call_id: CallId,
}

/// Call initiated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInitiated {
    pub base: CallEventBase,
    pub caller: Participant,
    pub callee: Participant,
    pub direction: CallDirection,
}

impl DomainEvent for CallInitiated {
    fn event_type(&self) -> &'static str {
        "call.initiated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Call ringing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRinging {
    pub base: CallEventBase,
    pub session_id: SessionId,
}

impl DomainEvent for CallRinging {
    fn event_type(&self) -> &'static str {
        "call.ringing"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Call answered event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallAnswered {
    pub base: CallEventBase,
    pub answered_at: DateTime<Utc>,
}

impl DomainEvent for CallAnswered {
    fn event_type(&self) -> &'static str {
        "call.answered"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Call held event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHeld {
    pub base: CallEventBase,
}

impl DomainEvent for CallHeld {
    fn event_type(&self) -> &'static str {
        "call.held"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Call resumed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResumed {
    pub base: CallEventBase,
}

impl DomainEvent for CallResumed {
    fn event_type(&self) -> &'static str {
        "call.resumed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Call ended event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEnded {
    pub base: CallEventBase,
    pub reason: EndReason,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: Option<i64>,
}

impl DomainEvent for CallEnded {
    fn event_type(&self) -> &'static str {
        "call.ended"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.base.metadata.occurred_at
    }
}

/// Union of all call events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallEvent {
    Initiated(CallInitiated),
    Ringing(CallRinging),
    Answered(CallAnswered),
    Held(CallHeld),
    Resumed(CallResumed),
    Ended(CallEnded),
}

impl CallEvent {
    pub fn call_id(&self) -> &CallId {
        match self {
            CallEvent::Initiated(e) => &e.base.call_id,
            CallEvent::Ringing(e) => &e.base.call_id,
            CallEvent::Answered(e) => &e.base.call_id,
            CallEvent::Held(e) => &e.base.call_id,
            CallEvent::Resumed(e) => &e.base.call_id,
            CallEvent::Ended(e) => &e.base.call_id,
        }
    }
}

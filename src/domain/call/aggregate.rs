//! Call aggregate root

use crate::domain::call::entity::Participant;
use crate::domain::call::event::{
    CallAnswered, CallEnded, CallEvent, CallEventBase, CallHeld, CallInitiated, CallResumed,
    CallRinging,
};
use crate::domain::call::value_object::{CallDirection, CallState, EndReason};
use crate::domain::shared::error::DomainError;
use crate::domain::shared::events::EventMetadata;
use crate::domain::shared::result::Result;
use crate::domain::shared::value_objects::{CallId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Call aggregate root
///
/// This is the main aggregate that manages the lifecycle of a call.
/// It enforces business rules and ensures state consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    /// Aggregate root ID
    id: CallId,
    /// Associated session ID (once session is established)
    session_id: Option<SessionId>,
    /// Current state
    state: CallState,
    /// Call direction
    direction: CallDirection,
    /// Calling party
    caller: Participant,
    /// Called party
    callee: Participant,
    /// When the call was initiated
    started_at: DateTime<Utc>,
    /// When the call was answered (if applicable)
    answered_at: Option<DateTime<Utc>>,
    /// When the call ended (if applicable)
    ended_at: Option<DateTime<Utc>>,
    /// Pending domain events
    #[serde(skip)]
    events: Vec<CallEvent>,
}

impl Call {
    /// Create a new call
    pub fn new(
        id: CallId,
        caller: Participant,
        callee: Participant,
        direction: CallDirection,
    ) -> Self {
        let started_at = Utc::now();
        let mut call = Self {
            id,
            session_id: None,
            state: CallState::Initiating,
            direction,
            caller: caller.clone(),
            callee: callee.clone(),
            started_at,
            answered_at: None,
            ended_at: None,
            events: Vec::new(),
        };

        // Record domain event
        call.record_event(CallEvent::Initiated(CallInitiated {
            base: CallEventBase {
                metadata: EventMetadata::new("call.initiated".to_string()),
                call_id: id,
            },
            caller,
            callee,
            direction,
        }));

        call
    }

    /// Ring the call
    pub fn ring(&mut self, session_id: SessionId) -> Result<()> {
        self.transition_to(CallState::Ringing)?;
        self.session_id = Some(session_id);

        self.record_event(CallEvent::Ringing(CallRinging {
            base: CallEventBase {
                metadata: EventMetadata::new("call.ringing".to_string()),
                call_id: self.id,
            },
            session_id,
        }));

        Ok(())
    }

    /// Answer the call
    pub fn answer(&mut self) -> Result<()> {
        self.transition_to(CallState::Answered)?;
        let answered_at = Utc::now();
        self.answered_at = Some(answered_at);

        self.record_event(CallEvent::Answered(CallAnswered {
            base: CallEventBase {
                metadata: EventMetadata::new("call.answered".to_string()),
                call_id: self.id,
            },
            answered_at,
        }));

        Ok(())
    }

    /// Put the call on hold
    pub fn hold(&mut self) -> Result<()> {
        self.transition_to(CallState::OnHold)?;

        self.record_event(CallEvent::Held(CallHeld {
            base: CallEventBase {
                metadata: EventMetadata::new("call.held".to_string()),
                call_id: self.id,
            },
        }));

        Ok(())
    }

    /// Resume the call from hold
    pub fn resume(&mut self) -> Result<()> {
        if !matches!(self.state, CallState::OnHold) {
            return Err(DomainError::InvalidStateTransition(
                "Can only resume from OnHold state".to_string(),
            ));
        }

        self.transition_to(CallState::Answered)?;

        self.record_event(CallEvent::Resumed(CallResumed {
            base: CallEventBase {
                metadata: EventMetadata::new("call.resumed".to_string()),
                call_id: self.id,
            },
        }));

        Ok(())
    }

    /// End the call
    pub fn end(&mut self, reason: EndReason) -> Result<()> {
        self.transition_to(CallState::Ended(reason.clone()))?;
        let ended_at = Utc::now();
        self.ended_at = Some(ended_at);

        let duration_seconds = self.answered_at.map(|answered| {
            (ended_at - answered).num_seconds()
        });

        self.record_event(CallEvent::Ended(CallEnded {
            base: CallEventBase {
                metadata: EventMetadata::new("call.ended".to_string()),
                call_id: self.id,
            },
            reason,
            ended_at,
            duration_seconds,
        }));

        Ok(())
    }

    /// Transition to a new state
    fn transition_to(&mut self, new_state: CallState) -> Result<()> {
        if !self.state.can_transition_to(&new_state) {
            return Err(DomainError::InvalidStateTransition(format!(
                "Cannot transition from {:?} to {:?}",
                self.state, new_state
            )));
        }

        self.state = new_state;
        Ok(())
    }

    /// Record a domain event
    fn record_event(&mut self, event: CallEvent) {
        self.events.push(event);
    }

    /// Take all pending events
    pub fn take_events(&mut self) -> Vec<CallEvent> {
        std::mem::take(&mut self.events)
    }

    // Getters
    pub fn id(&self) -> &CallId {
        &self.id
    }

    pub fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    pub fn state(&self) -> &CallState {
        &self.state
    }

    pub fn direction(&self) -> &CallDirection {
        &self.direction
    }

    pub fn caller(&self) -> &Participant {
        &self.caller
    }

    pub fn callee(&self) -> &Participant {
        &self.callee
    }

    pub fn started_at(&self) -> &DateTime<Utc> {
        &self.started_at
    }

    pub fn answered_at(&self) -> Option<&DateTime<Utc>> {
        self.answered_at.as_ref()
    }

    pub fn ended_at(&self) -> Option<&DateTime<Utc>> {
        self.ended_at.as_ref()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.answered_at.and_then(|answered| {
            self.ended_at.map(|ended| ended - answered)
        })
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::value_objects::{EndpointId, SipUri};

    fn create_test_call() -> Call {
        let caller = Participant::new(
            EndpointId::new(),
            SipUri::parse("sip:alice@example.com").unwrap(),
            Some("Alice".to_string()),
        );

        let callee = Participant::new(
            EndpointId::new(),
            SipUri::parse("sip:bob@example.com").unwrap(),
            Some("Bob".to_string()),
        );

        Call::new(CallId::new(), caller, callee, CallDirection::Internal)
    }

    #[test]
    fn test_call_lifecycle() {
        let mut call = create_test_call();

        assert!(matches!(call.state(), CallState::Initiating));
        assert_eq!(call.events.len(), 1); // Initiated event

        // Ring
        let session_id = SessionId::new();
        call.ring(session_id).unwrap();
        assert!(matches!(call.state(), CallState::Ringing));
        assert_eq!(call.session_id(), Some(&session_id));

        // Answer
        call.answer().unwrap();
        assert!(matches!(call.state(), CallState::Answered));
        assert!(call.answered_at().is_some());

        // Hold
        call.hold().unwrap();
        assert!(matches!(call.state(), CallState::OnHold));

        // Resume
        call.resume().unwrap();
        assert!(matches!(call.state(), CallState::Answered));

        // End
        call.end(EndReason::NormalClearing).unwrap();
        assert!(matches!(call.state(), CallState::Ended(_)));
        assert!(call.ended_at().is_some());
        assert!(call.duration().is_some());

        // Verify all events were recorded
        let events = call.take_events();
        assert_eq!(events.len(), 6); // Initiated, Ringing, Answered, Held, Resumed, Ended
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut call = create_test_call();

        // Try to answer without ringing
        let result = call.answer();
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_transition_from_ended() {
        let mut call = create_test_call();
        call.ring(SessionId::new()).unwrap();
        call.answer().unwrap();
        call.end(EndReason::NormalClearing).unwrap();

        // Try to hold after ended
        let result = call.hold();
        assert!(result.is_err());
    }
}

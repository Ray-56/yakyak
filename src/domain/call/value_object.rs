//! Call value objects

use serde::{Deserialize, Serialize};

/// Call direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallDirection {
    /// Inbound call from external
    Inbound,
    /// Outbound call to external
    Outbound,
    /// Internal call between endpoints
    Internal,
}

/// Call state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallState {
    /// Call is being initiated
    Initiating,
    /// Callee is being alerted (ringing)
    Ringing,
    /// Call has been answered and media is flowing
    Answered,
    /// Call is on hold
    OnHold,
    /// Call is being transferred
    Transferring,
    /// Call has ended
    Ended(EndReason),
}

impl CallState {
    /// Check if state transition is valid
    pub fn can_transition_to(&self, new_state: &CallState) -> bool {
        use CallState::*;

        match (self, new_state) {
            // From Initiating
            (Initiating, Ringing) => true,
            (Initiating, Ended(_)) => true,

            // From Ringing
            (Ringing, Answered) => true,
            (Ringing, Ended(_)) => true,

            // From Answered
            (Answered, OnHold) => true,
            (Answered, Transferring) => true,
            (Answered, Ended(_)) => true,

            // From OnHold
            (OnHold, Answered) => true,
            (OnHold, Ended(_)) => true,

            // From Transferring
            (Transferring, Answered) => true,
            (Transferring, Ended(_)) => true,

            // Can't transition from Ended
            (Ended(_), _) => false,

            // All other transitions are invalid
            _ => false,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, CallState::Ended(_))
    }
}

/// Reason for call ending
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    /// Normal call completion
    NormalClearing,
    /// Caller hung up
    CallerHangup,
    /// Callee hung up
    CalleeHangup,
    /// Call was rejected
    Rejected,
    /// No answer
    NoAnswer,
    /// Busy
    Busy,
    /// Call failed
    Failed(String),
    /// Call was canceled
    Canceled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_state_transitions() {
        let initiating = CallState::Initiating;
        assert!(initiating.can_transition_to(&CallState::Ringing));
        assert!(initiating.can_transition_to(&CallState::Ended(EndReason::Canceled)));
        assert!(!initiating.can_transition_to(&CallState::Answered));

        let ringing = CallState::Ringing;
        assert!(ringing.can_transition_to(&CallState::Answered));
        assert!(ringing.can_transition_to(&CallState::Ended(EndReason::NoAnswer)));

        let answered = CallState::Answered;
        assert!(answered.can_transition_to(&CallState::OnHold));
        assert!(answered.can_transition_to(&CallState::Ended(EndReason::NormalClearing)));
    }

    #[test]
    fn test_invalid_state_transitions() {
        let ended = CallState::Ended(EndReason::NormalClearing);
        assert!(!ended.can_transition_to(&CallState::Answered));
        assert!(!ended.can_transition_to(&CallState::Ringing));
    }

    #[test]
    fn test_is_active() {
        assert!(CallState::Initiating.is_active());
        assert!(CallState::Answered.is_active());
        assert!(!CallState::Ended(EndReason::NormalClearing).is_active());
    }
}

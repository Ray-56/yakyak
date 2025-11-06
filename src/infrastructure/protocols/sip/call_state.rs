//! Call State Machine
//!
//! Implements a complete call state machine for SIP calls

use std::time::Instant;

/// Call State
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    /// Initial state - INVITE received/sent
    Trying,
    /// 100 Trying sent/received
    Proceeding,
    /// 180 Ringing sent/received
    Ringing,
    /// 183 Session Progress sent/received
    EarlyMedia,
    /// 200 OK sent/received, call established
    Established,
    /// BYE sent/received
    Terminating,
    /// Call ended
    Terminated,
    /// Call failed (4xx, 5xx, 6xx)
    Failed,
}

impl CallState {
    /// Check if state is active (not terminated or failed)
    pub fn is_active(&self) -> bool {
        !matches!(self, CallState::Terminated | CallState::Failed)
    }

    /// Check if state is provisional
    pub fn is_provisional(&self) -> bool {
        matches!(
            self,
            CallState::Trying | CallState::Proceeding | CallState::Ringing | CallState::EarlyMedia
        )
    }

    /// Check if call is established
    pub fn is_established(&self) -> bool {
        matches!(self, CallState::Established)
    }

    /// Get state name
    pub fn name(&self) -> &'static str {
        match self {
            CallState::Trying => "Trying",
            CallState::Proceeding => "Proceeding",
            CallState::Ringing => "Ringing",
            CallState::EarlyMedia => "EarlyMedia",
            CallState::Established => "Established",
            CallState::Terminating => "Terminating",
            CallState::Terminated => "Terminated",
            CallState::Failed => "Failed",
        }
    }
}

/// Call Leg - represents one side of a call
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallLeg {
    /// Caller (UAC - User Agent Client)
    Caller,
    /// Callee (UAS - User Agent Server)
    Callee,
}

/// Call Direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallDirection {
    /// Inbound call (we received INVITE)
    Inbound,
    /// Outbound call (we sent INVITE)
    Outbound,
}

/// Call Statistics
#[derive(Debug, Clone)]
pub struct CallStats {
    /// When the call was created
    pub created_at: Instant,
    /// When the call was answered (if applicable)
    pub answered_at: Option<Instant>,
    /// When the call ended (if applicable)
    pub ended_at: Option<Instant>,
    /// Number of provisional responses received/sent
    pub provisional_count: u32,
}

impl CallStats {
    pub fn new() -> Self {
        Self {
            created_at: Instant::now(),
            answered_at: None,
            ended_at: None,
            provisional_count: 0,
        }
    }

    /// Get call setup duration (time from created to answered)
    pub fn setup_duration(&self) -> Option<std::time::Duration> {
        self.answered_at.map(|t| t.duration_since(self.created_at))
    }

    /// Get call duration (time from answered to ended)
    pub fn call_duration(&self) -> Option<std::time::Duration> {
        match (self.answered_at, self.ended_at) {
            (Some(answered), Some(ended)) => Some(ended.duration_since(answered)),
            _ => None,
        }
    }

    /// Get total duration (time from created to ended)
    pub fn total_duration(&self) -> Option<std::time::Duration> {
        self.ended_at.map(|t| t.duration_since(self.created_at))
    }
}

impl Default for CallStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Call State Machine Event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallEvent {
    /// INVITE sent/received
    Invite,
    /// 100 Trying sent/received
    Trying,
    /// 180 Ringing sent/received
    Ringing,
    /// 183 Session Progress sent/received
    SessionProgress,
    /// 200 OK sent/received
    Answer,
    /// BYE sent/received
    Bye,
    /// 4xx/5xx/6xx response
    Reject,
    /// Timeout
    Timeout,
}

/// State Machine
pub struct CallStateMachine {
    state: CallState,
    stats: CallStats,
}

impl CallStateMachine {
    pub fn new() -> Self {
        Self {
            state: CallState::Trying,
            stats: CallStats::new(),
        }
    }

    /// Get current state
    pub fn state(&self) -> &CallState {
        &self.state
    }

    /// Get statistics
    pub fn stats(&self) -> &CallStats {
        &self.stats
    }

    /// Process an event and transition state
    pub fn process_event(&mut self, event: CallEvent) -> Result<(), String> {
        let new_state = match (&self.state, &event) {
            // From Trying
            (CallState::Trying, CallEvent::Trying) => CallState::Proceeding,
            (CallState::Trying, CallEvent::Ringing) => CallState::Ringing,
            (CallState::Trying, CallEvent::SessionProgress) => CallState::EarlyMedia,
            (CallState::Trying, CallEvent::Answer) => CallState::Established,
            (CallState::Trying, CallEvent::Reject) => CallState::Failed,
            (CallState::Trying, CallEvent::Timeout) => CallState::Failed,

            // From Proceeding
            (CallState::Proceeding, CallEvent::Ringing) => CallState::Ringing,
            (CallState::Proceeding, CallEvent::SessionProgress) => CallState::EarlyMedia,
            (CallState::Proceeding, CallEvent::Answer) => CallState::Established,
            (CallState::Proceeding, CallEvent::Reject) => CallState::Failed,
            (CallState::Proceeding, CallEvent::Timeout) => CallState::Failed,

            // From Ringing
            (CallState::Ringing, CallEvent::Answer) => CallState::Established,
            (CallState::Ringing, CallEvent::Reject) => CallState::Failed,
            (CallState::Ringing, CallEvent::Timeout) => CallState::Failed,

            // From EarlyMedia
            (CallState::EarlyMedia, CallEvent::Answer) => CallState::Established,
            (CallState::EarlyMedia, CallEvent::Reject) => CallState::Failed,
            (CallState::EarlyMedia, CallEvent::Timeout) => CallState::Failed,

            // From Established
            (CallState::Established, CallEvent::Bye) => CallState::Terminating,

            // From Terminating
            (CallState::Terminating, _) => CallState::Terminated,

            // Invalid transitions
            _ => {
                return Err(format!(
                    "Invalid state transition: {} + {:?}",
                    self.state.name(),
                    event
                ))
            }
        };

        // Update statistics
        match event {
            CallEvent::Trying | CallEvent::Ringing | CallEvent::SessionProgress => {
                self.stats.provisional_count += 1;
            }
            CallEvent::Answer => {
                self.stats.answered_at = Some(Instant::now());
            }
            CallEvent::Bye | CallEvent::Reject | CallEvent::Timeout => {
                self.stats.ended_at = Some(Instant::now());
            }
            _ => {}
        }

        self.state = new_state;
        Ok(())
    }

    /// Check if call can be answered
    pub fn can_answer(&self) -> bool {
        matches!(
            self.state,
            CallState::Trying
                | CallState::Proceeding
                | CallState::Ringing
                | CallState::EarlyMedia
        )
    }

    /// Check if call can be rejected
    pub fn can_reject(&self) -> bool {
        self.can_answer()
    }

    /// Check if call can be terminated
    pub fn can_terminate(&self) -> bool {
        matches!(self.state, CallState::Established)
    }
}

impl Default for CallStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_state_transitions() {
        let mut sm = CallStateMachine::new();
        assert_eq!(sm.state(), &CallState::Trying);

        // Trying -> Proceeding
        sm.process_event(CallEvent::Trying).unwrap();
        assert_eq!(sm.state(), &CallState::Proceeding);

        // Proceeding -> Ringing
        sm.process_event(CallEvent::Ringing).unwrap();
        assert_eq!(sm.state(), &CallState::Ringing);

        // Ringing -> Established
        sm.process_event(CallEvent::Answer).unwrap();
        assert_eq!(sm.state(), &CallState::Established);
        assert!(sm.stats().answered_at.is_some());

        // Established -> Terminating
        sm.process_event(CallEvent::Bye).unwrap();
        assert_eq!(sm.state(), &CallState::Terminating);
    }

    #[test]
    fn test_fast_answer() {
        let mut sm = CallStateMachine::new();

        // Trying -> Established (fast answer without provisional responses)
        sm.process_event(CallEvent::Answer).unwrap();
        assert_eq!(sm.state(), &CallState::Established);
    }

    #[test]
    fn test_call_rejection() {
        let mut sm = CallStateMachine::new();
        sm.process_event(CallEvent::Trying).unwrap();
        sm.process_event(CallEvent::Ringing).unwrap();

        // Reject during ringing
        sm.process_event(CallEvent::Reject).unwrap();
        assert_eq!(sm.state(), &CallState::Failed);
        assert!(sm.stats().ended_at.is_some());
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = CallStateMachine::new();
        sm.process_event(CallEvent::Answer).unwrap();

        // Can't answer an already established call
        let result = sm.process_event(CallEvent::Answer);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_stats() {
        let mut sm = CallStateMachine::new();

        sm.process_event(CallEvent::Trying).unwrap();
        assert_eq!(sm.stats().provisional_count, 1);

        sm.process_event(CallEvent::Ringing).unwrap();
        assert_eq!(sm.stats().provisional_count, 2);

        sm.process_event(CallEvent::Answer).unwrap();
        assert!(sm.stats().setup_duration().is_some());
    }

    #[test]
    fn test_state_helpers() {
        assert!(CallState::Trying.is_active());
        assert!(CallState::Trying.is_provisional());
        assert!(!CallState::Trying.is_established());

        assert!(CallState::Established.is_active());
        assert!(!CallState::Established.is_provisional());
        assert!(CallState::Established.is_established());

        assert!(!CallState::Terminated.is_active());
        assert!(!CallState::Failed.is_active());
    }
}

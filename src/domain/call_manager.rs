//! Active call management and monitoring system
//!
//! Provides real-time call tracking, control, and monitoring capabilities
//! for operational dashboards and call center management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Call state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallState {
    /// Call is being initiated
    Initiating,
    /// Ringing (180 Ringing sent)
    Ringing,
    /// Call is active/connected
    Active,
    /// Call is on hold
    OnHold,
    /// Call is being transferred
    Transferring,
    /// Call is terminating
    Terminating,
    /// Call has ended
    Terminated,
}

impl CallState {
    pub fn as_str(&self) -> &str {
        match self {
            CallState::Initiating => "initiating",
            CallState::Ringing => "ringing",
            CallState::Active => "active",
            CallState::OnHold => "on_hold",
            CallState::Transferring => "transferring",
            CallState::Terminating => "terminating",
            CallState::Terminated => "terminated",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, CallState::Active | CallState::OnHold | CallState::Transferring)
    }
}

/// Call direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallDirection {
    Inbound,
    Outbound,
    Internal,
}

/// Active call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCall {
    pub id: Uuid,
    pub call_id: String,
    pub state: CallState,
    pub direction: CallDirection,
    pub caller: String,
    pub caller_name: Option<String>,
    pub callee: String,
    pub callee_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
    pub codec: String,
    pub caller_ip: Option<String>,
    pub callee_ip: Option<String>,
    pub quality_mos: Option<f64>,
    pub is_recording: bool,
    pub is_on_hold: bool,
    pub queue_id: Option<Uuid>,
    pub conference_id: Option<Uuid>,
    pub tags: Vec<String>,
}

impl ActiveCall {
    pub fn new(
        call_id: String,
        direction: CallDirection,
        caller: String,
        callee: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            call_id,
            state: CallState::Initiating,
            direction,
            caller,
            caller_name: None,
            callee,
            callee_name: None,
            started_at: Utc::now(),
            answered_at: None,
            duration_seconds: 0,
            codec: "PCMU".to_string(),
            caller_ip: None,
            callee_ip: None,
            quality_mos: None,
            is_recording: false,
            is_on_hold: false,
            queue_id: None,
            conference_id: None,
            tags: Vec::new(),
        }
    }

    /// Mark call as answered
    pub fn answer(&mut self) {
        self.state = CallState::Active;
        self.answered_at = Some(Utc::now());
    }

    /// Update call duration
    pub fn update_duration(&mut self) {
        if let Some(answered_at) = self.answered_at {
            self.duration_seconds = (Utc::now() - answered_at).num_seconds() as u64;
        } else {
            self.duration_seconds = (Utc::now() - self.started_at).num_seconds() as u64;
        }
    }

    /// Get total call duration including setup time
    pub fn get_total_duration(&self) -> Duration {
        let seconds = (Utc::now() - self.started_at).num_seconds();
        Duration::from_secs(seconds.max(0) as u64)
    }

    /// Get talk time (after answer)
    pub fn get_talk_time(&self) -> Option<Duration> {
        self.answered_at.map(|answered| {
            let seconds = (Utc::now() - answered).num_seconds();
            Duration::from_secs(seconds.max(0) as u64)
        })
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

/// Call statistics aggregation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CallStatistics {
    pub total_active_calls: usize,
    pub inbound_calls: usize,
    pub outbound_calls: usize,
    pub internal_calls: usize,
    pub calls_ringing: usize,
    pub calls_on_hold: usize,
    pub calls_recording: usize,
    pub calls_in_queue: usize,
    pub calls_in_conference: usize,
    pub average_duration_seconds: f64,
    pub longest_call_seconds: u64,
    pub average_mos: f64,
}

/// Call control action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallControlAction {
    /// Hangup the call
    Hangup { reason: String },
    /// Put call on hold
    Hold,
    /// Resume call from hold
    Resume,
    /// Transfer call to another party
    Transfer { target: String },
    /// Start recording
    StartRecording,
    /// Stop recording
    StopRecording,
    /// Mute a party
    Mute { party: String },
    /// Unmute a party
    Unmute { party: String },
}

/// Call control result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallControlResult {
    pub success: bool,
    pub message: String,
    pub call_id: String,
}

/// Active call manager
pub struct ActiveCallManager {
    calls: Arc<Mutex<HashMap<String, ActiveCall>>>,
    call_history: Arc<Mutex<Vec<ActiveCall>>>,
    max_history: usize,
}

impl ActiveCallManager {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// Register a new call
    pub fn register_call(
        &self,
        call_id: String,
        direction: CallDirection,
        caller: String,
        callee: String,
    ) -> Uuid {
        let call = ActiveCall::new(call_id.clone(), direction, caller, callee);
        let id = call.id;

        let mut calls = self.calls.lock().unwrap();
        calls.insert(call_id, call);

        id
    }

    /// Update call state
    pub fn update_state(&self, call_id: &str, state: CallState) -> Result<(), String> {
        let mut calls = self.calls.lock().unwrap();
        if let Some(call) = calls.get_mut(call_id) {
            call.state = state;

            // If answered, record the time
            if state == CallState::Active && call.answered_at.is_none() {
                call.answer();
            }

            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Mark call as terminated and move to history
    pub fn terminate_call(&self, call_id: &str) -> Result<ActiveCall, String> {
        let mut calls = self.calls.lock().unwrap();
        if let Some(mut call) = calls.remove(call_id) {
            call.state = CallState::Terminated;
            call.update_duration();

            // Add to history
            let mut history = self.call_history.lock().unwrap();
            history.push(call.clone());

            // Maintain max history size
            if history.len() > self.max_history {
                history.remove(0);
            }

            Ok(call)
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Get active call by call ID
    pub fn get_call(&self, call_id: &str) -> Option<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls.get(call_id).cloned()
    }

    /// Get all active calls
    pub fn get_all_calls(&self) -> Vec<ActiveCall> {
        let mut calls = self.calls.lock().unwrap();

        // Update durations before returning
        for call in calls.values_mut() {
            call.update_duration();
        }

        calls.values().cloned().collect()
    }

    /// Get calls by state
    pub fn get_calls_by_state(&self, state: CallState) -> Vec<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls
            .values()
            .filter(|c| c.state == state)
            .cloned()
            .collect()
    }

    /// Get calls by direction
    pub fn get_calls_by_direction(&self, direction: CallDirection) -> Vec<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls
            .values()
            .filter(|c| c.direction == direction)
            .cloned()
            .collect()
    }

    /// Get calls by user (as caller or callee)
    pub fn get_calls_by_user(&self, username: &str) -> Vec<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls
            .values()
            .filter(|c| c.caller == username || c.callee == username)
            .cloned()
            .collect()
    }

    /// Get calls in a specific queue
    pub fn get_calls_in_queue(&self, queue_id: Uuid) -> Vec<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls
            .values()
            .filter(|c| c.queue_id == Some(queue_id))
            .cloned()
            .collect()
    }

    /// Get calls in a specific conference
    pub fn get_calls_in_conference(&self, conference_id: Uuid) -> Vec<ActiveCall> {
        let calls = self.calls.lock().unwrap();
        calls
            .values()
            .filter(|c| c.conference_id == Some(conference_id))
            .cloned()
            .collect()
    }

    /// Update call quality metrics
    pub fn update_quality(&self, call_id: &str, mos: f64) -> Result<(), String> {
        let mut calls = self.calls.lock().unwrap();
        if let Some(call) = calls.get_mut(call_id) {
            call.quality_mos = Some(mos);
            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Set recording status
    pub fn set_recording(&self, call_id: &str, recording: bool) -> Result<(), String> {
        let mut calls = self.calls.lock().unwrap();
        if let Some(call) = calls.get_mut(call_id) {
            call.is_recording = recording;
            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Set hold status
    pub fn set_hold(&self, call_id: &str, on_hold: bool) -> Result<(), String> {
        let mut calls = self.calls.lock().unwrap();
        if let Some(call) = calls.get_mut(call_id) {
            call.is_on_hold = on_hold;
            if on_hold {
                call.state = CallState::OnHold;
            } else if call.state == CallState::OnHold {
                call.state = CallState::Active;
            }
            Ok(())
        } else {
            Err(format!("Call {} not found", call_id))
        }
    }

    /// Get call statistics
    pub fn get_statistics(&self) -> CallStatistics {
        let calls = self.calls.lock().unwrap();

        let mut stats = CallStatistics {
            total_active_calls: calls.len(),
            ..Default::default()
        };

        let mut total_duration = 0u64;
        let mut total_mos = 0.0;
        let mut mos_count = 0;

        for call in calls.values() {
            // Count by direction
            match call.direction {
                CallDirection::Inbound => stats.inbound_calls += 1,
                CallDirection::Outbound => stats.outbound_calls += 1,
                CallDirection::Internal => stats.internal_calls += 1,
            }

            // Count by state
            if call.state == CallState::Ringing {
                stats.calls_ringing += 1;
            }
            if call.is_on_hold {
                stats.calls_on_hold += 1;
            }
            if call.is_recording {
                stats.calls_recording += 1;
            }
            if call.queue_id.is_some() {
                stats.calls_in_queue += 1;
            }
            if call.conference_id.is_some() {
                stats.calls_in_conference += 1;
            }

            // Aggregate duration
            total_duration += call.duration_seconds;
            if call.duration_seconds > stats.longest_call_seconds {
                stats.longest_call_seconds = call.duration_seconds;
            }

            // Aggregate quality
            if let Some(mos) = call.quality_mos {
                total_mos += mos;
                mos_count += 1;
            }
        }

        // Calculate averages
        if !calls.is_empty() {
            stats.average_duration_seconds = total_duration as f64 / calls.len() as f64;
        }
        if mos_count > 0 {
            stats.average_mos = total_mos / mos_count as f64;
        }

        stats
    }

    /// Get recent call history
    pub fn get_recent_history(&self, count: usize) -> Vec<ActiveCall> {
        let history = self.call_history.lock().unwrap();
        history.iter().rev().take(count).cloned().collect()
    }

    /// Clear all calls (for testing)
    pub fn clear(&self) {
        let mut calls = self.calls.lock().unwrap();
        calls.clear();
    }

    /// Get total active call count
    pub fn count(&self) -> usize {
        let calls = self.calls.lock().unwrap();
        calls.len()
    }

    /// Execute call control action
    pub fn control_call(
        &self,
        call_id: &str,
        action: CallControlAction,
    ) -> CallControlResult {
        match action {
            CallControlAction::Hangup { reason } => {
                match self.terminate_call(call_id) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: format!("Call terminated: {}", reason),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::Hold => {
                match self.set_hold(call_id, true) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: "Call placed on hold".to_string(),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::Resume => {
                match self.set_hold(call_id, false) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: "Call resumed".to_string(),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::StartRecording => {
                match self.set_recording(call_id, true) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: "Recording started".to_string(),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::StopRecording => {
                match self.set_recording(call_id, false) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: "Recording stopped".to_string(),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::Transfer { target } => {
                match self.update_state(call_id, CallState::Transferring) {
                    Ok(_) => CallControlResult {
                        success: true,
                        message: format!("Transfer initiated to {}", target),
                        call_id: call_id.to_string(),
                    },
                    Err(e) => CallControlResult {
                        success: false,
                        message: e,
                        call_id: call_id.to_string(),
                    },
                }
            }
            CallControlAction::Mute { party: _ } | CallControlAction::Unmute { party: _ } => {
                CallControlResult {
                    success: false,
                    message: "Mute/unmute not yet implemented".to_string(),
                    call_id: call_id.to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_state_active() {
        assert!(CallState::Active.is_active());
        assert!(CallState::OnHold.is_active());
        assert!(!CallState::Terminated.is_active());
    }

    #[test]
    fn test_active_call_creation() {
        let call = ActiveCall::new(
            "call-123".to_string(),
            CallDirection::Inbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        assert_eq!(call.call_id, "call-123");
        assert_eq!(call.state, CallState::Initiating);
        assert!(call.answered_at.is_none());
    }

    #[test]
    fn test_active_call_answer() {
        let mut call = ActiveCall::new(
            "call-456".to_string(),
            CallDirection::Outbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        call.answer();
        assert_eq!(call.state, CallState::Active);
        assert!(call.answered_at.is_some());
    }

    #[test]
    fn test_active_call_manager_register() {
        let manager = ActiveCallManager::new();

        let id = manager.register_call(
            "call-789".to_string(),
            CallDirection::Internal,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        assert!(id != Uuid::nil());
        assert_eq!(manager.count(), 1);

        let call = manager.get_call("call-789");
        assert!(call.is_some());
    }

    #[test]
    fn test_active_call_manager_update_state() {
        let manager = ActiveCallManager::new();

        manager.register_call(
            "call-abc".to_string(),
            CallDirection::Inbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        assert!(manager.update_state("call-abc", CallState::Ringing).is_ok());

        let call = manager.get_call("call-abc").unwrap();
        assert_eq!(call.state, CallState::Ringing);
    }

    #[test]
    fn test_active_call_manager_terminate() {
        let manager = ActiveCallManager::new();

        manager.register_call(
            "call-xyz".to_string(),
            CallDirection::Outbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        assert_eq!(manager.count(), 1);

        let result = manager.terminate_call("call-xyz");
        assert!(result.is_ok());
        assert_eq!(manager.count(), 0);

        let history = manager.get_recent_history(10);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_active_call_manager_statistics() {
        let manager = ActiveCallManager::new();

        manager.register_call(
            "call-1".to_string(),
            CallDirection::Inbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        manager.register_call(
            "call-2".to_string(),
            CallDirection::Outbound,
            "bob@example.com".to_string(),
            "charlie@example.com".to_string(),
        );

        let stats = manager.get_statistics();
        assert_eq!(stats.total_active_calls, 2);
        assert_eq!(stats.inbound_calls, 1);
        assert_eq!(stats.outbound_calls, 1);
    }

    #[test]
    fn test_active_call_manager_filter_by_direction() {
        let manager = ActiveCallManager::new();

        manager.register_call(
            "call-in".to_string(),
            CallDirection::Inbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        manager.register_call(
            "call-out".to_string(),
            CallDirection::Outbound,
            "bob@example.com".to_string(),
            "charlie@example.com".to_string(),
        );

        let inbound = manager.get_calls_by_direction(CallDirection::Inbound);
        assert_eq!(inbound.len(), 1);

        let outbound = manager.get_calls_by_direction(CallDirection::Outbound);
        assert_eq!(outbound.len(), 1);
    }

    #[test]
    fn test_call_control_actions() {
        let manager = ActiveCallManager::new();

        manager.register_call(
            "call-ctrl".to_string(),
            CallDirection::Inbound,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        );

        // Test hold
        let result = manager.control_call("call-ctrl", CallControlAction::Hold);
        assert!(result.success);

        let call = manager.get_call("call-ctrl").unwrap();
        assert!(call.is_on_hold);

        // Test resume
        let result = manager.control_call("call-ctrl", CallControlAction::Resume);
        assert!(result.success);

        // Test hangup
        let result = manager.control_call(
            "call-ctrl",
            CallControlAction::Hangup {
                reason: "User request".to_string(),
            },
        );
        assert!(result.success);
        assert_eq!(manager.count(), 0);
    }
}

//! Do Not Disturb (DND) domain model
//!
//! Provides Do Not Disturb functionality for users to block incoming calls
//! with support for schedules, exceptions, and various rejection modes.

use chrono::{DateTime, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// DND rejection mode - how to handle blocked calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DndMode {
    /// Reject calls with busy signal (486 Busy Here)
    RejectBusy,
    /// Send calls to voicemail
    SendToVoicemail,
    /// Send calls to alternate number
    ForwardToAlternate,
    /// Silent reject (no response)
    SilentReject,
    /// Play custom announcement then disconnect
    PlayAnnouncementDisconnect,
}

impl DndMode {
    pub fn description(&self) -> &str {
        match self {
            DndMode::RejectBusy => "Reject with busy signal",
            DndMode::SendToVoicemail => "Send to voicemail",
            DndMode::ForwardToAlternate => "Forward to alternate number",
            DndMode::SilentReject => "Silent reject",
            DndMode::PlayAnnouncementDisconnect => "Play announcement then disconnect",
        }
    }

    /// Get SIP response code for this mode
    pub fn sip_response_code(&self) -> u16 {
        match self {
            DndMode::RejectBusy => 486, // Busy Here
            DndMode::SendToVoicemail => 302, // Moved Temporarily (to voicemail)
            DndMode::ForwardToAlternate => 302, // Moved Temporarily
            DndMode::SilentReject => 603, // Decline
            DndMode::PlayAnnouncementDisconnect => 480, // Temporarily Unavailable
        }
    }
}

/// Time-based DND schedule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DndSchedule {
    /// Schedule identifier
    pub id: Uuid,
    /// Schedule name
    pub name: String,
    /// Start time (HH:MM format)
    pub start_time: NaiveTime,
    /// End time (HH:MM format)
    pub end_time: NaiveTime,
    /// Days of week (empty = all days)
    pub days_of_week: Vec<Weekday>,
    /// Whether the schedule is enabled
    pub enabled: bool,
    /// DND mode to use during this schedule
    pub mode: DndMode,
}

impl DndSchedule {
    pub fn new(name: String, start_time: NaiveTime, end_time: NaiveTime, mode: DndMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            start_time,
            end_time,
            days_of_week: vec![],
            enabled: true,
            mode,
        }
    }

    pub fn with_days(mut self, days: Vec<Weekday>) -> Self {
        self.days_of_week = days;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if the current time falls within this schedule
    pub fn is_active(&self, time: NaiveTime, weekday: Weekday) -> bool {
        if !self.enabled {
            return false;
        }

        // Check day of week if specified
        if !self.days_of_week.is_empty() && !self.days_of_week.contains(&weekday) {
            return false;
        }

        // Handle time range that crosses midnight
        if self.start_time <= self.end_time {
            time >= self.start_time && time <= self.end_time
        } else {
            time >= self.start_time || time <= self.end_time
        }
    }

    /// Business hours preset (Monday-Friday, 9am-5pm)
    pub fn business_hours(mode: DndMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Business Hours".to_string(),
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            days_of_week: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            enabled: true,
            mode,
        }
    }

    /// Night hours preset (10pm-7am daily)
    pub fn night_hours(mode: DndMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Night Hours".to_string(),
            start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            days_of_week: vec![],
            enabled: true,
            mode,
        }
    }
}

/// Exception rule for allowing specific callers through DND
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DndException {
    /// Exception identifier
    pub id: Uuid,
    /// Exception type
    pub exception_type: ExceptionType,
    /// Caller identifiers (numbers, URIs, or contact groups)
    pub callers: Vec<String>,
    /// Whether the exception is enabled
    pub enabled: bool,
    /// Optional description
    pub description: Option<String>,
}

impl DndException {
    pub fn new(exception_type: ExceptionType, callers: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            exception_type,
            callers,
            enabled: true,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if a caller matches this exception
    pub fn matches_caller(&self, caller_id: &str) -> bool {
        if !self.enabled {
            return false;
        }

        match self.exception_type {
            ExceptionType::Exact => self.callers.iter().any(|c| c == caller_id),
            ExceptionType::Prefix => self.callers.iter().any(|c| caller_id.starts_with(c)),
            ExceptionType::Contains => self.callers.iter().any(|c| caller_id.contains(c)),
            ExceptionType::Wildcard => {
                // Simple wildcard matching (* = any characters)
                self.callers.iter().any(|pattern| {
                    wildcard_match(pattern, caller_id)
                })
            }
        }
    }
}

/// Simple wildcard matching helper
fn wildcard_match(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcards, exact match
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // First part must match at start
            if !text.starts_with(part) {
                return false;
            }
            pos += part.len();
        } else if i == parts.len() - 1 {
            // Last part must match at end
            return text.ends_with(part);
        } else {
            // Middle parts can match anywhere
            if let Some(found_pos) = text[pos..].find(part) {
                pos += found_pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Exception matching type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExceptionType {
    /// Exact match of caller ID
    Exact,
    /// Prefix match (e.g., "555" matches "5551234")
    Prefix,
    /// Contains match (e.g., "emergency" in caller ID)
    Contains,
    /// Wildcard match (e.g., "*911" or "555*")
    Wildcard,
}

/// DND status for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DndStatus {
    /// User identifier
    pub user_id: String,
    /// Whether DND is currently active
    pub enabled: bool,
    /// Current DND mode
    pub mode: DndMode,
    /// Alternate forward destination (if ForwardToAlternate mode)
    pub alternate_destination: Option<String>,
    /// Custom announcement file (if PlayAnnouncementDisconnect mode)
    pub announcement_file: Option<String>,
    /// Active schedules
    pub schedules: Vec<DndSchedule>,
    /// Exception rules
    pub exceptions: Vec<DndException>,
    /// When DND was last enabled
    pub enabled_at: Option<DateTime<Utc>>,
    /// When DND was last disabled
    pub disabled_at: Option<DateTime<Utc>>,
    /// Manual override (ignores schedules)
    pub manual_override: bool,
}

impl DndStatus {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            enabled: false,
            mode: DndMode::RejectBusy,
            alternate_destination: None,
            announcement_file: None,
            schedules: vec![],
            exceptions: vec![],
            enabled_at: None,
            disabled_at: None,
            manual_override: false,
        }
    }

    /// Check if DND should block a call from the given caller
    pub fn should_block_call(&self, caller_id: &str, current_time: DateTime<Utc>) -> bool {
        // Check if DND is enabled (manual or scheduled)
        let dnd_active = if self.manual_override {
            self.enabled
        } else {
            self.is_scheduled_active(current_time)
        };

        if !dnd_active {
            return false;
        }

        // Check exceptions - if caller matches any exception, don't block
        for exception in &self.exceptions {
            if exception.matches_caller(caller_id) {
                return false;
            }
        }

        true
    }

    /// Check if any schedule is currently active
    pub fn is_scheduled_active(&self, current_time: DateTime<Utc>) -> bool {
        if self.enabled && self.manual_override {
            return true;
        }

        let time = current_time.time();
        let weekday = current_time.weekday();

        self.schedules
            .iter()
            .any(|schedule| schedule.is_active(time, weekday))
    }

    /// Get the current effective DND mode
    pub fn get_effective_mode(&self, current_time: DateTime<Utc>) -> Option<DndMode> {
        if self.manual_override && self.enabled {
            return Some(self.mode);
        }

        let time = current_time.time();
        let weekday = current_time.weekday();

        // Find first active schedule
        self.schedules
            .iter()
            .find(|schedule| schedule.is_active(time, weekday))
            .map(|schedule| schedule.mode)
    }
}

/// DND statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DndStatistics {
    /// Total users in system
    pub total_users: usize,
    /// Users with DND enabled
    pub users_with_dnd_enabled: usize,
    /// Total blocked calls
    pub total_blocked_calls: u64,
    /// Blocked calls by mode
    pub blocked_by_mode: HashMap<String, u64>,
    /// Total exception matches
    pub total_exception_matches: u64,
    /// Calls allowed by exceptions
    pub calls_allowed_by_exception: u64,
}

impl DndStatistics {
    pub fn new() -> Self {
        Self {
            total_users: 0,
            users_with_dnd_enabled: 0,
            total_blocked_calls: 0,
            blocked_by_mode: HashMap::new(),
            total_exception_matches: 0,
            calls_allowed_by_exception: 0,
        }
    }
}

impl Default for DndStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Do Not Disturb manager
pub struct DndManager {
    /// User DND status
    user_status: Arc<Mutex<HashMap<String, DndStatus>>>,
    /// Blocked call counter
    blocked_calls: Arc<Mutex<u64>>,
    /// Blocked calls by mode
    blocked_by_mode: Arc<Mutex<HashMap<String, u64>>>,
    /// Exception match counter
    exception_matches: Arc<Mutex<u64>>,
}

impl DndManager {
    pub fn new() -> Self {
        Self {
            user_status: Arc::new(Mutex::new(HashMap::new())),
            blocked_calls: Arc::new(Mutex::new(0)),
            blocked_by_mode: Arc::new(Mutex::new(HashMap::new())),
            exception_matches: Arc::new(Mutex::new(0)),
        }
    }

    /// Enable DND for a user
    pub fn enable_dnd(&self, user_id: &str, mode: DndMode, manual: bool) {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .entry(user_id.to_string())
            .or_insert_with(|| DndStatus::new(user_id.to_string()));

        status.enabled = true;
        status.mode = mode;
        status.manual_override = manual;
        status.enabled_at = Some(Utc::now());
    }

    /// Disable DND for a user
    pub fn disable_dnd(&self, user_id: &str) {
        let mut users = self.user_status.lock().unwrap();
        if let Some(status) = users.get_mut(user_id) {
            status.enabled = false;
            status.manual_override = false;
            status.disabled_at = Some(Utc::now());
        }
    }

    /// Toggle DND for a user
    pub fn toggle_dnd(&self, user_id: &str, mode: DndMode) -> bool {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .entry(user_id.to_string())
            .or_insert_with(|| DndStatus::new(user_id.to_string()));

        status.enabled = !status.enabled;
        status.manual_override = true;
        if status.enabled {
            status.mode = mode;
            status.enabled_at = Some(Utc::now());
        } else {
            status.disabled_at = Some(Utc::now());
        }

        status.enabled
    }

    /// Check if DND is enabled for a user
    pub fn is_enabled(&self, user_id: &str) -> bool {
        let users = self.user_status.lock().unwrap();
        users
            .get(user_id)
            .map(|status| {
                if status.manual_override {
                    status.enabled
                } else {
                    status.is_scheduled_active(Utc::now())
                }
            })
            .unwrap_or(false)
    }

    /// Get DND status for a user
    pub fn get_status(&self, user_id: &str) -> Option<DndStatus> {
        let users = self.user_status.lock().unwrap();
        users.get(user_id).cloned()
    }

    /// Set alternate destination for forwarding mode
    pub fn set_alternate_destination(&self, user_id: &str, destination: String) {
        let mut users = self.user_status.lock().unwrap();
        if let Some(status) = users.get_mut(user_id) {
            status.alternate_destination = Some(destination);
        }
    }

    /// Set announcement file for announcement mode
    pub fn set_announcement_file(&self, user_id: &str, file_path: String) {
        let mut users = self.user_status.lock().unwrap();
        if let Some(status) = users.get_mut(user_id) {
            status.announcement_file = Some(file_path);
        }
    }

    /// Add a DND schedule
    pub fn add_schedule(&self, user_id: &str, schedule: DndSchedule) -> Uuid {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .entry(user_id.to_string())
            .or_insert_with(|| DndStatus::new(user_id.to_string()));

        let schedule_id = schedule.id;
        status.schedules.push(schedule);
        schedule_id
    }

    /// Remove a DND schedule
    pub fn remove_schedule(&self, user_id: &str, schedule_id: Uuid) -> Result<(), String> {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let initial_len = status.schedules.len();
        status.schedules.retain(|s| s.id != schedule_id);

        if status.schedules.len() == initial_len {
            return Err("Schedule not found".to_string());
        }

        Ok(())
    }

    /// Add an exception rule
    pub fn add_exception(&self, user_id: &str, exception: DndException) -> Uuid {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .entry(user_id.to_string())
            .or_insert_with(|| DndStatus::new(user_id.to_string()));

        let exception_id = exception.id;
        status.exceptions.push(exception);
        exception_id
    }

    /// Remove an exception rule
    pub fn remove_exception(&self, user_id: &str, exception_id: Uuid) -> Result<(), String> {
        let mut users = self.user_status.lock().unwrap();
        let status = users
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let initial_len = status.exceptions.len();
        status.exceptions.retain(|e| e.id != exception_id);

        if status.exceptions.len() == initial_len {
            return Err("Exception not found".to_string());
        }

        Ok(())
    }

    /// Check if a call should be blocked by DND
    pub fn should_block_call(&self, user_id: &str, caller_id: &str) -> (bool, Option<DndMode>) {
        let users = self.user_status.lock().unwrap();

        if let Some(status) = users.get(user_id) {
            let current_time = Utc::now();
            let should_block = status.should_block_call(caller_id, current_time);

            if should_block {
                let mode = status.get_effective_mode(current_time)
                    .or(Some(status.mode));

                // Update statistics
                drop(users);
                self.record_blocked_call(mode.unwrap());

                return (true, mode);
            } else if status.is_scheduled_active(current_time) || (status.manual_override && status.enabled) {
                // DND is active but exception matched
                drop(users);
                let mut exception_matches = self.exception_matches.lock().unwrap();
                *exception_matches += 1;
            }
        }

        (false, None)
    }

    /// Record a blocked call
    fn record_blocked_call(&self, mode: DndMode) {
        let mut blocked_calls = self.blocked_calls.lock().unwrap();
        *blocked_calls += 1;

        let mut blocked_by_mode = self.blocked_by_mode.lock().unwrap();
        let mode_name = format!("{:?}", mode);
        *blocked_by_mode.entry(mode_name).or_insert(0) += 1;
    }

    /// Get DND statistics
    pub fn get_statistics(&self) -> DndStatistics {
        let users = self.user_status.lock().unwrap();
        let blocked_calls = self.blocked_calls.lock().unwrap();
        let blocked_by_mode = self.blocked_by_mode.lock().unwrap();
        let exception_matches = self.exception_matches.lock().unwrap();

        let mut stats = DndStatistics::new();
        stats.total_users = users.len();
        stats.users_with_dnd_enabled = users
            .values()
            .filter(|s| {
                if s.manual_override {
                    s.enabled
                } else {
                    s.is_scheduled_active(Utc::now())
                }
            })
            .count();
        stats.total_blocked_calls = *blocked_calls;
        stats.blocked_by_mode = blocked_by_mode.clone();
        stats.total_exception_matches = *exception_matches;
        stats.calls_allowed_by_exception = *exception_matches;

        stats
    }

    /// List all users with DND enabled
    pub fn list_users_with_dnd(&self) -> Vec<String> {
        let users = self.user_status.lock().unwrap();
        let current_time = Utc::now();

        users
            .iter()
            .filter(|(_, status)| {
                if status.manual_override {
                    status.enabled
                } else {
                    status.is_scheduled_active(current_time)
                }
            })
            .map(|(user_id, _)| user_id.clone())
            .collect()
    }

    /// Clear DND for all users (for testing)
    pub fn clear_all(&self) {
        let mut users = self.user_status.lock().unwrap();
        users.clear();
    }
}

impl Default for DndManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnd_mode_description() {
        assert_eq!(DndMode::RejectBusy.description(), "Reject with busy signal");
        assert_eq!(DndMode::SendToVoicemail.description(), "Send to voicemail");
    }

    #[test]
    fn test_dnd_mode_sip_codes() {
        assert_eq!(DndMode::RejectBusy.sip_response_code(), 486);
        assert_eq!(DndMode::SendToVoicemail.sip_response_code(), 302);
        assert_eq!(DndMode::SilentReject.sip_response_code(), 603);
    }

    #[test]
    fn test_dnd_schedule_is_active() {
        let schedule = DndSchedule::new(
            "Test".to_string(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            DndMode::RejectBusy,
        );

        let morning = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(schedule.is_active(morning, Weekday::Mon));

        let evening = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert!(!schedule.is_active(evening, Weekday::Mon));
    }

    #[test]
    fn test_dnd_schedule_midnight_crossing() {
        let schedule = DndSchedule::new(
            "Night".to_string(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            DndMode::SendToVoicemail,
        );

        let late_night = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        assert!(schedule.is_active(late_night, Weekday::Mon));

        let early_morning = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        assert!(schedule.is_active(early_morning, Weekday::Mon));

        let afternoon = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert!(!schedule.is_active(afternoon, Weekday::Mon));
    }

    #[test]
    fn test_dnd_schedule_weekday_filter() {
        let schedule = DndSchedule::business_hours(DndMode::RejectBusy);

        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(schedule.is_active(time, Weekday::Mon));
        assert!(schedule.is_active(time, Weekday::Fri));
        assert!(!schedule.is_active(time, Weekday::Sat));
        assert!(!schedule.is_active(time, Weekday::Sun));
    }

    #[test]
    fn test_exception_exact_match() {
        let exception = DndException::new(
            ExceptionType::Exact,
            vec!["1001".to_string(), "1002".to_string()],
        );

        assert!(exception.matches_caller("1001"));
        assert!(exception.matches_caller("1002"));
        assert!(!exception.matches_caller("1003"));
        assert!(!exception.matches_caller("10011"));
    }

    #[test]
    fn test_exception_prefix_match() {
        let exception = DndException::new(
            ExceptionType::Prefix,
            vec!["555".to_string()],
        );

        assert!(exception.matches_caller("5551234"));
        assert!(exception.matches_caller("5559999"));
        assert!(!exception.matches_caller("4441234"));
    }

    #[test]
    fn test_exception_wildcard_match() {
        let exception = DndException::new(
            ExceptionType::Wildcard,
            vec!["*911".to_string(), "555*".to_string()],
        );

        assert!(exception.matches_caller("911"));
        assert!(exception.matches_caller("555123"));
        assert!(exception.matches_caller("0911"));
        assert!(!exception.matches_caller("9110"));
    }

    #[test]
    fn test_wildcard_matching() {
        assert!(wildcard_match("*911", "911"));
        assert!(wildcard_match("*911", "0911"));
        assert!(!wildcard_match("*911", "9110"));

        assert!(wildcard_match("555*", "5551234"));
        assert!(!wildcard_match("555*", "4551234"));

        assert!(wildcard_match("*emergency*", "this_is_emergency_call"));
        assert!(!wildcard_match("*emergency*", "normal_call"));
    }

    #[test]
    fn test_enable_disable_dnd() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::RejectBusy, true);
        assert!(manager.is_enabled("alice"));

        manager.disable_dnd("alice");
        assert!(!manager.is_enabled("alice"));
    }

    #[test]
    fn test_toggle_dnd() {
        let manager = DndManager::new();

        let enabled = manager.toggle_dnd("alice", DndMode::SendToVoicemail);
        assert!(enabled);
        assert!(manager.is_enabled("alice"));

        let disabled = manager.toggle_dnd("alice", DndMode::SendToVoicemail);
        assert!(!disabled);
        assert!(!manager.is_enabled("alice"));
    }

    #[test]
    fn test_should_block_call_basic() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::RejectBusy, true);

        let (should_block, mode) = manager.should_block_call("alice", "1001");
        assert!(should_block);
        assert_eq!(mode, Some(DndMode::RejectBusy));
    }

    #[test]
    fn test_should_block_call_with_exception() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::RejectBusy, true);

        let exception = DndException::new(
            ExceptionType::Exact,
            vec!["1001".to_string()],
        );
        manager.add_exception("alice", exception);

        // Exception caller should not be blocked
        let (should_block_vip, _) = manager.should_block_call("alice", "1001");
        assert!(!should_block_vip);

        // Other callers should be blocked
        let (should_block_other, mode) = manager.should_block_call("alice", "1002");
        assert!(should_block_other);
        assert_eq!(mode, Some(DndMode::RejectBusy));
    }

    #[test]
    fn test_schedule_based_dnd() {
        let manager = DndManager::new();

        let schedule = DndSchedule::night_hours(DndMode::SendToVoicemail);
        manager.add_schedule("alice", schedule);

        // Check if schedule would be active (depends on current time in real test)
        let status = manager.get_status("alice").unwrap();
        assert_eq!(status.schedules.len(), 1);
    }

    #[test]
    fn test_add_remove_schedule() {
        let manager = DndManager::new();

        let schedule = DndSchedule::business_hours(DndMode::RejectBusy);
        let schedule_id = manager.add_schedule("alice", schedule);

        let status = manager.get_status("alice").unwrap();
        assert_eq!(status.schedules.len(), 1);

        manager.remove_schedule("alice", schedule_id).unwrap();

        let status_after = manager.get_status("alice").unwrap();
        assert_eq!(status_after.schedules.len(), 0);
    }

    #[test]
    fn test_add_remove_exception() {
        let manager = DndManager::new();

        let exception = DndException::new(
            ExceptionType::Exact,
            vec!["1001".to_string()],
        );
        let exception_id = manager.add_exception("alice", exception);

        let status = manager.get_status("alice").unwrap();
        assert_eq!(status.exceptions.len(), 1);

        manager.remove_exception("alice", exception_id).unwrap();

        let status_after = manager.get_status("alice").unwrap();
        assert_eq!(status_after.exceptions.len(), 0);
    }

    #[test]
    fn test_set_alternate_destination() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::ForwardToAlternate, true);
        manager.set_alternate_destination("alice", "bob".to_string());

        let status = manager.get_status("alice").unwrap();
        assert_eq!(status.alternate_destination, Some("bob".to_string()));
    }

    #[test]
    fn test_dnd_statistics() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::RejectBusy, true);
        manager.enable_dnd("bob", DndMode::SendToVoicemail, true);

        manager.should_block_call("alice", "1001");
        manager.should_block_call("alice", "1002");
        manager.should_block_call("bob", "1003");

        let stats = manager.get_statistics();
        assert_eq!(stats.users_with_dnd_enabled, 2);
        assert_eq!(stats.total_blocked_calls, 3);
    }

    #[test]
    fn test_list_users_with_dnd() {
        let manager = DndManager::new();

        manager.enable_dnd("alice", DndMode::RejectBusy, true);
        manager.enable_dnd("bob", DndMode::SendToVoicemail, true);
        manager.enable_dnd("charlie", DndMode::RejectBusy, true);
        manager.disable_dnd("charlie");

        let users = manager.list_users_with_dnd();
        assert_eq!(users.len(), 2);
        assert!(users.contains(&"alice".to_string()));
        assert!(users.contains(&"bob".to_string()));
        assert!(!users.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_disabled_exception_not_matched() {
        let exception = DndException::new(
            ExceptionType::Exact,
            vec!["1001".to_string()],
        )
        .disabled();

        assert!(!exception.matches_caller("1001"));
    }

    #[test]
    fn test_disabled_schedule_not_active() {
        let schedule = DndSchedule::business_hours(DndMode::RejectBusy).disabled();

        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(!schedule.is_active(time, Weekday::Mon));
    }
}

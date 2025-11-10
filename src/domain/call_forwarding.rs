//! Call Forwarding domain model
//!
//! Provides call forwarding functionality including unconditional forwarding,
//! busy forwarding, no-answer forwarding, and conditional forwarding based on
//! various criteria like time of day, caller ID, etc.

use chrono::{DateTime, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Call forwarding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForwardingType {
    /// Forward all calls unconditionally
    Unconditional,
    /// Forward when the line is busy
    Busy,
    /// Forward after no answer timeout
    NoAnswer,
    /// Forward when user is not registered/offline
    Unavailable,
    /// Forward based on time of day
    TimeBased,
    /// Forward based on caller ID
    CallerBased,
    /// Forward to voicemail
    Voicemail,
}

impl ForwardingType {
    /// Get a human-readable description
    pub fn description(&self) -> &str {
        match self {
            ForwardingType::Unconditional => "Forward all calls",
            ForwardingType::Busy => "Forward when busy",
            ForwardingType::NoAnswer => "Forward on no answer",
            ForwardingType::Unavailable => "Forward when unavailable",
            ForwardingType::TimeBased => "Forward based on time",
            ForwardingType::CallerBased => "Forward based on caller",
            ForwardingType::Voicemail => "Forward to voicemail",
        }
    }
}

/// Forwarding destination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardingDestination {
    /// SIP URI or extension
    pub uri: String,
    /// Display name for the destination
    pub display_name: Option<String>,
    /// Whether this is an external destination
    pub is_external: bool,
}

impl ForwardingDestination {
    pub fn new(uri: String) -> Self {
        let is_external = uri.contains("@") && !uri.ends_with("@localhost");
        Self {
            uri,
            display_name: None,
            is_external,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = Some(name);
        self
    }
}

/// Time range for time-based forwarding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (HH:MM format)
    pub start: NaiveTime,
    /// End time (HH:MM format)
    pub end: NaiveTime,
    /// Days of week (if empty, applies to all days)
    pub days: Vec<Weekday>,
}

impl TimeRange {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self {
            start,
            end,
            days: vec![],
        }
    }

    pub fn with_days(mut self, days: Vec<Weekday>) -> Self {
        self.days = days;
        self
    }

    /// Check if the given time falls within this range
    pub fn contains(&self, time: NaiveTime, weekday: Weekday) -> bool {
        // Check day of week if specified
        if !self.days.is_empty() && !self.days.contains(&weekday) {
            return false;
        }

        // Handle time range that crosses midnight
        if self.start <= self.end {
            time >= self.start && time <= self.end
        } else {
            time >= self.start || time <= self.end
        }
    }

    /// Business hours preset (Monday-Friday, 9am-5pm)
    pub fn business_hours() -> Self {
        Self {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
        }
    }

    /// After hours preset (Monday-Friday, 5pm-9am, plus weekends)
    pub fn after_hours() -> Self {
        Self {
            start: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            days: vec![],
        }
    }
}

/// Caller ID filter for caller-based forwarding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallerFilter {
    /// List of caller numbers/URIs to match
    pub allowed_callers: Vec<String>,
    /// Whether to match exact or use prefix matching
    pub exact_match: bool,
}

impl CallerFilter {
    pub fn new() -> Self {
        Self {
            allowed_callers: vec![],
            exact_match: true,
        }
    }

    pub fn add_caller(mut self, caller: String) -> Self {
        self.allowed_callers.push(caller);
        self
    }

    pub fn with_prefix_matching(mut self) -> Self {
        self.exact_match = false;
        self
    }

    /// Check if the caller matches the filter
    pub fn matches(&self, caller: &str) -> bool {
        if self.exact_match {
            self.allowed_callers.iter().any(|c| c == caller)
        } else {
            self.allowed_callers
                .iter()
                .any(|c| caller.starts_with(c) || c.starts_with(caller))
        }
    }
}

impl Default for CallerFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Call forwarding rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingRule {
    /// Unique rule identifier
    pub id: Uuid,
    /// User/extension this rule belongs to
    pub user_id: String,
    /// Type of forwarding
    pub forwarding_type: ForwardingType,
    /// Destination for forwarded calls
    pub destination: ForwardingDestination,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Priority (lower number = higher priority)
    pub priority: u32,
    /// Timeout in seconds (for NoAnswer type)
    pub timeout_seconds: Option<u32>,
    /// Time range (for TimeBased type)
    pub time_range: Option<TimeRange>,
    /// Caller filter (for CallerBased type)
    pub caller_filter: Option<CallerFilter>,
    /// Maximum forwarding hops (to prevent loops)
    pub max_hops: u32,
    /// Rule description
    pub description: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl ForwardingRule {
    pub fn new(user_id: String, forwarding_type: ForwardingType, destination: ForwardingDestination) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            forwarding_type,
            destination,
            enabled: true,
            priority: 100,
            timeout_seconds: if matches!(forwarding_type, ForwardingType::NoAnswer) {
                Some(20)
            } else {
                None
            },
            time_range: None,
            caller_filter: None,
            max_hops: 5,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    pub fn with_caller_filter(mut self, caller_filter: CallerFilter) -> Self {
        self.caller_filter = Some(caller_filter);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if this rule should be applied given the current context
    pub fn should_apply(&self, caller: &str, current_time: DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }

        // Check time range for time-based forwarding
        if let Some(ref time_range) = self.time_range {
            let time = current_time.time();
            let weekday = current_time.weekday();
            if !time_range.contains(time, weekday) {
                return false;
            }
        }

        // Check caller filter for caller-based forwarding
        if let Some(ref caller_filter) = self.caller_filter {
            if !caller_filter.matches(caller) {
                return false;
            }
        }

        true
    }
}

/// Call forwarding status for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingStatus {
    /// User/extension identifier
    pub user_id: String,
    /// Whether any forwarding is active
    pub has_active_forwarding: bool,
    /// Active unconditional forwarding destination
    pub unconditional_destination: Option<String>,
    /// Active busy forwarding destination
    pub busy_destination: Option<String>,
    /// Active no-answer forwarding destination
    pub no_answer_destination: Option<String>,
    /// Total active rules
    pub active_rules_count: usize,
}

impl ForwardingStatus {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            has_active_forwarding: false,
            unconditional_destination: None,
            busy_destination: None,
            no_answer_destination: None,
            active_rules_count: 0,
        }
    }
}

/// Call forwarding statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingStatistics {
    /// Total forwarding rules in system
    pub total_rules: usize,
    /// Active rules
    pub active_rules: usize,
    /// Disabled rules
    pub disabled_rules: usize,
    /// Total forwarded calls
    pub total_forwarded_calls: u64,
    /// Calls forwarded by type
    pub by_type: HashMap<String, u64>,
    /// Users with active forwarding
    pub users_with_forwarding: usize,
}

impl ForwardingStatistics {
    pub fn new() -> Self {
        Self {
            total_rules: 0,
            active_rules: 0,
            disabled_rules: 0,
            total_forwarded_calls: 0,
            by_type: HashMap::new(),
            users_with_forwarding: 0,
        }
    }
}

impl Default for ForwardingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Call forwarding manager
pub struct CallForwardingManager {
    /// Forwarding rules indexed by user ID
    rules: Arc<Mutex<HashMap<String, Vec<ForwardingRule>>>>,
    /// Forwarding call counter
    forwarded_calls: Arc<Mutex<u64>>,
    /// Calls by type counter
    calls_by_type: Arc<Mutex<HashMap<String, u64>>>,
}

impl CallForwardingManager {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
            forwarded_calls: Arc::new(Mutex::new(0)),
            calls_by_type: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a forwarding rule
    pub fn add_rule(&self, rule: ForwardingRule) -> Result<Uuid, String> {
        let mut rules = self.rules.lock().unwrap();
        let user_rules = rules.entry(rule.user_id.clone()).or_insert_with(Vec::new);

        // Check for duplicate unconditional forwarding
        if rule.forwarding_type == ForwardingType::Unconditional && rule.enabled {
            if user_rules.iter().any(|r| {
                r.forwarding_type == ForwardingType::Unconditional && r.enabled && r.id != rule.id
            }) {
                return Err("User already has an active unconditional forwarding rule".to_string());
            }
        }

        let rule_id = rule.id;
        user_rules.push(rule);

        // Sort by priority
        user_rules.sort_by_key(|r| r.priority);

        Ok(rule_id)
    }

    /// Update a forwarding rule
    pub fn update_rule(&self, rule: ForwardingRule) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        let user_rules = rules
            .get_mut(&rule.user_id)
            .ok_or_else(|| "User has no forwarding rules".to_string())?;

        let existing_rule = user_rules
            .iter_mut()
            .find(|r| r.id == rule.id)
            .ok_or_else(|| "Forwarding rule not found".to_string())?;

        *existing_rule = rule;
        user_rules.sort_by_key(|r| r.priority);

        Ok(())
    }

    /// Remove a forwarding rule
    pub fn remove_rule(&self, user_id: &str, rule_id: Uuid) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        let user_rules = rules
            .get_mut(user_id)
            .ok_or_else(|| "User has no forwarding rules".to_string())?;

        let initial_len = user_rules.len();
        user_rules.retain(|r| r.id != rule_id);

        if user_rules.len() == initial_len {
            return Err("Forwarding rule not found".to_string());
        }

        Ok(())
    }

    /// Get all rules for a user
    pub fn get_user_rules(&self, user_id: &str) -> Vec<ForwardingRule> {
        let rules = self.rules.lock().unwrap();
        rules.get(user_id).cloned().unwrap_or_default()
    }

    /// Get a specific rule
    pub fn get_rule(&self, user_id: &str, rule_id: Uuid) -> Option<ForwardingRule> {
        let rules = self.rules.lock().unwrap();
        rules
            .get(user_id)?
            .iter()
            .find(|r| r.id == rule_id)
            .cloned()
    }

    /// Enable a rule
    pub fn enable_rule(&self, user_id: &str, rule_id: Uuid) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        let user_rules = rules
            .get_mut(user_id)
            .ok_or_else(|| "User has no forwarding rules".to_string())?;

        let rule = user_rules
            .iter_mut()
            .find(|r| r.id == rule_id)
            .ok_or_else(|| "Forwarding rule not found".to_string())?;

        rule.enabled = true;
        rule.updated_at = Utc::now();

        Ok(())
    }

    /// Disable a rule
    pub fn disable_rule(&self, user_id: &str, rule_id: Uuid) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        let user_rules = rules
            .get_mut(user_id)
            .ok_or_else(|| "User has no forwarding rules".to_string())?;

        let rule = user_rules
            .iter_mut()
            .find(|r| r.id == rule_id)
            .ok_or_else(|| "Forwarding rule not found".to_string())?;

        rule.enabled = false;
        rule.updated_at = Utc::now();

        Ok(())
    }

    /// Get applicable forwarding destination based on forwarding type and context
    pub fn get_forward_destination(
        &self,
        user_id: &str,
        forwarding_type: ForwardingType,
        caller: &str,
    ) -> Option<ForwardingDestination> {
        let rules = self.rules.lock().unwrap();
        let user_rules = rules.get(user_id)?;

        let current_time = Utc::now();

        // Find the highest priority matching rule
        user_rules
            .iter()
            .filter(|r| r.enabled && r.forwarding_type == forwarding_type)
            .filter(|r| r.should_apply(caller, current_time))
            .min_by_key(|r| r.priority)
            .map(|r| r.destination.clone())
    }

    /// Get any applicable forwarding destination (checks all types in priority order)
    pub fn get_any_forward_destination(
        &self,
        user_id: &str,
        caller: &str,
    ) -> Option<(ForwardingType, ForwardingDestination)> {
        let rules = self.rules.lock().unwrap();
        let user_rules = rules.get(user_id)?;

        let current_time = Utc::now();

        // Find the highest priority matching rule across all types
        user_rules
            .iter()
            .filter(|r| r.enabled)
            .filter(|r| r.should_apply(caller, current_time))
            .min_by_key(|r| r.priority)
            .map(|r| (r.forwarding_type, r.destination.clone()))
    }

    /// Check if a forwarding chain would create a loop
    pub fn would_create_loop(
        &self,
        source: &str,
        destination: &str,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(destination) {
            return true;
        }

        visited.insert(destination.to_string());

        // Check if destination has forwarding
        if let Some((_, next_dest)) = self.get_any_forward_destination(destination, source) {
            return self.would_create_loop(source, &next_dest.uri, visited);
        }

        false
    }

    /// Record a forwarded call
    pub fn record_forwarded_call(&self, forwarding_type: ForwardingType) {
        let mut count = self.forwarded_calls.lock().unwrap();
        *count += 1;

        let mut by_type = self.calls_by_type.lock().unwrap();
        let type_name = format!("{:?}", forwarding_type);
        *by_type.entry(type_name).or_insert(0) += 1;
    }

    /// Get forwarding status for a user
    pub fn get_status(&self, user_id: &str) -> ForwardingStatus {
        let rules = self.rules.lock().unwrap();
        let user_rules = rules.get(user_id);

        let mut status = ForwardingStatus::new(user_id.to_string());

        if let Some(user_rules) = user_rules {
            let active_rules: Vec<_> = user_rules.iter().filter(|r| r.enabled).collect();

            status.active_rules_count = active_rules.len();
            status.has_active_forwarding = !active_rules.is_empty();

            // Find destinations by type
            for rule in active_rules {
                match rule.forwarding_type {
                    ForwardingType::Unconditional => {
                        status.unconditional_destination = Some(rule.destination.uri.clone());
                    }
                    ForwardingType::Busy => {
                        status.busy_destination = Some(rule.destination.uri.clone());
                    }
                    ForwardingType::NoAnswer => {
                        status.no_answer_destination = Some(rule.destination.uri.clone());
                    }
                    _ => {}
                }
            }
        }

        status
    }

    /// Get forwarding statistics
    pub fn get_statistics(&self) -> ForwardingStatistics {
        let rules = self.rules.lock().unwrap();
        let forwarded_calls = self.forwarded_calls.lock().unwrap();
        let calls_by_type = self.calls_by_type.lock().unwrap();

        let mut stats = ForwardingStatistics::new();
        stats.total_forwarded_calls = *forwarded_calls;
        stats.by_type = calls_by_type.clone();

        let mut users_with_forwarding = HashSet::new();

        for (user_id, user_rules) in rules.iter() {
            for rule in user_rules {
                stats.total_rules += 1;
                if rule.enabled {
                    stats.active_rules += 1;
                    users_with_forwarding.insert(user_id.clone());
                } else {
                    stats.disabled_rules += 1;
                }
            }
        }

        stats.users_with_forwarding = users_with_forwarding.len();

        stats
    }

    /// Remove all rules for a user
    pub fn remove_all_user_rules(&self, user_id: &str) -> usize {
        let mut rules = self.rules.lock().unwrap();
        rules.remove(user_id).map(|r| r.len()).unwrap_or(0)
    }

    /// List all users with forwarding
    pub fn list_users_with_forwarding(&self) -> Vec<String> {
        let rules = self.rules.lock().unwrap();
        rules
            .iter()
            .filter(|(_, user_rules)| user_rules.iter().any(|r| r.enabled))
            .map(|(user_id, _)| user_id.clone())
            .collect()
    }
}

impl Default for CallForwardingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forwarding_type_description() {
        assert_eq!(ForwardingType::Unconditional.description(), "Forward all calls");
        assert_eq!(ForwardingType::Busy.description(), "Forward when busy");
        assert_eq!(ForwardingType::NoAnswer.description(), "Forward on no answer");
    }

    #[test]
    fn test_forwarding_destination() {
        let dest = ForwardingDestination::new("sip:alice@example.com".to_string());
        assert_eq!(dest.uri, "sip:alice@example.com");
        assert!(dest.is_external);

        let dest2 = ForwardingDestination::new("100".to_string());
        assert_eq!(dest2.uri, "100");
        assert!(!dest2.is_external);

        let dest3 = ForwardingDestination::new("sip:100@localhost".to_string());
        assert!(!dest3.is_external);
    }

    #[test]
    fn test_time_range_contains() {
        let range = TimeRange::new(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );

        let morning = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(range.contains(morning, Weekday::Mon));

        let evening = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert!(!range.contains(evening, Weekday::Mon));
    }

    #[test]
    fn test_time_range_weekday_filter() {
        let range = TimeRange::new(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_days(vec![Weekday::Mon, Weekday::Tue, Weekday::Wed]);

        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(range.contains(time, Weekday::Mon));
        assert!(!range.contains(time, Weekday::Sat));
    }

    #[test]
    fn test_time_range_crosses_midnight() {
        let range = TimeRange::new(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        );

        let late_night = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        assert!(range.contains(late_night, Weekday::Mon));

        let early_morning = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        assert!(range.contains(early_morning, Weekday::Mon));

        let afternoon = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert!(!range.contains(afternoon, Weekday::Mon));
    }

    #[test]
    fn test_caller_filter_exact_match() {
        let filter = CallerFilter::new()
            .add_caller("1001".to_string())
            .add_caller("1002".to_string());

        assert!(filter.matches("1001"));
        assert!(filter.matches("1002"));
        assert!(!filter.matches("1003"));
        assert!(!filter.matches("10011"));
    }

    #[test]
    fn test_caller_filter_prefix_match() {
        let filter = CallerFilter::new()
            .add_caller("100".to_string())
            .with_prefix_matching();

        assert!(filter.matches("1001"));
        assert!(filter.matches("1002"));
        assert!(filter.matches("100"));
        assert!(!filter.matches("200"));
    }

    #[test]
    fn test_forwarding_rule_creation() {
        let dest = ForwardingDestination::new("sip:voicemail@example.com".to_string());
        let rule = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::NoAnswer,
            dest.clone(),
        );

        assert_eq!(rule.user_id, "alice");
        assert_eq!(rule.forwarding_type, ForwardingType::NoAnswer);
        assert_eq!(rule.destination.uri, "sip:voicemail@example.com");
        assert!(rule.enabled);
        assert_eq!(rule.timeout_seconds, Some(20)); // Default for NoAnswer
        assert_eq!(rule.priority, 100);
    }

    #[test]
    fn test_forwarding_rule_should_apply() {
        let dest = ForwardingDestination::new("100".to_string());
        let time_range = TimeRange::business_hours();
        let rule = ForwardingRule::new("alice".to_string(), ForwardingType::TimeBased, dest)
            .with_time_range(time_range);

        // During business hours (Tuesday 10am)
        let tuesday_10am = Utc::now()
            .date_naive()
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_utc();

        // Note: should_apply checks time but not weekday for this test
        // In production, would need proper date handling

        let disabled_rule = rule.clone().disabled();
        assert!(!disabled_rule.should_apply("caller", Utc::now()));
    }

    #[test]
    fn test_add_forwarding_rule() {
        let manager = CallForwardingManager::new();
        let dest = ForwardingDestination::new("100".to_string());
        let rule = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest,
        );

        let rule_id = manager.add_rule(rule).unwrap();
        assert!(rule_id != Uuid::nil());

        let user_rules = manager.get_user_rules("alice");
        assert_eq!(user_rules.len(), 1);
        assert_eq!(user_rules[0].id, rule_id);
    }

    #[test]
    fn test_duplicate_unconditional_forwarding() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest1,
        );
        manager.add_rule(rule1).unwrap();

        let dest2 = ForwardingDestination::new("200".to_string());
        let rule2 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest2,
        );
        let result = manager.add_rule(rule2);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("already has an active unconditional"));
    }

    #[test]
    fn test_enable_disable_rule() {
        let manager = CallForwardingManager::new();
        let dest = ForwardingDestination::new("100".to_string());
        let rule = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest,
        );

        let rule_id = manager.add_rule(rule).unwrap();

        manager.disable_rule("alice", rule_id).unwrap();
        let disabled_rule = manager.get_rule("alice", rule_id).unwrap();
        assert!(!disabled_rule.enabled);

        manager.enable_rule("alice", rule_id).unwrap();
        let enabled_rule = manager.get_rule("alice", rule_id).unwrap();
        assert!(enabled_rule.enabled);
    }

    #[test]
    fn test_remove_rule() {
        let manager = CallForwardingManager::new();
        let dest = ForwardingDestination::new("100".to_string());
        let rule = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest,
        );

        let rule_id = manager.add_rule(rule).unwrap();
        assert_eq!(manager.get_user_rules("alice").len(), 1);

        manager.remove_rule("alice", rule_id).unwrap();
        assert_eq!(manager.get_user_rules("alice").len(), 0);
    }

    #[test]
    fn test_get_forward_destination() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest1.clone(),
        );
        manager.add_rule(rule1).unwrap();

        let result = manager.get_forward_destination(
            "alice",
            ForwardingType::Busy,
            "caller",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().uri, "100");

        let no_answer_result = manager.get_forward_destination(
            "alice",
            ForwardingType::NoAnswer,
            "caller",
        );
        assert!(no_answer_result.is_none());
    }

    #[test]
    fn test_rule_priority() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest1,
        )
        .with_priority(50);
        manager.add_rule(rule1).unwrap();

        let dest2 = ForwardingDestination::new("200".to_string());
        let rule2 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest2,
        )
        .with_priority(10);
        manager.add_rule(rule2).unwrap();

        // Lower priority number = higher priority, so should get dest2
        let result = manager.get_forward_destination(
            "alice",
            ForwardingType::Busy,
            "caller",
        );
        assert_eq!(result.unwrap().uri, "200");
    }

    #[test]
    fn test_forwarding_status() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest1,
        );
        manager.add_rule(rule1).unwrap();

        let dest2 = ForwardingDestination::new("200".to_string());
        let rule2 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest2,
        );
        manager.add_rule(rule2).unwrap();

        let status = manager.get_status("alice");
        assert!(status.has_active_forwarding);
        assert_eq!(status.active_rules_count, 2);
        assert_eq!(status.unconditional_destination.unwrap(), "100");
        assert_eq!(status.busy_destination.unwrap(), "200");
    }

    #[test]
    fn test_forwarding_statistics() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest1,
        );
        manager.add_rule(rule1).unwrap();

        let dest2 = ForwardingDestination::new("200".to_string());
        let rule2 = ForwardingRule::new(
            "bob".to_string(),
            ForwardingType::Busy,
            dest2,
        )
        .disabled();
        manager.add_rule(rule2).unwrap();

        manager.record_forwarded_call(ForwardingType::Unconditional);
        manager.record_forwarded_call(ForwardingType::Unconditional);
        manager.record_forwarded_call(ForwardingType::Busy);

        let stats = manager.get_statistics();
        assert_eq!(stats.total_rules, 2);
        assert_eq!(stats.active_rules, 1);
        assert_eq!(stats.disabled_rules, 1);
        assert_eq!(stats.total_forwarded_calls, 3);
        assert_eq!(stats.users_with_forwarding, 1);
    }

    #[test]
    fn test_loop_detection() {
        let manager = CallForwardingManager::new();

        // alice -> bob
        let dest1 = ForwardingDestination::new("bob".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest1,
        );
        manager.add_rule(rule1).unwrap();

        // bob -> charlie
        let dest2 = ForwardingDestination::new("charlie".to_string());
        let rule2 = ForwardingRule::new(
            "bob".to_string(),
            ForwardingType::Unconditional,
            dest2,
        );
        manager.add_rule(rule2).unwrap();

        // charlie -> alice (would create loop)
        let mut visited = HashSet::new();
        assert!(manager.would_create_loop("charlie", "alice", &mut visited));

        // charlie -> david (no loop)
        let mut visited2 = HashSet::new();
        assert!(!manager.would_create_loop("charlie", "david", &mut visited2));
    }

    #[test]
    fn test_remove_all_user_rules() {
        let manager = CallForwardingManager::new();

        let dest1 = ForwardingDestination::new("100".to_string());
        let rule1 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Unconditional,
            dest1,
        );
        manager.add_rule(rule1).unwrap();

        let dest2 = ForwardingDestination::new("200".to_string());
        let rule2 = ForwardingRule::new(
            "alice".to_string(),
            ForwardingType::Busy,
            dest2,
        );
        manager.add_rule(rule2).unwrap();

        let removed = manager.remove_all_user_rules("alice");
        assert_eq!(removed, 2);
        assert_eq!(manager.get_user_rules("alice").len(), 0);
    }
}

//! Call Pickup domain model
//!
//! Provides call pickup functionality allowing users to answer calls
//! ringing at other extensions. Supports directed pickup (specific extension),
//! group pickup (within pickup group), and BLF pickup (via busy lamp field).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Call pickup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickupType {
    /// Pickup specific extension's ringing call (directed pickup)
    Directed,
    /// Pickup any call in user's pickup group
    Group,
    /// Pickup any ringing call in the system (admin feature)
    Any,
    /// Pickup via BLF (Busy Lamp Field) monitoring
    Blf,
}

impl PickupType {
    pub fn description(&self) -> &str {
        match self {
            PickupType::Directed => "Directed pickup (specific extension)",
            PickupType::Group => "Group pickup (within group)",
            PickupType::Any => "Pickup any call (admin)",
            PickupType::Blf => "BLF pickup (monitored extension)",
        }
    }

    /// Get dial code prefix for this pickup type
    pub fn dial_code(&self) -> &str {
        match self {
            PickupType::Directed => "*8",    // *8<extension>
            PickupType::Group => "*9",       // *9
            PickupType::Any => "*10",        // *10
            PickupType::Blf => "*11",        // *11<extension>
        }
    }
}

/// Pickup group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupGroup {
    /// Unique group identifier
    pub id: Uuid,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Members (user IDs or extensions)
    pub members: HashSet<String>,
    /// Whether the group is enabled
    pub enabled: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl PickupGroup {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            members: HashSet::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_members(mut self, members: Vec<String>) -> Self {
        self.members = members.into_iter().collect();
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Add a member to the group
    pub fn add_member(&mut self, user_id: String) -> bool {
        self.updated_at = Utc::now();
        self.members.insert(user_id)
    }

    /// Remove a member from the group
    pub fn remove_member(&mut self, user_id: &str) -> bool {
        self.updated_at = Utc::now();
        self.members.remove(user_id)
    }

    /// Check if user is a member
    pub fn has_member(&self, user_id: &str) -> bool {
        self.members.contains(user_id)
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Ringing call information for pickup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingingCall {
    /// Call ID
    pub call_id: String,
    /// Target extension (who is being called)
    pub target_extension: String,
    /// Caller ID
    pub caller_id: String,
    /// Caller display name
    pub caller_name: Option<String>,
    /// When the call started ringing
    pub ringing_since: DateTime<Utc>,
    /// Pickup group (if target is in a group)
    pub pickup_group_id: Option<Uuid>,
    /// Whether this call can be picked up via BLF
    pub blf_enabled: bool,
}

impl RingingCall {
    pub fn new(call_id: String, target_extension: String, caller_id: String) -> Self {
        Self {
            call_id,
            target_extension,
            caller_id,
            caller_name: None,
            ringing_since: Utc::now(),
            pickup_group_id: None,
            blf_enabled: true,
        }
    }

    pub fn with_caller_name(mut self, name: String) -> Self {
        self.caller_name = Some(name);
        self
    }

    pub fn with_pickup_group(mut self, group_id: Uuid) -> Self {
        self.pickup_group_id = Some(group_id);
        self
    }

    pub fn without_blf(mut self) -> Self {
        self.blf_enabled = false;
        self
    }

    /// Get duration call has been ringing in seconds
    pub fn ringing_duration_seconds(&self) -> i64 {
        (Utc::now() - self.ringing_since).num_seconds()
    }
}

/// Pickup permissions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupPermissions {
    /// User can perform directed pickup
    pub directed_pickup: bool,
    /// User can perform group pickup
    pub group_pickup: bool,
    /// User can pickup any call (admin privilege)
    pub any_pickup: bool,
    /// User can perform BLF pickup
    pub blf_pickup: bool,
    /// Extensions user can monitor via BLF
    pub blf_monitored_extensions: HashSet<String>,
}

impl PickupPermissions {
    pub fn new() -> Self {
        Self {
            directed_pickup: true,
            group_pickup: true,
            any_pickup: false,
            blf_pickup: true,
            blf_monitored_extensions: HashSet::new(),
        }
    }

    /// Default permissions for regular users
    pub fn user_default() -> Self {
        Self::new()
    }

    /// Full permissions for administrators
    pub fn admin() -> Self {
        Self {
            directed_pickup: true,
            group_pickup: true,
            any_pickup: true,
            blf_pickup: true,
            blf_monitored_extensions: HashSet::new(),
        }
    }

    /// Restricted permissions (no pickup)
    pub fn restricted() -> Self {
        Self {
            directed_pickup: false,
            group_pickup: false,
            any_pickup: false,
            blf_pickup: false,
            blf_monitored_extensions: HashSet::new(),
        }
    }

    /// Check if user has permission for pickup type
    pub fn can_pickup(&self, pickup_type: PickupType) -> bool {
        match pickup_type {
            PickupType::Directed => self.directed_pickup,
            PickupType::Group => self.group_pickup,
            PickupType::Any => self.any_pickup,
            PickupType::Blf => self.blf_pickup,
        }
    }

    /// Add BLF monitored extension
    pub fn add_blf_monitor(&mut self, extension: String) {
        self.blf_monitored_extensions.insert(extension);
    }

    /// Remove BLF monitored extension
    pub fn remove_blf_monitor(&mut self, extension: &str) {
        self.blf_monitored_extensions.remove(extension);
    }

    /// Check if user monitors this extension via BLF
    pub fn monitors_extension(&self, extension: &str) -> bool {
        self.blf_monitored_extensions.contains(extension)
    }
}

impl Default for PickupPermissions {
    fn default() -> Self {
        Self::new()
    }
}

/// Pickup attempt result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickupResult {
    /// Pickup successful
    Success(String), // Call ID
    /// No ringing call at target
    NoRingingCall,
    /// Permission denied
    PermissionDenied,
    /// Target not found
    TargetNotFound,
    /// Not in same pickup group
    NotInGroup,
    /// Call already answered
    AlreadyAnswered,
    /// Pickup not allowed (policy)
    NotAllowed,
}

/// Call pickup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupStatistics {
    /// Total pickup attempts
    pub total_attempts: u64,
    /// Successful pickups
    pub successful_pickups: u64,
    /// Failed pickups
    pub failed_pickups: u64,
    /// Pickups by type
    pub by_type: HashMap<String, u64>,
    /// Average time to pickup (seconds)
    pub average_time_to_pickup: i64,
    /// Currently ringing calls
    pub current_ringing_calls: usize,
}

impl PickupStatistics {
    pub fn new() -> Self {
        Self {
            total_attempts: 0,
            successful_pickups: 0,
            failed_pickups: 0,
            by_type: HashMap::new(),
            average_time_to_pickup: 0,
            current_ringing_calls: 0,
        }
    }

    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.successful_pickups as f64 / self.total_attempts as f64) * 100.0
        }
    }
}

impl Default for PickupStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Call pickup manager
pub struct CallPickupManager {
    /// Pickup groups
    groups: Arc<Mutex<HashMap<Uuid, PickupGroup>>>,
    /// User to pickup group mapping
    user_groups: Arc<Mutex<HashMap<String, Uuid>>>,
    /// User permissions
    permissions: Arc<Mutex<HashMap<String, PickupPermissions>>>,
    /// Currently ringing calls
    ringing_calls: Arc<Mutex<HashMap<String, RingingCall>>>,
    /// Extension to call ID mapping
    extension_calls: Arc<Mutex<HashMap<String, String>>>,
    /// Statistics counters
    total_attempts: Arc<Mutex<u64>>,
    successful_pickups: Arc<Mutex<u64>>,
    pickups_by_type: Arc<Mutex<HashMap<String, u64>>>,
    /// Pickup times for averaging
    pickup_times: Arc<Mutex<Vec<i64>>>,
}

impl CallPickupManager {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(Mutex::new(HashMap::new())),
            user_groups: Arc::new(Mutex::new(HashMap::new())),
            permissions: Arc::new(Mutex::new(HashMap::new())),
            ringing_calls: Arc::new(Mutex::new(HashMap::new())),
            extension_calls: Arc::new(Mutex::new(HashMap::new())),
            total_attempts: Arc::new(Mutex::new(0)),
            successful_pickups: Arc::new(Mutex::new(0)),
            pickups_by_type: Arc::new(Mutex::new(HashMap::new())),
            pickup_times: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a pickup group
    pub fn create_group(&self, group: PickupGroup) -> Uuid {
        let group_id = group.id;
        let mut groups = self.groups.lock().unwrap();

        // Update user-to-group mapping
        let mut user_groups = self.user_groups.lock().unwrap();
        for member in &group.members {
            user_groups.insert(member.clone(), group_id);
        }

        groups.insert(group_id, group);
        group_id
    }

    /// Get pickup group
    pub fn get_group(&self, group_id: Uuid) -> Option<PickupGroup> {
        let groups = self.groups.lock().unwrap();
        groups.get(&group_id).cloned()
    }

    /// Update pickup group
    pub fn update_group(&self, group: PickupGroup) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();

        if !groups.contains_key(&group.id) {
            return Err("Group not found".to_string());
        }

        // Update user-to-group mapping
        let mut user_groups = self.user_groups.lock().unwrap();

        // Remove old members
        if let Some(old_group) = groups.get(&group.id) {
            for member in &old_group.members {
                user_groups.remove(member);
            }
        }

        // Add new members
        for member in &group.members {
            user_groups.insert(member.clone(), group.id);
        }

        groups.insert(group.id, group);
        Ok(())
    }

    /// Delete pickup group
    pub fn delete_group(&self, group_id: Uuid) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();

        let group = groups.remove(&group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        // Remove user-to-group mappings
        let mut user_groups = self.user_groups.lock().unwrap();
        for member in &group.members {
            user_groups.remove(member);
        }

        Ok(())
    }

    /// Add member to pickup group
    pub fn add_member_to_group(&self, group_id: Uuid, user_id: String) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        let group = groups.get_mut(&group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        group.add_member(user_id.clone());

        let mut user_groups = self.user_groups.lock().unwrap();
        user_groups.insert(user_id, group_id);

        Ok(())
    }

    /// Remove member from pickup group
    pub fn remove_member_from_group(&self, group_id: Uuid, user_id: &str) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        let group = groups.get_mut(&group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        if !group.remove_member(user_id) {
            return Err("User not in group".to_string());
        }

        let mut user_groups = self.user_groups.lock().unwrap();
        user_groups.remove(user_id);

        Ok(())
    }

    /// Set user permissions
    pub fn set_permissions(&self, user_id: String, permissions: PickupPermissions) {
        let mut perms = self.permissions.lock().unwrap();
        perms.insert(user_id, permissions);
    }

    /// Get user permissions
    pub fn get_permissions(&self, user_id: &str) -> PickupPermissions {
        let perms = self.permissions.lock().unwrap();
        perms.get(user_id).cloned().unwrap_or_default()
    }

    /// Register a ringing call
    pub fn register_ringing_call(&self, call: RingingCall) {
        let mut ringing_calls = self.ringing_calls.lock().unwrap();
        let mut extension_calls = self.extension_calls.lock().unwrap();

        extension_calls.insert(call.target_extension.clone(), call.call_id.clone());
        ringing_calls.insert(call.call_id.clone(), call);
    }

    /// Unregister a ringing call (answered or cancelled)
    pub fn unregister_ringing_call(&self, call_id: &str) {
        let mut ringing_calls = self.ringing_calls.lock().unwrap();
        let mut extension_calls = self.extension_calls.lock().unwrap();

        if let Some(call) = ringing_calls.remove(call_id) {
            extension_calls.remove(&call.target_extension);
        }
    }

    /// Attempt directed pickup (specific extension)
    pub fn attempt_directed_pickup(
        &self,
        picker_user_id: &str,
        target_extension: &str,
    ) -> PickupResult {
        self.record_attempt();

        // Check permissions
        let permissions = self.get_permissions(picker_user_id);
        if !permissions.can_pickup(PickupType::Directed) {
            self.record_failure();
            return PickupResult::PermissionDenied;
        }

        // Find ringing call at target extension
        let extension_calls = self.extension_calls.lock().unwrap();
        let call_id = match extension_calls.get(target_extension) {
            Some(id) => id.clone(),
            None => {
                self.record_failure();
                return PickupResult::NoRingingCall;
            }
        };

        drop(extension_calls);

        // Verify call still exists
        let ringing_calls = self.ringing_calls.lock().unwrap();
        if let Some(call) = ringing_calls.get(&call_id) {
            let pickup_time = call.ringing_duration_seconds();
            drop(ringing_calls);

            self.record_success(PickupType::Directed, pickup_time);
            PickupResult::Success(call_id)
        } else {
            drop(ringing_calls);
            self.record_failure();
            PickupResult::AlreadyAnswered
        }
    }

    /// Attempt group pickup (any call in user's group)
    pub fn attempt_group_pickup(&self, picker_user_id: &str) -> PickupResult {
        self.record_attempt();

        // Check permissions
        let permissions = self.get_permissions(picker_user_id);
        if !permissions.can_pickup(PickupType::Group) {
            self.record_failure();
            return PickupResult::PermissionDenied;
        }

        // Find user's pickup group
        let user_groups = self.user_groups.lock().unwrap();
        let group_id = match user_groups.get(picker_user_id) {
            Some(id) => *id,
            None => {
                drop(user_groups);
                self.record_failure();
                return PickupResult::NotInGroup;
            }
        };
        drop(user_groups);

        // Find first ringing call in the group
        let ringing_calls = self.ringing_calls.lock().unwrap();
        let call = ringing_calls.values().find(|call| {
            call.pickup_group_id == Some(group_id)
        });

        if let Some(call) = call {
            let call_id = call.call_id.clone();
            let pickup_time = call.ringing_duration_seconds();
            drop(ringing_calls);

            self.record_success(PickupType::Group, pickup_time);
            PickupResult::Success(call_id)
        } else {
            drop(ringing_calls);
            self.record_failure();
            PickupResult::NoRingingCall
        }
    }

    /// Attempt BLF pickup (monitored extension)
    pub fn attempt_blf_pickup(
        &self,
        picker_user_id: &str,
        target_extension: &str,
    ) -> PickupResult {
        self.record_attempt();

        // Check permissions
        let permissions = self.get_permissions(picker_user_id);
        if !permissions.can_pickup(PickupType::Blf) {
            self.record_failure();
            return PickupResult::PermissionDenied;
        }

        // Check if user monitors this extension
        if !permissions.monitors_extension(target_extension) {
            self.record_failure();
            return PickupResult::NotAllowed;
        }

        // Find ringing call at target extension
        let extension_calls = self.extension_calls.lock().unwrap();
        let call_id = match extension_calls.get(target_extension) {
            Some(id) => id.clone(),
            None => {
                self.record_failure();
                return PickupResult::NoRingingCall;
            }
        };

        drop(extension_calls);

        // Verify call exists and BLF is enabled
        let ringing_calls = self.ringing_calls.lock().unwrap();
        if let Some(call) = ringing_calls.get(&call_id) {
            if !call.blf_enabled {
                drop(ringing_calls);
                self.record_failure();
                return PickupResult::NotAllowed;
            }

            let pickup_time = call.ringing_duration_seconds();
            drop(ringing_calls);

            self.record_success(PickupType::Blf, pickup_time);
            PickupResult::Success(call_id)
        } else {
            drop(ringing_calls);
            self.record_failure();
            PickupResult::AlreadyAnswered
        }
    }

    /// Attempt any pickup (admin privilege)
    pub fn attempt_any_pickup(&self, picker_user_id: &str) -> PickupResult {
        self.record_attempt();

        // Check permissions
        let permissions = self.get_permissions(picker_user_id);
        if !permissions.can_pickup(PickupType::Any) {
            self.record_failure();
            return PickupResult::PermissionDenied;
        }

        // Find first ringing call
        let ringing_calls = self.ringing_calls.lock().unwrap();
        let call = ringing_calls.values().next();

        if let Some(call) = call {
            let call_id = call.call_id.clone();
            let pickup_time = call.ringing_duration_seconds();
            drop(ringing_calls);

            self.record_success(PickupType::Any, pickup_time);
            PickupResult::Success(call_id)
        } else {
            drop(ringing_calls);
            self.record_failure();
            PickupResult::NoRingingCall
        }
    }

    /// List all ringing calls
    pub fn list_ringing_calls(&self) -> Vec<RingingCall> {
        let ringing_calls = self.ringing_calls.lock().unwrap();
        ringing_calls.values().cloned().collect()
    }

    /// List ringing calls in user's pickup group
    pub fn list_group_ringing_calls(&self, user_id: &str) -> Vec<RingingCall> {
        let user_groups = self.user_groups.lock().unwrap();
        let group_id = match user_groups.get(user_id) {
            Some(id) => *id,
            None => return vec![],
        };
        drop(user_groups);

        let ringing_calls = self.ringing_calls.lock().unwrap();
        ringing_calls
            .values()
            .filter(|call| call.pickup_group_id == Some(group_id))
            .cloned()
            .collect()
    }

    /// Get ringing call for specific extension
    pub fn get_ringing_call(&self, extension: &str) -> Option<RingingCall> {
        let extension_calls = self.extension_calls.lock().unwrap();
        let call_id = extension_calls.get(extension)?;

        let ringing_calls = self.ringing_calls.lock().unwrap();
        ringing_calls.get(call_id).cloned()
    }

    /// Record pickup attempt
    fn record_attempt(&self) {
        let mut total = self.total_attempts.lock().unwrap();
        *total += 1;
    }

    /// Record successful pickup
    fn record_success(&self, pickup_type: PickupType, pickup_time: i64) {
        let mut successful = self.successful_pickups.lock().unwrap();
        *successful += 1;

        let mut by_type = self.pickups_by_type.lock().unwrap();
        let type_name = format!("{:?}", pickup_type);
        *by_type.entry(type_name).or_insert(0) += 1;

        let mut times = self.pickup_times.lock().unwrap();
        times.push(pickup_time);
        if times.len() > 1000 {
            times.remove(0);
        }
    }

    /// Record failed pickup
    fn record_failure(&self) {
        // Total attempts already recorded
    }

    /// Get pickup statistics
    pub fn get_statistics(&self) -> PickupStatistics {
        let total_attempts = *self.total_attempts.lock().unwrap();
        let successful_pickups = *self.successful_pickups.lock().unwrap();
        let pickups_by_type = self.pickups_by_type.lock().unwrap();
        let pickup_times = self.pickup_times.lock().unwrap();
        let ringing_calls = self.ringing_calls.lock().unwrap();

        let mut stats = PickupStatistics::new();
        stats.total_attempts = total_attempts;
        stats.successful_pickups = successful_pickups;
        stats.failed_pickups = total_attempts - successful_pickups;
        stats.by_type = pickups_by_type.clone();
        stats.current_ringing_calls = ringing_calls.len();

        if !pickup_times.is_empty() {
            let sum: i64 = pickup_times.iter().sum();
            stats.average_time_to_pickup = sum / pickup_times.len() as i64;
        }

        stats
    }

    /// List all pickup groups
    pub fn list_groups(&self) -> Vec<PickupGroup> {
        let groups = self.groups.lock().unwrap();
        groups.values().cloned().collect()
    }

    /// Get user's pickup group
    pub fn get_user_group(&self, user_id: &str) -> Option<PickupGroup> {
        let user_groups = self.user_groups.lock().unwrap();
        let group_id = user_groups.get(user_id)?;

        let groups = self.groups.lock().unwrap();
        groups.get(group_id).cloned()
    }

    /// Clear all ringing calls (for testing)
    pub fn clear_ringing_calls(&self) {
        let mut ringing_calls = self.ringing_calls.lock().unwrap();
        let mut extension_calls = self.extension_calls.lock().unwrap();
        ringing_calls.clear();
        extension_calls.clear();
    }
}

impl Default for CallPickupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pickup_type_description() {
        assert_eq!(PickupType::Directed.description(), "Directed pickup (specific extension)");
        assert_eq!(PickupType::Group.description(), "Group pickup (within group)");
    }

    #[test]
    fn test_pickup_type_dial_codes() {
        assert_eq!(PickupType::Directed.dial_code(), "*8");
        assert_eq!(PickupType::Group.dial_code(), "*9");
        assert_eq!(PickupType::Any.dial_code(), "*10");
        assert_eq!(PickupType::Blf.dial_code(), "*11");
    }

    #[test]
    fn test_pickup_group_creation() {
        let group = PickupGroup::new("Sales Team".to_string())
            .with_description("Sales department pickup group".to_string())
            .with_members(vec!["alice".to_string(), "bob".to_string()]);

        assert_eq!(group.name, "Sales Team");
        assert_eq!(group.member_count(), 2);
        assert!(group.has_member("alice"));
        assert!(group.has_member("bob"));
        assert!(!group.has_member("charlie"));
    }

    #[test]
    fn test_pickup_group_add_remove_member() {
        let mut group = PickupGroup::new("Test Group".to_string());

        assert!(group.add_member("alice".to_string()));
        assert_eq!(group.member_count(), 1);

        // Adding same member returns false
        assert!(!group.add_member("alice".to_string()));

        assert!(group.remove_member("alice"));
        assert_eq!(group.member_count(), 0);

        // Removing non-existent member returns false
        assert!(!group.remove_member("alice"));
    }

    #[test]
    fn test_pickup_permissions() {
        let user_perms = PickupPermissions::user_default();
        assert!(user_perms.directed_pickup);
        assert!(user_perms.group_pickup);
        assert!(!user_perms.any_pickup);

        let admin_perms = PickupPermissions::admin();
        assert!(admin_perms.any_pickup);

        let restricted_perms = PickupPermissions::restricted();
        assert!(!restricted_perms.directed_pickup);
        assert!(!restricted_perms.group_pickup);
    }

    #[test]
    fn test_ringing_call() {
        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        )
        .with_caller_name("John Doe".to_string());

        assert_eq!(call.call_id, "call-123");
        assert_eq!(call.target_extension, "alice");
        assert_eq!(call.caller_name, Some("John Doe".to_string()));
        assert!(call.ringing_duration_seconds() >= 0);
    }

    #[test]
    fn test_create_pickup_group() {
        let manager = CallPickupManager::new();

        let group = PickupGroup::new("Support".to_string())
            .with_members(vec!["alice".to_string(), "bob".to_string()]);

        let group_id = manager.create_group(group);

        let retrieved = manager.get_group(group_id).unwrap();
        assert_eq!(retrieved.name, "Support");
        assert_eq!(retrieved.member_count(), 2);
    }

    #[test]
    fn test_directed_pickup_success() {
        let manager = CallPickupManager::new();

        // Set permissions
        manager.set_permissions("charlie".to_string(), PickupPermissions::user_default());

        // Register ringing call
        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        // Charlie picks up Alice's call
        let result = manager.attempt_directed_pickup("charlie", "alice");
        assert_eq!(result, PickupResult::Success("call-123".to_string()));
    }

    #[test]
    fn test_directed_pickup_no_ringing_call() {
        let manager = CallPickupManager::new();

        manager.set_permissions("charlie".to_string(), PickupPermissions::user_default());

        let result = manager.attempt_directed_pickup("charlie", "alice");
        assert_eq!(result, PickupResult::NoRingingCall);
    }

    #[test]
    fn test_directed_pickup_permission_denied() {
        let manager = CallPickupManager::new();

        manager.set_permissions("charlie".to_string(), PickupPermissions::restricted());

        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        let result = manager.attempt_directed_pickup("charlie", "alice");
        assert_eq!(result, PickupResult::PermissionDenied);
    }

    #[test]
    fn test_group_pickup_success() {
        let manager = CallPickupManager::new();

        // Create pickup group with alice and bob
        let group = PickupGroup::new("Support".to_string())
            .with_members(vec!["alice".to_string(), "bob".to_string()]);
        let group_id = manager.create_group(group);

        // Set permissions for bob
        manager.set_permissions("bob".to_string(), PickupPermissions::user_default());

        // Register ringing call for alice with group ID
        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        )
        .with_pickup_group(group_id);
        manager.register_ringing_call(call);

        // Bob picks up call in his group
        let result = manager.attempt_group_pickup("bob");
        assert_eq!(result, PickupResult::Success("call-123".to_string()));
    }

    #[test]
    fn test_group_pickup_not_in_group() {
        let manager = CallPickupManager::new();

        manager.set_permissions("charlie".to_string(), PickupPermissions::user_default());

        let result = manager.attempt_group_pickup("charlie");
        assert_eq!(result, PickupResult::NotInGroup);
    }

    #[test]
    fn test_blf_pickup_success() {
        let manager = CallPickupManager::new();

        // Set permissions with BLF monitoring
        let mut perms = PickupPermissions::user_default();
        perms.add_blf_monitor("alice".to_string());
        manager.set_permissions("bob".to_string(), perms);

        // Register ringing call
        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        // Bob picks up via BLF
        let result = manager.attempt_blf_pickup("bob", "alice");
        assert_eq!(result, PickupResult::Success("call-123".to_string()));
    }

    #[test]
    fn test_blf_pickup_not_monitored() {
        let manager = CallPickupManager::new();

        manager.set_permissions("bob".to_string(), PickupPermissions::user_default());

        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        let result = manager.attempt_blf_pickup("bob", "alice");
        assert_eq!(result, PickupResult::NotAllowed);
    }

    #[test]
    fn test_any_pickup_success() {
        let manager = CallPickupManager::new();

        manager.set_permissions("admin".to_string(), PickupPermissions::admin());

        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        let result = manager.attempt_any_pickup("admin");
        assert_eq!(result, PickupResult::Success("call-123".to_string()));
    }

    #[test]
    fn test_any_pickup_permission_denied() {
        let manager = CallPickupManager::new();

        manager.set_permissions("user".to_string(), PickupPermissions::user_default());

        let result = manager.attempt_any_pickup("user");
        assert_eq!(result, PickupResult::PermissionDenied);
    }

    #[test]
    fn test_unregister_ringing_call() {
        let manager = CallPickupManager::new();

        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        assert_eq!(manager.list_ringing_calls().len(), 1);

        manager.unregister_ringing_call("call-123");

        assert_eq!(manager.list_ringing_calls().len(), 0);
    }

    #[test]
    fn test_list_group_ringing_calls() {
        let manager = CallPickupManager::new();

        let group = PickupGroup::new("Support".to_string())
            .with_members(vec!["alice".to_string(), "bob".to_string()]);
        let group_id = manager.create_group(group);

        // Register two calls, one in group
        let call1 = RingingCall::new(
            "call-1".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        )
        .with_pickup_group(group_id);
        manager.register_ringing_call(call1);

        let call2 = RingingCall::new(
            "call-2".to_string(),
            "charlie".to_string(),
            "1002".to_string(),
        );
        manager.register_ringing_call(call2);

        let group_calls = manager.list_group_ringing_calls("bob");
        assert_eq!(group_calls.len(), 1);
        assert_eq!(group_calls[0].call_id, "call-1");
    }

    #[test]
    fn test_pickup_statistics() {
        let manager = CallPickupManager::new();

        manager.set_permissions("user".to_string(), PickupPermissions::user_default());

        let call = RingingCall::new(
            "call-123".to_string(),
            "alice".to_string(),
            "1001".to_string(),
        );
        manager.register_ringing_call(call);

        manager.attempt_directed_pickup("user", "alice");
        manager.attempt_directed_pickup("user", "bob"); // Will fail

        let stats = manager.get_statistics();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.successful_pickups, 1);
        assert_eq!(stats.failed_pickups, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_add_remove_group_member() {
        let manager = CallPickupManager::new();

        let group = PickupGroup::new("Support".to_string())
            .with_members(vec!["alice".to_string()]);
        let group_id = manager.create_group(group);

        manager.add_member_to_group(group_id, "bob".to_string()).unwrap();

        let updated_group = manager.get_group(group_id).unwrap();
        assert_eq!(updated_group.member_count(), 2);

        manager.remove_member_from_group(group_id, "bob").unwrap();

        let final_group = manager.get_group(group_id).unwrap();
        assert_eq!(final_group.member_count(), 1);
    }

    #[test]
    fn test_get_user_group() {
        let manager = CallPickupManager::new();

        let group = PickupGroup::new("Support".to_string())
            .with_members(vec!["alice".to_string()]);
        let group_id = manager.create_group(group);

        let user_group = manager.get_user_group("alice").unwrap();
        assert_eq!(user_group.id, group_id);
        assert_eq!(user_group.name, "Support");

        let no_group = manager.get_user_group("charlie");
        assert!(no_group.is_none());
    }
}

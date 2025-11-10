//! User presence and status management system
//!
//! Provides comprehensive presence tracking, subscription management,
//! and real-time status notifications for SIP/VoIP systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// User presence state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceState {
    /// User is online and available
    Online,
    /// User is offline/unavailable
    Offline,
    /// User is away from device
    Away,
    /// User is busy/in a call
    Busy,
    /// User is in Do Not Disturb mode
    DoNotDisturb,
    /// User is on the phone
    OnThePhone,
    /// User is in a meeting
    InMeeting,
}

impl PresenceState {
    pub fn as_str(&self) -> &str {
        match self {
            PresenceState::Online => "online",
            PresenceState::Offline => "offline",
            PresenceState::Away => "away",
            PresenceState::Busy => "busy",
            PresenceState::DoNotDisturb => "dnd",
            PresenceState::OnThePhone => "on-the-phone",
            PresenceState::InMeeting => "in-meeting",
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, PresenceState::Online)
    }
}

/// Activity type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Activity {
    /// No specific activity
    None,
    /// Working
    Working,
    /// In a meeting
    Meeting,
    /// On lunch break
    Lunch,
    /// On vacation
    Vacation,
    /// Traveling
    Traveling,
    /// Custom activity
    Custom(String),
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub username: String,
    pub state: PresenceState,
    pub status_message: Option<String>,
    pub activity: Activity,
    pub last_seen: DateTime<Utc>,
    pub last_state_change: DateTime<Utc>,
    pub device_info: Option<String>,
    pub priority: i32,
}

impl UserPresence {
    pub fn new(username: String) -> Self {
        Self {
            username,
            state: PresenceState::Offline,
            status_message: None,
            activity: Activity::None,
            last_seen: Utc::now(),
            last_state_change: Utc::now(),
            device_info: None,
            priority: 0,
        }
    }

    pub fn set_online(&mut self) {
        self.state = PresenceState::Online;
        self.last_seen = Utc::now();
        self.last_state_change = Utc::now();
    }

    pub fn set_offline(&mut self) {
        self.state = PresenceState::Offline;
        self.last_state_change = Utc::now();
    }

    pub fn update_state(&mut self, state: PresenceState) {
        if self.state != state {
            self.state = state;
            self.last_state_change = Utc::now();
        }
        self.last_seen = Utc::now();
    }

    pub fn set_status_message(&mut self, message: Option<String>) {
        self.status_message = message;
        self.last_seen = Utc::now();
    }

    pub fn set_activity(&mut self, activity: Activity) {
        self.activity = activity;
        self.last_seen = Utc::now();
    }

    /// Check if user has been inactive for too long
    pub fn is_stale(&self, inactive_threshold_seconds: i64) -> bool {
        let now = Utc::now();
        (now - self.last_seen).num_seconds() > inactive_threshold_seconds
    }
}

/// Presence subscription
#[derive(Debug, Clone)]
pub struct PresenceSubscription {
    pub id: Uuid,
    pub subscriber: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub dialog_id: String,
}

impl PresenceSubscription {
    pub fn new(subscriber: String, target: String, expires_seconds: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            subscriber,
            target,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(expires_seconds as i64),
            dialog_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn refresh(&mut self, expires_seconds: u32) {
        self.expires_at = Utc::now() + chrono::Duration::seconds(expires_seconds as i64);
    }
}

/// Presence event notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub event_id: Uuid,
    pub username: String,
    pub presence: UserPresence,
    pub timestamp: DateTime<Utc>,
}

/// Presence manager
pub struct PresenceManager {
    presence_data: Arc<Mutex<HashMap<String, UserPresence>>>,
    subscriptions: Arc<Mutex<Vec<PresenceSubscription>>>,
    subscriber_map: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    event_callback: Option<Arc<dyn Fn(PresenceEvent) + Send + Sync>>,
    inactive_threshold_seconds: i64,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            presence_data: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            subscriber_map: Arc::new(Mutex::new(HashMap::new())),
            event_callback: None,
            inactive_threshold_seconds: 300, // 5 minutes
        }
    }

    /// Set event callback for presence changes
    pub fn set_event_callback<F>(&mut self, callback: F)
    where
        F: Fn(PresenceEvent) + Send + Sync + 'static,
    {
        self.event_callback = Some(Arc::new(callback));
    }

    /// Set inactive threshold in seconds
    pub fn set_inactive_threshold(&mut self, seconds: i64) {
        self.inactive_threshold_seconds = seconds;
    }

    /// Update user presence
    pub fn update_presence(
        &self,
        username: &str,
        state: PresenceState,
        status_message: Option<String>,
        activity: Option<Activity>,
    ) {
        let mut presence_data = self.presence_data.lock().unwrap();

        let presence = presence_data
            .entry(username.to_string())
            .or_insert_with(|| UserPresence::new(username.to_string()));

        let old_state = presence.state;
        presence.update_state(state);

        if let Some(msg) = status_message {
            presence.set_status_message(Some(msg));
        }

        if let Some(act) = activity {
            presence.set_activity(act);
        }

        // Notify subscribers if state changed
        if old_state != state {
            let event = PresenceEvent {
                event_id: Uuid::new_v4(),
                username: username.to_string(),
                presence: presence.clone(),
                timestamp: Utc::now(),
            };

            if let Some(ref callback) = self.event_callback {
                callback(event);
            }

            // Notify all subscribers
            self.notify_subscribers(username, presence);
        }
    }

    /// Set user online
    pub fn set_online(&self, username: &str) {
        self.update_presence(username, PresenceState::Online, None, None);
    }

    /// Set user offline
    pub fn set_offline(&self, username: &str) {
        self.update_presence(username, PresenceState::Offline, None, None);
    }

    /// Set user away
    pub fn set_away(&self, username: &str) {
        self.update_presence(username, PresenceState::Away, None, None);
    }

    /// Set user busy
    pub fn set_busy(&self, username: &str) {
        self.update_presence(username, PresenceState::Busy, None, None);
    }

    /// Get user presence
    pub fn get_presence(&self, username: &str) -> Option<UserPresence> {
        let presence_data = self.presence_data.lock().unwrap();
        presence_data.get(username).cloned()
    }

    /// Get all presence data
    pub fn get_all_presence(&self) -> Vec<UserPresence> {
        let presence_data = self.presence_data.lock().unwrap();
        presence_data.values().cloned().collect()
    }

    /// Get online users
    pub fn get_online_users(&self) -> Vec<UserPresence> {
        let presence_data = self.presence_data.lock().unwrap();
        presence_data
            .values()
            .filter(|p| p.state == PresenceState::Online)
            .cloned()
            .collect()
    }

    /// Subscribe to user presence
    pub fn subscribe(
        &self,
        subscriber: &str,
        target: &str,
        expires_seconds: u32,
    ) -> Uuid {
        let subscription = PresenceSubscription::new(
            subscriber.to_string(),
            target.to_string(),
            expires_seconds,
        );
        let sub_id = subscription.id;

        // Add subscription
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.push(subscription);

        // Update subscriber map
        let mut subscriber_map = self.subscriber_map.lock().unwrap();
        subscriber_map
            .entry(target.to_string())
            .or_insert_with(HashSet::new)
            .insert(subscriber.to_string());

        sub_id
    }

    /// Unsubscribe from user presence
    pub fn unsubscribe(&self, subscriber: &str, target: &str) {
        // Remove subscription
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.retain(|s| !(s.subscriber == subscriber && s.target == target));

        // Update subscriber map
        let mut subscriber_map = self.subscriber_map.lock().unwrap();
        if let Some(subscribers) = subscriber_map.get_mut(target) {
            subscribers.remove(subscriber);
            if subscribers.is_empty() {
                subscriber_map.remove(target);
            }
        }
    }

    /// Get subscriptions for a user (who is this user subscribed to)
    pub fn get_subscriptions(&self, subscriber: &str) -> Vec<PresenceSubscription> {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions
            .iter()
            .filter(|s| s.subscriber == subscriber)
            .cloned()
            .collect()
    }

    /// Get subscribers for a user (who is subscribed to this user)
    pub fn get_subscribers(&self, target: &str) -> Vec<String> {
        let subscriber_map = self.subscriber_map.lock().unwrap();
        subscriber_map
            .get(target)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Notify subscribers of presence change
    fn notify_subscribers(&self, username: &str, presence: &UserPresence) {
        let subscribers = self.get_subscribers(username);

        // In a real implementation, this would send SIP NOTIFY messages
        // For now, we just trigger the event callback
        for _subscriber in subscribers {
            // Send NOTIFY to subscriber with presence information
            // This would integrate with the SIP NOTIFY handler
        }
    }

    /// Clean up expired subscriptions
    pub fn cleanup_expired_subscriptions(&self) -> usize {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        let initial_count = subscriptions.len();

        subscriptions.retain(|s| !s.is_expired());

        let removed_count = initial_count - subscriptions.len();

        // Update subscriber map
        if removed_count > 0 {
            let mut subscriber_map = self.subscriber_map.lock().unwrap();
            for sub in subscriptions.iter() {
                subscriber_map
                    .entry(sub.target.clone())
                    .or_insert_with(HashSet::new)
                    .insert(sub.subscriber.clone());
            }
        }

        removed_count
    }

    /// Mark inactive users as away
    pub fn mark_inactive_users_away(&self) -> usize {
        let mut presence_data = self.presence_data.lock().unwrap();
        let mut marked_count = 0;

        for presence in presence_data.values_mut() {
            if presence.state == PresenceState::Online
                && presence.is_stale(self.inactive_threshold_seconds)
            {
                presence.update_state(PresenceState::Away);
                marked_count += 1;
            }
        }

        marked_count
    }

    /// Get presence summary statistics
    pub fn get_statistics(&self) -> PresenceStatistics {
        let presence_data = self.presence_data.lock().unwrap();
        let subscriptions = self.subscriptions.lock().unwrap();

        let mut stats = PresenceStatistics::default();
        stats.total_users = presence_data.len();
        stats.total_subscriptions = subscriptions.len();

        for presence in presence_data.values() {
            match presence.state {
                PresenceState::Online => stats.online_count += 1,
                PresenceState::Offline => stats.offline_count += 1,
                PresenceState::Away => stats.away_count += 1,
                PresenceState::Busy => stats.busy_count += 1,
                PresenceState::DoNotDisturb => stats.dnd_count += 1,
                PresenceState::OnThePhone => stats.on_phone_count += 1,
                PresenceState::InMeeting => stats.in_meeting_count += 1,
            }
        }

        stats
    }
}

/// Presence statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenceStatistics {
    pub total_users: usize,
    pub online_count: usize,
    pub offline_count: usize,
    pub away_count: usize,
    pub busy_count: usize,
    pub dnd_count: usize,
    pub on_phone_count: usize,
    pub in_meeting_count: usize,
    pub total_subscriptions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_state_available() {
        assert!(PresenceState::Online.is_available());
        assert!(!PresenceState::Offline.is_available());
        assert!(!PresenceState::Busy.is_available());
    }

    #[test]
    fn test_user_presence_creation() {
        let presence = UserPresence::new("alice@example.com".to_string());
        assert_eq!(presence.username, "alice@example.com");
        assert_eq!(presence.state, PresenceState::Offline);
    }

    #[test]
    fn test_user_presence_state_change() {
        let mut presence = UserPresence::new("bob@example.com".to_string());

        presence.set_online();
        assert_eq!(presence.state, PresenceState::Online);

        presence.set_offline();
        assert_eq!(presence.state, PresenceState::Offline);
    }

    #[test]
    fn test_presence_manager_update() {
        let manager = PresenceManager::new();

        manager.set_online("alice@example.com");

        let presence = manager.get_presence("alice@example.com");
        assert!(presence.is_some());
        assert_eq!(presence.unwrap().state, PresenceState::Online);
    }

    #[test]
    fn test_presence_subscription() {
        let manager = PresenceManager::new();

        let sub_id = manager.subscribe("alice@example.com", "bob@example.com", 3600);
        assert!(sub_id != Uuid::nil());

        let subscriptions = manager.get_subscriptions("alice@example.com");
        assert_eq!(subscriptions.len(), 1);

        let subscribers = manager.get_subscribers("bob@example.com");
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], "alice@example.com");
    }

    #[test]
    fn test_presence_unsubscribe() {
        let manager = PresenceManager::new();

        manager.subscribe("alice@example.com", "bob@example.com", 3600);
        manager.unsubscribe("alice@example.com", "bob@example.com");

        let subscriptions = manager.get_subscriptions("alice@example.com");
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn test_get_online_users() {
        let manager = PresenceManager::new();

        manager.set_online("alice@example.com");
        manager.set_online("bob@example.com");
        manager.set_offline("charlie@example.com");

        let online = manager.get_online_users();
        assert_eq!(online.len(), 2);
    }

    #[test]
    fn test_presence_statistics() {
        let manager = PresenceManager::new();

        manager.set_online("user1@example.com");
        manager.set_online("user2@example.com");
        manager.set_away("user3@example.com");
        manager.set_busy("user4@example.com");

        let stats = manager.get_statistics();
        assert_eq!(stats.total_users, 4);
        assert_eq!(stats.online_count, 2);
        assert_eq!(stats.away_count, 1);
        assert_eq!(stats.busy_count, 1);
    }

    #[test]
    fn test_subscription_expiry() {
        let sub = PresenceSubscription::new(
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            0, // Expires immediately
        );

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(sub.is_expired());
    }

    #[test]
    fn test_cleanup_expired_subscriptions() {
        let manager = PresenceManager::new();

        manager.subscribe("alice@example.com", "bob@example.com", 0);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let removed = manager.cleanup_expired_subscriptions();
        assert_eq!(removed, 1);
    }
}

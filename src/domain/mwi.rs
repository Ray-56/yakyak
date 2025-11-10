use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Message Waiting Indicator (MWI) for voicemail notifications
/// Implements RFC 3842 - Message Summary Event Package

/// MWI message account
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MwiAccount {
    /// Account URI (e.g., sip:1001@example.com)
    pub uri: String,
}

impl MwiAccount {
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    pub fn from_mailbox(mailbox: &str, domain: &str) -> Self {
        Self {
            uri: format!("sip:{}@{}", mailbox, domain),
        }
    }
}

/// Message summary for MWI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSummary {
    /// Account this summary is for
    pub account: MwiAccount,
    /// New voice messages count
    pub voice_new: u32,
    /// Old (read) voice messages count
    pub voice_old: u32,
    /// Urgent new voice messages
    pub voice_urgent_new: u32,
    /// Urgent old voice messages
    pub voice_urgent_old: u32,
    /// Optional message details
    pub messages: Vec<MessageDetail>,
}

impl MessageSummary {
    pub fn new(account: MwiAccount) -> Self {
        Self {
            account,
            voice_new: 0,
            voice_old: 0,
            voice_urgent_new: 0,
            voice_urgent_old: 0,
            messages: Vec::new(),
        }
    }

    pub fn with_counts(
        account: MwiAccount,
        new: u32,
        old: u32,
        urgent_new: u32,
        urgent_old: u32,
    ) -> Self {
        Self {
            account,
            voice_new: new,
            voice_old: old,
            voice_urgent_new: urgent_new,
            voice_urgent_old: urgent_old,
            messages: Vec::new(),
        }
    }

    pub fn has_new_messages(&self) -> bool {
        self.voice_new > 0 || self.voice_urgent_new > 0
    }

    pub fn total_new(&self) -> u32 {
        self.voice_new + self.voice_urgent_new
    }

    pub fn total_old(&self) -> u32 {
        self.voice_old + self.voice_urgent_old
    }

    pub fn total_messages(&self) -> u32 {
        self.total_new() + self.total_old()
    }

    /// Generate RFC 3842 message-summary body
    pub fn to_message_summary_body(&self) -> String {
        let mut body = String::new();

        // Messages-Waiting header
        let waiting = if self.has_new_messages() { "yes" } else { "no" };
        body.push_str(&format!("Messages-Waiting: {}\r\n", waiting));

        // Account
        body.push_str(&format!("Message-Account: {}\r\n", self.account.uri));

        // Voice-Message summary
        body.push_str(&format!(
            "Voice-Message: {}/{} ({}/{})\r\n",
            self.voice_new,
            self.voice_old,
            self.voice_urgent_new,
            self.voice_urgent_old
        ));

        body
    }
}

/// Individual message detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDetail {
    pub message_id: String,
    pub from: String,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u32,
    pub urgent: bool,
}

/// MWI subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwiSubscription {
    pub id: Uuid,
    /// Subscriber URI (who is subscribing)
    pub subscriber: String,
    /// Account being monitored
    pub account: MwiAccount,
    /// Contact URI for NOTIFY messages
    pub contact: String,
    /// Subscription expiry time
    pub expires_at: DateTime<Utc>,
    /// Dialog information
    pub call_id: String,
    pub from_tag: String,
    pub to_tag: String,
    /// Subscription state
    pub state: MwiSubscriptionState,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last NOTIFY sent
    pub last_notify_at: Option<DateTime<Utc>>,
}

impl MwiSubscription {
    pub fn new(
        subscriber: String,
        account: MwiAccount,
        contact: String,
        expires_seconds: u32,
        call_id: String,
        from_tag: String,
        to_tag: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subscriber,
            account,
            contact,
            expires_at: Utc::now() + Duration::seconds(expires_seconds as i64),
            call_id,
            from_tag,
            to_tag,
            state: MwiSubscriptionState::Active,
            created_at: Utc::now(),
            last_notify_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_active(&self) -> bool {
        self.state == MwiSubscriptionState::Active && !self.is_expired()
    }

    pub fn refresh(&mut self, expires_seconds: u32) {
        self.expires_at = Utc::now() + Duration::seconds(expires_seconds as i64);
        self.state = MwiSubscriptionState::Active;
    }

    pub fn terminate(&mut self) {
        self.state = MwiSubscriptionState::Terminated;
    }

    pub fn mark_notified(&mut self) {
        self.last_notify_at = Some(Utc::now());
    }

    pub fn time_until_expiry(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
}

/// MWI subscription state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MwiSubscriptionState {
    Active,
    Pending,
    Terminated,
}

/// MWI notification event
#[derive(Debug, Clone)]
pub struct MwiNotification {
    pub subscription_id: Uuid,
    pub summary: MessageSummary,
    pub state: MwiSubscriptionState,
}

/// MWI manager for handling subscriptions and notifications
pub struct MwiManager {
    subscriptions: Arc<Mutex<HashMap<Uuid, MwiSubscription>>>,
    /// Index: account URI -> subscription IDs
    account_index: Arc<Mutex<HashMap<String, Vec<Uuid>>>>,
    /// Current message summaries per account
    summaries: Arc<Mutex<HashMap<String, MessageSummary>>>,
    /// Notification callback
    notification_callback: Arc<Mutex<Option<Box<dyn Fn(MwiNotification) + Send + Sync>>>>,
}

impl MwiManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            account_index: Arc::new(Mutex::new(HashMap::new())),
            summaries: Arc::new(Mutex::new(HashMap::new())),
            notification_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set notification callback for sending SIP NOTIFY messages
    pub fn set_notification_callback<F>(&self, callback: F)
    where
        F: Fn(MwiNotification) + Send + Sync + 'static,
    {
        *self.notification_callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Subscribe to MWI for an account
    pub fn subscribe(
        &self,
        subscriber: String,
        account: MwiAccount,
        contact: String,
        expires_seconds: u32,
        call_id: String,
        from_tag: String,
        to_tag: String,
    ) -> Result<Uuid, String> {
        if expires_seconds == 0 {
            return Err("Cannot subscribe with 0 expiry".to_string());
        }

        let subscription = MwiSubscription::new(
            subscriber,
            account.clone(),
            contact,
            expires_seconds,
            call_id,
            from_tag,
            to_tag,
        );

        let sub_id = subscription.id;
        let account_uri = account.uri.clone();

        // Store subscription
        self.subscriptions.lock().unwrap().insert(sub_id, subscription);

        // Update account index
        self.account_index
            .lock()
            .unwrap()
            .entry(account_uri.clone())
            .or_insert_with(Vec::new)
            .push(sub_id);

        // Send initial NOTIFY with current state
        if let Some(summary) = self.summaries.lock().unwrap().get(&account_uri) {
            self.send_notification(sub_id, summary.clone());
        } else {
            // Send empty summary
            let summary = MessageSummary::new(account);
            self.send_notification(sub_id, summary);
        }

        Ok(sub_id)
    }

    /// Refresh an existing subscription
    pub fn refresh_subscription(&self, sub_id: &Uuid, expires_seconds: u32) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.lock().unwrap();

        if let Some(subscription) = subscriptions.get_mut(sub_id) {
            if expires_seconds == 0 {
                // Unsubscribe
                subscription.terminate();
                self.send_notification(*sub_id, MessageSummary::new(subscription.account.clone()));
            } else {
                subscription.refresh(expires_seconds);
            }
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    /// Unsubscribe from MWI
    pub fn unsubscribe(&self, sub_id: &Uuid) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.lock().unwrap();

        if let Some(mut subscription) = subscriptions.remove(sub_id) {
            subscription.terminate();

            // Remove from account index
            let account_uri = subscription.account.uri.clone();
            if let Some(subs) = self.account_index.lock().unwrap().get_mut(&account_uri) {
                subs.retain(|id| id != sub_id);
            }

            // Send final NOTIFY
            self.send_notification(*sub_id, MessageSummary::new(subscription.account));
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    /// Update message summary for an account and notify subscribers
    pub fn update_summary(&self, summary: MessageSummary) {
        let account_uri = summary.account.uri.clone();

        // Store summary
        self.summaries.lock().unwrap().insert(account_uri.clone(), summary.clone());

        // Notify all active subscribers
        if let Some(sub_ids) = self.account_index.lock().unwrap().get(&account_uri) {
            for &sub_id in sub_ids {
                self.send_notification(sub_id, summary.clone());
            }
        }
    }

    /// Get current message summary for an account
    pub fn get_summary(&self, account: &MwiAccount) -> Option<MessageSummary> {
        self.summaries.lock().unwrap().get(&account.uri).cloned()
    }

    /// Get subscription by ID
    pub fn get_subscription(&self, sub_id: &Uuid) -> Option<MwiSubscription> {
        self.subscriptions.lock().unwrap().get(sub_id).cloned()
    }

    /// List all active subscriptions for an account
    pub fn list_subscriptions(&self, account: &MwiAccount) -> Vec<MwiSubscription> {
        if let Some(sub_ids) = self.account_index.lock().unwrap().get(&account.uri) {
            let subscriptions = self.subscriptions.lock().unwrap();
            sub_ids
                .iter()
                .filter_map(|id| subscriptions.get(id))
                .filter(|s| s.is_active())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Cleanup expired subscriptions
    pub fn cleanup_expired(&self) -> usize {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        let mut account_index = self.account_index.lock().unwrap();

        let expired: Vec<Uuid> = subscriptions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| *id)
            .collect();

        for sub_id in &expired {
            if let Some(subscription) = subscriptions.remove(sub_id) {
                // Remove from account index
                if let Some(subs) = account_index.get_mut(&subscription.account.uri) {
                    subs.retain(|id| id != sub_id);
                }
            }
        }

        expired.len()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> MwiStatistics {
        let subscriptions = self.subscriptions.lock().unwrap();
        let summaries = self.summaries.lock().unwrap();

        let active_count = subscriptions.values().filter(|s| s.is_active()).count();
        let expired_count = subscriptions.values().filter(|s| s.is_expired()).count();

        let accounts_with_new = summaries
            .values()
            .filter(|s| s.has_new_messages())
            .count();

        let total_new_messages: u32 = summaries.values().map(|s| s.total_new()).sum();

        MwiStatistics {
            total_subscriptions: subscriptions.len(),
            active_subscriptions: active_count,
            expired_subscriptions: expired_count,
            monitored_accounts: summaries.len(),
            accounts_with_new_messages: accounts_with_new,
            total_new_messages,
        }
    }

    /// Send notification to subscriber
    fn send_notification(&self, sub_id: Uuid, summary: MessageSummary) {
        if let Some(subscription) = self.subscriptions.lock().unwrap().get_mut(&sub_id) {
            subscription.mark_notified();

            let notification = MwiNotification {
                subscription_id: sub_id,
                summary,
                state: subscription.state,
            };

            if let Some(callback) = self.notification_callback.lock().unwrap().as_ref() {
                callback(notification);
            }
        }
    }
}

impl Default for MwiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MWI statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwiStatistics {
    pub total_subscriptions: usize,
    pub active_subscriptions: usize,
    pub expired_subscriptions: usize,
    pub monitored_accounts: usize,
    pub accounts_with_new_messages: usize,
    pub total_new_messages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mwi_account_creation() {
        let account = MwiAccount::from_mailbox("1001", "example.com");
        assert_eq!(account.uri, "sip:1001@example.com");
    }

    #[test]
    fn test_message_summary_creation() {
        let account = MwiAccount::new("sip:1001@example.com".to_string());
        let summary = MessageSummary::with_counts(account, 3, 5, 1, 0);

        assert_eq!(summary.voice_new, 3);
        assert_eq!(summary.voice_old, 5);
        assert_eq!(summary.voice_urgent_new, 1);
        assert!(summary.has_new_messages());
        assert_eq!(summary.total_new(), 4);
        assert_eq!(summary.total_messages(), 9);
    }

    #[test]
    fn test_message_summary_body() {
        let account = MwiAccount::new("sip:1001@example.com".to_string());
        let summary = MessageSummary::with_counts(account, 2, 3, 0, 0);

        let body = summary.to_message_summary_body();
        assert!(body.contains("Messages-Waiting: yes"));
        assert!(body.contains("Message-Account: sip:1001@example.com"));
        assert!(body.contains("Voice-Message: 2/3 (0/0)"));
    }

    #[test]
    fn test_mwi_subscription() {
        let mut sub = MwiSubscription::new(
            "sip:user@example.com".to_string(),
            MwiAccount::new("sip:1001@example.com".to_string()),
            "sip:user@192.168.1.100".to_string(),
            3600,
            "call-123".to_string(),
            "from-tag".to_string(),
            "to-tag".to_string(),
        );

        assert!(sub.is_active());
        assert!(!sub.is_expired());

        sub.terminate();
        assert!(!sub.is_active());
    }

    #[test]
    fn test_mwi_manager_subscribe() {
        let manager = MwiManager::new();
        let account = MwiAccount::new("sip:1001@example.com".to_string());

        let result = manager.subscribe(
            "sip:user@example.com".to_string(),
            account.clone(),
            "sip:user@192.168.1.100".to_string(),
            3600,
            "call-123".to_string(),
            "from-tag".to_string(),
            "to-tag".to_string(),
        );

        assert!(result.is_ok());
        let sub_id = result.unwrap();

        let subscription = manager.get_subscription(&sub_id);
        assert!(subscription.is_some());
        assert_eq!(subscription.unwrap().subscriber, "sip:user@example.com");
    }

    #[test]
    fn test_mwi_manager_update_summary() {
        let manager = MwiManager::new();
        let account = MwiAccount::new("sip:1001@example.com".to_string());

        // Subscribe
        manager
            .subscribe(
                "sip:user@example.com".to_string(),
                account.clone(),
                "sip:user@192.168.1.100".to_string(),
                3600,
                "call-123".to_string(),
                "from-tag".to_string(),
                "to-tag".to_string(),
            )
            .unwrap();

        // Update summary
        let summary = MessageSummary::with_counts(account.clone(), 5, 10, 0, 0);
        manager.update_summary(summary);

        // Check summary
        let retrieved = manager.get_summary(&account);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().voice_new, 5);
    }

    #[test]
    fn test_mwi_manager_unsubscribe() {
        let manager = MwiManager::new();
        let account = MwiAccount::new("sip:1001@example.com".to_string());

        let sub_id = manager
            .subscribe(
                "sip:user@example.com".to_string(),
                account.clone(),
                "sip:user@192.168.1.100".to_string(),
                3600,
                "call-123".to_string(),
                "from-tag".to_string(),
                "to-tag".to_string(),
            )
            .unwrap();

        assert!(manager.get_subscription(&sub_id).is_some());

        manager.unsubscribe(&sub_id).unwrap();
        assert!(manager.get_subscription(&sub_id).is_none());
    }

    #[test]
    fn test_mwi_statistics() {
        let manager = MwiManager::new();
        let account1 = MwiAccount::new("sip:1001@example.com".to_string());
        let account2 = MwiAccount::new("sip:1002@example.com".to_string());

        // Create subscriptions
        manager
            .subscribe(
                "sip:user1@example.com".to_string(),
                account1.clone(),
                "sip:user1@192.168.1.100".to_string(),
                3600,
                "call-123".to_string(),
                "from-tag1".to_string(),
                "to-tag1".to_string(),
            )
            .unwrap();

        manager
            .subscribe(
                "sip:user2@example.com".to_string(),
                account2.clone(),
                "sip:user2@192.168.1.101".to_string(),
                3600,
                "call-456".to_string(),
                "from-tag2".to_string(),
                "to-tag2".to_string(),
            )
            .unwrap();

        // Add summaries
        manager.update_summary(MessageSummary::with_counts(account1, 3, 5, 0, 0));
        manager.update_summary(MessageSummary::with_counts(account2, 0, 2, 0, 0));

        let stats = manager.get_statistics();
        assert_eq!(stats.total_subscriptions, 2);
        assert_eq!(stats.active_subscriptions, 2);
        assert_eq!(stats.monitored_accounts, 2);
        assert_eq!(stats.accounts_with_new_messages, 1);
        assert_eq!(stats.total_new_messages, 3);
    }

    #[test]
    fn test_subscription_refresh() {
        let manager = MwiManager::new();
        let account = MwiAccount::new("sip:1001@example.com".to_string());

        let sub_id = manager
            .subscribe(
                "sip:user@example.com".to_string(),
                account,
                "sip:user@192.168.1.100".to_string(),
                60,
                "call-123".to_string(),
                "from-tag".to_string(),
                "to-tag".to_string(),
            )
            .unwrap();

        // Refresh with longer expiry
        assert!(manager.refresh_subscription(&sub_id, 3600).is_ok());

        let subscription = manager.get_subscription(&sub_id).unwrap();
        assert!(subscription.time_until_expiry() > 3500);
    }

    #[test]
    fn test_list_subscriptions() {
        let manager = MwiManager::new();
        let account = MwiAccount::new("sip:1001@example.com".to_string());

        // Add multiple subscriptions for same account
        for i in 0..3 {
            manager
                .subscribe(
                    format!("sip:user{}@example.com", i),
                    account.clone(),
                    format!("sip:user{}@192.168.1.10{}", i, i),
                    3600,
                    format!("call-{}", i),
                    format!("from-tag-{}", i),
                    format!("to-tag-{}", i),
                )
                .unwrap();
        }

        let subs = manager.list_subscriptions(&account);
        assert_eq!(subs.len(), 3);
    }
}

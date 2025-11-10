use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Message content type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageContentType {
    /// Plain text message
    TextPlain,
    /// HTML message
    TextHtml,
    /// JSON message
    ApplicationJson,
    /// Binary data
    ApplicationOctetStream,
    /// Custom content type
    Custom(String),
}

impl MessageContentType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "text/plain" => MessageContentType::TextPlain,
            "text/html" => MessageContentType::TextHtml,
            "application/json" => MessageContentType::ApplicationJson,
            "application/octet-stream" => MessageContentType::ApplicationOctetStream,
            _ => MessageContentType::Custom(s.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            MessageContentType::TextPlain => "text/plain".to_string(),
            MessageContentType::TextHtml => "text/html".to_string(),
            MessageContentType::ApplicationJson => "application/json".to_string(),
            MessageContentType::ApplicationOctetStream => "application/octet-stream".to_string(),
            MessageContentType::Custom(s) => s.clone(),
        }
    }
}

/// Message delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message is pending delivery
    Pending,
    /// Message was delivered successfully
    Delivered,
    /// Message delivery failed
    Failed,
    /// Message was read by recipient
    Read,
}

/// Instant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantMessage {
    pub id: Uuid,
    pub from: String,
    pub to: String,
    pub content_type: MessageContentType,
    pub content: Vec<u8>,
    pub status: MessageStatus,
    pub timestamp: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub group_id: Option<Uuid>,
}

impl InstantMessage {
    pub fn new(from: String, to: String, content: Vec<u8>, content_type: MessageContentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            content_type,
            content,
            status: MessageStatus::Pending,
            timestamp: Utc::now(),
            delivered_at: None,
            read_at: None,
            group_id: None,
        }
    }

    pub fn text(from: String, to: String, text: String) -> Self {
        Self::new(from, to, text.into_bytes(), MessageContentType::TextPlain)
    }

    pub fn for_group(mut self, group_id: Uuid) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn mark_delivered(&mut self) {
        self.status = MessageStatus::Delivered;
        self.delivered_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self) {
        self.status = MessageStatus::Failed;
    }

    pub fn mark_read(&mut self) {
        self.status = MessageStatus::Read;
        self.read_at = Some(Utc::now());
    }

    pub fn content_as_string(&self) -> Result<String, String> {
        String::from_utf8(self.content.clone())
            .map_err(|e| format!("Failed to decode content as UTF-8: {}", e))
    }

    pub fn is_group_message(&self) -> bool {
        self.group_id.is_some()
    }
}

/// Message group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageGroup {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MessageGroup {
    pub fn new(name: String, created_by: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            members: Vec::new(),
            created_by: created_by.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn add_member(&mut self, user: String) {
        if !self.members.contains(&user) {
            self.members.push(user);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_member(&mut self, user: &str) -> bool {
        if let Some(pos) = self.members.iter().position(|u| u == user) {
            self.members.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn is_member(&self, user: &str) -> bool {
        self.members.contains(&user.to_string())
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Offline message queue
#[derive(Debug)]
struct OfflineQueue {
    messages: VecDeque<InstantMessage>,
    max_size: usize,
}

impl OfflineQueue {
    fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_size,
        }
    }

    fn push(&mut self, message: InstantMessage) {
        if self.messages.len() >= self.max_size {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    fn drain(&mut self) -> Vec<InstantMessage> {
        self.messages.drain(..).collect()
    }

    fn count(&self) -> usize {
        self.messages.len()
    }
}

/// Message delivery callback
pub type MessageDeliveryCallback = Arc<dyn Fn(&InstantMessage) -> Result<(), String> + Send + Sync>;

/// Instant messaging manager
pub struct InstantMessagingManager {
    /// Message history
    message_history: Arc<Mutex<Vec<InstantMessage>>>,
    /// Offline message queues per user
    offline_queues: Arc<Mutex<HashMap<String, OfflineQueue>>>,
    /// Message groups
    groups: Arc<Mutex<HashMap<Uuid, MessageGroup>>>,
    /// Online users
    online_users: Arc<Mutex<Vec<String>>>,
    /// Message delivery callback
    delivery_callback: Arc<Mutex<Option<MessageDeliveryCallback>>>,
    /// Configuration
    max_offline_messages: usize,
    max_history_size: usize,
}

impl InstantMessagingManager {
    pub fn new() -> Self {
        Self {
            message_history: Arc::new(Mutex::new(Vec::new())),
            offline_queues: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
            online_users: Arc::new(Mutex::new(Vec::new())),
            delivery_callback: Arc::new(Mutex::new(None)),
            max_offline_messages: 100,
            max_history_size: 10000,
        }
    }

    pub fn with_limits(max_offline: usize, max_history: usize) -> Self {
        Self {
            message_history: Arc::new(Mutex::new(Vec::new())),
            offline_queues: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
            online_users: Arc::new(Mutex::new(Vec::new())),
            delivery_callback: Arc::new(Mutex::new(None)),
            max_offline_messages: max_offline,
            max_history_size: max_history,
        }
    }

    /// Set message delivery callback
    pub fn set_delivery_callback<F>(&self, callback: F)
    where
        F: Fn(&InstantMessage) -> Result<(), String> + Send + Sync + 'static,
    {
        *self.delivery_callback.lock().unwrap() = Some(Arc::new(callback));
    }

    /// Send a message
    pub fn send_message(&self, mut message: InstantMessage) -> Result<Uuid, String> {
        let message_id = message.id;

        // If it's a group message, send to all members
        if let Some(group_id) = message.group_id {
            let group = self.groups.lock().unwrap()
                .get(&group_id)
                .ok_or("Group not found")?
                .clone();

            for member in &group.members {
                if member != &message.from {
                    let mut member_msg = message.clone();
                    member_msg.id = Uuid::new_v4();
                    member_msg.to = member.clone();
                    self.route_message(member_msg)?;
                }
            }

            message.mark_delivered();
        } else {
            // Route single message
            self.route_message(message.clone())?;
        }

        // Store in history
        self.add_to_history(message);

        Ok(message_id)
    }

    /// Route message to recipient (online or offline)
    fn route_message(&self, mut message: InstantMessage) -> Result<(), String> {
        let is_online = self.is_user_online(&message.to);

        if is_online {
            // Try to deliver immediately
            if let Some(callback) = self.delivery_callback.lock().unwrap().as_ref() {
                match callback(&message) {
                    Ok(_) => {
                        message.mark_delivered();
                        Ok(())
                    }
                    Err(e) => {
                        message.mark_failed();
                        Err(format!("Delivery failed: {}", e))
                    }
                }
            } else {
                // No callback, queue as offline
                self.queue_offline_message(message);
                Ok(())
            }
        } else {
            // User is offline, queue the message
            self.queue_offline_message(message);
            Ok(())
        }
    }

    /// Queue message for offline delivery
    fn queue_offline_message(&self, message: InstantMessage) {
        let mut queues = self.offline_queues.lock().unwrap();
        let queue = queues
            .entry(message.to.clone())
            .or_insert_with(|| OfflineQueue::new(self.max_offline_messages));
        queue.push(message);
    }

    /// Mark user as online and deliver queued messages
    pub fn user_online(&self, user: String) -> Vec<InstantMessage> {
        // Add to online users
        let mut online_users = self.online_users.lock().unwrap();
        if !online_users.contains(&user) {
            online_users.push(user.clone());
        }
        drop(online_users);

        // Deliver queued messages
        let mut queues = self.offline_queues.lock().unwrap();
        if let Some(mut queue) = queues.remove(&user) {
            queue.drain()
        } else {
            Vec::new()
        }
    }

    /// Mark user as offline
    pub fn user_offline(&self, user: &str) {
        let mut online_users = self.online_users.lock().unwrap();
        online_users.retain(|u| u != user);
    }

    /// Check if user is online
    pub fn is_user_online(&self, user: &str) -> bool {
        self.online_users.lock().unwrap().contains(&user.to_string())
    }

    /// Create a message group
    pub fn create_group(&self, group: MessageGroup) -> Uuid {
        let group_id = group.id;
        self.groups.lock().unwrap().insert(group_id, group);
        group_id
    }

    /// Get a message group
    pub fn get_group(&self, group_id: &Uuid) -> Option<MessageGroup> {
        self.groups.lock().unwrap().get(group_id).cloned()
    }

    /// Add member to group
    pub fn add_group_member(&self, group_id: &Uuid, user: String) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        if let Some(group) = groups.get_mut(group_id) {
            group.add_member(user);
            Ok(())
        } else {
            Err("Group not found".to_string())
        }
    }

    /// Remove member from group
    pub fn remove_group_member(&self, group_id: &Uuid, user: &str) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        if let Some(group) = groups.get_mut(group_id) {
            if group.remove_member(user) {
                Ok(())
            } else {
                Err("User not in group".to_string())
            }
        } else {
            Err("Group not found".to_string())
        }
    }

    /// List user's groups
    pub fn list_user_groups(&self, user: &str) -> Vec<MessageGroup> {
        self.groups
            .lock()
            .unwrap()
            .values()
            .filter(|g| g.is_member(user))
            .cloned()
            .collect()
    }

    /// Add message to history
    fn add_to_history(&self, message: InstantMessage) {
        let mut history = self.message_history.lock().unwrap();

        // Limit history size
        if history.len() >= self.max_history_size {
            history.remove(0);
        }

        history.push(message);
    }

    /// Get message history for a conversation
    pub fn get_conversation_history(&self, user1: &str, user2: &str, limit: usize) -> Vec<InstantMessage> {
        let history = self.message_history.lock().unwrap();

        history
            .iter()
            .filter(|m| {
                (m.from == user1 && m.to == user2) || (m.from == user2 && m.to == user1)
            })
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get group message history
    pub fn get_group_history(&self, group_id: &Uuid, limit: usize) -> Vec<InstantMessage> {
        let history = self.message_history.lock().unwrap();

        history
            .iter()
            .filter(|m| m.group_id.as_ref() == Some(group_id))
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get offline message count for a user
    pub fn get_offline_count(&self, user: &str) -> usize {
        self.offline_queues
            .lock()
            .unwrap()
            .get(user)
            .map(|q| q.count())
            .unwrap_or(0)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> MessagingStatistics {
        let history = self.message_history.lock().unwrap();
        let queues = self.offline_queues.lock().unwrap();
        let groups = self.groups.lock().unwrap();
        let online_users = self.online_users.lock().unwrap();

        let total_offline: usize = queues.values().map(|q| q.count()).sum();

        MessagingStatistics {
            total_messages: history.len(),
            total_groups: groups.len(),
            online_users: online_users.len(),
            offline_message_count: total_offline,
        }
    }
}

impl Default for InstantMessagingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Messaging statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingStatistics {
    pub total_messages: usize,
    pub total_groups: usize,
    pub online_users: usize,
    pub offline_message_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_content_type() {
        let ct = MessageContentType::from_str("text/plain");
        assert_eq!(ct, MessageContentType::TextPlain);
        assert_eq!(ct.to_string(), "text/plain");
    }

    #[test]
    fn test_instant_message_creation() {
        let msg = InstantMessage::text(
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            "Hello!".to_string(),
        );

        assert_eq!(msg.from, "alice@example.com");
        assert_eq!(msg.to, "bob@example.com");
        assert_eq!(msg.content_as_string().unwrap(), "Hello!");
        assert_eq!(msg.status, MessageStatus::Pending);
    }

    #[test]
    fn test_message_status_transitions() {
        let mut msg = InstantMessage::text(
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            "Test".to_string(),
        );

        msg.mark_delivered();
        assert_eq!(msg.status, MessageStatus::Delivered);
        assert!(msg.delivered_at.is_some());

        msg.mark_read();
        assert_eq!(msg.status, MessageStatus::Read);
        assert!(msg.read_at.is_some());
    }

    #[test]
    fn test_message_group() {
        let mut group = MessageGroup::new("Team".to_string(), "admin@example.com".to_string())
            .with_description("Team chat".to_string());

        assert_eq!(group.name, "Team");
        assert_eq!(group.member_count(), 0);

        group.add_member("user1@example.com".to_string());
        group.add_member("user2@example.com".to_string());
        assert_eq!(group.member_count(), 2);

        assert!(group.is_member("user1@example.com"));
        assert!(!group.is_member("user3@example.com"));

        group.remove_member("user1@example.com");
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_send_message() {
        let manager = InstantMessagingManager::new();

        manager.user_online("bob@example.com".to_string());

        let msg = InstantMessage::text(
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            "Hello!".to_string(),
        );

        let result = manager.send_message(msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_offline_message_queue() {
        let manager = InstantMessagingManager::new();

        // Send message to offline user
        let msg = InstantMessage::text(
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            "Hello!".to_string(),
        );

        manager.send_message(msg).unwrap();

        // Check offline count
        assert_eq!(manager.get_offline_count("bob@example.com"), 1);

        // User comes online
        let queued = manager.user_online("bob@example.com".to_string());
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].content_as_string().unwrap(), "Hello!");
    }

    #[test]
    fn test_online_offline_status() {
        let manager = InstantMessagingManager::new();

        assert!(!manager.is_user_online("alice@example.com"));

        manager.user_online("alice@example.com".to_string());
        assert!(manager.is_user_online("alice@example.com"));

        manager.user_offline("alice@example.com");
        assert!(!manager.is_user_online("alice@example.com"));
    }

    #[test]
    fn test_group_messaging() {
        let manager = InstantMessagingManager::new();

        // Create group
        let mut group = MessageGroup::new("Team".to_string(), "admin@example.com".to_string());
        group.add_member("user1@example.com".to_string());
        group.add_member("user2@example.com".to_string());
        let group_id = manager.create_group(group);

        // Mark users online
        manager.user_online("user1@example.com".to_string());
        manager.user_online("user2@example.com".to_string());

        // Send group message
        let msg = InstantMessage::text(
            "admin@example.com".to_string(),
            "".to_string(),
            "Hello team!".to_string(),
        )
        .for_group(group_id);

        manager.send_message(msg).unwrap();

        // Check that messages were sent to members
        let stats = manager.get_statistics();
        assert!(stats.total_messages > 0);
    }

    #[test]
    fn test_conversation_history() {
        let manager = InstantMessagingManager::new();

        // Send messages
        for i in 0..5 {
            let msg = InstantMessage::text(
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
                format!("Message {}", i),
            );
            manager.send_message(msg).unwrap();
        }

        let history = manager.get_conversation_history("alice@example.com", "bob@example.com", 10);
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_group_management() {
        let manager = InstantMessagingManager::new();

        let group = MessageGroup::new("Test".to_string(), "admin@example.com".to_string());
        let group_id = manager.create_group(group);

        manager.add_group_member(&group_id, "user1@example.com".to_string()).unwrap();
        manager.add_group_member(&group_id, "user2@example.com".to_string()).unwrap();

        let group = manager.get_group(&group_id).unwrap();
        assert_eq!(group.member_count(), 2);

        let user_groups = manager.list_user_groups("user1@example.com");
        assert_eq!(user_groups.len(), 1);
    }

    #[test]
    fn test_messaging_statistics() {
        let manager = InstantMessagingManager::new();

        manager.user_online("user1@example.com".to_string());
        manager.user_online("user2@example.com".to_string());

        let msg = InstantMessage::text(
            "user1@example.com".to_string(),
            "user2@example.com".to_string(),
            "Test".to_string(),
        );
        manager.send_message(msg).unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.online_users, 2);
        assert!(stats.total_messages > 0);
    }
}

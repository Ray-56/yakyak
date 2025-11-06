/// Voicemail domain model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Voicemail message status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoicemailStatus {
    New,
    Read,
    Saved,
    Deleted,
}

/// Voicemail message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicemailMessage {
    pub id: Uuid,
    pub mailbox_id: String,
    pub caller: String,
    pub caller_name: Option<String>,
    pub duration_seconds: u32,
    pub audio_file_path: String,
    pub audio_format: String, // e.g., "wav", "mp3"
    pub status: VoicemailStatus,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub saved_at: Option<DateTime<Utc>>,
}

impl VoicemailMessage {
    pub fn new(
        mailbox_id: String,
        caller: String,
        caller_name: Option<String>,
        duration_seconds: u32,
        audio_file_path: String,
        audio_format: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            mailbox_id,
            caller,
            caller_name,
            duration_seconds,
            audio_file_path,
            audio_format,
            status: VoicemailStatus::New,
            created_at: Utc::now(),
            read_at: None,
            saved_at: None,
        }
    }

    /// Mark message as read
    pub fn mark_read(&mut self) {
        if self.status == VoicemailStatus::New {
            self.status = VoicemailStatus::Read;
            self.read_at = Some(Utc::now());
        }
    }

    /// Mark message as saved
    pub fn mark_saved(&mut self) {
        self.status = VoicemailStatus::Saved;
        self.saved_at = Some(Utc::now());
    }

    /// Mark message as deleted
    pub fn mark_deleted(&mut self) {
        self.status = VoicemailStatus::Deleted;
    }

    /// Check if message is new
    pub fn is_new(&self) -> bool {
        self.status == VoicemailStatus::New
    }
}

/// Voicemail mailbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicemailMailbox {
    pub mailbox_id: String,
    pub user_id: i32,
    pub pin: Option<String>,
    pub greeting_file: Option<String>,
    pub max_message_duration: u32, // seconds
    pub max_messages: u32,
    pub email_notification: bool,
    pub email_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VoicemailMailbox {
    pub fn new(mailbox_id: String, user_id: i32) -> Self {
        Self {
            mailbox_id,
            user_id,
            pin: None,
            greeting_file: None,
            max_message_duration: 180, // 3 minutes default
            max_messages: 100,
            email_notification: false,
            email_address: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Verify PIN
    pub fn verify_pin(&self, pin: &str) -> bool {
        match &self.pin {
            Some(mailbox_pin) => mailbox_pin == pin,
            None => true, // No PIN set, allow access
        }
    }
}

/// Voicemail repository trait
#[async_trait::async_trait]
pub trait VoicemailRepository: Send + Sync {
    /// Create a new voicemail message
    async fn create_message(&self, message: VoicemailMessage) -> Result<VoicemailMessage, String>;

    /// Get voicemail message by ID
    async fn get_message(&self, id: Uuid) -> Result<Option<VoicemailMessage>, String>;

    /// List messages for a mailbox
    async fn list_messages(&self, mailbox_id: &str, status: Option<VoicemailStatus>) -> Result<Vec<VoicemailMessage>, String>;

    /// Update message status
    async fn update_message_status(&self, id: Uuid, status: VoicemailStatus) -> Result<(), String>;

    /// Delete message (permanent)
    async fn delete_message(&self, id: Uuid) -> Result<(), String>;

    /// Count messages for a mailbox
    async fn count_messages(&self, mailbox_id: &str, status: Option<VoicemailStatus>) -> Result<u32, String>;

    /// Get mailbox configuration
    async fn get_mailbox(&self, mailbox_id: &str) -> Result<Option<VoicemailMailbox>, String>;

    /// Create or update mailbox
    async fn save_mailbox(&self, mailbox: VoicemailMailbox) -> Result<VoicemailMailbox, String>;
}

/// Voicemail filters for querying
#[derive(Debug, Clone)]
pub struct VoicemailFilters {
    pub mailbox_id: Option<String>,
    pub status: Option<VoicemailStatus>,
    pub caller: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl VoicemailFilters {
    pub fn new() -> Self {
        Self {
            mailbox_id: None,
            status: None,
            caller: None,
            created_after: None,
            created_before: None,
        }
    }

    pub fn mailbox(mut self, mailbox_id: String) -> Self {
        self.mailbox_id = Some(mailbox_id);
        self
    }

    pub fn status(mut self, status: VoicemailStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn caller(mut self, caller: String) -> Self {
        self.caller = Some(caller);
        self
    }
}

impl Default for VoicemailFilters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_voicemail_message() {
        let message = VoicemailMessage::new(
            "alice".to_string(),
            "sip:bob@example.com".to_string(),
            Some("Bob".to_string()),
            45,
            "/var/voicemail/alice/msg001.wav".to_string(),
            "wav".to_string(),
        );

        assert_eq!(message.mailbox_id, "alice");
        assert_eq!(message.caller, "sip:bob@example.com");
        assert_eq!(message.duration_seconds, 45);
        assert_eq!(message.status, VoicemailStatus::New);
        assert!(message.is_new());
    }

    #[test]
    fn test_mark_message_read() {
        let mut message = VoicemailMessage::new(
            "alice".to_string(),
            "sip:bob@example.com".to_string(),
            None,
            30,
            "/var/voicemail/alice/msg001.wav".to_string(),
            "wav".to_string(),
        );

        assert_eq!(message.status, VoicemailStatus::New);
        assert!(message.read_at.is_none());

        message.mark_read();

        assert_eq!(message.status, VoicemailStatus::Read);
        assert!(message.read_at.is_some());
        assert!(!message.is_new());
    }

    #[test]
    fn test_mark_message_saved() {
        let mut message = VoicemailMessage::new(
            "alice".to_string(),
            "sip:bob@example.com".to_string(),
            None,
            30,
            "/var/voicemail/alice/msg001.wav".to_string(),
            "wav".to_string(),
        );

        message.mark_saved();

        assert_eq!(message.status, VoicemailStatus::Saved);
        assert!(message.saved_at.is_some());
    }

    #[test]
    fn test_voicemail_mailbox() {
        let mailbox = VoicemailMailbox::new("alice".to_string(), 1);

        assert_eq!(mailbox.mailbox_id, "alice");
        assert_eq!(mailbox.user_id, 1);
        assert_eq!(mailbox.max_message_duration, 180);
        assert_eq!(mailbox.max_messages, 100);
        assert!(!mailbox.email_notification);
    }

    #[test]
    fn test_mailbox_pin_verification() {
        let mut mailbox = VoicemailMailbox::new("alice".to_string(), 1);

        // No PIN set - should allow access
        assert!(mailbox.verify_pin("1234"));

        // Set PIN
        mailbox.pin = Some("5678".to_string());

        // Correct PIN
        assert!(mailbox.verify_pin("5678"));

        // Wrong PIN
        assert!(!mailbox.verify_pin("1234"));
    }

    #[test]
    fn test_voicemail_filters() {
        let filters = VoicemailFilters::new()
            .mailbox("alice".to_string())
            .status(VoicemailStatus::New);

        assert_eq!(filters.mailbox_id, Some("alice".to_string()));
        assert_eq!(filters.status, Some(VoicemailStatus::New));
    }
}

/// Voicemail IVR (Interactive Voice Response) for dial-in access
use crate::domain::voicemail::{VoicemailMessage, VoicemailMailbox, VoicemailStatus};
use crate::domain::voicemail_service::{VoicemailPlayer, MwiState};
use std::collections::HashMap;
use uuid::Uuid;

/// Voicemail IVR state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoicemailIvrState {
    /// Initial state - prompting for mailbox/PIN
    Authenticating,
    /// PIN verification in progress
    VerifyingPin,
    /// Main menu
    MainMenu,
    /// Listening to messages
    PlayingMessage,
    /// Message management (delete, save, etc.)
    MessageOptions,
    /// Recording greeting
    RecordingGreeting,
    /// Finished/hung up
    Finished,
}

/// Voicemail IVR menu options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoicemailMenuOption {
    /// Play next message (1)
    PlayNext,
    /// Replay current message (2)
    Replay,
    /// Delete current message (3)
    Delete,
    /// Save current message (4)
    Save,
    /// Previous message (5)
    Previous,
    /// Skip to next message (6)
    Skip,
    /// Return to main menu (*)
    MainMenu,
    /// Exit voicemail (#)
    Exit,
    /// Record greeting (9)
    RecordGreeting,
}

impl VoicemailMenuOption {
    /// Get menu option from DTMF digit
    pub fn from_digit(digit: char) -> Option<Self> {
        match digit {
            '1' => Some(Self::PlayNext),
            '2' => Some(Self::Replay),
            '3' => Some(Self::Delete),
            '4' => Some(Self::Save),
            '5' => Some(Self::Previous),
            '6' => Some(Self::Skip),
            '*' => Some(Self::MainMenu),
            '#' => Some(Self::Exit),
            '9' => Some(Self::RecordGreeting),
            _ => None,
        }
    }

    /// Get digit for menu option
    pub fn to_digit(self) -> char {
        match self {
            Self::PlayNext => '1',
            Self::Replay => '2',
            Self::Delete => '3',
            Self::Save => '4',
            Self::Previous => '5',
            Self::Skip => '6',
            Self::MainMenu => '*',
            Self::Exit => '#',
            Self::RecordGreeting => '9',
        }
    }
}

/// Voicemail IVR session
pub struct VoicemailIvrSession {
    /// Session ID
    pub id: Uuid,
    /// Mailbox ID
    pub mailbox_id: Option<String>,
    /// Current state
    pub state: VoicemailIvrState,
    /// PIN buffer for authentication
    pin_buffer: String,
    /// PIN attempts remaining
    pin_attempts: u32,
    /// Current message index
    current_message_index: usize,
    /// List of messages
    messages: Vec<VoicemailMessage>,
    /// Session variables
    variables: HashMap<String, String>,
}

impl VoicemailIvrSession {
    /// Create new voicemail IVR session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            mailbox_id: None,
            state: VoicemailIvrState::Authenticating,
            pin_buffer: String::new(),
            pin_attempts: 3,
            current_message_index: 0,
            messages: Vec::new(),
            variables: HashMap::new(),
        }
    }

    /// Set mailbox ID (auto-detected from caller)
    pub fn set_mailbox(&mut self, mailbox_id: String) {
        self.mailbox_id = Some(mailbox_id);
    }

    /// Add PIN digit
    pub fn add_pin_digit(&mut self, digit: char) {
        if digit.is_ascii_digit() {
            self.pin_buffer.push(digit);
        }
    }

    /// Get entered PIN
    pub fn get_pin(&self) -> &str {
        &self.pin_buffer
    }

    /// Clear PIN buffer
    pub fn clear_pin(&mut self) {
        self.pin_buffer.clear();
    }

    /// Verify PIN against mailbox
    pub fn verify_pin(&mut self, mailbox: &VoicemailMailbox) -> bool {
        let verified = mailbox.verify_pin(&self.pin_buffer);
        if verified {
            self.state = VoicemailIvrState::MainMenu;
        } else {
            self.pin_attempts -= 1;
            self.clear_pin();
            if self.pin_attempts == 0 {
                self.state = VoicemailIvrState::Finished;
            }
        }
        verified
    }

    /// Load messages for current mailbox
    pub fn load_messages(&mut self, messages: Vec<VoicemailMessage>) {
        // Sort by created_at descending (newest first)
        let mut sorted = messages;
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        self.messages = sorted;
        self.current_message_index = 0;
    }

    /// Get current message
    pub fn current_message(&self) -> Option<&VoicemailMessage> {
        self.messages.get(self.current_message_index)
    }

    /// Get current message (mutable)
    pub fn current_message_mut(&mut self) -> Option<&mut VoicemailMessage> {
        self.messages.get_mut(self.current_message_index)
    }

    /// Move to next message
    pub fn next_message(&mut self) -> bool {
        if self.current_message_index + 1 < self.messages.len() {
            self.current_message_index += 1;
            true
        } else {
            false
        }
    }

    /// Move to previous message
    pub fn previous_message(&mut self) -> bool {
        if self.current_message_index > 0 {
            self.current_message_index -= 1;
            true
        } else {
            false
        }
    }

    /// Get number of new messages
    pub fn new_message_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| m.status == VoicemailStatus::New)
            .count()
    }

    /// Get number of saved messages
    pub fn saved_message_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| m.status == VoicemailStatus::Saved)
            .count()
    }

    /// Get total message count
    pub fn total_message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get current message position (1-based)
    pub fn current_position(&self) -> usize {
        self.current_message_index + 1
    }

    /// Check if has more messages
    pub fn has_more_messages(&self) -> bool {
        self.current_message_index + 1 < self.messages.len()
    }

    /// Mark current message as read
    pub fn mark_current_read(&mut self) {
        if let Some(msg) = self.current_message_mut() {
            msg.mark_read();
        }
    }

    /// Mark current message as saved
    pub fn mark_current_saved(&mut self) {
        if let Some(msg) = self.current_message_mut() {
            msg.mark_saved();
        }
    }

    /// Mark current message as deleted
    pub fn mark_current_deleted(&mut self) {
        if let Some(msg) = self.current_message_mut() {
            msg.mark_deleted();
        }
    }

    /// Remove current deleted message from list
    pub fn remove_current_message(&mut self) -> Option<VoicemailMessage> {
        if self.current_message_index < self.messages.len() {
            let removed = self.messages.remove(self.current_message_index);
            // Adjust index if we removed the last message
            if self.current_message_index >= self.messages.len() && self.current_message_index > 0 {
                self.current_message_index -= 1;
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Set session variable
    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// Get session variable
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Get remaining PIN attempts
    pub fn pin_attempts_remaining(&self) -> u32 {
        self.pin_attempts
    }

    /// Finish session
    pub fn finish(&mut self) {
        self.state = VoicemailIvrState::Finished;
    }

    /// Check if session is finished
    pub fn is_finished(&self) -> bool {
        self.state == VoicemailIvrState::Finished
    }
}

impl Default for VoicemailIvrSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Voicemail prompt types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoicemailPrompt {
    /// Welcome to voicemail
    Welcome,
    /// Enter your PIN
    EnterPin,
    /// Invalid PIN
    InvalidPin,
    /// You have X new messages
    NewMessageCount,
    /// You have X saved messages
    SavedMessageCount,
    /// No messages
    NoMessages,
    /// Main menu options
    MainMenu,
    /// Message from <caller> at <time>
    MessageHeader,
    /// Message deleted
    MessageDeleted,
    /// Message saved
    MessageSaved,
    /// No more messages
    NoMoreMessages,
    /// Recording greeting
    RecordGreeting,
    /// Greeting recorded
    GreetingRecorded,
    /// Goodbye
    Goodbye,
}

impl VoicemailPrompt {
    /// Get prompt audio file ID
    pub fn audio_id(&self) -> &'static str {
        match self {
            Self::Welcome => "vm_welcome",
            Self::EnterPin => "vm_enter_pin",
            Self::InvalidPin => "vm_invalid_pin",
            Self::NewMessageCount => "vm_new_messages",
            Self::SavedMessageCount => "vm_saved_messages",
            Self::NoMessages => "vm_no_messages",
            Self::MainMenu => "vm_main_menu",
            Self::MessageHeader => "vm_message_header",
            Self::MessageDeleted => "vm_deleted",
            Self::MessageSaved => "vm_saved",
            Self::NoMoreMessages => "vm_no_more",
            Self::RecordGreeting => "vm_record_greeting",
            Self::GreetingRecorded => "vm_greeting_saved",
            Self::Goodbye => "vm_goodbye",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_option_from_digit() {
        assert_eq!(
            VoicemailMenuOption::from_digit('1'),
            Some(VoicemailMenuOption::PlayNext)
        );
        assert_eq!(
            VoicemailMenuOption::from_digit('3'),
            Some(VoicemailMenuOption::Delete)
        );
        assert_eq!(
            VoicemailMenuOption::from_digit('#'),
            Some(VoicemailMenuOption::Exit)
        );
        assert_eq!(VoicemailMenuOption::from_digit('0'), None);
    }

    #[test]
    fn test_menu_option_to_digit() {
        assert_eq!(VoicemailMenuOption::PlayNext.to_digit(), '1');
        assert_eq!(VoicemailMenuOption::Delete.to_digit(), '3');
        assert_eq!(VoicemailMenuOption::Exit.to_digit(), '#');
    }

    #[test]
    fn test_ivr_session_creation() {
        let session = VoicemailIvrSession::new();
        assert_eq!(session.state, VoicemailIvrState::Authenticating);
        assert_eq!(session.pin_attempts_remaining(), 3);
        assert_eq!(session.total_message_count(), 0);
    }

    #[test]
    fn test_pin_entry() {
        let mut session = VoicemailIvrSession::new();
        session.add_pin_digit('1');
        session.add_pin_digit('2');
        session.add_pin_digit('3');
        session.add_pin_digit('4');

        assert_eq!(session.get_pin(), "1234");

        session.clear_pin();
        assert_eq!(session.get_pin(), "");
    }

    #[test]
    fn test_pin_verification() {
        let mut session = VoicemailIvrSession::new();
        let mut mailbox = VoicemailMailbox::new("alice".to_string(), 1);
        mailbox.pin = Some("1234".to_string());

        // Wrong PIN
        session.add_pin_digit('5');
        session.add_pin_digit('6');
        session.add_pin_digit('7');
        session.add_pin_digit('8');

        assert!(!session.verify_pin(&mailbox));
        assert_eq!(session.pin_attempts_remaining(), 2);
        assert_eq!(session.state, VoicemailIvrState::Authenticating);

        // Correct PIN
        session.add_pin_digit('1');
        session.add_pin_digit('2');
        session.add_pin_digit('3');
        session.add_pin_digit('4');

        assert!(session.verify_pin(&mailbox));
        assert_eq!(session.state, VoicemailIvrState::MainMenu);
    }

    #[test]
    fn test_message_navigation() {
        let mut session = VoicemailIvrSession::new();

        let messages = vec![
            VoicemailMessage::new(
                "alice".to_string(),
                "bob".to_string(),
                Some("Bob".to_string()),
                30,
                "msg1.wav".to_string(),
                "wav".to_string(),
            ),
            VoicemailMessage::new(
                "alice".to_string(),
                "charlie".to_string(),
                Some("Charlie".to_string()),
                45,
                "msg2.wav".to_string(),
                "wav".to_string(),
            ),
        ];

        session.load_messages(messages);

        assert_eq!(session.total_message_count(), 2);
        assert_eq!(session.current_position(), 1);

        // Move to next
        assert!(session.next_message());
        assert_eq!(session.current_position(), 2);

        // Cannot move past last
        assert!(!session.next_message());

        // Move back
        assert!(session.previous_message());
        assert_eq!(session.current_position(), 1);
    }

    #[test]
    fn test_message_status_updates() {
        let mut session = VoicemailIvrSession::new();

        let mut message = VoicemailMessage::new(
            "alice".to_string(),
            "bob".to_string(),
            None,
            30,
            "msg1.wav".to_string(),
            "wav".to_string(),
        );

        session.load_messages(vec![message.clone()]);

        assert_eq!(session.current_message().unwrap().status, VoicemailStatus::New);

        session.mark_current_read();
        assert_eq!(session.current_message().unwrap().status, VoicemailStatus::Read);

        session.mark_current_saved();
        assert_eq!(session.current_message().unwrap().status, VoicemailStatus::Saved);

        session.mark_current_deleted();
        assert_eq!(session.current_message().unwrap().status, VoicemailStatus::Deleted);
    }

    #[test]
    fn test_remove_message() {
        let mut session = VoicemailIvrSession::new();

        let messages = vec![
            VoicemailMessage::new(
                "alice".to_string(),
                "bob".to_string(),
                None,
                30,
                "msg1.wav".to_string(),
                "wav".to_string(),
            ),
            VoicemailMessage::new(
                "alice".to_string(),
                "charlie".to_string(),
                None,
                45,
                "msg2.wav".to_string(),
                "wav".to_string(),
            ),
        ];

        session.load_messages(messages);
        assert_eq!(session.total_message_count(), 2);

        session.remove_current_message();
        assert_eq!(session.total_message_count(), 1);
    }

    #[test]
    fn test_message_counts() {
        let mut session = VoicemailIvrSession::new();

        let mut msg1 = VoicemailMessage::new(
            "alice".to_string(),
            "bob".to_string(),
            None,
            30,
            "msg1.wav".to_string(),
            "wav".to_string(),
        );

        let mut msg2 = VoicemailMessage::new(
            "alice".to_string(),
            "charlie".to_string(),
            None,
            45,
            "msg2.wav".to_string(),
            "wav".to_string(),
        );
        msg2.mark_saved();

        session.load_messages(vec![msg1, msg2]);

        assert_eq!(session.new_message_count(), 1);
        assert_eq!(session.saved_message_count(), 1);
        assert_eq!(session.total_message_count(), 2);
    }

    #[test]
    fn test_prompt_audio_ids() {
        assert_eq!(VoicemailPrompt::Welcome.audio_id(), "vm_welcome");
        assert_eq!(VoicemailPrompt::EnterPin.audio_id(), "vm_enter_pin");
        assert_eq!(VoicemailPrompt::MainMenu.audio_id(), "vm_main_menu");
    }
}

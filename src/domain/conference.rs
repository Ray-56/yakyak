/// Conference domain model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Conference participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_uri: String,
    pub display_name: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub is_muted: bool,
    pub is_moderator: bool,
    pub audio_codec: Option<String>,
}

impl Participant {
    pub fn new(user_uri: String, display_name: Option<String>, is_moderator: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_uri,
            display_name,
            joined_at: Utc::now(),
            is_muted: false,
            is_moderator,
            audio_codec: None,
        }
    }

    /// Mute participant
    pub fn mute(&mut self) {
        self.is_muted = true;
    }

    /// Unmute participant
    pub fn unmute(&mut self) {
        self.is_muted = false;
    }

    /// Toggle mute status
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }
}

/// Conference room state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConferenceState {
    Waiting,   // Waiting for moderator
    Active,    // Conference in progress
    Locked,    // No new participants allowed
    Ended,     // Conference ended
}

/// Conference room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceRoom {
    pub id: Uuid,
    pub name: String,
    pub pin: Option<String>,
    pub max_participants: usize,
    pub state: ConferenceState,
    pub participants: HashMap<Uuid, Participant>,
    pub moderator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub recording_enabled: bool,
    pub recording_path: Option<String>,
}

impl ConferenceRoom {
    pub fn new(name: String, pin: Option<String>, max_participants: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            pin,
            max_participants,
            state: ConferenceState::Waiting,
            participants: HashMap::new(),
            moderator_id: None,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            recording_enabled: false,
            recording_path: None,
        }
    }

    /// Verify PIN
    pub fn verify_pin(&self, pin: &str) -> bool {
        match &self.pin {
            Some(room_pin) => room_pin == pin,
            None => true, // No PIN required
        }
    }

    /// Add participant to conference
    pub fn add_participant(&mut self, participant: Participant) -> Result<(), String> {
        if self.state == ConferenceState::Locked {
            return Err("Conference is locked".to_string());
        }

        if self.state == ConferenceState::Ended {
            return Err("Conference has ended".to_string());
        }

        if self.participants.len() >= self.max_participants {
            return Err("Conference is full".to_string());
        }

        let participant_id = participant.id;
        let is_moderator = participant.is_moderator;

        self.participants.insert(participant_id, participant);

        // If this is a moderator and conference is waiting, start it
        if is_moderator && self.state == ConferenceState::Waiting {
            self.moderator_id = Some(participant_id);
            self.state = ConferenceState::Active;
            self.started_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Remove participant from conference
    pub fn remove_participant(&mut self, participant_id: Uuid) -> Result<(), String> {
        self.participants.remove(&participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;

        // If moderator left, end conference or assign new moderator
        if Some(participant_id) == self.moderator_id {
            // Try to assign another moderator
            self.moderator_id = self.participants
                .iter()
                .find(|(_, p)| p.is_moderator)
                .map(|(id, _)| *id);

            // If no moderator left, end conference
            if self.moderator_id.is_none() && !self.participants.is_empty() {
                self.state = ConferenceState::Waiting;
            }
        }

        // If all participants left, end conference
        if self.participants.is_empty() {
            self.end();
        }

        Ok(())
    }

    /// Get participant
    pub fn get_participant(&self, participant_id: Uuid) -> Option<&Participant> {
        self.participants.get(&participant_id)
    }

    /// Get participant (mutable)
    pub fn get_participant_mut(&mut self, participant_id: Uuid) -> Option<&mut Participant> {
        self.participants.get_mut(&participant_id)
    }

    /// Mute participant
    pub fn mute_participant(&mut self, participant_id: Uuid) -> Result<(), String> {
        let participant = self.get_participant_mut(participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;
        participant.mute();
        Ok(())
    }

    /// Unmute participant
    pub fn unmute_participant(&mut self, participant_id: Uuid) -> Result<(), String> {
        let participant = self.get_participant_mut(participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;
        participant.unmute();
        Ok(())
    }

    /// Mute all participants (except moderator)
    pub fn mute_all(&mut self) {
        for (id, participant) in self.participants.iter_mut() {
            if Some(*id) != self.moderator_id {
                participant.mute();
            }
        }
    }

    /// Unmute all participants
    pub fn unmute_all(&mut self) {
        for participant in self.participants.values_mut() {
            participant.unmute();
        }
    }

    /// Lock conference (no new participants)
    pub fn lock(&mut self) {
        if self.state == ConferenceState::Active {
            self.state = ConferenceState::Locked;
        }
    }

    /// Unlock conference
    pub fn unlock(&mut self) {
        if self.state == ConferenceState::Locked {
            self.state = ConferenceState::Active;
        }
    }

    /// Start recording
    pub fn start_recording(&mut self, path: String) {
        self.recording_enabled = true;
        self.recording_path = Some(path);
    }

    /// Stop recording
    pub fn stop_recording(&mut self) {
        self.recording_enabled = false;
    }

    /// End conference
    pub fn end(&mut self) {
        self.state = ConferenceState::Ended;
        self.ended_at = Some(Utc::now());
        self.stop_recording();
    }

    /// Get participant count
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Check if user is moderator
    pub fn is_moderator(&self, participant_id: Uuid) -> bool {
        Some(participant_id) == self.moderator_id
    }

    /// Get duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }
}

/// Conference repository trait
#[async_trait::async_trait]
pub trait ConferenceRepository: Send + Sync {
    /// Create conference room
    async fn create(&self, room: ConferenceRoom) -> Result<ConferenceRoom, String>;

    /// Get conference by ID
    async fn get(&self, id: Uuid) -> Result<Option<ConferenceRoom>, String>;

    /// List all active conferences
    async fn list_active(&self) -> Result<Vec<ConferenceRoom>, String>;

    /// Update conference
    async fn update(&self, room: ConferenceRoom) -> Result<(), String>;

    /// Delete conference
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_conference() {
        let room = ConferenceRoom::new(
            "Test Room".to_string(),
            Some("1234".to_string()),
            10,
        );

        assert_eq!(room.name, "Test Room");
        assert_eq!(room.max_participants, 10);
        assert_eq!(room.state, ConferenceState::Waiting);
        assert_eq!(room.participant_count(), 0);
    }

    #[test]
    fn test_pin_verification() {
        let room = ConferenceRoom::new(
            "Test Room".to_string(),
            Some("1234".to_string()),
            10,
        );

        assert!(room.verify_pin("1234"));
        assert!(!room.verify_pin("5678"));

        let room_no_pin = ConferenceRoom::new("Test".to_string(), None, 10);
        assert!(room_no_pin.verify_pin("anything"));
    }

    #[test]
    fn test_add_participant() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        let participant = Participant::new(
            "sip:alice@example.com".to_string(),
            Some("Alice".to_string()),
            false,
        );

        room.add_participant(participant).unwrap();
        assert_eq!(room.participant_count(), 1);
    }

    #[test]
    fn test_add_moderator_starts_conference() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);
        assert_eq!(room.state, ConferenceState::Waiting);

        let moderator = Participant::new(
            "sip:mod@example.com".to_string(),
            Some("Moderator".to_string()),
            true,
        );

        room.add_participant(moderator).unwrap();
        assert_eq!(room.state, ConferenceState::Active);
        assert!(room.moderator_id.is_some());
        assert!(room.started_at.is_some());
    }

    #[test]
    fn test_conference_full() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 2);

        let p1 = Participant::new("sip:user1@example.com".to_string(), None, false);
        let p2 = Participant::new("sip:user2@example.com".to_string(), None, false);
        let p3 = Participant::new("sip:user3@example.com".to_string(), None, false);

        room.add_participant(p1).unwrap();
        room.add_participant(p2).unwrap();

        let result = room.add_participant(p3);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("full"));
    }

    #[test]
    fn test_mute_operations() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        let p1 = Participant::new("sip:user1@example.com".to_string(), None, true);
        let p1_id = p1.id;

        room.add_participant(p1).unwrap();

        // Mute participant
        room.mute_participant(p1_id).unwrap();
        assert!(room.get_participant(p1_id).unwrap().is_muted);

        // Unmute participant
        room.unmute_participant(p1_id).unwrap();
        assert!(!room.get_participant(p1_id).unwrap().is_muted);
    }

    #[test]
    fn test_mute_all() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        let moderator = Participant::new("sip:mod@example.com".to_string(), None, true);
        let p1 = Participant::new("sip:user1@example.com".to_string(), None, false);
        let p2 = Participant::new("sip:user2@example.com".to_string(), None, false);

        let p1_id = p1.id;
        let p2_id = p2.id;

        room.add_participant(moderator).unwrap();
        room.add_participant(p1).unwrap();
        room.add_participant(p2).unwrap();

        room.mute_all();

        // Moderator should not be muted
        assert!(!room.get_participant(room.moderator_id.unwrap()).unwrap().is_muted);
        // Others should be muted
        assert!(room.get_participant(p1_id).unwrap().is_muted);
        assert!(room.get_participant(p2_id).unwrap().is_muted);
    }

    #[test]
    fn test_lock_unlock() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        let moderator = Participant::new("sip:mod@example.com".to_string(), None, true);
        room.add_participant(moderator).unwrap();

        assert_eq!(room.state, ConferenceState::Active);

        room.lock();
        assert_eq!(room.state, ConferenceState::Locked);

        // Cannot add when locked
        let p1 = Participant::new("sip:user1@example.com".to_string(), None, false);
        assert!(room.add_participant(p1).is_err());

        room.unlock();
        assert_eq!(room.state, ConferenceState::Active);
    }

    #[test]
    fn test_recording() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        assert!(!room.recording_enabled);

        room.start_recording("/path/to/recording.wav".to_string());
        assert!(room.recording_enabled);
        assert_eq!(room.recording_path, Some("/path/to/recording.wav".to_string()));

        room.stop_recording();
        assert!(!room.recording_enabled);
    }

    #[test]
    fn test_end_conference() {
        let mut room = ConferenceRoom::new("Test".to_string(), None, 10);

        let moderator = Participant::new("sip:mod@example.com".to_string(), None, true);
        room.add_participant(moderator).unwrap();

        room.start_recording("/path/recording.wav".to_string());
        room.end();

        assert_eq!(room.state, ConferenceState::Ended);
        assert!(room.ended_at.is_some());
        assert!(!room.recording_enabled);
    }

    #[test]
    fn test_participant_toggle_mute() {
        let mut participant = Participant::new(
            "sip:user@example.com".to_string(),
            None,
            false,
        );

        assert!(!participant.is_muted);

        participant.toggle_mute();
        assert!(participant.is_muted);

        participant.toggle_mute();
        assert!(!participant.is_muted);
    }
}

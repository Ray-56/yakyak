/// Conference Manager for runtime conference management
///
/// Manages active conferences, integrates with audio mixing, and coordinates
/// between SIP calls and conference participants.

use crate::domain::conference::{ConferenceRoom, Participant, ParticipantRole, ConferenceState};
use crate::infrastructure::media::{AudioMixer, AudioFrame};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Conference manager for coordinating multi-party calls
pub struct ConferenceManager {
    /// Active conference rooms (room_id -> room)
    rooms: Arc<RwLock<HashMap<Uuid, ConferenceRoom>>>,
    /// Audio mixers per conference (room_id -> mixer)
    mixers: Arc<RwLock<HashMap<Uuid, Arc<AudioMixer>>>>,
    /// Call ID to participant mapping (call_id -> (room_id, participant_id))
    call_participants: Arc<RwLock<HashMap<String, (Uuid, Uuid)>>>,
}

impl ConferenceManager {
    /// Create new conference manager
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            mixers: Arc::new(RwLock::new(HashMap::new())),
            call_participants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new conference room
    pub async fn create_room(
        &self,
        name: String,
        pin: Option<String>,
        max_participants: usize,
    ) -> Result<Uuid, String> {
        let room = ConferenceRoom::new(name, pin, max_participants);
        let room_id = room.id;

        // Create audio mixer for this conference (8kHz telephony sample rate, mono)
        let mixer = Arc::new(AudioMixer::new(8000, 1));

        let mut rooms = self.rooms.write().await;
        let mut mixers = self.mixers.write().await;

        rooms.insert(room_id, room);
        mixers.insert(room_id, mixer);

        info!("Created conference room: {}", room_id);
        Ok(room_id)
    }

    /// Get conference room
    pub async fn get_room(&self, room_id: Uuid) -> Result<ConferenceRoom, String> {
        let rooms = self.rooms.read().await;
        rooms
            .get(&room_id)
            .cloned()
            .ok_or_else(|| "Conference room not found".to_string())
    }

    /// Join conference
    pub async fn join_conference(
        &self,
        room_id: Uuid,
        call_id: String,
        name: String,
        role: ParticipantRole,
        pin: Option<String>,
    ) -> Result<Uuid, String> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&room_id)
            .ok_or_else(|| "Conference room not found".to_string())?;

        // Verify PIN if required
        if let Some(pin_value) = pin {
            if !room.verify_pin(&pin_value) {
                return Err("Invalid PIN".to_string());
            }
        } else if room.pin.is_some() {
            return Err("PIN required".to_string());
        }

        // Create participant
        let participant = Participant::new(name, call_id.clone(), role);
        let participant_id = participant.id;

        // Add to conference
        room.add_participant(participant)?;

        // Add to audio mixer
        let mixers = self.mixers.read().await;
        if let Some(mixer) = mixers.get(&room_id) {
            mixer.add_stream(participant_id).await;
        }

        // Track call -> participant mapping
        let mut call_participants = self.call_participants.write().await;
        call_participants.insert(call_id.clone(), (room_id, participant_id));

        info!(
            "Participant {} joined conference {} (call: {})",
            participant_id, room_id, call_id
        );
        Ok(participant_id)
    }

    /// Leave conference
    pub async fn leave_conference(&self, call_id: &str) -> Result<(), String> {
        // Get room and participant IDs
        let (room_id, participant_id) = {
            let mut call_participants = self.call_participants.write().await;
            call_participants
                .remove(call_id)
                .ok_or_else(|| "Call not in conference".to_string())?
        };

        // Remove from conference room
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.remove_participant(participant_id)?;

            // If room is ended, clean up
            if room.state == ConferenceState::Ended {
                self.cleanup_room(room_id).await?;
            }
        }

        // Remove from audio mixer
        let mixers = self.mixers.read().await;
        if let Some(mixer) = mixers.get(&room_id) {
            mixer.remove_stream(participant_id).await;
        }

        info!(
            "Participant {} left conference {} (call: {})",
            participant_id, room_id, call_id
        );
        Ok(())
    }

    /// Mute participant in conference
    pub async fn mute_participant(&self, call_id: &str) -> Result<(), String> {
        let call_participants = self.call_participants.read().await;
        let (room_id, participant_id) = call_participants
            .get(call_id)
            .ok_or_else(|| "Call not in conference".to_string())?;

        // Mute in conference room
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.mute_participant(*participant_id)?;
        }

        // Mute in audio mixer
        let mixers = self.mixers.read().await;
        if let Some(mixer) = mixers.get(room_id) {
            mixer.mute_participant(*participant_id).await?;
        }

        debug!("Muted participant {} in conference {}", participant_id, room_id);
        Ok(())
    }

    /// Unmute participant in conference
    pub async fn unmute_participant(&self, call_id: &str) -> Result<(), String> {
        let call_participants = self.call_participants.read().await;
        let (room_id, participant_id) = call_participants
            .get(call_id)
            .ok_or_else(|| "Call not in conference".to_string())?;

        // Unmute in conference room
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.unmute_participant(*participant_id)?;
        }

        // Unmute in audio mixer
        let mixers = self.mixers.read().await;
        if let Some(mixer) = mixers.get(room_id) {
            mixer.unmute_participant(*participant_id).await?;
        }

        debug!("Unmuted participant {} in conference {}", participant_id, room_id);
        Ok(())
    }

    /// Set participant volume
    pub async fn set_participant_volume(&self, call_id: &str, volume: f32) -> Result<(), String> {
        let call_participants = self.call_participants.read().await;
        let (room_id, participant_id) = call_participants
            .get(call_id)
            .ok_or_else(|| "Call not in conference".to_string())?;

        // Set in audio mixer
        let mixers = self.mixers.read().await;
        if let Some(mixer) = mixers.get(room_id) {
            mixer.set_participant_gain(*participant_id, volume).await?;
        }

        debug!("Set volume {} for participant {} in conference {}", volume, participant_id, room_id);
        Ok(())
    }

    /// Mix audio for a specific participant (excludes their own audio)
    pub async fn mix_audio_for_participant(
        &self,
        call_id: &str,
        frames: Vec<(String, AudioFrame)>, // (call_id, frame) pairs
    ) -> Result<AudioFrame, String> {
        let call_participants = self.call_participants.read().await;
        let (room_id, participant_id) = call_participants
            .get(call_id)
            .ok_or_else(|| "Call not in conference".to_string())?;

        // Convert call_id -> participant_id for all frames
        let participant_frames: Vec<(Uuid, AudioFrame)> = frames
            .into_iter()
            .filter_map(|(cid, frame)| {
                call_participants
                    .get(&cid)
                    .map(|(_, pid)| (*pid, frame))
            })
            .collect();

        // Mix audio
        let mixers = self.mixers.read().await;
        let mixer = mixers
            .get(room_id)
            .ok_or_else(|| "Conference mixer not found".to_string())?;

        let mixed = mixer.mix_frames(participant_frames, Some(*participant_id)).await;
        Ok(mixed)
    }

    /// End conference
    pub async fn end_conference(&self, room_id: Uuid) -> Result<(), String> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&room_id)
            .ok_or_else(|| "Conference room not found".to_string())?;

        room.end();
        info!("Ended conference: {}", room_id);

        // Cleanup will happen when last participant leaves
        Ok(())
    }

    /// Lock conference (no new participants)
    pub async fn lock_conference(&self, room_id: Uuid) -> Result<(), String> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&room_id)
            .ok_or_else(|| "Conference room not found".to_string())?;

        room.lock()?;
        info!("Locked conference: {}", room_id);
        Ok(())
    }

    /// Unlock conference
    pub async fn unlock_conference(&self, room_id: Uuid) -> Result<(), String> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&room_id)
            .ok_or_else(|| "Conference room not found".to_string())?;

        room.unlock()?;
        info!("Unlocked conference: {}", room_id);
        Ok(())
    }

    /// Get active conference count
    pub async fn active_conference_count(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms
            .values()
            .filter(|r| r.state == ConferenceState::Active || r.state == ConferenceState::Locked)
            .count()
    }

    /// Get participant count in conference
    pub async fn participant_count(&self, room_id: Uuid) -> Result<usize, String> {
        let rooms = self.rooms.read().await;
        let room = rooms
            .get(&room_id)
            .ok_or_else(|| "Conference room not found".to_string())?;
        Ok(room.participant_count())
    }

    /// Check if call is in a conference
    pub async fn is_in_conference(&self, call_id: &str) -> bool {
        let call_participants = self.call_participants.read().await;
        call_participants.contains_key(call_id)
    }

    /// Get conference room ID for a call
    pub async fn get_conference_for_call(&self, call_id: &str) -> Option<Uuid> {
        let call_participants = self.call_participants.read().await;
        call_participants.get(call_id).map(|(room_id, _)| *room_id)
    }

    /// Cleanup conference room (remove from tracking)
    async fn cleanup_room(&self, room_id: Uuid) -> Result<(), String> {
        let mut rooms = self.rooms.write().await;
        let mut mixers = self.mixers.write().await;

        rooms.remove(&room_id);
        mixers.remove(&room_id);

        info!("Cleaned up conference room: {}", room_id);
        Ok(())
    }

    /// List all active conferences
    pub async fn list_active_conferences(&self) -> Vec<ConferenceRoom> {
        let rooms = self.rooms.read().await;
        rooms
            .values()
            .filter(|r| r.state == ConferenceState::Active || r.state == ConferenceState::Locked)
            .cloned()
            .collect()
    }
}

impl Default for ConferenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_room() {
        let manager = ConferenceManager::new();
        let room_id = manager
            .create_room("Test Room".to_string(), None, 10)
            .await
            .unwrap();

        let room = manager.get_room(room_id).await.unwrap();
        assert_eq!(room.name, "Test Room");
        assert_eq!(room.max_participants, 10);
    }

    #[tokio::test]
    async fn test_join_conference() {
        let manager = ConferenceManager::new();
        let room_id = manager
            .create_room("Test Room".to_string(), None, 10)
            .await
            .unwrap();

        let participant_id = manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Moderator,
                None,
            )
            .await
            .unwrap();

        assert!(manager.is_in_conference("call-123").await);
        assert_eq!(manager.participant_count(room_id).await.unwrap(), 1);

        // Check participant was added to mixer
        let room = manager.get_room(room_id).await.unwrap();
        assert!(room.participants.contains_key(&participant_id));
    }

    #[tokio::test]
    async fn test_leave_conference() {
        let manager = ConferenceManager::new();
        let room_id = manager
            .create_room("Test Room".to_string(), None, 10)
            .await
            .unwrap();

        manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Attendee,
                None,
            )
            .await
            .unwrap();

        manager.leave_conference("call-123").await.unwrap();

        assert!(!manager.is_in_conference("call-123").await);
    }

    #[tokio::test]
    async fn test_mute_unmute() {
        let manager = ConferenceManager::new();
        let room_id = manager
            .create_room("Test Room".to_string(), None, 10)
            .await
            .unwrap();

        manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Attendee,
                None,
            )
            .await
            .unwrap();

        manager.mute_participant("call-123").await.unwrap();
        manager.unmute_participant("call-123").await.unwrap();
    }

    #[tokio::test]
    async fn test_pin_protection() {
        let manager = ConferenceManager::new();
        let room_id = manager
            .create_room("Test Room".to_string(), Some("1234".to_string()), 10)
            .await
            .unwrap();

        // Try without PIN - should fail
        let result = manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Attendee,
                None,
            )
            .await;
        assert!(result.is_err());

        // Try with wrong PIN - should fail
        let result = manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Attendee,
                Some("9999".to_string()),
            )
            .await;
        assert!(result.is_err());

        // Try with correct PIN - should succeed
        let result = manager
            .join_conference(
                room_id,
                "call-123".to_string(),
                "Alice".to_string(),
                ParticipantRole::Attendee,
                Some("1234".to_string()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_active_conference_count() {
        let manager = ConferenceManager::new();

        assert_eq!(manager.active_conference_count().await, 0);

        let room_id1 = manager
            .create_room("Room 1".to_string(), None, 10)
            .await
            .unwrap();

        manager
            .join_conference(
                room_id1,
                "call-1".to_string(),
                "Alice".to_string(),
                ParticipantRole::Moderator,
                None,
            )
            .await
            .unwrap();

        assert_eq!(manager.active_conference_count().await, 1);

        let room_id2 = manager
            .create_room("Room 2".to_string(), None, 10)
            .await
            .unwrap();

        manager
            .join_conference(
                room_id2,
                "call-2".to_string(),
                "Bob".to_string(),
                ParticipantRole::Moderator,
                None,
            )
            .await
            .unwrap();

        assert_eq!(manager.active_conference_count().await, 2);
    }
}

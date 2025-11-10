use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Conference recording format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingFormat {
    /// WAV PCM format
    Wav,
    /// MP3 compressed format
    Mp3,
    /// Opus compressed format
    Opus,
}

impl RecordingFormat {
    pub fn extension(&self) -> &str {
        match self {
            RecordingFormat::Wav => "wav",
            RecordingFormat::Mp3 => "mp3",
            RecordingFormat::Opus => "opus",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            RecordingFormat::Wav => "audio/wav",
            RecordingFormat::Mp3 => "audio/mpeg",
            RecordingFormat::Opus => "audio/opus",
        }
    }
}

/// Recording mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingMode {
    /// Record mixed audio of all participants
    Mixed,
    /// Record each participant separately (multi-track)
    Separate,
    /// Record both mixed and separate tracks
    Both,
}

/// Conference recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    /// Recording is active
    Recording,
    /// Recording is paused
    Paused,
    /// Recording has stopped
    Stopped,
    /// Recording failed
    Failed,
}

/// Conference recording metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceRecording {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub conference_name: String,
    pub state: RecordingState,
    pub format: RecordingFormat,
    pub mode: RecordingMode,
    pub file_path: PathBuf,
    pub file_size_bytes: u64,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub paused_duration_ms: u64,
    pub participants: Vec<RecordingParticipant>,
    pub metadata: RecordingMetadata,
}

impl ConferenceRecording {
    pub fn new(
        conference_id: Uuid,
        conference_name: String,
        format: RecordingFormat,
        mode: RecordingMode,
        file_path: PathBuf,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            conference_id,
            conference_name,
            state: RecordingState::Recording,
            format,
            mode,
            file_path,
            file_size_bytes: 0,
            duration_ms: 0,
            started_at: Utc::now(),
            stopped_at: None,
            paused_duration_ms: 0,
            participants: Vec::new(),
            metadata: RecordingMetadata::default(),
        }
    }

    pub fn add_participant(&mut self, participant: RecordingParticipant) {
        self.participants.push(participant);
    }

    pub fn pause(&mut self) {
        if self.state == RecordingState::Recording {
            self.state = RecordingState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == RecordingState::Paused {
            self.state = RecordingState::Recording;
        }
    }

    pub fn stop(&mut self) {
        self.state = RecordingState::Stopped;
        self.stopped_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self) {
        self.state = RecordingState::Failed;
        self.stopped_at = Some(Utc::now());
    }

    pub fn is_active(&self) -> bool {
        self.state == RecordingState::Recording
    }

    pub fn is_paused(&self) -> bool {
        self.state == RecordingState::Paused
    }

    pub fn is_stopped(&self) -> bool {
        self.state == RecordingState::Stopped || self.state == RecordingState::Failed
    }

    pub fn total_duration(&self) -> chrono::Duration {
        if let Some(stopped_at) = self.stopped_at {
            stopped_at - self.started_at
        } else {
            Utc::now() - self.started_at
        }
    }

    pub fn actual_recording_duration_ms(&self) -> u64 {
        let total_ms = self.total_duration().num_milliseconds() as u64;
        total_ms.saturating_sub(self.paused_duration_ms)
    }
}

/// Recording participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingParticipant {
    pub user_id: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub separate_track_path: Option<PathBuf>,
}

impl RecordingParticipant {
    pub fn new(user_id: String, display_name: String) -> Self {
        Self {
            user_id,
            display_name,
            joined_at: Utc::now(),
            left_at: None,
            separate_track_path: None,
        }
    }

    pub fn with_separate_track(mut self, track_path: PathBuf) -> Self {
        self.separate_track_path = Some(track_path);
        self
    }

    pub fn mark_left(&mut self) {
        self.left_at = Some(Utc::now());
    }

    pub fn duration(&self) -> chrono::Duration {
        if let Some(left_at) = self.left_at {
            left_at - self.joined_at
        } else {
            Utc::now() - self.joined_at
        }
    }
}

/// Recording metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub recorded_by: Option<String>,
    pub max_participants: usize,
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: Option<u32>,
}

impl RecordingMetadata {
    pub fn new() -> Self {
        Self {
            description: String::new(),
            tags: Vec::new(),
            recorded_by: None,
            max_participants: 0,
            sample_rate: 48000,
            channels: 2,
            bitrate: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

/// Conference recording configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub auto_record: bool,
    pub default_format: RecordingFormat,
    pub default_mode: RecordingMode,
    pub storage_path: PathBuf,
    pub max_file_size_mb: u64,
    pub max_duration_hours: u64,
    pub sample_rate: u32,
    pub channels: u8,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            auto_record: false,
            default_format: RecordingFormat::Wav,
            default_mode: RecordingMode::Mixed,
            storage_path: PathBuf::from("/var/recordings/conferences"),
            max_file_size_mb: 1024,
            max_duration_hours: 4,
            sample_rate: 48000,
            channels: 2,
        }
    }
}

/// Conference recording manager
pub struct ConferenceRecordingManager {
    active_recordings: Arc<Mutex<HashMap<Uuid, ConferenceRecording>>>,
    completed_recordings: Arc<Mutex<Vec<ConferenceRecording>>>,
    config: Arc<Mutex<RecordingConfig>>,
}

impl ConferenceRecordingManager {
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            active_recordings: Arc::new(Mutex::new(HashMap::new())),
            completed_recordings: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Start recording a conference
    pub fn start_recording(
        &self,
        conference_id: Uuid,
        conference_name: String,
        format: Option<RecordingFormat>,
        mode: Option<RecordingMode>,
    ) -> Result<Uuid, String> {
        let config = self.config.lock().unwrap();
        let format = format.unwrap_or(config.default_format);
        let mode = mode.unwrap_or(config.default_mode);

        // Generate file path
        let filename = format!(
            "conference_{}_{}.{}",
            conference_id,
            Utc::now().timestamp(),
            format.extension()
        );
        let file_path = config.storage_path.join(filename);

        drop(config); // Release lock

        let recording = ConferenceRecording::new(
            conference_id,
            conference_name,
            format,
            mode,
            file_path,
        );

        let recording_id = recording.id;
        self.active_recordings
            .lock()
            .unwrap()
            .insert(conference_id, recording);

        Ok(recording_id)
    }

    /// Stop recording a conference
    pub fn stop_recording(&self, conference_id: &Uuid) -> Result<ConferenceRecording, String> {
        let mut active = self.active_recordings.lock().unwrap();

        if let Some(mut recording) = active.remove(conference_id) {
            recording.stop();

            // Move to completed
            let recording_clone = recording.clone();
            self.completed_recordings
                .lock()
                .unwrap()
                .push(recording_clone);

            Ok(recording)
        } else {
            Err("No active recording for this conference".to_string())
        }
    }

    /// Pause recording
    pub fn pause_recording(&self, conference_id: &Uuid) -> Result<(), String> {
        let mut active = self.active_recordings.lock().unwrap();

        if let Some(recording) = active.get_mut(conference_id) {
            recording.pause();
            Ok(())
        } else {
            Err("No active recording for this conference".to_string())
        }
    }

    /// Resume recording
    pub fn resume_recording(&self, conference_id: &Uuid) -> Result<(), String> {
        let mut active = self.active_recordings.lock().unwrap();

        if let Some(recording) = active.get_mut(conference_id) {
            recording.resume();
            Ok(())
        } else {
            Err("No active recording for this conference".to_string())
        }
    }

    /// Add participant to recording
    pub fn add_participant(
        &self,
        conference_id: &Uuid,
        participant: RecordingParticipant,
    ) -> Result<(), String> {
        let mut active = self.active_recordings.lock().unwrap();

        if let Some(recording) = active.get_mut(conference_id) {
            recording.add_participant(participant);
            Ok(())
        } else {
            Err("No active recording for this conference".to_string())
        }
    }

    /// Update recording metadata
    pub fn update_metadata(
        &self,
        conference_id: &Uuid,
        metadata: RecordingMetadata,
    ) -> Result<(), String> {
        let mut active = self.active_recordings.lock().unwrap();

        if let Some(recording) = active.get_mut(conference_id) {
            recording.metadata = metadata;
            Ok(())
        } else {
            Err("No active recording for this conference".to_string())
        }
    }

    /// Get active recording for a conference
    pub fn get_recording(&self, conference_id: &Uuid) -> Option<ConferenceRecording> {
        self.active_recordings
            .lock()
            .unwrap()
            .get(conference_id)
            .cloned()
    }

    /// Check if a conference is being recorded
    pub fn is_recording(&self, conference_id: &Uuid) -> bool {
        self.active_recordings
            .lock()
            .unwrap()
            .contains_key(conference_id)
    }

    /// List all active recordings
    pub fn list_active_recordings(&self) -> Vec<ConferenceRecording> {
        self.active_recordings
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// List completed recordings
    pub fn list_completed_recordings(&self) -> Vec<ConferenceRecording> {
        self.completed_recordings.lock().unwrap().clone()
    }

    /// Get recording by ID
    pub fn get_recording_by_id(&self, recording_id: &Uuid) -> Option<ConferenceRecording> {
        // Check active first
        if let Some(recording) = self
            .active_recordings
            .lock()
            .unwrap()
            .values()
            .find(|r| &r.id == recording_id)
        {
            return Some(recording.clone());
        }

        // Check completed
        self.completed_recordings
            .lock()
            .unwrap()
            .iter()
            .find(|r| &r.id == recording_id)
            .cloned()
    }

    /// Delete completed recording
    pub fn delete_recording(&self, recording_id: &Uuid) -> Result<(), String> {
        let mut completed = self.completed_recordings.lock().unwrap();

        if let Some(pos) = completed.iter().position(|r| &r.id == recording_id) {
            completed.remove(pos);
            Ok(())
        } else {
            Err("Recording not found in completed recordings".to_string())
        }
    }

    /// Get statistics
    pub fn get_statistics(&self) -> RecordingStatistics {
        let active = self.active_recordings.lock().unwrap();
        let completed = self.completed_recordings.lock().unwrap();

        let total_size: u64 = completed.iter().map(|r| r.file_size_bytes).sum();
        let total_duration: u64 = completed.iter().map(|r| r.duration_ms).sum();

        RecordingStatistics {
            active_recordings: active.len(),
            completed_recordings: completed.len(),
            total_recordings: active.len() + completed.len(),
            total_size_bytes: total_size,
            total_duration_ms: total_duration,
        }
    }
}

impl Default for ConferenceRecordingManager {
    fn default() -> Self {
        Self::new(RecordingConfig::default())
    }
}

/// Recording statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatistics {
    pub active_recordings: usize,
    pub completed_recordings: usize,
    pub total_recordings: usize,
    pub total_size_bytes: u64,
    pub total_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_format() {
        assert_eq!(RecordingFormat::Wav.extension(), "wav");
        assert_eq!(RecordingFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(RecordingFormat::Opus.extension(), "opus");
    }

    #[test]
    fn test_conference_recording_creation() {
        let conference_id = Uuid::new_v4();
        let recording = ConferenceRecording::new(
            conference_id,
            "Test Conference".to_string(),
            RecordingFormat::Wav,
            RecordingMode::Mixed,
            PathBuf::from("/tmp/recording.wav"),
        );

        assert_eq!(recording.conference_id, conference_id);
        assert_eq!(recording.state, RecordingState::Recording);
        assert!(recording.is_active());
        assert!(!recording.is_stopped());
    }

    #[test]
    fn test_recording_state_transitions() {
        let mut recording = ConferenceRecording::new(
            Uuid::new_v4(),
            "Test".to_string(),
            RecordingFormat::Wav,
            RecordingMode::Mixed,
            PathBuf::from("/tmp/test.wav"),
        );

        assert!(recording.is_active());

        recording.pause();
        assert!(recording.is_paused());
        assert_eq!(recording.state, RecordingState::Paused);

        recording.resume();
        assert!(recording.is_active());

        recording.stop();
        assert!(recording.is_stopped());
        assert!(recording.stopped_at.is_some());
    }

    #[test]
    fn test_recording_participant() {
        let mut participant = RecordingParticipant::new(
            "user123".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(participant.user_id, "user123");
        assert!(participant.left_at.is_none());

        participant.mark_left();
        assert!(participant.left_at.is_some());
    }

    #[test]
    fn test_recording_metadata() {
        let mut metadata = RecordingMetadata::new()
            .with_description("Important meeting".to_string());

        metadata.add_tag("sales".to_string());
        metadata.add_tag("q4".to_string());

        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.description, "Important meeting");
    }

    #[test]
    fn test_recording_manager_start_stop() {
        let manager = ConferenceRecordingManager::default();
        let conference_id = Uuid::new_v4();

        // Start recording
        let result = manager.start_recording(
            conference_id,
            "Test Conference".to_string(),
            None,
            None,
        );
        assert!(result.is_ok());
        assert!(manager.is_recording(&conference_id));

        // Stop recording
        let result = manager.stop_recording(&conference_id);
        assert!(result.is_ok());
        assert!(!manager.is_recording(&conference_id));
    }

    #[test]
    fn test_recording_manager_pause_resume() {
        let manager = ConferenceRecordingManager::default();
        let conference_id = Uuid::new_v4();

        manager
            .start_recording(conference_id, "Test".to_string(), None, None)
            .unwrap();

        assert!(manager.pause_recording(&conference_id).is_ok());
        let recording = manager.get_recording(&conference_id).unwrap();
        assert!(recording.is_paused());

        assert!(manager.resume_recording(&conference_id).is_ok());
        let recording = manager.get_recording(&conference_id).unwrap();
        assert!(recording.is_active());
    }

    #[test]
    fn test_add_participant() {
        let manager = ConferenceRecordingManager::default();
        let conference_id = Uuid::new_v4();

        manager
            .start_recording(conference_id, "Test".to_string(), None, None)
            .unwrap();

        let participant = RecordingParticipant::new(
            "user1".to_string(),
            "User One".to_string(),
        );

        assert!(manager.add_participant(&conference_id, participant).is_ok());

        let recording = manager.get_recording(&conference_id).unwrap();
        assert_eq!(recording.participants.len(), 1);
    }

    #[test]
    fn test_recording_statistics() {
        let manager = ConferenceRecordingManager::default();

        let conf1 = Uuid::new_v4();
        let conf2 = Uuid::new_v4();

        manager
            .start_recording(conf1, "Conf 1".to_string(), None, None)
            .unwrap();
        manager
            .start_recording(conf2, "Conf 2".to_string(), None, None)
            .unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.active_recordings, 2);
        assert_eq!(stats.total_recordings, 2);

        manager.stop_recording(&conf1).unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.active_recordings, 1);
        assert_eq!(stats.completed_recordings, 1);
    }

    #[test]
    fn test_list_recordings() {
        let manager = ConferenceRecordingManager::default();

        let conf1 = Uuid::new_v4();
        let conf2 = Uuid::new_v4();

        manager
            .start_recording(conf1, "Conf 1".to_string(), None, None)
            .unwrap();
        manager
            .start_recording(conf2, "Conf 2".to_string(), None, None)
            .unwrap();

        let active = manager.list_active_recordings();
        assert_eq!(active.len(), 2);

        manager.stop_recording(&conf1).unwrap();

        let completed = manager.list_completed_recordings();
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_get_recording_by_id() {
        let manager = ConferenceRecordingManager::default();
        let conference_id = Uuid::new_v4();

        let recording_id = manager
            .start_recording(conference_id, "Test".to_string(), None, None)
            .unwrap();

        let recording = manager.get_recording_by_id(&recording_id);
        assert!(recording.is_some());
        assert_eq!(recording.unwrap().id, recording_id);
    }

    #[test]
    fn test_delete_recording() {
        let manager = ConferenceRecordingManager::default();
        let conference_id = Uuid::new_v4();

        let recording_id = manager
            .start_recording(conference_id, "Test".to_string(), None, None)
            .unwrap();

        manager.stop_recording(&conference_id).unwrap();

        assert!(manager.delete_recording(&recording_id).is_ok());
        assert!(manager.get_recording_by_id(&recording_id).is_none());
    }
}

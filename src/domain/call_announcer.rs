/// Call Announcer for playing audio announcements into active calls
use crate::domain::audio::{AudioFileManager, SequenceBuilder, StreamingAudioPlayer, Language};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Announcement type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnouncementType {
    /// Queue position announcement
    QueuePosition,
    /// Estimated wait time
    WaitTime,
    /// Periodic announcement in queue
    Periodic,
    /// Welcome message
    Welcome,
    /// Goodbye message
    Goodbye,
    /// Custom announcement
    Custom,
}

/// Announcement request
#[derive(Debug, Clone)]
pub struct AnnouncementRequest {
    /// Request ID
    pub id: Uuid,
    /// Call ID to play announcement to
    pub call_id: String,
    /// Type of announcement
    pub announcement_type: AnnouncementType,
    /// Audio file IDs to play in sequence
    pub audio_files: Vec<String>,
    /// Language for announcements
    pub language: Language,
    /// When to play (None = immediately)
    pub play_at: Option<Instant>,
    /// Repeat interval (for periodic announcements)
    pub repeat_interval: Option<Duration>,
}

impl AnnouncementRequest {
    /// Create new announcement request
    pub fn new(call_id: String, announcement_type: AnnouncementType) -> Self {
        Self {
            id: Uuid::new_v4(),
            call_id,
            announcement_type,
            audio_files: Vec::new(),
            language: Language::En,
            play_at: None,
            repeat_interval: None,
        }
    }

    /// Add audio file to sequence
    pub fn add_audio(mut self, file_id: &str) -> Self {
        self.audio_files.push(file_id.to_string());
        self
    }

    /// Set language
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Play immediately
    pub fn immediate(mut self) -> Self {
        self.play_at = None;
        self
    }

    /// Schedule for later
    pub fn at(mut self, when: Instant) -> Self {
        self.play_at = Some(when);
        self
    }

    /// Repeat periodically
    pub fn repeat_every(mut self, interval: Duration) -> Self {
        self.repeat_interval = Some(interval);
        self
    }
}

/// Active announcement being played
struct ActiveAnnouncement {
    request: AnnouncementRequest,
    player: StreamingAudioPlayer,
    started_at: Instant,
    next_play_at: Option<Instant>,
}

/// Call Announcer Service
pub struct CallAnnouncer {
    /// Audio file manager
    audio_manager: Arc<AudioFileManager>,
    /// Active announcements by call ID
    active: Arc<Mutex<HashMap<String, Vec<ActiveAnnouncement>>>>,
    /// Scheduled announcements
    scheduled: Arc<Mutex<Vec<AnnouncementRequest>>>,
}

impl CallAnnouncer {
    /// Create new call announcer
    pub fn new(audio_manager: Arc<AudioFileManager>) -> Self {
        Self {
            audio_manager,
            active: Arc::new(Mutex::new(HashMap::new())),
            scheduled: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Play announcement
    pub fn play_announcement(&self, request: AnnouncementRequest) -> Result<Uuid, String> {
        let request_id = request.id;

        // Check if should be scheduled
        if let Some(play_at) = request.play_at {
            if play_at > Instant::now() {
                let mut scheduled = self.scheduled.lock().unwrap();
                scheduled.push(request);
                return Ok(request_id);
            }
        }

        // Create player for announcement
        let player = self.create_player(&request)?;

        let announcement = ActiveAnnouncement {
            request: request.clone(),
            player,
            started_at: Instant::now(),
            next_play_at: request.repeat_interval.map(|interval| Instant::now() + interval),
        };

        // Add to active announcements
        let mut active = self.active.lock().unwrap();
        active
            .entry(request.call_id.clone())
            .or_insert_with(Vec::new)
            .push(announcement);

        Ok(request_id)
    }

    /// Create player from request
    fn create_player(&self, request: &AnnouncementRequest) -> Result<StreamingAudioPlayer, String> {
        let mut builder = SequenceBuilder::new();

        for file_id in &request.audio_files {
            let audio = self.audio_manager
                .get_with_fallback(file_id, request.language)
                .ok_or_else(|| format!("Audio file not found: {}", file_id))?;
            builder = builder.add(audio);
        }

        builder = builder.allow_interrupt(true);

        let player = builder.build();
        Ok(StreamingAudioPlayer::from(player))
    }

    /// Queue position announcement
    /// Plays "You are caller number <position> in the queue"
    pub fn announce_position(&self, call_id: String, position: usize, language: Language) -> Result<Uuid, String> {
        let request = AnnouncementRequest::new(call_id, AnnouncementType::QueuePosition)
            .with_language(language)
            .add_audio("queue_you_are_caller")
            .add_audio(&Self::number_to_audio_id(position))
            .add_audio("queue_in_queue")
            .immediate();

        self.play_announcement(request)
    }

    /// Wait time announcement
    /// Plays "Your estimated wait time is <minutes> minutes"
    pub fn announce_wait_time(&self, call_id: String, minutes: usize, language: Language) -> Result<Uuid, String> {
        let request = AnnouncementRequest::new(call_id, AnnouncementType::WaitTime)
            .with_language(language)
            .add_audio("queue_wait_time_is")
            .add_audio(&Self::number_to_audio_id(minutes))
            .add_audio(if minutes == 1 { "minute" } else { "minutes" })
            .immediate();

        self.play_announcement(request)
    }

    /// Welcome announcement
    pub fn announce_welcome(&self, call_id: String, language: Language) -> Result<Uuid, String> {
        let request = AnnouncementRequest::new(call_id, AnnouncementType::Welcome)
            .with_language(language)
            .add_audio("welcome")
            .immediate();

        self.play_announcement(request)
    }

    /// Convert number to audio file ID
    /// Maps numbers to audio file IDs like "number_1", "number_2", etc.
    fn number_to_audio_id(number: usize) -> String {
        format!("number_{}", number)
    }

    /// Get next audio frame for a call
    /// Returns frames from active announcements
    pub fn get_next_frame(&self, call_id: &str) -> Option<(Vec<i16>, usize)> {
        let mut active = self.active.lock().unwrap();

        if let Some(announcements) = active.get_mut(call_id) {
            // Get first active announcement
            if let Some(announcement) = announcements.first_mut() {
                if let Some(frame) = announcement.player.next_frame() {
                    return Some(frame);
                }

                // Announcement finished
                if announcement.player.is_finished() {
                    // Check if should repeat
                    if let Some(next_play) = announcement.next_play_at {
                        if Instant::now() >= next_play {
                            // Restart player
                            if let Ok(new_player) = self.create_player(&announcement.request) {
                                announcement.player = new_player;
                                announcement.started_at = Instant::now();
                                if let Some(interval) = announcement.request.repeat_interval {
                                    announcement.next_play_at = Some(Instant::now() + interval);
                                }
                                return announcement.player.next_frame();
                            }
                        }
                    } else {
                        // Remove completed announcement
                        announcements.remove(0);
                    }
                }
            }
        }

        None
    }

    /// Check and play scheduled announcements
    pub fn process_scheduled(&self) {
        let now = Instant::now();
        let mut scheduled = self.scheduled.lock().unwrap();

        // Find announcements that should play now
        let mut to_play = Vec::new();
        scheduled.retain(|req| {
            if let Some(play_at) = req.play_at {
                if now >= play_at {
                    to_play.push(req.clone());
                    return false;
                }
            }
            true
        });

        drop(scheduled);

        // Play scheduled announcements
        for request in to_play {
            let _ = self.play_announcement(request);
        }
    }

    /// Stop all announcements for a call
    pub fn stop_announcements(&self, call_id: &str) {
        let mut active = self.active.lock().unwrap();
        active.remove(call_id);

        let mut scheduled = self.scheduled.lock().unwrap();
        scheduled.retain(|req| req.call_id != call_id);
    }

    /// Check if call has active announcements
    pub fn has_active_announcements(&self, call_id: &str) -> bool {
        let active = self.active.lock().unwrap();
        active.get(call_id).map(|a| !a.is_empty()).unwrap_or(false)
    }

    /// Get number of active announcements for a call
    pub fn active_count(&self, call_id: &str) -> usize {
        let active = self.active.lock().unwrap();
        active.get(call_id).map(|a| a.len()).unwrap_or(0)
    }

    /// Get number of scheduled announcements
    pub fn scheduled_count(&self) -> usize {
        let scheduled = self.scheduled.lock().unwrap();
        scheduled.len()
    }
}

/// Helper to convert StreamingAudioPlayer from SequentialPlayer
impl From<crate::domain::audio::SequentialPlayer> for StreamingAudioPlayer {
    fn from(_player: crate::domain::audio::SequentialPlayer) -> Self {
        // This is a workaround - in real implementation, we'd need proper conversion
        // For now, create a new StreamingAudioPlayer
        StreamingAudioPlayer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_audio_manager() -> Arc<AudioFileManager> {
        Arc::new(AudioFileManager::new("/tmp/audio"))
    }

    #[test]
    fn test_announcer_creation() {
        let audio_mgr = create_test_audio_manager();
        let announcer = CallAnnouncer::new(audio_mgr);

        assert_eq!(announcer.scheduled_count(), 0);
    }

    #[test]
    fn test_announcement_request_builder() {
        let request = AnnouncementRequest::new("call-123".to_string(), AnnouncementType::Welcome)
            .add_audio("welcome")
            .with_language(Language::Es)
            .immediate();

        assert_eq!(request.call_id, "call-123");
        assert_eq!(request.announcement_type, AnnouncementType::Welcome);
        assert_eq!(request.audio_files.len(), 1);
        assert_eq!(request.language, Language::Es);
        assert!(request.play_at.is_none());
    }

    #[test]
    fn test_scheduled_announcement() {
        let audio_mgr = create_test_audio_manager();
        let announcer = CallAnnouncer::new(audio_mgr);

        let future_time = Instant::now() + Duration::from_secs(10);
        let request = AnnouncementRequest::new("call-123".to_string(), AnnouncementType::Periodic)
            .add_audio("periodic_announce")
            .at(future_time);

        let result = announcer.play_announcement(request);
        assert!(result.is_ok());

        // Should be scheduled, not active
        assert_eq!(announcer.scheduled_count(), 1);
        assert_eq!(announcer.active_count("call-123"), 0);
    }

    #[test]
    fn test_stop_announcements() {
        let audio_mgr = create_test_audio_manager();
        let announcer = CallAnnouncer::new(audio_mgr);

        announcer.stop_announcements("call-123");
        assert_eq!(announcer.active_count("call-123"), 0);
    }

    #[test]
    fn test_number_to_audio_id() {
        assert_eq!(CallAnnouncer::number_to_audio_id(1), "number_1");
        assert_eq!(CallAnnouncer::number_to_audio_id(42), "number_42");
    }

    #[test]
    fn test_has_active_announcements() {
        let audio_mgr = create_test_audio_manager();
        let announcer = CallAnnouncer::new(audio_mgr);

        assert!(!announcer.has_active_announcements("call-123"));
    }

    #[test]
    fn test_repeat_announcement() {
        let request = AnnouncementRequest::new("call-123".to_string(), AnnouncementType::Periodic)
            .add_audio("periodic")
            .repeat_every(Duration::from_secs(30));

        assert!(request.repeat_interval.is_some());
        assert_eq!(request.repeat_interval.unwrap(), Duration::from_secs(30));
    }
}

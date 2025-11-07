/// Voicemail recording and playback services
use crate::domain::audio::wav::{WavFile, WavFormat};
use crate::domain::audio::player::{AudioPlayer, PlaybackOptions};
use crate::domain::voicemail::{VoicemailMessage, VoicemailMailbox};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use uuid::Uuid;

/// Voicemail recorder for capturing audio to WAV files
pub struct VoicemailRecorder {
    /// Recording buffer for audio samples
    buffer: Vec<i16>,
    /// Maximum recording duration in seconds
    max_duration: u32,
    /// Sample rate (8000 Hz for telephony)
    sample_rate: u32,
    /// Recording start time
    start_time: Option<std::time::Instant>,
    /// Is currently recording
    is_recording: bool,
}

impl VoicemailRecorder {
    /// Create new voicemail recorder
    pub fn new(max_duration: u32) -> Self {
        Self {
            buffer: Vec::new(),
            max_duration,
            sample_rate: 8000, // Standard telephony sample rate
            start_time: None,
            is_recording: false,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.buffer.clear();
        self.start_time = Some(std::time::Instant::now());
        self.is_recording = true;
    }

    /// Add audio samples to recording
    /// Samples should be 16-bit PCM mono at sample_rate Hz
    pub fn add_samples(&mut self, samples: &[i16]) -> Result<(), String> {
        if !self.is_recording {
            return Err("Not currently recording".to_string());
        }

        // Check if we've exceeded max duration
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs();
            if elapsed >= self.max_duration as u64 {
                return Err("Maximum recording duration exceeded".to_string());
            }
        }

        self.buffer.extend_from_slice(samples);
        Ok(())
    }

    /// Stop recording and return duration in seconds
    pub fn stop(&mut self) -> u32 {
        self.is_recording = false;
        let duration = if let Some(start) = self.start_time {
            start.elapsed().as_secs() as u32
        } else {
            0
        };
        duration
    }

    /// Get current recording duration
    pub fn duration(&self) -> u32 {
        if let Some(start) = self.start_time {
            start.elapsed().as_secs() as u32
        } else {
            0
        }
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Save recording to WAV file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        self.write_wav(file)
    }

    /// Write WAV file format
    fn write_wav<W: Write>(&self, mut writer: W) -> Result<(), String> {
        let num_samples = self.buffer.len() as u32;
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = self.sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        let block_align = num_channels * bits_per_sample / 8;
        let data_size = num_samples * num_channels as u32 * bits_per_sample as u32 / 8;
        let file_size = 36 + data_size;

        // RIFF header
        writer.write_all(b"RIFF")
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&file_size.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(b"WAVE")
            .map_err(|e| format!("Write error: {}", e))?;

        // fmt chunk
        writer.write_all(b"fmt ")
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&16u32.to_le_bytes()) // fmt chunk size
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&1u16.to_le_bytes()) // audio format (1 = PCM)
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&num_channels.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&self.sample_rate.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&byte_rate.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&block_align.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&bits_per_sample.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // data chunk
        writer.write_all(b"data")
            .map_err(|e| format!("Write error: {}", e))?;
        writer.write_all(&data_size.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // Write audio samples
        for &sample in &self.buffer {
            writer.write_all(&sample.to_le_bytes())
                .map_err(|e| format!("Write error: {}", e))?;
        }

        Ok(())
    }

    /// Get number of samples recorded
    pub fn sample_count(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the recording buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.start_time = None;
        self.is_recording = false;
    }
}

/// Voicemail playback service
pub struct VoicemailPlayer {
    /// Audio player
    player: AudioPlayer,
    /// Current message being played
    current_message: Option<VoicemailMessage>,
    /// Base directory for voicemail files
    base_dir: PathBuf,
}

impl VoicemailPlayer {
    /// Create new voicemail player
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            player: AudioPlayer::with_options(PlaybackOptions {
                frame_duration_ms: 20,
                loop_playback: false,
                allow_interrupt: true,
            }),
            current_message: None,
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Load and play voicemail message
    pub fn play_message(&mut self, message: VoicemailMessage) -> Result<(), String> {
        let audio_path = if Path::new(&message.audio_file_path).is_absolute() {
            PathBuf::from(&message.audio_file_path)
        } else {
            self.base_dir.join(&message.audio_file_path)
        };

        let wav_file = WavFile::from_file(&audio_path)
            .map_err(|e| format!("Failed to load audio file: {:?}", e))?;

        // Convert to G.711 compatible format if needed
        let compatible = wav_file.to_g711_compatible();

        self.player.load(Arc::new(compatible));
        self.player.play();
        self.current_message = Some(message);

        Ok(())
    }

    /// Play greeting for mailbox
    pub fn play_greeting(&mut self, mailbox: &VoicemailMailbox) -> Result<(), String> {
        if let Some(ref greeting_file) = mailbox.greeting_file {
            let greeting_path = if Path::new(greeting_file).is_absolute() {
                PathBuf::from(greeting_file)
            } else {
                self.base_dir.join(greeting_file)
            };

            let wav_file = WavFile::from_file(&greeting_path)
                .map_err(|e| format!("Failed to load greeting file: {:?}", e))?;

            let compatible = wav_file.to_g711_compatible();
            self.player.load(Arc::new(compatible));
            self.player.play();
            Ok(())
        } else {
            Err("No greeting file configured".to_string())
        }
    }

    /// Get next audio frame for RTP streaming
    pub fn next_frame(&mut self) -> Option<(Vec<i16>, usize)> {
        self.player.next_frame()
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.player.pause();
    }

    /// Resume playback
    pub fn resume(&mut self) {
        self.player.play();
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.player.stop();
        self.current_message = None;
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    /// Get current message
    pub fn current_message(&self) -> Option<&VoicemailMessage> {
        self.current_message.as_ref()
    }

    /// Get playback progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        self.player.progress()
    }

    /// Seek to position in seconds
    pub fn seek(&mut self, seconds: f64) {
        self.player.seek(seconds);
    }

    /// Fast forward by seconds
    pub fn fast_forward(&mut self, seconds: f64) {
        let current = self.player.position_seconds();
        self.player.seek(current + seconds);
    }

    /// Rewind by seconds
    pub fn rewind(&mut self, seconds: f64) {
        let current = self.player.position_seconds();
        let new_pos = (current - seconds).max(0.0);
        self.player.seek(new_pos);
    }

    /// Replay from beginning
    pub fn replay(&mut self) {
        self.player.seek(0.0);
        self.player.play();
    }
}

/// Voicemail service for managing recordings and playback
pub struct VoicemailService {
    /// Base directory for voicemail storage
    base_dir: PathBuf,
}

impl VoicemailService {
    /// Create new voicemail service
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Get directory for mailbox
    fn mailbox_dir(&self, mailbox_id: &str) -> PathBuf {
        self.base_dir.join(mailbox_id)
    }

    /// Ensure mailbox directory exists
    fn ensure_mailbox_dir(&self, mailbox_id: &str) -> Result<PathBuf, String> {
        let dir = self.mailbox_dir(mailbox_id);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create mailbox directory: {}", e))?;
        Ok(dir)
    }

    /// Generate unique filename for new message
    fn generate_filename(&self, mailbox_id: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let uuid = Uuid::new_v4();
        format!("{}/msg_{}_{}.wav", mailbox_id, timestamp, uuid)
    }

    /// Create recorder for mailbox
    pub fn create_recorder(&self, mailbox: &VoicemailMailbox) -> VoicemailRecorder {
        VoicemailRecorder::new(mailbox.max_message_duration)
    }

    /// Save recording and create voicemail message
    pub fn save_recording(
        &self,
        mailbox_id: &str,
        caller: String,
        caller_name: Option<String>,
        recorder: &VoicemailRecorder,
    ) -> Result<VoicemailMessage, String> {
        // Ensure mailbox directory exists
        self.ensure_mailbox_dir(mailbox_id)?;

        // Generate filename
        let filename = self.generate_filename(mailbox_id);
        let full_path = self.base_dir.join(&filename);

        // Save recording
        recorder.save_to_file(&full_path)?;

        // Get duration
        let duration = recorder.duration();

        // Create voicemail message
        let message = VoicemailMessage::new(
            mailbox_id.to_string(),
            caller,
            caller_name,
            duration,
            filename,
            "wav".to_string(),
        );

        Ok(message)
    }

    /// Create player
    pub fn create_player(&self) -> VoicemailPlayer {
        VoicemailPlayer::new(&self.base_dir)
    }

    /// Delete voicemail audio file
    pub fn delete_audio_file(&self, message: &VoicemailMessage) -> Result<(), String> {
        let path = self.base_dir.join(&message.audio_file_path);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete audio file: {}", e))?;
        }
        Ok(())
    }

    /// Get file size for message
    pub fn get_file_size(&self, message: &VoicemailMessage) -> Result<u64, String> {
        let path = self.base_dir.join(&message.audio_file_path);
        fs::metadata(&path)
            .map(|m| m.len())
            .map_err(|e| format!("Failed to get file size: {}", e))
    }
}

/// Message Waiting Indicator (MWI) state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MwiState {
    /// Mailbox ID
    pub mailbox_id: String,
    /// Number of new (unread) messages
    pub new_messages: u32,
    /// Number of old (read/saved) messages
    pub old_messages: u32,
    /// Number of urgent new messages
    pub new_urgent: u32,
    /// Number of urgent old messages
    pub old_urgent: u32,
}

impl MwiState {
    /// Create new MWI state
    pub fn new(mailbox_id: String) -> Self {
        Self {
            mailbox_id,
            new_messages: 0,
            old_messages: 0,
            new_urgent: 0,
            old_urgent: 0,
        }
    }

    /// Check if there are new messages
    pub fn has_new_messages(&self) -> bool {
        self.new_messages > 0
    }

    /// Get total messages
    pub fn total_messages(&self) -> u32 {
        self.new_messages + self.old_messages
    }

    /// Format MWI for SIP NOTIFY body
    pub fn to_sip_notify_body(&self) -> String {
        format!(
            "Messages-Waiting: {}\r\n\
             Message-Account: {}\r\n\
             Voice-Message: {}/{} ({}/{})\r\n",
            if self.has_new_messages() {
                "yes"
            } else {
                "no"
            },
            self.mailbox_id,
            self.new_messages,
            self.old_messages,
            self.new_urgent,
            self.old_urgent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_creation() {
        let recorder = VoicemailRecorder::new(180);
        assert!(!recorder.is_recording());
        assert_eq!(recorder.duration(), 0);
        assert_eq!(recorder.sample_count(), 0);
    }

    #[test]
    fn test_recorder_start_stop() {
        let mut recorder = VoicemailRecorder::new(180);
        recorder.start();
        assert!(recorder.is_recording());

        let duration = recorder.stop();
        assert!(!recorder.is_recording());
        assert!(duration < 2); // Should be very short
    }

    #[test]
    fn test_recorder_add_samples() {
        let mut recorder = VoicemailRecorder::new(180);

        // Cannot add samples when not recording
        let samples = vec![100i16, 200, 300];
        assert!(recorder.add_samples(&samples).is_err());

        // Can add samples when recording
        recorder.start();
        assert!(recorder.add_samples(&samples).is_ok());
        assert_eq!(recorder.sample_count(), 3);
    }

    #[test]
    fn test_recorder_clear() {
        let mut recorder = VoicemailRecorder::new(180);
        recorder.start();

        let samples = vec![100i16, 200, 300];
        recorder.add_samples(&samples).unwrap();
        assert_eq!(recorder.sample_count(), 3);

        recorder.clear();
        assert_eq!(recorder.sample_count(), 0);
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_mwi_state() {
        let mut mwi = MwiState::new("alice".to_string());
        assert_eq!(mwi.mailbox_id, "alice");
        assert!(!mwi.has_new_messages());
        assert_eq!(mwi.total_messages(), 0);

        mwi.new_messages = 3;
        mwi.old_messages = 5;
        assert!(mwi.has_new_messages());
        assert_eq!(mwi.total_messages(), 8);
    }

    #[test]
    fn test_mwi_sip_notify_body() {
        let mut mwi = MwiState::new("alice".to_string());
        mwi.new_messages = 2;
        mwi.old_messages = 3;

        let body = mwi.to_sip_notify_body();
        assert!(body.contains("Messages-Waiting: yes"));
        assert!(body.contains("Message-Account: alice"));
        assert!(body.contains("Voice-Message: 2/3"));
    }

    #[test]
    fn test_mwi_no_messages() {
        let mwi = MwiState::new("bob".to_string());
        let body = mwi.to_sip_notify_body();
        assert!(body.contains("Messages-Waiting: no"));
        assert!(body.contains("Voice-Message: 0/0"));
    }

    #[test]
    fn test_voicemail_service_filename_generation() {
        let service = VoicemailService::new("/var/voicemail");
        let filename1 = service.generate_filename("alice");
        let filename2 = service.generate_filename("alice");

        // Should be different due to UUID
        assert_ne!(filename1, filename2);
        assert!(filename1.starts_with("alice/msg_"));
        assert!(filename1.ends_with(".wav"));
    }
}

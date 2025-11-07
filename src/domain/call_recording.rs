//! Call recording service for capturing and storing call audio
//!
//! Provides functionality to record calls for compliance, quality monitoring,
//! and training purposes. Supports both single-party and multi-party recordings.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Recording format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFormat {
    /// WAV format with PCM encoding
    Wav,
    /// MP3 format (compressed)
    Mp3,
    /// Opus format (compressed, recommended)
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
}

/// Recording direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingDirection {
    /// Record inbound calls only
    Inbound,
    /// Record outbound calls only
    Outbound,
    /// Record both directions
    Both,
    /// Record local calls only
    Local,
}

/// Recording quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingQuality {
    /// Telephony quality (8kHz, mono)
    Telephony,
    /// Standard quality (16kHz, mono)
    Standard,
    /// High quality (48kHz, stereo)
    High,
}

impl RecordingQuality {
    pub fn sample_rate(&self) -> u32 {
        match self {
            RecordingQuality::Telephony => 8000,
            RecordingQuality::Standard => 16000,
            RecordingQuality::High => 48000,
        }
    }

    pub fn channels(&self) -> u16 {
        match self {
            RecordingQuality::Telephony | RecordingQuality::Standard => 1,
            RecordingQuality::High => 2,
        }
    }
}

/// Recording metadata
#[derive(Debug, Clone)]
pub struct RecordingMetadata {
    pub id: Uuid,
    pub call_id: String,
    pub filename: String,
    pub format: RecordingFormat,
    pub quality: RecordingQuality,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub file_size_bytes: u64,
    pub caller: String,
    pub callee: String,
    pub direction: RecordingDirection,
    pub tags: Vec<String>,
}

impl RecordingMetadata {
    pub fn new(
        call_id: String,
        filename: String,
        format: RecordingFormat,
        quality: RecordingQuality,
        caller: String,
        callee: String,
        direction: RecordingDirection,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            call_id,
            filename,
            format,
            quality,
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: 0,
            file_size_bytes: 0,
            caller,
            callee,
            direction,
            tags: vec![],
        }
    }

    pub fn finish(&mut self, file_size: u64) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.duration_ms = (now - self.started_at).num_milliseconds() as u64;
        self.file_size_bytes = file_size;
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}

/// Active recording session
pub struct RecordingSession {
    pub metadata: RecordingMetadata,
    pub file_path: PathBuf,
    buffer: Vec<i16>,
    file_handle: Option<File>,
    sample_count: usize,
    is_paused: bool,
}

impl RecordingSession {
    pub fn new(metadata: RecordingMetadata, file_path: PathBuf, sample_rate: u32) -> Result<Self, String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create recording directory: {}", e))?;
        }

        // For WAV format, we'll write the header when we start
        let file_handle = File::create(&file_path)
            .map_err(|e| format!("Failed to create recording file: {}", e))?;

        Ok(Self {
            metadata,
            file_path,
            buffer: Vec::new(),
            file_handle: Some(file_handle),
            sample_count: 0,
            is_paused: false,
        })
    }

    /// Add audio samples to the recording
    pub fn add_samples(&mut self, samples: &[i16]) -> Result<(), String> {
        if self.is_paused {
            return Ok(());
        }

        self.buffer.extend_from_slice(samples);
        self.sample_count += samples.len();

        // Flush buffer periodically (every 1 second of audio)
        let sample_rate = self.metadata.quality.sample_rate() as usize;
        if self.buffer.len() >= sample_rate {
            self.flush_buffer()?;
        }

        Ok(())
    }

    /// Flush audio buffer to disk
    fn flush_buffer(&mut self) -> Result<(), String> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        if let Some(ref mut file) = self.file_handle {
            // Convert i16 samples to bytes (little-endian)
            let mut bytes = Vec::with_capacity(self.buffer.len() * 2);
            for &sample in &self.buffer {
                bytes.push((sample & 0xFF) as u8);
                bytes.push(((sample >> 8) & 0xFF) as u8);
            }

            file.write_all(&bytes)
                .map_err(|e| format!("Failed to write audio data: {}", e))?;

            file.flush()
                .map_err(|e| format!("Failed to flush audio data: {}", e))?;
        }

        self.buffer.clear();
        Ok(())
    }

    /// Pause recording
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume recording
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Stop recording and finalize file
    pub fn stop(&mut self) -> Result<(), String> {
        // Flush any remaining buffer
        self.flush_buffer()?;

        // Write WAV header
        if self.metadata.format == RecordingFormat::Wav {
            self.write_wav_header()?;
        }

        // Close file handle
        self.file_handle = None;

        // Update metadata
        let file_size = fs::metadata(&self.file_path)
            .map(|m| m.len())
            .unwrap_or(0);
        self.metadata.finish(file_size);

        Ok(())
    }

    /// Write WAV file header
    fn write_wav_header(&mut self) -> Result<(), String> {
        let sample_rate = self.metadata.quality.sample_rate();
        let channels = self.metadata.quality.channels();
        let bits_per_sample = 16u16;

        let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
        let block_align = channels * (bits_per_sample / 8);
        let data_size = self.sample_count * 2; // 16-bit samples

        // Open file for writing header
        let mut file = File::options()
            .write(true)
            .open(&self.file_path)
            .map_err(|e| format!("Failed to open file for header: {}", e))?;

        // Write RIFF header
        file.write_all(b"RIFF")
            .map_err(|e| format!("Failed to write RIFF header: {}", e))?;
        file.write_all(&((data_size + 36) as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write file size: {}", e))?;
        file.write_all(b"WAVE")
            .map_err(|e| format!("Failed to write WAVE header: {}", e))?;

        // Write fmt chunk
        file.write_all(b"fmt ")
            .map_err(|e| format!("Failed to write fmt header: {}", e))?;
        file.write_all(&16u32.to_le_bytes()) // fmt chunk size
            .map_err(|e| format!("Failed to write fmt size: {}", e))?;
        file.write_all(&1u16.to_le_bytes()) // PCM format
            .map_err(|e| format!("Failed to write audio format: {}", e))?;
        file.write_all(&channels.to_le_bytes())
            .map_err(|e| format!("Failed to write channels: {}", e))?;
        file.write_all(&sample_rate.to_le_bytes())
            .map_err(|e| format!("Failed to write sample rate: {}", e))?;
        file.write_all(&byte_rate.to_le_bytes())
            .map_err(|e| format!("Failed to write byte rate: {}", e))?;
        file.write_all(&block_align.to_le_bytes())
            .map_err(|e| format!("Failed to write block align: {}", e))?;
        file.write_all(&bits_per_sample.to_le_bytes())
            .map_err(|e| format!("Failed to write bits per sample: {}", e))?;

        // Write data chunk
        file.write_all(b"data")
            .map_err(|e| format!("Failed to write data header: {}", e))?;
        file.write_all(&(data_size as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write data size: {}", e))?;

        Ok(())
    }

    pub fn get_duration_ms(&self) -> u64 {
        let sample_rate = self.metadata.quality.sample_rate() as u64;
        if sample_rate == 0 {
            return 0;
        }
        (self.sample_count as u64 * 1000) / sample_rate
    }
}

/// Call recording manager
pub struct CallRecordingManager {
    base_dir: PathBuf,
    active_recordings: Arc<Mutex<HashMap<String, RecordingSession>>>,
    completed_recordings: Arc<Mutex<Vec<RecordingMetadata>>>,
    default_format: RecordingFormat,
    default_quality: RecordingQuality,
    auto_record: bool,
}

impl CallRecordingManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            active_recordings: Arc::new(Mutex::new(HashMap::new())),
            completed_recordings: Arc::new(Mutex::new(Vec::new())),
            default_format: RecordingFormat::Wav,
            default_quality: RecordingQuality::Standard,
            auto_record: false,
        }
    }

    /// Enable automatic recording for all calls
    pub fn enable_auto_record(&mut self) {
        self.auto_record = true;
    }

    /// Set default recording format
    pub fn set_default_format(&mut self, format: RecordingFormat) {
        self.default_format = format;
    }

    /// Set default recording quality
    pub fn set_default_quality(&mut self, quality: RecordingQuality) {
        self.default_quality = quality;
    }

    /// Start recording a call
    pub fn start_recording(
        &self,
        call_id: String,
        caller: String,
        callee: String,
        direction: RecordingDirection,
    ) -> Result<Uuid, String> {
        let mut recordings = self.active_recordings.lock().unwrap();

        // Check if already recording this call
        if recordings.contains_key(&call_id) {
            return Err(format!("Call {} is already being recorded", call_id));
        }

        // Generate filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.{}", call_id, timestamp, self.default_format.extension());
        let file_path = self.base_dir.join(&filename);

        // Create metadata
        let metadata = RecordingMetadata::new(
            call_id.clone(),
            filename,
            self.default_format,
            self.default_quality,
            caller,
            callee,
            direction,
        );

        let recording_id = metadata.id;

        // Create recording session
        let session = RecordingSession::new(
            metadata,
            file_path,
            self.default_quality.sample_rate(),
        )?;

        recordings.insert(call_id, session);

        Ok(recording_id)
    }

    /// Stop recording a call
    pub fn stop_recording(&self, call_id: &str) -> Result<RecordingMetadata, String> {
        let mut recordings = self.active_recordings.lock().unwrap();

        let mut session = recordings
            .remove(call_id)
            .ok_or_else(|| format!("No active recording found for call {}", call_id))?;

        session.stop()?;

        let metadata = session.metadata.clone();

        // Add to completed recordings
        let mut completed = self.completed_recordings.lock().unwrap();
        completed.push(metadata.clone());

        Ok(metadata)
    }

    /// Pause recording
    pub fn pause_recording(&self, call_id: &str) -> Result<(), String> {
        let mut recordings = self.active_recordings.lock().unwrap();
        let session = recordings
            .get_mut(call_id)
            .ok_or_else(|| format!("No active recording found for call {}", call_id))?;

        session.pause();
        Ok(())
    }

    /// Resume recording
    pub fn resume_recording(&self, call_id: &str) -> Result<(), String> {
        let mut recordings = self.active_recordings.lock().unwrap();
        let session = recordings
            .get_mut(call_id)
            .ok_or_else(|| format!("No active recording found for call {}", call_id))?;

        session.resume();
        Ok(())
    }

    /// Add audio samples to recording
    pub fn add_samples(&self, call_id: &str, samples: &[i16]) -> Result<(), String> {
        let mut recordings = self.active_recordings.lock().unwrap();
        if let Some(session) = recordings.get_mut(call_id) {
            session.add_samples(samples)?;
        }
        Ok(())
    }

    /// Get active recording metadata
    pub fn get_active_recording(&self, call_id: &str) -> Option<RecordingMetadata> {
        let recordings = self.active_recordings.lock().unwrap();
        recordings.get(call_id).map(|s| s.metadata.clone())
    }

    /// Get all active recordings
    pub fn get_active_recordings(&self) -> Vec<RecordingMetadata> {
        let recordings = self.active_recordings.lock().unwrap();
        recordings.values().map(|s| s.metadata.clone()).collect()
    }

    /// Get completed recordings
    pub fn get_completed_recordings(&self) -> Vec<RecordingMetadata> {
        let completed = self.completed_recordings.lock().unwrap();
        completed.clone()
    }

    /// Get recording by ID
    pub fn get_recording_by_id(&self, id: Uuid) -> Option<RecordingMetadata> {
        let completed = self.completed_recordings.lock().unwrap();
        completed.iter().find(|r| r.id == id).cloned()
    }

    /// Delete recording file
    pub fn delete_recording(&self, id: Uuid) -> Result<(), String> {
        let mut completed = self.completed_recordings.lock().unwrap();

        if let Some(index) = completed.iter().position(|r| r.id == id) {
            let recording = completed.remove(index);
            let file_path = self.base_dir.join(&recording.filename);

            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete recording file: {}", e))?;

            Ok(())
        } else {
            Err(format!("Recording {} not found", id))
        }
    }

    /// Get total storage used by recordings
    pub fn get_total_storage_bytes(&self) -> u64 {
        let completed = self.completed_recordings.lock().unwrap();
        completed.iter().map(|r| r.file_size_bytes).sum()
    }

    /// Clean up old recordings (older than specified days)
    pub fn cleanup_old_recordings(&self, days: i64) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let mut completed = self.completed_recordings.lock().unwrap();

        let mut removed_count = 0;
        completed.retain(|recording| {
            if recording.started_at < cutoff {
                let file_path = self.base_dir.join(&recording.filename);
                if let Err(e) = fs::remove_file(&file_path) {
                    eprintln!("Failed to delete old recording {}: {}", recording.filename, e);
                }
                removed_count += 1;
                false
            } else {
                true
            }
        });

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_recording_format_extension() {
        assert_eq!(RecordingFormat::Wav.extension(), "wav");
        assert_eq!(RecordingFormat::Mp3.extension(), "mp3");
        assert_eq!(RecordingFormat::Opus.extension(), "opus");
    }

    #[test]
    fn test_recording_quality() {
        assert_eq!(RecordingQuality::Telephony.sample_rate(), 8000);
        assert_eq!(RecordingQuality::Standard.sample_rate(), 16000);
        assert_eq!(RecordingQuality::High.sample_rate(), 48000);

        assert_eq!(RecordingQuality::Telephony.channels(), 1);
        assert_eq!(RecordingQuality::High.channels(), 2);
    }

    #[test]
    fn test_recording_metadata() {
        let mut metadata = RecordingMetadata::new(
            "call-123".to_string(),
            "recording.wav".to_string(),
            RecordingFormat::Wav,
            RecordingQuality::Standard,
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            RecordingDirection::Both,
        );

        assert!(metadata.is_active());
        assert_eq!(metadata.tags.len(), 0);

        metadata.add_tag("important".to_string());
        assert_eq!(metadata.tags.len(), 1);

        metadata.finish(1024);
        assert!(!metadata.is_active());
        assert_eq!(metadata.file_size_bytes, 1024);
    }

    #[test]
    fn test_call_recording_manager() {
        let temp_dir = env::temp_dir().join("yakyak_test_recordings");
        let manager = CallRecordingManager::new(temp_dir.clone());

        let result = manager.start_recording(
            "call-456".to_string(),
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            RecordingDirection::Both,
        );

        assert!(result.is_ok());

        let active = manager.get_active_recordings();
        assert_eq!(active.len(), 1);

        // Add some samples
        let samples: Vec<i16> = vec![0, 100, 200, 300];
        manager.add_samples("call-456", &samples).unwrap();

        let result = manager.stop_recording("call-456");
        assert!(result.is_ok());

        let completed = manager.get_completed_recordings();
        assert_eq!(completed.len(), 1);

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_pause_resume_recording() {
        let temp_dir = env::temp_dir().join("yakyak_test_pause");
        let manager = CallRecordingManager::new(temp_dir.clone());

        manager.start_recording(
            "call-789".to_string(),
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            RecordingDirection::Both,
        ).unwrap();

        assert!(manager.pause_recording("call-789").is_ok());
        assert!(manager.resume_recording("call-789").is_ok());

        manager.stop_recording("call-789").unwrap();

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_manager_auto_record() {
        let temp_dir = env::temp_dir().join("yakyak_test_auto");
        let mut manager = CallRecordingManager::new(temp_dir);

        assert!(!manager.auto_record);
        manager.enable_auto_record();
        assert!(manager.auto_record);
    }

    #[test]
    fn test_manager_set_defaults() {
        let temp_dir = env::temp_dir().join("yakyak_test_defaults");
        let mut manager = CallRecordingManager::new(temp_dir);

        manager.set_default_format(RecordingFormat::Mp3);
        assert_eq!(manager.default_format, RecordingFormat::Mp3);

        manager.set_default_quality(RecordingQuality::High);
        assert_eq!(manager.default_quality, RecordingQuality::High);
    }

    #[test]
    fn test_delete_recording() {
        let temp_dir = env::temp_dir().join("yakyak_test_delete");
        let manager = CallRecordingManager::new(temp_dir.clone());

        let id = manager.start_recording(
            "call-delete".to_string(),
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            RecordingDirection::Both,
        ).unwrap();

        manager.stop_recording("call-delete").unwrap();

        assert!(manager.delete_recording(id).is_ok());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Music on Hold playback mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MohPlaybackMode {
    /// Play files in order, then repeat
    Sequential,
    /// Play files in random order
    Random,
    /// Play files in order once, then silence
    Once,
    /// Loop single file continuously
    Loop,
}

impl Default for MohPlaybackMode {
    fn default() -> Self {
        MohPlaybackMode::Sequential
    }
}

/// Music on Hold audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MohAudioFormat {
    /// WAV PCM format
    Wav,
    /// MP3 format
    Mp3,
    /// Opus format
    Opus,
    /// Raw PCM
    Raw,
}

impl MohAudioFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(MohAudioFormat::Wav),
            "mp3" => Some(MohAudioFormat::Mp3),
            "opus" => Some(MohAudioFormat::Opus),
            "pcm" | "raw" => Some(MohAudioFormat::Raw),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            MohAudioFormat::Wav => "wav",
            MohAudioFormat::Mp3 => "mp3",
            MohAudioFormat::Opus => "opus",
            MohAudioFormat::Raw => "pcm",
        }
    }
}

/// Music on Hold audio file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MohAudioFile {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub format: MohAudioFormat,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u8,
    pub file_size_bytes: u64,
    pub enabled: bool,
    pub added_at: DateTime<Utc>,
}

impl MohAudioFile {
    pub fn new(name: String, file_path: PathBuf, format: MohAudioFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            file_path,
            format,
            duration_ms: 0,
            sample_rate: 8000,
            channels: 1,
            file_size_bytes: 0,
            enabled: true,
            added_at: Utc::now(),
        }
    }

    pub fn with_metadata(
        mut self,
        duration_ms: u64,
        sample_rate: u32,
        channels: u8,
        file_size_bytes: u64,
    ) -> Self {
        self.duration_ms = duration_ms;
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.file_size_bytes = file_size_bytes;
        self
    }

    pub fn is_valid(&self) -> bool {
        self.enabled && self.file_path.exists()
    }

    pub fn duration_seconds(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }
}

/// Music on Hold playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MohPlaylist {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub audio_files: Vec<Uuid>,
    pub playback_mode: MohPlaybackMode,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MohPlaylist {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            audio_files: Vec::new(),
            playback_mode: MohPlaybackMode::default(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_playback_mode(mut self, mode: MohPlaybackMode) -> Self {
        self.playback_mode = mode;
        self
    }

    pub fn add_file(&mut self, file_id: Uuid) {
        if !self.audio_files.contains(&file_id) {
            self.audio_files.push(file_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_file(&mut self, file_id: &Uuid) -> bool {
        if let Some(pos) = self.audio_files.iter().position(|id| id == file_id) {
            self.audio_files.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn clear_files(&mut self) {
        self.audio_files.clear();
        self.updated_at = Utc::now();
    }

    pub fn file_count(&self) -> usize {
        self.audio_files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.audio_files.is_empty()
    }

    pub fn total_duration_ms(&self, file_manager: &MohFileManager) -> u64 {
        self.audio_files
            .iter()
            .filter_map(|id| file_manager.get_file(id))
            .map(|f| f.duration_ms)
            .sum()
    }
}

/// Music on Hold session for an active call
#[derive(Debug)]
pub struct MohSession {
    pub id: Uuid,
    pub call_id: String,
    pub playlist_id: Uuid,
    pub current_file_index: usize,
    pub playback_position_ms: u64,
    pub started_at: DateTime<Utc>,
    pub paused: bool,
    pub loop_count: u32,
}

impl MohSession {
    pub fn new(call_id: String, playlist_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            call_id,
            playlist_id,
            current_file_index: 0,
            playback_position_ms: 0,
            started_at: Utc::now(),
            paused: false,
            loop_count: 0,
        }
    }

    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

    pub fn duration_seconds(&self) -> i64 {
        self.duration().num_seconds()
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn advance_to_next(&mut self, file_count: usize) -> bool {
        if file_count == 0 {
            return false;
        }

        self.current_file_index = (self.current_file_index + 1) % file_count;
        self.playback_position_ms = 0;

        if self.current_file_index == 0 {
            self.loop_count += 1;
        }

        true
    }

    pub fn reset(&mut self) {
        self.current_file_index = 0;
        self.playback_position_ms = 0;
        self.paused = false;
    }
}

/// Music on Hold file manager
pub struct MohFileManager {
    audio_files: Arc<Mutex<HashMap<Uuid, MohAudioFile>>>,
    base_directory: PathBuf,
}

impl MohFileManager {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            audio_files: Arc::new(Mutex::new(HashMap::new())),
            base_directory,
        }
    }

    pub fn add_file(&self, file: MohAudioFile) -> Uuid {
        let file_id = file.id;
        self.audio_files.lock().unwrap().insert(file_id, file);
        file_id
    }

    pub fn remove_file(&self, file_id: &Uuid) -> bool {
        self.audio_files.lock().unwrap().remove(file_id).is_some()
    }

    pub fn get_file(&self, file_id: &Uuid) -> Option<MohAudioFile> {
        self.audio_files.lock().unwrap().get(file_id).cloned()
    }

    pub fn list_files(&self) -> Vec<MohAudioFile> {
        self.audio_files.lock().unwrap().values().cloned().collect()
    }

    pub fn list_enabled_files(&self) -> Vec<MohAudioFile> {
        self.audio_files
            .lock()
            .unwrap()
            .values()
            .filter(|f| f.is_valid())
            .cloned()
            .collect()
    }

    pub fn enable_file(&self, file_id: &Uuid) -> bool {
        if let Some(file) = self.audio_files.lock().unwrap().get_mut(file_id) {
            file.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_file(&self, file_id: &Uuid) -> bool {
        if let Some(file) = self.audio_files.lock().unwrap().get_mut(file_id) {
            file.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn scan_directory(&self, directory: &Path) -> Result<Vec<Uuid>, String> {
        let mut added_files = Vec::new();

        if !directory.exists() {
            return Err(format!("Directory does not exist: {:?}", directory));
        }

        let entries = std::fs::read_dir(directory)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                if let Some(format) = MohAudioFormat::from_extension(&ext.to_string_lossy()) {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let file_size = std::fs::metadata(&path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    let file = MohAudioFile::new(name, path, format)
                        .with_metadata(0, 8000, 1, file_size);

                    let file_id = self.add_file(file);
                    added_files.push(file_id);
                }
            }
        }

        Ok(added_files)
    }

    pub fn total_storage_bytes(&self) -> u64 {
        self.audio_files
            .lock()
            .unwrap()
            .values()
            .map(|f| f.file_size_bytes)
            .sum()
    }

    pub fn file_count(&self) -> usize {
        self.audio_files.lock().unwrap().len()
    }
}

/// Music on Hold manager
pub struct MohManager {
    file_manager: Arc<MohFileManager>,
    playlists: Arc<Mutex<HashMap<Uuid, MohPlaylist>>>,
    active_sessions: Arc<Mutex<HashMap<String, MohSession>>>,
    default_playlist_id: Arc<Mutex<Option<Uuid>>>,
}

impl MohManager {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            file_manager: Arc::new(MohFileManager::new(base_directory)),
            playlists: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            default_playlist_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn file_manager(&self) -> &MohFileManager {
        &self.file_manager
    }

    pub fn create_playlist(&self, playlist: MohPlaylist) -> Uuid {
        let playlist_id = playlist.id;
        self.playlists.lock().unwrap().insert(playlist_id, playlist);

        // Set as default if no default exists
        let mut default = self.default_playlist_id.lock().unwrap();
        if default.is_none() {
            *default = Some(playlist_id);
        }

        playlist_id
    }

    pub fn get_playlist(&self, playlist_id: &Uuid) -> Option<MohPlaylist> {
        self.playlists.lock().unwrap().get(playlist_id).cloned()
    }

    pub fn update_playlist(&self, playlist: MohPlaylist) -> bool {
        if self.playlists.lock().unwrap().contains_key(&playlist.id) {
            self.playlists.lock().unwrap().insert(playlist.id, playlist);
            true
        } else {
            false
        }
    }

    pub fn delete_playlist(&self, playlist_id: &Uuid) -> bool {
        // Don't delete if it's the default
        if self.is_default_playlist(playlist_id) {
            return false;
        }

        self.playlists.lock().unwrap().remove(playlist_id).is_some()
    }

    pub fn list_playlists(&self) -> Vec<MohPlaylist> {
        self.playlists.lock().unwrap().values().cloned().collect()
    }

    pub fn set_default_playlist(&self, playlist_id: Uuid) -> bool {
        if self.playlists.lock().unwrap().contains_key(&playlist_id) {
            *self.default_playlist_id.lock().unwrap() = Some(playlist_id);
            true
        } else {
            false
        }
    }

    pub fn get_default_playlist(&self) -> Option<MohPlaylist> {
        let default_id = *self.default_playlist_id.lock().unwrap();
        default_id.and_then(|id| self.get_playlist(&id))
    }

    pub fn is_default_playlist(&self, playlist_id: &Uuid) -> bool {
        *self.default_playlist_id.lock().unwrap() == Some(*playlist_id)
    }

    pub fn start_moh(&self, call_id: String, playlist_id: Option<Uuid>) -> Result<Uuid, String> {
        // Use specified playlist or default
        let playlist_id = playlist_id
            .or_else(|| *self.default_playlist_id.lock().unwrap())
            .ok_or("No playlist specified and no default playlist set")?;

        // Verify playlist exists and has files
        let playlist = self
            .get_playlist(&playlist_id)
            .ok_or("Playlist not found")?;

        if !playlist.enabled {
            return Err("Playlist is disabled".to_string());
        }

        if playlist.is_empty() {
            return Err("Playlist is empty".to_string());
        }

        // Create session
        let session = MohSession::new(call_id.clone(), playlist_id);
        let session_id = session.id;

        self.active_sessions
            .lock()
            .unwrap()
            .insert(call_id, session);

        Ok(session_id)
    }

    pub fn stop_moh(&self, call_id: &str) -> bool {
        self.active_sessions.lock().unwrap().remove(call_id).is_some()
    }

    pub fn pause_moh(&self, call_id: &str) -> bool {
        if let Some(session) = self.active_sessions.lock().unwrap().get_mut(call_id) {
            session.pause();
            true
        } else {
            false
        }
    }

    pub fn resume_moh(&self, call_id: &str) -> bool {
        if let Some(session) = self.active_sessions.lock().unwrap().get_mut(call_id) {
            session.resume();
            true
        } else {
            false
        }
    }

    pub fn get_session(&self, call_id: &str) -> Option<MohSession> {
        // Clone the session data for safe return
        self.active_sessions
            .lock()
            .unwrap()
            .get(call_id)
            .map(|s| MohSession {
                id: s.id,
                call_id: s.call_id.clone(),
                playlist_id: s.playlist_id,
                current_file_index: s.current_file_index,
                playback_position_ms: s.playback_position_ms,
                started_at: s.started_at,
                paused: s.paused,
                loop_count: s.loop_count,
            })
    }

    pub fn get_current_file(&self, call_id: &str) -> Option<MohAudioFile> {
        let sessions = self.active_sessions.lock().unwrap();
        let session = sessions.get(call_id)?;

        let playlist = self.get_playlist(&session.playlist_id)?;
        if session.current_file_index >= playlist.audio_files.len() {
            return None;
        }

        let file_id = playlist.audio_files[session.current_file_index];
        self.file_manager.get_file(&file_id)
    }

    pub fn advance_to_next_file(&self, call_id: &str) -> bool {
        let mut sessions = self.active_sessions.lock().unwrap();
        let session = match sessions.get_mut(call_id) {
            Some(s) => s,
            None => return false,
        };

        let playlist = match self.get_playlist(&session.playlist_id) {
            Some(p) => p,
            None => return false,
        };

        session.advance_to_next(playlist.audio_files.len())
    }

    pub fn get_active_session_count(&self) -> usize {
        self.active_sessions.lock().unwrap().len()
    }

    pub fn list_active_sessions(&self) -> Vec<String> {
        self.active_sessions
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> MohStatistics {
        let sessions = self.active_sessions.lock().unwrap();
        let playlists = self.playlists.lock().unwrap();

        MohStatistics {
            total_playlists: playlists.len(),
            enabled_playlists: playlists.values().filter(|p| p.enabled).count(),
            total_audio_files: self.file_manager.file_count(),
            enabled_audio_files: self.file_manager.list_enabled_files().len(),
            active_sessions: sessions.len(),
            total_storage_bytes: self.file_manager.total_storage_bytes(),
        }
    }
}

/// Music on Hold statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MohStatistics {
    pub total_playlists: usize,
    pub enabled_playlists: usize,
    pub total_audio_files: usize,
    pub enabled_audio_files: usize,
    pub active_sessions: usize,
    pub total_storage_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_moh_playback_mode_default() {
        assert_eq!(MohPlaybackMode::default(), MohPlaybackMode::Sequential);
    }

    #[test]
    fn test_moh_audio_format_from_extension() {
        assert_eq!(
            MohAudioFormat::from_extension("wav"),
            Some(MohAudioFormat::Wav)
        );
        assert_eq!(
            MohAudioFormat::from_extension("MP3"),
            Some(MohAudioFormat::Mp3)
        );
        assert_eq!(MohAudioFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_moh_audio_file_creation() {
        let file = MohAudioFile::new(
            "test.wav".to_string(),
            PathBuf::from("/tmp/test.wav"),
            MohAudioFormat::Wav,
        )
        .with_metadata(10000, 8000, 1, 1024);

        assert_eq!(file.name, "test.wav");
        assert_eq!(file.duration_ms, 10000);
        assert_eq!(file.duration_seconds(), 10.0);
        assert!(file.enabled);
    }

    #[test]
    fn test_moh_playlist_creation() {
        let mut playlist = MohPlaylist::new("Test Playlist".to_string())
            .with_description("Test description".to_string())
            .with_playback_mode(MohPlaybackMode::Random);

        assert_eq!(playlist.name, "Test Playlist");
        assert_eq!(playlist.playback_mode, MohPlaybackMode::Random);
        assert!(playlist.is_empty());

        let file_id = Uuid::new_v4();
        playlist.add_file(file_id);
        assert_eq!(playlist.file_count(), 1);
        assert!(!playlist.is_empty());

        playlist.remove_file(&file_id);
        assert!(playlist.is_empty());
    }

    #[test]
    fn test_moh_session() {
        let mut session = MohSession::new("call-123".to_string(), Uuid::new_v4());

        assert_eq!(session.call_id, "call-123");
        assert_eq!(session.current_file_index, 0);
        assert!(!session.paused);

        session.pause();
        assert!(session.paused);

        session.resume();
        assert!(!session.paused);

        assert!(session.advance_to_next(3));
        assert_eq!(session.current_file_index, 1);

        session.reset();
        assert_eq!(session.current_file_index, 0);
    }

    #[test]
    fn test_moh_file_manager() {
        let temp_dir = env::temp_dir();
        let manager = MohFileManager::new(temp_dir);

        let file = MohAudioFile::new(
            "test.wav".to_string(),
            PathBuf::from("/tmp/test.wav"),
            MohAudioFormat::Wav,
        );
        let file_id = manager.add_file(file);

        assert_eq!(manager.file_count(), 1);
        assert!(manager.get_file(&file_id).is_some());

        manager.disable_file(&file_id);
        let disabled_file = manager.get_file(&file_id).unwrap();
        assert!(!disabled_file.enabled);

        manager.enable_file(&file_id);
        let enabled_file = manager.get_file(&file_id).unwrap();
        assert!(enabled_file.enabled);

        manager.remove_file(&file_id);
        assert_eq!(manager.file_count(), 0);
    }

    #[test]
    fn test_moh_manager_playlist_operations() {
        let temp_dir = env::temp_dir();
        let manager = MohManager::new(temp_dir);

        let playlist = MohPlaylist::new("Test".to_string());
        let playlist_id = manager.create_playlist(playlist);

        assert!(manager.get_playlist(&playlist_id).is_some());
        assert_eq!(manager.list_playlists().len(), 1);

        // Should be default since it's the first playlist
        assert!(manager.is_default_playlist(&playlist_id));

        let playlist2 = MohPlaylist::new("Test 2".to_string());
        let playlist2_id = manager.create_playlist(playlist2);

        manager.set_default_playlist(playlist2_id);
        assert!(manager.is_default_playlist(&playlist2_id));

        // Cannot delete default playlist
        assert!(!manager.delete_playlist(&playlist2_id));

        // Can delete non-default
        assert!(manager.delete_playlist(&playlist_id));
    }

    #[test]
    fn test_moh_manager_session_operations() {
        let temp_dir = env::temp_dir();
        let manager = MohManager::new(temp_dir);

        // Create playlist with files
        let mut playlist = MohPlaylist::new("Test".to_string());
        let file = MohAudioFile::new(
            "test.wav".to_string(),
            PathBuf::from("/tmp/test.wav"),
            MohAudioFormat::Wav,
        );
        let file_id = manager.file_manager().add_file(file);
        playlist.add_file(file_id);

        let playlist_id = manager.create_playlist(playlist);

        // Start MOH
        let result = manager.start_moh("call-123".to_string(), Some(playlist_id));
        assert!(result.is_ok());

        assert_eq!(manager.get_active_session_count(), 1);
        assert!(manager.get_session("call-123").is_some());

        // Pause/resume
        assert!(manager.pause_moh("call-123"));
        let session = manager.get_session("call-123").unwrap();
        assert!(session.paused);

        assert!(manager.resume_moh("call-123"));
        let session = manager.get_session("call-123").unwrap();
        assert!(!session.paused);

        // Stop MOH
        assert!(manager.stop_moh("call-123"));
        assert_eq!(manager.get_active_session_count(), 0);
    }

    #[test]
    fn test_moh_statistics() {
        let temp_dir = env::temp_dir();
        let manager = MohManager::new(temp_dir);

        let file = MohAudioFile::new(
            "test.wav".to_string(),
            PathBuf::from("/tmp/test.wav"),
            MohAudioFormat::Wav,
        )
        .with_metadata(10000, 8000, 1, 2048);
        manager.file_manager().add_file(file);

        let playlist = MohPlaylist::new("Test".to_string());
        manager.create_playlist(playlist);

        let stats = manager.get_statistics();
        assert_eq!(stats.total_playlists, 1);
        assert_eq!(stats.total_audio_files, 1);
        assert_eq!(stats.total_storage_bytes, 2048);
    }
}

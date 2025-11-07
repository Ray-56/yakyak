/// Audio player for streaming audio via RTP
use crate::domain::audio::wav::WavFile;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Audio player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlayerState {
    /// Player is idle (no audio loaded)
    Idle,
    /// Player is playing audio
    Playing,
    /// Player is paused
    Paused,
    /// Player has finished playing
    Finished,
    /// Player stopped by user or interrupt
    Stopped,
}

/// Playback options
#[derive(Debug, Clone)]
pub struct PlaybackOptions {
    /// Frame duration in milliseconds (default: 20ms)
    pub frame_duration_ms: u32,
    /// Loop playback (default: false)
    pub loop_playback: bool,
    /// Allow DTMF interrupt (default: true)
    pub allow_interrupt: bool,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        }
    }
}

/// Audio player for streaming audio data
pub struct AudioPlayer {
    /// Current audio file
    audio: Option<Arc<WavFile>>,
    /// Playback state
    state: Arc<Mutex<AudioPlayerState>>,
    /// Current playback position (in samples)
    position: Arc<Mutex<usize>>,
    /// Playback options
    options: PlaybackOptions,
    /// Last frame time for pacing
    last_frame_time: Arc<Mutex<Option<Instant>>>,
}

impl AudioPlayer {
    /// Create new audio player
    pub fn new() -> Self {
        Self {
            audio: None,
            state: Arc::new(Mutex::new(AudioPlayerState::Idle)),
            position: Arc::new(Mutex::new(0)),
            options: PlaybackOptions::default(),
            last_frame_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Create audio player with options
    pub fn with_options(options: PlaybackOptions) -> Self {
        Self {
            audio: None,
            state: Arc::new(Mutex::new(AudioPlayerState::Idle)),
            position: Arc::new(Mutex::new(0)),
            options,
            last_frame_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Load audio file for playback
    pub fn load(&mut self, audio: Arc<WavFile>) {
        self.audio = Some(audio);
        *self.state.lock().unwrap() = AudioPlayerState::Idle;
        *self.position.lock().unwrap() = 0;
        *self.last_frame_time.lock().unwrap() = None;
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            AudioPlayerState::Idle | AudioPlayerState::Paused => {
                *state = AudioPlayerState::Playing;
                *self.last_frame_time.lock().unwrap() = Some(Instant::now());
            }
            AudioPlayerState::Finished | AudioPlayerState::Stopped => {
                // Restart from beginning
                *self.position.lock().unwrap() = 0;
                *state = AudioPlayerState::Playing;
                *self.last_frame_time.lock().unwrap() = Some(Instant::now());
            }
            _ => {}
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        let mut state = self.state.lock().unwrap();
        if *state == AudioPlayerState::Playing {
            *state = AudioPlayerState::Paused;
        }
    }

    /// Stop playback and reset position
    pub fn stop(&mut self) {
        *self.state.lock().unwrap() = AudioPlayerState::Stopped;
        *self.position.lock().unwrap() = 0;
        *self.last_frame_time.lock().unwrap() = None;
    }

    /// Get current state
    pub fn state(&self) -> AudioPlayerState {
        *self.state.lock().unwrap()
    }

    /// Check if player is playing
    pub fn is_playing(&self) -> bool {
        *self.state.lock().unwrap() == AudioPlayerState::Playing
    }

    /// Check if player has finished
    pub fn is_finished(&self) -> bool {
        let state = *self.state.lock().unwrap();
        state == AudioPlayerState::Finished || state == AudioPlayerState::Stopped
    }

    /// Get current playback position in seconds
    pub fn position_seconds(&self) -> f64 {
        if let Some(ref audio) = self.audio {
            let position = *self.position.lock().unwrap();
            let samples_per_sec = audio.format.sample_rate as f64;
            position as f64 / samples_per_sec
        } else {
            0.0
        }
    }

    /// Get total duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        self.audio.as_ref().map(|a| a.duration()).unwrap_or(0.0)
    }

    /// Get progress as percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let duration = self.duration_seconds();
        if duration > 0.0 {
            self.position_seconds() / duration
        } else {
            0.0
        }
    }

    /// Get next audio frame for RTP streaming
    /// Returns (samples, actual_samples_read) where samples is i16 PCM data
    /// Returns None if playback is not active or finished
    pub fn next_frame(&mut self) -> Option<(Vec<i16>, usize)> {
        // Check if we should return a frame
        {
            let state = *self.state.lock().unwrap();
            if state != AudioPlayerState::Playing {
                return None;
            }
        }

        let audio = self.audio.as_ref()?;

        // Calculate samples per frame
        let samples_per_frame =
            (audio.format.sample_rate as u32 * self.options.frame_duration_ms / 1000) as usize;

        // Get current position
        let mut position = self.position.lock().unwrap();
        let samples = audio.samples_i16();

        // Check if we've reached the end
        if *position >= samples.len() {
            if self.options.loop_playback {
                *position = 0; // Loop back to start
            } else {
                *self.state.lock().unwrap() = AudioPlayerState::Finished;
                return None;
            }
        }

        // Extract frame samples
        let end_pos = (*position + samples_per_frame).min(samples.len());
        let frame_samples = samples[*position..end_pos].to_vec();
        let actual_read = frame_samples.len();

        // Update position
        *position = end_pos;

        // Pace the playback (sleep if needed to maintain real-time)
        self.pace_playback();

        Some((frame_samples, actual_read))
    }

    /// Pace playback to maintain real-time streaming
    fn pace_playback(&self) {
        let mut last_time = self.last_frame_time.lock().unwrap();

        if let Some(last) = *last_time {
            let elapsed = last.elapsed();
            let target_duration = Duration::from_millis(self.options.frame_duration_ms as u64);

            if elapsed < target_duration {
                let sleep_duration = target_duration - elapsed;
                std::thread::sleep(sleep_duration);
            }
        }

        *last_time = Some(Instant::now());
    }

    /// Interrupt playback (e.g., on DTMF)
    /// Only works if allow_interrupt is true
    pub fn interrupt(&mut self) {
        if self.options.allow_interrupt {
            self.stop();
        }
    }

    /// Seek to position in seconds
    pub fn seek(&mut self, seconds: f64) {
        if let Some(ref audio) = self.audio {
            let target_sample = (seconds * audio.format.sample_rate as f64) as usize;
            let samples = audio.samples_i16();
            let clamped = target_sample.min(samples.len());
            *self.position.lock().unwrap() = clamped;
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming audio player that can be used in async contexts
/// This is a wrapper around AudioPlayer that provides async-friendly methods
#[derive(Clone)]
pub struct StreamingAudioPlayer {
    inner: Arc<Mutex<AudioPlayer>>,
}

impl StreamingAudioPlayer {
    /// Create new streaming audio player
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioPlayer::new())),
        }
    }

    /// Create with options
    pub fn with_options(options: PlaybackOptions) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioPlayer::with_options(options))),
        }
    }

    /// Load audio file
    pub fn load(&self, audio: Arc<WavFile>) {
        self.inner.lock().unwrap().load(audio);
    }

    /// Start playback
    pub fn play(&self) {
        self.inner.lock().unwrap().play();
    }

    /// Pause playback
    pub fn pause(&self) {
        self.inner.lock().unwrap().pause();
    }

    /// Stop playback
    pub fn stop(&self) {
        self.inner.lock().unwrap().stop();
    }

    /// Get current state
    pub fn state(&self) -> AudioPlayerState {
        self.inner.lock().unwrap().state()
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.inner.lock().unwrap().is_playing()
    }

    /// Check if finished
    pub fn is_finished(&self) -> bool {
        self.inner.lock().unwrap().is_finished()
    }

    /// Get next frame
    pub fn next_frame(&self) -> Option<(Vec<i16>, usize)> {
        self.inner.lock().unwrap().next_frame()
    }

    /// Interrupt playback
    pub fn interrupt(&self) {
        self.inner.lock().unwrap().interrupt();
    }

    /// Get playback progress
    pub fn progress(&self) -> f64 {
        self.inner.lock().unwrap().progress()
    }

    /// Seek to position
    pub fn seek(&self, seconds: f64) {
        self.inner.lock().unwrap().seek(seconds);
    }
}

impl Default for StreamingAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audio::wav::{WavFile, WavFormat};

    fn create_test_audio(duration_ms: u32) -> Arc<WavFile> {
        let sample_rate = 8000;
        let samples_count = (sample_rate * duration_ms / 1000) as usize;

        // Generate sine wave
        let mut data = Vec::with_capacity(samples_count * 2);
        for i in 0..samples_count {
            let t = i as f64 / sample_rate as f64;
            let freq = 440.0; // A4 note
            let amplitude = 0.5;
            let sample = (amplitude * (2.0 * std::f64::consts::PI * freq * t).sin() * 32767.0) as i16;
            data.extend_from_slice(&sample.to_le_bytes());
        }

        Arc::new(WavFile {
            format: WavFormat {
                channels: 1,
                sample_rate,
                bits_per_sample: 16,
                audio_format: 1,
            },
            data: Arc::new(data),
        })
    }

    #[test]
    fn test_player_initial_state() {
        let player = AudioPlayer::new();
        assert_eq!(player.state(), AudioPlayerState::Idle);
        assert!(!player.is_playing());
        assert!(!player.is_finished());
    }

    #[test]
    fn test_player_load_and_play() {
        let mut player = AudioPlayer::new();
        let audio = create_test_audio(100); // 100ms
        player.load(audio);

        assert_eq!(player.state(), AudioPlayerState::Idle);

        player.play();
        assert_eq!(player.state(), AudioPlayerState::Playing);
        assert!(player.is_playing());
    }

    #[test]
    fn test_player_pause_resume() {
        let mut player = AudioPlayer::new();
        let audio = create_test_audio(100);
        player.load(audio);

        player.play();
        assert_eq!(player.state(), AudioPlayerState::Playing);

        player.pause();
        assert_eq!(player.state(), AudioPlayerState::Paused);

        player.play();
        assert_eq!(player.state(), AudioPlayerState::Playing);
    }

    #[test]
    fn test_player_stop() {
        let mut player = AudioPlayer::new();
        let audio = create_test_audio(100);
        player.load(audio);

        player.play();
        player.stop();

        assert_eq!(player.state(), AudioPlayerState::Stopped);
        assert!(player.is_finished());
        assert_eq!(player.position_seconds(), 0.0);
    }

    #[test]
    fn test_player_next_frame() {
        let mut player = AudioPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });
        let audio = create_test_audio(100);
        player.load(audio);

        // No frame when not playing
        assert!(player.next_frame().is_none());

        player.play();

        // Should get frames now
        let frame = player.next_frame();
        assert!(frame.is_some());

        let (samples, count) = frame.unwrap();
        // 20ms at 8kHz = 160 samples
        assert_eq!(count, 160);
        assert_eq!(samples.len(), 160);
    }

    #[test]
    fn test_player_duration() {
        let mut player = AudioPlayer::new();
        let audio = create_test_audio(500); // 500ms = 0.5s
        player.load(audio);

        let duration = player.duration_seconds();
        assert!((duration - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_player_progress() {
        let mut player = AudioPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });
        let audio = create_test_audio(100); // 100ms
        player.load(audio);

        assert_eq!(player.progress(), 0.0);

        player.play();

        // Read one frame
        player.next_frame();

        let progress = player.progress();
        // 20ms out of 100ms = 0.2
        assert!((progress - 0.2).abs() < 0.05);
    }

    #[test]
    fn test_player_loop() {
        let mut player = AudioPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 50, // Use larger frames to finish quickly
            loop_playback: true,
            allow_interrupt: true,
        });
        let audio = create_test_audio(100); // 100ms
        player.load(audio);

        player.play();

        // Read enough frames to exceed audio duration
        for _ in 0..5 {
            let frame = player.next_frame();
            assert!(frame.is_some());
        }

        // Should still be playing due to loop
        assert_eq!(player.state(), AudioPlayerState::Playing);
    }

    #[test]
    fn test_player_interrupt() {
        let mut player = AudioPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });
        let audio = create_test_audio(100);
        player.load(audio);

        player.play();
        assert!(player.is_playing());

        player.interrupt();
        assert!(player.is_finished());
        assert_eq!(player.state(), AudioPlayerState::Stopped);
    }

    #[test]
    fn test_streaming_player() {
        let player = StreamingAudioPlayer::new();
        let audio = create_test_audio(100);

        player.load(audio);
        assert_eq!(player.state(), AudioPlayerState::Idle);

        player.play();
        assert!(player.is_playing());

        player.pause();
        assert_eq!(player.state(), AudioPlayerState::Paused);

        player.stop();
        assert!(player.is_finished());
    }
}

/// Sequential audio playback for playing multiple files
use crate::domain::audio::player::{AudioPlayer, AudioPlayerState, PlaybackOptions};
use crate::domain::audio::wav::WavFile;
use std::collections::VecDeque;
use std::sync::Arc;

/// Sequential audio player
/// Plays a sequence of audio files one after another
pub struct SequentialPlayer {
    /// Queue of audio files to play
    queue: VecDeque<Arc<WavFile>>,
    /// Current player
    player: AudioPlayer,
    /// Playback options
    options: PlaybackOptions,
}

impl SequentialPlayer {
    /// Create new sequential player
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            player: AudioPlayer::new(),
            options: PlaybackOptions::default(),
        }
    }

    /// Create with options
    pub fn with_options(options: PlaybackOptions) -> Self {
        Self {
            queue: VecDeque::new(),
            player: AudioPlayer::with_options(options.clone()),
            options,
        }
    }

    /// Add audio file to the end of the queue
    pub fn enqueue(&mut self, audio: Arc<WavFile>) {
        self.queue.push_back(audio);
    }

    /// Add audio file to the front of the queue
    pub fn enqueue_front(&mut self, audio: Arc<WavFile>) {
        self.queue.push_front(audio);
    }

    /// Add multiple audio files to the queue
    pub fn enqueue_all(&mut self, audio_files: Vec<Arc<WavFile>>) {
        for audio in audio_files {
            self.queue.push_back(audio);
        }
    }

    /// Clear the queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Get number of files in queue
    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        // If player is idle or finished, load next audio
        if self.player.state() == AudioPlayerState::Idle
            || self.player.state() == AudioPlayerState::Finished
            || self.player.state() == AudioPlayerState::Stopped
        {
            if let Some(audio) = self.queue.pop_front() {
                self.player.load(audio);
                self.player.play();
            }
        } else {
            // Resume current playback
            self.player.play();
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.player.pause();
    }

    /// Stop playback and clear queue
    pub fn stop(&mut self) {
        self.player.stop();
        self.queue.clear();
    }

    /// Skip to next audio file
    pub fn skip(&mut self) {
        self.player.stop();
        self.play(); // Will load next audio
    }

    /// Get current player state
    pub fn state(&self) -> AudioPlayerState {
        self.player.state()
    }

    /// Check if sequence is playing
    pub fn is_playing(&self) -> bool {
        self.player.is_playing() || !self.queue.is_empty()
    }

    /// Check if sequence has finished (no more audio)
    pub fn is_finished(&self) -> bool {
        self.player.is_finished() && self.queue.is_empty()
    }

    /// Get next audio frame
    /// Automatically advances to next audio file when current one finishes
    pub fn next_frame(&mut self) -> Option<(Vec<i16>, usize)> {
        // Try to get frame from current player
        if let Some(frame) = self.player.next_frame() {
            return Some(frame);
        }

        // Current audio finished, check if we have more
        if self.player.state() == AudioPlayerState::Finished {
            if let Some(next_audio) = self.queue.pop_front() {
                self.player.load(next_audio);
                self.player.play();
                return self.player.next_frame();
            }
        }

        None
    }

    /// Interrupt playback
    pub fn interrupt(&mut self) {
        self.player.interrupt();
        self.queue.clear();
    }

    /// Get progress of current audio (0.0 to 1.0)
    pub fn current_progress(&self) -> f64 {
        self.player.progress()
    }

    /// Get overall progress including queue (0.0 to 1.0)
    pub fn overall_progress(&self) -> f64 {
        if self.is_finished() {
            return 1.0;
        }

        // This is approximate since we don't know duration of queued files
        let total_items = self.queue.len() + 1; // +1 for current
        let completed_items = 1.0 - ((self.queue.len() + 1) as f64 / total_items as f64);
        let current_progress = self.player.progress() / total_items as f64;

        completed_items + current_progress
    }
}

impl Default for SequentialPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for sequential player
pub struct SequenceBuilder {
    files: Vec<Arc<WavFile>>,
    options: PlaybackOptions,
}

impl SequenceBuilder {
    /// Create new sequence builder
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            options: PlaybackOptions::default(),
        }
    }

    /// Add audio file to sequence
    pub fn add(mut self, audio: Arc<WavFile>) -> Self {
        self.files.push(audio);
        self
    }

    /// Set playback options
    pub fn options(mut self, options: PlaybackOptions) -> Self {
        self.options = options;
        self
    }

    /// Set frame duration
    pub fn frame_duration_ms(mut self, ms: u32) -> Self {
        self.options.frame_duration_ms = ms;
        self
    }

    /// Enable loop playback for the sequence
    pub fn loop_playback(mut self, loop_enabled: bool) -> Self {
        self.options.loop_playback = loop_enabled;
        self
    }

    /// Set interrupt on DTMF
    pub fn allow_interrupt(mut self, allow: bool) -> Self {
        self.options.allow_interrupt = allow;
        self
    }

    /// Build the sequential player
    pub fn build(self) -> SequentialPlayer {
        let mut player = SequentialPlayer::with_options(self.options);
        player.enqueue_all(self.files);
        player
    }
}

impl Default for SequenceBuilder {
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
        let mut data = Vec::with_capacity(samples_count * 2);

        for _ in 0..samples_count {
            let sample = 100i16;
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
    fn test_sequential_player_creation() {
        let player = SequentialPlayer::new();
        assert_eq!(player.queue_length(), 0);
        assert!(player.is_queue_empty());
    }

    #[test]
    fn test_enqueue() {
        let mut player = SequentialPlayer::new();
        let audio1 = create_test_audio(100);
        let audio2 = create_test_audio(200);

        player.enqueue(audio1);
        player.enqueue(audio2);

        assert_eq!(player.queue_length(), 2);
        assert!(!player.is_queue_empty());
    }

    #[test]
    fn test_enqueue_front() {
        let mut player = SequentialPlayer::new();
        let audio1 = create_test_audio(100);
        let audio2 = create_test_audio(200);

        player.enqueue(audio1);
        player.enqueue_front(audio2); // Should be played first

        assert_eq!(player.queue_length(), 2);
    }

    #[test]
    fn test_play_sequence() {
        let mut player = SequentialPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });

        let audio1 = create_test_audio(50);
        let audio2 = create_test_audio(50);

        player.enqueue(audio1);
        player.enqueue(audio2);

        player.play();
        assert!(player.is_playing());

        // Should get frames
        let frame = player.next_frame();
        assert!(frame.is_some());
    }

    #[test]
    fn test_clear_queue() {
        let mut player = SequentialPlayer::new();
        player.enqueue(create_test_audio(100));
        player.enqueue(create_test_audio(200));

        assert_eq!(player.queue_length(), 2);

        player.clear_queue();
        assert_eq!(player.queue_length(), 0);
        assert!(player.is_queue_empty());
    }

    #[test]
    fn test_skip() {
        let mut player = SequentialPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });

        player.enqueue(create_test_audio(100));
        player.enqueue(create_test_audio(100));

        player.play();
        assert_eq!(player.queue_length(), 1); // One loaded, one in queue

        player.skip();
        // Should have advanced to next audio
        assert!(player.is_playing() || player.queue_length() == 0);
    }

    #[test]
    fn test_stop() {
        let mut player = SequentialPlayer::new();
        player.enqueue(create_test_audio(100));
        player.enqueue(create_test_audio(200));

        player.play();
        player.stop();

        assert_eq!(player.queue_length(), 0);
        assert!(player.is_finished());
    }

    #[test]
    fn test_sequence_builder() {
        let audio1 = create_test_audio(100);
        let audio2 = create_test_audio(200);

        let player = SequenceBuilder::new()
            .add(audio1)
            .add(audio2)
            .frame_duration_ms(20)
            .allow_interrupt(true)
            .build();

        assert_eq!(player.queue_length(), 2);
    }

    #[test]
    fn test_interrupt() {
        let mut player = SequentialPlayer::with_options(PlaybackOptions {
            frame_duration_ms: 20,
            loop_playback: false,
            allow_interrupt: true,
        });

        player.enqueue(create_test_audio(100));
        player.enqueue(create_test_audio(100));

        player.play();
        assert!(player.is_playing());

        player.interrupt();
        assert!(player.is_finished());
        assert_eq!(player.queue_length(), 0);
    }
}

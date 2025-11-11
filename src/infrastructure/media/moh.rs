//! Music on Hold (MOH) implementation
//!
//! Provides audio playback for callers on hold

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Music on Hold state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MohState {
    /// MOH is not playing
    Idle,
    /// MOH is playing
    Playing,
}

/// Music on Hold configuration
#[derive(Debug, Clone)]
pub struct MohConfig {
    /// MOH audio source (file path or URL)
    pub source: String,
    /// Loop the audio (default: true)
    pub loop_audio: bool,
    /// Volume (0.0 - 1.0, default: 0.7)
    pub volume: f32,
}

impl Default for MohConfig {
    fn default() -> Self {
        Self {
            source: "moh/default.wav".to_string(),
            loop_audio: true,
            volume: 0.7,
        }
    }
}

/// Music on Hold player
pub struct MohPlayer {
    config: MohConfig,
    state: Arc<RwLock<MohState>>,
}

impl MohPlayer {
    /// Create new MOH player with default configuration
    pub fn new() -> Self {
        Self {
            config: MohConfig::default(),
            state: Arc::new(RwLock::new(MohState::Idle)),
        }
    }

    /// Create MOH player with custom configuration
    pub fn with_config(config: MohConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(MohState::Idle)),
        }
    }

    /// Start playing music on hold
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.write().await;

        if *state == MohState::Playing {
            return Err("MOH is already playing".to_string());
        }

        // TODO: Implement actual audio playback
        // This would involve:
        // 1. Loading the audio file from config.source
        // 2. Decoding the audio (WAV, MP3, etc.)
        // 3. Generating RTP packets with the audio data
        // 4. Sending to the media stream

        info!("Starting MOH playback from: {}", self.config.source);
        *state = MohState::Playing;

        Ok(())
    }

    /// Stop playing music on hold
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.write().await;

        if *state == MohState::Idle {
            return Ok(()); // Already stopped
        }

        // TODO: Implement actual audio stop
        // This would stop the playback task and cleanup resources

        info!("Stopping MOH playback");
        *state = MohState::Idle;

        Ok(())
    }

    /// Check if MOH is playing
    pub async fn is_playing(&self) -> bool {
        *self.state.read().await == MohState::Playing
    }

    /// Get current state
    pub async fn state(&self) -> MohState {
        self.state.read().await.clone()
    }
}

impl Default for MohPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple tone generator for MOH (as fallback)
/// Generates a simple sine wave tone
pub struct ToneGenerator {
    frequency: f32,
    sample_rate: u32,
    amplitude: f32,
    phase: Arc<RwLock<f32>>,
}

impl ToneGenerator {
    /// Create new tone generator
    ///
    /// # Arguments
    /// * `frequency` - Tone frequency in Hz (default: 440.0 Hz - A4 note)
    /// * `sample_rate` - Audio sample rate (default: 8000 Hz for telephony)
    /// * `amplitude` - Amplitude 0.0-1.0 (default: 0.3)
    pub fn new(frequency: f32, sample_rate: u32, amplitude: f32) -> Self {
        Self {
            frequency,
            sample_rate,
            amplitude,
            phase: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Create default tone generator (440 Hz, 8000 Hz sample rate, 0.3 amplitude)
    pub fn default_tone() -> Self {
        Self::new(440.0, 8000, 0.3)
    }

    /// Generate next audio sample
    pub async fn next_sample(&self) -> i16 {
        let mut phase = self.phase.write().await;

        // Calculate sine wave sample
        let sample = (self.amplitude * (*phase * 2.0 * std::f32::consts::PI).sin()) as f32;

        // Convert to 16-bit PCM
        let pcm_sample = (sample * 32767.0) as i16;

        // Increment phase
        *phase += self.frequency / self.sample_rate as f32;
        if *phase >= 1.0 {
            *phase -= 1.0;
        }

        pcm_sample
    }

    /// Generate a buffer of samples
    pub async fn generate_samples(&self, count: usize) -> Vec<i16> {
        let mut samples = Vec::with_capacity(count);
        for _ in 0..count {
            samples.push(self.next_sample().await);
        }
        samples
    }

    /// Reset tone generator phase
    pub async fn reset(&self) {
        let mut phase = self.phase.write().await;
        *phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_moh_player_creation() {
        let player = MohPlayer::new();
        assert_eq!(player.state().await, MohState::Idle);
    }

    #[tokio::test]
    async fn test_moh_start_stop() {
        let player = MohPlayer::new();

        // Start MOH
        player.start().await.unwrap();
        assert_eq!(player.state().await, MohState::Playing);
        assert!(player.is_playing().await);

        // Try to start again (should fail)
        assert!(player.start().await.is_err());

        // Stop MOH
        player.stop().await.unwrap();
        assert_eq!(player.state().await, MohState::Idle);
        assert!(!player.is_playing().await);
    }

    #[tokio::test]
    async fn test_tone_generator() {
        let gen = ToneGenerator::default_tone();

        // Generate some samples
        let samples = gen.generate_samples(100).await;
        assert_eq!(samples.len(), 100);

        // Samples should be in valid range
        for sample in samples {
            assert!(sample >= -32768 && sample <= 32767);
        }
    }

    #[tokio::test]
    async fn test_tone_generator_reset() {
        let gen = ToneGenerator::new(440.0, 8000, 0.5);

        // Generate some samples
        gen.generate_samples(100).await;

        // Reset
        gen.reset().await;

        // Phase should be reset to 0
        let phase = gen.phase.read().await;
        assert!(*phase == 0.0);
    }

    #[tokio::test]
    async fn test_moh_config_default() {
        let config = MohConfig::default();
        assert_eq!(config.source, "moh/default.wav");
        assert!(config.loop_audio);
        assert_eq!(config.volume, 0.7);
    }

    #[tokio::test]
    async fn test_moh_with_config() {
        let config = MohConfig {
            source: "custom.wav".to_string(),
            loop_audio: false,
            volume: 0.5,
        };

        let player = MohPlayer::with_config(config.clone());
        assert_eq!(player.config.source, "custom.wav");
        assert!(!player.config.loop_audio);
        assert_eq!(player.config.volume, 0.5);
    }
}

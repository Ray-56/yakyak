/// Audio mixer for conference calls
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Audio sample format
pub type AudioSample = i16;

/// Audio frame (collection of samples)
#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: Vec<AudioSample>,
    pub sample_rate: u32,
    pub channels: u8,
    pub timestamp: u64,
}

impl AudioFrame {
    pub fn new(samples: Vec<AudioSample>, sample_rate: u32, channels: u8, timestamp: u64) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            timestamp,
        }
    }

    /// Get frame duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        (self.samples.len() as u32 * 1000) / (self.sample_rate * self.channels as u32)
    }

    /// Get frame length in samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if frame is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

/// Participant audio stream
#[derive(Debug)]
pub struct ParticipantStream {
    pub participant_id: Uuid,
    pub is_muted: bool,
    pub gain: f32, // Volume gain (0.0 to 2.0, 1.0 = normal)
}

impl ParticipantStream {
    pub fn new(participant_id: Uuid) -> Self {
        Self {
            participant_id,
            is_muted: false,
            gain: 1.0,
        }
    }

    /// Mute stream
    pub fn mute(&mut self) {
        self.is_muted = true;
    }

    /// Unmute stream
    pub fn unmute(&mut self) {
        self.is_muted = false;
    }

    /// Set gain (volume)
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 2.0);
    }
}

/// Audio mixer for combining multiple audio streams
pub struct AudioMixer {
    streams: Arc<RwLock<HashMap<Uuid, ParticipantStream>>>,
    sample_rate: u32,
    channels: u8,
}

impl AudioMixer {
    /// Create new audio mixer
    pub fn new(sample_rate: u32, channels: u8) -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            sample_rate,
            channels,
        }
    }

    /// Add participant stream
    pub async fn add_stream(&self, participant_id: Uuid) {
        let stream = ParticipantStream::new(participant_id);
        let mut streams = self.streams.write().await;
        streams.insert(participant_id, stream);
        info!("Added audio stream for participant {}", participant_id);
    }

    /// Remove participant stream
    pub async fn remove_stream(&self, participant_id: Uuid) {
        let mut streams = self.streams.write().await;
        streams.remove(&participant_id);
        info!("Removed audio stream for participant {}", participant_id);
    }

    /// Mute participant
    pub async fn mute_participant(&self, participant_id: Uuid) -> Result<(), String> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(&participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;
        stream.mute();
        Ok(())
    }

    /// Unmute participant
    pub async fn unmute_participant(&self, participant_id: Uuid) -> Result<(), String> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(&participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;
        stream.unmute();
        Ok(())
    }

    /// Set participant gain
    pub async fn set_participant_gain(&self, participant_id: Uuid, gain: f32) -> Result<(), String> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(&participant_id)
            .ok_or_else(|| "Participant not found".to_string())?;
        stream.set_gain(gain);
        Ok(())
    }

    /// Mix audio frames from multiple participants
    /// Excludes the specified participant from the mix (for their own audio)
    pub async fn mix_frames(
        &self,
        frames: Vec<(Uuid, AudioFrame)>,
        exclude_participant: Option<Uuid>,
    ) -> AudioFrame {
        let streams = self.streams.read().await;

        // Find maximum frame length
        let max_len = frames.iter().map(|(_, f)| f.len()).max().unwrap_or(0);

        if max_len == 0 {
            return AudioFrame::new(Vec::new(), self.sample_rate, self.channels, 0);
        }

        // Initialize output buffer
        let mut mixed_samples = vec![0i32; max_len];

        // Mix all frames
        for (participant_id, frame) in frames.iter() {
            // Skip excluded participant
            if Some(*participant_id) == exclude_participant {
                continue;
            }

            // Get stream info
            let stream = match streams.get(participant_id) {
                Some(s) => s,
                None => continue,
            };

            // Skip if muted
            if stream.is_muted {
                continue;
            }

            // Mix samples with gain
            for (i, sample) in frame.samples.iter().enumerate() {
                if i < max_len {
                    let scaled = (*sample as f32 * stream.gain) as i32;
                    mixed_samples[i] += scaled;
                }
            }
        }

        // Clamp and convert to i16
        let output_samples: Vec<AudioSample> = mixed_samples
            .iter()
            .map(|&sample| sample.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
            .collect();

        AudioFrame::new(output_samples, self.sample_rate, self.channels, 0)
    }

    /// Simple mix without participant filtering
    pub async fn mix_simple(&self, frames: Vec<AudioFrame>) -> AudioFrame {
        let frames_with_ids: Vec<(Uuid, AudioFrame)> = frames
            .into_iter()
            .map(|f| (Uuid::new_v4(), f))
            .collect();

        self.mix_frames(frames_with_ids, None).await
    }

    /// Get active stream count
    pub async fn stream_count(&self) -> usize {
        let streams = self.streams.read().await;
        streams.len()
    }

    /// Get unmuted stream count
    pub async fn active_stream_count(&self) -> usize {
        let streams = self.streams.read().await;
        streams.values().filter(|s| !s.is_muted).count()
    }
}

/// Automatic Gain Control (AGC) for audio normalization
pub struct AutomaticGainControl {
    target_level: f32,
    current_gain: f32,
    attack_rate: f32,
    release_rate: f32,
}

impl AutomaticGainControl {
    pub fn new(target_level: f32) -> Self {
        Self {
            target_level,
            current_gain: 1.0,
            attack_rate: 0.1,   // Fast attack
            release_rate: 0.01, // Slow release
        }
    }

    /// Process audio frame with AGC
    pub fn process(&mut self, frame: &mut AudioFrame) {
        if frame.is_empty() {
            return;
        }

        // Calculate RMS level
        let sum_squares: i64 = frame.samples.iter()
            .map(|&s| (s as i64) * (s as i64))
            .sum();
        let rms = (sum_squares as f32 / frame.samples.len() as f32).sqrt();

        // Calculate target gain
        let target_gain = if rms > 0.0 {
            self.target_level / rms
        } else {
            1.0
        };

        // Smoothly adjust gain
        if target_gain > self.current_gain {
            // Release (slow increase)
            self.current_gain += (target_gain - self.current_gain) * self.release_rate;
        } else {
            // Attack (fast decrease)
            self.current_gain += (target_gain - self.current_gain) * self.attack_rate;
        }

        // Clamp gain
        self.current_gain = self.current_gain.clamp(0.1, 10.0);

        // Apply gain
        for sample in frame.samples.iter_mut() {
            let adjusted = (*sample as f32 * self.current_gain) as i32;
            *sample = adjusted.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
    }

    /// Get current gain
    pub fn current_gain(&self) -> f32 {
        self.current_gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_frame_creation() {
        let samples = vec![100, 200, 300, 400];
        let frame = AudioFrame::new(samples.clone(), 8000, 1, 0);

        assert_eq!(frame.samples, samples);
        assert_eq!(frame.sample_rate, 8000);
        assert_eq!(frame.channels, 1);
        assert_eq!(frame.len(), 4);
    }

    #[test]
    fn test_audio_frame_duration() {
        let samples = vec![0; 160]; // 20ms at 8000Hz
        let frame = AudioFrame::new(samples, 8000, 1, 0);

        assert_eq!(frame.duration_ms(), 20);
    }

    #[test]
    fn test_participant_stream() {
        let mut stream = ParticipantStream::new(Uuid::new_v4());

        assert!(!stream.is_muted);
        assert_eq!(stream.gain, 1.0);

        stream.mute();
        assert!(stream.is_muted);

        stream.unmute();
        assert!(!stream.is_muted);

        stream.set_gain(1.5);
        assert_eq!(stream.gain, 1.5);

        // Test clamping
        stream.set_gain(3.0);
        assert_eq!(stream.gain, 2.0);
    }

    #[tokio::test]
    async fn test_mixer_add_remove_stream() {
        let mixer = AudioMixer::new(8000, 1);

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        mixer.add_stream(p1).await;
        mixer.add_stream(p2).await;

        assert_eq!(mixer.stream_count().await, 2);

        mixer.remove_stream(p1).await;
        assert_eq!(mixer.stream_count().await, 1);
    }

    #[tokio::test]
    async fn test_mixer_mute() {
        let mixer = AudioMixer::new(8000, 1);
        let p1 = Uuid::new_v4();

        mixer.add_stream(p1).await;
        assert_eq!(mixer.active_stream_count().await, 1);

        mixer.mute_participant(p1).await.unwrap();
        assert_eq!(mixer.active_stream_count().await, 0);

        mixer.unmute_participant(p1).await.unwrap();
        assert_eq!(mixer.active_stream_count().await, 1);
    }

    #[tokio::test]
    async fn test_mix_frames() {
        let mixer = AudioMixer::new(8000, 1);

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        mixer.add_stream(p1).await;
        mixer.add_stream(p2).await;

        // Create test frames
        let frame1 = AudioFrame::new(vec![100, 200, 300], 8000, 1, 0);
        let frame2 = AudioFrame::new(vec![50, 100, 150], 8000, 1, 0);

        let frames = vec![(p1, frame1), (p2, frame2)];

        let mixed = mixer.mix_frames(frames, None).await;

        // Expected: [150, 300, 450]
        assert_eq!(mixed.samples, vec![150, 300, 450]);
    }

    #[tokio::test]
    async fn test_mix_excludes_participant() {
        let mixer = AudioMixer::new(8000, 1);

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        mixer.add_stream(p1).await;
        mixer.add_stream(p2).await;

        let frame1 = AudioFrame::new(vec![100, 200, 300], 8000, 1, 0);
        let frame2 = AudioFrame::new(vec![50, 100, 150], 8000, 1, 0);

        let frames = vec![(p1, frame1), (p2, frame2)];

        // Exclude p1, should only get p2's audio
        let mixed = mixer.mix_frames(frames, Some(p1)).await;

        assert_eq!(mixed.samples, vec![50, 100, 150]);
    }

    #[test]
    fn test_agc_process() {
        let mut agc = AutomaticGainControl::new(1000.0);

        let mut frame = AudioFrame::new(vec![100, 200, 300], 8000, 1, 0);

        agc.process(&mut frame);

        // Samples should be amplified
        assert!(frame.samples[0] > 100);
        assert!(frame.samples[1] > 200);
        assert!(frame.samples[2] > 300);
    }

    #[test]
    fn test_mix_with_gain() {
        // This would be an async test in real usage
        // Testing the concept of gain mixing
        let sample1 = 100i16;
        let sample2 = 200i16;
        let gain1 = 1.5f32;
        let gain2 = 0.5f32;

        let mixed = ((sample1 as f32 * gain1) as i32 + (sample2 as f32 * gain2) as i32)
            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        assert_eq!(mixed, 250); // 150 + 100
    }
}

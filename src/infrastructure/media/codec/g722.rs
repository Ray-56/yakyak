//! G.722 Wideband Audio Codec Implementation
//!
//! G.722 is a wideband speech codec operating at 48, 56, and 64 kbit/s.
//! It provides 7 kHz audio bandwidth (50-7000 Hz) sampled at 16 kHz.
//! Standardized by ITU-T as G.722.
//!
//! Key Features:
//! - Wideband audio (50-7000 Hz)
//! - 16 kHz sampling rate
//! - 64 kbps bitrate (also 56 and 48 kbps modes)
//! - Sub-band ADPCM (SB-ADPCM) encoding
//! - Low complexity
//! - Widely supported in VoIP systems

/// G.722 codec configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G722Config {
    /// Sample rate (always 16000 Hz for G.722)
    pub sample_rate: u32,
    /// Number of channels (always 1 for G.722)
    pub channels: u8,
    /// Bitrate mode
    pub bitrate_mode: G722BitrateMode,
}

impl Default for G722Config {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bitrate_mode: G722BitrateMode::Mode64kbps,
        }
    }
}

impl G722Config {
    /// Create a new G.722 configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration for 48 kbps mode
    pub fn mode_48kbps() -> Self {
        Self {
            bitrate_mode: G722BitrateMode::Mode48kbps,
            ..Default::default()
        }
    }

    /// Create configuration for 56 kbps mode
    pub fn mode_56kbps() -> Self {
        Self {
            bitrate_mode: G722BitrateMode::Mode56kbps,
            ..Default::default()
        }
    }

    /// Create configuration for 64 kbps mode (default)
    pub fn mode_64kbps() -> Self {
        Self {
            bitrate_mode: G722BitrateMode::Mode64kbps,
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_rate != 16000 {
            return Err(format!(
                "Invalid sample rate for G.722: {} (must be 16000)",
                self.sample_rate
            ));
        }

        if self.channels != 1 {
            return Err(format!(
                "Invalid channels for G.722: {} (must be 1)",
                self.channels
            ));
        }

        Ok(())
    }

    /// Get bitrate in bits per second
    pub fn bitrate(&self) -> u32 {
        match self.bitrate_mode {
            G722BitrateMode::Mode48kbps => 48000,
            G722BitrateMode::Mode56kbps => 56000,
            G722BitrateMode::Mode64kbps => 64000,
        }
    }

    /// Calculate encoded size for given number of samples
    pub fn encoded_size(&self, samples: usize) -> usize {
        // G.722 encodes 2 samples per byte (at 64 kbps)
        // At 16kHz, 20ms = 320 samples = 160 bytes
        samples / 2
    }
}

/// G.722 bitrate mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum G722BitrateMode {
    /// 48 kbps mode (6 bits lower sub-band, 0 bits upper sub-band)
    Mode48kbps,
    /// 56 kbps mode (6 bits lower, 1 bit upper)
    Mode56kbps,
    /// 64 kbps mode (6 bits lower, 2 bits upper) - default and most common
    Mode64kbps,
}

/// G.722 encoder state
pub struct G722Encoder {
    config: G722Config,
    // Internal state for SB-ADPCM encoding
    band1: SubbandState,
    band2: SubbandState,
    // QMF filter state
    x: [i32; 24],
}

impl G722Encoder {
    /// Create a new G.722 encoder
    pub fn new(config: G722Config) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            config,
            band1: SubbandState::new(),
            band2: SubbandState::new(),
            x: [0; 24],
        })
    }

    /// Encode PCM samples to G.722
    ///
    /// # Arguments
    /// * `pcm` - Input PCM samples (16-bit signed, 16 kHz)
    /// * `output` - Output buffer for G.722 encoded data
    ///
    /// # Returns
    /// Number of bytes written to output buffer
    ///
    /// Note: This is a simplified implementation. Production code should use
    /// optimized G.722 library (e.g., spandsp or g722_1 crate)
    pub fn encode(&mut self, pcm: &[i16], output: &mut [u8]) -> Result<usize, String> {
        if pcm.len() % 2 != 0 {
            return Err("PCM input must have even number of samples".to_string());
        }

        let encoded_size = self.config.encoded_size(pcm.len());
        if output.len() < encoded_size {
            return Err(format!(
                "Output buffer too small: {} < {}",
                output.len(),
                encoded_size
            ));
        }

        // Simplified encoding (placeholder)
        // Real implementation would:
        // 1. Split signal into two sub-bands using QMF
        // 2. Encode lower sub-band (6 bits) and upper sub-band (0-2 bits)
        // 3. Combine into output bytes

        for (i, chunk) in pcm.chunks(2).enumerate() {
            if i >= encoded_size {
                break;
            }
            // Placeholder: Simple downsampling without proper filtering
            let lower = self.encode_subband(&chunk[0], &mut self.band1);
            let upper = if chunk.len() > 1 {
                self.encode_subband(&chunk[1], &mut self.band2)
            } else {
                0
            };

            output[i] = match self.config.bitrate_mode {
                G722BitrateMode::Mode48kbps => (lower & 0x3F) << 2,
                G722BitrateMode::Mode56kbps => ((lower & 0x3F) << 2) | ((upper & 0x01) << 1),
                G722BitrateMode::Mode64kbps => ((lower & 0x3F) << 2) | (upper & 0x03),
            };
        }

        Ok(encoded_size)
    }

    fn encode_subband(&self, sample: &i16, _state: &SubbandState) -> u8 {
        // Simplified ADPCM encoding
        // Real implementation would use proper ADPCM quantization tables
        ((*sample as i32 >> 9) & 0x3F) as u8
    }

    /// Get encoder configuration
    pub fn config(&self) -> &G722Config {
        &self.config
    }

    /// Reset encoder state
    pub fn reset(&mut self) {
        self.band1 = SubbandState::new();
        self.band2 = SubbandState::new();
        self.x = [0; 24];
    }
}

/// G.722 decoder state
pub struct G722Decoder {
    config: G722Config,
    // Internal state for SB-ADPCM decoding
    band1: SubbandState,
    band2: SubbandState,
    // QMF filter state
    y: [i32; 24],
}

impl G722Decoder {
    /// Create a new G.722 decoder
    pub fn new(config: G722Config) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            config,
            band1: SubbandState::new(),
            band2: SubbandState::new(),
            y: [0; 24],
        })
    }

    /// Decode G.722 to PCM samples
    ///
    /// # Arguments
    /// * `encoded` - G.722 encoded data
    /// * `output` - Output buffer for PCM samples (16-bit signed, 16 kHz)
    ///
    /// # Returns
    /// Number of samples decoded
    ///
    /// Note: This is a simplified implementation. Production code should use
    /// optimized G.722 library (e.g., spandsp or g722_1 crate)
    pub fn decode(&mut self, encoded: &[u8], output: &mut [i16]) -> Result<usize, String> {
        let samples_needed = encoded.len() * 2;
        if output.len() < samples_needed {
            return Err(format!(
                "Output buffer too small: {} < {}",
                output.len(),
                samples_needed
            ));
        }

        // Simplified decoding (placeholder)
        // Real implementation would:
        // 1. Extract lower and upper sub-band codes
        // 2. Decode each sub-band using ADPCM
        // 3. Combine sub-bands using QMF synthesis

        for (i, &byte) in encoded.iter().enumerate() {
            let lower = (byte >> 2) & 0x3F;
            let upper = match self.config.bitrate_mode {
                G722BitrateMode::Mode48kbps => 0,
                G722BitrateMode::Mode56kbps => (byte >> 1) & 0x01,
                G722BitrateMode::Mode64kbps => byte & 0x03,
            };

            let idx = i * 2;
            output[idx] = self.decode_subband(lower, &mut self.band1);
            if idx + 1 < output.len() {
                output[idx + 1] = self.decode_subband(upper, &mut self.band2);
            }
        }

        Ok(samples_needed)
    }

    fn decode_subband(&self, code: u8, _state: &SubbandState) -> i16 {
        // Simplified ADPCM decoding
        // Real implementation would use proper ADPCM dequantization tables
        ((code as i16) << 9) - 16384
    }

    /// Get decoder configuration
    pub fn config(&self) -> &G722Config {
        &self.config
    }

    /// Reset decoder state
    pub fn reset(&mut self) {
        self.band1 = SubbandState::new();
        self.band2 = SubbandState::new();
        self.y = [0; 24];
    }
}

/// Sub-band ADPCM state
#[derive(Debug, Clone, Copy)]
struct SubbandState {
    // Quantizer state
    s: i32,
    // Scale factor
    scale: i32,
    // Adaptive predictor coefficients
    a: [i32; 2],
    b: [i32; 6],
    // Signal estimates
    d: [i32; 7],
    p: [i32; 3],
}

impl SubbandState {
    fn new() -> Self {
        Self {
            s: 0,
            scale: 0,
            a: [0; 2],
            b: [0; 6],
            d: [0; 7],
            p: [0; 3],
        }
    }
}

/// G.722 RTP payload
pub struct G722Payload<'a> {
    data: &'a [u8],
}

impl<'a> G722Payload<'a> {
    /// Parse a G.722 RTP payload
    pub fn parse(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Get encoded data
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Get number of samples this payload represents
    pub fn sample_count(&self) -> usize {
        // G.722 encodes 2 samples per byte
        self.data.len() * 2
    }

    /// Calculate duration in milliseconds (at 16 kHz)
    pub fn duration_ms(&self) -> f64 {
        (self.sample_count() as f64 / 16000.0) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g722_config_default() {
        let config = G722Config::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.bitrate_mode, G722BitrateMode::Mode64kbps);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_g722_config_modes() {
        let config48 = G722Config::mode_48kbps();
        assert_eq!(config48.bitrate(), 48000);

        let config56 = G722Config::mode_56kbps();
        assert_eq!(config56.bitrate(), 56000);

        let config64 = G722Config::mode_64kbps();
        assert_eq!(config64.bitrate(), 64000);
    }

    #[test]
    fn test_g722_config_validation() {
        let mut config = G722Config::default();
        assert!(config.validate().is_ok());

        config.sample_rate = 8000; // Invalid
        assert!(config.validate().is_err());

        config.sample_rate = 16000;
        config.channels = 2; // Invalid
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_g722_encoded_size() {
        let config = G722Config::default();
        assert_eq!(config.encoded_size(320), 160); // 20ms at 16kHz
        assert_eq!(config.encoded_size(160), 80);  // 10ms at 16kHz
    }

    #[test]
    fn test_g722_encoder_creation() {
        let config = G722Config::default();
        let encoder = G722Encoder::new(config);
        assert!(encoder.is_ok());

        let mut invalid_config = G722Config::default();
        invalid_config.sample_rate = 8000;
        let encoder = G722Encoder::new(invalid_config);
        assert!(encoder.is_err());
    }

    #[test]
    fn test_g722_decoder_creation() {
        let config = G722Config::default();
        let decoder = G722Decoder::new(config);
        assert!(decoder.is_ok());
    }

    #[test]
    fn test_g722_encode_decode() {
        let config = G722Config::default();
        let mut encoder = G722Encoder::new(config).unwrap();
        let mut decoder = G722Decoder::new(config).unwrap();

        // Test with 20ms of audio (320 samples at 16kHz)
        let pcm_input: Vec<i16> = (0..320).map(|i| (i * 100) as i16).collect();
        let mut encoded = vec![0u8; 160];
        let mut pcm_output = vec![0i16; 320];

        let encoded_size = encoder.encode(&pcm_input, &mut encoded).unwrap();
        assert_eq!(encoded_size, 160);

        let decoded_samples = decoder.decode(&encoded, &mut pcm_output).unwrap();
        assert_eq!(decoded_samples, 320);
    }

    #[test]
    fn test_g722_payload() {
        let data = vec![0u8; 160]; // 20ms of G.722 data
        let payload = G722Payload::parse(&data);

        assert_eq!(payload.sample_count(), 320);
        assert!((payload.duration_ms() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_g722_encoder_reset() {
        let config = G722Config::default();
        let mut encoder = G722Encoder::new(config).unwrap();

        let pcm: Vec<i16> = vec![100; 320];
        let mut encoded1 = vec![0u8; 160];
        let mut encoded2 = vec![0u8; 160];

        encoder.encode(&pcm, &mut encoded1).unwrap();
        encoder.reset();
        encoder.encode(&pcm, &mut encoded2).unwrap();

        // After reset, same input should produce same output
        assert_eq!(encoded1, encoded2);
    }

    #[test]
    fn test_subband_state_creation() {
        let state = SubbandState::new();
        assert_eq!(state.s, 0);
        assert_eq!(state.scale, 0);
    }
}

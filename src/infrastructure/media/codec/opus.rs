//! Opus Audio Codec Implementation
//!
//! Opus is a lossy audio coding format developed by the Xiph.Org Foundation
//! and standardized by IETF as RFC 6716. It's designed for interactive speech
//! and audio transmission over the Internet.
//!
//! Key Features:
//! - Variable bitrate (6-510 kbps)
//! - Multiple sampling rates (8, 12, 16, 24, 48 kHz)
//! - Low latency (2.5 to 60 ms)
//! - Excellent quality at low bitrates
//! - Built-in FEC (Forward Error Correction)
//! - Seamless switching between speech and music

use std::fmt;

/// Opus codec configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpusConfig {
    /// Sampling rate in Hz (8000, 12000, 16000, 24000, or 48000)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Bitrate in bits per second (6000-510000)
    pub bitrate: u32,
    /// Frame duration in milliseconds (2.5, 5, 10, 20, 40, 60)
    pub frame_duration_ms: u8,
    /// Application mode
    pub application: OpusApplication,
    /// Enable Forward Error Correction
    pub fec_enabled: bool,
    /// Enable Discontinuous Transmission
    pub dtx_enabled: bool,
    /// Complexity (0-10, higher = better quality but slower)
    pub complexity: u8,
}

impl Default for OpusConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bitrate: 64000,
            frame_duration_ms: 20,
            application: OpusApplication::Audio,
            fec_enabled: true,
            dtx_enabled: false,
            complexity: 10,
        }
    }
}

impl OpusConfig {
    /// Create config for VoIP usage
    pub fn voip() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bitrate: 24000,
            frame_duration_ms: 20,
            application: OpusApplication::Voip,
            fec_enabled: true,
            dtx_enabled: true,
            complexity: 5,
        }
    }

    /// Create config for audio usage
    pub fn audio() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bitrate: 64000,
            frame_duration_ms: 20,
            application: OpusApplication::Audio,
            fec_enabled: true,
            dtx_enabled: false,
            complexity: 10,
        }
    }

    /// Create config for low latency
    pub fn low_latency() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bitrate: 32000,
            frame_duration_ms: 10,
            application: OpusApplication::RestrictedLowdelay,
            fec_enabled: false,
            dtx_enabled: false,
            complexity: 5,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate sample rate
        match self.sample_rate {
            8000 | 12000 | 16000 | 24000 | 48000 => {}
            _ => return Err(format!("Invalid sample rate: {}", self.sample_rate)),
        }

        // Validate channels
        if self.channels < 1 || self.channels > 2 {
            return Err(format!("Invalid channels: {}", self.channels));
        }

        // Validate bitrate
        if self.bitrate < 6000 || self.bitrate > 510000 {
            return Err(format!("Invalid bitrate: {}", self.bitrate));
        }

        // Validate frame duration
        match self.frame_duration_ms {
            2 | 5 | 10 | 20 | 40 | 60 => {}
            _ => return Err(format!("Invalid frame duration: {}ms", self.frame_duration_ms)),
        }

        // Validate complexity
        if self.complexity > 10 {
            return Err(format!("Invalid complexity: {}", self.complexity));
        }

        Ok(())
    }

    /// Calculate frame size in samples
    pub fn frame_size(&self) -> usize {
        (self.sample_rate as usize * self.frame_duration_ms as usize) / 1000
    }

    /// Calculate max packet size in bytes
    pub fn max_packet_size(&self) -> usize {
        // Opus can produce up to 1275 bytes per frame
        1500
    }
}

/// Opus application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusApplication {
    /// Optimize for VoIP/speech
    Voip,
    /// Optimize for general audio
    Audio,
    /// Optimize for low delay
    RestrictedLowdelay,
}

impl fmt::Display for OpusApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpusApplication::Voip => write!(f, "voip"),
            OpusApplication::Audio => write!(f, "audio"),
            OpusApplication::RestrictedLowdelay => write!(f, "restricted-lowdelay"),
        }
    }
}

/// Opus encoder (placeholder for actual implementation)
///
/// Note: This is a framework. Actual encoding requires the opus-rs crate
/// or FFI bindings to libopus.
pub struct OpusEncoder {
    config: OpusConfig,
    // In real implementation: encoder state from opus-rs
}

impl OpusEncoder {
    /// Create a new Opus encoder
    pub fn new(config: OpusConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Encode PCM samples to Opus
    ///
    /// # Arguments
    /// * `pcm` - Input PCM samples (16-bit signed integers)
    /// * `output` - Output buffer for encoded data
    ///
    /// # Returns
    /// Number of bytes written to output buffer
    ///
    /// Note: This is a placeholder. Real implementation would use opus-rs:
    /// ```ignore
    /// use opus::Encoder;
    /// let mut encoder = Encoder::new(
    ///     config.sample_rate,
    ///     opus::Channels::from(config.channels),
    ///     config.application.into()
    /// )?;
    /// encoder.set_bitrate(config.bitrate)?;
    /// encoder.encode(&pcm, output)
    /// ```
    pub fn encode(&mut self, _pcm: &[i16], output: &mut [u8]) -> Result<usize, String> {
        // Placeholder: In real implementation, this would call libopus
        // For now, return error indicating library not linked
        Err("Opus encoding requires libopus (not yet integrated)".to_string())
    }

    /// Get encoder configuration
    pub fn config(&self) -> &OpusConfig {
        &self.config
    }

    /// Set bitrate dynamically
    pub fn set_bitrate(&mut self, bitrate: u32) {
        self.config.bitrate = bitrate;
    }

    /// Enable/disable FEC
    pub fn set_fec(&mut self, enabled: bool) {
        self.config.fec_enabled = enabled;
    }

    /// Enable/disable DTX
    pub fn set_dtx(&mut self, enabled: bool) {
        self.config.dtx_enabled = enabled;
    }
}

/// Opus decoder (placeholder for actual implementation)
pub struct OpusDecoder {
    config: OpusConfig,
    // In real implementation: decoder state from opus-rs
}

impl OpusDecoder {
    /// Create a new Opus decoder
    pub fn new(config: OpusConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Decode Opus to PCM samples
    ///
    /// # Arguments
    /// * `encoded` - Opus-encoded data
    /// * `output` - Output buffer for PCM samples (16-bit signed integers)
    /// * `fec` - Use Forward Error Correction to recover from packet loss
    ///
    /// # Returns
    /// Number of samples decoded
    ///
    /// Note: This is a placeholder. Real implementation would use opus-rs:
    /// ```ignore
    /// use opus::Decoder;
    /// let mut decoder = Decoder::new(
    ///     config.sample_rate,
    ///     opus::Channels::from(config.channels)
    /// )?;
    /// decoder.decode(&encoded, output, fec)
    /// ```
    pub fn decode(&mut self, _encoded: &[u8], _output: &mut [i16], _fec: bool) -> Result<usize, String> {
        // Placeholder: In real implementation, this would call libopus
        Err("Opus decoding requires libopus (not yet integrated)".to_string())
    }

    /// Decode with packet loss concealment
    ///
    /// When a packet is lost, this can generate audio to conceal the loss
    pub fn decode_plc(&mut self, output: &mut [i16]) -> Result<usize, String> {
        // Placeholder for packet loss concealment
        Err("Opus PLC requires libopus (not yet integrated)".to_string())
    }

    /// Get decoder configuration
    pub fn config(&self) -> &OpusConfig {
        &self.config
    }
}

/// Opus packet analyzer
pub struct OpusPacket<'a> {
    data: &'a [u8],
}

impl<'a> OpusPacket<'a> {
    /// Parse an Opus packet
    pub fn parse(data: &'a [u8]) -> Result<Self, String> {
        if data.is_empty() {
            return Err("Empty Opus packet".to_string());
        }
        Ok(Self { data })
    }

    /// Get TOC (Table of Contents) byte
    pub fn toc(&self) -> u8 {
        self.data[0]
    }

    /// Get configuration number (0-31)
    pub fn config(&self) -> u8 {
        (self.toc() >> 3) & 0x1F
    }

    /// Get stereo flag
    pub fn is_stereo(&self) -> bool {
        (self.toc() & 0x04) != 0
    }

    /// Get number of frames in packet (0-3)
    pub fn frame_count(&self) -> u8 {
        self.toc() & 0x03
    }

    /// Get bandwidth
    pub fn bandwidth(&self) -> OpusBandwidth {
        match self.config() {
            0..=11 => OpusBandwidth::Narrowband,
            12..=13 => OpusBandwidth::Mediumband,
            14..=15 => OpusBandwidth::Wideband,
            16..=19 => OpusBandwidth::SuperWideband,
            20..=31 => OpusBandwidth::Fullband,
            _ => OpusBandwidth::Narrowband,
        }
    }

    /// Estimate duration in samples
    pub fn duration_samples(&self, sample_rate: u32) -> usize {
        let frame_size = match self.config() {
            16..=19 => 480,  // 10ms at 48kHz
            20..=23 => 960,  // 20ms at 48kHz
            24..=27 => 1920, // 40ms at 48kHz
            28..=31 => 2880, // 60ms at 48kHz
            _ => 960,        // Default 20ms
        };

        let frames = match self.frame_count() {
            0 => 1,
            1 => 2,
            2 => 2,
            3 => 3, // Actually means "arbitrary number", but use 3 as estimate
            _ => 1,
        };

        (frame_size * frames * sample_rate as usize) / 48000
    }
}

/// Opus bandwidth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusBandwidth {
    Narrowband,      // 4 kHz
    Mediumband,      // 6 kHz
    Wideband,        // 8 kHz
    SuperWideband,   // 12 kHz
    Fullband,        // 20 kHz
}

impl fmt::Display for OpusBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpusBandwidth::Narrowband => write!(f, "narrowband"),
            OpusBandwidth::Mediumband => write!(f, "mediumband"),
            OpusBandwidth::Wideband => write!(f, "wideband"),
            OpusBandwidth::SuperWideband => write!(f, "superwideband"),
            OpusBandwidth::Fullband => write!(f, "fullband"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opus_config_default() {
        let config = OpusConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.bitrate, 64000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_opus_config_voip() {
        let config = OpusConfig::voip();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.application, OpusApplication::Voip);
        assert!(config.fec_enabled);
        assert!(config.dtx_enabled);
    }

    #[test]
    fn test_opus_config_audio() {
        let config = OpusConfig::audio();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.application, OpusApplication::Audio);
    }

    #[test]
    fn test_opus_config_validation() {
        let mut config = OpusConfig::default();
        assert!(config.validate().is_ok());

        config.sample_rate = 44100; // Invalid
        assert!(config.validate().is_err());

        config.sample_rate = 48000;
        config.channels = 3; // Invalid
        assert!(config.validate().is_err());

        config.channels = 1;
        config.bitrate = 5000; // Too low
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_opus_frame_size() {
        let config = OpusConfig {
            sample_rate: 48000,
            frame_duration_ms: 20,
            ..Default::default()
        };
        assert_eq!(config.frame_size(), 960);

        let config = OpusConfig {
            sample_rate: 16000,
            frame_duration_ms: 20,
            ..Default::default()
        };
        assert_eq!(config.frame_size(), 320);
    }

    #[test]
    fn test_opus_encoder_creation() {
        let config = OpusConfig::voip();
        let encoder = OpusEncoder::new(config);
        assert!(encoder.is_ok());

        let mut invalid_config = OpusConfig::default();
        invalid_config.sample_rate = 44100;
        let encoder = OpusEncoder::new(invalid_config);
        assert!(encoder.is_err());
    }

    #[test]
    fn test_opus_decoder_creation() {
        let config = OpusConfig::audio();
        let decoder = OpusDecoder::new(config);
        assert!(decoder.is_ok());
    }

    #[test]
    fn test_opus_packet_parse() {
        let packet_data = vec![0x78, 0x01, 0x02]; // Example TOC byte
        let packet = OpusPacket::parse(&packet_data);
        assert!(packet.is_ok());

        let packet = packet.unwrap();
        assert_eq!(packet.toc(), 0x78);
        assert_eq!(packet.config(), 15); // (0x78 >> 3) & 0x1F
    }

    #[test]
    fn test_opus_config_setters() {
        let config = OpusConfig::default();
        let mut encoder = OpusEncoder::new(config).unwrap();

        encoder.set_bitrate(32000);
        assert_eq!(encoder.config().bitrate, 32000);

        encoder.set_fec(false);
        assert!(!encoder.config().fec_enabled);

        encoder.set_dtx(true);
        assert!(encoder.config().dtx_enabled);
    }
}

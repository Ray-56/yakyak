/// WAV file parsing and audio data extraction
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

/// WAV file format errors
#[derive(Debug, Clone, PartialEq)]
pub enum WavError {
    IoError(String),
    InvalidFormat(String),
    UnsupportedFormat(String),
}

impl From<io::Error> for WavError {
    fn from(err: io::Error) -> Self {
        WavError::IoError(err.to_string())
    }
}

/// WAV audio format
#[derive(Debug, Clone, PartialEq)]
pub struct WavFormat {
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Sample rate in Hz (e.g., 8000, 16000, 44100, 48000)
    pub sample_rate: u32,
    /// Bits per sample (8, 16, 24, 32)
    pub bits_per_sample: u16,
    /// Audio format code (1 = PCM)
    pub audio_format: u16,
}

impl WavFormat {
    /// Check if this format is compatible with G.711 (8kHz, mono, 8-bit)
    pub fn is_g711_compatible(&self) -> bool {
        self.sample_rate == 8000 && self.channels == 1 && self.bits_per_sample == 8
    }

    /// Check if this format needs resampling for telephony use
    pub fn needs_resampling(&self) -> bool {
        self.sample_rate != 8000
    }

    /// Get bytes per sample
    pub fn bytes_per_sample(&self) -> usize {
        (self.bits_per_sample / 8) as usize
    }

    /// Get bytes per frame (all channels)
    pub fn bytes_per_frame(&self) -> usize {
        self.bytes_per_sample() * self.channels as usize
    }

    /// Calculate duration in seconds
    pub fn calculate_duration(&self, data_size: usize) -> f64 {
        let frames = data_size / self.bytes_per_frame();
        frames as f64 / self.sample_rate as f64
    }
}

/// WAV file representation
#[derive(Debug, Clone)]
pub struct WavFile {
    /// Audio format information
    pub format: WavFormat,
    /// Raw audio data (PCM samples)
    pub data: Arc<Vec<u8>>,
}

impl WavFile {
    /// Load WAV file from path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, WavError> {
        let mut file = File::open(path)?;
        Self::from_reader(&mut file)
    }

    /// Parse WAV file from reader
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, WavError> {
        // Read RIFF header
        let mut riff_header = [0u8; 12];
        reader.read_exact(&mut riff_header)?;

        // Verify "RIFF" signature
        if &riff_header[0..4] != b"RIFF" {
            return Err(WavError::InvalidFormat(
                "Missing RIFF signature".to_string(),
            ));
        }

        // Verify "WAVE" format
        if &riff_header[8..12] != b"WAVE" {
            return Err(WavError::InvalidFormat(
                "Not a WAVE file".to_string(),
            ));
        }

        let _file_size = u32::from_le_bytes([
            riff_header[4],
            riff_header[5],
            riff_header[6],
            riff_header[7],
        ]);

        // Parse chunks until we find fmt and data
        let mut format: Option<WavFormat> = None;
        let mut data: Option<Vec<u8>> = None;

        loop {
            let mut chunk_header = [0u8; 8];
            if reader.read_exact(&mut chunk_header).is_err() {
                break; // End of file
            }

            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]) as usize;

            match chunk_id {
                b"fmt " => {
                    format = Some(Self::parse_fmt_chunk(reader, chunk_size)?);
                }
                b"data" => {
                    let mut audio_data = vec![0u8; chunk_size];
                    reader.read_exact(&mut audio_data)?;
                    data = Some(audio_data);
                }
                _ => {
                    // Skip unknown chunks
                    reader.seek(SeekFrom::Current(chunk_size as i64))?;
                }
            }

            // Align to even byte boundary (WAV chunks are word-aligned)
            if chunk_size % 2 != 0 {
                reader.seek(SeekFrom::Current(1))?;
            }

            // If we have both format and data, we're done
            if format.is_some() && data.is_some() {
                break;
            }
        }

        let format = format.ok_or_else(|| {
            WavError::InvalidFormat("Missing fmt chunk".to_string())
        })?;

        let data = data.ok_or_else(|| {
            WavError::InvalidFormat("Missing data chunk".to_string())
        })?;

        Ok(WavFile {
            format,
            data: Arc::new(data),
        })
    }

    /// Parse fmt chunk
    fn parse_fmt_chunk<R: Read>(
        reader: &mut R,
        chunk_size: usize,
    ) -> Result<WavFormat, WavError> {
        if chunk_size < 16 {
            return Err(WavError::InvalidFormat(
                "fmt chunk too small".to_string(),
            ));
        }

        let mut fmt_data = vec![0u8; chunk_size];
        reader.read_exact(&mut fmt_data)?;

        let audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
        let channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
        let sample_rate = u32::from_le_bytes([
            fmt_data[4],
            fmt_data[5],
            fmt_data[6],
            fmt_data[7],
        ]);
        let _byte_rate = u32::from_le_bytes([
            fmt_data[8],
            fmt_data[9],
            fmt_data[10],
            fmt_data[11],
        ]);
        let _block_align = u16::from_le_bytes([fmt_data[12], fmt_data[13]]);
        let bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

        // Only support PCM format
        if audio_format != 1 {
            return Err(WavError::UnsupportedFormat(format!(
                "Only PCM format (1) is supported, got {}",
                audio_format
            )));
        }

        // Validate parameters
        if channels == 0 || channels > 2 {
            return Err(WavError::InvalidFormat(format!(
                "Invalid number of channels: {}",
                channels
            )));
        }

        if sample_rate == 0 {
            return Err(WavError::InvalidFormat(
                "Invalid sample rate: 0".to_string(),
            ));
        }

        if bits_per_sample != 8 && bits_per_sample != 16 && bits_per_sample != 24 && bits_per_sample != 32 {
            return Err(WavError::UnsupportedFormat(format!(
                "Unsupported bits per sample: {}",
                bits_per_sample
            )));
        }

        Ok(WavFormat {
            channels,
            sample_rate,
            bits_per_sample,
            audio_format,
        })
    }

    /// Get audio duration in seconds
    pub fn duration(&self) -> f64 {
        self.format.calculate_duration(self.data.len())
    }

    /// Get audio data as signed 16-bit samples
    pub fn samples_i16(&self) -> Vec<i16> {
        match self.format.bits_per_sample {
            8 => {
                // 8-bit unsigned to 16-bit signed
                self.data
                    .iter()
                    .map(|&b| ((b as i16) - 128) * 256)
                    .collect()
            }
            16 => {
                // Already 16-bit signed
                self.data
                    .chunks_exact(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect()
            }
            24 => {
                // 24-bit to 16-bit (take upper 16 bits)
                self.data
                    .chunks_exact(3)
                    .map(|chunk| i16::from_le_bytes([chunk[1], chunk[2]]))
                    .collect()
            }
            32 => {
                // 32-bit to 16-bit (take upper 16 bits)
                self.data
                    .chunks_exact(4)
                    .map(|chunk| i16::from_le_bytes([chunk[2], chunk[3]]))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Convert stereo to mono by averaging channels
    pub fn to_mono(&self) -> Self {
        if self.format.channels == 1 {
            return self.clone();
        }

        let samples = self.samples_i16();
        let mono_samples: Vec<i16> = samples
            .chunks_exact(2)
            .map(|chunk| {
                let avg = (chunk[0] as i32 + chunk[1] as i32) / 2;
                avg as i16
            })
            .collect();

        // Convert back to bytes
        let mut mono_data = Vec::with_capacity(mono_samples.len() * 2);
        for sample in mono_samples {
            mono_data.extend_from_slice(&sample.to_le_bytes());
        }

        WavFile {
            format: WavFormat {
                channels: 1,
                bits_per_sample: 16,
                sample_rate: self.format.sample_rate,
                audio_format: 1,
            },
            data: Arc::new(mono_data),
        }
    }

    /// Simple resampling to target sample rate
    /// Uses linear interpolation for upsampling and decimation for downsampling
    pub fn resample(&self, target_rate: u32) -> Self {
        if self.format.sample_rate == target_rate {
            return self.clone();
        }

        let samples = self.samples_i16();
        let ratio = self.format.sample_rate as f64 / target_rate as f64;
        let new_length = (samples.len() as f64 / ratio) as usize;

        let resampled: Vec<i16> = (0..new_length)
            .map(|i| {
                let src_pos = i as f64 * ratio;
                let src_idx = src_pos as usize;

                if src_idx + 1 >= samples.len() {
                    return samples[samples.len() - 1];
                }

                // Linear interpolation
                let frac = src_pos - src_idx as f64;
                let sample1 = samples[src_idx] as f64;
                let sample2 = samples[src_idx + 1] as f64;
                let interpolated = sample1 + (sample2 - sample1) * frac;
                interpolated as i16
            })
            .collect();

        // Convert to bytes
        let mut resampled_data = Vec::with_capacity(resampled.len() * 2);
        for sample in resampled {
            resampled_data.extend_from_slice(&sample.to_le_bytes());
        }

        WavFile {
            format: WavFormat {
                sample_rate: target_rate,
                bits_per_sample: 16,
                channels: self.format.channels,
                audio_format: 1,
            },
            data: Arc::new(resampled_data),
        }
    }

    /// Convert to G.711 compatible format (8kHz, mono, 16-bit)
    pub fn to_g711_compatible(&self) -> Self {
        let mut result = self.clone();

        // Convert to mono if stereo
        if result.format.channels > 1 {
            result = result.to_mono();
        }

        // Resample to 8kHz if needed
        if result.format.sample_rate != 8000 {
            result = result.resample(8000);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_format_g711_compatibility() {
        let format = WavFormat {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 8,
            audio_format: 1,
        };
        assert!(format.is_g711_compatible());

        let format_stereo = WavFormat {
            channels: 2,
            sample_rate: 8000,
            bits_per_sample: 8,
            audio_format: 1,
        };
        assert!(!format_stereo.is_g711_compatible());
    }

    #[test]
    fn test_wav_format_needs_resampling() {
        let format_8k = WavFormat {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 16,
            audio_format: 1,
        };
        assert!(!format_8k.needs_resampling());

        let format_44k = WavFormat {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            audio_format: 1,
        };
        assert!(format_44k.needs_resampling());
    }

    #[test]
    fn test_wav_format_bytes_per_sample() {
        let format = WavFormat {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 16,
            audio_format: 1,
        };
        assert_eq!(format.bytes_per_sample(), 2);
        assert_eq!(format.bytes_per_frame(), 2);
    }

    #[test]
    fn test_wav_format_duration() {
        let format = WavFormat {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 16,
            audio_format: 1,
        };
        // 1 second of audio = 8000 samples * 2 bytes = 16000 bytes
        let duration = format.calculate_duration(16000);
        assert!((duration - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_samples_i16_conversion() {
        // Create a simple WAV with known values
        let data = vec![
            0x00, 0x01, // Sample 1: 256 (0x0100)
            0xFF, 0xFF, // Sample 2: -1 (0xFFFF)
            0x00, 0x80, // Sample 3: -32768 (0x8000)
            0xFF, 0x7F, // Sample 4: 32767 (0x7FFF)
        ];

        let wav = WavFile {
            format: WavFormat {
                channels: 1,
                sample_rate: 8000,
                bits_per_sample: 16,
                audio_format: 1,
            },
            data: Arc::new(data),
        };

        let samples = wav.samples_i16();
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0], 256);
        assert_eq!(samples[1], -1);
        assert_eq!(samples[2], -32768);
        assert_eq!(samples[3], 32767);
    }

    #[test]
    fn test_to_mono_conversion() {
        // Create stereo data: [L1, R1, L2, R2]
        let samples = vec![100i16, 200i16, 300i16, 400i16];
        let mut data = Vec::new();
        for sample in samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }

        let stereo_wav = WavFile {
            format: WavFormat {
                channels: 2,
                sample_rate: 8000,
                bits_per_sample: 16,
                audio_format: 1,
            },
            data: Arc::new(data),
        };

        let mono_wav = stereo_wav.to_mono();
        assert_eq!(mono_wav.format.channels, 1);

        let mono_samples = mono_wav.samples_i16();
        assert_eq!(mono_samples.len(), 2);
        assert_eq!(mono_samples[0], 150); // (100 + 200) / 2
        assert_eq!(mono_samples[1], 350); // (300 + 400) / 2
    }
}

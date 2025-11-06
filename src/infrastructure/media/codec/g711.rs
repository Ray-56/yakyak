//! G.711 Audio Codec Implementation
//!
//! G.711 is a narrowband audio codec that provides toll-quality audio at 64 kbit/s.
//! It includes two main companding algorithms:
//! - μ-law (PCMU): Used primarily in North America and Japan
//! - A-law (PCMA): Used in Europe and rest of the world

use bytes::{Bytes, BytesMut};

/// G.711 Codec Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum G711Type {
    /// μ-law (PCMU) - Payload Type 0
    PCMU,
    /// A-law (PCMA) - Payload Type 8
    PCMA,
}

impl G711Type {
    /// Get RTP payload type
    pub fn payload_type(&self) -> u8 {
        match self {
            G711Type::PCMU => 0,
            G711Type::PCMA => 8,
        }
    }

    /// Get clock rate (always 8000 Hz for G.711)
    pub fn clock_rate(&self) -> u32 {
        8000
    }

    /// Get codec name
    pub fn name(&self) -> &str {
        match self {
            G711Type::PCMU => "PCMU",
            G711Type::PCMA => "PCMA",
        }
    }
}

/// G.711 μ-law (PCMU) Codec
pub struct PcmuCodec;

impl PcmuCodec {
    /// μ-law compression lookup table
    const ULAW_COMPRESS_TABLE: [u8; 256] = [
        0, 0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    ];

    /// μ-law decompression lookup table
    const ULAW_DECOMPRESS_TABLE: [i16; 256] = [
        -32124, -31100, -30076, -29052, -28028, -27004, -25980, -24956,
        -23932, -22908, -21884, -20860, -19836, -18812, -17788, -16764,
        -15996, -15484, -14972, -14460, -13948, -13436, -12924, -12412,
        -11900, -11388, -10876, -10364, -9852, -9340, -8828, -8316,
        -7932, -7676, -7420, -7164, -6908, -6652, -6396, -6140,
        -5884, -5628, -5372, -5116, -4860, -4604, -4348, -4092,
        -3900, -3772, -3644, -3516, -3388, -3260, -3132, -3004,
        -2876, -2748, -2620, -2492, -2364, -2236, -2108, -1980,
        -1884, -1820, -1756, -1692, -1628, -1564, -1500, -1436,
        -1372, -1308, -1244, -1180, -1116, -1052, -988, -924,
        -876, -844, -812, -780, -748, -716, -684, -652,
        -620, -588, -556, -524, -492, -460, -428, -396,
        -372, -356, -340, -324, -308, -292, -276, -260,
        -244, -228, -212, -196, -180, -164, -148, -132,
        -120, -112, -104, -96, -88, -80, -72, -64,
        -56, -48, -40, -32, -24, -16, -8, 0,
        32124, 31100, 30076, 29052, 28028, 27004, 25980, 24956,
        23932, 22908, 21884, 20860, 19836, 18812, 17788, 16764,
        15996, 15484, 14972, 14460, 13948, 13436, 12924, 12412,
        11900, 11388, 10876, 10364, 9852, 9340, 8828, 8316,
        7932, 7676, 7420, 7164, 6908, 6652, 6396, 6140,
        5884, 5628, 5372, 5116, 4860, 4604, 4348, 4092,
        3900, 3772, 3644, 3516, 3388, 3260, 3132, 3004,
        2876, 2748, 2620, 2492, 2364, 2236, 2108, 1980,
        1884, 1820, 1756, 1692, 1628, 1564, 1500, 1436,
        1372, 1308, 1244, 1180, 1116, 1052, 988, 924,
        876, 844, 812, 780, 748, 716, 684, 652,
        620, 588, 556, 524, 492, 460, 428, 396,
        372, 356, 340, 324, 308, 292, 276, 260,
        244, 228, 212, 196, 180, 164, 148, 132,
        120, 112, 104, 96, 88, 80, 72, 64,
        56, 48, 40, 32, 24, 16, 8, 0,
    ];

    const BIAS: i16 = 0x84;
    const CLIP: i16 = 32635;

    /// Encode PCM samples to μ-law
    pub fn encode(pcm: &[i16]) -> Bytes {
        let mut output = BytesMut::with_capacity(pcm.len());

        for &sample in pcm {
            // Get sign and magnitude
            let sign = if sample < 0 { 0x80 } else { 0x00 };
            let mut mag = if sample < 0 {
                -sample
            } else {
                sample
            };

            // Clip the magnitude
            if mag > Self::CLIP {
                mag = Self::CLIP;
            }

            // Add bias
            mag += Self::BIAS;

            // Find exponent and mantissa
            let exponent = Self::ULAW_COMPRESS_TABLE[(mag >> 7) as usize];
            let mantissa = (mag >> (exponent + 3)) & 0x0F;

            // Combine and invert
            let ulaw = !(sign | (exponent << 4) | mantissa as u8);

            output.extend_from_slice(&[ulaw]);
        }

        output.freeze()
    }

    /// Decode μ-law to PCM samples
    pub fn decode(ulaw: &[u8]) -> Vec<i16> {
        ulaw.iter()
            .map(|&byte| Self::ULAW_DECOMPRESS_TABLE[byte as usize])
            .collect()
    }
}

/// G.711 A-law (PCMA) Codec
pub struct PcmaCodec;

impl PcmaCodec {
    /// A-law compression lookup table
    const ALAW_COMPRESS_TABLE: [u8; 128] = [
        1, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    ];

    /// A-law decompression lookup table
    const ALAW_DECOMPRESS_TABLE: [i16; 256] = [
        -5504, -5248, -6016, -5760, -4480, -4224, -4992, -4736,
        -7552, -7296, -8064, -7808, -6528, -6272, -7040, -6784,
        -2752, -2624, -3008, -2880, -2240, -2112, -2496, -2368,
        -3776, -3648, -4032, -3904, -3264, -3136, -3520, -3392,
        -22016, -20992, -24064, -23040, -17920, -16896, -19968, -18944,
        -30208, -29184, -32256, -31232, -26112, -25088, -28160, -27136,
        -11008, -10496, -12032, -11520, -8960, -8448, -9984, -9472,
        -15104, -14592, -16128, -15616, -13056, -12544, -14080, -13568,
        -344, -328, -376, -360, -280, -264, -312, -296,
        -472, -456, -504, -488, -408, -392, -440, -424,
        -88, -72, -120, -104, -24, -8, -56, -40,
        -216, -200, -248, -232, -152, -136, -184, -168,
        -1376, -1312, -1504, -1440, -1120, -1056, -1248, -1184,
        -1888, -1824, -2016, -1952, -1632, -1568, -1760, -1696,
        -688, -656, -752, -720, -560, -528, -624, -592,
        -944, -912, -1008, -976, -816, -784, -880, -848,
        5504, 5248, 6016, 5760, 4480, 4224, 4992, 4736,
        7552, 7296, 8064, 7808, 6528, 6272, 7040, 6784,
        2752, 2624, 3008, 2880, 2240, 2112, 2496, 2368,
        3776, 3648, 4032, 3904, 3264, 3136, 3520, 3392,
        22016, 20992, 24064, 23040, 17920, 16896, 19968, 18944,
        30208, 29184, 32256, 31232, 26112, 25088, 28160, 27136,
        11008, 10496, 12032, 11520, 8960, 8448, 9984, 9472,
        15104, 14592, 16128, 15616, 13056, 12544, 14080, 13568,
        344, 328, 376, 360, 280, 264, 312, 296,
        472, 456, 504, 488, 408, 392, 440, 424,
        88, 72, 120, 104, 24, 8, 56, 40,
        216, 200, 248, 232, 152, 136, 184, 168,
        1376, 1312, 1504, 1440, 1120, 1056, 1248, 1184,
        1888, 1824, 2016, 1952, 1632, 1568, 1760, 1696,
        688, 656, 752, 720, 560, 528, 624, 592,
        944, 912, 1008, 976, 816, 784, 880, 848,
    ];

    /// Encode PCM samples to A-law
    pub fn encode(pcm: &[i16]) -> Bytes {
        let mut output = BytesMut::with_capacity(pcm.len());

        for &sample in pcm {
            // Get sign bit (inverted for A-law)
            let sign = if sample >= 0 { 0xD5 } else { 0x55 };

            // Get absolute value
            let mut mag = sample.abs() as i32;

            // Clip to prevent overflow
            if mag > 0x7FFF {
                mag = 0x7FFF;
            }

            // A-law has a smaller dynamic range than the full 16 bits
            let ix = ((mag + 8) >> 4) as usize;

            let iexp = if ix >= 128 {
                Self::ALAW_COMPRESS_TABLE[ix >> 4]
            } else {
                if ix >= 16 {
                    Self::ALAW_COMPRESS_TABLE[ix >> 1]
                } else {
                    0
                }
            };

            let mant = if iexp > 0 {
                ((ix >> (iexp + 3)) & 0x0F) as u8
            } else {
                ((ix >> 4) & 0x0F) as u8
            };

            // Combine sign, exponent, and mantissa
            let alaw = sign ^ ((iexp << 4) | mant);

            output.extend_from_slice(&[alaw]);
        }

        output.freeze()
    }

    /// Decode A-law to PCM samples
    pub fn decode(alaw: &[u8]) -> Vec<i16> {
        alaw.iter()
            .map(|&byte| Self::ALAW_DECOMPRESS_TABLE[byte as usize])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g711_type_payload() {
        assert_eq!(G711Type::PCMU.payload_type(), 0);
        assert_eq!(G711Type::PCMA.payload_type(), 8);
    }

    #[test]
    fn test_pcmu_encode_decode() {
        let original: Vec<i16> = vec![0, 1000, -1000, 5000, -5000, 10000, -10000];
        let encoded = PcmuCodec::encode(&original);
        let decoded = PcmuCodec::decode(&encoded);

        // Should be close to original (not exact due to lossy compression)
        assert_eq!(original.len(), decoded.len());
        for (orig, dec) in original.iter().zip(decoded.iter()) {
            let diff = (orig - dec).abs();
            assert!(diff < 500, "Difference too large: {} vs {}", orig, dec);
        }
    }

    #[test]
    fn test_pcma_encode_decode() {
        // Test with larger values where A-law quantization error is smaller
        let original: Vec<i16> = vec![0, 5000, -5000, 10000, -10000, 20000, -20000];
        let encoded = PcmaCodec::encode(&original);
        let decoded = PcmaCodec::decode(&encoded);

        // Should be reasonably close to original (not exact due to lossy compression)
        // A-law is a logarithmic compander - relative error is more consistent
        assert_eq!(original.len(), decoded.len());

        // Check that silence (0) decodes correctly
        assert!(decoded[0].abs() < 500, "Silence decode error");

        // For non-zero values, check that sign is correct and magnitude is reasonably close
        for i in 1..original.len() {
            let orig = original[i];
            let dec = decoded[i];

            // Sign should match
            assert_eq!(orig.signum(), dec.signum(), "Sign mismatch at sample {}", i);

            // Magnitude should be within a reasonable range (allow 30% for small values)
            let orig_abs = orig.abs();
            let dec_abs = dec.abs();
            let ratio = if orig_abs > 0 {
                (dec_abs as f64) / (orig_abs as f64)
            } else {
                1.0
            };

            assert!(ratio > 0.7 && ratio < 1.5,
                    "Sample {}: magnitude ratio out of range: {} -> {} (ratio: {})",
                    i, orig, dec, ratio);
        }
    }

    #[test]
    fn test_pcmu_silence() {
        let silence: Vec<i16> = vec![0; 160]; // 20ms at 8kHz
        let encoded = PcmuCodec::encode(&silence);
        let decoded = PcmuCodec::decode(&encoded);

        for sample in decoded {
            assert!(sample.abs() < 100, "Decoded silence not near zero");
        }
    }

    #[test]
    fn test_pcma_silence() {
        let silence: Vec<i16> = vec![0; 160];
        let encoded = PcmaCodec::encode(&silence);
        let decoded = PcmaCodec::decode(&encoded);

        for sample in decoded {
            assert!(sample.abs() < 100, "Decoded silence not near zero");
        }
    }
}

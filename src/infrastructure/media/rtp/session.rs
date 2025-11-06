//! RTP Session Management

use super::packet::{RtpError, RtpPacket};
use bytes::Bytes;
use rand::Rng;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::Arc;
use tracing::{debug, warn};

/// RTP Session
///
/// Manages RTP packet generation and reception for a single media stream
pub struct RtpSession {
    /// Synchronization source identifier (randomly generated)
    ssrc: u32,
    /// Sequence number (starts random, increments for each packet)
    sequence: Arc<AtomicU16>,
    /// Timestamp base (starts at random value)
    timestamp_base: u32,
    /// Packets sent counter
    packets_sent: Arc<AtomicU32>,
    /// Bytes sent counter
    bytes_sent: Arc<AtomicU32>,
    /// Payload type
    payload_type: u8,
    /// Clock rate (samples per second)
    clock_rate: u32,
}

impl RtpSession {
    /// Create a new RTP session
    pub fn new(payload_type: u8, clock_rate: u32) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            ssrc: rng.gen(),
            sequence: Arc::new(AtomicU16::new(rng.gen())),
            timestamp_base: rng.gen(),
            packets_sent: Arc::new(AtomicU32::new(0)),
            bytes_sent: Arc::new(AtomicU32::new(0)),
            payload_type,
            clock_rate,
        }
    }

    /// Create with specific SSRC
    pub fn with_ssrc(ssrc: u32, payload_type: u8, clock_rate: u32) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            ssrc,
            sequence: Arc::new(AtomicU16::new(rng.gen())),
            timestamp_base: rng.gen(),
            packets_sent: Arc::new(AtomicU32::new(0)),
            bytes_sent: Arc::new(AtomicU32::new(0)),
            payload_type,
            clock_rate,
        }
    }

    /// Get SSRC
    pub fn ssrc(&self) -> u32 {
        self.ssrc
    }

    /// Get current sequence number (without incrementing)
    pub fn sequence(&self) -> u16 {
        self.sequence.load(Ordering::Relaxed)
    }

    /// Get next sequence number (with increment)
    fn next_sequence(&self) -> u16 {
        self.sequence.fetch_add(1, Ordering::Relaxed)
    }

    /// Calculate timestamp for current time
    pub fn calculate_timestamp(&self, samples: u32) -> u32 {
        // Calculate timestamp based on clock rate
        // For audio: timestamp increments by number of samples in packet
        self.timestamp_base.wrapping_add(samples)
    }

    /// Calculate timestamp from duration in milliseconds
    pub fn timestamp_from_ms(&self, ms: u64) -> u32 {
        let samples = (ms * self.clock_rate as u64) / 1000;
        self.timestamp_base.wrapping_add(samples as u32)
    }

    /// Create an RTP packet
    pub fn create_packet(&self, payload: Bytes, timestamp: u32, marker: bool) -> RtpPacket {
        let sequence = self.next_sequence();

        let mut packet = RtpPacket::new(
            self.payload_type,
            sequence,
            timestamp,
            self.ssrc,
            payload.clone(),
        );

        packet.set_marker(marker);

        // Update statistics
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(payload.len() as u32, Ordering::Relaxed);

        debug!(
            "Created RTP packet: seq={}, ts={}, ssrc={:08x}, size={}",
            sequence,
            timestamp,
            self.ssrc,
            payload.len()
        );

        packet
    }

    /// Get packets sent count
    pub fn packets_sent(&self) -> u32 {
        self.packets_sent.load(Ordering::Relaxed)
    }

    /// Get bytes sent count
    pub fn bytes_sent(&self) -> u32 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Validate received packet
    pub fn validate_packet(&self, packet: &RtpPacket) -> Result<(), RtpError> {
        if packet.payload_type != self.payload_type {
            warn!(
                "Unexpected payload type: expected {}, got {}",
                self.payload_type, packet.payload_type
            );
            return Err(RtpError::InvalidPayloadType(packet.payload_type));
        }

        Ok(())
    }
}

/// SSRC Generator
///
/// Generates unique SSRC identifiers with collision detection
pub struct SsrcGenerator {
    used_ssrcs: Arc<tokio::sync::RwLock<std::collections::HashSet<u32>>>,
}

impl SsrcGenerator {
    /// Create a new SSRC generator
    pub fn new() -> Self {
        Self {
            used_ssrcs: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Generate a unique SSRC
    pub async fn generate(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let mut used_ssrcs = self.used_ssrcs.write().await;

        loop {
            let ssrc = rng.gen();
            if ssrc != 0 && !used_ssrcs.contains(&ssrc) {
                used_ssrcs.insert(ssrc);
                debug!("Generated new SSRC: {:08x}", ssrc);
                return ssrc;
            }
        }
    }

    /// Release an SSRC
    pub async fn release(&self, ssrc: u32) {
        let mut used_ssrcs = self.used_ssrcs.write().await;
        used_ssrcs.remove(&ssrc);
        debug!("Released SSRC: {:08x}", ssrc);
    }

    /// Check if SSRC is in use
    pub async fn is_used(&self, ssrc: u32) -> bool {
        let used_ssrcs = self.used_ssrcs.read().await;
        used_ssrcs.contains(&ssrc)
    }
}

impl Default for SsrcGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// RTP Statistics
#[derive(Debug, Clone, Default)]
pub struct RtpStats {
    pub packets_sent: u32,
    pub packets_received: u32,
    pub bytes_sent: u32,
    pub bytes_received: u32,
    pub packets_lost: u32,
    pub jitter: f64,
}

impl RtpStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn packet_loss_rate(&self) -> f64 {
        if self.packets_received == 0 {
            return 0.0;
        }
        (self.packets_lost as f64) / ((self.packets_received + self.packets_lost) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_session_creation() {
        let session = RtpSession::new(0, 8000);
        assert_eq!(session.payload_type, 0);
        assert_eq!(session.clock_rate, 8000);
        assert_ne!(session.ssrc, 0);
    }

    #[test]
    fn test_sequence_increment() {
        let session = RtpSession::new(0, 8000);
        let seq1 = session.next_sequence();
        let seq2 = session.next_sequence();
        assert_eq!(seq2, seq1.wrapping_add(1));
    }

    #[test]
    fn test_create_packet() {
        let session = RtpSession::new(8, 8000);
        let payload = Bytes::from_static(b"test");
        let packet = session.create_packet(payload.clone(), 1000, false);

        assert_eq!(packet.payload_type, 8);
        assert_eq!(packet.ssrc, session.ssrc);
        assert_eq!(packet.payload, payload);
        assert!(!packet.marker);
    }

    #[test]
    fn test_timestamp_calculation() {
        let session = RtpSession::new(0, 8000);
        // For 8000 Hz clock, 160 samples = 20ms
        let ts1 = session.calculate_timestamp(0);
        let ts2 = session.calculate_timestamp(160);
        assert_eq!(ts2.wrapping_sub(ts1), 160);
    }

    #[tokio::test]
    async fn test_ssrc_generator() {
        let generator = SsrcGenerator::new();
        let ssrc1 = generator.generate().await;
        let ssrc2 = generator.generate().await;

        assert_ne!(ssrc1, 0);
        assert_ne!(ssrc2, 0);
        assert_ne!(ssrc1, ssrc2);
        assert!(generator.is_used(ssrc1).await);
        assert!(generator.is_used(ssrc2).await);
    }

    #[tokio::test]
    async fn test_ssrc_release() {
        let generator = SsrcGenerator::new();
        let ssrc = generator.generate().await;

        assert!(generator.is_used(ssrc).await);
        generator.release(ssrc).await;
        assert!(!generator.is_used(ssrc).await);
    }
}

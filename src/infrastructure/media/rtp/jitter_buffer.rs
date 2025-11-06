//! Jitter Buffer Implementation
//!
//! Handles packet reordering and delay variation

use super::packet::RtpPacket;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Jitter Buffer Configuration
#[derive(Debug, Clone)]
pub struct JitterBufferConfig {
    /// Minimum delay in milliseconds
    pub min_delay_ms: u32,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u32,
    /// Maximum buffer size in packets
    pub max_packets: usize,
}

impl Default for JitterBufferConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 20,
            max_delay_ms: 200,
            max_packets: 100,
        }
    }
}

/// Buffered packet with arrival time
#[derive(Debug, Clone)]
struct BufferedPacket {
    packet: RtpPacket,
    arrival_time: Instant,
}

/// Jitter Buffer
///
/// Buffers RTP packets to smooth out network jitter
pub struct JitterBuffer {
    config: JitterBufferConfig,
    buffer: VecDeque<BufferedPacket>,
    next_sequence: Option<u16>,
    base_timestamp: Option<u32>,
    packets_received: u64,
    packets_dropped: u64,
    packets_late: u64,
}

impl JitterBuffer {
    pub fn new(config: JitterBufferConfig) -> Self {
        let max_packets = config.max_packets;
        Self {
            config,
            buffer: VecDeque::with_capacity(max_packets),
            next_sequence: None,
            base_timestamp: None,
            packets_received: 0,
            packets_dropped: 0,
            packets_late: 0,
        }
    }

    /// Add packet to buffer
    pub fn add_packet(&mut self, packet: RtpPacket) {
        self.packets_received += 1;

        // Initialize timestamp tracking
        if self.base_timestamp.is_none() {
            self.base_timestamp = Some(packet.timestamp);
        }

        // Check buffer capacity
        if self.buffer.len() >= self.config.max_packets {
            warn!("Jitter buffer full, dropping oldest packet");
            self.buffer.pop_front();
            self.packets_dropped += 1;
        }

        // Add packet with arrival time
        let buffered = BufferedPacket {
            packet,
            arrival_time: Instant::now(),
        };

        // Insert in sequence order
        let seq = buffered.packet.sequence;
        let insert_pos = self
            .buffer
            .iter()
            .position(|p| self.sequence_less_than(seq, p.packet.sequence))
            .unwrap_or(self.buffer.len());

        self.buffer.insert(insert_pos, buffered);

        debug!(
            "Buffered packet seq={}, buffer_size={}",
            seq,
            self.buffer.len()
        );
    }

    /// Get next packet if ready
    pub fn get_packet(&mut self) -> Option<RtpPacket> {
        if self.buffer.is_empty() {
            return None;
        }

        let front = &self.buffer[0];
        let age = front.arrival_time.elapsed();

        // Wait for minimum delay
        if age < Duration::from_millis(self.config.min_delay_ms as u64) {
            return None;
        }

        // Initialize sequence tracking from first packet
        if self.next_sequence.is_none() {
            self.next_sequence = Some(front.packet.sequence);
        }

        // Check if this is the expected sequence
        if let Some(expected_seq) = self.next_sequence {
            let actual_seq = front.packet.sequence;

            if actual_seq == expected_seq {
                // Expected packet
                let packet = self.buffer.pop_front().unwrap().packet;
                self.next_sequence = Some(actual_seq.wrapping_add(1));
                return Some(packet);
            } else if self.sequence_less_than(actual_seq, expected_seq) {
                // Late packet (already processed)
                warn!(
                    "Late packet: expected={}, got={}",
                    expected_seq, actual_seq
                );
                self.buffer.pop_front();
                self.packets_late += 1;
                return self.get_packet(); // Try next
            } else {
                // Packet in future, check if we should skip
                if age > Duration::from_millis(self.config.max_delay_ms as u64) {
                    // Too old, assume packet lost and skip to this one
                    warn!("Skipping lost packets: {} to {}", expected_seq, actual_seq);
                    let packet = self.buffer.pop_front().unwrap().packet;
                    self.next_sequence = Some(actual_seq.wrapping_add(1));
                    return Some(packet);
                }
            }
        }

        None
    }

    /// Check if a < b considering wraparound
    fn sequence_less_than(&self, a: u16, b: u16) -> bool {
        // Handle sequence number wraparound
        const SEQ_DIFF_THRESHOLD: i32 = 32768;
        let diff = (b as i32) - (a as i32);

        if diff.abs() < SEQ_DIFF_THRESHOLD {
            diff > 0
        } else {
            diff < 0
        }
    }

    /// Get buffer statistics
    pub fn stats(&self) -> JitterBufferStats {
        JitterBufferStats {
            buffer_size: self.buffer.len(),
            packets_received: self.packets_received,
            packets_dropped: self.packets_dropped,
            packets_late: self.packets_late,
        }
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.next_sequence = None;
        self.base_timestamp = None;
    }
}

/// Jitter Buffer Statistics
#[derive(Debug, Clone)]
pub struct JitterBufferStats {
    pub buffer_size: usize,
    pub packets_received: u64,
    pub packets_dropped: u64,
    pub packets_late: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn create_test_packet(sequence: u16, timestamp: u32) -> RtpPacket {
        RtpPacket::new(0, sequence, timestamp, 12345, Bytes::from(vec![0u8; 160]))
    }

    #[test]
    fn test_jitter_buffer_creation() {
        let config = JitterBufferConfig::default();
        let buffer = JitterBuffer::new(config);
        assert_eq!(buffer.buffer.len(), 0);
    }

    #[test]
    fn test_add_and_get_packet() {
        let config = JitterBufferConfig {
            min_delay_ms: 0, // No delay for testing
            max_delay_ms: 100,
            max_packets: 10,
        };
        let mut buffer = JitterBuffer::new(config);

        let packet = create_test_packet(100, 1000);
        buffer.add_packet(packet.clone());

        // Should get packet immediately with 0 delay
        let retrieved = buffer.get_packet();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sequence, 100);
    }

    #[test]
    fn test_packet_reordering() {
        let config = JitterBufferConfig {
            min_delay_ms: 0,
            max_delay_ms: 100,
            max_packets: 10,
        };
        let mut buffer = JitterBuffer::new(config);

        // Add packets out of order
        buffer.add_packet(create_test_packet(102, 1020));
        buffer.add_packet(create_test_packet(100, 1000));
        buffer.add_packet(create_test_packet(101, 1010));

        // Should retrieve in order
        assert_eq!(buffer.get_packet().unwrap().sequence, 100);
        assert_eq!(buffer.get_packet().unwrap().sequence, 101);
        assert_eq!(buffer.get_packet().unwrap().sequence, 102);
    }

    #[test]
    fn test_sequence_wraparound() {
        let buffer = JitterBuffer::new(JitterBufferConfig::default());

        // Test wraparound
        assert!(buffer.sequence_less_than(65535, 0));
        assert!(buffer.sequence_less_than(65534, 65535));
        assert!(!buffer.sequence_less_than(0, 65535));
    }

    #[test]
    fn test_buffer_overflow() {
        let config = JitterBufferConfig {
            min_delay_ms: 0,
            max_delay_ms: 100,
            max_packets: 3, // Small buffer
        };
        let mut buffer = JitterBuffer::new(config);

        // Add more packets than capacity
        for i in 0..5 {
            buffer.add_packet(create_test_packet(i, i as u32 * 10));
        }

        let stats = buffer.stats();
        assert_eq!(stats.buffer_size, 3); // Should be at max
        assert_eq!(stats.packets_dropped, 2); // 2 packets dropped
    }

    #[test]
    fn test_statistics() {
        let config = JitterBufferConfig {
            min_delay_ms: 0,
            max_delay_ms: 100,
            max_packets: 10,
        };
        let mut buffer = JitterBuffer::new(config);

        buffer.add_packet(create_test_packet(100, 1000));
        buffer.add_packet(create_test_packet(101, 1010));

        let stats = buffer.stats();
        assert_eq!(stats.packets_received, 2);
        assert_eq!(stats.buffer_size, 2);
    }
}

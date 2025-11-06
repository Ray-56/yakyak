//! RTP Packet Implementation (RFC 3550)

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt;

/// RTP Packet Structure
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |V=2|P|X|  CC   |M|     PT      |       sequence number         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           timestamp                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           synchronization source (SSRC) identifier            |
/// +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
/// |            contributing source (CSRC) identifiers             |
/// |                             ....                              |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone)]
pub struct RtpPacket {
    /// RTP version (should be 2)
    pub version: u8,
    /// Padding flag
    pub padding: bool,
    /// Extension flag
    pub extension: bool,
    /// CSRC count
    pub csrc_count: u8,
    /// Marker bit
    pub marker: bool,
    /// Payload type
    pub payload_type: u8,
    /// Sequence number
    pub sequence: u16,
    /// Timestamp
    pub timestamp: u32,
    /// Synchronization source identifier
    pub ssrc: u32,
    /// Contributing source identifiers
    pub csrc: Vec<u32>,
    /// Extension header (if extension flag is set)
    pub extension_profile: Option<u16>,
    pub extension_data: Option<Bytes>,
    /// Payload data
    pub payload: Bytes,
    /// Padding length (if padding flag is set)
    pub padding_len: u8,
}

impl RtpPacket {
    /// Minimum RTP header size (without CSRC, extension, or padding)
    pub const MIN_HEADER_SIZE: usize = 12;

    /// Create a new RTP packet
    pub fn new(payload_type: u8, sequence: u16, timestamp: u32, ssrc: u32, payload: Bytes) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            csrc: Vec::new(),
            extension_profile: None,
            extension_data: None,
            payload,
            padding_len: 0,
        }
    }

    /// Parse RTP packet from bytes
    pub fn parse(data: &[u8]) -> Result<Self, RtpError> {
        if data.len() < Self::MIN_HEADER_SIZE {
            return Err(RtpError::PacketTooShort);
        }

        let mut buf = &data[..];

        // Byte 0: V(2), P(1), X(1), CC(4)
        let byte0 = buf.get_u8();
        let version = (byte0 >> 6) & 0x03;
        let padding = (byte0 & 0x20) != 0;
        let extension = (byte0 & 0x10) != 0;
        let csrc_count = byte0 & 0x0F;

        if version != 2 {
            return Err(RtpError::InvalidVersion(version));
        }

        // Byte 1: M(1), PT(7)
        let byte1 = buf.get_u8();
        let marker = (byte1 & 0x80) != 0;
        let payload_type = byte1 & 0x7F;

        // Bytes 2-3: Sequence number
        let sequence = buf.get_u16();

        // Bytes 4-7: Timestamp
        let timestamp = buf.get_u32();

        // Bytes 8-11: SSRC
        let ssrc = buf.get_u32();

        // CSRC list
        let mut csrc = Vec::with_capacity(csrc_count as usize);
        for _ in 0..csrc_count {
            if buf.remaining() < 4 {
                return Err(RtpError::PacketTooShort);
            }
            csrc.push(buf.get_u32());
        }

        // Extension header
        let (extension_profile, extension_data) = if extension {
            if buf.remaining() < 4 {
                return Err(RtpError::PacketTooShort);
            }
            let profile = buf.get_u16();
            let length = buf.get_u16() as usize * 4; // Length in 32-bit words

            if buf.remaining() < length {
                return Err(RtpError::PacketTooShort);
            }

            let ext_data = Bytes::copy_from_slice(&buf[..length]);
            buf.advance(length);

            (Some(profile), Some(ext_data))
        } else {
            (None, None)
        };

        // Payload and padding
        let mut payload_len = buf.remaining();
        let mut padding_len = 0;

        if padding {
            if payload_len == 0 {
                return Err(RtpError::InvalidPadding);
            }
            padding_len = buf[payload_len - 1];
            if padding_len == 0 || padding_len as usize > payload_len {
                return Err(RtpError::InvalidPadding);
            }
            payload_len -= padding_len as usize;
        }

        let payload = Bytes::copy_from_slice(&buf[..payload_len]);

        Ok(Self {
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            csrc,
            extension_profile,
            extension_data,
            payload,
            padding_len,
        })
    }

    /// Serialize RTP packet to bytes
    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.calculate_size());

        // Byte 0: V(2), P(1), X(1), CC(4)
        let byte0 = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        buf.put_u8(byte0);

        // Byte 1: M(1), PT(7)
        let byte1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        buf.put_u8(byte1);

        // Bytes 2-3: Sequence number
        buf.put_u16(self.sequence);

        // Bytes 4-7: Timestamp
        buf.put_u32(self.timestamp);

        // Bytes 8-11: SSRC
        buf.put_u32(self.ssrc);

        // CSRC list
        for csrc in &self.csrc {
            buf.put_u32(*csrc);
        }

        // Extension header
        if let (Some(profile), Some(data)) = (&self.extension_profile, &self.extension_data) {
            buf.put_u16(*profile);
            buf.put_u16((data.len() / 4) as u16); // Length in 32-bit words
            buf.put_slice(data);
        }

        // Payload
        buf.put_slice(&self.payload);

        // Padding
        if self.padding && self.padding_len > 0 {
            for _ in 0..(self.padding_len - 1) {
                buf.put_u8(0);
            }
            buf.put_u8(self.padding_len);
        }

        buf.freeze()
    }

    /// Calculate total packet size
    fn calculate_size(&self) -> usize {
        let mut size = Self::MIN_HEADER_SIZE;
        size += self.csrc.len() * 4;
        if let Some(data) = &self.extension_data {
            size += 4 + data.len();
        }
        size += self.payload.len();
        if self.padding {
            size += self.padding_len as usize;
        }
        size
    }

    /// Set marker bit
    pub fn set_marker(&mut self, marker: bool) {
        self.marker = marker;
    }

    /// Add CSRC
    pub fn add_csrc(&mut self, csrc: u32) {
        if self.csrc.len() < 15 {
            self.csrc.push(csrc);
            self.csrc_count = self.csrc.len() as u8;
        }
    }

    /// Set extension
    pub fn set_extension(&mut self, profile: u16, data: Bytes) {
        self.extension = true;
        self.extension_profile = Some(profile);
        self.extension_data = Some(data);
    }

    /// Add padding
    pub fn add_padding(&mut self, target_size: usize) {
        let current_size = self.calculate_size();
        if target_size > current_size {
            self.padding = true;
            self.padding_len = (target_size - current_size) as u8;
        }
    }
}

impl fmt::Display for RtpPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RTP[PT={}, Seq={}, TS={}, SSRC={:08x}, Marker={}, Payload={}]",
            self.payload_type,
            self.sequence,
            self.timestamp,
            self.ssrc,
            self.marker,
            self.payload.len()
        )
    }
}

/// RTP errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RtpError {
    #[error("Packet too short")]
    PacketTooShort,
    #[error("Invalid version: {0}")]
    InvalidVersion(u8),
    #[error("Invalid padding")]
    InvalidPadding,
    #[error("Invalid payload type: {0}")]
    InvalidPayloadType(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_packet_serialize_parse() {
        let payload = Bytes::from_static(b"Hello RTP");
        let packet = RtpPacket::new(0, 1234, 567890, 0x12345678, payload.clone());

        let data = packet.serialize();
        let parsed = RtpPacket::parse(&data).unwrap();

        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.payload_type, 0);
        assert_eq!(parsed.sequence, 1234);
        assert_eq!(parsed.timestamp, 567890);
        assert_eq!(parsed.ssrc, 0x12345678);
        assert_eq!(parsed.payload, payload);
    }

    #[test]
    fn test_rtp_packet_with_marker() {
        let payload = Bytes::from_static(b"Test");
        let mut packet = RtpPacket::new(8, 100, 1000, 0xAABBCCDD, payload);
        packet.set_marker(true);

        let data = packet.serialize();
        let parsed = RtpPacket::parse(&data).unwrap();

        assert!(parsed.marker);
    }

    #[test]
    fn test_rtp_packet_with_csrc() {
        let payload = Bytes::from_static(b"Test");
        let mut packet = RtpPacket::new(8, 100, 1000, 0xAABBCCDD, payload);
        packet.add_csrc(0x11111111);
        packet.add_csrc(0x22222222);

        let data = packet.serialize();
        let parsed = RtpPacket::parse(&data).unwrap();

        assert_eq!(parsed.csrc_count, 2);
        assert_eq!(parsed.csrc[0], 0x11111111);
        assert_eq!(parsed.csrc[1], 0x22222222);
    }

    #[test]
    fn test_rtp_min_size() {
        let data = vec![0u8; 11]; // Too short
        assert!(matches!(RtpPacket::parse(&data), Err(RtpError::PacketTooShort)));
    }

    #[test]
    fn test_rtp_invalid_version() {
        let mut data = vec![0u8; 12];
        data[0] = 0x40; // Version 1
        assert!(matches!(RtpPacket::parse(&data), Err(RtpError::InvalidVersion(1))));
    }
}

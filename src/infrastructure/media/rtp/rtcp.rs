//! RTCP (RTP Control Protocol) Implementation (RFC 3550)

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::{SystemTime, UNIX_EPOCH};

/// RTCP Packet Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RtcpPacketType {
    /// Sender Report
    SR = 200,
    /// Receiver Report
    RR = 201,
    /// Source Description
    SDES = 202,
    /// Goodbye
    BYE = 203,
    /// Application Defined
    APP = 204,
}

impl RtcpPacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            200 => Some(Self::SR),
            201 => Some(Self::RR),
            202 => Some(Self::SDES),
            203 => Some(Self::BYE),
            204 => Some(Self::APP),
            _ => None,
        }
    }
}

/// RTCP Packet
#[derive(Debug, Clone)]
pub enum RtcpPacket {
    SenderReport(SenderReport),
    ReceiverReport(ReceiverReport),
    SourceDescription(SourceDescription),
    Goodbye(Goodbye),
}

impl RtcpPacket {
    /// Parse RTCP packet from bytes
    pub fn parse(data: &[u8]) -> Result<Self, RtcpError> {
        if data.len() < 4 {
            return Err(RtcpError::PacketTooShort);
        }

        let buf = &data[..];
        let byte0 = buf[0];
        let version = (byte0 >> 6) & 0x03;

        if version != 2 {
            return Err(RtcpError::InvalidVersion(version));
        }

        let packet_type = buf[1];

        match RtcpPacketType::from_u8(packet_type) {
            Some(RtcpPacketType::SR) => Ok(RtcpPacket::SenderReport(SenderReport::parse(data)?)),
            Some(RtcpPacketType::RR) => Ok(RtcpPacket::ReceiverReport(ReceiverReport::parse(data)?)),
            Some(RtcpPacketType::SDES) => Ok(RtcpPacket::SourceDescription(SourceDescription::parse(data)?)),
            Some(RtcpPacketType::BYE) => Ok(RtcpPacket::Goodbye(Goodbye::parse(data)?)),
            _ => Err(RtcpError::UnsupportedPacketType(packet_type)),
        }
    }

    /// Serialize RTCP packet to bytes
    pub fn serialize(&self) -> Bytes {
        match self {
            RtcpPacket::SenderReport(sr) => sr.serialize(),
            RtcpPacket::ReceiverReport(rr) => rr.serialize(),
            RtcpPacket::SourceDescription(sdes) => sdes.serialize(),
            RtcpPacket::Goodbye(bye) => bye.serialize(),
        }
    }
}

/// Sender Report (SR)
#[derive(Debug, Clone)]
pub struct SenderReport {
    pub ssrc: u32,
    pub ntp_timestamp: u64,
    pub rtp_timestamp: u32,
    pub packet_count: u32,
    pub octet_count: u32,
    pub reports: Vec<ReceptionReport>,
}

impl SenderReport {
    pub fn new(ssrc: u32, rtp_timestamp: u32, packet_count: u32, octet_count: u32) -> Self {
        let ntp_timestamp = Self::get_ntp_timestamp();
        Self {
            ssrc,
            ntp_timestamp,
            rtp_timestamp,
            packet_count,
            octet_count,
            reports: Vec::new(),
        }
    }

    fn get_ntp_timestamp() -> u64 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();

        // NTP timestamp is seconds since 1900, Unix is since 1970
        // Difference is 2208988800 seconds
        const NTP_EPOCH_OFFSET: u64 = 2208988800;

        let seconds = duration.as_secs() + NTP_EPOCH_OFFSET;
        let fraction = ((duration.subsec_nanos() as u64) << 32) / 1_000_000_000;

        (seconds << 32) | fraction
    }

    pub fn add_report(&mut self, report: ReceptionReport) {
        if self.reports.len() < 31 {
            self.reports.push(report);
        }
    }

    pub fn parse(data: &[u8]) -> Result<Self, RtcpError> {
        if data.len() < 28 {
            return Err(RtcpError::PacketTooShort);
        }

        let mut buf = &data[..];

        let byte0 = buf.get_u8();
        let count = byte0 & 0x1F;
        let _pt = buf.get_u8(); // packet type
        let length = buf.get_u16() as usize;

        if data.len() < (length + 1) * 4 {
            return Err(RtcpError::PacketTooShort);
        }

        let ssrc = buf.get_u32();
        let ntp_timestamp = buf.get_u64();
        let rtp_timestamp = buf.get_u32();
        let packet_count = buf.get_u32();
        let octet_count = buf.get_u32();

        let mut reports = Vec::new();
        for _ in 0..count {
            if buf.remaining() < 24 {
                break;
            }
            reports.push(ReceptionReport::parse_from_buf(&mut buf)?);
        }

        Ok(Self {
            ssrc,
            ntp_timestamp,
            rtp_timestamp,
            packet_count,
            octet_count,
            reports,
        })
    }

    pub fn serialize(&self) -> Bytes {
        let length = 6 + (self.reports.len() * 6);
        let mut buf = BytesMut::with_capacity((length + 1) * 4);

        // Header
        let byte0 = 0x80 | (self.reports.len() as u8 & 0x1F);
        buf.put_u8(byte0);
        buf.put_u8(RtcpPacketType::SR as u8);
        buf.put_u16(length as u16);

        // Sender info
        buf.put_u32(self.ssrc);
        buf.put_u64(self.ntp_timestamp);
        buf.put_u32(self.rtp_timestamp);
        buf.put_u32(self.packet_count);
        buf.put_u32(self.octet_count);

        // Reception reports
        for report in &self.reports {
            report.serialize_into_buf(&mut buf);
        }

        buf.freeze()
    }
}

/// Receiver Report (RR)
#[derive(Debug, Clone)]
pub struct ReceiverReport {
    pub ssrc: u32,
    pub reports: Vec<ReceptionReport>,
}

impl ReceiverReport {
    pub fn new(ssrc: u32) -> Self {
        Self {
            ssrc,
            reports: Vec::new(),
        }
    }

    pub fn add_report(&mut self, report: ReceptionReport) {
        if self.reports.len() < 31 {
            self.reports.push(report);
        }
    }

    pub fn parse(data: &[u8]) -> Result<Self, RtcpError> {
        if data.len() < 8 {
            return Err(RtcpError::PacketTooShort);
        }

        let mut buf = &data[..];

        let byte0 = buf.get_u8();
        let count = byte0 & 0x1F;
        let _pt = buf.get_u8();
        let length = buf.get_u16() as usize;

        if data.len() < (length + 1) * 4 {
            return Err(RtcpError::PacketTooShort);
        }

        let ssrc = buf.get_u32();

        let mut reports = Vec::new();
        for _ in 0..count {
            if buf.remaining() < 24 {
                break;
            }
            reports.push(ReceptionReport::parse_from_buf(&mut buf)?);
        }

        Ok(Self { ssrc, reports })
    }

    pub fn serialize(&self) -> Bytes {
        let length = 1 + (self.reports.len() * 6);
        let mut buf = BytesMut::with_capacity((length + 1) * 4);

        let byte0 = 0x80 | (self.reports.len() as u8 & 0x1F);
        buf.put_u8(byte0);
        buf.put_u8(RtcpPacketType::RR as u8);
        buf.put_u16(length as u16);

        buf.put_u32(self.ssrc);

        for report in &self.reports {
            report.serialize_into_buf(&mut buf);
        }

        buf.freeze()
    }
}

/// Reception Report Block
#[derive(Debug, Clone)]
pub struct ReceptionReport {
    pub ssrc: u32,
    pub fraction_lost: u8,
    pub cumulative_lost: u32,
    pub highest_seq: u32,
    pub jitter: u32,
    pub lsr: u32,  // Last SR timestamp
    pub dlsr: u32, // Delay since last SR
}

impl ReceptionReport {
    pub fn new(ssrc: u32) -> Self {
        Self {
            ssrc,
            fraction_lost: 0,
            cumulative_lost: 0,
            highest_seq: 0,
            jitter: 0,
            lsr: 0,
            dlsr: 0,
        }
    }

    fn parse_from_buf(buf: &mut &[u8]) -> Result<Self, RtcpError> {
        let ssrc = buf.get_u32();
        let lost_byte = buf.get_u8();
        let fraction_lost = lost_byte;
        let cumulative_lost = ((buf.get_u8() as u32) << 16)
                            | ((buf.get_u8() as u32) << 8)
                            | (buf.get_u8() as u32);
        let highest_seq = buf.get_u32();
        let jitter = buf.get_u32();
        let lsr = buf.get_u32();
        let dlsr = buf.get_u32();

        Ok(Self {
            ssrc,
            fraction_lost,
            cumulative_lost,
            highest_seq,
            jitter,
            lsr,
            dlsr,
        })
    }

    fn serialize_into_buf(&self, buf: &mut BytesMut) {
        buf.put_u32(self.ssrc);
        buf.put_u8(self.fraction_lost);
        buf.put_u8(((self.cumulative_lost >> 16) & 0xFF) as u8);
        buf.put_u8(((self.cumulative_lost >> 8) & 0xFF) as u8);
        buf.put_u8((self.cumulative_lost & 0xFF) as u8);
        buf.put_u32(self.highest_seq);
        buf.put_u32(self.jitter);
        buf.put_u32(self.lsr);
        buf.put_u32(self.dlsr);
    }
}

/// Source Description (SDES)
#[derive(Debug, Clone)]
pub struct SourceDescription {
    pub items: Vec<SdesItem>,
}

impl SourceDescription {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn parse(_data: &[u8]) -> Result<Self, RtcpError> {
        // Simplified SDES parsing
        Ok(Self { items: Vec::new() })
    }

    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(8);

        buf.put_u8(0x80);
        buf.put_u8(RtcpPacketType::SDES as u8);
        buf.put_u16(1);
        buf.put_u32(0);

        buf.freeze()
    }
}

#[derive(Debug, Clone)]
pub struct SdesItem {
    pub ssrc: u32,
    pub items: Vec<(u8, String)>,
}

/// Goodbye (BYE)
#[derive(Debug, Clone)]
pub struct Goodbye {
    pub ssrcs: Vec<u32>,
    pub reason: Option<String>,
}

impl Goodbye {
    pub fn new(ssrc: u32) -> Self {
        Self {
            ssrcs: vec![ssrc],
            reason: None,
        }
    }

    pub fn parse(data: &[u8]) -> Result<Self, RtcpError> {
        if data.len() < 4 {
            return Err(RtcpError::PacketTooShort);
        }

        let mut buf = &data[..];
        let byte0 = buf.get_u8();
        let count = byte0 & 0x1F;
        let _pt = buf.get_u8();
        let _length = buf.get_u16();

        let mut ssrcs = Vec::new();
        for _ in 0..count {
            if buf.remaining() < 4 {
                break;
            }
            ssrcs.push(buf.get_u32());
        }

        Ok(Self {
            ssrcs,
            reason: None,
        })
    }

    pub fn serialize(&self) -> Bytes {
        let length = self.ssrcs.len();
        let mut buf = BytesMut::with_capacity((length + 1) * 4);

        let byte0 = 0x80 | (self.ssrcs.len() as u8 & 0x1F);
        buf.put_u8(byte0);
        buf.put_u8(RtcpPacketType::BYE as u8);
        buf.put_u16(length as u16);

        for ssrc in &self.ssrcs {
            buf.put_u32(*ssrc);
        }

        buf.freeze()
    }
}

/// RTCP Errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RtcpError {
    #[error("Packet too short")]
    PacketTooShort,
    #[error("Invalid version: {0}")]
    InvalidVersion(u8),
    #[error("Unsupported packet type: {0}")]
    UnsupportedPacketType(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_report() {
        let sr = SenderReport::new(0x12345678, 1000, 100, 16000);
        let data = sr.serialize();
        let parsed = SenderReport::parse(&data).unwrap();

        assert_eq!(parsed.ssrc, 0x12345678);
        assert_eq!(parsed.rtp_timestamp, 1000);
        assert_eq!(parsed.packet_count, 100);
        assert_eq!(parsed.octet_count, 16000);
    }

    #[test]
    fn test_receiver_report() {
        let rr = ReceiverReport::new(0xAABBCCDD);
        let data = rr.serialize();
        let parsed = ReceiverReport::parse(&data).unwrap();

        assert_eq!(parsed.ssrc, 0xAABBCCDD);
        assert_eq!(parsed.reports.len(), 0);
    }

    #[test]
    fn test_goodbye() {
        let bye = Goodbye::new(0x11223344);
        let data = bye.serialize();
        let parsed = Goodbye::parse(&data).unwrap();

        assert_eq!(parsed.ssrcs.len(), 1);
        assert_eq!(parsed.ssrcs[0], 0x11223344);
    }
}

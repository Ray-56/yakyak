/// TURN message types and parsing (RFC 5766)
use std::net::SocketAddr;

/// TURN message methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMethod {
    /// Allocate - Request an allocation on the server
    Allocate = 0x003,
    /// Refresh - Refresh an existing allocation
    Refresh = 0x004,
    /// Send - Send data through the relay
    Send = 0x006,
    /// Data - Receive data from the relay
    Data = 0x007,
    /// CreatePermission - Create a permission for a peer
    CreatePermission = 0x008,
    /// ChannelBind - Bind a channel to a peer
    ChannelBind = 0x009,
}

impl TurnMethod {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x003 => Some(TurnMethod::Allocate),
            0x004 => Some(TurnMethod::Refresh),
            0x006 => Some(TurnMethod::Send),
            0x007 => Some(TurnMethod::Data),
            0x008 => Some(TurnMethod::CreatePermission),
            0x009 => Some(TurnMethod::ChannelBind),
            _ => None,
        }
    }

    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

/// TURN message class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMessageClass {
    Request = 0x00,
    Indication = 0x01,
    SuccessResponse = 0x02,
    ErrorResponse = 0x03,
}

/// TURN message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnMessageType {
    pub method: TurnMethod,
    pub class: TurnMessageClass,
}

impl TurnMessageType {
    pub fn new(method: TurnMethod, class: TurnMessageClass) -> Self {
        Self { method, class }
    }

    /// Encode as 14-bit value for message header
    pub fn encode(&self) -> u16 {
        let method = self.method.to_u16();
        let class = self.class as u16;

        // Encode according to RFC 5389 Section 6
        let m11_m7 = (method & 0x0F80) << 2;
        let m6_m4 = (method & 0x0070) << 1;
        let m3_m0 = method & 0x000F;
        let c1_c0 = class & 0x0003;

        m11_m7 | (c1_c0 << 4) | m6_m4 | m3_m0
    }

    /// Decode from 14-bit value
    pub fn decode(value: u16) -> Option<Self> {
        let m11_m7 = (value & 0x3E00) >> 2;
        let m6_m4 = (value & 0x00E0) >> 1;
        let m3_m0 = value & 0x000F;
        let c1_c0 = (value & 0x0110) >> 4;

        let method_value = m11_m7 | m6_m4 | m3_m0;
        let method = TurnMethod::from_u16(method_value)?;

        let class = match c1_c0 {
            0x00 => TurnMessageClass::Request,
            0x01 => TurnMessageClass::Indication,
            0x02 => TurnMessageClass::SuccessResponse,
            0x03 => TurnMessageClass::ErrorResponse,
            _ => return None,
        };

        Some(Self { method, class })
    }
}

/// TURN attributes
#[derive(Debug, Clone)]
pub enum TurnAttribute {
    /// MAPPED-ADDRESS
    MappedAddress(SocketAddr),
    /// XOR-MAPPED-ADDRESS
    XorMappedAddress(SocketAddr),
    /// XOR-RELAYED-ADDRESS - The relayed transport address
    XorRelayedAddress(SocketAddr),
    /// XOR-PEER-ADDRESS - The peer address
    XorPeerAddress(SocketAddr),
    /// LIFETIME - Allocation lifetime in seconds
    Lifetime(u32),
    /// DATA - Application data
    Data(Vec<u8>),
    /// REALM - Authentication realm
    Realm(String),
    /// NONCE - Authentication nonce
    Nonce(String),
    /// USERNAME - Username for authentication
    Username(String),
    /// MESSAGE-INTEGRITY - HMAC-SHA1 fingerprint
    MessageIntegrity(Vec<u8>),
    /// FINGERPRINT - CRC-32 fingerprint
    Fingerprint(u32),
    /// ERROR-CODE - Error code and reason
    ErrorCode { code: u16, reason: String },
    /// CHANNEL-NUMBER - Channel number for data transmission
    ChannelNumber(u16),
    /// REQUESTED-TRANSPORT - Requested transport protocol
    RequestedTransport(u8),
    /// Unknown attribute
    Unknown { attr_type: u16, value: Vec<u8> },
}

/// TURN message
#[derive(Debug, Clone)]
pub struct TurnMessage {
    pub message_type: TurnMessageType,
    pub length: u16,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<TurnAttribute>,
}

impl TurnMessage {
    /// TURN magic cookie (same as STUN)
    pub const MAGIC_COOKIE: u32 = 0x2112A442;

    /// Create a new TURN message
    pub fn new(message_type: TurnMessageType) -> Self {
        Self {
            message_type,
            length: 0,
            transaction_id: Self::generate_transaction_id(),
            attributes: Vec::new(),
        }
    }

    /// Generate a random transaction ID
    pub fn generate_transaction_id() -> [u8; 12] {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut id = [0u8; 12];
        rng.fill(&mut id);
        id
    }

    /// Add an attribute
    pub fn add_attribute(&mut self, attr: TurnAttribute) {
        self.attributes.push(attr);
    }

    /// Get attribute by type
    pub fn get_relayed_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let TurnAttribute::XorRelayedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }

    /// Get peer address
    pub fn get_peer_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let TurnAttribute::XorPeerAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }

    /// Get lifetime
    pub fn get_lifetime(&self) -> Option<u32> {
        for attr in &self.attributes {
            if let TurnAttribute::Lifetime(lifetime) = attr {
                return Some(*lifetime);
            }
        }
        None
    }

    /// Get data
    pub fn get_data(&self) -> Option<&[u8]> {
        for attr in &self.attributes {
            if let TurnAttribute::Data(data) = attr {
                return Some(data);
            }
        }
        None
    }

    /// Parse TURN message from bytes
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 20 {
            return Err("Message too short".to_string());
        }

        // Parse header
        let message_type_raw = u16::from_be_bytes([data[0], data[1]]);
        let message_type = TurnMessageType::decode(message_type_raw)
            .ok_or_else(|| "Invalid message type".to_string())?;

        let length = u16::from_be_bytes([data[2], data[3]]);
        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        if magic_cookie != Self::MAGIC_COOKIE {
            return Err("Invalid magic cookie".to_string());
        }

        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&data[8..20]);

        // Parse attributes
        let attributes = Vec::new();
        // TODO: Parse attributes from data[20..20+length]

        Ok(Self {
            message_type,
            length,
            transaction_id,
            attributes,
        })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Message type (2 bytes)
        let msg_type = self.message_type.encode();
        bytes.extend_from_slice(&msg_type.to_be_bytes());

        // Message length (2 bytes) - will update later
        bytes.extend_from_slice(&0u16.to_be_bytes());

        // Magic cookie (4 bytes)
        bytes.extend_from_slice(&Self::MAGIC_COOKIE.to_be_bytes());

        // Transaction ID (12 bytes)
        bytes.extend_from_slice(&self.transaction_id);

        // Attributes
        let attr_start = bytes.len();
        for attr in &self.attributes {
            self.serialize_attribute(attr, &mut bytes);
        }

        // Update length field
        let length = (bytes.len() - 20) as u16;
        bytes[2..4].copy_from_slice(&length.to_be_bytes());

        bytes
    }

    /// Serialize a single attribute
    fn serialize_attribute(&self, attr: &TurnAttribute, bytes: &mut Vec<u8>) {
        match attr {
            TurnAttribute::Lifetime(lifetime) => {
                bytes.extend_from_slice(&0x000Du16.to_be_bytes()); // LIFETIME
                bytes.extend_from_slice(&4u16.to_be_bytes()); // Length
                bytes.extend_from_slice(&lifetime.to_be_bytes());
            }
            TurnAttribute::RequestedTransport(protocol) => {
                bytes.extend_from_slice(&0x0019u16.to_be_bytes()); // REQUESTED-TRANSPORT
                bytes.extend_from_slice(&4u16.to_be_bytes()); // Length
                bytes.push(*protocol);
                bytes.extend_from_slice(&[0u8; 3]); // RFFU
            }
            TurnAttribute::Data(data) => {
                bytes.extend_from_slice(&0x0013u16.to_be_bytes()); // DATA
                let len = data.len() as u16;
                bytes.extend_from_slice(&len.to_be_bytes());
                bytes.extend_from_slice(data);
                // Padding to 4-byte boundary
                let padding = (4 - (len % 4)) % 4;
                for _ in 0..padding {
                    bytes.push(0);
                }
            }
            TurnAttribute::Username(username) => {
                bytes.extend_from_slice(&0x0006u16.to_be_bytes()); // USERNAME
                let username_bytes = username.as_bytes();
                let len = username_bytes.len() as u16;
                bytes.extend_from_slice(&len.to_be_bytes());
                bytes.extend_from_slice(username_bytes);
                // Padding
                let padding = (4 - (len % 4)) % 4;
                for _ in 0..padding {
                    bytes.push(0);
                }
            }
            _ => {
                // TODO: Implement other attributes
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_method_encoding() {
        assert_eq!(TurnMethod::Allocate.to_u16(), 0x003);
        assert_eq!(TurnMethod::Refresh.to_u16(), 0x004);
        assert_eq!(TurnMethod::Send.to_u16(), 0x006);
    }

    #[test]
    fn test_message_type_encoding() {
        let msg_type = TurnMessageType::new(TurnMethod::Allocate, TurnMessageClass::Request);
        let encoded = msg_type.encode();
        let decoded = TurnMessageType::decode(encoded).unwrap();
        assert_eq!(decoded.method, TurnMethod::Allocate);
        assert_eq!(decoded.class, TurnMessageClass::Request);
    }

    #[test]
    fn test_message_creation() {
        let msg_type = TurnMessageType::new(TurnMethod::Allocate, TurnMessageClass::Request);
        let mut message = TurnMessage::new(msg_type);
        message.add_attribute(TurnAttribute::Lifetime(600));
        message.add_attribute(TurnAttribute::RequestedTransport(17)); // UDP

        assert_eq!(message.attributes.len(), 2);
        assert_eq!(message.get_lifetime(), Some(600));
    }

    #[test]
    fn test_message_serialization() {
        let msg_type = TurnMessageType::new(TurnMethod::Allocate, TurnMessageClass::Request);
        let mut message = TurnMessage::new(msg_type);
        message.add_attribute(TurnAttribute::Lifetime(600));

        let bytes = message.to_bytes();
        assert!(bytes.len() >= 20); // At least header
        assert_eq!(&bytes[4..8], &TurnMessage::MAGIC_COOKIE.to_be_bytes());
    }

    #[test]
    fn test_transaction_id_generation() {
        let id1 = TurnMessage::generate_transaction_id();
        let id2 = TurnMessage::generate_transaction_id();
        assert_ne!(id1, id2); // Should be random
    }
}

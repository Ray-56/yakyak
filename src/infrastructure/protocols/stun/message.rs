/// STUN message format (RFC 5389)
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// STUN message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMessageType {
    BindingRequest,
    BindingResponse,
    BindingErrorResponse,
}

impl StunMessageType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(StunMessageType::BindingRequest),
            0x0101 => Some(StunMessageType::BindingResponse),
            0x0111 => Some(StunMessageType::BindingErrorResponse),
            _ => None,
        }
    }

    pub fn to_u16(self) -> u16 {
        match self {
            StunMessageType::BindingRequest => 0x0001,
            StunMessageType::BindingResponse => 0x0101,
            StunMessageType::BindingErrorResponse => 0x0111,
        }
    }
}

/// STUN method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMethod {
    Binding,
}

/// STUN attribute type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunAttributeType {
    MappedAddress = 0x0001,
    Username = 0x0006,
    MessageIntegrity = 0x0008,
    ErrorCode = 0x0009,
    UnknownAttributes = 0x000A,
    Realm = 0x0014,
    Nonce = 0x0015,
    XorMappedAddress = 0x0020,
    Software = 0x8022,
    AlternateServer = 0x8023,
    Fingerprint = 0x8028,
}

/// STUN attribute
#[derive(Debug, Clone)]
pub enum StunAttribute {
    MappedAddress(SocketAddr),
    XorMappedAddress(SocketAddr),
    Username(String),
    Software(String),
    ErrorCode(u16, String),
    Unknown(u16, Vec<u8>),
}

/// STUN message
#[derive(Debug, Clone)]
pub struct StunMessage {
    pub message_type: StunMessageType,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<StunAttribute>,
}

impl StunMessage {
    /// Magic cookie for STUN (RFC 5389)
    pub const MAGIC_COOKIE: u32 = 0x2112A442;

    /// Create new STUN Binding Request
    pub fn new_binding_request() -> Self {
        let mut transaction_id = [0u8; 12];
        for byte in transaction_id.iter_mut() {
            *byte = rand::random();
        }

        Self {
            message_type: StunMessageType::BindingRequest,
            transaction_id,
            attributes: Vec::new(),
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Message Type (2 bytes)
        buffer.extend_from_slice(&self.message_type.to_u16().to_be_bytes());

        // Message Length (2 bytes) - will update later
        let length_pos = buffer.len();
        buffer.extend_from_slice(&0u16.to_be_bytes());

        // Magic Cookie (4 bytes)
        buffer.extend_from_slice(&Self::MAGIC_COOKIE.to_be_bytes());

        // Transaction ID (12 bytes)
        buffer.extend_from_slice(&self.transaction_id);

        // Attributes
        let attributes_start = buffer.len();
        for attr in &self.attributes {
            self.write_attribute(&mut buffer, attr);
        }

        // Update message length
        let attributes_len = buffer.len() - attributes_start;
        buffer[length_pos..length_pos + 2].copy_from_slice(&(attributes_len as u16).to_be_bytes());

        buffer
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 20 {
            return Err("Message too short".to_string());
        }

        // Parse message type
        let message_type_raw = u16::from_be_bytes([data[0], data[1]]);
        let message_type = StunMessageType::from_u16(message_type_raw)
            .ok_or_else(|| format!("Unknown message type: {:#x}", message_type_raw))?;

        // Parse message length
        let message_length = u16::from_be_bytes([data[2], data[3]]) as usize;

        // Check magic cookie
        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        if magic_cookie != Self::MAGIC_COOKIE {
            return Err("Invalid magic cookie".to_string());
        }

        // Parse transaction ID
        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&data[8..20]);

        // Parse attributes
        let mut attributes = Vec::new();
        let mut offset = 20;

        while offset < 20 + message_length {
            if offset + 4 > data.len() {
                break;
            }

            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;

            offset += 4;

            if offset + attr_length > data.len() {
                break;
            }

            let attr_data = &data[offset..offset + attr_length];
            if let Some(attr) = Self::parse_attribute(attr_type, attr_data, &transaction_id) {
                attributes.push(attr);
            }

            // Attributes are padded to 4-byte boundary
            offset += attr_length;
            let padding = (4 - (attr_length % 4)) % 4;
            offset += padding;
        }

        Ok(Self {
            message_type,
            transaction_id,
            attributes,
        })
    }

    /// Write attribute to buffer
    fn write_attribute(&self, buffer: &mut Vec<u8>, attr: &StunAttribute) {
        match attr {
            StunAttribute::Software(software) => {
                buffer.extend_from_slice(&(StunAttributeType::Software as u16).to_be_bytes());
                buffer.extend_from_slice(&(software.len() as u16).to_be_bytes());
                buffer.extend_from_slice(software.as_bytes());
                // Padding
                let padding = (4 - (software.len() % 4)) % 4;
                buffer.extend_from_slice(&vec![0u8; padding]);
            }
            _ => {
                // TODO: Implement other attributes
            }
        }
    }

    /// Parse attribute from bytes
    fn parse_attribute(attr_type: u16, data: &[u8], transaction_id: &[u8; 12]) -> Option<StunAttribute> {
        match attr_type {
            0x0001 => {
                // MAPPED-ADDRESS
                if data.len() >= 8 {
                    let family = data[1];
                    let port = u16::from_be_bytes([data[2], data[3]]);

                    let addr = match family {
                        0x01 => {
                            // IPv4
                            let ip = Ipv4Addr::new(data[4], data[5], data[6], data[7]);
                            SocketAddr::new(IpAddr::V4(ip), port)
                        }
                        0x02 if data.len() >= 20 => {
                            // IPv6
                            let mut octets = [0u8; 16];
                            octets.copy_from_slice(&data[4..20]);
                            let ip = Ipv6Addr::from(octets);
                            SocketAddr::new(IpAddr::V6(ip), port)
                        }
                        _ => return None,
                    };

                    Some(StunAttribute::MappedAddress(addr))
                } else {
                    None
                }
            }
            0x0020 => {
                // XOR-MAPPED-ADDRESS
                if data.len() >= 8 {
                    let family = data[1];
                    let xor_port = u16::from_be_bytes([data[2], data[3]]);
                    let port = xor_port ^ (Self::MAGIC_COOKIE >> 16) as u16;

                    let addr = match family {
                        0x01 => {
                            // IPv4
                            let xor_ip = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                            let ip_u32 = xor_ip ^ Self::MAGIC_COOKIE;
                            let ip = Ipv4Addr::from(ip_u32);
                            SocketAddr::new(IpAddr::V4(ip), port)
                        }
                        0x02 if data.len() >= 20 => {
                            // IPv6
                            let mut xor_bytes = [0u8; 16];
                            xor_bytes.copy_from_slice(&data[4..20]);

                            // XOR with magic cookie + transaction ID
                            let mut xor_key = [0u8; 16];
                            xor_key[0..4].copy_from_slice(&Self::MAGIC_COOKIE.to_be_bytes());
                            xor_key[4..16].copy_from_slice(transaction_id);

                            let mut ip_bytes = [0u8; 16];
                            for i in 0..16 {
                                ip_bytes[i] = xor_bytes[i] ^ xor_key[i];
                            }

                            let ip = Ipv6Addr::from(ip_bytes);
                            SocketAddr::new(IpAddr::V6(ip), port)
                        }
                        _ => return None,
                    };

                    Some(StunAttribute::XorMappedAddress(addr))
                } else {
                    None
                }
            }
            0x8022 => {
                // SOFTWARE
                if let Ok(software) = String::from_utf8(data.to_vec()) {
                    Some(StunAttribute::Software(software))
                } else {
                    None
                }
            }
            _ => Some(StunAttribute::Unknown(attr_type, data.to_vec())),
        }
    }

    /// Add SOFTWARE attribute
    pub fn add_software(&mut self, software: String) {
        self.attributes.push(StunAttribute::Software(software));
    }

    /// Get XOR-MAPPED-ADDRESS attribute
    pub fn get_xor_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::XorMappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }

    /// Get MAPPED-ADDRESS attribute
    pub fn get_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::MappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(StunMessageType::BindingRequest.to_u16(), 0x0001);
        assert_eq!(StunMessageType::from_u16(0x0001), Some(StunMessageType::BindingRequest));
        assert_eq!(StunMessageType::from_u16(0x9999), None);
    }

    #[test]
    fn test_create_binding_request() {
        let msg = StunMessage::new_binding_request();

        assert_eq!(msg.message_type, StunMessageType::BindingRequest);
        assert_eq!(msg.transaction_id.len(), 12);
    }

    #[test]
    fn test_serialize_binding_request() {
        let msg = StunMessage::new_binding_request();
        let bytes = msg.to_bytes();

        assert!(bytes.len() >= 20);
        assert_eq!(bytes[0], 0x00);
        assert_eq!(bytes[1], 0x01); // Binding Request
    }

    #[test]
    fn test_parse_binding_response() {
        // Minimal binding response with XOR-MAPPED-ADDRESS
        let mut data = vec![
            0x01, 0x01, // Binding Response
            0x00, 0x0C, // Length: 12 bytes
        ];
        data.extend_from_slice(&StunMessage::MAGIC_COOKIE.to_be_bytes());
        data.extend_from_slice(&[0u8; 12]); // Transaction ID

        // XOR-MAPPED-ADDRESS attribute
        data.extend_from_slice(&0x0020u16.to_be_bytes()); // Type
        data.extend_from_slice(&0x0008u16.to_be_bytes()); // Length: 8
        data.push(0x00); // Reserved
        data.push(0x01); // Family: IPv4
        data.extend_from_slice(&0x1234u16.to_be_bytes()); // XOR-ed port
        data.extend_from_slice(&0xC0A80001u32.to_be_bytes()); // XOR-ed IPv4

        let msg = StunMessage::from_bytes(&data).unwrap();

        assert_eq!(msg.message_type, StunMessageType::BindingResponse);
        assert!(msg.get_xor_mapped_address().is_some());
    }
}

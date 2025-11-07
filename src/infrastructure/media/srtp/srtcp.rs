/// SRTCP (Secure RTCP) implementation
/// RFC 3711 Section 3.4
use super::crypto::*;
use super::context::SrtpError;
use std::sync::{Arc, Mutex};

/// SRTCP context for encrypting/decrypting RTCP packets
pub struct SrtcpContext {
    /// Protection profile
    profile: SrtpProfile,
    /// Session keys (uses SRTCP-specific keys)
    srtp_cipher_key: Vec<u8>,
    srtp_auth_key: Vec<u8>,
    srtp_salt: Vec<u8>,
    /// SRTCP index for this context
    srtcp_index: Arc<Mutex<u32>>,
}

impl SrtcpContext {
    /// Create new SRTCP context from session keys
    pub fn new(
        cipher_key: Vec<u8>,
        auth_key: Vec<u8>,
        salt: Vec<u8>,
        profile: SrtpProfile,
    ) -> Self {
        Self {
            profile,
            srtp_cipher_key: cipher_key,
            srtp_auth_key: auth_key,
            srtp_salt: salt,
            srtcp_index: Arc::new(Mutex::new(0)),
        }
    }

    /// Parse RTCP packet header to extract SSRC
    fn parse_rtcp_header(packet: &[u8]) -> Result<u32, SrtpError> {
        if packet.len() < 8 {
            return Err(SrtpError::InvalidPacket("RTCP packet too short".to_string()));
        }

        let v = (packet[0] >> 6) & 0x03;
        if v != 2 {
            return Err(SrtpError::InvalidPacket(format!("Invalid RTCP version: {}", v)));
        }

        // SSRC at bytes 4-7
        let ssrc = u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]);

        Ok(ssrc)
    }

    /// Get RTCP header length (fixed 8 bytes for SR/RR)
    fn get_rtcp_header_len(packet: &[u8]) -> Result<usize, SrtpError> {
        if packet.len() < 8 {
            return Err(SrtpError::InvalidPacket("RTCP packet too short".to_string()));
        }

        // RTCP packets have an 8-byte common header
        // After that, the structure varies by packet type
        // For SRTCP, we encrypt from byte 8 onwards
        Ok(8)
    }

    /// Encrypt RTCP packet in-place
    /// Returns the encrypted packet with E flag and SRTCP index
    pub fn encrypt_rtcp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        let ssrc = Self::parse_rtcp_header(packet)?;
        let header_len = Self::get_rtcp_header_len(packet)?;

        // Get and increment SRTCP index
        let mut index_lock = self.srtcp_index.lock().unwrap();
        let srtcp_index = *index_lock;
        *index_lock = index_lock.wrapping_add(1);
        drop(index_lock);

        // Generate IV for SRTCP
        // SRTCP uses SSRC || SRTCP_index (not packet index like SRTP)
        let iv = generate_iv(&self.srtp_salt, ssrc, srtcp_index as u64);

        // Encrypt payload (everything after header)
        if packet.len() > header_len {
            let payload_len = packet.len() - header_len;
            let keystream = aes_cm_keystream(&self.srtp_cipher_key, &iv, payload_len);
            xor_keystream(&mut packet[header_len..], &keystream);
        }

        // Append E flag (1 bit) and SRTCP index (31 bits)
        // E flag is set to 1 to indicate encryption
        let e_and_index = 0x80000000u32 | srtcp_index;
        packet.extend_from_slice(&e_and_index.to_be_bytes());

        // Compute authentication tag over entire packet (including E|index)
        let auth_tag = compute_auth_tag(
            &self.srtp_auth_key,
            packet,
            self.profile.auth_tag_len(),
        );

        // Append authentication tag
        packet.extend_from_slice(&auth_tag);

        Ok(())
    }

    /// Decrypt RTCP packet in-place
    pub fn decrypt_rtcp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        let tag_len = self.profile.auth_tag_len();

        if packet.len() < tag_len + 8 + 4 {
            return Err(SrtpError::InvalidPacket("SRTCP packet too short".to_string()));
        }

        // Split off authentication tag
        let packet_len = packet.len() - tag_len;
        let auth_tag = packet[packet_len..].to_vec();
        packet.truncate(packet_len);

        // Verify authentication tag
        if !verify_auth_tag(&self.srtp_auth_key, packet, &auth_tag) {
            return Err(SrtpError::AuthenticationFailed);
        }

        // Extract E flag and SRTCP index (last 4 bytes before auth tag)
        let e_and_index_pos = packet.len() - 4;
        let e_and_index = u32::from_be_bytes([
            packet[e_and_index_pos],
            packet[e_and_index_pos + 1],
            packet[e_and_index_pos + 2],
            packet[e_and_index_pos + 3],
        ]);

        // Remove E|index from packet
        packet.truncate(e_and_index_pos);

        // Check E flag
        let e_flag = (e_and_index & 0x80000000) != 0;
        if !e_flag {
            // Packet was not encrypted, just authenticated
            return Ok(());
        }

        // Extract SRTCP index
        let srtcp_index = e_and_index & 0x7FFFFFFF;

        // Parse header
        let ssrc = Self::parse_rtcp_header(packet)?;
        let header_len = Self::get_rtcp_header_len(packet)?;

        // Generate IV
        let iv = generate_iv(&self.srtp_salt, ssrc, srtcp_index as u64);

        // Decrypt payload
        if packet.len() > header_len {
            let payload_len = packet.len() - header_len;
            let keystream = aes_cm_keystream(&self.srtp_cipher_key, &iv, payload_len);
            xor_keystream(&mut packet[header_len..], &keystream);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rtcp_packet(ssrc: u32, payload_size: usize) -> Vec<u8> {
        let mut packet = vec![0u8; 8 + payload_size];

        // RTCP version 2, no padding, report count 0
        packet[0] = 0x80;
        // Packet type (e.g., 200 for SR - Sender Report)
        packet[1] = 200;
        // Length in 32-bit words minus 1
        let length = ((packet.len() - 4) / 4) as u16;
        packet[2..4].copy_from_slice(&length.to_be_bytes());
        // SSRC
        packet[4..8].copy_from_slice(&ssrc.to_be_bytes());
        // Payload (test pattern)
        for i in 0..payload_size {
            packet[8 + i] = (i % 256) as u8;
        }

        packet
    }

    #[test]
    fn test_parse_rtcp_header() {
        let packet = create_test_rtcp_packet(0x12345678, 100);
        let ssrc = SrtcpContext::parse_rtcp_header(&packet).unwrap();
        assert_eq!(ssrc, 0x12345678);
    }

    #[test]
    fn test_get_rtcp_header_len() {
        let packet = create_test_rtcp_packet(0x12345678, 100);
        let header_len = SrtcpContext::get_rtcp_header_len(&packet).unwrap();
        assert_eq!(header_len, 8);
    }

    #[test]
    fn test_srtcp_encrypt_decrypt() {
        let cipher_key = vec![0xAAu8; 16];
        let auth_key = vec![0xBBu8; 20];
        let salt = vec![0xCCu8; 14];

        let ctx = SrtcpContext::new(
            cipher_key,
            auth_key,
            salt,
            SrtpProfile::Aes128CmHmacSha1_80,
        );

        let mut packet = create_test_rtcp_packet(0x12345678, 100);
        let original = packet.clone();

        // Encrypt
        ctx.encrypt_rtcp(&mut packet).unwrap();

        // Packet should be longer (E|index + auth tag appended)
        assert_eq!(packet.len(), original.len() + 4 + 10); // 4 bytes E|index, 10 bytes tag

        // Payload should be encrypted (different)
        assert_ne!(&packet[8..108], &original[8..108]);

        // Decrypt
        ctx.decrypt_rtcp(&mut packet).unwrap();

        // Should match original
        assert_eq!(packet, original);
    }

    #[test]
    fn test_srtcp_authentication_failure() {
        let cipher_key = vec![0xAAu8; 16];
        let auth_key = vec![0xBBu8; 20];
        let salt = vec![0xCCu8; 14];

        let ctx = SrtcpContext::new(
            cipher_key,
            auth_key,
            salt,
            SrtpProfile::Aes128CmHmacSha1_80,
        );

        let mut packet = create_test_rtcp_packet(0x12345678, 100);

        // Encrypt
        ctx.encrypt_rtcp(&mut packet).unwrap();

        // Tamper with packet
        packet[20] ^= 0xFF;

        // Decrypt should fail
        let result = ctx.decrypt_rtcp(&mut packet);
        assert_eq!(result, Err(SrtpError::AuthenticationFailed));
    }

    #[test]
    fn test_srtcp_multiple_packets() {
        let cipher_key = vec![0xAAu8; 16];
        let auth_key = vec![0xBBu8; 20];
        let salt = vec![0xCCu8; 14];

        let ctx = SrtcpContext::new(
            cipher_key,
            auth_key,
            salt,
            SrtpProfile::Aes128CmHmacSha1_80,
        );

        // Encrypt multiple packets
        for i in 0..5 {
            let mut packet = create_test_rtcp_packet(0x12345678 + i, 50);
            let original = packet.clone();

            ctx.encrypt_rtcp(&mut packet).unwrap();
            ctx.decrypt_rtcp(&mut packet).unwrap();

            assert_eq!(packet, original);
        }
    }

    #[test]
    fn test_srtcp_index_increments() {
        let cipher_key = vec![0xAAu8; 16];
        let auth_key = vec![0xBBu8; 20];
        let salt = vec![0xCCu8; 14];

        let ctx = SrtcpContext::new(
            cipher_key,
            auth_key,
            salt,
            SrtpProfile::Aes128CmHmacSha1_80,
        );

        let mut encrypted_packets = Vec::new();

        // Encrypt 3 packets
        for _ in 0..3 {
            let mut packet = create_test_rtcp_packet(0x12345678, 50);
            ctx.encrypt_rtcp(&mut packet).unwrap();
            encrypted_packets.push(packet);
        }

        // Extract E|index from each packet
        for (i, packet) in encrypted_packets.iter().enumerate() {
            let e_and_index_pos = packet.len() - 10 - 4; // before auth tag and E|index
            let e_and_index = u32::from_be_bytes([
                packet[e_and_index_pos],
                packet[e_and_index_pos + 1],
                packet[e_and_index_pos + 2],
                packet[e_and_index_pos + 3],
            ]);

            let index = e_and_index & 0x7FFFFFFF;
            assert_eq!(index, i as u32);
        }
    }
}

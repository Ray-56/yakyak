/// SRTP (Secure Real-time Transport Protocol) implementation
/// RFC 3711
///
/// This module provides encryption and authentication for RTP and RTCP packets.

pub mod crypto;
pub mod context;
pub mod srtcp;

pub use crypto::{
    SrtpProfile, SrtpMasterKey, SrtpSessionKeys,
    derive_session_keys, compute_auth_tag, verify_auth_tag,
};
pub use context::{SrtpContext, SrtpError};
pub use srtcp::SrtcpContext;

/// Combined SRTP/SRTCP context for a media session
pub struct MediaCryptoContext {
    /// SRTP context for RTP packets
    pub srtp: SrtpContext,
    /// SRTCP context for RTCP packets
    pub srtcp: SrtcpContext,
    /// Protection profile
    pub profile: SrtpProfile,
}

impl MediaCryptoContext {
    /// Create new media crypto context from master key
    pub fn new(master_key: SrtpMasterKey, profile: SrtpProfile) -> Self {
        let session_keys = derive_session_keys(&master_key, profile);

        let srtp = SrtpContext::from_session_keys(
            SrtpSessionKeys {
                srtp_cipher_key: session_keys.srtp_cipher_key.clone(),
                srtp_auth_key: session_keys.srtp_auth_key.clone(),
                srtp_salt: session_keys.srtp_salt.clone(),
                srtcp_cipher_key: session_keys.srtcp_cipher_key.clone(),
                srtcp_auth_key: session_keys.srtcp_auth_key.clone(),
                srtcp_salt: session_keys.srtcp_salt.clone(),
            },
            profile,
        );

        let srtcp = SrtcpContext::new(
            session_keys.srtcp_cipher_key,
            session_keys.srtcp_auth_key,
            session_keys.srtcp_salt,
            profile,
        );

        Self {
            srtp,
            srtcp,
            profile,
        }
    }

    /// Encrypt RTP packet
    pub fn protect_rtp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        self.srtp.encrypt_rtp(packet)
    }

    /// Decrypt RTP packet
    pub fn unprotect_rtp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        self.srtp.decrypt_rtp(packet)
    }

    /// Encrypt RTCP packet
    pub fn protect_rtcp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        self.srtcp.encrypt_rtcp(packet)
    }

    /// Decrypt RTCP packet
    pub fn unprotect_rtcp(&self, packet: &mut Vec<u8>) -> Result<(), SrtpError> {
        self.srtcp.decrypt_rtcp(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rtp_packet(ssrc: u32, seq: u16) -> Vec<u8> {
        let mut packet = vec![0u8; 12 + 100];
        packet[0] = 0x80; // V=2
        packet[1] = 96;   // PT
        packet[2..4].copy_from_slice(&seq.to_be_bytes());
        packet[4..8].copy_from_slice(&1000u32.to_be_bytes());
        packet[8..12].copy_from_slice(&ssrc.to_be_bytes());
        packet
    }

    fn create_test_rtcp_packet(ssrc: u32) -> Vec<u8> {
        let mut packet = vec![0u8; 8 + 100];
        packet[0] = 0x80; // V=2
        packet[1] = 200;  // PT=SR
        let length = ((packet.len() - 4) / 4) as u16;
        packet[2..4].copy_from_slice(&length.to_be_bytes());
        packet[4..8].copy_from_slice(&ssrc.to_be_bytes());
        packet
    }

    #[test]
    fn test_media_crypto_context_creation() {
        let master_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
        let ctx = MediaCryptoContext::new(master_key, SrtpProfile::Aes128CmHmacSha1_80);

        assert_eq!(ctx.profile, SrtpProfile::Aes128CmHmacSha1_80);
    }

    #[test]
    fn test_media_crypto_rtp_protection() {
        let master_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
        let ctx = MediaCryptoContext::new(master_key, SrtpProfile::Aes128CmHmacSha1_80);

        let mut packet = create_test_rtp_packet(0x12345678, 1000);
        let original = packet.clone();

        // Protect
        ctx.protect_rtp(&mut packet).unwrap();
        assert_ne!(packet, original);

        // Unprotect
        ctx.unprotect_rtp(&mut packet).unwrap();
        assert_eq!(packet, original);
    }

    #[test]
    fn test_media_crypto_rtcp_protection() {
        let master_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
        let ctx = MediaCryptoContext::new(master_key, SrtpProfile::Aes128CmHmacSha1_80);

        let mut packet = create_test_rtcp_packet(0x12345678);
        let original = packet.clone();

        // Protect
        ctx.protect_rtcp(&mut packet).unwrap();
        assert_ne!(packet, original);

        // Unprotect
        ctx.unprotect_rtcp(&mut packet).unwrap();
        assert_eq!(packet, original);
    }

    #[test]
    fn test_media_crypto_mixed_traffic() {
        let master_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
        let ctx = MediaCryptoContext::new(master_key, SrtpProfile::Aes128CmHmacSha1_80);

        // Protect and unprotect multiple RTP and RTCP packets
        for i in 0..5 {
            let mut rtp = create_test_rtp_packet(0x11111111, 1000 + i);
            let mut rtcp = create_test_rtcp_packet(0x11111111);

            let rtp_orig = rtp.clone();
            let rtcp_orig = rtcp.clone();

            ctx.protect_rtp(&mut rtp).unwrap();
            ctx.protect_rtcp(&mut rtcp).unwrap();

            ctx.unprotect_rtp(&mut rtp).unwrap();
            ctx.unprotect_rtcp(&mut rtcp).unwrap();

            assert_eq!(rtp, rtp_orig);
            assert_eq!(rtcp, rtcp_orig);
        }
    }
}

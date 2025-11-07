/// SRTP (Secure Real-time Transport Protocol) implementation
/// RFC 3711 - The Secure Real-time Transport Protocol (SRTP)

use hmac::{Hmac, Mac};
use sha1::Sha1;
use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, BlockDecrypt, KeyInit,
    generic_array::GenericArray,
};

type HmacSha1 = Hmac<Sha1>;

/// SRTP protection profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrtpProfile {
    /// AES-128-CM with HMAC-SHA1-80 (default)
    Aes128CmHmacSha1_80,
    /// AES-128-CM with HMAC-SHA1-32
    Aes128CmHmacSha1_32,
    /// AES-256-CM with HMAC-SHA1-80
    Aes256CmHmacSha1_80,
    /// AES-256-CM with HMAC-SHA1-32
    Aes256CmHmacSha1_32,
}

impl SrtpProfile {
    /// Get master key length in bytes
    pub fn master_key_len(&self) -> usize {
        match self {
            Self::Aes128CmHmacSha1_80 | Self::Aes128CmHmacSha1_32 => 16,
            Self::Aes256CmHmacSha1_80 | Self::Aes256CmHmacSha1_32 => 32,
        }
    }

    /// Get master salt length in bytes
    pub fn master_salt_len(&self) -> usize {
        14 // All profiles use 112-bit (14-byte) salt
    }

    /// Get auth tag length in bytes
    pub fn auth_tag_len(&self) -> usize {
        match self {
            Self::Aes128CmHmacSha1_80 | Self::Aes256CmHmacSha1_80 => 10, // 80 bits
            Self::Aes128CmHmacSha1_32 | Self::Aes256CmHmacSha1_32 => 4,  // 32 bits
        }
    }

    /// Get cipher key length
    pub fn cipher_key_len(&self) -> usize {
        self.master_key_len()
    }

    /// Get auth key length
    pub fn auth_key_len(&self) -> usize {
        20 // HMAC-SHA1 uses 160-bit (20-byte) key
    }

    /// Get salt key length
    pub fn salt_key_len(&self) -> usize {
        self.master_salt_len()
    }
}

impl Default for SrtpProfile {
    fn default() -> Self {
        Self::Aes128CmHmacSha1_80
    }
}

/// SRTP master key material
#[derive(Clone)]
pub struct SrtpMasterKey {
    /// Master key
    pub key: Vec<u8>,
    /// Master salt
    pub salt: Vec<u8>,
}

impl SrtpMasterKey {
    /// Create new master key material
    pub fn new(key: Vec<u8>, salt: Vec<u8>) -> Self {
        Self { key, salt }
    }

    /// Generate random master key for profile
    pub fn generate(profile: SrtpProfile) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let key: Vec<u8> = (0..profile.master_key_len())
            .map(|_| rng.gen())
            .collect();

        let salt: Vec<u8> = (0..profile.master_salt_len())
            .map(|_| rng.gen())
            .collect();

        Self { key, salt }
    }
}

/// Key derivation label for SRTP key derivation
#[derive(Debug, Clone, Copy)]
enum KeyLabel {
    SrtpEncryption = 0x00,
    SrtpAuthentication = 0x01,
    SrtpSalting = 0x02,
    SrtcpEncryption = 0x03,
    SrtcpAuthentication = 0x04,
    SrtcpSalting = 0x05,
}

/// SRTP session keys derived from master key
#[derive(Clone)]
pub struct SrtpSessionKeys {
    /// Encryption key for SRTP
    pub srtp_cipher_key: Vec<u8>,
    /// Authentication key for SRTP
    pub srtp_auth_key: Vec<u8>,
    /// Salting key for SRTP
    pub srtp_salt: Vec<u8>,
    /// Encryption key for SRTCP
    pub srtcp_cipher_key: Vec<u8>,
    /// Authentication key for SRTCP
    pub srtcp_auth_key: Vec<u8>,
    /// Salting key for SRTCP
    pub srtcp_salt: Vec<u8>,
}

/// Key Derivation Function (KDF) for SRTP
/// Implements the key derivation from RFC 3711 Section 4.3
pub fn srtp_kdf(
    master_key: &[u8],
    master_salt: &[u8],
    label: KeyLabel,
    index: u64,
    output_len: usize,
) -> Vec<u8> {
    // Construct the PRF input
    // key_id = <label> || index || 0x00...
    let mut key_id = vec![0u8; 14];

    // Set label in first byte
    key_id[0] = label as u8;

    // Set index in bytes 1-6 (48-bit index / r value)
    // For most cases, index is 0
    let index_bytes = index.to_be_bytes();
    key_id[1..7].copy_from_slice(&index_bytes[2..8]);

    // XOR key_id with master_salt
    for i in 0..14.min(master_salt.len()) {
        key_id[i] ^= master_salt[i];
    }

    // AES-CM PRF: use AES in counter mode
    let cipher = Aes128::new(GenericArray::from_slice(master_key));

    let mut output = Vec::with_capacity(output_len);
    let mut counter = 0u128;

    while output.len() < output_len {
        // Construct counter block
        let mut counter_block = [0u8; 16];
        counter_block[..14].copy_from_slice(&key_id);

        // Add counter to last 2 bytes
        let counter_bytes = counter.to_be_bytes();
        counter_block[14] = counter_bytes[14];
        counter_block[15] = counter_bytes[15];

        // Encrypt counter block
        let mut block = GenericArray::clone_from_slice(&counter_block);
        cipher.encrypt_block(&mut block);

        // Append to output
        let remaining = output_len - output.len();
        if remaining >= 16 {
            output.extend_from_slice(&block);
        } else {
            output.extend_from_slice(&block[..remaining]);
        }

        counter += 1;
    }

    output
}

/// Derive session keys from master key
pub fn derive_session_keys(
    master_key: &SrtpMasterKey,
    profile: SrtpProfile,
) -> SrtpSessionKeys {
    let index = 0u64; // Typically 0 for initial key derivation

    let srtp_cipher_key = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtpEncryption,
        index,
        profile.cipher_key_len(),
    );

    let srtp_auth_key = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtpAuthentication,
        index,
        profile.auth_key_len(),
    );

    let srtp_salt = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtpSalting,
        index,
        profile.salt_key_len(),
    );

    let srtcp_cipher_key = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtcpEncryption,
        index,
        profile.cipher_key_len(),
    );

    let srtcp_auth_key = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtcpAuthentication,
        index,
        profile.auth_key_len(),
    );

    let srtcp_salt = srtp_kdf(
        &master_key.key,
        &master_key.salt,
        KeyLabel::SrtcpSalting,
        index,
        profile.salt_key_len(),
    );

    SrtpSessionKeys {
        srtp_cipher_key,
        srtp_auth_key,
        srtp_salt,
        srtcp_cipher_key,
        srtcp_auth_key,
        srtcp_salt,
    }
}

/// Compute HMAC-SHA1 authentication tag
pub fn compute_auth_tag(key: &[u8], data: &[u8], tag_len: usize) -> Vec<u8> {
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    bytes[..tag_len].to_vec()
}

/// Verify HMAC-SHA1 authentication tag
pub fn verify_auth_tag(key: &[u8], data: &[u8], expected_tag: &[u8]) -> bool {
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    mac.verify_slice(expected_tag).is_ok()
}

/// Generate IV for AES-CM encryption
pub fn generate_iv(salt: &[u8], ssrc: u32, packet_index: u64) -> [u8; 16] {
    let mut iv = [0u8; 16];

    // Copy salt (first 14 bytes)
    iv[..14].copy_from_slice(&salt[..14]);

    // XOR with SSRC || packet_index
    let ssrc_bytes = ssrc.to_be_bytes();
    let index_bytes = packet_index.to_be_bytes();

    // XOR SSRC into bytes 4-7
    for i in 0..4 {
        iv[4 + i] ^= ssrc_bytes[i];
    }

    // XOR packet index into bytes 8-13 (48-bit index)
    for i in 0..6 {
        iv[8 + i] ^= index_bytes[2 + i];
    }

    iv
}

/// AES Counter Mode encryption/decryption
/// Returns the keystream for XOR operation
pub fn aes_cm_keystream(key: &[u8], iv: &[u8; 16], length: usize) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(key));

    let mut keystream = Vec::with_capacity(length);
    let mut counter = u128::from_be_bytes(*iv);

    while keystream.len() < length {
        let counter_bytes = counter.to_be_bytes();
        let mut block = GenericArray::clone_from_slice(&counter_bytes);

        cipher.encrypt_block(&mut block);

        let remaining = length - keystream.len();
        if remaining >= 16 {
            keystream.extend_from_slice(&block);
        } else {
            keystream.extend_from_slice(&block[..remaining]);
        }

        counter = counter.wrapping_add(1);
    }

    keystream
}

/// XOR data with keystream
pub fn xor_keystream(data: &mut [u8], keystream: &[u8]) {
    for (d, k) in data.iter_mut().zip(keystream.iter()) {
        *d ^= k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srtp_profile_lengths() {
        let profile = SrtpProfile::Aes128CmHmacSha1_80;
        assert_eq!(profile.master_key_len(), 16);
        assert_eq!(profile.master_salt_len(), 14);
        assert_eq!(profile.auth_tag_len(), 10);
    }

    #[test]
    fn test_master_key_generation() {
        let profile = SrtpProfile::Aes128CmHmacSha1_80;
        let key1 = SrtpMasterKey::generate(profile);
        let key2 = SrtpMasterKey::generate(profile);

        assert_eq!(key1.key.len(), 16);
        assert_eq!(key1.salt.len(), 14);
        assert_ne!(key1.key, key2.key); // Should be random
    }

    #[test]
    fn test_key_derivation() {
        let master_key = vec![0x12u8; 16];
        let master_salt = vec![0x34u8; 14];

        let derived = srtp_kdf(
            &master_key,
            &master_salt,
            KeyLabel::SrtpEncryption,
            0,
            16,
        );

        assert_eq!(derived.len(), 16);
        // Derived key should be different from master
        assert_ne!(derived, master_key);
    }

    #[test]
    fn test_session_keys_derivation() {
        let master = SrtpMasterKey {
            key: vec![0xAAu8; 16],
            salt: vec![0xBBu8; 14],
        };

        let profile = SrtpProfile::Aes128CmHmacSha1_80;
        let session_keys = derive_session_keys(&master, profile);

        assert_eq!(session_keys.srtp_cipher_key.len(), 16);
        assert_eq!(session_keys.srtp_auth_key.len(), 20);
        assert_eq!(session_keys.srtp_salt.len(), 14);

        // All keys should be different
        assert_ne!(session_keys.srtp_cipher_key, session_keys.srtp_auth_key);
        assert_ne!(session_keys.srtp_cipher_key, session_keys.srtcp_cipher_key);
    }

    #[test]
    fn test_hmac_auth_tag() {
        let key = vec![0x42u8; 20];
        let data = b"Hello, SRTP!";

        let tag = compute_auth_tag(&key, data, 10);
        assert_eq!(tag.len(), 10);

        // Verify should succeed with correct tag
        assert!(verify_auth_tag(&key, data, &tag));

        // Verify should fail with wrong tag
        let wrong_tag = vec![0u8; 10];
        assert!(!verify_auth_tag(&key, data, &wrong_tag));
    }

    #[test]
    fn test_iv_generation() {
        let salt = vec![0x12u8; 14];
        let ssrc = 0x12345678u32;
        let packet_index = 1000u64;

        let iv = generate_iv(&salt, ssrc, packet_index);
        assert_eq!(iv.len(), 16);

        // IV should be deterministic
        let iv2 = generate_iv(&salt, ssrc, packet_index);
        assert_eq!(iv, iv2);

        // Different index should produce different IV
        let iv3 = generate_iv(&salt, ssrc, packet_index + 1);
        assert_ne!(iv, iv3);
    }

    #[test]
    fn test_aes_cm_keystream() {
        let key = vec![0xAAu8; 16];
        let iv = [0xBBu8; 16];

        let keystream = aes_cm_keystream(&key, &iv, 32);
        assert_eq!(keystream.len(), 32);

        // Keystream should be deterministic
        let keystream2 = aes_cm_keystream(&key, &iv, 32);
        assert_eq!(keystream, keystream2);
    }

    #[test]
    fn test_xor_keystream() {
        let mut data = b"Hello, World!".to_vec();
        let original = data.clone();
        let keystream = vec![0x42u8; data.len()];

        // Encrypt
        xor_keystream(&mut data, &keystream);
        assert_ne!(data, original);

        // Decrypt (XOR is symmetric)
        xor_keystream(&mut data, &keystream);
        assert_eq!(data, original);
    }
}

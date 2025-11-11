//! Simple SDP (Session Description Protocol) handling

use std::net::IpAddr;
use crate::infrastructure::media::srtp::{SrtpMasterKey, SrtpProfile};

/// Simple SDP session
#[derive(Debug, Clone)]
pub struct SdpSession {
    pub version: u32,
    pub origin: SdpOrigin,
    pub session_name: String,
    pub connection: SdpConnection,
    pub media: Vec<SdpMedia>,
}

#[derive(Debug, Clone)]
pub struct SdpOrigin {
    pub username: String,
    pub session_id: String,
    pub session_version: String,
    pub network_type: String,
    pub address_type: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct SdpConnection {
    pub network_type: String,
    pub address_type: String,
    pub address: String,
}

/// SRTP crypto line (SDES)
#[derive(Debug, Clone)]
pub struct SdpCrypto {
    pub tag: u32,
    pub crypto_suite: String,
    pub key_params: String, // base64-encoded key material
    pub session_params: Option<String>,
}

impl SdpCrypto {
    /// Create from SRTP master key
    pub fn from_master_key(tag: u32, master_key: &SrtpMasterKey, profile: SrtpProfile) -> Self {
        // Concatenate master key and salt
        let mut key_material = Vec::new();
        key_material.extend_from_slice(&master_key.key);
        key_material.extend_from_slice(&master_key.salt);

        // Base64 encode
        let key_params = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &key_material);

        let crypto_suite = match profile {
            SrtpProfile::Aes128CmHmacSha1_80 => "AES_CM_128_HMAC_SHA1_80",
            SrtpProfile::Aes128CmHmacSha1_32 => "AES_CM_128_HMAC_SHA1_32",
            SrtpProfile::Aes256CmHmacSha1_80 => "AES_CM_256_HMAC_SHA1_80",
            SrtpProfile::Aes256CmHmacSha1_32 => "AES_CM_256_HMAC_SHA1_32",
        };

        Self {
            tag,
            crypto_suite: crypto_suite.to_string(),
            key_params,
            session_params: None,
        }
    }

    /// Parse from crypto attribute line
    pub fn parse(value: &str) -> Option<Self> {
        // Format: <tag> <crypto-suite> inline:<key-params> [<session-params>]
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let tag = parts[0].parse().ok()?;
        let crypto_suite = parts[1].to_string();

        let inline_part = parts[2];
        if !inline_part.starts_with("inline:") {
            return None;
        }
        let key_params = inline_part[7..].to_string();

        let session_params = if parts.len() > 3 {
            Some(parts[3..].join(" "))
        } else {
            None
        };

        Some(Self {
            tag,
            crypto_suite,
            key_params,
            session_params,
        })
    }

    /// Convert to master key
    pub fn to_master_key(&self) -> Option<(SrtpMasterKey, SrtpProfile)> {
        // Decode base64
        let key_material = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &self.key_params
        ).ok()?;

        // Determine profile and key/salt lengths
        let profile = match self.crypto_suite.as_str() {
            "AES_CM_128_HMAC_SHA1_80" => SrtpProfile::Aes128CmHmacSha1_80,
            "AES_CM_128_HMAC_SHA1_32" => SrtpProfile::Aes128CmHmacSha1_32,
            "AES_CM_256_HMAC_SHA1_80" => SrtpProfile::Aes256CmHmacSha1_80,
            "AES_CM_256_HMAC_SHA1_32" => SrtpProfile::Aes256CmHmacSha1_32,
            _ => return None,
        };

        let key_len = profile.master_key_len();
        let salt_len = profile.master_salt_len();

        if key_material.len() < key_len + salt_len {
            return None;
        }

        let key = key_material[..key_len].to_vec();
        let salt = key_material[key_len..key_len + salt_len].to_vec();

        Some((SrtpMasterKey::new(key, salt), profile))
    }

    /// Convert to attribute string
    pub fn to_string(&self) -> String {
        let mut result = format!("{} {} inline:{}", self.tag, self.crypto_suite, self.key_params);
        if let Some(ref params) = self.session_params {
            result.push(' ');
            result.push_str(params);
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct SdpMedia {
    pub media_type: String,  // "audio", "video"
    pub port: u16,
    pub protocol: String,    // "RTP/AVP" or "RTP/SAVP"
    pub formats: Vec<String>, // Codec payload types
    pub rtpmap: Vec<(String, String)>, // (payload_type, encoding)
    pub crypto: Vec<SdpCrypto>, // SRTP crypto lines
}

impl SdpSession {
    /// Create a simple audio SDP
    pub fn create_audio_session(local_ip: IpAddr, local_port: u16) -> Self {
        Self {
            version: 0,
            origin: SdpOrigin {
                username: "yakyak".to_string(),
                session_id: chrono::Utc::now().timestamp().to_string(),
                session_version: "1".to_string(),
                network_type: "IN".to_string(),
                address_type: if local_ip.is_ipv4() { "IP4" } else { "IP6" }.to_string(),
                address: local_ip.to_string(),
            },
            session_name: "YakYak Call".to_string(),
            connection: SdpConnection {
                network_type: "IN".to_string(),
                address_type: if local_ip.is_ipv4() { "IP4" } else { "IP6" }.to_string(),
                address: local_ip.to_string(),
            },
            media: vec![SdpMedia {
                media_type: "audio".to_string(),
                port: local_port,
                protocol: "RTP/AVP".to_string(),
                formats: vec!["0".to_string(), "8".to_string(), "101".to_string()],
                rtpmap: vec![
                    ("0".to_string(), "PCMU/8000".to_string()),
                    ("8".to_string(), "PCMA/8000".to_string()),
                    ("101".to_string(), "telephone-event/8000".to_string()),
                ],
                crypto: Vec::new(),
            }],
        }
    }

    /// Convert to SDP string
    pub fn to_string(&self) -> String {
        let mut sdp = String::new();

        // Version
        sdp.push_str(&format!("v={}\r\n", self.version));

        // Origin
        sdp.push_str(&format!(
            "o={} {} {} {} {} {}\r\n",
            self.origin.username,
            self.origin.session_id,
            self.origin.session_version,
            self.origin.network_type,
            self.origin.address_type,
            self.origin.address
        ));

        // Session name
        sdp.push_str(&format!("s={}\r\n", self.session_name));

        // Connection
        sdp.push_str(&format!(
            "c={} {} {}\r\n",
            self.connection.network_type, self.connection.address_type, self.connection.address
        ));

        // Time
        sdp.push_str("t=0 0\r\n");

        // Media descriptions
        for media in &self.media {
            sdp.push_str(&format!(
                "m={} {} {} {}\r\n",
                media.media_type,
                media.port,
                media.protocol,
                media.formats.join(" ")
            ));

            // Crypto (SRTP)
            for crypto in &media.crypto {
                sdp.push_str(&format!("a=crypto:{}\r\n", crypto.to_string()));
            }

            // RTP map
            for (pt, encoding) in &media.rtpmap {
                sdp.push_str(&format!("a=rtpmap:{} {}\r\n", pt, encoding));
            }

            // Send/receive
            sdp.push_str("a=sendrecv\r\n");
        }

        sdp
    }

    /// Parse SDP from string
    pub fn parse(sdp_body: &str) -> Option<Self> {
        let mut version = 0;
        let mut origin: Option<SdpOrigin> = None;
        let mut session_name = String::new();
        let mut connection: Option<SdpConnection> = None;
        let mut media: Vec<SdpMedia> = Vec::new();
        let mut current_media: Option<SdpMedia> = None;

        for line in sdp_body.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip \r if present
            let line = line.trim_end_matches('\r');

            if line.len() < 2 || !line.contains('=') {
                continue;
            }

            let (field_type, value) = line.split_at(2);
            let value = value.trim();

            match field_type {
                "v=" => {
                    version = value.parse().unwrap_or(0);
                }
                "o=" => {
                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if parts.len() >= 6 {
                        origin = Some(SdpOrigin {
                            username: parts[0].to_string(),
                            session_id: parts[1].to_string(),
                            session_version: parts[2].to_string(),
                            network_type: parts[3].to_string(),
                            address_type: parts[4].to_string(),
                            address: parts[5].to_string(),
                        });
                    }
                }
                "s=" => {
                    session_name = value.to_string();
                }
                "c=" => {
                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let conn = SdpConnection {
                            network_type: parts[0].to_string(),
                            address_type: parts[1].to_string(),
                            address: parts[2].to_string(),
                        };

                        if current_media.is_none() {
                            // Session-level connection
                            connection = Some(conn);
                        }
                        // Media-level connection is currently ignored
                    }
                }
                "m=" => {
                    // Save previous media if any
                    if let Some(m) = current_media.take() {
                        media.push(m);
                    }

                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let media_type = parts[0].to_string();
                        let port = parts[1].parse().unwrap_or(0);
                        let protocol = parts[2].to_string();
                        let formats: Vec<String> = parts[3..].iter().map(|s| s.to_string()).collect();

                        current_media = Some(SdpMedia {
                            media_type,
                            port,
                            protocol,
                            formats,
                            rtpmap: Vec::new(),
                            crypto: Vec::new(),
                        });
                    }
                }
                "a=" => {
                    // Parse attributes
                    if let Some(media) = current_media.as_mut() {
                        if value.starts_with("rtpmap:") {
                            let rtpmap_value = &value[7..]; // Skip "rtpmap:"
                            if let Some(space_pos) = rtpmap_value.find(' ') {
                                let pt = rtpmap_value[..space_pos].to_string();
                                let encoding = rtpmap_value[space_pos + 1..].to_string();
                                media.rtpmap.push((pt, encoding));
                            }
                        } else if value.starts_with("crypto:") {
                            let crypto_value = &value[7..]; // Skip "crypto:"
                            if let Some(crypto) = SdpCrypto::parse(crypto_value) {
                                media.crypto.push(crypto);
                            }
                        }
                    }
                }
                _ => {
                    // Ignore other fields
                }
            }
        }

        // Save last media if any
        if let Some(m) = current_media.take() {
            media.push(m);
        }

        // Validate required fields
        let origin = origin?;
        let connection = connection?;

        Some(Self {
            version,
            origin,
            session_name,
            connection,
            media,
        })
    }

    /// Get media description for audio
    pub fn audio_media(&self) -> Option<&SdpMedia> {
        self.media.iter().find(|m| m.media_type == "audio")
    }

    /// Get supported codecs
    pub fn audio_codecs(&self) -> Vec<u8> {
        if let Some(audio) = self.audio_media() {
            audio
                .formats
                .iter()
                .filter_map(|f| f.parse::<u8>().ok())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Add SRTP crypto to audio media
    pub fn add_srtp_crypto(&mut self, master_key: &SrtpMasterKey, profile: SrtpProfile) {
        if let Some(media) = self.media.iter_mut().find(|m| m.media_type == "audio") {
            // Change protocol to RTP/SAVP for SRTP
            media.protocol = "RTP/SAVP".to_string();

            // Add crypto line
            let tag = (media.crypto.len() + 1) as u32;
            media.crypto.push(SdpCrypto::from_master_key(tag, master_key, profile));
        }
    }

    /// Get SRTP crypto from audio media
    pub fn get_srtp_crypto(&self) -> Option<(SrtpMasterKey, SrtpProfile)> {
        if let Some(audio) = self.audio_media() {
            // Get first crypto line (highest priority)
            audio.crypto.first()?.to_master_key()
        } else {
            None
        }
    }

    /// Check if SRTP is enabled
    pub fn is_srtp_enabled(&self) -> bool {
        if let Some(audio) = self.audio_media() {
            !audio.crypto.is_empty() && audio.protocol.contains("SAVP")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sdp() {
        let local_ip: IpAddr = "192.168.1.100".parse().unwrap();
        let sdp = SdpSession::create_audio_session(local_ip, 10000);

        let sdp_str = sdp.to_string();
        assert!(sdp_str.contains("v=0"));
        assert!(sdp_str.contains("m=audio 10000"));
        assert!(sdp_str.contains("PCMU"));
    }

    #[test]
    fn test_parse_sdp() {
        let sdp_str = r#"v=0
o=user1 123456 7890 IN IP4 192.168.1.100
s=Test Session
c=IN IP4 192.168.1.100
t=0 0
m=audio 10000 RTP/AVP 0 8
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
"#;

        let sdp = SdpSession::parse(sdp_str).unwrap();
        assert_eq!(sdp.version, 0);
        assert_eq!(sdp.session_name, "Test Session");
        assert_eq!(sdp.origin.username, "user1");
        assert_eq!(sdp.connection.address, "192.168.1.100");
        assert_eq!(sdp.media.len(), 1);

        let audio = sdp.audio_media().unwrap();
        assert_eq!(audio.media_type, "audio");
        assert_eq!(audio.port, 10000);
        assert_eq!(audio.formats, vec!["0", "8"]);
        assert_eq!(audio.rtpmap.len(), 2);

        let codecs = sdp.audio_codecs();
        assert_eq!(codecs, vec![0, 8]);
    }

    #[test]
    fn test_parse_generate_roundtrip() {
        let local_ip: IpAddr = "10.0.0.5".parse().unwrap();
        let original = SdpSession::create_audio_session(local_ip, 20000);
        let sdp_str = original.to_string();

        let parsed = SdpSession::parse(&sdp_str).unwrap();
        assert_eq!(parsed.version, 0);
        assert_eq!(parsed.connection.address, "10.0.0.5");

        let audio = parsed.audio_media().unwrap();
        assert_eq!(audio.port, 20000);
        assert_eq!(audio.media_type, "audio");
    }

    #[test]
    fn test_sdp_crypto_parse() {
        let crypto_str = "1 AES_CM_128_HMAC_SHA1_80 inline:d0RmdmcmVCspeEc3QGZiNWpVLFJhQX1cfHAwJSoj";
        let crypto = SdpCrypto::parse(crypto_str).unwrap();

        assert_eq!(crypto.tag, 1);
        assert_eq!(crypto.crypto_suite, "AES_CM_128_HMAC_SHA1_80");
        assert_eq!(crypto.key_params, "d0RmdmcmVCspeEc3QGZiNWpVLFJhQX1cfHAwJSoj");
    }

    #[test]
    fn test_sdp_crypto_roundtrip() {
        use crate::infrastructure::media::srtp::{SrtpMasterKey, SrtpProfile};

        let profile = SrtpProfile::Aes128CmHmacSha1_80;
        let master_key = SrtpMasterKey::generate(profile);

        // Create crypto line
        let crypto = SdpCrypto::from_master_key(1, &master_key, profile);

        // Convert back to master key
        let (decoded_key, decoded_profile) = crypto.to_master_key().unwrap();

        assert_eq!(decoded_profile, profile);
        assert_eq!(decoded_key.key, master_key.key);
        assert_eq!(decoded_key.salt, master_key.salt);
    }

    #[test]
    fn test_sdp_with_srtp() {
        use crate::infrastructure::media::srtp::{SrtpMasterKey, SrtpProfile};

        let local_ip: IpAddr = "192.168.1.100".parse().unwrap();
        let mut sdp = SdpSession::create_audio_session(local_ip, 10000);

        // Initially no SRTP
        assert!(!sdp.is_srtp_enabled());

        // Add SRTP
        let profile = SrtpProfile::Aes128CmHmacSha1_80;
        let master_key = SrtpMasterKey::generate(profile);
        sdp.add_srtp_crypto(&master_key, profile);

        // Check SRTP enabled
        assert!(sdp.is_srtp_enabled());

        // Convert to string
        let sdp_str = sdp.to_string();
        assert!(sdp_str.contains("RTP/SAVP"));
        assert!(sdp_str.contains("a=crypto:1 AES_CM_128_HMAC_SHA1_80 inline:"));

        // Parse back
        let parsed = SdpSession::parse(&sdp_str).unwrap();
        assert!(parsed.is_srtp_enabled());

        // Get crypto
        let (decoded_key, decoded_profile) = parsed.get_srtp_crypto().unwrap();
        assert_eq!(decoded_profile, profile);
        assert_eq!(decoded_key.key, master_key.key);
        assert_eq!(decoded_key.salt, master_key.salt);
    }

    #[test]
    fn test_parse_sdp_with_crypto() {
        let sdp_str = r#"v=0
o=user1 123456 7890 IN IP4 192.168.1.100
s=Test Session
c=IN IP4 192.168.1.100
t=0 0
m=audio 10000 RTP/SAVP 0 8
a=crypto:1 AES_CM_128_HMAC_SHA1_80 inline:d0RmdmcmVCspeEc3QGZiNWpVLFJhQX1cfHAwJSoj
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
"#;

        let sdp = SdpSession::parse(sdp_str).unwrap();
        assert!(sdp.is_srtp_enabled());

        let audio = sdp.audio_media().unwrap();
        assert_eq!(audio.protocol, "RTP/SAVP");
        assert_eq!(audio.crypto.len(), 1);

        let crypto = &audio.crypto[0];
        assert_eq!(crypto.tag, 1);
        assert_eq!(crypto.crypto_suite, "AES_CM_128_HMAC_SHA1_80");
    }
}

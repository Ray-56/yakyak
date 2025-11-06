/// WebRTC SDP (Session Description Protocol) support
use crate::infrastructure::protocols::ice::candidate::IceCandidate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SDP session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdpType {
    Offer,
    Answer,
    Pranswer,
}

impl SdpType {
    pub fn to_string(&self) -> &str {
        match self {
            SdpType::Offer => "offer",
            SdpType::Answer => "answer",
            SdpType::Pranswer => "pranswer",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "offer" => Some(SdpType::Offer),
            "answer" => Some(SdpType::Answer),
            "pranswer" => Some(SdpType::Pranswer),
            _ => None,
        }
    }
}

/// Media direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaDirection {
    SendRecv,
    SendOnly,
    RecvOnly,
    Inactive,
}

impl MediaDirection {
    pub fn to_string(&self) -> &str {
        match self {
            MediaDirection::SendRecv => "sendrecv",
            MediaDirection::SendOnly => "sendonly",
            MediaDirection::RecvOnly => "recvonly",
            MediaDirection::Inactive => "inactive",
        }
    }
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Audio,
    Video,
    Application,
}

impl MediaType {
    pub fn to_string(&self) -> &str {
        match self {
            MediaType::Audio => "audio",
            MediaType::Video => "video",
            MediaType::Application => "application",
        }
    }
}

/// RTP codec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtpCodec {
    pub payload_type: u8,
    pub name: String,
    pub clock_rate: u32,
    pub channels: Option<u8>,
    pub parameters: HashMap<String, String>,
}

impl RtpCodec {
    pub fn new(payload_type: u8, name: String, clock_rate: u32) -> Self {
        Self {
            payload_type,
            name,
            clock_rate,
            channels: None,
            parameters: HashMap::new(),
        }
    }

    /// Create PCMU codec
    pub fn pcmu() -> Self {
        Self::new(0, "PCMU".to_string(), 8000)
    }

    /// Create PCMA codec
    pub fn pcma() -> Self {
        Self::new(8, "PCMA".to_string(), 8000)
    }

    /// Create Opus codec
    pub fn opus() -> Self {
        Self {
            payload_type: 111,
            name: "opus".to_string(),
            clock_rate: 48000,
            channels: Some(2),
            parameters: HashMap::new(),
        }
    }

    /// Create VP8 codec
    pub fn vp8() -> Self {
        Self::new(96, "VP8".to_string(), 90000)
    }

    /// To rtpmap format: "96 VP8/90000"
    pub fn to_rtpmap(&self) -> String {
        if let Some(channels) = self.channels {
            format!("{} {}/{}/{}", self.payload_type, self.name, self.clock_rate, channels)
        } else {
            format!("{} {}/{}", self.payload_type, self.name, self.clock_rate)
        }
    }
}

/// Media description
#[derive(Debug, Clone)]
pub struct MediaDescription {
    pub media_type: MediaType,
    pub port: u16,
    pub protocol: String,
    pub codecs: Vec<RtpCodec>,
    pub direction: MediaDirection,
    pub ice_ufrag: Option<String>,
    pub ice_pwd: Option<String>,
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_fingerprint: Option<DtlsFingerprint>,
    pub dtls_setup: Option<DtlsSetup>,
    pub rtcp_mux: bool,
    pub mid: Option<String>,
}

impl MediaDescription {
    pub fn new(media_type: MediaType, port: u16) -> Self {
        Self {
            media_type,
            port,
            protocol: "UDP/TLS/RTP/SAVPF".to_string(), // WebRTC default
            codecs: Vec::new(),
            direction: MediaDirection::SendRecv,
            ice_ufrag: None,
            ice_pwd: None,
            ice_candidates: Vec::new(),
            dtls_fingerprint: None,
            dtls_setup: None,
            rtcp_mux: true,
            mid: None,
        }
    }

    /// Add codec
    pub fn add_codec(&mut self, codec: RtpCodec) {
        self.codecs.push(codec);
    }

    /// Set ICE credentials
    pub fn set_ice_credentials(&mut self, ufrag: String, pwd: String) {
        self.ice_ufrag = Some(ufrag);
        self.ice_pwd = Some(pwd);
    }

    /// Add ICE candidate
    pub fn add_ice_candidate(&mut self, candidate: IceCandidate) {
        self.ice_candidates.push(candidate);
    }
}

/// DTLS fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtlsFingerprint {
    pub algorithm: String,
    pub value: String,
}

impl DtlsFingerprint {
    pub fn new(algorithm: String, value: String) -> Self {
        Self { algorithm, value }
    }

    /// SHA-256 fingerprint
    pub fn sha256(value: String) -> Self {
        Self::new("sha-256".to_string(), value)
    }

    /// To SDP format: "sha-256 AA:BB:CC:..."
    pub fn to_sdp(&self) -> String {
        format!("{} {}", self.algorithm, self.value)
    }
}

/// DTLS setup role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DtlsSetup {
    Active,
    Passive,
    Actpass,
}

impl DtlsSetup {
    pub fn to_string(&self) -> &str {
        match self {
            DtlsSetup::Active => "active",
            DtlsSetup::Passive => "passive",
            DtlsSetup::Actpass => "actpass",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "active" => Some(DtlsSetup::Active),
            "passive" => Some(DtlsSetup::Passive),
            "actpass" => Some(DtlsSetup::Actpass),
            _ => None,
        }
    }
}

/// WebRTC Session Description
#[derive(Debug, Clone)]
pub struct WebRtcSdp {
    pub sdp_type: SdpType,
    pub session_id: String,
    pub session_version: u64,
    pub origin_username: String,
    pub origin_address: String,
    pub session_name: String,
    pub media_descriptions: Vec<MediaDescription>,
    pub ice_lite: bool,
    pub bundle_group: Option<Vec<String>>,
}

impl WebRtcSdp {
    /// Create a new WebRTC SDP
    pub fn new(sdp_type: SdpType) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sdp_type,
            session_id: format!("{}", timestamp),
            session_version: timestamp,
            origin_username: "-".to_string(),
            origin_address: "0.0.0.0".to_string(),
            session_name: "YakYak WebRTC Session".to_string(),
            media_descriptions: Vec::new(),
            ice_lite: false,
            bundle_group: None,
        }
    }

    /// Add media description
    pub fn add_media(&mut self, media: MediaDescription) {
        self.media_descriptions.push(media);
    }

    /// Enable BUNDLE
    pub fn enable_bundle(&mut self) {
        let mids: Vec<String> = self.media_descriptions
            .iter()
            .filter_map(|m| m.mid.clone())
            .collect();

        if !mids.is_empty() {
            self.bundle_group = Some(mids);
        }
    }

    /// Convert to SDP string
    pub fn to_sdp_string(&self) -> String {
        let mut sdp = String::new();

        // Session description
        sdp.push_str("v=0\r\n");
        sdp.push_str(&format!(
            "o={} {} {} IN IP4 {}\r\n",
            self.origin_username,
            self.session_id,
            self.session_version,
            self.origin_address
        ));
        sdp.push_str(&format!("s={}\r\n", self.session_name));
        sdp.push_str("t=0 0\r\n");

        // BUNDLE group
        if let Some(ref bundle) = self.bundle_group {
            sdp.push_str(&format!("a=group:BUNDLE {}\r\n", bundle.join(" ")));
        }

        // ICE lite
        if self.ice_lite {
            sdp.push_str("a=ice-lite\r\n");
        }

        // Media descriptions
        for media in &self.media_descriptions {
            self.add_media_to_sdp(&mut sdp, media);
        }

        sdp
    }

    /// Add media description to SDP string
    fn add_media_to_sdp(&self, sdp: &mut String, media: &MediaDescription) {
        // m= line
        let payload_types: Vec<String> = media.codecs
            .iter()
            .map(|c| c.payload_type.to_string())
            .collect();

        sdp.push_str(&format!(
            "m={} {} {} {}\r\n",
            media.media_type.to_string(),
            media.port,
            media.protocol,
            payload_types.join(" ")
        ));

        // c= line
        sdp.push_str("c=IN IP4 0.0.0.0\r\n");

        // a=rtcp-mux
        if media.rtcp_mux {
            sdp.push_str("a=rtcp-mux\r\n");
        }

        // a=mid
        if let Some(ref mid) = media.mid {
            sdp.push_str(&format!("a=mid:{}\r\n", mid));
        }

        // Direction
        sdp.push_str(&format!("a={}\r\n", media.direction.to_string()));

        // ICE credentials
        if let Some(ref ufrag) = media.ice_ufrag {
            sdp.push_str(&format!("a=ice-ufrag:{}\r\n", ufrag));
        }
        if let Some(ref pwd) = media.ice_pwd {
            sdp.push_str(&format!("a=ice-pwd:{}\r\n", pwd));
        }

        // DTLS
        if let Some(ref fingerprint) = media.dtls_fingerprint {
            sdp.push_str(&format!("a=fingerprint:{}\r\n", fingerprint.to_sdp()));
        }
        if let Some(ref setup) = media.dtls_setup {
            sdp.push_str(&format!("a=setup:{}\r\n", setup.to_string()));
        }

        // Codecs (rtpmap)
        for codec in &media.codecs {
            sdp.push_str(&format!("a=rtpmap:{}\r\n", codec.to_rtpmap()));

            // fmtp if parameters exist
            if !codec.parameters.is_empty() {
                let params: Vec<String> = codec.parameters
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                sdp.push_str(&format!("a=fmtp:{} {}\r\n", codec.payload_type, params.join(";")));
            }
        }

        // ICE candidates
        for candidate in &media.ice_candidates {
            sdp.push_str(&format!("a={}\r\n", candidate.to_sdp()));
        }
    }

    /// Parse from SDP string (basic implementation)
    pub fn from_sdp_string(sdp: &str, sdp_type: SdpType) -> Result<Self, String> {
        let mut webrtc_sdp = Self::new(sdp_type);

        // Basic parsing - in production would use a proper SDP parser
        for line in sdp.lines() {
            if line.starts_with("s=") {
                webrtc_sdp.session_name = line[2..].to_string();
            }
            // Add more parsing as needed
        }

        Ok(webrtc_sdp)
    }
}

/// Create audio-only offer
pub fn create_audio_offer(ice_ufrag: String, ice_pwd: String) -> WebRtcSdp {
    let mut offer = WebRtcSdp::new(SdpType::Offer);

    let mut audio = MediaDescription::new(MediaType::Audio, 9);
    audio.mid = Some("0".to_string());
    audio.set_ice_credentials(ice_ufrag, ice_pwd);

    // Add codecs
    audio.add_codec(RtpCodec::opus());
    audio.add_codec(RtpCodec::pcmu());
    audio.add_codec(RtpCodec::pcma());

    offer.add_media(audio);
    offer.enable_bundle();

    offer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_type() {
        assert_eq!(SdpType::Offer.to_string(), "offer");
        assert_eq!(SdpType::Answer.to_string(), "answer");
        assert_eq!(SdpType::from_string("offer"), Some(SdpType::Offer));
    }

    #[test]
    fn test_codec_creation() {
        let opus = RtpCodec::opus();
        assert_eq!(opus.name, "opus");
        assert_eq!(opus.clock_rate, 48000);
        assert_eq!(opus.channels, Some(2));

        let pcmu = RtpCodec::pcmu();
        assert_eq!(pcmu.payload_type, 0);
        assert_eq!(pcmu.name, "PCMU");
    }

    #[test]
    fn test_codec_rtpmap() {
        let opus = RtpCodec::opus();
        assert_eq!(opus.to_rtpmap(), "111 opus/48000/2");

        let pcmu = RtpCodec::pcmu();
        assert_eq!(pcmu.to_rtpmap(), "0 PCMU/8000");
    }

    #[test]
    fn test_media_description() {
        let mut media = MediaDescription::new(MediaType::Audio, 9);
        media.mid = Some("0".to_string());
        media.set_ice_credentials("test_ufrag".to_string(), "test_pwd".to_string());
        media.add_codec(RtpCodec::opus());

        assert_eq!(media.media_type, MediaType::Audio);
        assert_eq!(media.port, 9);
        assert_eq!(media.ice_ufrag, Some("test_ufrag".to_string()));
        assert_eq!(media.codecs.len(), 1);
    }

    #[test]
    fn test_dtls_fingerprint() {
        let fp = DtlsFingerprint::sha256("AA:BB:CC:DD".to_string());
        assert_eq!(fp.algorithm, "sha-256");
        assert_eq!(fp.to_sdp(), "sha-256 AA:BB:CC:DD");
    }

    #[test]
    fn test_webrtc_sdp_creation() {
        let sdp = WebRtcSdp::new(SdpType::Offer);
        assert_eq!(sdp.sdp_type, SdpType::Offer);
        assert_eq!(sdp.media_descriptions.len(), 0);
    }

    #[test]
    fn test_audio_offer_creation() {
        let offer = create_audio_offer("ufrag123".to_string(), "pwd456".to_string());

        assert_eq!(offer.sdp_type, SdpType::Offer);
        assert_eq!(offer.media_descriptions.len(), 1);

        let audio = &offer.media_descriptions[0];
        assert_eq!(audio.media_type, MediaType::Audio);
        assert_eq!(audio.ice_ufrag, Some("ufrag123".to_string()));
        assert_eq!(audio.codecs.len(), 3); // Opus, PCMU, PCMA
    }

    #[test]
    fn test_sdp_string_generation() {
        let offer = create_audio_offer("ufrag123".to_string(), "pwd456".to_string());
        let sdp_string = offer.to_sdp_string();

        assert!(sdp_string.contains("v=0"));
        assert!(sdp_string.contains("m=audio"));
        assert!(sdp_string.contains("a=ice-ufrag:ufrag123"));
        assert!(sdp_string.contains("a=ice-pwd:pwd456"));
        assert!(sdp_string.contains("a=rtpmap:111 opus/48000/2"));
        assert!(sdp_string.contains("a=group:BUNDLE"));
    }

    #[test]
    fn test_bundle_enable() {
        let mut offer = WebRtcSdp::new(SdpType::Offer);

        let mut audio = MediaDescription::new(MediaType::Audio, 9);
        audio.mid = Some("0".to_string());
        offer.add_media(audio);

        let mut video = MediaDescription::new(MediaType::Video, 9);
        video.mid = Some("1".to_string());
        offer.add_media(video);

        offer.enable_bundle();

        assert!(offer.bundle_group.is_some());
        let bundle = offer.bundle_group.unwrap();
        assert_eq!(bundle.len(), 2);
        assert!(bundle.contains(&"0".to_string()));
        assert!(bundle.contains(&"1".to_string()));
    }
}

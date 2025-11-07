//! WebRTC protocol implementation
pub mod sdp;

pub use sdp::{
    WebRtcSdp, SdpType, MediaDescription, MediaType, MediaDirection,
    RtpCodec, DtlsFingerprint, DtlsSetup, create_audio_offer,
};

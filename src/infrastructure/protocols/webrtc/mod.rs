//! WebRTC protocol implementation
pub mod sdp;
pub mod session_manager;

pub use sdp::{
    WebRtcSdp, SdpType, MediaDescription, MediaType, MediaDirection,
    RtpCodec, DtlsFingerprint, DtlsSetup, create_audio_offer,
};
pub use session_manager::{WebRtcSession, WebRtcSessionManager, SessionState};

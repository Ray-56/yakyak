///! Protocol bridges for interoperability
///!
///! This module contains bridges between different protocols to enable
///! seamless communication between various client types.

pub mod webrtc_sip;

pub use webrtc_sip::{WebRtcSipBridge, BridgeSession, BridgeState};

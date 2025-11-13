//! SIP protocol implementation
//!
//! This module implements a SIP (Session Initiation Protocol) stack based on RFC 3261.
//!
//! Architecture:
//! ```
//! ┌─────────────────────────┐
//! │   Application Layer     │
//! │  (User Agent, Server)   │
//! └───────────┬─────────────┘
//!             │
//! ┌───────────▼─────────────┐
//! │    Dialog Layer         │
//! │   (SIP Dialogs)         │
//! └───────────┬─────────────┘
//!             │
//! ┌───────────▼─────────────┐
//! │   Transaction Layer     │
//! │  (Client/Server TXN)    │
//! └───────────┬─────────────┘
//!             │
//! ┌───────────▼─────────────┐
//! │   Transport Layer       │
//! │  (UDP, TCP, TLS, WS)    │
//! └─────────────────────────┘
//! ```

pub mod auth;
#[cfg(feature = "postgres")]
pub mod auth_db;
pub mod auth_enhanced;
pub mod builder;
pub mod call_handler;
pub mod call_router;
pub mod call_state;
pub mod dialog;
pub mod handler;
pub mod hold_manager;
pub mod message;
// Temporarily disabled - under development
// pub mod message_handler;
// pub mod notify_handler;
// pub mod refer_handler;
pub mod registrar;
pub mod rport;
pub mod sdp;
pub mod server;
// pub mod subscribe_handler;
pub mod transaction;
pub mod transport;

pub use auth::{AuthChallenge, DigestAuth, SipAuthenticator, UserCredentials};
#[cfg(feature = "postgres")]
pub use auth_db::DigestAuthDb;
pub use call_handler::{AckHandler, ByeHandler, CallSession, CancelHandler, InviteHandler};
pub use call_router::{ActiveCallInfo, BridgedCall, CallLegInfo, CallRouter};
pub use call_state::{CallDirection, CallEvent, CallLeg, CallState, CallStateMachine, CallStats};
pub use message::{SipMessage, SipMethod, SipRequest, SipResponse};
pub use registrar::Registrar;
pub use sdp::SdpSession;
pub use server::{SipServer, SipServerConfig};
pub use transaction::{
    InviteClientState, InviteServerState, NonInviteClientState, NonInviteServerState,
    SipTimers, TimerType, Transaction, TransactionId, TransactionLayer, TransactionState,
    TransactionTimerAction,
};
pub use transport::{Transport, TransportProtocol};

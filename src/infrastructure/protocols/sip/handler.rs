//! SIP message handlers

use super::message::{SipError, SipMethod, SipRequest, SipResponse};
use async_trait::async_trait;

/// Trait for handling SIP requests
#[async_trait]
pub trait SipHandler: Send + Sync {
    /// Handle a SIP request
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError>;

    /// Check if this handler can handle the given method
    fn can_handle(&self, method: SipMethod) -> bool;
}

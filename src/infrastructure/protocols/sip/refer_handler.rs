/// REFER handler for call transfer (Blind Transfer)
use async_trait::async_trait;
use rsip::{Request, Response, SipMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::handler::SipHandler;
use super::message::SipMessageBuilder;
use super::call_router::CallRouter;

/// REFER handler for blind call transfer
pub struct ReferHandler {
    call_router: Arc<RwLock<CallRouter>>,
}

impl ReferHandler {
    /// Create a new REFER handler
    pub fn new(call_router: Arc<RwLock<CallRouter>>) -> Self {
        Self { call_router }
    }

    /// Extract Refer-To header from REFER request
    fn extract_refer_to(request: &Request) -> Option<String> {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "refer-to")
            .and_then(|h| h.value().to_string().ok())
    }

    /// Extract Referred-By header from REFER request
    fn extract_referred_by(request: &Request) -> Option<String> {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "referred-by")
            .and_then(|h| h.value().to_string().ok())
    }
}

#[async_trait]
impl SipHandler for ReferHandler {
    async fn handle(&self, request: Request, source: SocketAddr) -> Option<Response> {
        info!("Handling REFER request from {}", source);

        // Extract Call-ID to identify the call being transferred
        let call_id = request
            .call_id_header()
            .ok()
            .and_then(|h| h.value().to_string().ok())?;

        debug!("REFER for Call-ID: {}", call_id);

        // Extract Refer-To header (transfer target)
        let refer_to = match Self::extract_refer_to(&request) {
            Some(target) => target,
            None => {
                warn!("REFER request missing Refer-To header");
                return Some(SipMessageBuilder::create_response(
                    &request,
                    400,
                    "Bad Request - Missing Refer-To header",
                ));
            }
        };

        info!("Transfer target: {}", refer_to);

        // Extract Referred-By header (optional)
        let referred_by = Self::extract_referred_by(&request);
        if let Some(ref referrer) = referred_by {
            debug!("Referred by: {}", referrer);
        }

        // Check if call exists
        let router = self.call_router.read().await;
        if !router.has_call(&call_id).await {
            warn!("REFER request for non-existent call: {}", call_id);
            return Some(SipMessageBuilder::create_response(
                &request,
                481,
                "Call/Transaction Does Not Exist",
            ));
        }
        drop(router);

        // Accept the REFER request
        info!("Accepting REFER request for Call-ID: {}", call_id);

        // TODO: Implement actual call transfer logic
        // 1. Send NOTIFY with SIP fragment (100 Trying)
        // 2. Establish new call to transfer target
        // 3. Send NOTIFY with SIP fragment (200 OK or error)
        // 4. Terminate original call after successful transfer

        // For now, just accept the REFER
        Some(SipMessageBuilder::create_response(
            &request,
            202,
            "Accepted",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsip::{Method, Uri};
    use std::collections::HashMap;
    use std::net::IpAddr;

    async fn create_test_handler() -> ReferHandler {
        let registrar = Arc::new(super::super::registrar::Registrar::new());
        let media_bridge_manager = Arc::new(RwLock::new(
            crate::infrastructure::media::bridge::MediaBridgeManager::new(),
        ));
        let local_ip: IpAddr = "127.0.0.1".parse().unwrap();
        let call_router = Arc::new(RwLock::new(CallRouter::new(
            registrar,
            media_bridge_manager,
            local_ip,
        )));

        ReferHandler::new(call_router)
    }

    #[tokio::test]
    async fn test_refer_missing_refer_to() {
        let handler = create_test_handler().await;

        // Create REFER request without Refer-To header
        let request = Request {
            method: Method::Refer,
            uri: Uri {
                scheme: Some(rsip::Scheme::Sip),
                auth: None,
                host_with_port: rsip::HostWithPort {
                    host: rsip::Host::Domain("example.com".into()),
                    port: None,
                },
                params: vec![],
                headers: vec![],
            },
            version: rsip::Version::V2,
            headers: rsip::Headers::default(),
            body: vec![],
        };

        let source: SocketAddr = "127.0.0.1:5060".parse().unwrap();
        let response = handler.handle(request, source).await.unwrap();

        assert_eq!(response.status_code.into_inner(), 400);
    }

    #[tokio::test]
    async fn test_refer_non_existent_call() {
        let handler = create_test_handler().await;

        // Create REFER request with Refer-To but for non-existent call
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "nonexistent@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Refer-To".into(), "sip:bob@example.com".as_bytes().to_vec()).into(),
        );

        let request = Request {
            method: Method::Refer,
            uri: Uri {
                scheme: Some(rsip::Scheme::Sip),
                auth: None,
                host_with_port: rsip::HostWithPort {
                    host: rsip::Host::Domain("example.com".into()),
                    port: None,
                },
                params: vec![],
                headers: vec![],
            },
            version: rsip::Version::V2,
            headers,
            body: vec![],
        };

        let source: SocketAddr = "127.0.0.1:5060".parse().unwrap();
        let response = handler.handle(request, source).await.unwrap();

        assert_eq!(response.status_code.into_inner(), 481);
    }
}

/// NOTIFY handler for event notifications
use async_trait::async_trait;
use rsip::{Request, Response};
use std::net::SocketAddr;
use tracing::{debug, info};

use super::handler::SipHandler;
use super::message::SipMessageBuilder;

/// NOTIFY handler for SIP event notifications
/// Used with SUBSCRIBE/REFER for presence, message-waiting, refer, etc.
pub struct NotifyHandler {
}

impl NotifyHandler {
    /// Create a new NOTIFY handler
    pub fn new() -> Self {
        Self {}
    }

    /// Extract Event header from NOTIFY request
    fn extract_event(request: &Request) -> Option<String> {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "event")
            .and_then(|h| h.value().to_string().ok())
    }

    /// Extract Subscription-State header
    fn extract_subscription_state(request: &Request) -> Option<String> {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "subscription-state")
            .and_then(|h| h.value().to_string().ok())
    }
}

impl Default for NotifyHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SipHandler for NotifyHandler {
    async fn handle(&self, request: Request, source: SocketAddr) -> Option<Response> {
        info!("Handling NOTIFY request from {}", source);

        // Extract Event header
        let event = Self::extract_event(&request);
        if let Some(ref evt) = event {
            debug!("NOTIFY event type: {}", evt);
        }

        // Extract Subscription-State
        let sub_state = Self::extract_subscription_state(&request);
        if let Some(ref state) = sub_state {
            debug!("Subscription state: {}", state);
        }

        // Extract body (event payload)
        let body = String::from_utf8_lossy(&request.body);
        if !body.is_empty() {
            debug!("NOTIFY body: {}", body);
        }

        // TODO: Process NOTIFY based on event type
        // - refer: REFER progress notification (SIP fragment)
        // - message-summary: Voicemail notification
        // - presence: Presence/BLF update
        // - dialog: Call state notification
        // - reg: Registration event

        // For now, just accept all NOTIFY requests
        info!("Accepting NOTIFY request");

        Some(SipMessageBuilder::create_response(
            &request,
            200,
            "OK",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsip::{Method, Uri};

    async fn create_test_handler() -> NotifyHandler {
        NotifyHandler::new()
    }

    #[tokio::test]
    async fn test_notify_refer_event() {
        let handler = create_test_handler().await;

        // Create NOTIFY request for REFER event
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-call@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Event".into(), "refer".as_bytes().to_vec()).into(),
        );
        headers.push(
            rsip::Header::Other("Subscription-State".into(), "active".as_bytes().to_vec()).into(),
        );

        // SIP fragment body (refer progress)
        let body = b"SIP/2.0 100 Trying\r\n".to_vec();

        let request = Request {
            method: Method::Notify,
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
            body,
        };

        let source: SocketAddr = "127.0.0.1:5060".parse().unwrap();
        let response = handler.handle(request, source).await.unwrap();

        assert_eq!(response.status_code.into_inner(), 200);
    }

    #[tokio::test]
    async fn test_notify_message_summary() {
        let handler = create_test_handler().await;

        // Create NOTIFY request for message-summary (voicemail)
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-call@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Event".into(), "message-summary".as_bytes().to_vec()).into(),
        );
        headers.push(
            rsip::Header::Other("Subscription-State".into(), "active".as_bytes().to_vec()).into(),
        );

        // Message summary body
        let body = b"Messages-Waiting: yes\r\nVoice-Message: 2/0 (0/0)\r\n".to_vec();

        let request = Request {
            method: Method::Notify,
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
            body,
        };

        let source: SocketAddr = "127.0.0.1:5060".parse().unwrap();
        let response = handler.handle(request, source).await.unwrap();

        assert_eq!(response.status_code.into_inner(), 200);
    }
}

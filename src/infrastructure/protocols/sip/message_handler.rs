/// MESSAGE handler for instant messaging
use async_trait::async_trait;
use chrono::Utc;
use rsip::{Request, Response};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::handler::SipHandler;
use super::message::SipMessageBuilder;
use super::registrar::Registrar;

/// Message record for history
#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content_type: String,
    pub body: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub delivered: bool,
}

/// Message store for offline messages and history
pub struct MessageStore {
    messages: Arc<RwLock<Vec<MessageRecord>>>,
}

impl MessageStore {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn store(&self, message: MessageRecord) {
        let mut msgs = self.messages.write().await;
        msgs.push(message);
    }

    pub async fn get_undelivered(&self, user: &str) -> Vec<MessageRecord> {
        let msgs = self.messages.read().await;
        msgs.iter()
            .filter(|m| m.to == user && !m.delivered)
            .cloned()
            .collect()
    }

    pub async fn mark_delivered(&self, id: &str) {
        let mut msgs = self.messages.write().await;
        if let Some(msg) = msgs.iter_mut().find(|m| m.id == id) {
            msg.delivered = true;
        }
    }

    pub async fn count(&self) -> usize {
        let msgs = self.messages.read().await;
        msgs.len()
    }
}

impl Default for MessageStore {
    fn default() -> Self {
        Self::new()
    }
}

/// MESSAGE handler for SIP instant messaging
pub struct MessageHandler {
    registrar: Arc<Registrar>,
    message_store: Arc<MessageStore>,
}

impl MessageHandler {
    /// Create a new MESSAGE handler
    pub fn new(registrar: Arc<Registrar>, message_store: Arc<MessageStore>) -> Self {
        Self {
            registrar,
            message_store,
        }
    }

    /// Extract From URI
    fn extract_from(request: &Request) -> Option<String> {
        request
            .from_header()
            .ok()
            .and_then(|h| h.uri.to_string().ok())
    }

    /// Extract To URI
    fn extract_to(request: &Request) -> Option<String> {
        request
            .to_header()
            .ok()
            .and_then(|h| h.uri.to_string().ok())
    }

    /// Extract Content-Type header
    fn extract_content_type(request: &Request) -> String {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "content-type")
            .and_then(|h| h.value().to_string().ok())
            .unwrap_or_else(|| "text/plain".to_string())
    }

    /// Extract username from SIP URI
    fn extract_username(uri: &str) -> String {
        // Simple extraction: sip:user@domain -> user
        uri.split('@')
            .next()
            .unwrap_or(uri)
            .trim_start_matches("sip:")
            .trim_start_matches("sips:")
            .to_string()
    }
}

#[async_trait]
impl SipHandler for MessageHandler {
    async fn handle(&self, request: Request, source: SocketAddr) -> Option<Response> {
        info!("Handling MESSAGE request from {}", source);

        // Extract From and To
        let from = Self::extract_from(&request)?;
        let to = Self::extract_to(&request)?;

        debug!("MESSAGE from {} to {}", from, to);

        // Extract Content-Type
        let content_type = Self::extract_content_type(&request);
        debug!("Content-Type: {}", content_type);

        // Extract message body
        let body = String::from_utf8_lossy(&request.body).to_string();
        if body.is_empty() {
            warn!("MESSAGE with empty body");
            return Some(SipMessageBuilder::create_response(
                &request,
                400,
                "Bad Request - Empty body",
            ));
        }

        debug!("Message content: {}", body);

        // Extract recipient username
        let to_username = Self::extract_username(&to);

        // Check if recipient is registered (online)
        let is_online = self.registrar.get_contact(&to_username).await.is_some();

        // Create message record
        let message_record = MessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.clone(),
            to: to.clone(),
            content_type: content_type.clone(),
            body: body.clone(),
            timestamp: Utc::now(),
            delivered: is_online,
        };

        // Store message
        self.message_store.store(message_record.clone()).await;

        if is_online {
            info!("Recipient {} is online, delivering message", to_username);
            // TODO: Forward MESSAGE to recipient using contact from registrar
            // For now, just mark as delivered
        } else {
            info!("Recipient {} is offline, message stored for later delivery", to_username);
        }

        // Accept message
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

    async fn create_test_handler() -> MessageHandler {
        let registrar = Arc::new(Registrar::new());
        let message_store = Arc::new(MessageStore::new());
        MessageHandler::new(registrar, message_store)
    }

    #[tokio::test]
    async fn test_message_text_plain() {
        let handler = create_test_handler().await;

        // Create MESSAGE request
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::From(rsip::headers::From {
                display_name: Some("Alice".into()),
                uri: rsip::Uri::try_from("sip:alice@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::To(rsip::headers::To {
                display_name: Some("Bob".into()),
                uri: rsip::Uri::try_from("sip:bob@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-msg@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Content-Type".into(), "text/plain".as_bytes().to_vec()).into(),
        );

        let body = b"Hello, Bob!".to_vec();

        let request = Request {
            method: Method::Message,
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

        assert_eq!(response.status_code.into_inner(), 202);

        // Check message was stored
        assert_eq!(handler.message_store.count().await, 1);
    }

    #[tokio::test]
    async fn test_message_empty_body() {
        let handler = create_test_handler().await;

        // Create MESSAGE request with empty body
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::From(rsip::headers::From {
                display_name: Some("Alice".into()),
                uri: rsip::Uri::try_from("sip:alice@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::To(rsip::headers::To {
                display_name: Some("Bob".into()),
                uri: rsip::Uri::try_from("sip:bob@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-msg@example.com".to_string(),
            })
            .into(),
        );

        let request = Request {
            method: Method::Message,
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

        assert_eq!(response.status_code.into_inner(), 400);
    }

    #[tokio::test]
    async fn test_message_store_offline() {
        let handler = create_test_handler().await;

        // Create MESSAGE request to offline user
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::From(rsip::headers::From {
                display_name: Some("Alice".into()),
                uri: rsip::Uri::try_from("sip:alice@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::To(rsip::headers::To {
                display_name: Some("Bob".into()),
                uri: rsip::Uri::try_from("sip:bob@example.com").unwrap(),
                params: vec![],
            })
            .into(),
        );
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-msg@example.com".to_string(),
            })
            .into(),
        );

        let body = b"Hello, Bob!".to_vec();

        let request = Request {
            method: Method::Message,
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

        assert_eq!(response.status_code.into_inner(), 202);

        // Check undelivered messages for bob
        let undelivered = handler.message_store.get_undelivered("sip:bob@example.com").await;
        assert_eq!(undelivered.len(), 1);
    }
}

/// SUBSCRIBE handler for event subscription
use async_trait::async_trait;
use rsip::{Request, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::handler::SipHandler;
use super::message::SipMessageBuilder;

/// Subscription information
#[derive(Debug, Clone)]
pub struct Subscription {
    pub subscriber: String,
    pub event: String,
    pub expires: u32,
    pub dialog_id: String,
}

/// Subscription manager
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_subscription(&self, dialog_id: String, subscription: Subscription) {
        let mut subs = self.subscriptions.write().await;
        subs.insert(dialog_id, subscription);
    }

    pub async fn remove_subscription(&self, dialog_id: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(dialog_id);
    }

    pub async fn get_subscription(&self, dialog_id: &str) -> Option<Subscription> {
        let subs = self.subscriptions.read().await;
        subs.get(dialog_id).cloned()
    }

    pub async fn count(&self) -> usize {
        let subs = self.subscriptions.read().await;
        subs.len()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// SUBSCRIBE handler for SIP event subscriptions
pub struct SubscribeHandler {
    subscription_manager: Arc<SubscriptionManager>,
}

impl SubscribeHandler {
    /// Create a new SUBSCRIBE handler
    pub fn new(subscription_manager: Arc<SubscriptionManager>) -> Self {
        Self {
            subscription_manager,
        }
    }

    /// Extract Event header from SUBSCRIBE request
    fn extract_event(request: &Request) -> Option<String> {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "event")
            .and_then(|h| h.value().to_string().ok())
    }

    /// Extract Expires header (default 3600 seconds)
    fn extract_expires(request: &Request) -> u32 {
        request
            .headers
            .iter()
            .find(|h| h.name().to_string().to_lowercase() == "expires")
            .and_then(|h| h.value().to_string().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3600)
    }

    /// Extract From header
    fn extract_from(request: &Request) -> Option<String> {
        request
            .from_header()
            .ok()
            .and_then(|h| h.value().to_string().ok())
    }

    /// Generate dialog ID from Call-ID and tags
    fn generate_dialog_id(request: &Request) -> Option<String> {
        let call_id = request
            .call_id_header()
            .ok()
            .and_then(|h| h.value().to_string().ok())?;

        let from_tag = request
            .from_header()
            .ok()
            .and_then(|h| {
                h.params
                    .iter()
                    .find(|p| p.name().to_string().to_lowercase() == "tag")
                    .and_then(|p| p.value().to_string().ok())
            });

        if let Some(tag) = from_tag {
            Some(format!("{}:{}", call_id, tag))
        } else {
            Some(call_id)
        }
    }
}

#[async_trait]
impl SipHandler for SubscribeHandler {
    async fn handle(&self, request: Request, source: SocketAddr) -> Option<Response> {
        info!("Handling SUBSCRIBE request from {}", source);

        // Extract Event header (required)
        let event = match Self::extract_event(&request) {
            Some(evt) => evt,
            None => {
                warn!("SUBSCRIBE request missing Event header");
                return Some(SipMessageBuilder::create_response(
                    &request,
                    400,
                    "Bad Request - Missing Event header",
                ));
            }
        };

        debug!("SUBSCRIBE event type: {}", event);

        // Extract Expires
        let expires = Self::extract_expires(&request);
        debug!("Subscription expires: {} seconds", expires);

        // Extract From
        let subscriber = Self::extract_from(&request).unwrap_or_else(|| "unknown".to_string());

        // Generate dialog ID
        let dialog_id = Self::generate_dialog_id(&request).unwrap_or_else(|| format!("sub-{}", uuid::Uuid::new_v4()));

        // Handle unsubscribe (Expires: 0)
        if expires == 0 {
            info!("Unsubscribing dialog: {}", dialog_id);
            self.subscription_manager.remove_subscription(&dialog_id).await;

            return Some(SipMessageBuilder::create_response(
                &request,
                200,
                "OK",
            ));
        }

        // Check supported events
        // Common event packages: presence, dialog, message-summary, reg
        let supported_events = vec!["presence", "dialog", "message-summary", "reg", "refer"];
        if !supported_events.contains(&event.as_str()) {
            warn!("Unsupported event package: {}", event);
            return Some(SipMessageBuilder::create_response(
                &request,
                489,
                "Bad Event",
            ));
        }

        // Create subscription
        let subscription = Subscription {
            subscriber: subscriber.clone(),
            event: event.clone(),
            expires,
            dialog_id: dialog_id.clone(),
        };

        self.subscription_manager
            .add_subscription(dialog_id.clone(), subscription)
            .await;

        info!("Created subscription for {} (dialog: {})", subscriber, dialog_id);

        // TODO: Send initial NOTIFY with current state

        // Accept subscription
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

    async fn create_test_handler() -> SubscribeHandler {
        let manager = Arc::new(SubscriptionManager::new());
        SubscribeHandler::new(manager)
    }

    #[tokio::test]
    async fn test_subscribe_presence() {
        let handler = create_test_handler().await;

        // Create SUBSCRIBE request for presence
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-sub@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Event".into(), "presence".as_bytes().to_vec()).into(),
        );
        headers.push(
            rsip::Header::Other("Expires".into(), "3600".as_bytes().to_vec()).into(),
        );

        let request = Request {
            method: Method::Subscribe,
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

        assert_eq!(response.status_code.into_inner(), 202);
    }

    #[tokio::test]
    async fn test_subscribe_missing_event() {
        let handler = create_test_handler().await;

        // Create SUBSCRIBE request without Event header
        let headers = rsip::Headers::default();

        let request = Request {
            method: Method::Subscribe,
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
    async fn test_subscribe_unsupported_event() {
        let handler = create_test_handler().await;

        // Create SUBSCRIBE request with unsupported event
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-sub@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Event".into(), "unsupported-event".as_bytes().to_vec()).into(),
        );

        let request = Request {
            method: Method::Subscribe,
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

        assert_eq!(response.status_code.into_inner(), 489);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let handler = create_test_handler().await;

        // Create SUBSCRIBE request with Expires: 0 (unsubscribe)
        let mut headers = rsip::Headers::default();
        headers.push(
            rsip::Header::CallId(rsip::headers::CallId {
                value: "test-sub@example.com".to_string(),
            })
            .into(),
        );
        headers.push(
            rsip::Header::Other("Event".into(), "presence".as_bytes().to_vec()).into(),
        );
        headers.push(
            rsip::Header::Other("Expires".into(), "0".as_bytes().to_vec()).into(),
        );

        let request = Request {
            method: Method::Subscribe,
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

        assert_eq!(response.status_code.into_inner(), 200);
    }
}

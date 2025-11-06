//! SIP Registrar - manages endpoint registrations

use super::auth::SipAuthenticator;
use super::builder::{build_register_response, ResponseBuilder};
use super::handler::SipHandler;
use super::message::{SipError, SipMethod, SipRequest, SipResponse};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rsip::Header;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Registration binding
#[derive(Debug, Clone)]
pub struct Binding {
    /// Contact URI
    pub contact: String,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// User Agent
    pub user_agent: Option<String>,
}

impl Binding {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Registration entry for an Address of Record (AoR)
#[derive(Debug, Clone)]
pub struct Registration {
    /// Address of Record (e.g., sip:alice@example.com)
    pub aor: String,
    /// Contact bindings
    pub bindings: Vec<Binding>,
}

/// In-memory registrar
pub struct Registrar {
    /// Map of AoR to Registration
    registrations: Arc<RwLock<HashMap<String, Registration>>>,
    /// Default expiration time (seconds)
    default_expires: u32,
    /// Maximum expiration time (seconds)
    max_expires: u32,
    /// Minimum expiration time (seconds)
    min_expires: u32,
    /// Optional digest authentication
    auth: Option<Arc<dyn SipAuthenticator>>,
}

impl Registrar {
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            default_expires: 3600, // 1 hour
            max_expires: 7200,     // 2 hours
            min_expires: 60,       // 1 minute
            auth: None,
        }
    }

    /// Create registrar with authentication
    pub fn with_auth(auth: Arc<dyn SipAuthenticator>) -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            default_expires: 3600,
            max_expires: 7200,
            min_expires: 60,
            auth: Some(auth),
        }
    }

    /// Set authentication (for existing registrar)
    pub fn set_auth(&mut self, auth: Arc<dyn SipAuthenticator>) {
        self.auth = Some(auth);
    }

    /// Get effective expiration time
    fn get_expires(&self, requested: Option<u32>) -> u32 {
        match requested {
            Some(expires) if expires == 0 => 0, // Unregister
            Some(expires) if expires < self.min_expires => self.min_expires,
            Some(expires) if expires > self.max_expires => self.max_expires,
            Some(expires) => expires,
            None => self.default_expires,
        }
    }

    /// Register a binding (public for testing)
    pub async fn add_binding(
        &self,
        aor: String,
        contact: String,
        expires: u32,
    ) -> Result<(), SipError> {
        self.register_binding(&aor, &contact, expires, None).await
    }

    /// Register a binding
    async fn register_binding(
        &self,
        aor: &str,
        contact: &str,
        expires: u32,
        user_agent: Option<String>,
    ) -> Result<(), SipError> {
        let mut registrations = self.registrations.write().await;

        if expires == 0 {
            // Unregister
            info!("Unregistering: {}", aor);
            registrations.remove(aor);
            return Ok(());
        }

        let expires_at = Utc::now() + Duration::seconds(expires as i64);
        let binding = Binding {
            contact: contact.to_string(),
            expires_at,
            user_agent,
        };

        let registration = registrations
            .entry(aor.to_string())
            .or_insert_with(|| Registration {
                aor: aor.to_string(),
                bindings: Vec::new(),
            });

        // Remove existing binding with same contact
        registration
            .bindings
            .retain(|b| b.contact != contact);

        // Add new binding
        registration.bindings.push(binding);

        info!(
            "Registered: {} -> {} (expires in {}s)",
            aor, contact, expires
        );

        Ok(())
    }

    /// Get bindings for an AoR
    pub async fn get_bindings(&self, aor: &str) -> Option<Vec<Binding>> {
        let mut registrations = self.registrations.write().await;

        if let Some(registration) = registrations.get_mut(aor) {
            // Remove expired bindings
            registration.bindings.retain(|b| !b.is_expired());

            if registration.bindings.is_empty() {
                registrations.remove(aor);
                return None;
            }

            return Some(registration.bindings.clone());
        }

        None
    }

    /// Get all registered users (AoRs)
    pub async fn get_all_registrations(&self) -> Vec<Registration> {
        let mut registrations = self.registrations.write().await;

        // Remove expired bindings and collect valid registrations
        let mut valid_registrations = Vec::new();
        let mut expired_aors = Vec::new();

        for (aor, registration) in registrations.iter_mut() {
            registration.bindings.retain(|b| !b.is_expired());

            if registration.bindings.is_empty() {
                expired_aors.push(aor.clone());
            } else {
                valid_registrations.push(registration.clone());
            }
        }

        // Remove expired registrations
        for aor in expired_aors {
            registrations.remove(&aor);
        }

        valid_registrations
    }

    /// Get registration count
    pub async fn get_registration_count(&self) -> usize {
        let registrations = self.registrations.read().await;
        registrations.len()
    }

    /// Check if a user is registered
    pub async fn is_registered(&self, aor: &str) -> bool {
        self.get_bindings(aor).await.is_some()
    }

    /// Extract AoR from request
    fn extract_aor(request: &SipRequest) -> Result<String, SipError> {
        // Get To header
        let to_header = request
            .headers()
            .iter()
            .find_map(|h| match h {
                Header::To(to) => Some(to),
                _ => None,
            })
            .ok_or_else(|| SipError::InvalidMessage("Missing To header".to_string()))?;

        // Extract URI from To header
        let uri = to_header.uri().ok().map(|u| u.to_string()).unwrap_or_default();
        Ok(uri)
    }

    /// Extract Contact from request
    fn extract_contact(request: &SipRequest) -> Option<String> {
        request.headers().iter().find_map(|h| match h {
            Header::Contact(contact) => contact.uri().ok().map(|u| u.to_string()),
            _ => None,
        })
    }

    /// Extract Expires from request
    fn extract_expires(request: &SipRequest) -> Option<u32> {
        // Try Expires header first
        if let Some(expires) = request.headers().iter().find_map(|h| match h {
            Header::Expires(exp) => exp.to_string().parse().ok(),
            _ => None,
        }) {
            return Some(expires);
        }

        // Simplified - TODO: parse expires parameter from Contact header params
        None
    }

    /// Extract User-Agent from request
    fn extract_user_agent(request: &SipRequest) -> Option<String> {
        request.headers().iter().find_map(|h| match h {
            Header::UserAgent(ua) => Some(ua.to_string()),
            _ => None,
        })
    }
}

impl Default for Registrar {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SipHandler for Registrar {
    async fn handle_request(&self, request: SipRequest) -> Result<SipResponse, SipError> {
        debug!("Handling REGISTER request");

        // Check authentication if enabled
        if let Some(auth) = &self.auth {
            // Check if Authorization header is present
            let has_auth = request.headers().iter().any(|h| {
                matches!(h, Header::Authorization(_) | Header::ProxyAuthorization(_))
            });

            if !has_auth {
                // Send 401 Unauthorized with challenge
                warn!("REGISTER without authentication - sending challenge");
                let challenge = auth.create_challenge().await;

                return ResponseBuilder::new(401)
                    .header(Header::Other(
                        "WWW-Authenticate".to_string(),
                        challenge.to_header_value(),
                    ))
                    .build_for_request(&request);
            }

            // Verify authentication
            match auth.verify_request(&request, "REGISTER").await {
                Ok(username) => {
                    info!("REGISTER authenticated for user: {}", username);
                }
                Err(e) => {
                    warn!("Authentication failed: {:?}", e);
                    // Send 401 with new challenge
                    let challenge = auth.create_challenge().await;

                    return ResponseBuilder::new(401)
                        .header(Header::Other(
                            "WWW-Authenticate".to_string(),
                            challenge.to_header_value(),
                        ))
                        .build_for_request(&request);
                }
            }
        }

        // Extract required information
        let aor = Self::extract_aor(&request)?;
        let contact = Self::extract_contact(&request);
        let requested_expires = Self::extract_expires(&request);
        let user_agent = Self::extract_user_agent(&request);

        // Get effective expiration time
        let expires = self.get_expires(requested_expires);

        // Register the binding if contact is present
        if let Some(contact_uri) = contact.as_ref() {
            self.register_binding(&aor, contact_uri, expires, user_agent)
                .await?;
        }

        // Build response
        let response = build_register_response(&request, 200)?;

        Ok(response)
    }

    fn can_handle(&self, method: SipMethod) -> bool {
        matches!(method, SipMethod::Register)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registrar() {
        let registrar = Registrar::new();

        // Test registration
        registrar
            .register_binding(
                "sip:alice@example.com",
                "sip:alice@192.168.1.100:5060",
                3600,
                Some("YakYak/0.1".to_string()),
            )
            .await
            .unwrap();

        // Get bindings
        let bindings = registrar
            .get_bindings("sip:alice@example.com")
            .await
            .unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].contact, "sip:alice@192.168.1.100:5060");
    }

    #[tokio::test]
    async fn test_unregister() {
        let registrar = Registrar::new();

        // Register
        registrar
            .register_binding(
                "sip:bob@example.com",
                "sip:bob@192.168.1.101:5060",
                3600,
                None,
            )
            .await
            .unwrap();

        // Unregister (expires = 0)
        registrar
            .register_binding("sip:bob@example.com", "sip:bob@192.168.1.101:5060", 0, None)
            .await
            .unwrap();

        // Should have no bindings
        let bindings = registrar.get_bindings("sip:bob@example.com").await;
        assert!(bindings.is_none());
    }
}

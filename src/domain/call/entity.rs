//! Call entities

use crate::domain::shared::value_objects::{EndpointId, SipUri};
use serde::{Deserialize, Serialize};

/// Participant in a call
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participant {
    /// Endpoint identifier
    endpoint_id: EndpointId,
    /// SIP URI
    uri: SipUri,
    /// Display name
    display_name: Option<String>,
}

impl Participant {
    pub fn new(endpoint_id: EndpointId, uri: SipUri, display_name: Option<String>) -> Self {
        Self {
            endpoint_id,
            uri,
            display_name,
        }
    }

    pub fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    pub fn uri(&self) -> &SipUri {
        &self.uri
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn set_display_name(&mut self, name: Option<String>) {
        self.display_name = name;
    }
}

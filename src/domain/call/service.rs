//! Call domain service

use crate::domain::call::aggregate::Call;
use crate::domain::call::entity::Participant;
use crate::domain::call::value_object::CallDirection;
use crate::domain::shared::result::Result;
use crate::domain::shared::value_objects::SipUri;

/// Domain service for call-related operations
///
/// Domain services contain business logic that doesn't naturally
/// fit within a single aggregate.
pub struct CallDomainService;

impl CallDomainService {
    pub fn new() -> Self {
        Self
    }

    /// Determine call direction based on participants
    pub fn determine_direction(
        caller_uri: &SipUri,
        callee_uri: &SipUri,
        internal_domain: &str,
    ) -> CallDirection {
        let caller_is_internal = caller_uri.host() == internal_domain;
        let callee_is_internal = callee_uri.host() == internal_domain;

        match (caller_is_internal, callee_is_internal) {
            (true, true) => CallDirection::Internal,
            (true, false) => CallDirection::Outbound,
            (false, true) => CallDirection::Inbound,
            (false, false) => CallDirection::Outbound, // Unusual case
        }
    }

    /// Check if two calls can be bridged together
    pub fn can_bridge_calls(call1: &Call, call2: &Call) -> bool {
        // Both calls must be active
        call1.is_active() && call2.is_active()
    }

    /// Validate call setup
    pub fn validate_call_setup(
        caller: &Participant,
        callee: &Participant,
    ) -> Result<()> {
        // Basic validation - can be extended
        if caller.uri() == callee.uri() {
            return Err(crate::domain::shared::error::DomainError::ValidationError(
                "Cannot call yourself".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for CallDomainService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::value_objects::EndpointId;

    #[test]
    fn test_determine_direction() {
        let internal_domain = "example.com";

        let internal_to_internal = CallDomainService::determine_direction(
            &SipUri::parse("sip:alice@example.com").unwrap(),
            &SipUri::parse("sip:bob@example.com").unwrap(),
            internal_domain,
        );
        assert_eq!(internal_to_internal, CallDirection::Internal);

        let internal_to_external = CallDomainService::determine_direction(
            &SipUri::parse("sip:alice@example.com").unwrap(),
            &SipUri::parse("sip:charlie@external.com").unwrap(),
            internal_domain,
        );
        assert_eq!(internal_to_external, CallDirection::Outbound);

        let external_to_internal = CallDomainService::determine_direction(
            &SipUri::parse("sip:charlie@external.com").unwrap(),
            &SipUri::parse("sip:alice@example.com").unwrap(),
            internal_domain,
        );
        assert_eq!(external_to_internal, CallDirection::Inbound);
    }

    #[test]
    fn test_validate_call_setup() {
        let uri = SipUri::parse("sip:alice@example.com").unwrap();
        let participant = Participant::new(
            EndpointId::new(),
            uri.clone(),
            Some("Alice".to_string()),
        );

        let result = CallDomainService::validate_call_setup(&participant, &participant);
        assert!(result.is_err());
    }
}

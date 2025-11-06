//! Codec Negotiation
//!
//! Handles SDP codec negotiation between endpoints

use super::g711::G711Type;

/// Codec Information
#[derive(Debug, Clone, PartialEq)]
pub struct CodecInfo {
    pub payload_type: u8,
    pub name: String,
    pub clock_rate: u32,
    pub channels: u8,
}

impl CodecInfo {
    pub fn new(payload_type: u8, name: String, clock_rate: u32) -> Self {
        Self {
            payload_type,
            name,
            clock_rate,
            channels: 1,
        }
    }

    /// Get encoding name for rtpmap
    pub fn encoding(&self) -> String {
        format!("{}/{}", self.name, self.clock_rate)
    }
}

/// Codec Negotiator
pub struct CodecNegotiator {
    supported_codecs: Vec<CodecInfo>,
}

impl CodecNegotiator {
    /// Create negotiator with default supported codecs
    pub fn new() -> Self {
        let supported_codecs = vec![
            CodecInfo::new(0, "PCMU".to_string(), 8000),
            CodecInfo::new(8, "PCMA".to_string(), 8000),
        ];

        Self { supported_codecs }
    }

    /// Negotiate codecs based on offer
    ///
    /// Returns the list of codecs that both sides support, in preference order
    pub fn negotiate(&self, offered_codecs: &[u8]) -> Vec<CodecInfo> {
        offered_codecs
            .iter()
            .filter_map(|pt| self.find_codec(*pt))
            .cloned()
            .collect()
    }

    /// Find codec by payload type
    pub fn find_codec(&self, payload_type: u8) -> Option<&CodecInfo> {
        self.supported_codecs
            .iter()
            .find(|c| c.payload_type == payload_type)
    }

    /// Get preferred codec
    pub fn preferred_codec(&self) -> &CodecInfo {
        &self.supported_codecs[0]
    }

    /// Select best codec from negotiated list
    pub fn select_best<'a>(&self, negotiated: &'a [CodecInfo]) -> Option<&'a CodecInfo> {
        negotiated.first()
    }

    /// Get G.711 type for payload type
    pub fn g711_type(&self, payload_type: u8) -> Option<G711Type> {
        match payload_type {
            0 => Some(G711Type::PCMU),
            8 => Some(G711Type::PCMA),
            _ => None,
        }
    }

    /// Get all supported payload types
    pub fn supported_payload_types(&self) -> Vec<u8> {
        self.supported_codecs
            .iter()
            .map(|c| c.payload_type)
            .collect()
    }
}

impl Default for CodecNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_info() {
        let codec = CodecInfo::new(0, "PCMU".to_string(), 8000);
        assert_eq!(codec.payload_type, 0);
        assert_eq!(codec.name, "PCMU");
        assert_eq!(codec.encoding(), "PCMU/8000");
    }

    #[test]
    fn test_negotiator_creation() {
        let negotiator = CodecNegotiator::new();
        assert_eq!(negotiator.supported_codecs.len(), 2);

        let pts = negotiator.supported_payload_types();
        assert!(pts.contains(&0));
        assert!(pts.contains(&8));
    }

    #[test]
    fn test_codec_negotiation() {
        let negotiator = CodecNegotiator::new();

        // Offer both PCMU and PCMA
        let offered = vec![0, 8];
        let negotiated = negotiator.negotiate(&offered);
        assert_eq!(negotiated.len(), 2);
        assert_eq!(negotiated[0].payload_type, 0);
        assert_eq!(negotiated[1].payload_type, 8);
    }

    #[test]
    fn test_codec_negotiation_partial() {
        let negotiator = CodecNegotiator::new();

        // Offer only PCMA and unsupported codec
        let offered = vec![8, 97]; // 97 is not supported
        let negotiated = negotiator.negotiate(&offered);
        assert_eq!(negotiated.len(), 1);
        assert_eq!(negotiated[0].payload_type, 8);
        assert_eq!(negotiated[0].name, "PCMA");
    }

    #[test]
    fn test_select_best() {
        let negotiator = CodecNegotiator::new();

        let offered = vec![8, 0]; // PCMA first
        let negotiated = negotiator.negotiate(&offered);
        let best = negotiator.select_best(&negotiated).unwrap();
        assert_eq!(best.payload_type, 8); // Should select first offered
    }

    #[test]
    fn test_g711_type_mapping() {
        let negotiator = CodecNegotiator::new();

        assert_eq!(negotiator.g711_type(0), Some(G711Type::PCMU));
        assert_eq!(negotiator.g711_type(8), Some(G711Type::PCMA));
        assert_eq!(negotiator.g711_type(97), None);
    }
}

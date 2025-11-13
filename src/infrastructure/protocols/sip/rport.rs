/// RPORT support for NAT traversal (RFC 3581)
///
/// Allows SIP endpoints to learn the source port/address from which requests were sent,
/// which is critical for NAT traversal.

use std::net::SocketAddr;
use tracing::{debug, info};

/// Add rport parameter to Via header string
///
/// This signals to the server that we want to learn our public address/port
pub fn add_rport_to_via_string(via_header: &str) -> String {
    // Check if rport already present
    if !via_header.contains("rport") {
        // Add rport parameter (without value)
        let result = format!("{};rport", via_header);
        debug!("Added rport to Via header");
        result
    } else {
        via_header.to_string()
    }
}

/// Extract rport value from Via header
///
/// Returns the port number if rport parameter is present with a value
pub fn extract_rport_from_via(via_header: &str) -> Option<u16> {
    // Look for rport=<port> pattern
    for param in via_header.split(';') {
        let param = param.trim();
        if param.starts_with("rport=") {
            if let Some(port_str) = param.strip_prefix("rport=") {
                if let Ok(port) = port_str.parse::<u16>() {
                    debug!("Extracted rport value: {}", port);
                    return Some(port);
                }
            }
        }
    }
    None
}

/// Extract received parameter from Via header
///
/// Returns the IP address if received parameter is present
pub fn extract_received_from_via(via_header: &str) -> Option<String> {
    for param in via_header.split(';') {
        let param = param.trim();
        if param.starts_with("received=") {
            if let Some(ip_str) = param.strip_prefix("received=") {
                debug!("Extracted received value: {}", ip_str);
                return Some(ip_str.to_string());
            }
        }
    }
    None
}

/// Add rport and received parameters to Via header based on actual source address
///
/// This is typically done by the server when processing incoming requests
pub fn add_rport_and_received(via_header: &str, source_addr: SocketAddr) -> String {
    let mut result = via_header.to_string();

    // Add/update rport parameter with actual source port
    if via_header.contains("rport=") {
        // Replace existing rport value
        result = via_header
            .split(';')
            .map(|part| {
                if part.trim().starts_with("rport") {
                    format!("rport={}", source_addr.port())
                } else {
                    part.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(";");
    } else if via_header.contains("rport") {
        // Add rport value where there was just "rport" param
        result = via_header.replace("rport", &format!("rport={}", source_addr.port()));
    } else {
        // Add rport parameter with value
        result = format!("{};rport={}", result, source_addr.port());
    }

    // Add received parameter if not present
    if !result.contains("received=") {
        result = format!("{};received={}", result, source_addr.ip());
        info!(
            "Added rport={} and received={} to Via header",
            source_addr.port(),
            source_addr.ip()
        );
    }

    result
}

/// Get public address from Via header (rport + received)
///
/// Returns (IP, port) if both rport and received parameters are present
pub fn get_public_address_from_via(via_header: &str) -> Option<SocketAddr> {
    let rport = extract_rport_from_via(via_header);
    let received = extract_received_from_via(via_header);

    if let (Some(port), Some(ip_str)) = (rport, received) {
        if let Ok(ip) = ip_str.parse() {
            let addr = SocketAddr::new(ip, port);
            info!("Learned public address from rport: {}", addr);
            return Some(addr);
        }
    }
    None
}

/// Check if Via header has rport parameter
pub fn has_rport_parameter(via_header: &str) -> bool {
    via_header.contains("rport")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rport_to_via_string() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds";
        let result = add_rport_to_via_string(via);
        assert!(result.contains("rport"));
        assert!(result.contains("192.168.1.100:5060"));
    }

    #[test]
    fn test_add_rport_already_present() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport";
        let result = add_rport_to_via_string(via);
        assert_eq!(result, via);
    }

    #[test]
    fn test_extract_rport_from_via() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport=51234";
        let port = extract_rport_from_via(via);
        assert_eq!(port, Some(51234));
    }

    #[test]
    fn test_extract_rport_no_value() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport";
        let port = extract_rport_from_via(via);
        assert_eq!(port, None);
    }

    #[test]
    fn test_extract_received_from_via() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;received=203.0.113.1";
        let received = extract_received_from_via(via);
        assert_eq!(received, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_add_rport_and_received() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport";
        let source: SocketAddr = "203.0.113.1:51234".parse().unwrap();
        let result = add_rport_and_received(via, source);

        assert!(result.contains("rport=51234"));
        assert!(result.contains("received=203.0.113.1"));
    }

    #[test]
    fn test_has_rport_parameter() {
        let via1 = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport";
        assert!(has_rport_parameter(via1));

        let via2 = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds";
        assert!(!has_rport_parameter(via2));
    }

    #[test]
    fn test_get_public_address_from_via() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport=51234;received=203.0.113.1";
        let addr = get_public_address_from_via(via);

        assert!(addr.is_some());
        let addr = addr.unwrap();
        assert_eq!(addr.ip().to_string(), "203.0.113.1");
        assert_eq!(addr.port(), 51234);
    }

    #[test]
    fn test_extract_both_rport_and_received() {
        let via = "SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds;rport=51234;received=203.0.113.1";

        let rport = extract_rport_from_via(via);
        let received = extract_received_from_via(via);

        assert_eq!(rport, Some(51234));
        assert_eq!(received, Some("203.0.113.1".to_string()));
    }
}

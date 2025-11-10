/// ICE candidate types and management
use std::net::SocketAddr;
use uuid::Uuid;

/// ICE candidate types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateType {
    /// Host candidate - local interface address
    Host,
    /// Server reflexive candidate - public address discovered via STUN
    ServerReflexive,
    /// Peer reflexive candidate - discovered during connectivity checks
    PeerReflexive,
    /// Relay candidate - address on TURN relay server
    Relay,
}

impl CandidateType {
    pub fn priority_preference(&self) -> u32 {
        match self {
            CandidateType::Host => 126,
            CandidateType::PeerReflexive => 110,
            CandidateType::ServerReflexive => 100,
            CandidateType::Relay => 0,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            CandidateType::Host => "host",
            CandidateType::ServerReflexive => "srflx",
            CandidateType::PeerReflexive => "prflx",
            CandidateType::Relay => "relay",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "host" => Some(CandidateType::Host),
            "srflx" => Some(CandidateType::ServerReflexive),
            "prflx" => Some(CandidateType::PeerReflexive),
            "relay" => Some(CandidateType::Relay),
            _ => None,
        }
    }
}

/// Transport protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProtocol {
    UDP,
    TCP,
}

impl TransportProtocol {
    pub fn to_string(&self) -> &str {
        match self {
            TransportProtocol::UDP => "udp",
            TransportProtocol::TCP => "tcp",
        }
    }
}

/// ICE candidate
#[derive(Debug, Clone)]
pub struct IceCandidate {
    /// Unique identifier
    pub id: Uuid,
    /// Foundation - unique identifier for candidates from same source
    pub foundation: String,
    /// Component ID (1 = RTP, 2 = RTCP)
    pub component: u16,
    /// Transport protocol
    pub transport: TransportProtocol,
    /// Priority
    pub priority: u32,
    /// Connection address
    pub address: SocketAddr,
    /// Candidate type
    pub candidate_type: CandidateType,
    /// Related address (for reflexive and relay candidates)
    pub related_address: Option<SocketAddr>,
}

impl IceCandidate {
    /// Create a new ICE candidate
    pub fn new(
        candidate_type: CandidateType,
        address: SocketAddr,
        component: u16,
    ) -> Self {
        let foundation = Self::compute_foundation(&candidate_type, &address);
        let priority = Self::compute_priority(candidate_type, component);

        Self {
            id: Uuid::new_v4(),
            foundation,
            component,
            transport: TransportProtocol::UDP,
            priority,
            address,
            candidate_type,
            related_address: None,
        }
    }

    /// Create with related address
    pub fn with_related_address(mut self, related_addr: SocketAddr) -> Self {
        self.related_address = Some(related_addr);
        self
    }

    /// Compute foundation based on candidate type and address
    fn compute_foundation(candidate_type: &CandidateType, address: &SocketAddr) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}{}", candidate_type, address).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Compute priority according to RFC 5245
    /// priority = (2^24)*(type preference) + (2^8)*(local preference) + (256 - component ID)
    pub fn compute_priority(candidate_type: CandidateType, component: u16) -> u32 {
        let type_pref = candidate_type.priority_preference();
        let local_pref = 65535u32; // Maximum local preference
        let component_id = component as u32;

        (1 << 24) * type_pref + (1 << 8) * local_pref + (256 - component_id)
    }

    /// Convert to SDP candidate attribute format
    /// Example: "candidate:1 1 UDP 2130706431 192.168.1.100 5000 typ host"
    pub fn to_sdp(&self) -> String {
        let mut sdp = format!(
            "candidate:{} {} {} {} {} {} typ {}",
            self.foundation,
            self.component,
            self.transport.to_string().to_uppercase(),
            self.priority,
            self.address.ip(),
            self.address.port(),
            self.candidate_type.to_string()
        );

        if let Some(related) = self.related_address {
            sdp.push_str(&format!(
                " raddr {} rport {}",
                related.ip(),
                related.port()
            ));
        }

        sdp
    }

    /// Parse from SDP candidate string
    pub fn from_sdp(sdp: &str) -> Result<Self, String> {
        let parts: Vec<&str> = sdp.split_whitespace().collect();

        if parts.len() < 8 {
            return Err("Invalid SDP candidate format".to_string());
        }

        // Remove "candidate:" prefix if present
        let foundation = parts[0].trim_start_matches("candidate:").to_string();
        let component = parts[1]
            .parse::<u16>()
            .map_err(|_| "Invalid component".to_string())?;
        let transport = match parts[2].to_lowercase().as_str() {
            "udp" => TransportProtocol::UDP,
            "tcp" => TransportProtocol::TCP,
            _ => return Err("Invalid transport protocol".to_string()),
        };
        let priority = parts[3]
            .parse::<u32>()
            .map_err(|_| "Invalid priority".to_string())?;
        let ip = parts[4];
        let port = parts[5]
            .parse::<u16>()
            .map_err(|_| "Invalid port".to_string())?;

        let address: SocketAddr = format!("{}:{}", ip, port)
            .parse()
            .map_err(|_| "Invalid address".to_string())?;

        // Find "typ" keyword
        let typ_idx = parts.iter().position(|&p| p == "typ")
            .ok_or_else(|| "Missing 'typ' keyword".to_string())?;

        if typ_idx + 1 >= parts.len() {
            return Err("Missing candidate type".to_string());
        }

        let candidate_type = CandidateType::from_string(parts[typ_idx + 1])
            .ok_or_else(|| "Invalid candidate type".to_string())?;

        let mut candidate = Self {
            id: Uuid::new_v4(),
            foundation,
            component,
            transport,
            priority,
            address,
            candidate_type,
            related_address: None,
        };

        // Parse related address if present
        if let Some(raddr_idx) = parts.iter().position(|&p| p == "raddr") {
            if raddr_idx + 1 < parts.len() {
                if let Some(rport_idx) = parts.iter().position(|&p| p == "rport") {
                    if rport_idx + 1 < parts.len() {
                        let raddr_ip = parts[raddr_idx + 1];
                        let raddr_port = parts[rport_idx + 1].parse::<u16>().ok();

                        if let Some(port) = raddr_port {
                            if let Ok(addr) = format!("{}:{}", raddr_ip, port).parse() {
                                candidate.related_address = Some(addr);
                            }
                        }
                    }
                }
            }
        }

        Ok(candidate)
    }
}

/// ICE candidate pair
#[derive(Debug, Clone)]
pub struct IceCandidatePair {
    pub local: IceCandidate,
    pub remote: IceCandidate,
    pub state: CandidatePairState,
    pub priority: u64,
}

/// Candidate pair state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidatePairState {
    Frozen,
    Waiting,
    InProgress,
    Succeeded,
    Failed,
}

impl IceCandidatePair {
    pub fn new(local: IceCandidate, remote: IceCandidate) -> Self {
        let priority = Self::compute_pair_priority(&local, &remote, true);

        Self {
            local,
            remote,
            state: CandidatePairState::Frozen,
            priority,
        }
    }

    /// Compute pair priority according to RFC 5245
    pub fn compute_pair_priority(local: &IceCandidate, remote: &IceCandidate, is_controlling: bool) -> u64 {
        let (g, d) = if is_controlling {
            (local.priority as u64, remote.priority as u64)
        } else {
            (remote.priority as u64, local.priority as u64)
        };

        (1u64 << 32) * g.min(d) + 2 * g.max(d) + (if g > d { 1 } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_type_priority() {
        assert_eq!(CandidateType::Host.priority_preference(), 126);
        assert_eq!(CandidateType::ServerReflexive.priority_preference(), 100);
        assert_eq!(CandidateType::Relay.priority_preference(), 0);
    }

    #[test]
    fn test_candidate_creation() {
        let addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let candidate = IceCandidate::new(CandidateType::Host, addr, 1);

        assert_eq!(candidate.address, addr);
        assert_eq!(candidate.component, 1);
        assert_eq!(candidate.candidate_type, CandidateType::Host);
    }

    #[test]
    fn test_candidate_sdp_serialization() {
        let addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let candidate = IceCandidate::new(CandidateType::Host, addr, 1);

        let sdp = candidate.to_sdp();
        assert!(sdp.contains("192.168.1.100"));
        assert!(sdp.contains("5000"));
        assert!(sdp.contains("typ host"));
    }

    #[test]
    fn test_candidate_sdp_parsing() {
        let sdp = "candidate:1 1 UDP 2130706431 192.168.1.100 5000 typ host";
        let candidate = IceCandidate::from_sdp(sdp).unwrap();

        assert_eq!(candidate.component, 1);
        assert_eq!(candidate.address.port(), 5000);
        assert_eq!(candidate.candidate_type, CandidateType::Host);
    }

    #[test]
    fn test_candidate_with_related_address() {
        let sdp = "candidate:2 1 UDP 1694498815 203.0.113.1 45000 typ srflx raddr 192.168.1.100 rport 5000";
        let candidate = IceCandidate::from_sdp(sdp).unwrap();

        assert_eq!(candidate.candidate_type, CandidateType::ServerReflexive);
        assert!(candidate.related_address.is_some());

        let related = candidate.related_address.unwrap();
        assert_eq!(related.port(), 5000);
    }

    #[test]
    fn test_candidate_pair_creation() {
        let local_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let remote_addr: SocketAddr = "203.0.113.1:6000".parse().unwrap();

        let local = IceCandidate::new(CandidateType::Host, local_addr, 1);
        let remote = IceCandidate::new(CandidateType::Host, remote_addr, 1);

        let pair = IceCandidatePair::new(local, remote);
        assert_eq!(pair.state, CandidatePairState::Frozen);
        assert!(pair.priority > 0);
    }

    #[test]
    fn test_priority_computation() {
        let priority = IceCandidate::compute_priority(CandidateType::Host, 1);
        assert!(priority > 0);

        let host_priority = IceCandidate::compute_priority(CandidateType::Host, 1);
        let relay_priority = IceCandidate::compute_priority(CandidateType::Relay, 1);
        assert!(host_priority > relay_priority);
    }
}

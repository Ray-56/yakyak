/// ICE agent for candidate gathering and connectivity establishment
use super::candidate::{CandidateType, IceCandidate, IceCandidatePair, CandidatePairState};
use crate::infrastructure::protocols::stun::client::StunClient;
use crate::infrastructure::protocols::turn::client::TurnClient;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// ICE connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceConnectionState {
    New,
    Checking,
    Connected,
    Completed,
    Failed,
    Disconnected,
    Closed,
}

/// ICE gathering state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceGatheringState {
    New,
    Gathering,
    Complete,
}

/// ICE agent configuration
#[derive(Debug, Clone)]
pub struct IceConfig {
    pub stun_servers: Vec<SocketAddr>,
    pub turn_servers: Vec<SocketAddr>,
    pub local_candidates: Vec<SocketAddr>,
    pub gather_host_candidates: bool,
    pub gather_srflx_candidates: bool,
    pub gather_relay_candidates: bool,
}

impl Default for IceConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".parse().unwrap(),
                "stun1.l.google.com:19302".parse().unwrap(),
            ],
            turn_servers: Vec::new(),
            local_candidates: Vec::new(),
            gather_host_candidates: true,
            gather_srflx_candidates: true,
            gather_relay_candidates: false,
        }
    }
}

/// ICE agent
pub struct IceAgent {
    config: IceConfig,
    connection_state: Arc<RwLock<IceConnectionState>>,
    gathering_state: Arc<RwLock<IceGatheringState>>,
    local_candidates: Arc<RwLock<Vec<IceCandidate>>>,
    remote_candidates: Arc<RwLock<Vec<IceCandidate>>>,
    candidate_pairs: Arc<RwLock<Vec<IceCandidatePair>>>,
    selected_pair: Arc<RwLock<Option<IceCandidatePair>>>,
}

impl IceAgent {
    /// Create a new ICE agent
    pub fn new(config: IceConfig) -> Self {
        Self {
            config,
            connection_state: Arc::new(RwLock::new(IceConnectionState::New)),
            gathering_state: Arc::new(RwLock::new(IceGatheringState::New)),
            local_candidates: Arc::new(RwLock::new(Vec::new())),
            remote_candidates: Arc::new(RwLock::new(Vec::new())),
            candidate_pairs: Arc::new(RwLock::new(Vec::new())),
            selected_pair: Arc::new(RwLock::new(None)),
        }
    }

    /// Gather local candidates
    pub async fn gather_candidates(&self) -> Result<(), String> {
        info!("Starting ICE candidate gathering");
        *self.gathering_state.write().await = IceGatheringState::Gathering;

        let mut candidates = Vec::new();

        // Gather host candidates
        if self.config.gather_host_candidates {
            let host_candidates = self.gather_host_candidates().await?;
            candidates.extend(host_candidates);
        }

        // Gather server reflexive candidates (via STUN)
        if self.config.gather_srflx_candidates {
            let srflx_candidates = self.gather_srflx_candidates().await?;
            candidates.extend(srflx_candidates);
        }

        // Gather relay candidates (via TURN)
        if self.config.gather_relay_candidates {
            let relay_candidates = self.gather_relay_candidates().await?;
            candidates.extend(relay_candidates);
        }

        // Store candidates
        *self.local_candidates.write().await = candidates.clone();

        *self.gathering_state.write().await = IceGatheringState::Complete;
        info!("Gathered {} local candidates", candidates.len());

        Ok(())
    }

    /// Gather host candidates from local interfaces
    async fn gather_host_candidates(&self) -> Result<Vec<IceCandidate>, String> {
        debug!("Gathering host candidates");
        let mut candidates = Vec::new();

        // Use configured local addresses or discover from interfaces
        if !self.config.local_candidates.is_empty() {
            for addr in &self.config.local_candidates {
                let candidate = IceCandidate::new(CandidateType::Host, *addr, 1);
                candidates.push(candidate);
            }
        } else {
            // Default local address (would normally enumerate network interfaces)
            let default_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
            let candidate = IceCandidate::new(CandidateType::Host, default_addr, 1);
            candidates.push(candidate);
        }

        debug!("Gathered {} host candidates", candidates.len());
        Ok(candidates)
    }

    /// Gather server reflexive candidates using STUN
    async fn gather_srflx_candidates(&self) -> Result<Vec<IceCandidate>, String> {
        debug!("Gathering server reflexive candidates");
        let mut candidates = Vec::new();

        for stun_server in &self.config.stun_servers {
            let client = StunClient::new(*stun_server);

            match client.get_public_address().await {
                Ok((public_ip, public_port)) => {
                    let public_addr: SocketAddr = format!("{}:{}", public_ip, public_port)
                        .parse()
                        .map_err(|e| format!("Invalid address: {}", e))?;

                    // Create srflx candidate
                    let candidate = IceCandidate::new(CandidateType::ServerReflexive, public_addr, 1);
                    candidates.push(candidate);

                    debug!("Discovered server reflexive candidate: {}", public_addr);
                }
                Err(e) => {
                    warn!("Failed to get public address from {}: {}", stun_server, e);
                }
            }
        }

        debug!("Gathered {} server reflexive candidates", candidates.len());
        Ok(candidates)
    }

    /// Gather relay candidates using TURN
    async fn gather_relay_candidates(&self) -> Result<Vec<IceCandidate>, String> {
        debug!("Gathering relay candidates");
        let mut candidates = Vec::new();

        for turn_server in &self.config.turn_servers {
            let client = TurnClient::new(*turn_server);

            match client.allocate().await {
                Ok(allocation) => {
                    let candidate = IceCandidate::new(
                        CandidateType::Relay,
                        allocation.relayed_address,
                        1,
                    );
                    candidates.push(candidate);

                    debug!("Allocated relay candidate: {}", allocation.relayed_address);
                }
                Err(e) => {
                    warn!("Failed to allocate relay from {}: {}", turn_server, e);
                }
            }
        }

        debug!("Gathered {} relay candidates", candidates.len());
        Ok(candidates)
    }

    /// Add remote candidates
    pub async fn add_remote_candidates(&self, candidates: Vec<IceCandidate>) {
        debug!("Adding {} remote candidates", candidates.len());
        let mut remote = self.remote_candidates.write().await;
        remote.extend(candidates);

        // Form candidate pairs
        self.form_candidate_pairs().await;
    }

    /// Form candidate pairs from local and remote candidates
    async fn form_candidate_pairs(&self) {
        let local_candidates = self.local_candidates.read().await;
        let remote_candidates = self.remote_candidates.read().await;

        let mut pairs = Vec::new();

        for local in local_candidates.iter() {
            for remote in remote_candidates.iter() {
                // Only pair candidates from same component
                if local.component == remote.component {
                    let pair = IceCandidatePair::new(local.clone(), remote.clone());
                    pairs.push(pair);
                }
            }
        }

        // Sort by priority (descending)
        pairs.sort_by(|a, b| b.priority.cmp(&a.priority));

        debug!("Formed {} candidate pairs", pairs.len());
        *self.candidate_pairs.write().await = pairs;
    }

    /// Start connectivity checks
    pub async fn start_checks(&self) -> Result<(), String> {
        info!("Starting ICE connectivity checks");
        *self.connection_state.write().await = IceConnectionState::Checking;

        let pairs = self.candidate_pairs.read().await;

        // In a real implementation, would perform STUN binding requests
        // For now, simulate selecting the highest priority pair
        if let Some(pair) = pairs.first() {
            let mut selected = pair.clone();
            selected.state = CandidatePairState::Succeeded;

            *self.selected_pair.write().await = Some(selected.clone());
            *self.connection_state.write().await = IceConnectionState::Connected;

            info!(
                "Selected candidate pair: {} <-> {}",
                selected.local.address, selected.remote.address
            );

            Ok(())
        } else {
            *self.connection_state.write().await = IceConnectionState::Failed;
            Err("No candidate pairs available".to_string())
        }
    }

    /// Get local candidates
    pub async fn get_local_candidates(&self) -> Vec<IceCandidate> {
        self.local_candidates.read().await.clone()
    }

    /// Get connection state
    pub async fn get_connection_state(&self) -> IceConnectionState {
        *self.connection_state.read().await
    }

    /// Get gathering state
    pub async fn get_gathering_state(&self) -> IceGatheringState {
        *self.gathering_state.read().await
    }

    /// Get selected candidate pair
    pub async fn get_selected_pair(&self) -> Option<IceCandidatePair> {
        self.selected_pair.read().await.clone()
    }

    /// Close the ICE agent
    pub async fn close(&self) {
        info!("Closing ICE agent");
        *self.connection_state.write().await = IceConnectionState::Closed;
        *self.local_candidates.write().await = Vec::new();
        *self.remote_candidates.write().await = Vec::new();
        *self.candidate_pairs.write().await = Vec::new();
        *self.selected_pair.write().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ice_agent_creation() {
        let config = IceConfig::default();
        let agent = IceAgent::new(config);

        let state = agent.get_connection_state().await;
        assert_eq!(state, IceConnectionState::New);

        let gathering = agent.get_gathering_state().await;
        assert_eq!(gathering, IceGatheringState::New);
    }

    #[tokio::test]
    async fn test_host_candidate_gathering() {
        let mut config = IceConfig::default();
        config.gather_srflx_candidates = false;
        config.gather_relay_candidates = false;

        let local_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        config.local_candidates = vec![local_addr];

        let agent = IceAgent::new(config);
        agent.gather_candidates().await.unwrap();

        let candidates = agent.get_local_candidates().await;
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].candidate_type, CandidateType::Host);
    }

    #[tokio::test]
    async fn test_add_remote_candidates() {
        let config = IceConfig::default();
        let agent = IceAgent::new(config);

        let remote_addr: SocketAddr = "203.0.113.1:6000".parse().unwrap();
        let remote_candidate = IceCandidate::new(CandidateType::Host, remote_addr, 1);

        agent.add_remote_candidates(vec![remote_candidate]).await;

        // Should form pairs after adding remote candidates
        let pairs = agent.candidate_pairs.read().await;
        // Pairs will be empty if no local candidates yet
    }

    #[tokio::test]
    async fn test_candidate_pair_formation() {
        let mut config = IceConfig::default();
        config.gather_srflx_candidates = false;
        config.gather_relay_candidates = false;

        let local_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        config.local_candidates = vec![local_addr];

        let agent = IceAgent::new(config);
        agent.gather_candidates().await.unwrap();

        let remote_addr: SocketAddr = "203.0.113.1:6000".parse().unwrap();
        let remote_candidate = IceCandidate::new(CandidateType::Host, remote_addr, 1);

        agent.add_remote_candidates(vec![remote_candidate]).await;

        let pairs = agent.candidate_pairs.read().await;
        assert!(!pairs.is_empty());
    }
}

/// NAT Traversal Manager
///
/// Coordinates STUN, TURN, and ICE for comprehensive NAT traversal

use crate::infrastructure::protocols::ice::{IceAgent, IceConfig};
use crate::infrastructure::protocols::stun::client::{NatType, StunClient, StunResult};
use crate::infrastructure::protocols::turn::client::{TurnAllocation, TurnClient};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// NAT configuration
#[derive(Debug, Clone)]
pub struct NatConfig {
    /// STUN server addresses
    pub stun_servers: Vec<SocketAddr>,
    /// TURN server addresses
    pub turn_servers: Vec<SocketAddr>,
    /// TURN credentials (username, password)
    pub turn_credentials: Option<(String, String)>,
    /// Enable automatic NAT keepalive
    pub enable_keepalive: bool,
    /// Keepalive interval
    pub keepalive_interval: Duration,
    /// Enable ICE
    pub enable_ice: bool,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".parse().unwrap(),
                "stun1.l.google.com:19302".parse().unwrap(),
            ],
            turn_servers: Vec::new(),
            turn_credentials: None,
            enable_keepalive: true,
            keepalive_interval: Duration::from_secs(25),
            enable_ice: true,
        }
    }
}

/// NAT traversal state
#[derive(Debug, Clone)]
pub struct NatState {
    pub nat_type: NatType,
    pub public_ip: Option<IpAddr>,
    pub public_port: Option<u16>,
    pub local_addr: SocketAddr,
    pub relay_address: Option<SocketAddr>,
}

/// NAT Manager - coordinates all NAT traversal mechanisms
pub struct NatManager {
    config: NatConfig,
    state: Arc<RwLock<Option<NatState>>>,
    stun_clients: Vec<StunClient>,
    turn_clients: Vec<TurnClient>,
    ice_agent: Arc<RwLock<Option<IceAgent>>>,
}

impl NatManager {
    /// Create new NAT manager
    pub fn new(config: NatConfig) -> Self {
        // Create STUN clients for each server
        let stun_clients: Vec<StunClient> = config
            .stun_servers
            .iter()
            .map(|&addr| StunClient::new(addr))
            .collect();

        // Create TURN clients for each server
        let mut turn_clients: Vec<TurnClient> = config
            .turn_servers
            .iter()
            .map(|&addr| {
                let mut client = TurnClient::new(addr);
                if let Some((username, password)) = &config.turn_credentials {
                    client = client.with_credentials(username.clone(), password.clone());
                }
                client
            })
            .collect();

        Self {
            config,
            state: Arc::new(RwLock::new(None)),
            stun_clients,
            turn_clients,
            ice_agent: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize NAT traversal (discover public address and NAT type)
    pub async fn initialize(&self, local_addr: SocketAddr) -> Result<NatState, String> {
        info!("Initializing NAT traversal for {}", local_addr);

        // Try STUN servers in order
        let mut last_error = String::from("No STUN servers configured");

        for client in &self.stun_clients {
            match client.binding_request(local_addr) {
                Ok(result) => {
                    info!("STUN discovery successful via {:?}", client);
                    let public_ip = result.public_addr.ip();
                    let public_port = result.public_addr.port();

                    // Detect NAT type
                    let nat_type = client
                        .detect_nat_type_enhanced(local_addr)
                        .await
                        .unwrap_or(NatType::Unknown);

                    let nat_state = NatState {
                        nat_type,
                        public_ip: Some(public_ip),
                        public_port: Some(public_port),
                        local_addr,
                        relay_address: None,
                    };

                    *self.state.write().await = Some(nat_state.clone());
                    info!("NAT initialized: type={:?}, public={}:{}",
                          nat_type, public_ip, public_port);

                    return Ok(nat_state);
                }
                Err(e) => {
                    warn!("STUN request failed: {}", e);
                    last_error = e;
                }
            }
        }

        Err(format!("All STUN servers failed. Last error: {}", last_error))
    }

    /// Allocate TURN relay (for symmetric NAT or when STUN fails)
    pub async fn allocate_relay(&self) -> Result<SocketAddr, String> {
        info!("Allocating TURN relay");

        if self.turn_clients.is_empty() {
            return Err("No TURN servers configured".to_string());
        }

        // Try TURN servers in order
        for client in &self.turn_clients {
            match client.allocate().await {
                Ok(allocation) => {
                    info!("TURN relay allocated: {}", allocation.relayed_address);

                    // Update state
                    if let Some(state) = self.state.write().await.as_mut() {
                        state.relay_address = Some(allocation.relayed_address);
                    }

                    return Ok(allocation.relayed_address);
                }
                Err(e) => {
                    warn!("TURN allocation failed: {}", e);
                }
            }
        }

        Err("All TURN servers failed".to_string())
    }

    /// Initialize ICE agent
    pub async fn initialize_ice(&self) -> Result<(), String> {
        if !self.config.enable_ice {
            return Err("ICE is disabled in configuration".to_string());
        }

        info!("Initializing ICE agent");

        let ice_config = IceConfig {
            stun_servers: self.config.stun_servers.clone(),
            turn_servers: self.config.turn_servers.clone(),
            local_candidates: Vec::new(),
            gather_host_candidates: true,
            gather_srflx_candidates: true,
            gather_relay_candidates: !self.turn_clients.is_empty(),
        };

        let agent = IceAgent::new(ice_config);
        agent.gather_candidates().await?;

        *self.ice_agent.write().await = Some(agent);
        info!("ICE agent initialized");

        Ok(())
    }

    /// Get current NAT state
    pub async fn get_state(&self) -> Option<NatState> {
        self.state.read().await.clone()
    }

    /// Get public address
    pub async fn get_public_address(&self) -> Option<(IpAddr, u16)> {
        let state = self.state.read().await;
        state.as_ref().and_then(|s| {
            s.public_ip.zip(s.public_port)
        })
    }

    /// Check if behind NAT
    pub async fn is_behind_nat(&self) -> bool {
        let state = self.state.read().await;
        match state.as_ref() {
            Some(s) => s.nat_type != NatType::OpenInternet,
            None => false,
        }
    }

    /// Get NAT type
    pub async fn get_nat_type(&self) -> Option<NatType> {
        let state = self.state.read().await;
        state.as_ref().map(|s| s.nat_type)
    }

    /// Refresh NAT binding (keepalive)
    pub async fn refresh_binding(&self) -> Result<(), String> {
        debug!("Refreshing NAT binding");

        let state = self.state.read().await;
        let local_addr = state
            .as_ref()
            .map(|s| s.local_addr)
            .ok_or_else(|| "NAT not initialized".to_string())?;

        // Use first available STUN client
        if let Some(client) = self.stun_clients.first() {
            client.refresh_binding(local_addr)?;
            debug!("NAT binding refreshed");
            Ok(())
        } else {
            Err("No STUN clients available".to_string())
        }
    }

    /// Start automatic keepalive (runs in background)
    pub async fn start_keepalive(&self) -> Result<(), String> {
        if !self.config.enable_keepalive {
            return Ok(());
        }

        info!("Starting NAT keepalive (interval: {:?})", self.config.keepalive_interval);

        let state = self.state.read().await;
        let local_addr = state
            .as_ref()
            .map(|s| s.local_addr)
            .ok_or_else(|| "NAT not initialized".to_string())?;

        let interval = self.config.keepalive_interval;
        let client = self.stun_clients.first()
            .ok_or_else(|| "No STUN clients available".to_string())?
            .clone();

        // Spawn background task
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                match client.refresh_binding(local_addr) {
                    Ok(_) => debug!("Keepalive successful"),
                    Err(e) => warn!("Keepalive failed: {}", e),
                }
            }
        });

        info!("NAT keepalive started");
        Ok(())
    }

    /// Get recommended transport address (considers NAT type)
    pub async fn get_recommended_address(&self) -> Option<SocketAddr> {
        let state = self.state.read().await;
        state.as_ref().map(|s| {
            // If we have a relay, use it for symmetric NAT
            if s.nat_type == NatType::Symmetric {
                s.relay_address.unwrap_or_else(|| {
                    // Fall back to public address
                    SocketAddr::new(s.public_ip.unwrap(), s.public_port.unwrap())
                })
            } else {
                // For other NAT types, use public address
                SocketAddr::new(s.public_ip.unwrap(), s.public_port.unwrap())
            }
        })
    }

    /// Check if ICE is initialized
    pub async fn has_ice_agent(&self) -> bool {
        self.ice_agent.read().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_config_default() {
        let config = NatConfig::default();
        assert!(!config.stun_servers.is_empty());
        assert!(config.enable_keepalive);
        assert_eq!(config.keepalive_interval, Duration::from_secs(25));
    }

    #[test]
    fn test_nat_manager_creation() {
        let config = NatConfig::default();
        let manager = NatManager::new(config);
        assert!(!manager.stun_clients.is_empty());
    }

    #[tokio::test]
    async fn test_nat_manager_state() {
        let config = NatConfig::default();
        let manager = NatManager::new(config);

        let state = manager.get_state().await;
        assert!(state.is_none());

        assert!(!manager.is_behind_nat().await);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_nat_initialization() {
        let config = NatConfig::default();
        let manager = NatManager::new(config);

        let local_addr: SocketAddr = "0.0.0.0:5060".parse().unwrap();

        match manager.initialize(local_addr).await {
            Ok(state) => {
                println!("NAT Type: {:?}", state.nat_type);
                println!("Public IP: {:?}", state.public_ip);
                assert!(state.public_ip.is_some());
            }
            Err(e) => {
                println!("NAT initialization failed (expected in some environments): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_recommended_address() {
        let config = NatConfig::default();
        let manager = NatManager::new(config);

        // No state yet
        assert!(manager.get_recommended_address().await.is_none());

        // Simulate state
        let state = NatState {
            nat_type: NatType::FullCone,
            public_ip: Some("203.0.113.1".parse().unwrap()),
            public_port: Some(5060),
            local_addr: "192.168.1.100:5060".parse().unwrap(),
            relay_address: None,
        };

        *manager.state.write().await = Some(state);

        let addr = manager.get_recommended_address().await;
        assert!(addr.is_some());
        assert_eq!(addr.unwrap().ip().to_string(), "203.0.113.1");
    }
}

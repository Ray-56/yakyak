/// TURN relay server implementation
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// TURN relay allocation
#[derive(Debug, Clone)]
pub struct RelayAllocation {
    pub id: Uuid,
    pub client_addr: SocketAddr,
    pub relay_addr: SocketAddr,
    pub lifetime: u32,
    pub created_at: Instant,
    pub last_refresh: Instant,
    pub permissions: Vec<SocketAddr>,
    pub transaction_id: [u8; 12],
}

impl RelayAllocation {
    pub fn new(
        client_addr: SocketAddr,
        relay_addr: SocketAddr,
        lifetime: u32,
        transaction_id: [u8; 12],
    ) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            client_addr,
            relay_addr,
            lifetime,
            created_at: now,
            last_refresh: now,
            permissions: Vec::new(),
            transaction_id,
        }
    }

    /// Check if allocation is expired
    pub fn is_expired(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(self.lifetime as u64)
    }

    /// Refresh the allocation
    pub fn refresh(&mut self, lifetime: u32) {
        self.lifetime = lifetime;
        self.last_refresh = Instant::now();
    }

    /// Add permission for a peer address
    pub fn add_permission(&mut self, peer_addr: SocketAddr) {
        if !self.permissions.contains(&peer_addr) {
            self.permissions.push(peer_addr);
        }
    }

    /// Check if peer has permission
    pub fn has_permission(&self, peer_addr: &SocketAddr) -> bool {
        self.permissions.contains(peer_addr)
    }

    /// Remove permission
    pub fn remove_permission(&mut self, peer_addr: &SocketAddr) {
        self.permissions.retain(|addr| addr != peer_addr);
    }
}

/// TURN relay server
pub struct TurnRelay {
    allocations: Arc<RwLock<HashMap<Uuid, RelayAllocation>>>,
    base_port: u16,
    max_port: u16,
    cleanup_interval: Duration,
}

impl TurnRelay {
    /// Create a new TURN relay server
    pub fn new(base_port: u16, max_port: u16) -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            base_port,
            max_port,
            cleanup_interval: Duration::from_secs(60),
        }
    }

    /// Start the relay server
    pub async fn start(self: Arc<Self>) {
        info!(
            "Starting TURN relay server (ports {}-{})",
            self.base_port, self.max_port
        );

        // Start cleanup task
        let cleanup_relay = Arc::clone(&self);
        tokio::spawn(async move {
            cleanup_relay.cleanup_loop().await;
        });

        info!("TURN relay server started");
    }

    /// Allocate a relay address for a client
    pub async fn allocate(
        &self,
        client_addr: SocketAddr,
        lifetime: u32,
        transaction_id: [u8; 12],
    ) -> Result<RelayAllocation, String> {
        debug!("Allocating relay for client: {}", client_addr);

        // Find available port
        let relay_port = self.find_available_port().await?;
        let relay_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), relay_port);

        // Create allocation
        let allocation = RelayAllocation::new(client_addr, relay_addr, lifetime, transaction_id);
        let id = allocation.id;

        // Store allocation
        let mut allocations = self.allocations.write().await;
        allocations.insert(id, allocation.clone());

        info!(
            "Allocated relay {} for client {} (lifetime: {}s)",
            relay_addr, client_addr, lifetime
        );

        Ok(allocation)
    }

    /// Refresh an existing allocation
    pub async fn refresh(&self, allocation_id: Uuid, lifetime: u32) -> Result<(), String> {
        let mut allocations = self.allocations.write().await;

        if let Some(allocation) = allocations.get_mut(&allocation_id) {
            allocation.refresh(lifetime);
            debug!(
                "Refreshed allocation {} (new lifetime: {}s)",
                allocation_id, lifetime
            );
            Ok(())
        } else {
            Err("Allocation not found".to_string())
        }
    }

    /// Create permission for a peer
    pub async fn create_permission(
        &self,
        allocation_id: Uuid,
        peer_addr: SocketAddr,
    ) -> Result<(), String> {
        let mut allocations = self.allocations.write().await;

        if let Some(allocation) = allocations.get_mut(&allocation_id) {
            allocation.add_permission(peer_addr);
            debug!(
                "Created permission for {} on allocation {}",
                peer_addr, allocation_id
            );
            Ok(())
        } else {
            Err("Allocation not found".to_string())
        }
    }

    /// Relay data from client to peer
    pub async fn relay_to_peer(
        &self,
        allocation_id: Uuid,
        peer_addr: SocketAddr,
        data: &[u8],
    ) -> Result<(), String> {
        let allocations = self.allocations.read().await;

        if let Some(allocation) = allocations.get(&allocation_id) {
            if !allocation.has_permission(&peer_addr) {
                return Err("No permission for peer".to_string());
            }

            // Send data to peer via relay socket
            // TODO: Implement actual socket sending
            debug!(
                "Relaying {} bytes from {} to {} via {}",
                data.len(),
                allocation.client_addr,
                peer_addr,
                allocation.relay_addr
            );

            Ok(())
        } else {
            Err("Allocation not found".to_string())
        }
    }

    /// Relay data from peer to client
    pub async fn relay_to_client(
        &self,
        relay_addr: SocketAddr,
        peer_addr: SocketAddr,
        data: &[u8],
    ) -> Result<(), String> {
        let allocations = self.allocations.read().await;

        // Find allocation by relay address
        for allocation in allocations.values() {
            if allocation.relay_addr == relay_addr {
                if !allocation.has_permission(&peer_addr) {
                    return Err("No permission for peer".to_string());
                }

                // Send data to client
                debug!(
                    "Relaying {} bytes from {} to {} via {}",
                    data.len(),
                    peer_addr,
                    allocation.client_addr,
                    relay_addr
                );

                return Ok(());
            }
        }

        Err("No allocation found for relay address".to_string())
    }

    /// Remove an allocation
    pub async fn remove_allocation(&self, allocation_id: Uuid) -> Result<(), String> {
        let mut allocations = self.allocations.write().await;

        if allocations.remove(&allocation_id).is_some() {
            info!("Removed allocation {}", allocation_id);
            Ok(())
        } else {
            Err("Allocation not found".to_string())
        }
    }

    /// Get allocation by ID
    pub async fn get_allocation(&self, allocation_id: Uuid) -> Option<RelayAllocation> {
        let allocations = self.allocations.read().await;
        allocations.get(&allocation_id).cloned()
    }

    /// List all allocations
    pub async fn list_allocations(&self) -> Vec<RelayAllocation> {
        let allocations = self.allocations.read().await;
        allocations.values().cloned().collect()
    }

    /// Find an available port for relay
    async fn find_available_port(&self) -> Result<u16, String> {
        let allocations = self.allocations.read().await;
        let used_ports: Vec<u16> = allocations
            .values()
            .map(|a| a.relay_addr.port())
            .collect();

        for port in self.base_port..=self.max_port {
            if !used_ports.contains(&port) {
                return Ok(port);
            }
        }

        Err("No available ports".to_string())
    }

    /// Cleanup expired allocations
    async fn cleanup_loop(&self) {
        let mut interval = tokio::time::interval(self.cleanup_interval);

        loop {
            interval.tick().await;
            self.cleanup_expired_allocations().await;
        }
    }

    /// Remove expired allocations
    async fn cleanup_expired_allocations(&self) {
        let mut allocations = self.allocations.write().await;
        let expired: Vec<Uuid> = allocations
            .iter()
            .filter(|(_, alloc)| alloc.is_expired())
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            allocations.remove(&id);
            info!("Removed expired allocation {}", id);
        }
    }

    /// Get statistics
    pub async fn get_stats(&self) -> TurnRelayStats {
        let allocations = self.allocations.read().await;
        TurnRelayStats {
            total_allocations: allocations.len(),
            available_ports: (self.max_port - self.base_port + 1) as usize - allocations.len(),
        }
    }
}

/// TURN relay statistics
#[derive(Debug, Clone)]
pub struct TurnRelayStats {
    pub total_allocations: usize,
    pub available_ports: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_allocation_creation() {
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let relay_addr: SocketAddr = "10.0.0.1:50000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let allocation = RelayAllocation::new(client_addr, relay_addr, 600, transaction_id);

        assert_eq!(allocation.client_addr, client_addr);
        assert_eq!(allocation.relay_addr, relay_addr);
        assert_eq!(allocation.lifetime, 600);
        assert!(!allocation.is_expired());
    }

    #[test]
    fn test_relay_allocation_permissions() {
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let relay_addr: SocketAddr = "10.0.0.1:50000".parse().unwrap();
        let peer_addr: SocketAddr = "192.168.1.200:6000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let mut allocation = RelayAllocation::new(client_addr, relay_addr, 600, transaction_id);

        assert!(!allocation.has_permission(&peer_addr));

        allocation.add_permission(peer_addr);
        assert!(allocation.has_permission(&peer_addr));

        allocation.remove_permission(&peer_addr);
        assert!(!allocation.has_permission(&peer_addr));
    }

    #[tokio::test]
    async fn test_turn_relay_creation() {
        let relay = TurnRelay::new(50000, 50100);
        assert_eq!(relay.base_port, 50000);
        assert_eq!(relay.max_port, 50100);
    }

    #[tokio::test]
    async fn test_turn_relay_allocation() {
        let relay = TurnRelay::new(50000, 50100);
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let allocation = relay.allocate(client_addr, 600, transaction_id).await.unwrap();

        assert_eq!(allocation.client_addr, client_addr);
        assert_eq!(allocation.lifetime, 600);
        assert!(allocation.relay_addr.port() >= 50000 && allocation.relay_addr.port() <= 50100);

        let allocations = relay.list_allocations().await;
        assert_eq!(allocations.len(), 1);
    }

    #[tokio::test]
    async fn test_turn_relay_refresh() {
        let relay = TurnRelay::new(50000, 50100);
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let allocation = relay.allocate(client_addr, 600, transaction_id).await.unwrap();
        let allocation_id = allocation.id;

        relay.refresh(allocation_id, 1200).await.unwrap();

        let updated_allocation = relay.get_allocation(allocation_id).await.unwrap();
        assert_eq!(updated_allocation.lifetime, 1200);
    }

    #[tokio::test]
    async fn test_turn_relay_permissions() {
        let relay = TurnRelay::new(50000, 50100);
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let peer_addr: SocketAddr = "192.168.1.200:6000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let allocation = relay.allocate(client_addr, 600, transaction_id).await.unwrap();
        let allocation_id = allocation.id;

        relay
            .create_permission(allocation_id, peer_addr)
            .await
            .unwrap();

        let updated_allocation = relay.get_allocation(allocation_id).await.unwrap();
        assert!(updated_allocation.has_permission(&peer_addr));
    }

    #[tokio::test]
    async fn test_turn_relay_stats() {
        let relay = TurnRelay::new(50000, 50010); // 11 ports
        let client_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let transaction_id = [1u8; 12];

        let stats = relay.get_stats().await;
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.available_ports, 11);

        relay.allocate(client_addr, 600, transaction_id).await.unwrap();

        let stats = relay.get_stats().await;
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.available_ports, 10);
    }
}

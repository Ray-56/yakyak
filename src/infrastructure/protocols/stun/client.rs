/// STUN client for NAT discovery and binding
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use tracing::{debug, info, warn};

use super::message::{StunMessage, StunMessageType};

/// STUN result containing discovered address
#[derive(Debug, Clone)]
pub struct StunResult {
    pub public_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub server_addr: SocketAddr,
}

/// NAT type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT (direct internet connection)
    OpenInternet,
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT
    Symmetric,
    /// Unknown/Error
    Unknown,
}

/// STUN client
#[derive(Debug, Clone)]
pub struct StunClient {
    server_addr: SocketAddr,
    timeout: Duration,
}

impl StunClient {
    /// Create new STUN client
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            timeout: Duration::from_secs(3),
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Perform STUN binding request
    pub fn binding_request(&self, local_addr: SocketAddr) -> Result<StunResult, String> {
        info!("Performing STUN binding request to {}", self.server_addr);

        // Create UDP socket
        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        socket
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        // Create STUN Binding Request
        let mut request = StunMessage::new_binding_request();
        request.add_software("YakYak STUN Client".to_string());

        let request_bytes = request.to_bytes();

        // Send request
        socket
            .send_to(&request_bytes, self.server_addr)
            .map_err(|e| format!("Failed to send request: {}", e))?;

        debug!("Sent STUN request to {}", self.server_addr);

        // Receive response
        let mut buffer = [0u8; 1500];
        let (size, from) = socket
            .recv_from(&mut buffer)
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        debug!("Received {} bytes from {}", size, from);

        // Parse response
        let response = StunMessage::from_bytes(&buffer[..size])
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Check message type
        if response.message_type != StunMessageType::BindingResponse {
            return Err(format!("Unexpected message type: {:?}", response.message_type));
        }

        // Check transaction ID
        if response.transaction_id != request.transaction_id {
            return Err("Transaction ID mismatch".to_string());
        }

        // Extract public address
        let public_addr = response.get_xor_mapped_address()
            .or_else(|| response.get_mapped_address())
            .ok_or_else(|| "No mapped address in response".to_string())?;

        info!("Discovered public address: {}", public_addr);

        Ok(StunResult {
            public_addr,
            local_addr,
            server_addr: self.server_addr,
        })
    }

    /// Detect NAT type (simplified)
    pub fn detect_nat_type(&self, local_addr: SocketAddr) -> Result<NatType, String> {
        info!("Detecting NAT type");

        // Perform basic binding request
        let result = self.binding_request(local_addr)?;

        // Compare local and public addresses
        if result.local_addr.ip() == result.public_addr.ip() {
            // Same IP means no NAT
            info!("NAT type: Open Internet");
            return Ok(NatType::OpenInternet);
        }

        // If IPs differ, there is NAT
        // More sophisticated detection would require multiple STUN servers
        // and checking if the same public address is seen from different servers

        info!("NAT type: Unknown (requires additional testing)");
        Ok(NatType::Unknown)
    }

    /// Refresh binding (keep NAT binding alive)
    pub fn refresh_binding(&self, local_addr: SocketAddr) -> Result<StunResult, String> {
        debug!("Refreshing STUN binding");
        self.binding_request(local_addr)
    }

    /// Get public IP and port (async-compatible wrapper)
    pub async fn get_public_address(&self) -> Result<(std::net::IpAddr, u16), String> {
        let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        let result = self.binding_request(local_addr)?;
        Ok((result.public_addr.ip(), result.public_addr.port()))
    }

    /// Detect NAT type with enhanced algorithm
    pub async fn detect_nat_type_enhanced(&self, local_addr: SocketAddr) -> Result<NatType, String> {
        info!("Performing enhanced NAT type detection");

        // Test 1: Basic binding request
        let result1 = self.binding_request(local_addr)?;

        // If local and public IPs are the same, no NAT
        if result1.local_addr.ip() == result1.public_addr.ip() {
            info!("NAT type: Open Internet (no NAT)");
            return Ok(NatType::OpenInternet);
        }

        // Test 2: Check if port changed (symmetric NAT indicator)
        if result1.local_addr.port() != result1.public_addr.port() {
            // Port changed - likely symmetric or port-restricted
            info!("Port mapping detected: local {} -> public {}",
                  result1.local_addr.port(), result1.public_addr.port());

            // For more accurate detection, would need multiple STUN servers
            // and compare if public IP/port changes across servers
            info!("NAT type: Likely Symmetric or Port Restricted");
            return Ok(NatType::Symmetric);
        }

        // If port same, likely Full Cone or Restricted Cone
        info!("NAT type: Likely Full Cone or Restricted Cone");
        Ok(NatType::FullCone)
    }
}

/// STUN keepalive manager
pub struct StunKeepalive {
    client: StunClient,
    local_addr: SocketAddr,
    interval: Duration,
}

impl StunKeepalive {
    /// Create new keepalive manager
    pub fn new(client: StunClient, local_addr: SocketAddr, interval: Duration) -> Self {
        Self {
            client,
            local_addr,
            interval,
        }
    }

    /// Start keepalive loop (blocking)
    pub fn run(&self) -> Result<(), String> {
        info!("Starting STUN keepalive (interval: {:?})", self.interval);

        loop {
            match self.client.refresh_binding(self.local_addr) {
                Ok(result) => {
                    debug!("Keepalive successful: {}", result.public_addr);
                }
                Err(e) => {
                    warn!("Keepalive failed: {}", e);
                }
            }

            std::thread::sleep(self.interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_client_creation() {
        let server: SocketAddr = "64.233.177.127:19302".parse().unwrap();
        let client = StunClient::new(server);

        assert_eq!(client.server_addr, server);
        assert_eq!(client.timeout, Duration::from_secs(3));
    }

    #[test]
    fn test_stun_client_with_timeout() {
        let server: SocketAddr = "64.233.177.127:19302".parse().unwrap();
        let client = StunClient::new(server).with_timeout(Duration::from_secs(5));

        assert_eq!(client.timeout, Duration::from_secs(5));
    }

    // Note: The following tests require network access and a STUN server
    // They are commented out to avoid test failures in offline environments

    // #[test]
    // fn test_binding_request_real() {
    //     let server: SocketAddr = "stun.l.google.com:19302".parse().unwrap();
    //     let client = StunClient::new(server);
    //     let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
    //
    //     let result = client.binding_request(local).unwrap();
    //     println!("Public address: {}", result.public_addr);
    // }

    // #[test]
    // fn test_detect_nat_type_real() {
    //     let server: SocketAddr = "stun.l.google.com:19302".parse().unwrap();
    //     let client = StunClient::new(server);
    //     let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
    //
    //     let nat_type = client.detect_nat_type(local).unwrap();
    //     println!("NAT type: {:?}", nat_type);
    // }
}

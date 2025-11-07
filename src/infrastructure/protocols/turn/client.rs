/// TURN client for relay allocation
use super::message::{TurnAttribute, TurnMessage, TurnMessageClass, TurnMessageType, TurnMethod};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, error, warn};

/// TURN client for allocating relay addresses
pub struct TurnClient {
    server_addr: SocketAddr,
    timeout: Duration,
    username: Option<String>,
    password: Option<String>,
}

/// TURN allocation result
#[derive(Debug, Clone)]
pub struct TurnAllocation {
    pub relayed_address: SocketAddr,
    pub lifetime: u32,
    pub transaction_id: [u8; 12],
}

impl TurnClient {
    /// Create a new TURN client
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            timeout: Duration::from_secs(5),
            username: None,
            password: None,
        }
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Allocate a relay address
    pub async fn allocate(&self) -> Result<TurnAllocation, String> {
        debug!("Requesting TURN allocation from {}", self.server_addr);

        // Create UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        // Build Allocate request
        let msg_type = TurnMessageType::new(TurnMethod::Allocate, TurnMessageClass::Request);
        let mut request = TurnMessage::new(msg_type);

        // Add LIFETIME attribute (default 600 seconds)
        request.add_attribute(TurnAttribute::Lifetime(600));

        // Add REQUESTED-TRANSPORT attribute (UDP = 17)
        request.add_attribute(TurnAttribute::RequestedTransport(17));

        // Add authentication if credentials provided
        if let Some(username) = &self.username {
            request.add_attribute(TurnAttribute::Username(username.clone()));
            // TODO: Add MESSAGE-INTEGRITY with HMAC-SHA1
        }

        // Send request
        let request_bytes = request.to_bytes();
        socket
            .send_to(&request_bytes, self.server_addr)
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        // Receive response with timeout
        let mut buffer = [0u8; 1500];
        let response_bytes = match tokio::time::timeout(self.timeout, socket.recv(&mut buffer)).await
        {
            Ok(Ok(size)) => &buffer[..size],
            Ok(Err(e)) => return Err(format!("Failed to receive response: {}", e)),
            Err(_) => return Err("Request timeout".to_string()),
        };

        // Parse response
        let response = TurnMessage::parse(response_bytes)?;

        // Check if success response
        if response.message_type.class != TurnMessageClass::SuccessResponse {
            if response.message_type.class == TurnMessageClass::ErrorResponse {
                return Err("TURN server returned error".to_string());
            }
            return Err("Unexpected response type".to_string());
        }

        // Extract relayed address
        let relayed_address = response
            .get_relayed_address()
            .ok_or_else(|| "No relayed address in response".to_string())?;

        let lifetime = response.get_lifetime().unwrap_or(600);

        debug!(
            "TURN allocation successful: relay={}, lifetime={}",
            relayed_address, lifetime
        );

        Ok(TurnAllocation {
            relayed_address,
            lifetime,
            transaction_id: response.transaction_id,
        })
    }

    /// Refresh an existing allocation
    pub async fn refresh(&self, transaction_id: [u8; 12], lifetime: u32) -> Result<u32, String> {
        debug!("Refreshing TURN allocation");

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        let msg_type = TurnMessageType::new(TurnMethod::Refresh, TurnMessageClass::Request);
        let mut request = TurnMessage::new(msg_type);
        request.transaction_id = transaction_id;
        request.add_attribute(TurnAttribute::Lifetime(lifetime));

        if let Some(username) = &self.username {
            request.add_attribute(TurnAttribute::Username(username.clone()));
        }

        let request_bytes = request.to_bytes();
        socket
            .send_to(&request_bytes, self.server_addr)
            .await
            .map_err(|e| format!("Failed to send refresh: {}", e))?;

        let mut buffer = [0u8; 1500];
        let response_bytes = match tokio::time::timeout(self.timeout, socket.recv(&mut buffer)).await
        {
            Ok(Ok(size)) => &buffer[..size],
            Ok(Err(e)) => return Err(format!("Failed to receive response: {}", e)),
            Err(_) => return Err("Request timeout".to_string()),
        };

        let response = TurnMessage::parse(response_bytes)?;

        if response.message_type.class != TurnMessageClass::SuccessResponse {
            return Err("Refresh failed".to_string());
        }

        let new_lifetime = response.get_lifetime().unwrap_or(lifetime);
        debug!("Allocation refreshed, new lifetime: {}", new_lifetime);

        Ok(new_lifetime)
    }

    /// Send data through the TURN relay
    pub async fn send_indication(
        &self,
        peer_addr: SocketAddr,
        data: &[u8],
    ) -> Result<(), String> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        let msg_type = TurnMessageType::new(TurnMethod::Send, TurnMessageClass::Indication);
        let mut message = TurnMessage::new(msg_type);
        message.add_attribute(TurnAttribute::XorPeerAddress(peer_addr));
        message.add_attribute(TurnAttribute::Data(data.to_vec()));

        let message_bytes = message.to_bytes();
        socket
            .send_to(&message_bytes, self.server_addr)
            .await
            .map_err(|e| format!("Failed to send data: {}", e))?;

        Ok(())
    }

    /// Create a permission for a peer address
    pub async fn create_permission(&self, peer_addr: SocketAddr) -> Result<(), String> {
        debug!("Creating permission for peer: {}", peer_addr);

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        let msg_type = TurnMessageType::new(TurnMethod::CreatePermission, TurnMessageClass::Request);
        let mut request = TurnMessage::new(msg_type);
        request.add_attribute(TurnAttribute::XorPeerAddress(peer_addr));

        if let Some(username) = &self.username {
            request.add_attribute(TurnAttribute::Username(username.clone()));
        }

        let request_bytes = request.to_bytes();
        socket
            .send_to(&request_bytes, self.server_addr)
            .await
            .map_err(|e| format!("Failed to send permission request: {}", e))?;

        let mut buffer = [0u8; 1500];
        let response_bytes = match tokio::time::timeout(self.timeout, socket.recv(&mut buffer)).await
        {
            Ok(Ok(size)) => &buffer[..size],
            Ok(Err(e)) => return Err(format!("Failed to receive response: {}", e)),
            Err(_) => return Err("Request timeout".to_string()),
        };

        let response = TurnMessage::parse(response_bytes)?;

        if response.message_type.class == TurnMessageClass::SuccessResponse {
            debug!("Permission created for {}", peer_addr);
            Ok(())
        } else {
            Err("Failed to create permission".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_client_creation() {
        let server_addr: SocketAddr = "192.168.1.1:3478".parse().unwrap();
        let client = TurnClient::new(server_addr);
        assert_eq!(client.server_addr, server_addr);
        assert_eq!(client.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_turn_client_with_credentials() {
        let server_addr: SocketAddr = "192.168.1.1:3478".parse().unwrap();
        let client = TurnClient::new(server_addr)
            .with_credentials("user".to_string(), "pass".to_string());
        assert_eq!(client.username, Some("user".to_string()));
        assert_eq!(client.password, Some("pass".to_string()));
    }

    #[test]
    fn test_turn_client_with_timeout() {
        let server_addr: SocketAddr = "192.168.1.1:3478".parse().unwrap();
        let client = TurnClient::new(server_addr).with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    // Note: Integration tests require a running TURN server
    // Run with: cargo test --features turn turn_client -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_turn_allocation() {
        let server_addr: SocketAddr = "stun.l.google.com:19302".parse().unwrap();
        let client = TurnClient::new(server_addr);

        // Note: This will likely fail without proper TURN server
        // but demonstrates the API
        match client.allocate().await {
            Ok(allocation) => {
                println!("Relayed address: {}", allocation.relayed_address);
                println!("Lifetime: {}", allocation.lifetime);
            }
            Err(e) => {
                println!("Allocation failed (expected): {}", e);
            }
        }
    }
}

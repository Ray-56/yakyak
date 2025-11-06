//! SIP transport layer - handles UDP, TCP, TLS, WebSocket

use super::message::{SipError, SipMessage};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Transport protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportProtocol {
    Udp,
    Tcp,
    Tls,
    Ws,
    Wss,
}

impl TransportProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportProtocol::Udp => "UDP",
            TransportProtocol::Tcp => "TCP",
            TransportProtocol::Tls => "TLS",
            TransportProtocol::Ws => "WS",
            TransportProtocol::Wss => "WSS",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            TransportProtocol::Udp => 5060,
            TransportProtocol::Tcp => 5060,
            TransportProtocol::Tls => 5061,
            TransportProtocol::Ws => 80,
            TransportProtocol::Wss => 443,
        }
    }
}

/// Incoming SIP message with source information
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub message: SipMessage,
    pub source: SocketAddr,
    pub protocol: TransportProtocol,
}

/// Outgoing SIP message with destination information
#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    pub data: Bytes,
    pub destination: SocketAddr,
    pub protocol: TransportProtocol,
}

/// Transport layer trait
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Start the transport
    async fn start(&mut self) -> Result<(), SipError>;

    /// Stop the transport
    async fn stop(&mut self) -> Result<(), SipError>;

    /// Send a message
    async fn send(&self, message: OutgoingMessage) -> Result<(), SipError>;

    /// Get the receiver for incoming messages
    fn receiver(&mut self) -> &mut mpsc::Receiver<IncomingMessage>;
}

/// UDP transport implementation
pub struct UdpTransport {
    bind_addr: SocketAddr,
    pub socket: Option<Arc<UdpSocket>>,
    tx: mpsc::Sender<IncomingMessage>,
    rx: mpsc::Receiver<IncomingMessage>,
}

impl UdpTransport {
    pub fn new(bind_addr: SocketAddr) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            bind_addr,
            socket: None,
            tx,
            rx,
        }
    }

    async fn receive_loop(socket: Arc<UdpSocket>, tx: mpsc::Sender<IncomingMessage>) {
        let mut buf = vec![0u8; 65535];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, source)) => {
                    debug!("Received {} bytes from {} via UDP", size, source);

                    match SipMessage::parse(&buf[..size]) {
                        Ok(message) => {
                            let incoming = IncomingMessage {
                                message,
                                source,
                                protocol: TransportProtocol::Udp,
                            };

                            if let Err(e) = tx.send(incoming).await {
                                error!("Failed to send incoming message to channel: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse SIP message from {}: {}", source, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive UDP packet: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Transport for UdpTransport {
    async fn start(&mut self) -> Result<(), SipError> {
        info!("Starting UDP transport on {}", self.bind_addr);

        let socket = UdpSocket::bind(self.bind_addr)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to bind UDP socket: {}", e)))?;

        info!("UDP transport listening on {}", socket.local_addr().unwrap());

        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());

        // Start receive loop in background
        let tx = self.tx.clone();
        tokio::spawn(async move {
            Self::receive_loop(socket, tx).await;
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), SipError> {
        info!("Stopping UDP transport");
        self.socket = None;
        Ok(())
    }

    async fn send(&self, message: OutgoingMessage) -> Result<(), SipError> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| SipError::TransportError("Socket not initialized".to_string()))?;

        debug!(
            "Sending {} bytes to {} via UDP",
            message.data.len(),
            message.destination
        );

        socket
            .send_to(&message.data, message.destination)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to send UDP packet: {}", e)))?;

        Ok(())
    }

    fn receiver(&mut self) -> &mut mpsc::Receiver<IncomingMessage> {
        &mut self.rx
    }
}

/// TCP transport implementation
pub struct TcpTransport {
    bind_addr: SocketAddr,
    listener: Option<TcpListener>,
    tx: mpsc::Sender<IncomingMessage>,
    rx: mpsc::Receiver<IncomingMessage>,
}

impl TcpTransport {
    pub fn new(bind_addr: SocketAddr) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            bind_addr,
            listener: None,
            tx,
            rx,
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        source: SocketAddr,
        tx: mpsc::Sender<IncomingMessage>,
    ) {
        use tokio::io::AsyncReadExt;

        let mut buf = vec![0u8; 65535];

        loop {
            match stream.read(&mut buf).await {
                Ok(0) => {
                    debug!("TCP connection closed by {}", source);
                    break;
                }
                Ok(size) => {
                    debug!("Received {} bytes from {} via TCP", size, source);

                    match SipMessage::parse(&buf[..size]) {
                        Ok(message) => {
                            let incoming = IncomingMessage {
                                message,
                                source,
                                protocol: TransportProtocol::Tcp,
                            };

                            if let Err(e) = tx.send(incoming).await {
                                error!("Failed to send incoming message to channel: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse SIP message from {}: {}", source, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from TCP connection: {}", e);
                    break;
                }
            }
        }
    }

    async fn accept_loop(listener: TcpListener, tx: mpsc::Sender<IncomingMessage>) {
        loop {
            match listener.accept().await {
                Ok((stream, source)) => {
                    info!("Accepted TCP connection from {}", source);
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        Self::handle_connection(stream, source, tx).await;
                    });
                }
                Err(e) => {
                    error!("Failed to accept TCP connection: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn start(&mut self) -> Result<(), SipError> {
        info!("Starting TCP transport on {}", self.bind_addr);

        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to bind TCP socket: {}", e)))?;

        info!("TCP transport listening on {}", listener.local_addr().unwrap());

        // Start accept loop in background
        let tx = self.tx.clone();
        tokio::spawn(async move {
            Self::accept_loop(listener, tx).await;
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), SipError> {
        info!("Stopping TCP transport");
        self.listener = None;
        Ok(())
    }

    async fn send(&self, message: OutgoingMessage) -> Result<(), SipError> {
        use tokio::io::AsyncWriteExt;

        debug!(
            "Sending {} bytes to {} via TCP",
            message.data.len(),
            message.destination
        );

        let mut stream = TcpStream::connect(message.destination)
            .await
            .map_err(|e| {
                SipError::TransportError(format!("Failed to connect to {}: {}", message.destination, e))
            })?;

        stream
            .write_all(&message.data)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to send TCP data: {}", e)))?;

        stream
            .flush()
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to flush TCP stream: {}", e)))?;

        Ok(())
    }

    fn receiver(&mut self) -> &mut mpsc::Receiver<IncomingMessage> {
        &mut self.rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_transport_start() {
        let bind_addr = "127.0.0.1:0".parse().unwrap();
        let mut transport = UdpTransport::new(bind_addr);

        let result = transport.start().await;
        assert!(result.is_ok());

        // Clean up
        transport.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_tcp_transport_start() {
        let bind_addr = "127.0.0.1:0".parse().unwrap();
        let mut transport = TcpTransport::new(bind_addr);

        let result = transport.start().await;
        assert!(result.is_ok());

        // Clean up
        transport.stop().await.unwrap();
    }
}

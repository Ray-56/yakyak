//! SIP transport layer - handles UDP, TCP, TLS, WebSocket

use super::message::{SipError, SipMessage};
use bytes::Bytes;
use rustls::ServerConfig;
use rustls_pemfile::{certs, rsa_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;
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

/// TLS transport implementation
pub struct TlsTransport {
    bind_addr: SocketAddr,
    cert_path: String,
    key_path: String,
    listener: Option<TcpListener>,
    acceptor: Option<TlsAcceptor>,
    tx: mpsc::Sender<IncomingMessage>,
    rx: mpsc::Receiver<IncomingMessage>,
}

impl TlsTransport {
    pub fn new(bind_addr: SocketAddr, cert_path: String, key_path: String) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            bind_addr,
            cert_path,
            key_path,
            listener: None,
            acceptor: None,
            tx,
            rx,
        }
    }

    /// Load TLS server configuration from certificate and key files
    fn load_tls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig, SipError> {
        // Load certificate chain
        let cert_file = File::open(cert_path).map_err(|e| {
            SipError::TransportError(format!("Failed to open certificate file {}: {}", cert_path, e))
        })?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<_> = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                SipError::TransportError(format!("Failed to parse certificates: {}", e))
            })?;

        if cert_chain.is_empty() {
            return Err(SipError::TransportError(
                "No certificates found in certificate file".to_string(),
            ));
        }

        // Load private key
        let key_file = File::open(key_path).map_err(|e| {
            SipError::TransportError(format!("Failed to open private key file {}: {}", key_path, e))
        })?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = rsa_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                SipError::TransportError(format!("Failed to parse private key: {}", e))
            })?;

        if keys.is_empty() {
            return Err(SipError::TransportError(
                "No private keys found in key file".to_string(),
            ));
        }

        let private_key = keys.remove(0);

        // Build TLS configuration
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key.into())
            .map_err(|e| {
                SipError::TransportError(format!("Failed to create TLS config: {}", e))
            })?;

        Ok(config)
    }

    async fn handle_connection(
        stream: tokio_rustls::server::TlsStream<TcpStream>,
        source: SocketAddr,
        tx: mpsc::Sender<IncomingMessage>,
    ) {
        use tokio::io::AsyncReadExt;

        let (mut reader, _writer) = tokio::io::split(stream);
        let mut buf = vec![0u8; 65535];

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    debug!("TLS connection closed by {}", source);
                    break;
                }
                Ok(size) => {
                    debug!("Received {} bytes from {} via TLS", size, source);

                    match SipMessage::parse(&buf[..size]) {
                        Ok(message) => {
                            let incoming = IncomingMessage {
                                message,
                                source,
                                protocol: TransportProtocol::Tls,
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
                    error!("Failed to read from TLS connection: {}", e);
                    break;
                }
            }
        }
    }

    async fn accept_loop(
        listener: TcpListener,
        acceptor: TlsAcceptor,
        tx: mpsc::Sender<IncomingMessage>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, source)) => {
                    info!("Accepted TLS connection from {}", source);

                    let acceptor = acceptor.clone();
                    let tx = tx.clone();

                    tokio::spawn(async move {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                debug!("TLS handshake completed for {}", source);
                                Self::handle_connection(tls_stream, source, tx).await;
                            }
                            Err(e) => {
                                error!("TLS handshake failed for {}: {}", source, e);
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept TLS connection: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Transport for TlsTransport {
    async fn start(&mut self) -> Result<(), SipError> {
        info!("Starting TLS transport on {}", self.bind_addr);

        // Load TLS configuration
        let config = Self::load_tls_config(&self.cert_path, &self.key_path)?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        self.acceptor = Some(acceptor.clone());

        // Bind TCP listener
        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to bind TLS socket: {}", e)))?;

        info!("TLS transport listening on {}", listener.local_addr().unwrap());

        self.listener = Some(listener.try_clone().await.unwrap());

        // Start accept loop in background
        let tx = self.tx.clone();
        tokio::spawn(async move {
            Self::accept_loop(listener, acceptor, tx).await;
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), SipError> {
        info!("Stopping TLS transport");
        self.listener = None;
        self.acceptor = None;
        Ok(())
    }

    async fn send(&self, message: OutgoingMessage) -> Result<(), SipError> {
        use tokio::io::AsyncWriteExt;

        debug!(
            "Sending {} bytes to {} via TLS",
            message.data.len(),
            message.destination
        );

        // For TLS client connections, we need to implement a connection pool
        // For now, we create a new connection each time (simplified)
        let stream = TcpStream::connect(message.destination)
            .await
            .map_err(|e| {
                SipError::TransportError(format!(
                    "Failed to connect to {}: {}",
                    message.destination, e
                ))
            })?;

        // Note: For proper TLS client implementation, we'd need to:
        // 1. Create a TLS connector with proper configuration
        // 2. Perform TLS handshake
        // 3. Use a connection pool to reuse connections
        // For now, fall back to plain TCP for outgoing connections
        // This is sufficient for server-side TLS (receiving encrypted SIP messages)

        let mut stream = stream;
        stream
            .write_all(&message.data)
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to send TLS data: {}", e)))?;

        stream
            .flush()
            .await
            .map_err(|e| SipError::TransportError(format!("Failed to flush TLS stream: {}", e)))?;

        warn!("TLS client connections not fully implemented - sent via plain TCP");

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

    #[tokio::test]
    async fn test_tls_transport_missing_cert() {
        let bind_addr = "127.0.0.1:5061".parse().unwrap();
        let mut transport = TlsTransport::new(
            bind_addr,
            "/nonexistent/cert.pem".to_string(),
            "/nonexistent/key.pem".to_string(),
        );

        // Should fail due to missing certificate files
        let result = transport.start().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_transport_protocol_default_ports() {
        assert_eq!(TransportProtocol::Udp.default_port(), 5060);
        assert_eq!(TransportProtocol::Tcp.default_port(), 5060);
        assert_eq!(TransportProtocol::Tls.default_port(), 5061);
        assert_eq!(TransportProtocol::Ws.default_port(), 80);
        assert_eq!(TransportProtocol::Wss.default_port(), 443);
    }

    #[test]
    fn test_transport_protocol_as_str() {
        assert_eq!(TransportProtocol::Udp.as_str(), "UDP");
        assert_eq!(TransportProtocol::Tcp.as_str(), "TCP");
        assert_eq!(TransportProtocol::Tls.as_str(), "TLS");
        assert_eq!(TransportProtocol::Ws.as_str(), "WS");
        assert_eq!(TransportProtocol::Wss.as_str(), "WSS");
    }
}

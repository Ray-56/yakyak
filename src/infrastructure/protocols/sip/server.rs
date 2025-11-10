//! SIP server implementation (Simplified version)

use super::builder::ResponseBuilder;
use super::handler::SipHandler;
use super::message::{SipError, SipMessage, SipMethod};
use super::transport::{IncomingMessage, TcpTransport, Transport, UdpTransport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// SIP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SipServerConfig {
    pub udp_bind: SocketAddr,
    pub tcp_bind: SocketAddr,
    pub domain: String,
    pub enable_tcp: bool,
    /// Enable TLS transport (SIPS)
    pub enable_tls: bool,
    /// TLS bind address (default: port 5061)
    pub tls_bind: SocketAddr,
    /// Path to TLS certificate file
    pub tls_cert_path: String,
    /// Path to TLS private key file
    pub tls_key_path: String,
}

impl Default for SipServerConfig {
    fn default() -> Self {
        Self {
            udp_bind: "0.0.0.0:5060".parse().unwrap(),
            tcp_bind: "0.0.0.0:5060".parse().unwrap(),
            domain: "localhost".to_string(),
            enable_tcp: true,
            enable_tls: false,
            tls_bind: "0.0.0.0:5061".parse().unwrap(),
            tls_cert_path: "certs/server.crt".to_string(),
            tls_key_path: "certs/server.key".to_string(),
        }
    }
}

use super::transport::TlsTransport;

/// SIP server
pub struct SipServer {
    config: SipServerConfig,
    udp_transport: Option<UdpTransport>,
    tcp_transport: Option<TcpTransport>,
    tls_transport: Option<TlsTransport>,
    handlers: Arc<RwLock<HashMap<SipMethod, Arc<dyn SipHandler>>>>,
}

impl SipServer {
    pub fn new(config: SipServerConfig) -> Self {
        Self {
            config: config.clone(),
            udp_transport: Some(UdpTransport::new(config.udp_bind)),
            tcp_transport: if config.enable_tcp {
                Some(TcpTransport::new(config.tcp_bind))
            } else {
                None
            },
            tls_transport: if config.enable_tls {
                Some(TlsTransport::new(
                    config.tls_bind,
                    config.tls_cert_path.clone(),
                    config.tls_key_path.clone(),
                ))
            } else {
                None
            },
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_handler(&self, method: SipMethod, handler: Arc<dyn SipHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(method, handler);
        info!("Registered handler for SIP method: {}", method);
    }

    pub async fn start(&mut self) -> Result<(), SipError> {
        info!("Starting SIP server");
        info!("Domain: {}", self.config.domain);

        // Start UDP transport and get receiver
        let mut udp_rx = None;
        let udp_socket = if let Some(transport) = &mut self.udp_transport {
            transport.start().await?;
            info!("UDP transport started on {}", self.config.udp_bind);
            udp_rx = Some(std::mem::replace(
                transport.receiver(),
                mpsc::channel(1).1,
            ));
            transport.socket.clone()
        } else {
            None
        };

        // Start TCP transport and get receiver
        let mut tcp_rx = None;
        if let Some(transport) = &mut self.tcp_transport {
            transport.start().await?;
            info!("TCP transport started on {}", self.config.tcp_bind);
            tcp_rx = Some(std::mem::replace(
                transport.receiver(),
                mpsc::channel(1).1,
            ));
        }

        // Start TLS transport and get receiver
        let mut tls_rx = None;
        if let Some(transport) = &mut self.tls_transport {
            match transport.start().await {
                Ok(_) => {
                    info!("TLS transport started on {}", self.config.tls_bind);
                    tls_rx = Some(std::mem::replace(
                        transport.receiver(),
                        mpsc::channel(1).1,
                    ));
                }
                Err(e) => {
                    warn!("Failed to start TLS transport: {}. Continuing without TLS.", e);
                }
            }
        }

        // Start message processing
        if let Some(mut rx) = udp_rx {
            let handlers = self.handlers.clone();
            let socket = udp_socket;
            tokio::spawn(async move {
                while let Some(incoming) = rx.recv().await {
                    let handlers = handlers.clone();
                    let socket = socket.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::process_udp_message(incoming, handlers, socket).await {
                            error!("Error processing UDP message: {}", e);
                        }
                    });
                }
            });
        }

        if let Some(mut rx) = tcp_rx {
            let handlers = self.handlers.clone();
            tokio::spawn(async move {
                while let Some(incoming) = rx.recv().await {
                    let handlers = handlers.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::process_tcp_message(incoming, handlers).await {
                            error!("Error processing TCP message: {}", e);
                        }
                    });
                }
            });
        }

        if let Some(mut rx) = tls_rx {
            let handlers = self.handlers.clone();
            tokio::spawn(async move {
                while let Some(incoming) = rx.recv().await {
                    let handlers = handlers.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::process_tls_message(incoming, handlers).await {
                            error!("Error processing TLS message: {}", e);
                        }
                    });
                }
            });
        }

        info!("SIP server started successfully");
        Ok(())
    }

    async fn process_udp_message(
        incoming: IncomingMessage,
        handlers: Arc<RwLock<HashMap<SipMethod, Arc<dyn SipHandler>>>>,
        socket: Option<Arc<tokio::net::UdpSocket>>,
    ) -> Result<(), SipError> {
        match incoming.message {
            SipMessage::Request(request) => {
                let method = request.method();
                debug!("Processing SIP request: {:?}", method);

                let handlers = handlers.read().await;
                if let Some(method) = method {
                    if let Some(handler) = handlers.get(&method) {
                        match handler.handle_request(request.clone()).await {
                            Ok(response) => {
                                if let Some(sock) = socket.as_ref() {
                                    let data = response.to_bytes();
                                    if let Err(e) = sock.send_to(&data, incoming.source).await {
                                        error!("Failed to send response: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Handler error: {}", e);
                                if let Some(sock) = socket.as_ref() {
                                    if let Ok(error_response) =
                                        ResponseBuilder::server_internal_error()
                                            .build_for_request(&request)
                                    {
                                        let data = error_response.to_bytes();
                                        let _ = sock.send_to(&data, incoming.source).await;
                                    }
                                }
                            }
                        }
                    } else {
                        warn!("No handler registered for method: {}", method);
                        if let Some(sock) = socket.as_ref() {
                            if let Ok(response) =
                                ResponseBuilder::new(501).build_for_request(&request)
                            {
                                let data = response.to_bytes();
                                let _ = sock.send_to(&data, incoming.source).await;
                            }
                        }
                    }
                }
            }
            SipMessage::Response(response) => {
                debug!("Received SIP response: {}", response.status_code());
            }
        }

        Ok(())
    }

    async fn process_tcp_message(
        incoming: IncomingMessage,
        handlers: Arc<RwLock<HashMap<SipMethod, Arc<dyn SipHandler>>>>,
    ) -> Result<(), SipError> {
        match incoming.message {
            SipMessage::Request(request) => {
                let method = request.method();
                debug!("Processing SIP request via TCP: {:?}", method);

                let handlers = handlers.read().await;
                if let Some(method) = method {
                    if let Some(handler) = handlers.get(&method) {
                        match handler.handle_request(request).await {
                            Ok(response) => {
                                debug!("Response generated: {}", response.status_code());
                            }
                            Err(e) => {
                                error!("Handler error: {}", e);
                            }
                        }
                    }
                }
            }
            SipMessage::Response(response) => {
                debug!("Received SIP response via TCP: {}", response.status_code());
            }
        }

        Ok(())
    }

    async fn process_tls_message(
        incoming: IncomingMessage,
        handlers: Arc<RwLock<HashMap<SipMethod, Arc<dyn SipHandler>>>>,
    ) -> Result<(), SipError> {
        match incoming.message {
            SipMessage::Request(request) => {
                let method = request.method();
                debug!("Processing SIP request via TLS: {:?}", method);

                let handlers = handlers.read().await;
                if let Some(method) = method {
                    if let Some(handler) = handlers.get(&method) {
                        match handler.handle_request(request).await {
                            Ok(response) => {
                                debug!("Response generated: {}", response.status_code());
                            }
                            Err(e) => {
                                error!("Handler error: {}", e);
                            }
                        }
                    } else {
                        warn!("No handler registered for method: {}", method);
                    }
                }
            }
            SipMessage::Response(response) => {
                debug!("Received SIP response via TLS: {}", response.status_code());
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), SipError> {
        info!("Stopping SIP server");

        if let Some(transport) = &mut self.udp_transport {
            transport.stop().await?;
        }

        if let Some(transport) = &mut self.tcp_transport {
            transport.stop().await?;
        }

        if let Some(transport) = &mut self.tls_transport {
            transport.stop().await?;
        }

        info!("SIP server stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sip_server_creation() {
        let config = SipServerConfig {
            udp_bind: "127.0.0.1:0".parse().unwrap(),
            tcp_bind: "127.0.0.1:0".parse().unwrap(),
            domain: "test.com".to_string(),
            enable_tcp: false,
        };

        let server = SipServer::new(config);
        assert_eq!(server.config.domain, "test.com");
    }
}

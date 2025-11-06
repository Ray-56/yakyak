//! Media Stream Management

use super::rtp::{RtpPacket, RtpSession, SenderReport};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Media Stream Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDirection {
    SendOnly,
    RecvOnly,
    SendRecv,
    Inactive,
}

/// Media Stream
///
/// Manages RTP and RTCP for a single media stream
pub struct MediaStream {
    /// RTP session
    rtp_session: Arc<RtpSession>,
    /// Local RTP socket
    rtp_socket: Arc<UdpSocket>,
    /// Local RTCP socket
    rtcp_socket: Arc<UdpSocket>,
    /// Remote RTP address
    remote_rtp: Arc<RwLock<Option<SocketAddr>>>,
    /// Remote RTCP address
    remote_rtcp: Arc<RwLock<Option<SocketAddr>>>,
    /// Stream direction
    direction: Arc<RwLock<StreamDirection>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl MediaStream {
    /// Create a new media stream
    pub async fn new(
        local_rtp_port: u16,
        payload_type: u8,
        clock_rate: u32,
    ) -> Result<Self, std::io::Error> {
        // Bind RTP socket
        let rtp_addr = format!("0.0.0.0:{}", local_rtp_port);
        let rtp_socket = UdpSocket::bind(&rtp_addr).await?;
        info!("RTP socket bound to {}", rtp_addr);

        // Bind RTCP socket (RTP port + 1)
        let rtcp_addr = format!("0.0.0.0:{}", local_rtp_port + 1);
        let rtcp_socket = UdpSocket::bind(&rtcp_addr).await?;
        info!("RTCP socket bound to {}", rtcp_addr);

        let rtp_session = Arc::new(RtpSession::new(payload_type, clock_rate));

        Ok(Self {
            rtp_session,
            rtp_socket: Arc::new(rtp_socket),
            rtcp_socket: Arc::new(rtcp_socket),
            remote_rtp: Arc::new(RwLock::new(None)),
            remote_rtcp: Arc::new(RwLock::new(None)),
            direction: Arc::new(RwLock::new(StreamDirection::Inactive)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Set remote addresses
    pub async fn set_remote(&self, rtp_addr: SocketAddr, rtcp_addr: SocketAddr) {
        *self.remote_rtp.write().await = Some(rtp_addr);
        *self.remote_rtcp.write().await = Some(rtcp_addr);
        info!("Remote RTP: {}, RTCP: {}", rtp_addr, rtcp_addr);
    }

    /// Set stream direction
    pub async fn set_direction(&self, direction: StreamDirection) {
        *self.direction.write().await = direction;
        info!("Stream direction: {:?}", direction);
    }

    /// Get SSRC
    pub fn ssrc(&self) -> u32 {
        self.rtp_session.ssrc()
    }

    /// Get local RTP port
    pub fn local_rtp_port(&self) -> Result<u16, std::io::Error> {
        Ok(self.rtp_socket.local_addr()?.port())
    }

    /// Send RTP packet
    pub async fn send_rtp(&self, payload: Bytes, timestamp: u32, marker: bool) -> Result<(), std::io::Error> {
        let direction = *self.direction.read().await;
        if direction == StreamDirection::RecvOnly || direction == StreamDirection::Inactive {
            return Ok(()); // Can't send in recv-only or inactive mode
        }

        let packet = self.rtp_session.create_packet(payload, timestamp, marker);
        let data = packet.serialize();

        if let Some(remote) = *self.remote_rtp.read().await {
            self.rtp_socket.send_to(&data, remote).await?;
            debug!("Sent RTP packet to {}: {} bytes", remote, data.len());
        } else {
            warn!("No remote RTP address set");
        }

        Ok(())
    }

    /// Start receiving RTP packets
    pub async fn start(&self) -> Result<(), std::io::Error> {
        *self.running.write().await = true;

        // Spawn RTP receiver task
        let rtp_socket = self.rtp_socket.clone();
        let direction = self.direction.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];

            while *running.read().await {
                let dir = *direction.read().await;
                if dir == StreamDirection::SendOnly || dir == StreamDirection::Inactive {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                match rtp_socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        debug!("Received RTP packet from {}: {} bytes", addr, len);

                        match RtpPacket::parse(&buf[..len]) {
                            Ok(packet) => {
                                debug!("Parsed RTP: {}", packet);
                                // TODO: Process received packet (decode, play, etc.)
                            }
                            Err(e) => {
                                warn!("Failed to parse RTP packet: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        if *running.read().await {
                            error!("RTP recv error: {}", e);
                        }
                    }
                }
            }

            info!("RTP receiver stopped");
        });

        // Spawn RTCP sender task
        let rtcp_socket = self.rtcp_socket.clone();
        let rtp_session = self.rtp_session.clone();
        let remote_rtcp = self.remote_rtcp.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(5));

            while *running.read().await {
                timer.tick().await;

                if let Some(remote) = *remote_rtcp.read().await {
                    // Send Sender Report
                    let sr = SenderReport::new(
                        rtp_session.ssrc(),
                        0, // timestamp
                        rtp_session.packets_sent(),
                        rtp_session.bytes_sent(),
                    );

                    let data = sr.serialize();

                    match rtcp_socket.send_to(&data, remote).await {
                        Ok(_) => {
                            debug!("Sent RTCP SR to {}", remote);
                        }
                        Err(e) => {
                            warn!("Failed to send RTCP SR: {}", e);
                        }
                    }
                }
            }

            info!("RTCP sender stopped");
        });

        info!("Media stream started");
        Ok(())
    }

    /// Stop the stream
    pub async fn stop(&self) {
        *self.running.write().await = false;
        info!("Media stream stopped");
    }
}

impl Drop for MediaStream {
    fn drop(&mut self) {
        // Ensure stream is stopped
        let running = self.running.clone();
        tokio::spawn(async move {
            *running.write().await = false;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_media_stream_creation() {
        let stream = MediaStream::new(10000, 0, 8000).await.unwrap();
        assert_eq!(stream.local_rtp_port().unwrap(), 10000);
    }

    #[tokio::test]
    async fn test_set_remote() {
        let stream = MediaStream::new(10002, 0, 8000).await.unwrap();
        let rtp_addr: SocketAddr = "127.0.0.1:20000".parse().unwrap();
        let rtcp_addr: SocketAddr = "127.0.0.1:20001".parse().unwrap();

        stream.set_remote(rtp_addr, rtcp_addr).await;

        assert!(stream.remote_rtp.read().await.is_some());
        assert!(stream.remote_rtcp.read().await.is_some());
    }

    #[tokio::test]
    async fn test_stream_direction() {
        let stream = MediaStream::new(10004, 0, 8000).await.unwrap();
        stream.set_direction(StreamDirection::SendRecv).await;

        assert_eq!(*stream.direction.read().await, StreamDirection::SendRecv);
    }
}

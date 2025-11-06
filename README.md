# YakYak

Talk the Yak, Connect the Pack! 🦬📞

<p align="center">
  <img src="./assets/images/logo.png" alt="YakYak Logo" width="150"/>
</p>

<h1 align="center">YakYak</h1>
<p align="center">Talk the Yak, Connect the Pack! 🦬📞</p>

<p align="center">
  <a href="https://github.com/Ray-56/yakyak/actions"><img src="https://img.shields.io/github/workflow/status/Ray-56/yakyak/CI?label=CI" alt="CI Status"></a>
  <a href="https://github.com/Ray-56/yakyak/blob/main/LICENSE"><img src="https://img.shields.io/github/license/Ray-56/yakyak" alt="License"></a>
  <a href="https://github.com/Ray-56/yakyak/releases"><img src="https://img.shields.io/github/v/release/Ray-56/yakyak" alt="Release"></a>
</p>

---

YakYak is an open-source VoIP platform built for real-time voice and video communication. Powered by SIP, WebRTC, and WebSocket JSON-RPC, it’s designed to be robust, extensible, and fun to hack on. Whether you're building a chat app, a conference system, or just want to yak with friends, YakYak has you covered!

### ✨ Features

- 📞 **SIP-based Calling**: Seamless voice and video calls with SIP protocol support.
- 🔐 **SIP Digest Authentication**: Secure user authentication with MD5 Digest (RFC 2617). [See AUTH.md](AUTH.md)
- 📝 **User Registration**: SIP REGISTER support with automatic expiration management.
- 🌐 **WebRTC Integration**: Peer-to-peer audio/video with modern browser compatibility (planned).
- ⚡ **WebSocket JSON-RPC**: Real-time signaling for call control and management (planned).
- 🏗️ **DDD Architecture**: Clean domain-driven design with proper bounded contexts.
- 🦀 **Memory Safe**: Built with Rust for safety and performance.
- 🦬 **Scalable & Fun**: Built for developers who love to tinker and connect communities.

_More features (RTP media, conferencing, IVR, CDR) are on the way!_

### 🚀 Quick Start

#### Prerequisites

- Rust (stable, 1.65+)
- PostgreSQL (for user data)
- A STUN/TURN server (e.g., `coturn`)

#### Installation

```bash
# Clone the repo
git clone https://github.com/Ray-56/yakyak.git
cd yakyak

# Build and run
cargo build --release
cargo run -- --config config.toml
```

# 加密配置指南

本文档描述如何在 YakYak PBX 系统中配置和使用 TLS 和 SRTP 加密。

## 目录

- [TLS (传输层加密)](#tls-传输层加密)
- [SRTP (媒体加密)](#srtp-媒体加密)
- [生产环境部署](#生产环境部署)
- [故障排除](#故障排除)

---

## TLS (传输层加密)

TLS 加密保护 SIP 信令流量，防止窃听和中间人攻击。

### 配置 TLS

#### 1. 生成证书

**开发/测试环境**（使用自签名证书）：

```bash
./scripts/generate_test_certs.sh
```

这将在 `certs/` 目录下生成：
- `server.crt` - 自签名证书
- `server.key` - 私钥

⚠️ **警告**: 不要在生产环境使用自签名证书！

**生产环境**（使用 CA 签名证书）：

1. 生成 CSR（证书签名请求）：
```bash
openssl req -new -newkey rsa:2048 -nodes \
    -keyout server.key \
    -out server.csr \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=pbx.example.com"
```

2. 将 CSR 提交给 CA（如 Let's Encrypt, DigiCert等）获取证书

3. 将证书和私钥放置在安全目录（如 `/etc/yakyak/certs/`）

#### 2. 配置 SipServer

在代码中启用 TLS：

```rust
use yakyak::infrastructure::protocols::sip::{SipServer, SipServerConfig};

let config = SipServerConfig {
    // UDP/TCP 配置
    udp_bind: "0.0.0.0:5060".parse().unwrap(),
    tcp_bind: "0.0.0.0:5060".parse().unwrap(),
    domain: "pbx.example.com".to_string(),
    enable_tcp: true,

    // TLS 配置
    enable_tls: true,
    tls_bind: "0.0.0.0:5061".parse().unwrap(),  // 标准 SIPS 端口
    tls_cert_path: "/path/to/server.crt".to_string(),
    tls_key_path: "/path/to/server.key".to_string(),
};

let sip_server = SipServer::new(config);
sip_server.start().await?;
```

#### 3. 客户端配置

SIP 客户端需要配置使用 SIPS URI：

```
sips:user@pbx.example.com:5061
```

而不是：
```
sip:user@pbx.example.com:5060
```

### TLS 功能特性

- ✅ **TLS 1.2 和 TLS 1.3** 支持
- ✅ **服务器端 TLS** - 接受加密的 SIP 连接
- ✅ **客户端 TLS** - 发送加密的 SIP 消息
- ✅ **自签名证书支持** - 适用于开发测试
- ✅ **灵活的证书验证** - 可配置严格模式或宽松模式

### TLS 最佳实践

1. **使用强密钥** - 至少 2048 位 RSA 或 256 位 ECC
2. **保护私钥** - 文件权限设为 600，仅 root/服务账户可读
3. **定期轮换证书** - 建议每年更换
4. **监控证书过期** - 设置提醒在过期前 30 天更新
5. **使用 CA 签名证书** - 生产环境避免自签名

---

## SRTP (媒体加密)

SRTP 加密 RTP 媒体流（语音/视频），保护通话内容隐私。

### SRTP 配置

#### 1. SDP 协商（SDES 方式）

YakYak 使用 SDES（Session Description Protocol Security Descriptions）在 SDP 中交换密钥。

**生成 SRTP 密钥并添加到 SDP：**

```rust
use yakyak::infrastructure::protocols::sip::SdpSession;
use yakyak::infrastructure::media::srtp::{SrtpMasterKey, SrtpProfile};

// 创建 SDP
let mut sdp = SdpSession::create_audio_session(local_ip, rtp_port);

// 生成 SRTP 密钥
let profile = SrtpProfile::Aes128CmHmacSha1_80;
let master_key = SrtpMasterKey::generate(profile);

// 添加 crypto 行到 SDP
sdp.add_srtp_crypto(&master_key, profile);

// SDP 现在包含 a=crypto: 行
let sdp_str = sdp.to_string();
println!("{}", sdp_str);
```

输出示例：
```
v=0
o=yakyak 1699564800 1 IN IP4 192.168.1.100
s=YakYak Call
c=IN IP4 192.168.1.100
t=0 0
m=audio 10000 RTP/SAVP 0 8 101
a=crypto:1 AES_CM_128_HMAC_SHA1_80 inline:WVNfX19zZW1jdGwgKCkgewkyMjA7fQp9CnVubGVz
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
a=rtpmap:101 telephone-event/8000
a=sendrecv
```

**从 SDP 解析 SRTP 密钥：**

```rust
// 解析接收到的 SDP
let sdp = SdpSession::parse(&sdp_str)?;

// 检查是否启用 SRTP
if sdp.is_srtp_enabled() {
    // 获取 SRTP 密钥
    let (master_key, profile) = sdp.get_srtp_crypto()
        .expect("SRTP enabled but no crypto found");

    println!("Remote uses SRTP with profile: {:?}", profile);
}
```

#### 2. MediaStream 集成

**启用 SRTP 加密：**

```rust
use yakyak::infrastructure::media::MediaStream;
use yakyak::infrastructure::media::srtp::{SrtpMasterKey, SrtpProfile};

// 创建媒体流
let stream = MediaStream::new(10000, 0, 8000).await?;

// 启用 SRTP
let profile = SrtpProfile::Aes128CmHmacSha1_80;
let master_key = SrtpMasterKey::generate(profile);
stream.enable_srtp(master_key, profile).await;

// 之后所有 send_rtp() 和接收的 RTP 都会自动加密/解密
stream.start().await?;
```

**完整的 INVITE 流程示例：**

```rust
// 发起方（Alice）
let mut offer_sdp = SdpSession::create_audio_session(alice_ip, alice_rtp_port);
let alice_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
offer_sdp.add_srtp_crypto(&alice_key, SrtpProfile::Aes128CmHmacSha1_80);

// 发送 INVITE with SDP offer
send_invite(offer_sdp.to_string());

// 接收方（Bob）收到 INVITE
let offer = SdpSession::parse(&invite_body)?;
if offer.is_srtp_enabled() {
    // Bob 也启用 SRTP
    let mut answer_sdp = SdpSession::create_audio_session(bob_ip, bob_rtp_port);
    let bob_key = SrtpMasterKey::generate(SrtpProfile::Aes128CmHmacSha1_80);
    answer_sdp.add_srtp_crypto(&bob_key, SrtpProfile::Aes128CmHmacSha1_80);

    // 获取 Alice 的密钥
    let (alice_key, profile) = offer.get_srtp_crypto().unwrap();

    // Bob 配置 MediaStream
    let bob_stream = MediaStream::new(bob_rtp_port, 0, 8000).await?;
    bob_stream.enable_srtp(bob_key.clone(), profile).await;  // Bob 发送用自己的密钥
    // 注意：接收需要用 Alice 的密钥（在实际实现中需要支持不对称密钥）

    // 发送 200 OK with SDP answer
    send_200_ok(answer_sdp.to_string());
}
```

### SRTP 加密套件

支持的加密套件（按优先级排序）：

| 套件 | 密钥长度 | 认证标签 | 说明 |
|------|---------|---------|-----|
| `AES_CM_128_HMAC_SHA1_80` | 128-bit | 80-bit | **推荐**，最常用 |
| `AES_CM_128_HMAC_SHA1_32` | 128-bit | 32-bit | 低带宽场景 |
| `AES_CM_256_HMAC_SHA1_80` | 256-bit | 80-bit | 高安全性 |
| `AES_CM_256_HMAC_SHA1_32` | 256-bit | 32-bit | 高安全+低带宽 |

**选择建议：**
- **默认**: `AES_CM_128_HMAC_SHA1_80` - 安全性和性能平衡
- **高安全**: `AES_CM_256_HMAC_SHA1_80` - 政府/金融场景
- **移动网络**: `AES_CM_128_HMAC_SHA1_32` - 节省带宽

### SRTP 功能特性

- ✅ **AES-CM 加密** - Counter Mode 加密
- ✅ **HMAC-SHA1 认证** - 消息完整性保护
- ✅ **重放保护** - 64 包滑动窗口
- ✅ **ROC 管理** - 自动序列号翻转
- ✅ **Per-SSRC 上下文** - 多流支持
- ✅ **SDES 密钥交换** - SDP a=crypto 行
- ✅ **自动加密/解密** - MediaStream 透明集成

### SRTP 最佳实践

1. **密钥轮换** - 长时间通话应定期重新协商密钥
2. **检查对端支持** - 优雅降级到 RTP（如果对端不支持）
3. **安全删除密钥** - 通话结束后从内存清除密钥
4. **日志安全** - 永远不要记录密钥材料
5. **结合 TLS** - TLS + SRTP 提供端到端保护

---

## 生产环境部署

### 端口配置

| 协议 | 默认端口 | 用途 |
|------|---------|-----|
| SIP (UDP/TCP) | 5060 | 未加密信令 |
| SIPS (TLS) | 5061 | 加密信令 |
| RTP | 10000-20000 | 未加密媒体 |
| SRTP | 10000-20000 | 加密媒体 |

### 防火墙规则

```bash
# 允许 SIP/SIPS
iptables -A INPUT -p tcp --dport 5060 -j ACCEPT
iptables -A INPUT -p udp --dport 5060 -j ACCEPT
iptables -A INPUT -p tcp --dport 5061 -j ACCEPT

# 允许 RTP/SRTP 范围
iptables -A INPUT -p udp --dport 10000:20000 -j ACCEPT
```

### 证书管理

#### Let's Encrypt 自动续期

```bash
# 安装 certbot
apt-get install certbot

# 获取证书
certbot certonly --standalone -d pbx.example.com

# 证书位置
/etc/letsencrypt/live/pbx.example.com/fullchain.pem  # 证书链
/etc/letsencrypt/live/pbx.example.com/privkey.pem    # 私钥

# 自动续期（crontab）
0 0 1 * * certbot renew --quiet && systemctl restart yakyak
```

#### 证书权限

```bash
chmod 600 /etc/yakyak/certs/server.key
chmod 644 /etc/yakyak/certs/server.crt
chown yakyak:yakyak /etc/yakyak/certs/*
```

### 环境变量配置

```bash
# .env
ENABLE_TLS=true
TLS_CERT_PATH=/etc/yakyak/certs/server.crt
TLS_KEY_PATH=/etc/yakyak/certs/server.key

ENABLE_SRTP=true
SRTP_DEFAULT_PROFILE=AES_CM_128_HMAC_SHA1_80
```

---

## 故障排除

### TLS 问题

#### 问题：TLS 握手失败

**症状**: `TLS handshake failed: certificate verify failed`

**解决方法**:
1. 检查证书是否过期：
   ```bash
   openssl x509 -in server.crt -noout -dates
   ```

2. 检查证书链完整性：
   ```bash
   openssl verify -CAfile ca.crt server.crt
   ```

3. 查看详细错误：
   ```bash
   openssl s_client -connect pbx.example.com:5061 -showcerts
   ```

#### 问题：权限被拒绝

**症状**: `Failed to open private key file: Permission denied`

**解决方法**:
```bash
sudo chmod 600 /path/to/server.key
sudo chown yakyak:yakyak /path/to/server.key
```

### SRTP 问题

#### 问题：SRTP 认证失败

**症状**: `SRTP decryption failed: Authentication failed`

**可能原因**:
1. 密钥不匹配 - 检查 SDP 协商
2. 数据包损坏 - 检查网络
3. 重放攻击被阻止 - 正常行为

**调试**:
```rust
// 临时禁用重放保护进行测试
srtp_context.disable_replay_protection();
```

#### 问题：无法协商 SRTP

**症状**: SDP 中没有 `a=crypto:` 行

**检查**:
1. 确认双方都支持 SRTP
2. 检查 SDP protocol 是否为 `RTP/SAVP`
3. 验证密钥生成：
   ```rust
   let key = SrtpMasterKey::generate(profile);
   println!("Key length: {}", key.key.len());  // 应为 16 (AES-128)
   println!("Salt length: {}", key.salt.len()); // 应为 14
   ```

### 日志调试

启用详细日志：

```bash
RUST_LOG=yakyak::infrastructure::protocols::sip::transport=debug,yakyak::infrastructure::media=debug cargo run
```

关键日志点：
- `TLS transport started on` - TLS 启动成功
- `Encrypted RTP packet with SRTP` - SRTP 加密
- `Decrypted RTP packet with SRTP` - SRTP 解密
- `TLS handshake failed` - TLS 错误

---

## 安全建议

### 高安全性场景

1. **强制加密**：
   ```rust
   // 拒绝非加密连接
   if !sdp.is_srtp_enabled() {
       return Err("SRTP required");
   }
   ```

2. **证书锁定**：
   ```rust
   // 只接受特定 CA 签发的证书
   config.set_trusted_ca_cert(ca_cert);
   ```

3. **完美前向保密** (Perfect Forward Secrecy):
   - 定期重新协商密钥
   - 使用短期会话密钥

### 合规性

- **PCI DSS**: 需要 TLS + SRTP
- **HIPAA**: 传输加密是必需的
- **GDPR**: 保护个人通话内容

---

## 性能影响

### TLS 开销

- **CPU**: +5-10% (握手时)
- **延迟**: +10-50ms (首次连接)
- **吞吐量**: 几乎无影响

### SRTP 开销

- **CPU**: +2-5% (持续)
- **延迟**: <1ms
- **带宽**: +4-10 bytes/packet (认证标签)

### 优化建议

1. **连接复用** - 重用 TLS 连接
2. **硬件加速** - 使用 AES-NI 指令
3. **批处理** - 批量处理 SRTP 包

---

## 参考资料

- [RFC 3261 - SIP](https://tools.ietf.org/html/rfc3261)
- [RFC 3711 - SRTP](https://tools.ietf.org/html/rfc3711)
- [RFC 4568 - SDP Security Descriptions (SDES)](https://tools.ietf.org/html/rfc4568)
- [RFC 5246 - TLS 1.2](https://tools.ietf.org/html/rfc5246)
- [RFC 8446 - TLS 1.3](https://tools.ietf.org/html/rfc8446)

---

## 联系支持

遇到问题？

- GitHub Issues: https://github.com/anthropics/yakyak/issues
- Email: support@yakyak.example.com
- 文档: https://docs.yakyak.example.com

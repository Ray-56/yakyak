# YakYak 基本呼叫流程

## 功能概览

YakYak 现在支持基本的 SIP 呼叫功能：

### 已实现的 SIP 方法
- ✅ **REGISTER** - 端点注册
- ✅ **INVITE** - 发起呼叫
- ✅ **ACK** - 确认呼叫建立
- ✅ **BYE** - 终止呼叫

### 基本呼叫流程

```
呼叫方 (Alice)         YakYak PBX         被叫方 (Bob)
     |                      |                    |
     |---- REGISTER ------->|                    |
     |<--- 200 OK ----------|                    |
     |                      |<---- REGISTER -----|
     |                      |---- 200 OK -------->|
     |                      |                    |
     |---- INVITE --------->|                    |
     |                      | (查找 Bob)         |
     |                      | (Bob 已注册)       |
     |<--- 200 OK ----------|                    |
     | (包含 SDP)           |                    |
     |                      |                    |
     |---- ACK ------------>|                    |
     |                      |                    |
     |   [通话进行中]       |                    |
     |                      |                    |
     |---- BYE ------------>|                    |
     |<--- 200 OK ----------|                    |
     |                      |                    |
```

## 测试呼叫功能

### 方式 1: 使用 SIP 客户端

推荐的 SIP 客户端：
- **Linphone** (跨平台，免费)
- **X-Lite** (Windows/Mac)
- **MicroSIP** (Windows，轻量级)
- **Blink** (Mac)

#### 配置步骤：

1. **启动 YakYak 服务器**
   ```bash
   cargo run
   ```

2. **配置 SIP 客户端 - 用户 Alice**
   - SIP Server: `localhost:5060`
   - 用户名: `alice`
   - 密码: (暂不需要，未启用认证)
   - 域: `localhost`
   - 传输协议: UDP

3. **配置 SIP 客户端 - 用户 Bob**
   - SIP Server: `localhost:5060`
   - 用户名: `bob`
   - 密码: (暂不需要)
   - 域: `localhost`
   - 传输协议: UDP

4. **注册两个客户端**
   - 两个客户端都应显示为"已注册"状态

5. **发起呼叫**
   - 从 Alice 呼叫 `bob@localhost`
   - YakYak 会自动接听（当前简化实现）
   - 发送 200 OK 响应（包含 SDP）

### 方式 2: 使用 SIPp 工具测试

SIPp 是专业的 SIP 测试工具：

```bash
# 安装 SIPp
# macOS: brew install sipp
# Linux: apt-get install sipp

# 测试 REGISTER
sipp -sn uac_register localhost:5060

# 测试 INVITE
sipp -sn uac localhost:5060
```

## 当前实现特性

### 1. SIP 注册 (REGISTER)
- ✅ 端点注册到 PBX
- ✅ 内存存储注册信息
- ✅ 支持过期时间 (Expires)
- ✅ 支持注销 (Expires: 0)

### 2. 呼叫发起 (INVITE)
- ✅ 接收 INVITE 请求
- ✅ 查找被叫方注册状态
- ✅ 返回 404 (未注册用户)
- ✅ 自动接听并返回 200 OK
- ✅ 生成 SDP 应答 (音频)

### 3. SDP 媒体协商
- ✅ 支持 PCMU (G.711 μ-law)
- ✅ 支持 PCMA (G.711 A-law)
- ✅ 支持 telephone-event (DTMF)
- ✅ RTP 端口: 10000

### 4. 呼叫确认 (ACK)
- ✅ 接收 ACK 确认
- ✅ 记录呼叫建立成功

### 5. 呼叫终止 (BYE)
- ✅ 接收 BYE 请求
- ✅ 清理呼叫会话
- ✅ 返回 200 OK

## 日志输出示例

```
INFO  Starting YakYak PBX System
INFO  Registered handlers: REGISTER, INVITE, ACK, BYE
INFO  UDP transport listening on 0.0.0.0:5060
INFO  TCP transport listening on 0.0.0.0:5060
INFO  SIP server started successfully

# 注册
INFO  Handling REGISTER request
INFO  Registered: sip:alice@localhost -> sip:alice@192.168.1.100:5060 (expires in 3600s)

# 呼叫
INFO  Handling INVITE request
DEBUG Call: sip:alice@localhost -> sip:bob@localhost
INFO  Auto-answering call (simplified)
INFO  Sent 200 OK for call abc123...

# 确认
INFO  Received ACK for call abc123...
INFO  Call abc123... confirmed: sip:alice@localhost -> sip:bob@localhost

# 终止
INFO  Received BYE for call abc123...
INFO  Call abc123... terminated: sip:alice@localhost -> sip:bob@localhost
```

## 当前限制和待开发功能

### 当前限制
- ⚠️ **自动接听**: 所有呼叫自动接听（未实现真实的呼叫转发）
- ⚠️ **无实际媒体流**: 只返回 SDP，不处理 RTP 媒体流
- ⚠️ **简化状态机**: 未实现完整的 SIP 事务状态机
- ⚠️ **无 NAT 穿透**: 未实现 STUN/TURN
- ⚠️ **内存密码存储**: 密码明文存储在内存中（仅用于开发）

### 已实现功能
- ✅ **SIP Digest 认证**
  - MD5 Digest 认证（RFC 2617）
  - Challenge/Response 机制
  - Nonce 管理和防重放
  - REGISTER 和 INVITE 认证支持
  - 详见 [AUTH.md](AUTH.md)

### 下一步开发优先级

1. **数据库用户管理** (高优先级)
   - 用户凭据持久化
   - 密码哈希存储
   - 动态用户管理 API

2. **真实呼叫转发** (高优先级)
   - 100 Trying
   - 180 Ringing
   - 转发 INVITE 到被叫方
   - 双向媒体桥接

3. **RTP 媒体处理** (中优先级)
   - RTP 接收和发送
   - 编解码器实现 (G.711)
   - 媒体混音（会议）

4. **高级呼叫功能** (中优先级)
   - CANCEL (取消呼叫)
   - 呼叫保持 (HOLD)
   - 呼叫转移 (REFER)
   - 呼叫转接 (Blind/Attended Transfer)

5. **NAT 穿透** (低优先级)
   - STUN 客户端
   - TURN 中继
   - ICE 协商

6. **WebRTC 支持** (低优先级)
   - WebSocket 信令
   - DTLS-SRTP
   - ICE 候选收集

## 测试建议

### 基础测试场景

1. **注册测试**
   ```
   - 注册单个用户
   - 注册多个用户
   - 注销用户
   - 过期自动清理
   ```

2. **呼叫测试**
   ```
   - Alice 呼叫 Bob (Bob 已注册)
   - Alice 呼叫 Charlie (Charlie 未注册 - 应返回 404)
   - 并发多路呼叫
   ```

3. **异常测试**
   ```
   - 发送格式错误的 SIP 消息
   - 超时场景
   - 连接断开
   ```

## 性能指标

当前性能（开发模式）：
- **并发呼叫**: 支持
- **注册容量**: 内存限制
- **消息处理**: 异步（Tokio）
- **响应延迟**: < 10ms

## 与 FreeSWITCH 对比

| 功能 | FreeSWITCH | YakYak (当前) | 状态 |
|------|-----------|--------------|------|
| SIP 注册 | ✅ | ✅ | ✅ 完成 |
| 基本呼叫 | ✅ | ✅ (简化) | 🔄 进行中 |
| SIP 认证 | ✅ | ✅ (Digest) | ✅ 完成 |
| RTP 媒体 | ✅ | ❌ | 📋 计划中 |
| 编解码器 | 多种 | SDP only | 📋 计划中 |
| 呼叫转移 | ✅ | ❌ | 📋 计划中 |
| 会议 | ✅ | ❌ | 📋 计划中 |
| IVR | ✅ | ❌ | ⏸️ 未计划 |
| WebRTC | ✅ | ❌ | 📋 计划中 |
| 内存安全 | ❌ | ✅ | 🎯 优势 |
| 异步架构 | 部分 | ✅ | 🎯 优势 |
| DDD 设计 | ❌ | ✅ | 🎯 优势 |

## 贡献

欢迎贡献代码！优先级开发任务：
1. ~~SIP Digest 认证~~ ✅ 已完成
2. 数据库用户管理
3. RTP 媒体处理
4. 呼叫转发优化
5. 更多测试用例

## 问题反馈

如遇到问题，请提供：
1. SIP 消息抓包 (wireshark/tcpdump)
2. YakYak 日志输出
3. 复现步骤

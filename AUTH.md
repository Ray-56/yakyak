# YakYak SIP 认证系统

## 概览

YakYak 现在支持 SIP Digest 认证（RFC 2617 和 RFC 3261），提供安全的用户认证机制。

## 认证架构

### 支持的认证方法
- ✅ **Digest Authentication** (RFC 2617)
  - MD5 哈希算法
  - Challenge/Response 机制
  - Nonce 防重放攻击
  - QoP (Quality of Protection) 支持

### 认证流程

```
客户端                    YakYak PBX
  |                           |
  |---- REGISTER ------------>|
  |     (无认证)              |
  |                           |
  |<--- 401 Unauthorized -----|
  |     WWW-Authenticate      |
  |     (包含 challenge)      |
  |                           |
  |---- REGISTER ------------>|
  |     Authorization         |
  |     (包含 response)       |
  |                           |
  |     [验证凭据]            |
  |                           |
  |<--- 200 OK ---------------|
  |     (认证成功)            |
```

## 认证机制详解

### 1. Challenge 生成

当收到未认证的请求时，服务器生成一个 challenge：

```
WWW-Authenticate: Digest realm="localhost", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", algorithm=MD5, qop="auth"
```

参数说明：
- **realm**: 认证域（通常是 SIP 域名）
- **nonce**: 随机生成的唯一值，有效期 5 分钟
- **algorithm**: 哈希算法（MD5）
- **qop**: 保护质量（"auth" 表示认证）

### 2. Response 计算

客户端需要计算 response 值：

```
HA1 = MD5(username:realm:password)
HA2 = MD5(method:uri)

# 如果有 qop
response = MD5(HA1:nonce:nc:cnonce:qop:HA2)

# 如果没有 qop
response = MD5(HA1:nonce:HA2)
```

### 3. 认证验证

服务器接收到认证请求后：
1. 验证 nonce 是否有效且未过期
2. 查找用户凭据
3. 使用相同算法计算期望的 response
4. 比对客户端提供的 response
5. 验证成功则处理请求，失败则返回 401

## 配置和使用

### 添加用户

在 `main.rs` 中添加用户：

```rust
// 初始化认证系统
let auth = Arc::new(DigestAuth::new(&config.sip.domain));

// 添加用户
auth.add_user("alice", "secret123").await;
auth.add_user("bob", "secret456").await;
```

### 测试用户

当前系统包含以下测试用户：

| 用户名 | 密码 | 说明 |
|--------|------|------|
| alice  | secret123 | 测试用户 1 |
| bob    | secret456 | 测试用户 2 |

### SIP 客户端配置

#### Linphone 配置示例

1. **创建账号**
   - 用户名: `alice`
   - 密码: `secret123`
   - 域: `localhost`
   - 传输: UDP
   - 启用注册: 是

2. **高级设置**
   - 外呼路由: `sip:localhost:5060`
   - 认证用户名: `alice`

#### MicroSIP 配置示例

```
Account name: Alice
SIP server: localhost:5060
Username: alice
Domain: localhost
Login: alice
Password: secret123
```

## 认证安全特性

### 1. Nonce 管理
- ✅ 随机生成的 nonce（16 字节）
- ✅ Nonce 有效期：5 分钟
- ✅ 自动清理过期的 nonce
- ✅ 防止重放攻击

### 2. 密码存储
- ⚠️ **当前**: 内存中明文存储（仅用于开发测试）
- 🔜 **计划**: 数据库中哈希存储

### 3. 支持的 SIP 方法
- ✅ **REGISTER**: 401 Unauthorized + WWW-Authenticate
- ✅ **INVITE**: 407 Proxy Authentication Required + Proxy-Authenticate

## API 使用

### DigestAuth

```rust
use yakyak::infrastructure::protocols::sip::DigestAuth;

// 创建认证管理器
let auth = DigestAuth::new("example.com");

// 添加用户
auth.add_user("username", "password").await;

// 生成 challenge
let challenge = auth.create_challenge().await;
let header_value = challenge.to_header_value();

// 验证请求
match auth.verify_request(&request, "REGISTER").await {
    Ok(username) => {
        println!("认证成功: {}", username);
    }
    Err(e) => {
        println!("认证失败: {:?}", e);
    }
}

// 清理过期 nonce
auth.cleanup_nonces().await;
```

### 集成到 Handler

#### REGISTER Handler

```rust
let registrar = Registrar::with_auth(auth.clone());
```

#### INVITE Handler

```rust
let invite_handler = InviteHandler::with_auth(
    registrar.clone(),
    local_ip,
    auth.clone(),
);
```

## 测试认证

### 使用 SIPp 测试

```bash
# 测试 REGISTER 认证
sipp -sf register_auth.xml localhost:5060

# 测试 INVITE 认证
sipp -sf invite_auth.xml localhost:5060
```

### 使用 Wireshark 抓包

1. 捕获 SIP 流量：
   ```bash
   wireshark -i lo0 -f "port 5060"
   ```

2. 查看认证流程：
   - 第一个 REGISTER 请求（无认证）
   - 401 响应（包含 WWW-Authenticate）
   - 第二个 REGISTER 请求（包含 Authorization）
   - 200 OK（认证成功）

## 日志输出

启用认证后的日志示例：

```
INFO  Added user: alice
INFO  Added user: bob
INFO  Added test users: alice, bob
INFO  Registered handlers: REGISTER, INVITE, ACK, BYE

# 认证流程
WARN  REGISTER without authentication - sending challenge
DEBUG Created auth challenge with nonce: dcd98b7102dd2f0e8b11d0f600bfb0c093
DEBUG Parsing Authorization header: Digest username="alice"...
DEBUG Calculated response for user alice: abc123...
INFO  REGISTER authenticated for user: alice
INFO  Registered: sip:alice@localhost -> sip:alice@192.168.1.100:5060
```

## 安全建议

### 开发环境
- ✅ 使用测试用户和简单密码
- ✅ Nonce 有效期较短（5 分钟）
- ⚠️ 明文密码存储（仅用于测试）

### 生产环境建议
- 🔐 使用强密码策略
- 🔐 密码哈希存储（bcrypt/argon2）
- 🔐 启用 TLS/DTLS 传输加密
- 🔐 实施速率限制防止暴力破解
- 🔐 日志记录认证失败事件
- 🔐 定期轮换 nonce
- 🔐 考虑使用 SIP Identity (RFC 8224)

## 当前限制

### 已知限制
- ⚠️ **密码存储**: 内存中明文存储
- ⚠️ **算法支持**: 仅支持 MD5（RFC 3261 要求）
- ⚠️ **用户管理**: 静态配置，无动态管理 API
- ⚠️ **会话管理**: 无会话超时机制

### 未来改进

1. **数据库集成** (高优先级)
   - 用户凭据持久化存储
   - 哈希密码存储（bcrypt）
   - 用户权限管理

2. **增强安全性** (高优先级)
   - TLS/SRTP 支持
   - SHA-256/SHA-512-256 算法支持（RFC 8760）
   - 速率限制和防暴力破解

3. **用户管理** (中优先级)
   - RESTful API 管理用户
   - 用户组和权限
   - 动态加载用户配置

4. **高级功能** (低优先级)
   - SIP Identity (RFC 8224)
   - OAuth 2.0 集成
   - 多因素认证 (MFA)

## 故障排除

### 常见问题

#### 1. 认证总是失败

检查：
- 用户名和密码是否正确
- realm 是否匹配（应该是 SIP 域名）
- 客户端是否正确计算 response

#### 2. Nonce 过期

如果认证过程超过 5 分钟：
```
Authentication failed: Invalid or expired nonce
```

解决：重新发起注册

#### 3. 密码中包含特殊字符

确保密码正确 URL 编码。

## 性能考虑

- **MD5 计算**: 每次认证需要 3 次 MD5 哈希
- **Nonce 存储**: 内存中使用 HashMap，自动清理
- **并发认证**: 使用 RwLock 保证线程安全
- **性能指标**: 单次认证 < 1ms

## 参考文档

- [RFC 2617 - HTTP Authentication: Basic and Digest Access Authentication](https://tools.ietf.org/html/rfc2617)
- [RFC 3261 - SIP: Session Initiation Protocol (Section 22)](https://tools.ietf.org/html/rfc3261#section-22)
- [RFC 8760 - The Digest-Response "auth-int" Parameter](https://tools.ietf.org/html/rfc8760)

## 贡献

欢迎贡献改进认证系统：
1. 实现 SHA-256 算法支持
2. 添加数据库用户管理
3. 实现 TLS/DTLS 传输
4. 添加更多测试用例

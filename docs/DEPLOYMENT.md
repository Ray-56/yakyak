# YakYak Deployment Guide

Complete guide for deploying YakYak PBX system in production environments.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Database Setup](#database-setup)
5. [Security](#security)
6. [Running](#running)
7. [Monitoring](#monitoring)
8. [Maintenance](#maintenance)
9. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Hardware Requirements

**Minimum:**
- CPU: 2 cores
- RAM: 4 GB
- Disk: 20 GB
- Network: 100 Mbps

**Recommended:**
- CPU: 4+ cores
- RAM: 8+ GB
- Disk: 100+ GB (SSD recommended)
- Network: 1 Gbps

### Software Requirements

- **Operating System**: Linux (Ubuntu 20.04+, Debian 11+, CentOS 8+)
- **Rust**: 1.65 or later
- **PostgreSQL**: 13 or later
- **Optional**: Docker, Nginx (for reverse proxy)

### Network Requirements

**Required Ports:**
- `5060/UDP`: SIP signaling
- `5060/TCP`: SIP signaling (optional)
- `10000-20000/UDP`: RTP media (configurable range)
- `8080/TCP`: REST API and WebSocket

**Firewall Rules:**
```bash
# SIP
sudo ufw allow 5060/udp
sudo ufw allow 5060/tcp

# RTP
sudo ufw allow 10000:20000/udp

# API
sudo ufw allow 8080/tcp
```

---

## Installation

### Method 1: Build from Source

```bash
# Clone repository
git clone https://github.com/Ray-56/yakyak.git
cd yakyak

# Build release binary
cargo build --release --features postgres

# Install binary
sudo cp target/release/yakyak /usr/local/bin/
sudo chmod +x /usr/local/bin/yakyak
```

### Method 2: Docker (Future)

```bash
# Pull image
docker pull yakyak/yakyak:latest

# Run container
docker run -d \
  --name yakyak \
  -p 5060:5060/udp \
  -p 10000-20000:10000-20000/udp \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@localhost/yakyak \
  yakyak/yakyak:latest
```

---

## Configuration

### Configuration File

Create `/etc/yakyak/config.toml`:

```toml
[server]
listen_address = "0.0.0.0:5060"
transport = "udp"  # or "tcp"
realm = "example.com"
local_ip = "192.168.1.100"  # Server's IP address

[api]
listen_address = "0.0.0.0:8080"
enable_cors = true

[database]
url = "postgres://yakyak:password@localhost/yakyak"
max_connections = 10
min_connections = 2
connect_timeout = 30  # seconds
idle_timeout = 600  # seconds
max_lifetime = 1800  # seconds

[rtp]
port_range_start = 10000
port_range_end = 20000
jitter_buffer_min_delay = 20  # ms
jitter_buffer_max_delay = 200  # ms

[security]
# Brute force protection
max_auth_attempts = 5
lockout_duration = 900  # 15 minutes in seconds
auth_failure_window = 300  # 5 minutes

# Rate limiting
rate_limit_requests = 10
rate_limit_window = 60  # 1 minute

# Supported digest algorithms
digest_algorithms = ["MD5", "SHA-256"]  # SHA-512 also available

[ivr]
audio_path = "/var/lib/yakyak/audio"
default_language = "en"

[voicemail]
storage_path = "/var/lib/yakyak/voicemail"
max_message_duration = 180  # 3 minutes
max_messages_per_mailbox = 100

[conference]
max_participants = 50
recording_path = "/var/lib/yakyak/recordings"

[stun]
enabled = true
server = "stun.l.google.com:19302"
keepalive_interval = 30  # seconds

[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # or "pretty"
output = "file"  # or "stdout"
file_path = "/var/log/yakyak/yakyak.log"
rotation = "daily"  # or "size"
max_size = "100MB"
max_files = 10
```

### Environment Variables

```bash
# Database
export DATABASE_URL="postgres://yakyak:password@localhost/yakyak"

# Server
export YAKYAK_LISTEN_ADDRESS="0.0.0.0:5060"
export YAKYAK_REALM="example.com"
export YAKYAK_LOCAL_IP="192.168.1.100"

# Logging
export RUST_LOG="yakyak=info"
```

---

## Database Setup

### 1. Install PostgreSQL

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### 2. Create Database and User

```bash
sudo -u postgres psql

# In PostgreSQL shell:
CREATE DATABASE yakyak;
CREATE USER yakyak WITH ENCRYPTED PASSWORD 'your_password_here';
GRANT ALL PRIVILEGES ON DATABASE yakyak TO yakyak;
\q
```

### 3. Run Migrations

```bash
# Using sqlx-cli (install if not present)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
export DATABASE_URL="postgres://yakyak:password@localhost/yakyak"
sqlx database create
sqlx migrate run
```

### 4. Create Initial Admin User

```bash
# Connect to database
psql -U yakyak -d yakyak

# Insert admin user (password: admin123)
INSERT INTO users (username, password_hash, sip_ha1, realm, display_name, role_id, enabled)
VALUES (
    'admin',
    '$2b$12$...',  -- bcrypt hash of 'admin123'
    'e10adc3949ba59abbe56e057f20f883e',  -- MD5(admin:example.com:admin123)
    'example.com',
    'Administrator',
    'a0000000-0000-0000-0000-000000000001',  -- administrator role
    true
);
```

---

## Security

### 1. SSL/TLS for SIP (SIP over TLS)

```toml
[server]
transport = "tls"
tls_cert = "/etc/yakyak/ssl/cert.pem"
tls_key = "/etc/yakyak/ssl/key.pem"
```

Generate self-signed certificate (for testing):

```bash
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem -out cert.pem \
  -days 365 -nodes \
  -subj "/CN=example.com"
```

### 2. API Authentication (Future)

```toml
[api]
enable_authentication = true
jwt_secret = "your-secret-key-here"
jwt_expiration = 3600  # 1 hour
```

### 3. Firewall Configuration

```bash
# Allow only necessary ports
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 5060/udp
sudo ufw allow 10000:20000/udp
sudo ufw allow 8080/tcp
sudo ufw enable
```

### 4. Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## Running

### Systemd Service

Create `/etc/systemd/system/yakyak.service`:

```ini
[Unit]
Description=YakYak PBX Server
After=network.target postgresql.service

[Service]
Type=simple
User=yakyak
Group=yakyak
WorkingDirectory=/opt/yakyak
ExecStart=/usr/local/bin/yakyak --config /etc/yakyak/config.toml
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/yakyak /var/log/yakyak

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

Create service user:

```bash
sudo useradd -r -s /bin/false yakyak
sudo mkdir -p /var/lib/yakyak /var/log/yakyak
sudo chown yakyak:yakyak /var/lib/yakyak /var/log/yakyak
```

Enable and start service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable yakyak
sudo systemctl start yakyak
```

Check status:

```bash
sudo systemctl status yakyak
sudo journalctl -u yakyak -f
```

---

## Monitoring

### 1. Prometheus Metrics

Configure Prometheus to scrape metrics:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'yakyak'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### 2. Health Checks

```bash
# Check API health
curl http://localhost:8080/health

# Check system health
curl http://localhost:8080/api/monitoring/health
```

### 3. Log Monitoring

```bash
# Follow logs
sudo journalctl -u yakyak -f

# View recent logs
sudo journalctl -u yakyak -n 100

# View logs with errors
sudo journalctl -u yakyak -p err
```

### 4. Database Monitoring

```sql
-- Active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'yakyak';

-- Table sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

---

## Maintenance

### Database Backup

```bash
# Manual backup
pg_dump -U yakyak yakyak > yakyak_backup_$(date +%Y%m%d).sql

# Automated daily backup (cron)
0 2 * * * pg_dump -U yakyak yakyak | gzip > /backup/yakyak_$(date +\%Y\%m\%d).sql.gz
```

### Database Restore

```bash
# Restore from backup
psql -U yakyak yakyak < yakyak_backup_20251106.sql
```

### Log Rotation

Create `/etc/logrotate.d/yakyak`:

```
/var/log/yakyak/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 yakyak yakyak
    sharedscripts
    postrotate
        systemctl reload yakyak > /dev/null 2>&1 || true
    endscript
}
```

### CDR Cleanup

```bash
# Delete CDRs older than 1 year
psql -U yakyak -d yakyak -c "DELETE FROM call_records WHERE created_at < NOW() - INTERVAL '1 year';"
```

---

## Troubleshooting

### SIP Registration Fails

**Check:**
1. Firewall allows UDP 5060
2. Correct realm configured
3. User exists and is enabled
4. Password/HA1 is correct

```bash
# Test SIP connectivity
nmap -sU -p 5060 your-server-ip

# Check logs
sudo journalctl -u yakyak | grep -i "register"
```

### No Audio in Calls

**Check:**
1. RTP ports (10000-20000) are open
2. NAT/firewall allows UDP traffic
3. Correct local_ip configured
4. STUN server accessible

```bash
# Test RTP ports
nmap -sU -p 10000-10010 your-server-ip
```

### High CPU Usage

**Check:**
1. Number of active calls
2. Conference rooms with many participants
3. Database query performance

```bash
# Check active calls
curl http://localhost:8080/api/calls | jq '.total'

# Database connections
psql -U yakyak -d yakyak -c "SELECT count(*) FROM pg_stat_activity;"
```

### Database Connection Errors

**Check:**
1. PostgreSQL is running
2. Database credentials are correct
3. Connection pool settings

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql -U yakyak -d yakyak -c "SELECT version();"
```

---

## Performance Tuning

### PostgreSQL

Edit `/etc/postgresql/13/main/postgresql.conf`:

```ini
# Memory
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB

# Connections
max_connections = 200

# WAL
wal_buffers = 16MB
checkpoint_completion_target = 0.9
```

### System Limits

Edit `/etc/security/limits.conf`:

```
yakyak soft nofile 65536
yakyak hard nofile 65536
yakyak soft nproc 4096
yakyak hard nproc 4096
```

---

## See Also

- [Architecture Documentation](ARCHITECTURE.md)
- [API Documentation](API.md)
- [Database Schema](DATABASE_SCHEMA.md)
- [Security Best Practices](SECURITY.md)

---

## Support

For issues and support:
- GitHub Issues: https://github.com/Ray-56/yakyak/issues
- Documentation: https://docs.yakyak.io
- Community: https://community.yakyak.io

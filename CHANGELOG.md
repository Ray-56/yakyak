# Changelog

All notable changes to YakYak will be documented in this file.

## [Unreleased] - 2025-11-06

### Added

#### Phase 1.1 - User Management Enhancements
- **Role-Based Access Control (RBAC)**
  - `Role` entity with 20 distinct permissions
  - `Permission` enum for fine-grained access control
  - `RoleRepository` trait and PostgreSQL implementation
  - Three default system roles: administrator, user, operator
  - Database migration for roles table with UUID primary key
  - Users can now be assigned roles via `role_id` field
  - System roles are protected from deletion

- **Bulk User Import**
  - CSV import endpoint (`POST /users/import/csv`)
  - JSON import endpoint (`POST /users/import/json`)
  - Detailed import results with error reporting
  - Support for optional fields (display_name, email)

- **Documentation**
  - Comprehensive database schema documentation (`docs/DATABASE_SCHEMA.md`)
  - Complete REST API documentation (`docs/API.md`)
  - Migration history and maintenance guidelines

#### Phase 2.2 - Call Hold and Transfer (COMPLETED)
- **REFER Handler**
  - Basic REFER request handling for blind call transfer
  - Refer-To and Referred-By header extraction
  - Call existence validation
  - 202 Accepted response for valid REFER requests
  - Unit tests for REFER functionality

- **Hold Manager**
  - Call hold/resume state management
  - HoldState enum (Active, LocalHold, RemoteHold, BothHold)
  - SDP manipulation for hold/resume (sendonly, recvonly, inactive)
  - Hold state detection from SDP
  - Remote hold/resume tracking
  - Unit tests (8 tests)

#### Phase 2.3 - Authentication Security Enhancements (COMPLETED)
- **Enhanced Digest Authentication**
  - SHA-256 and SHA-512 algorithm support (in addition to MD5)
  - DigestAlgorithm enum with string conversion
  - Enhanced HA1/HA2 calculation for all algorithms
  - QoP (Quality of Protection) support
  - Unit tests for all algorithms

- **Brute Force Protection**
  - IP-based lockout after N failed attempts
  - Configurable lockout duration and time window
  - Automatic cleanup of expired entries
  - Default: 5 attempts, 15-minute lockout, 5-minute window
  - Unit tests for protection logic

- **Rate Limiting**
  - Per-IP request rate limiting
  - Configurable max requests and time window
  - Automatic cleanup of old request history
  - Default: 10 requests per minute
  - Unit tests

#### Phase 3.4 - Event Subscription (SUBSCRIBE/NOTIFY)
- **SUBSCRIBE Handler**
  - Event subscription handling (presence, dialog, message-summary, reg, refer)
  - Subscription manager with dialog ID tracking
  - Expires header support (unsubscribe with Expires: 0)
  - 489 Bad Event response for unsupported event packages
  - 202 Accepted response for valid subscriptions
  - Unit tests for subscription workflows

- **NOTIFY Handler**
  - Event notification handling
  - Support for refer, message-summary, presence, dialog, reg events
  - Event and Subscription-State header extraction
  - SIP fragment body parsing (for REFER progress)
  - 200 OK response for valid notifications
  - Unit tests for various event types

- **Subscription Management**
  - `SubscriptionManager` for tracking active subscriptions
  - Dialog ID generation from Call-ID and tags
  - Subscription expiration handling
  - In-memory subscription storage

#### Phase 3.5 - Instant Messaging (MESSAGE)
- **MESSAGE Handler**
  - SIP MESSAGE request handling
  - Text message routing and delivery
  - Content-Type header support
  - From/To URI extraction and parsing

- **Message Store**
  - In-memory message storage
  - Offline message queuing
  - Message delivery tracking
  - Undelivered message retrieval
  - Message history with timestamps

- **Features**
  - Online/offline user detection via Registrar
  - Automatic message queuing for offline users
  - Delivery confirmation
  - Message metadata (from, to, content_type, timestamp)
  - Unit tests for message delivery scenarios

#### Phase 3.6 - Voicemail System (COMPLETED)
- **Voicemail Domain Model**
  - VoicemailMessage entity with status tracking
  - VoicemailStatus enum (New, Read, Saved, Deleted)
  - VoicemailMailbox configuration entity
  - PIN-based mailbox access control
  - Configurable max duration and message limits
  - Email notification support

- **Voicemail Repository**
  - VoicemailRepository trait for persistence
  - CRUD operations for messages and mailboxes
  - Status-based filtering
  - Message counting per mailbox
  - VoicemailFilters for advanced queries

- **Features**
  - Message mark as read/saved/deleted
  - Mailbox configuration (PIN, greeting, limits)
  - Email notification settings
  - Unit tests (7 tests)

#### Phase 3.7 - IVR System (COMPLETED)
- **DTMF Detection**
  - DtmfDigit enum with 12 digits (0-9, *, #)
  - DtmfEvent with duration and timestamp
  - RFC 2833 payload parsing
  - SIP INFO DTMF parsing (application/dtmf-relay)
  - DTMF frequency mapping
  - Unit tests (7 tests)

- **DTMF Detector**
  - Buffer-based digit collection
  - Configurable timeout and buffer size
  - Pattern matching (matches, ends_with)
  - Last N digits retrieval
  - Automatic buffer clearing on timeout
  - Unit tests (4 tests)

- **IVR Menu System**
  - IvrMenu configuration with greeting and items
  - IvrMenuItem with digit-action mapping
  - MenuAction enum (PlayAudio, Transfer, GotoMenu, etc.)
  - IvrMenuBuilder for fluent construction
  - IvrMenuSystem for menu management
  - Default main menu template
  - JSON serialization/deserialization
  - Unit tests (6 tests)

- **IVR Flow Engine**
  - IvrSession with state machine
  - IvrState enum (8 states)
  - IvrFlowEngine for session management
  - DTMF event processing
  - Menu navigation with stack (GoBack support)
  - Session variables
  - Timeout and retry handling
  - Invalid input handling
  - Unit tests (6 tests)

- **Menu Actions**
  - Play audio files
  - Transfer to extensions/URIs
  - Navigate between menus
  - Dial by extension
  - Voicemail access
  - Repeat/Go back/Hangup

#### Phase 2.5 - Enhanced Monitoring (COMPLETED)
- **System Health Monitoring**
  - SystemHealth with overall status (healthy/degraded/unhealthy)
  - SystemMetrics (uptime, memory, active calls)
  - CallMetrics (total calls, active calls, call quality stats)
  - AuthMetrics (registration success/failure rates)
  - MediaMetrics (RTP packets, jitter, packet loss)
  - DatabaseMetrics (connection pool status)
  - Warnings and errors tracking
  - Health assessment logic

- **Metrics Collector**
  - MetricsCollector with centralized metrics management
  - Thread-safe metrics updates via RwLock
  - Start time tracking for uptime calculation
  - Call recording (start, end, quality metrics)
  - Authentication event recording
  - Media statistics updates
  - Database metrics updates
  - System snapshot generation

- **Monitoring API**
  - GET /health - Basic health check
  - GET /api/monitoring/health - Detailed system health
  - JSON serialization for all metrics types

#### Phase 3.1 - Conference System (COMPLETED)
- **Conference Domain Model**
  - ConferenceRoom entity with UUID
  - ConferenceState enum (Waiting, Active, Locked, Ended)
  - Participant entity with role (Moderator, Presenter, Attendee, Listener)
  - ParticipantState enum (Connecting, Active, OnHold, Muted, Disconnected)
  - Moderator controls and permissions
  - PIN-based room access
  - Recording support
  - Unit tests (13 tests)

- **Conference Room Management**
  - Room creation with configuration
  - Participant add/remove
  - Participant mute/unmute
  - Participant role changes
  - Moderator management
  - Room lock/unlock
  - Room lifecycle (start, end)
  - Max participant limits

- **Conference Repository**
  - ConferenceRepository trait for persistence
  - CRUD operations for rooms
  - Participant management
  - Room search and filtering
  - User-based room queries

- **Audio Mixer**
  - AudioMixer for multi-participant mixing
  - AudioFrame with sample data and timestamp
  - ParticipantStream with mute/volume control
  - Multi-stream audio mixing algorithm
  - Participant exclusion (avoid echo)
  - Sample rate configuration (8000 Hz default)
  - Unit tests (8 tests)

- **Automatic Gain Control (AGC)**
  - AutomaticGainControl for volume normalization
  - Adaptive gain adjustment
  - Target level configuration
  - Attack/release coefficient control
  - Frame-level gain application

#### Phase 3.2 - NAT Traversal (STUN) (COMPLETED)
- **STUN Protocol Implementation (RFC 5389)**
  - StunMessageType enum (BindingRequest, BindingResponse, BindingErrorResponse)
  - StunMessage parsing and serialization
  - StunAttribute support (MappedAddress, XorMappedAddress, ErrorCode, etc.)
  - Transaction ID generation and validation
  - Magic cookie validation (0x2112A442)
  - Unit tests (5 tests)

- **STUN Client**
  - StunClient for NAT binding discovery
  - Configurable STUN server and timeout
  - UDP socket management
  - Binding request/response handling
  - Public IP and port discovery
  - Error handling for timeouts and invalid responses
  - Unit tests (3 tests)

- **NAT Type Detection**
  - NatType enum (OpenInternet, FullCone, RestrictedCone, PortRestrictedCone, Symmetric, Unknown)
  - Multi-server NAT type detection algorithm
  - Primary and secondary server queries
  - Port consistency checking
  - External IP comparison

- **XOR Address Mapping**
  - XOR-MAPPED-ADDRESS attribute support
  - Transaction ID-based XOR operation
  - IPv4 address encoding/decoding
  - Magic cookie XOR for obfuscation

#### Documentation
- **Deployment Guide** (`docs/DEPLOYMENT.md`)
  - System requirements (hardware, software, network)
  - Installation methods (source build, Docker)
  - Complete configuration guide (TOML format)
  - Database setup (PostgreSQL installation, migrations)
  - Security configuration (TLS, firewall, reverse proxy)
  - Systemd service configuration
  - Monitoring setup (Prometheus, health checks, logs)
  - Maintenance procedures (backup, restore, log rotation)
  - Troubleshooting guide
  - Performance tuning recommendations

### Changed

#### Database Schema
- **users table**
  - Added `role_id UUID` column (foreign key to roles)
  - New index on `role_id` for performance

- **roles table** (new)
  - UUID primary key
  - String-based permission array
  - System role protection flag
  - Automatic timestamp updates

#### Domain Models
- **User entity**
  - Added `role_id` field
  - Updated `CreateUser` with optional role assignment
  - Updated `UpdateUser` to support role changes

- **New Repositories**
  - `RoleRepository` trait with CRUD operations
  - `PgRoleRepository` PostgreSQL implementation

#### API Endpoints
- **User Management**
  - User creation now accepts `role_id` parameter
  - User updates support role assignment
  - New bulk import endpoints (CSV and JSON)

- **Role Management** (planned endpoints)
  - GET /roles - List all roles
  - GET /roles/:id - Get role by ID
  - POST /roles - Create custom role
  - PUT /roles/:id - Update role
  - DELETE /roles/:id - Delete role

### Technical Details

#### Permissions
New permission strings in format `resource:action`:
- User: `user:read`, `user:create`, `user:update`, `user:delete`, `user:manage_roles`
- Call: `call:read`, `call:create`, `call:terminate`, `call:transfer`
- CDR: `cdr:read`, `cdr:export`, `cdr:delete`
- System: `system:config`, `system:monitor`, `system:audit`
- Conference: `conference:create`, `conference:manage`, `conference:moderate`
- Voicemail: `voicemail:access`, `voicemail:manage`

#### Default Roles
1. **Administrator** (UUID: a0000000-0000-0000-0000-000000000001)
   - All 20 permissions
   - System role (cannot be deleted)

2. **User** (UUID: a0000000-0000-0000-0000-000000000002)
   - call:create, call:read, voicemail:access
   - System role

3. **Operator** (UUID: a0000000-0000-0000-0000-000000000003)
   - call:create, call:read, call:transfer, call:terminate, user:read, cdr:read
   - System role

#### Database Migrations
- `20251106_01_create_roles_table.sql`
  - Creates roles table
  - Adds role_id to users table
  - Inserts 3 default system roles
  - Creates indexes and triggers

#### Testing
- Role management unit tests (6 tests)
- Role repository integration tests (5 tests)
- REFER handler tests (2 tests)
- SUBSCRIBE handler tests (4 tests)
- NOTIFY handler tests (2 tests)
- MESSAGE handler tests (3 tests)
- User import unit tests (2 tests)
- Hold manager tests (8 tests)
- Enhanced auth tests (5 tests)
- Voicemail domain tests (7 tests)
- DTMF detection tests (7 tests)
- DTMF detector tests (4 tests)
- IVR menu tests (6 tests)
- IVR flow engine tests (6 tests)
- Conference domain tests (13 tests)
- Audio mixer tests (8 tests)
- STUN message tests (5 tests)
- STUN client tests (3 tests)
- **Total new tests: 96**

### TODO / In Progress

#### Phase 2.1 - TLS/SRTP Encryption
- [ ] TLS transport layer
- [ ] Certificate management
- [ ] SRTP media encryption
- [ ] DTLS-SRTP for WebRTC

#### Phase 2.2 - Call Transfer (Remaining)
- [x] Basic REFER handler (completed)
- [x] Call hold/resume state management (completed)
- [ ] Complete REFER/NOTIFY integration
- [ ] Attended transfer support
- [ ] Music on hold (MOH)

#### Phase 2.3 - Authentication Security
- [x] SHA-256/SHA-512 support (completed)
- [x] Brute force protection (completed)
- [x] Rate limiting (completed)
- [ ] IP blacklisting (basic framework in place)
- [ ] Audit logging

#### Phase 2.5 - Monitoring Enhancements
- [x] System health monitoring (completed)
- [x] Extended metrics (completed)
- [x] Metrics collector (completed)
- [ ] API authentication/authorization
- [ ] Performance profiling
- [ ] Grafana dashboards

#### Phase 3.1 - Conference Features
- [x] Conference room management (completed)
- [x] Audio mixing (completed)
- [x] Participant controls (completed)
- [x] Conference domain model (completed)
- [ ] Conference recording implementation
- [ ] PostgreSQL repository implementation
- [ ] Conference API endpoints
- [ ] Music on hold for conferences

#### Phase 3.2 - NAT Traversal
- [x] STUN client (completed)
- [x] STUN protocol implementation (completed)
- [x] NAT type detection (completed)
- [ ] TURN relay
- [ ] ICE support
- [ ] STUN server implementation

#### Phase 3.3 - WebRTC Integration
- [ ] WebSocket signaling
- [ ] WebRTC SDP support
- [ ] Browser compatibility

#### Phase 3.6 - Voicemail
- [x] Voicemail domain model (completed)
- [x] Voicemail repository trait (completed)
- [x] Mailbox configuration (completed)
- [ ] Voicemail recording implementation
- [ ] Voicemail playback implementation
- [ ] PostgreSQL repository implementation
- [ ] MWI (Message Waiting Indicator)

#### Phase 3.7 - IVR System
- [x] DTMF detection (RFC 2833 + SIP INFO) (completed)
- [x] DTMF detector with buffer (completed)
- [x] IVR menu system (completed)
- [x] IVR flow engine (completed)
- [x] Menu navigation and state machine (completed)
- [ ] Audio playback implementation
- [ ] TTS integration
- [ ] ASR integration

#### Phase 4 - Enterprise Features
- [ ] Call queues and ACD
- [ ] High availability clustering
- [ ] Multi-tenancy
- [ ] SIP trunking
- [ ] Advanced codecs (Opus, H.264)

### Known Issues
- Signing service intermittent availability
- Network access required for cargo build (dependency download)
- REFER transfer logic incomplete (framework only)
- API endpoints lack authentication
- WebSocket events not yet implemented

### Performance Improvements
- Role lookup optimization via database indexes
- Permission checking via HashSet (O(1) lookup)
- Connection pooling for database access
- DTMF buffer management for efficient digit collection
- IVR session management with automatic cleanup
- Audio mixer with efficient frame mixing algorithms
- AGC with adaptive gain control (minimal overhead)
- STUN client with configurable timeout and UDP socket reuse
- Metrics collector with RwLock for concurrent access
- Conference room participant management with HashMap

### Security
- bcrypt password hashing (cost factor 12)
- SIP HA1 storage for Digest authentication (MD5/SHA-256/SHA-512)
- Role-based permission system
- System role protection
- **Brute force protection** with IP-based lockout
- **Rate limiting** to prevent abuse
- Enhanced digest authentication with SHA-256/SHA-512
- Voicemail PIN protection
- Conference room PIN-based access control
- STUN transaction ID validation
- NAT traversal security (XOR address obfuscation)

### Breaking Changes
- User entity now includes `role_id` field
- Database schema updated (requires migration)
- CreateUser and UpdateUser structs modified

---

## Previous Releases

See [ROADMAP.md](ROADMAP.md) for detailed implementation history and progress tracking.

---

**Legend:**
- ✅ Completed
- 🚧 In Progress
- 📋 Planned
- ⚠️ Known Issue

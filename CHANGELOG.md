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

#### Phase 3.7 - Audio Playback System (COMPLETED)
- **WAV File Support**
  - WavFile parser with RIFF/WAVE format support
  - WavFormat with channel, sample rate, bits per sample
  - Support for 8, 16, 24, 32-bit PCM audio
  - Chunk parsing (RIFF header, fmt chunk, data chunk)
  - G.711 compatibility checking (8kHz, mono, 8-bit)
  - Duration calculation
  - Unit tests (7 tests)

- **Audio Conversion**
  - 8/16/24/32-bit to 16-bit signed sample conversion
  - Stereo to mono conversion by channel averaging
  - Sample rate resampling using linear interpolation
  - Upsampling and downsampling support
  - G.711 compatible format conversion (8kHz mono)
  - Automatic format detection

- **Audio Player**
  - AudioPlayer with state machine (Idle, Playing, Paused, Finished, Stopped)
  - Frame-by-frame audio streaming (configurable frame duration, default 20ms)
  - Real-time pacing with sleep to maintain timing
  - Pause/resume/stop controls
  - Loop playback option
  - DTMF interrupt support
  - Progress tracking (position, duration, percentage)
  - Seek functionality
  - StreamingAudioPlayer for async contexts
  - Unit tests (10 tests)

- **Multi-language Audio Management**
  - AudioFileManager for organizing audio files
  - Language support (English, Spanish, French, German, Chinese, Japanese, Korean, Portuguese, Russian, Arabic)
  - Audio file registration with metadata (ID, path, language, description, duration, size)
  - Automatic language detection from directory structure (base_dir/lang/file.wav)
  - Get with fallback to default language
  - List by language, list all IDs, list all languages
  - Bulk directory loading
  - AudioFileInfo with metadata tracking
  - Unit tests (7 tests)

- **Sequential Playback**
  - SequentialPlayer for playing multiple files in sequence
  - Queue management (enqueue, enqueue_front, clear)
  - Automatic advancement to next audio file
  - Skip to next file
  - Overall and per-file progress tracking
  - Interrupt support with queue clearing
  - SequenceBuilder for fluent construction
  - Unit tests (9 tests)

- **Playback Options**
  - Configurable frame duration (default 20ms for telephony)
  - Loop playback mode
  - DTMF interrupt enable/disable
  - Reusable across player instances

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

#### Phase 2.5 - WebSocket Event Streaming (COMPLETED)
- **Real-time Event Broadcasting**
  - EventBroadcaster with broadcast channel (1000 event capacity)
  - SystemEvent enum with multiple event types
  - Call events (Created, Ringing, Answered, Terminated, Failed, Hold, Resume, Transfer)
  - Registration events (Registered, Unregistered, Expired)
  - Authentication events (Success, Failure, Lockout, RateLimited)
  - Conference events (Created, Started, Ended, ParticipantJoined/Left/Muted/Unmuted)
  - Health update events
  - Custom events support
  - Unit tests (7 tests)

- **WebSocket Handler**
  - WebSocket upgrade handler
  - Bidirectional WebSocket connection management
  - Welcome message on connection
  - JSON event serialization
  - Ping/pong support
  - Client disconnect handling
  - Concurrent send/receive tasks

#### Phase 3.1 - Conference PostgreSQL and API (COMPLETED)
- **Conference PostgreSQL Repository**
  - PgConferenceRepository implementation
  - Conference room CRUD operations
  - Participant management (add, remove, update, list)
  - State filtering and queries
  - Cascading deletes
  - Integration tests (3 tests)

- **Conference REST API**
  - POST /conferences - Create conference
  - GET /conferences - List conferences (with state filter)
  - GET /conferences/:id - Get conference details
  - PUT /conferences/:id - Update conference
  - DELETE /conferences/:id - Delete conference
  - POST /conferences/:id/start - Start conference
  - POST /conferences/:id/end - End conference
  - POST /conferences/:id/lock - Lock conference
  - POST /conferences/:id/unlock - Unlock conference
  - POST /conferences/:id/participants - Add participant
  - GET /conferences/:id/participants - List participants
  - DELETE /conferences/:id/participants/:pid - Remove participant
  - POST /conferences/:id/participants/:pid/mute - Mute participant
  - POST /conferences/:id/participants/:pid/unmute - Unmute participant
  - PUT /conferences/:id/participants/:pid/role - Update participant role
  - Full JSON request/response DTOs
  - ConferenceApiState for dependency injection

#### Phase 3.6 - Voicemail PostgreSQL (COMPLETED)
- **Voicemail PostgreSQL Repository**
  - PgVoicemailRepository implementation
  - Voicemail message CRUD operations
  - Mailbox configuration management
  - Status updates (New, Read, Saved, Deleted)
  - Message listing with status filtering
  - Message counting
  - Upsert support for mailboxes
  - Automatic timestamp updates
  - Integration tests (3 tests)

#### Database Migrations
- **20251106_02_create_conference_tables.sql**
  - conference_rooms table (UUID primary key, state, PIN, recording)
  - conference_participants table (role, state, mute, volume)
  - Cascading deletes
  - 5 performance indexes
  - Comprehensive table/column comments

- **20251106_03_create_voicemail_tables.sql**
  - voicemail_mailboxes table (PIN, greetings, limits, email notification)
  - voicemail_messages table (caller info, audio file, status, timestamps)
  - Cascading deletes
  - 5 performance indexes
  - Automatic updated_at trigger
  - Comprehensive table/column comments

#### Phase 3.6 - Voicemail API (COMPLETED)
- **Voicemail REST API**
  - GET /voicemail/mailboxes/:id - Get mailbox configuration
  - PUT /voicemail/mailboxes/:id - Update mailbox settings
  - GET /voicemail/mailboxes/:id/messages - List messages (with status filter)
  - POST /voicemail/mailboxes/:id/messages - Create message
  - GET /voicemail/messages/:id - Get message details
  - DELETE /voicemail/messages/:id - Delete message
  - PUT /voicemail/messages/:id/status - Update message status
  - POST /voicemail/messages/:id/mark-read - Mark as read
  - POST /voicemail/messages/:id/mark-saved - Mark as saved
  - GET /voicemail/mailboxes/:id/count - Get message count statistics
  - Full JSON request/response DTOs
  - VoicemailApiState for dependency injection

#### Phase 3.6 - Voicemail Recording and Playback (COMPLETED)
- **Voicemail Recorder**
  - VoicemailRecorder for capturing audio to WAV files
  - Configurable maximum recording duration (default 180s)
  - Sample buffer with 16-bit PCM mono at 8kHz
  - Real-time duration tracking during recording
  - Start/stop recording controls
  - WAV file generation with proper RIFF headers
  - Automatic format: 8kHz mono 16-bit PCM
  - Unit tests (4 tests)

- **WAV File Writing**
  - Complete RIFF/WAVE file format writer
  - RIFF header (RIFF signature, file size, WAVE format)
  - fmt chunk (audio format, channels, sample rate, bit depth)
  - data chunk (PCM audio samples)
  - Proper little-endian byte ordering
  - Block alignment and byte rate calculation

- **Voicemail Player**
  - VoicemailPlayer for playing back messages
  - Frame-by-frame audio streaming (20ms frames)
  - Play voicemail messages from files
  - Play custom greetings
  - Automatic G.711 format conversion
  - Playback controls (pause, resume, stop, replay)
  - Progress tracking and seeking
  - Fast forward/rewind support (configurable seconds)
  - Integration with AudioPlayer

- **Voicemail Service**
  - VoicemailService for managing recordings and playback
  - Mailbox directory structure management
  - Automatic directory creation per mailbox
  - Unique filename generation (timestamp + UUID)
  - Save recordings with metadata
  - Create VoicemailMessage entities from recordings
  - Audio file deletion
  - File size queries
  - Base directory configuration

- **Message Waiting Indicator (MWI)**
  - MwiState with message counts
  - New vs old message tracking
  - Urgent message counts (separate counters)
  - SIP NOTIFY body formatting (RFC 3842)
  - Messages-Waiting header (yes/no)
  - Message-Account header
  - Voice-Message header with counts (new/old/urgent)
  - Total message count calculation
  - Unit tests (4 tests)

- **Voicemail IVR Access**
  - VoicemailIvrSession for dial-in access
  - State machine (Authenticating, VerifyingPin, MainMenu, PlayingMessage, etc.)
  - PIN authentication with retry limit (3 attempts)
  - Mailbox auto-detection from caller ID
  - Message navigation (next, previous, current position)
  - Message status management (mark read, saved, deleted)
  - Message sorting (newest first)
  - Session variables for state tracking
  - Unit tests (9 tests)

- **DTMF Menu Options**
  - VoicemailMenuOption enum for IVR navigation
  - Play next (1), Replay (2), Delete (3), Save (4)
  - Previous (5), Skip (6), Main menu (*), Exit (#)
  - Record greeting (9)
  - Bidirectional digit <-> option mapping

- **Voicemail Prompts**
  - VoicemailPrompt enum with audio IDs
  - Welcome, Enter PIN, Invalid PIN
  - New/saved message count announcements
  - Main menu options, Message headers
  - Deletion/save confirmations
  - No more messages, Goodbye
  - Recording prompts

- **Message Management**
  - Load messages for mailbox
  - Sort by date (newest first)
  - Filter by status (new, read, saved, deleted)
  - Count new vs saved messages
  - Remove deleted messages from list
  - Current position tracking (1-based)
  - Has more messages detection

#### Phase 3.2 - TURN Relay (COMPLETED)
- **TURN Protocol Implementation (RFC 5766)**
  - TurnMethod enum (Allocate, Refresh, Send, Data, CreatePermission, ChannelBind)
  - TurnMessage parsing and serialization
  - TurnAttribute support (Lifetime, RequestedTransport, Data, XorRelayedAddress, etc.)
  - Message type encoding/decoding
  - Transaction ID generation
  - Unit tests (5 tests)

- **TURN Client**
  - TurnClient for relay allocation
  - Authentication support (username/password)
  - Allocate relay address
  - Refresh allocation lifetime
  - Create permissions for peers
  - Send indications through relay
  - Configurable timeout
  - Unit tests (3 tests)

- **TURN Relay Server**
  - TurnRelay with port management (base_port to max_port)
  - RelayAllocation tracking (client, relay address, lifetime, permissions)
  - Allocation lifecycle management
  - Permission management per allocation
  - Bidirectional relay (client<->peer)
  - Automatic cleanup of expired allocations
  - Statistics and monitoring
  - Unit tests (6 tests)

#### Phase 3.2 - ICE Implementation (COMPLETED)
- **ICE Candidate Types**
  - CandidateType enum (Host, ServerReflexive, PeerReflexive, Relay)
  - Priority computation (RFC 5245)
  - Foundation generation
  - SDP candidate format parsing and serialization
  - Related address support for reflexive/relay candidates
  - Unit tests (8 tests)

- **ICE Candidate Pairs**
  - IceCandidatePair with state machine
  - CandidatePairState (Frozen, Waiting, InProgress, Succeeded, Failed)
  - Pair priority computation
  - Controlling/controlled role support

- **ICE Agent**
  - IceAgent with complete candidate gathering
  - IceConfig with STUN/TURN server lists
  - Host candidate gathering from local interfaces
  - Server reflexive candidate gathering via STUN
  - Relay candidate gathering via TURN
  - Candidate pair formation
  - Connection state machine (New, Checking, Connected, Completed, Failed, etc.)
  - Gathering state tracking
  - Selected pair management
  - Unit tests (4 tests)

#### Phase 2.3 - Audit Logging (COMPLETED)
- **Audit Event System**
  - AuditEvent with comprehensive metadata
  - AuditEventType enum with 20+ event types
  - AuditLevel (Info, Warning, Critical)
  - Event categories: Authentication, User Management, Roles, Calls, Conferences, System Config, Security, Data Access
  - IP address, user agent, session tracking
  - Custom metadata support
  - Timestamp and UUID for each event

- **Audit Backend**
  - AuditBackend trait for pluggable backends
  - MemoryAuditBackend with configurable capacity
  - FIFO event retention
  - Query support with multiple filters (time range, level, username, IP)
  - Async logging

- **Audit Logger**
  - AuditLogger with convenience methods
  - Integration with tracing for operational logging
  - Common event helpers (auth_success, auth_failure, auth_lockout, etc.)
  - Query API for audit trail analysis
  - Unit tests (4 tests)

#### Phase 3.3 - WebRTC SDP Support (COMPLETED)
- **WebRTC SDP Implementation**
  - SdpType enum (Offer, Answer, Pranswer)
  - MediaType enum (Audio, Video, Application)
  - MediaDirection (SendRecv, SendOnly, RecvOnly, Inactive)
  - Complete SDP string parsing and serialization
  - BUNDLE support for multiplexing
  - Unit tests (9 tests)

- **RTP Codec Support**
  - RtpCodec with payload type, name, clock rate, channels
  - Predefined codecs: Opus (111/48000/2), PCMU (0/8000), PCMA (8/8000), VP8 (96/90000), H264 (97/90000)
  - Format parameters support
  - RTCP feedback support

- **Media Description**
  - MediaDescription with codec list
  - ICE credentials (ufrag, pwd)
  - ICE candidate embedding
  - DTLS fingerprint integration
  - DtlsSetup (Active, Passive, ActPass)
  - RTP/RTCP multiplexing
  - RTCP feedback configuration

- **Helper Functions**
  - create_audio_offer() for quick audio SDP generation
  - SDP string generation with proper v=0, o=, s=, t=, m= lines
  - ICE candidate line formatting
  - DTLS fingerprint formatting

#### Phase 3.3 - WebRTC Signaling Server (COMPLETED)
- **WebSocket-based Signaling**
  - WebSocket upgrade handler at `/webrtc/signaling/:peer_id`
  - Bidirectional real-time communication
  - Split send/receive tasks for concurrent operation
  - Automatic peer cleanup on disconnect
  - Ping/pong support for connection health
  - Unit tests (4 tests)

- **Signaling Protocol**
  - SignalingMessage enum with 12 message types
  - Register/Unregister: Peer registration and discovery
  - Offer/Answer: SDP offer/answer exchange for WebRTC negotiation
  - IceCandidate: ICE candidate exchange with sdp_mid and sdp_m_line_index
  - Call/Accept/Reject/Hangup: Call control signaling
  - PeerStatus: Online/offline presence notifications
  - Error/Success: Status messages with codes and descriptions
  - JSON serialization with tagged union format

- **Peer Management**
  - SignalingState with thread-safe peer tracking
  - WebRtcPeer entity (peer_id, username, connected_at)
  - Peer registration with duplicate detection
  - Automatic online/offline status broadcasting
  - Peer lookup by ID
  - List all online peers

- **Message Routing**
  - Broadcast channel for pub/sub messaging (1000 message capacity)
  - Send to specific peer by ID
  - Broadcast to all peers
  - Sender verification for security
  - Recipient validation before forwarding
  - Peer ID mismatch detection

- **Security Features**
  - Sender ID verification on all forwarded messages
  - Peer ID must match WebSocket connection
  - Recipient existence validation
  - Error responses for invalid messages
  - Parse error handling with detailed error messages

#### Phase 4 - Call Queue and ACD System (COMPLETED)
- **Queue Strategies**
  - RingAll: All available agents ring simultaneously
  - Linear: Agents ring in order until answered
  - LeastRecent: Agent who hasn't received call longest
  - FewestCalls: Agent with lowest total calls
  - LeastTalkTime: Agent with lowest total talk time
  - Random: Random agent selection
  - RoundRobin: Rotating distribution

- **Queue Member Management**
  - QueueMember entity with agent tracking
  - AgentStatus (Available, OnCall, Paused, Unavailable)
  - Call statistics: total calls, answered, missed
  - Talk time tracking
  - Last call timestamp
  - Pause reasons
  - Login/logout tracking

- **Call Queue Configuration**
  - Queue name and strategy
  - Max wait time before overflow
  - Max queue size
  - Ring timeout per agent
  - Announce position option
  - Music on hold configuration
  - Join/leave events
  - Overflow destination

- **Queue State Management**
  - QueuedCall tracking with timestamp and position
  - Active call mapping
  - Round-robin position tracking
  - Get next agent by strategy
  - Queue statistics (waiting calls, avg wait time, abandonment rate)
  - Unit tests (8 tests)

#### Phase 4 - Call Queue Engine (COMPLETED)
- **CallQueueEngine Service**
  - Central orchestration for all queue operations
  - Thread-safe session management with Arc<Mutex>
  - Multiple queue support with HashMap-based routing
  - Start/stop queue sessions
  - Integration with AudioFileManager for announcements

- **Call Queueing**
  - enqueue_call() with overflow detection
  - Queue full checking against max_queue_size
  - Automatic position assignment (1-based)
  - Caller information tracking (name, number)
  - Wait time calculation from enqueued_at timestamp
  - Priority support for future enhancement

- **Agent Selection Strategies**
  - RingAll: Return first available (caller rings all)
  - Linear: Agents by join time (FIFO)
  - LeastRecent: Agent with oldest last_call_time
  - FewestCalls: Agent with minimum total_calls
  - LeastTalkTime: Agent with minimum talk_time
  - Random: Random selection from available pool
  - RoundRobin: Rotating with position tracking

- **Agent State Management**
  - Add/remove members from queue
  - Available agent tracking
  - Mark busy on call connect
  - Mark available on call end
  - Pause/unpause support
  - Statistics per agent (calls, talk time)

- **Call Connection Flow**
  - connect_call() links call to agent
  - Remove from waiting queue
  - Add to active calls HashMap
  - Mark agent as busy
  - Update queue statistics
  - SLA threshold checking
  - Position updates for remaining calls

- **Call Lifecycle**
  - end_call() with talk_time tracking
  - Agent statistics updates
  - Mark agent available
  - Remove from active calls
  - abandon_call() for caller hangups
  - Abandonment statistics tracking

- **Queue Statistics (Real-time)**
  - Total calls received
  - Calls waiting (current)
  - Calls active (current)
  - Calls answered (cumulative)
  - Calls abandoned (cumulative)
  - Calls overflowed (cumulative)
  - Average wait time (dynamic)
  - Longest wait time (peak)
  - Service level percentage
  - Configurable SLA threshold (default 20s)

- **Service Level Agreement (SLA)**
  - Track calls answered within threshold
  - Automatic calculation: (within_threshold / total_answered) * 100
  - Default threshold: 20 seconds
  - Real-time percentage updates

- **Music on Hold Integration**
  - create_moh_player() from queue config
  - Integration with AudioFileManager
  - Loop playback for continuous music
  - StreamingAudioPlayer with 20ms frames
  - Automatic audio file lookup by ID

- **Queue Monitoring**
  - get_statistics() for real-time metrics
  - get_position() for caller position announcements
  - get_waiting_calls() for queue visibility
  - get_available_agents() for capacity planning
  - Per-queue session tracking

- **Error Handling**
  - QueueEngineError enum
  - QueueFull when max_queue_size exceeded
  - QueueNotFound for invalid queue_id
  - NoAvailableAgents when all busy
  - CallNotFound for invalid call operations
  - MemberNotFound for agent operations

- **Overflow Handling**
  - Detect queue full condition
  - Increment overflow statistics
  - Return QueueFull error
  - Caller can trigger overflow action
  - Support for forwarding/voicemail/announcement

- **Unit Tests**
  - Engine creation (1 test)
  - Enqueue call (1 test)
  - Queue full detection (1 test)
  - Add/remove member (1 test)
  - Connect call flow (1 test)
  - Abandon call (1 test)
  - End call (1 test)
  - Round-robin strategy (1 test)
  - Total: 8 comprehensive tests

#### Phase 4.1 - Call Announcer Service (COMPLETED)
- **CallAnnouncer Service**
  - Audio announcement playback into active calls
  - Integration with AudioFileManager
  - Multi-language support via Language enum
  - Thread-safe active announcement tracking
  - Scheduled announcement queue with time-based execution
  - Frame-by-frame audio delivery (20ms frames)

- **Announcement Types**
  - QueuePosition: "You are caller number X"
  - WaitTime: "Estimated wait time X minutes"
  - Periodic: Recurring announcements at intervals
  - Welcome: Queue entry greeting
  - Goodbye: Queue exit message
  - Custom: User-defined announcements

- **Announcement Request**
  - AnnouncementRequest with builder pattern
  - Unique UUID per announcement
  - Call ID targeting
  - Audio file list for sequential playback
  - Language selection
  - Scheduled playback (play_at timestamp)
  - Repeat interval for periodic announcements

- **Position Announcements**
  - announce_position() helper method
  - Number-to-speech conversion (1-999)
  - Multi-language prompts:
    - "queue-position" (introductory prompt)
    - Number words: "one", "two", ... "nine-hundred"
    - Tens: "ten", "twenty", "thirty", etc.
    - Teens: "eleven", "twelve", "thirteen", etc.
  - Audio sequence building

- **Wait Time Announcements**
  - announce_wait_time() helper method
  - Minute-based estimates
  - Multi-language prompts:
    - "estimated-wait" (introductory prompt)
    - Number words for minutes
    - "minute" / "minutes" (singular/plural)
  - Automatic sequence generation

- **Welcome Announcements**
  - announce_welcome() helper
  - Customizable audio file ID
  - Immediate playback
  - Queue greeting use case

- **Active Announcement Tracking**
  - Per-call announcement lists
  - ActiveAnnouncement struct with player
  - Automatic cleanup on completion
  - Multiple simultaneous announcements per call
  - UUID-based announcement identification

- **Frame Delivery**
  - get_next_frame() for RTP streaming
  - Returns (samples: Vec<i16>, sample_rate: usize)
  - Automatic advancement to next file
  - Cleanup on sequence completion
  - Integration with existing audio players

- **Scheduled Announcements**
  - Scheduled announcement queue
  - process_scheduled() for time checking
  - Automatic promotion to active on time match
  - Repeat interval support for periodic announcements
  - Manual stop via stop_announcement()

- **Number-to-Speech Conversion**
  - number_to_audio_id() helper function
  - Handles 1-999 range
  - Digit-by-digit decomposition:
    - Hundreds place: "one-hundred", "two-hundred", etc.
    - Tens place: "twenty", "thirty", etc.
    - Teens: "eleven" through "nineteen"
    - Ones place: "one" through "nine"
  - Returns Vec<String> of audio file IDs
  - Used for position and wait time announcements

- **Integration Points**
  - AudioFileManager for file lookup
  - AudioPlayer for playback control
  - CallQueue for position/wait time data
  - RTP media stream for audio injection
  - Multi-language audio file structure

- **Error Handling**
  - Audio file not found errors
  - Invalid call ID detection
  - Player creation failures
  - Scheduled announcement validation
  - Graceful degradation on missing files

- **Unit Tests**
  - Announcement creation (1 test)
  - Position announcement (1 test)
  - Wait time announcement (1 test)
  - Welcome announcement (1 test)
  - Frame retrieval (1 test)
  - Number-to-speech conversion (1 test)
  - Scheduled announcements (1 test)
  - Total: 7 comprehensive tests

#### Phase 2.1 - TLS/DTLS Configuration (COMPLETED)
- **TLS Configuration**
  - TlsMode enum (Disabled, Optional, Required)
  - TlsConfig with certificate/key paths
  - Peer verification options
  - TLS version control (min_version)
  - Cipher suite configuration
  - Mutual TLS support
  - Configuration validation
  - Unit tests (7 tests)

- **DTLS Configuration**
  - DtlsConfig for WebRTC
  - Fingerprint algorithm (SHA-256, SHA-384, SHA-512)
  - DtlsRole (Client, Server)
  - DtlsSetup (Active, Passive, ActPass)
  - SRTP profile support
  - Fingerprint generation

- **Certificate Management**
  - Certificate entity with metadata
  - CertificateType (Server, Client, CA)
  - X.509 certificate parsing (placeholder)
  - Certificate validity checking
  - Expiration warning (days until expiry)
  - Self-signed certificate detection
  - SAN (Subject Alternative Name) support
  - DTLS fingerprint generation for SDP

- **Certificate Manager**
  - Load certificates from PEM files
  - Load private keys from PEM files
  - Certificate storage and retrieval
  - Get certificates by type (Server, CA)
  - Find expiring certificates
  - Generate self-signed certificates (placeholder)
  - Certificate removal
  - Unit tests (3 tests)

#### Phase 2.1 - SRTP/SRTCP Encryption (COMPLETED)
- **SRTP Protection Profiles**
  - SrtpProfile enum (AES-128-CM with HMAC-SHA1-80/32, AES-256-CM with HMAC-SHA1-80/32)
  - Master key length (16 bytes for AES-128, 32 bytes for AES-256)
  - Master salt length (14 bytes for all profiles)
  - Authentication tag length (10 bytes for 80-bit, 4 bytes for 32-bit)
  - Cipher, auth, and salt key length configuration

- **Key Derivation (RFC 3711)**
  - SRTP Key Derivation Function (KDF) implementation
  - AES-CM PRF (Pseudo-Random Function)
  - Key labels for SRTP/SRTCP encryption, authentication, salting
  - Session key derivation from master key
  - Separate keys for SRTP and SRTCP
  - 48-bit index support for key derivation

- **Master Key Management**
  - SrtpMasterKey with key and salt
  - Random master key generation per profile
  - Session key derivation (6 keys: SRTP cipher/auth/salt, SRTCP cipher/auth/salt)
  - SrtpSessionKeys structure

- **Cryptographic Primitives**
  - AES-128 Counter Mode encryption/decryption
  - HMAC-SHA1 authentication (160-bit key, configurable tag length)
  - IV generation from salt, SSRC, and packet index
  - Keystream generation for XOR encryption
  - Authentication tag computation and verification

- **SRTP Context**
  - Per-SSRC stream context management
  - Packet index calculation (ROC * 65536 + SEQ)
  - ROC (Rollover Counter) tracking for 32-bit sequence extension
  - RTP header parsing (SSRC, sequence number)
  - RTP header length calculation (with CSRC and extensions)
  - Encrypt/decrypt RTP packets in-place
  - Authentication tag append/verify

- **Replay Protection**
  - Sliding window bitmap (64 packets)
  - Highest sequence number tracking
  - Check for replay attacks before decryption
  - Window update after accepting packet
  - Configurable replay protection (can be disabled)

- **SRTCP Context**
  - SRTCP index management
  - E flag handling (1 bit for encryption indicator)
  - SRTCP index (31 bits) in packet
  - RTCP header parsing
  - Encrypt/decrypt RTCP packets
  - Authentication for both encrypted and unencrypted RTCP

- **MediaCryptoContext**
  - Combined SRTP/SRTCP context for media sessions
  - Unified API for RTP/RTCP protection
  - protect_rtp() / unprotect_rtp() methods
  - protect_rtcp() / unprotect_rtcp() methods
  - Single master key for both SRTP and SRTCP

- **Unit Tests**
  - SRTP profile length tests (3 tests)
  - Key derivation tests (3 tests)
  - Session key derivation (1 test)
  - HMAC authentication tests (1 test)
  - IV generation tests (2 tests)
  - AES-CM keystream tests (2 tests)
  - Replay window tests (1 test)
  - Stream context tests (1 test)
  - RTP header parsing tests (2 tests)
  - SRTP encrypt/decrypt tests (4 tests)
  - SRTP authentication failure tests (1 test)
  - SRTP replay protection tests (1 test)
  - Multi-stream tests (1 test)
  - SRTCP encrypt/decrypt tests (4 tests)
  - SRTCP authentication tests (1 test)
  - SRTCP index increment tests (1 test)
  - MediaCryptoContext tests (4 tests)
  - Total: 33 unit tests

#### Phase 4 - SIP Trunk Support (COMPLETED)
- **SIP Trunk Configuration**
  - TrunkType (Register, IpBased, Peer)
  - TrunkDirection (Inbound, Outbound, Bidirectional)
  - Provider configuration (name, SIP server, port)
  - Backup server support
  - Unit tests (8 tests)

- **Authentication**
  - Username/password authentication
  - Auth username and realm
  - IP-based authentication with allowed IP list
  - Add/check allowed IPs

- **Registration Management**
  - Registration enable/disable
  - Configurable registration interval
  - Registration expiry tracking
  - Last registration timestamp
  - Needs registration check (auto-refresh 60s before expiry)
  - Mark registered/unregistered

- **Call Routing**
  - Prefix matching
  - Prefix stripping
  - Prefix addition
  - Caller ID number override
  - Caller ID name override
  - Number formatting for outbound calls

- **Codec Configuration**
  - CodecPreference with priority
  - Default codecs: PCMU (100), PCMA (99), G729 (98)
  - DtmfMode (Rfc2833, SipInfo, Inband)

- **Capacity and Quality**
  - Max concurrent calls limit
  - Max calls per second limit
  - RTCP enable/disable
  - T.38 fax support
  - SRTP encryption support

- **Trunk Statistics**
  - TrunkStatistics for monitoring
  - Current calls tracking
  - Total calls, successful calls, failed calls
  - Success rate calculation
  - Average call duration
  - Total minutes
  - Last call time

#### Phase 4 - Multi-tenancy Support (COMPLETED)
- **Tenant Management**
  - Tenant entity with UUID
  - TenantStatus (Active, Suspended, Trial, Deactivated)
  - SubscriptionPlan (Free, Starter, Professional, Enterprise, Custom)
  - Tenant slug for URL-safe identifier
  - SIP realm per tenant for isolation
  - Trial period with expiration tracking
  - Unit tests (8 tests)

- **Subscription Plans and Quotas**
  - TenantQuota with resource limits
  - Free tier: 5 users, 2 calls, 100 min/mo, 1GB storage
  - Starter: 25 users, 10 calls, 1000 min/mo, 10GB storage
  - Professional: 100 users, 50 calls, 5000 min/mo, 50GB storage
  - Enterprise: Unlimited users, 1000 calls, unlimited min, 500GB storage
  - Feature flags per plan (voicemail, IVR, call_queue, analytics, SIP trunk, WebRTC, API access)

- **Tenant Features**
  - Plan upgrade/downgrade
  - Suspend/reactivate tenant
  - Suspension reason tracking
  - Feature availability checking
  - User limit enforcement
  - Concurrent call limit enforcement
  - Trial expiration checking

- **Tenant Configuration**
  - Contact information (admin email/name, phone, company)
  - Billing information (email, address)
  - Custom domain support
  - Timezone and language
  - Branding (logo URL, primary color)
  - Custom metadata

- **Usage Tracking**
  - TenantUsage for consumption monitoring
  - Current users and calls
  - Monthly call minutes tracking
  - Storage usage (GB)
  - Last activity timestamp
  - Quota compliance checking
  - Usage percentage calculations (users, calls, minutes, storage)

#### Phase 4 - Call Queue PostgreSQL and API (COMPLETED)
- **Call Queue PostgreSQL Repository**
  - PgCallQueueRepository implementation
  - Queue CRUD operations (create, get, update, delete, list)
  - Get queue by extension
  - Member management (add, remove, update, get members)
  - Full PostgreSQL persistence

- **Call Queue REST API**
  - POST /queues - Create call queue
  - GET /queues - List all queues
  - GET /queues/:id - Get queue by ID
  - PUT /queues/:id - Update queue
  - DELETE /queues/:id - Delete queue
  - GET /queues/extension/:extension - Get queue by extension
  - POST /queues/:id/members - Add member to queue
  - GET /queues/:id/members - List queue members
  - DELETE /queues/:id/members/:member_id - Remove member
  - PUT /queues/:id/members/:member_id - Update member status
  - POST /queues/:id/members/:member_id/pause - Pause member
  - POST /queues/:id/members/:member_id/unpause - Unpause member
  - Full JSON request/response DTOs

#### Phase 4 - Multi-tenancy PostgreSQL and API (COMPLETED)
- **Tenant PostgreSQL Repository**
  - PgTenantRepository implementation
  - Tenant CRUD operations
  - Get tenant by slug
  - List tenants with status filter
  - Tenant usage tracking (get, update)
  - Full PostgreSQL persistence

- **Tenant REST API**
  - POST /tenants - Create tenant
  - GET /tenants - List all tenants (with status filter)
  - GET /tenants/:id - Get tenant by ID
  - PUT /tenants/:id - Update tenant
  - DELETE /tenants/:id - Delete tenant
  - GET /tenants/slug/:slug - Get tenant by slug
  - POST /tenants/:id/suspend - Suspend tenant
  - POST /tenants/:id/reactivate - Reactivate tenant
  - POST /tenants/:id/upgrade - Upgrade subscription plan
  - GET /tenants/:id/usage - Get tenant usage statistics
  - Full JSON request/response DTOs

#### Phase 4 - SIP Trunk PostgreSQL and API (COMPLETED)
- **SIP Trunk PostgreSQL Repository**
  - PgSipTrunkRepository implementation
  - Trunk CRUD operations
  - Get trunk by name
  - List trunks (all or enabled only)
  - Trunk statistics tracking (get, update)
  - Full PostgreSQL persistence

- **SIP Trunk REST API**
  - POST /trunks - Create SIP trunk
  - GET /trunks - List all trunks
  - GET /trunks/:id - Get trunk by ID
  - PUT /trunks/:id - Update trunk
  - DELETE /trunks/:id - Delete trunk
  - GET /trunks/name/:name - Get trunk by name
  - POST /trunks/:id/register - Trigger registration
  - GET /trunks/:id/statistics - Get trunk statistics
  - Full JSON request/response DTOs

#### Database Migrations
- **20251106_04_create_call_queue_tables.sql**
  - call_queues table (queue configuration with strategy, timeouts, overflow)
  - queue_members table (agent status, statistics, pause reasons)
  - 6 performance indexes
  - Automatic updated_at trigger
  - Comprehensive table/column comments

- **20251106_05_create_tenant_tables.sql**
  - tenants table (name, slug, status, plan, quotas, branding, metadata)
  - tenant_usage table (real-time usage tracking)
  - 6 performance indexes
  - Automatic updated_at trigger
  - Comprehensive table/column comments

- **20251106_06_create_sip_trunk_tables.sql**
  - sip_trunks table (provider config, registration, codecs, routing)
  - trunk_statistics table (real-time call statistics)
  - 5 performance indexes
  - Automatic updated_at trigger
  - Comprehensive table/column comments

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
- `20251106_02_create_conference_tables.sql`
  - Creates conference_rooms table
  - Creates conference_participants table
  - Adds indexes and comments
- `20251106_03_create_voicemail_tables.sql`
  - Creates voicemail_mailboxes table
  - Creates voicemail_messages table
  - Adds indexes, triggers, and comments

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
- WebSocket event tests (7 tests)
- Conference repository tests (3 tests)
- Voicemail repository tests (3 tests)
- TURN message tests (5 tests)
- TURN client tests (3 tests)
- TURN relay tests (6 tests)
- ICE candidate tests (8 tests)
- ICE agent tests (4 tests)
- Audit logging tests (4 tests)
- WebRTC SDP tests (9 tests)
- Call queue tests (8 tests)
- TLS configuration tests (7 tests)
- Certificate management tests (3 tests)
- SIP trunk tests (8 tests)
- Multi-tenancy tests (8 tests)
- **Total new tests: 190**

### TODO / In Progress

#### Phase 2.1 - TLS/SRTP Encryption
- [x] TLS configuration framework (completed)
- [x] DTLS configuration (completed)
- [x] Certificate management (completed)
- [ ] TLS listener implementation
- [ ] SRTP media encryption implementation
- [ ] DTLS-SRTP for WebRTC implementation

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
- [x] Audit logging (completed)
- [ ] IP blacklisting (basic framework in place)

#### Phase 2.5 - Monitoring Enhancements
- [x] System health monitoring (completed)
- [x] Extended metrics (completed)
- [x] Metrics collector (completed)
- [x] WebSocket event streaming (completed)
- [ ] API authentication/authorization
- [ ] Performance profiling
- [ ] Grafana dashboards

#### Phase 3.1 - Conference Features
- [x] Conference room management (completed)
- [x] Audio mixing (completed)
- [x] Participant controls (completed)
- [x] Conference domain model (completed)
- [x] PostgreSQL repository implementation (completed)
- [x] Conference API endpoints (completed)
- [ ] Conference recording implementation
- [ ] Music on hold for conferences

#### Phase 3.2 - NAT Traversal
- [x] STUN client (completed)
- [x] STUN protocol implementation (completed)
- [x] NAT type detection (completed)
- [x] TURN relay (completed)
- [x] TURN client (completed)
- [x] ICE candidate gathering (completed)
- [x] ICE agent (completed)
- [ ] STUN server implementation
- [ ] ICE connectivity checks implementation

#### Phase 3.3 - WebRTC Integration
- [x] WebRTC SDP support (completed)
- [ ] WebSocket signaling server
- [ ] Browser compatibility testing

#### Phase 3.6 - Voicemail
- [x] Voicemail domain model (completed)
- [x] Voicemail repository trait (completed)
- [x] Mailbox configuration (completed)
- [x] PostgreSQL repository implementation (completed)
- [x] Voicemail API endpoints (completed)
- [ ] Voicemail recording implementation
- [ ] Voicemail playback implementation
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
- [x] Call queue domain model (completed)
- [x] Multi-tenancy domain model (completed)
- [x] SIP trunk domain model (completed)
- [x] Call queue persistence and API (completed)
- [x] Multi-tenancy persistence and API (completed)
- [x] SIP trunk persistence and API (completed)
- [ ] Call queue integration with SIP routing
- [ ] SIP trunk integration with SIP registration
- [ ] High availability clustering
- [ ] Advanced codecs (Opus, H.264)

### Known Issues
- Signing service intermittent availability
- Network access required for cargo build (dependency download)
- REFER transfer logic incomplete (framework only)
- API endpoints lack authentication
- Conference recording not yet implemented
- Voicemail recording/playback not yet implemented

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
- WebSocket event broadcasting with tokio broadcast channel (1000 event capacity)
- Conference PostgreSQL queries with appropriate indexes
- Voicemail message filtering with indexed queries
- Cascading deletes for referential integrity

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

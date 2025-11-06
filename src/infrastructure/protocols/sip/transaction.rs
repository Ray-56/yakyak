//! SIP Transaction Layer
//!
//! Implements client and server transactions as per RFC 3261 Section 17
//!
//! ## Transaction Types
//! - INVITE Client Transaction (ICT) - Section 17.1.1
//! - INVITE Server Transaction (IST) - Section 17.2.1
//! - Non-INVITE Client Transaction (NICT) - Section 17.1.2
//! - Non-INVITE Server Transaction (NIST) - Section 17.2.2

use super::message::{SipRequest, SipResponse};
use rsip::{Header, Headers};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::{debug, info, warn};

/// Extract branch parameter from Via header
fn extract_branch(headers: &Headers) -> Option<String> {
    headers.iter().find_map(|h| match h {
        Header::Via(via) => {
            // Convert Via header to string and extract branch parameter
            let via_str = via.to_string();
            via_str
                .split(';')
                .find(|p| p.trim().starts_with("branch="))
                .and_then(|b| b.split('=').nth(1))
                .map(|s| s.trim().to_string())
        }
        _ => None,
    })
}

/// Transaction ID - uniquely identifies a transaction
/// Based on branch parameter in Via header
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TransactionId(pub String);

impl TransactionId {
    /// Create transaction ID from branch parameter
    pub fn from_branch(branch: &str) -> Self {
        Self(branch.to_string())
    }

    /// Generate a new transaction ID
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random: u64 = rng.gen();
        Self(format!("z9hG4bK{:x}", random))
    }
}

/// SIP Timers (RFC 3261 Section 17.1.1.1)
#[derive(Debug, Clone, Copy)]
pub struct SipTimers {
    /// T1: RTT Estimate (default 500ms)
    pub t1: Duration,
    /// T2: Maximum retransmit interval (default 4s)
    pub t2: Duration,
    /// T4: Maximum duration a message remains in network (default 5s)
    pub t4: Duration,
}

impl Default for SipTimers {
    fn default() -> Self {
        Self {
            t1: Duration::from_millis(500),
            t2: Duration::from_secs(4),
            t4: Duration::from_secs(5),
        }
    }
}

/// Timer types for SIP transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    /// Timer A: INVITE request retransmit interval (default: T1)
    TimerA,
    /// Timer B: INVITE transaction timeout (default: 64*T1)
    TimerB,
    /// Timer D: Wait time for response retransmits (default: >32s for UDP, 0s for TCP)
    TimerD,
    /// Timer E: Non-INVITE request retransmit (default: T1)
    TimerE,
    /// Timer F: Non-INVITE transaction timeout (default: 64*T1)
    TimerF,
    /// Timer G: INVITE response retransmit (default: T1)
    TimerG,
    /// Timer H: Wait time for ACK receipt (default: 64*T1)
    TimerH,
    /// Timer I: Wait time for ACK retransmits (default: T4 for UDP, 0s for TCP)
    TimerI,
    /// Timer J: Wait time for retransmits of non-INVITE requests (default: 64*T1 for UDP, 0s for TCP)
    TimerJ,
    /// Timer K: Wait time for response retransmits (default: T4 for UDP, 0s for TCP)
    TimerK,
}

impl TimerType {
    /// Get default duration for this timer
    pub fn default_duration(&self, timers: &SipTimers, is_reliable: bool) -> Duration {
        match self {
            TimerType::TimerA => timers.t1,
            TimerType::TimerB => timers.t1 * 64,
            TimerType::TimerD => {
                if is_reliable {
                    Duration::from_secs(0)
                } else {
                    Duration::from_secs(32)
                }
            }
            TimerType::TimerE => timers.t1,
            TimerType::TimerF => timers.t1 * 64,
            TimerType::TimerG => timers.t1,
            TimerType::TimerH => timers.t1 * 64,
            TimerType::TimerI => {
                if is_reliable {
                    Duration::from_secs(0)
                } else {
                    timers.t4
                }
            }
            TimerType::TimerJ => {
                if is_reliable {
                    Duration::from_secs(0)
                } else {
                    timers.t1 * 64
                }
            }
            TimerType::TimerK => {
                if is_reliable {
                    Duration::from_secs(0)
                } else {
                    timers.t4
                }
            }
        }
    }
}

/// INVITE Client Transaction States (RFC 3261 Section 17.1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InviteClientState {
    /// Initial state - request sent
    Calling,
    /// Received provisional response (1xx)
    Proceeding,
    /// Received final response (2xx-6xx)
    Completed,
    /// Transaction terminated
    Terminated,
}

/// INVITE Server Transaction States (RFC 3261 Section 17.2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InviteServerState {
    /// Request received, provisional response sent
    Proceeding,
    /// Final response sent
    Completed,
    /// ACK received
    Confirmed,
    /// Transaction terminated
    Terminated,
}

/// Non-INVITE Client Transaction States (RFC 3261 Section 17.1.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonInviteClientState {
    /// Request sent, no response yet
    Trying,
    /// Received provisional response (1xx)
    Proceeding,
    /// Received final response (2xx-6xx)
    Completed,
    /// Transaction terminated
    Terminated,
}

/// Non-INVITE Server Transaction States (RFC 3261 Section 17.2.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonInviteServerState {
    /// Request received
    Trying,
    /// Provisional response sent
    Proceeding,
    /// Final response sent
    Completed,
    /// Transaction terminated
    Terminated,
}

/// Transaction type and state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// INVITE Client Transaction
    InviteClient(InviteClientState),
    /// INVITE Server Transaction
    InviteServer(InviteServerState),
    /// Non-INVITE Client Transaction
    NonInviteClient(NonInviteClientState),
    /// Non-INVITE Server Transaction
    NonInviteServer(NonInviteServerState),
}

impl TransactionState {
    /// Check if transaction is terminated
    pub fn is_terminated(&self) -> bool {
        matches!(
            self,
            TransactionState::InviteClient(InviteClientState::Terminated)
                | TransactionState::InviteServer(InviteServerState::Terminated)
                | TransactionState::NonInviteClient(NonInviteClientState::Terminated)
                | TransactionState::NonInviteServer(NonInviteServerState::Terminated)
        )
    }

    /// Get state name for debugging
    pub fn name(&self) -> &'static str {
        match self {
            TransactionState::InviteClient(InviteClientState::Calling) => "ICT-Calling",
            TransactionState::InviteClient(InviteClientState::Proceeding) => "ICT-Proceeding",
            TransactionState::InviteClient(InviteClientState::Completed) => "ICT-Completed",
            TransactionState::InviteClient(InviteClientState::Terminated) => "ICT-Terminated",
            TransactionState::InviteServer(InviteServerState::Proceeding) => "IST-Proceeding",
            TransactionState::InviteServer(InviteServerState::Completed) => "IST-Completed",
            TransactionState::InviteServer(InviteServerState::Confirmed) => "IST-Confirmed",
            TransactionState::InviteServer(InviteServerState::Terminated) => "IST-Terminated",
            TransactionState::NonInviteClient(NonInviteClientState::Trying) => "NICT-Trying",
            TransactionState::NonInviteClient(NonInviteClientState::Proceeding) => {
                "NICT-Proceeding"
            }
            TransactionState::NonInviteClient(NonInviteClientState::Completed) => "NICT-Completed",
            TransactionState::NonInviteClient(NonInviteClientState::Terminated) => {
                "NICT-Terminated"
            }
            TransactionState::NonInviteServer(NonInviteServerState::Trying) => "NIST-Trying",
            TransactionState::NonInviteServer(NonInviteServerState::Proceeding) => {
                "NIST-Proceeding"
            }
            TransactionState::NonInviteServer(NonInviteServerState::Completed) => "NIST-Completed",
            TransactionState::NonInviteServer(NonInviteServerState::Terminated) => {
                "NIST-Terminated"
            }
        }
    }
}

/// Active timer in a transaction
#[derive(Debug, Clone)]
pub struct ActiveTimer {
    pub timer_type: TimerType,
    pub expires_at: Instant,
    pub interval: Duration,
}

/// SIP Transaction
pub struct Transaction {
    /// Unique transaction ID
    pub id: TransactionId,
    /// Current state
    pub state: TransactionState,
    /// Original request
    pub request: SipRequest,
    /// Destination address
    pub destination: SocketAddr,
    /// Is transport reliable (TCP/TLS vs UDP)
    pub is_reliable: bool,
    /// Active timers
    pub timers: Vec<ActiveTimer>,
    /// Last response (for retransmission)
    pub last_response: Option<SipResponse>,
    /// Creation time
    pub created_at: Instant,
    /// SIP timer configuration
    pub sip_timers: SipTimers,
}

impl Transaction {
    /// Create a new INVITE client transaction
    pub fn new_invite_client(
        id: TransactionId,
        request: SipRequest,
        destination: SocketAddr,
        is_reliable: bool,
    ) -> Self {
        let sip_timers = SipTimers::default();
        let mut transaction = Self {
            id,
            state: TransactionState::InviteClient(InviteClientState::Calling),
            request,
            destination,
            is_reliable,
            timers: Vec::new(),
            last_response: None,
            created_at: Instant::now(),
            sip_timers,
        };

        // Start Timer A (request retransmit) and Timer B (timeout)
        if !is_reliable {
            transaction.start_timer(TimerType::TimerA);
        }
        transaction.start_timer(TimerType::TimerB);

        transaction
    }

    /// Create a new INVITE server transaction
    pub fn new_invite_server(
        id: TransactionId,
        request: SipRequest,
        destination: SocketAddr,
        is_reliable: bool,
    ) -> Self {
        Self {
            id,
            state: TransactionState::InviteServer(InviteServerState::Proceeding),
            request,
            destination,
            is_reliable,
            timers: Vec::new(),
            last_response: None,
            created_at: Instant::now(),
            sip_timers: SipTimers::default(),
        }
    }

    /// Create a new non-INVITE client transaction
    pub fn new_non_invite_client(
        id: TransactionId,
        request: SipRequest,
        destination: SocketAddr,
        is_reliable: bool,
    ) -> Self {
        let sip_timers = SipTimers::default();
        let mut transaction = Self {
            id,
            state: TransactionState::NonInviteClient(NonInviteClientState::Trying),
            request,
            destination,
            is_reliable,
            timers: Vec::new(),
            last_response: None,
            created_at: Instant::now(),
            sip_timers,
        };

        // Start Timer E (request retransmit) and Timer F (timeout)
        if !is_reliable {
            transaction.start_timer(TimerType::TimerE);
        }
        transaction.start_timer(TimerType::TimerF);

        transaction
    }

    /// Create a new non-INVITE server transaction
    pub fn new_non_invite_server(
        id: TransactionId,
        request: SipRequest,
        destination: SocketAddr,
        is_reliable: bool,
    ) -> Self {
        Self {
            id,
            state: TransactionState::NonInviteServer(NonInviteServerState::Trying),
            request,
            destination,
            is_reliable,
            timers: Vec::new(),
            last_response: None,
            created_at: Instant::now(),
            sip_timers: SipTimers::default(),
        }
    }

    /// Start a timer
    fn start_timer(&mut self, timer_type: TimerType) {
        let duration = timer_type.default_duration(&self.sip_timers, self.is_reliable);
        if duration.as_millis() > 0 {
            let timer = ActiveTimer {
                timer_type,
                expires_at: Instant::now() + duration,
                interval: duration,
            };
            self.timers.push(timer);
            debug!(
                "Started {:?} for transaction {} (expires in {:?})",
                timer_type, self.id.0, duration
            );
        }
    }

    /// Stop a timer
    fn stop_timer(&mut self, timer_type: TimerType) {
        self.timers.retain(|t| t.timer_type != timer_type);
        debug!("Stopped {:?} for transaction {}", timer_type, self.id.0);
    }

    /// Stop all timers
    fn stop_all_timers(&mut self) {
        self.timers.clear();
        debug!("Stopped all timers for transaction {}", self.id.0);
    }

    /// Process received response (for client transactions)
    pub fn process_response(&mut self, response: &SipResponse) -> Result<(), String> {
        let status = response.status_code();

        match &self.state {
            // INVITE Client Transaction
            TransactionState::InviteClient(state) => match state {
                InviteClientState::Calling | InviteClientState::Proceeding => {
                    if status >= 100 && status < 200 {
                        // Provisional response
                        self.state =
                            TransactionState::InviteClient(InviteClientState::Proceeding);
                        self.stop_timer(TimerType::TimerA); // Stop retransmitting
                    } else if status >= 200 && status < 300 {
                        // 2xx response - pass to TU, terminate transaction
                        self.state =
                            TransactionState::InviteClient(InviteClientState::Terminated);
                        self.stop_all_timers();
                    } else if status >= 300 {
                        // 3xx-6xx response
                        self.state =
                            TransactionState::InviteClient(InviteClientState::Completed);
                        self.stop_timer(TimerType::TimerA);
                        self.stop_timer(TimerType::TimerB);
                        self.start_timer(TimerType::TimerD);
                    }
                    Ok(())
                }
                InviteClientState::Completed => {
                    // Absorb retransmitted responses
                    Ok(())
                }
                InviteClientState::Terminated => Err("Transaction already terminated".to_string()),
            },

            // Non-INVITE Client Transaction
            TransactionState::NonInviteClient(state) => match state {
                NonInviteClientState::Trying | NonInviteClientState::Proceeding => {
                    if status >= 100 && status < 200 {
                        // Provisional response
                        self.state =
                            TransactionState::NonInviteClient(NonInviteClientState::Proceeding);
                    } else if status >= 200 {
                        // Final response
                        self.state =
                            TransactionState::NonInviteClient(NonInviteClientState::Completed);
                        self.stop_timer(TimerType::TimerE);
                        self.stop_timer(TimerType::TimerF);
                        self.start_timer(TimerType::TimerK);
                    }
                    Ok(())
                }
                NonInviteClientState::Completed => {
                    // Absorb retransmitted responses
                    Ok(())
                }
                NonInviteClientState::Terminated => {
                    Err("Transaction already terminated".to_string())
                }
            },

            _ => Err("Not a client transaction".to_string()),
        }
    }

    /// Process received ACK (for INVITE server transactions)
    pub fn process_ack(&mut self) -> Result<(), String> {
        match &self.state {
            TransactionState::InviteServer(InviteServerState::Completed) => {
                self.state = TransactionState::InviteServer(InviteServerState::Confirmed);
                self.stop_timer(TimerType::TimerG);
                self.stop_timer(TimerType::TimerH);
                self.start_timer(TimerType::TimerI);
                Ok(())
            }
            _ => Err("Invalid state for ACK processing".to_string()),
        }
    }

    /// Send response (for server transactions)
    pub fn send_response(&mut self, response: SipResponse) -> Result<(), String> {
        let status = response.status_code();
        self.last_response = Some(response);

        match &self.state {
            // INVITE Server Transaction
            TransactionState::InviteServer(state) => match state {
                InviteServerState::Proceeding => {
                    if status >= 200 && status < 300 {
                        // 2xx response - pass to transport, terminate
                        self.state =
                            TransactionState::InviteServer(InviteServerState::Terminated);
                        self.stop_all_timers();
                    } else if status >= 300 {
                        // 3xx-6xx response
                        self.state = TransactionState::InviteServer(InviteServerState::Completed);
                        if !self.is_reliable {
                            self.start_timer(TimerType::TimerG);
                        }
                        self.start_timer(TimerType::TimerH);
                    }
                    Ok(())
                }
                _ => Err("Invalid state for sending response".to_string()),
            },

            // Non-INVITE Server Transaction
            TransactionState::NonInviteServer(state) => match state {
                NonInviteServerState::Trying | NonInviteServerState::Proceeding => {
                    if status >= 100 && status < 200 {
                        // Provisional response
                        self.state =
                            TransactionState::NonInviteServer(NonInviteServerState::Proceeding);
                    } else if status >= 200 {
                        // Final response
                        self.state =
                            TransactionState::NonInviteServer(NonInviteServerState::Completed);
                        self.start_timer(TimerType::TimerJ);
                    }
                    Ok(())
                }
                _ => Err("Invalid state for sending response".to_string()),
            },

            _ => Err("Not a server transaction".to_string()),
        }
    }

    /// Handle timer expiration
    pub fn handle_timer_fired(&mut self, timer_type: TimerType) -> TransactionTimerAction {
        debug!(
            "Timer {:?} fired for transaction {} in state {}",
            timer_type,
            self.id.0,
            self.state.name()
        );

        match timer_type {
            TimerType::TimerA => {
                // Retransmit INVITE request
                if let TransactionState::InviteClient(InviteClientState::Calling) = self.state {
                    // Double the interval (exponential backoff)
                    if let Some(timer) = self.timers.iter_mut().find(|t| t.timer_type == TimerType::TimerA) {
                        timer.interval = std::cmp::min(timer.interval * 2, self.sip_timers.t2);
                        timer.expires_at = Instant::now() + timer.interval;
                    }
                    TransactionTimerAction::RetransmitRequest
                } else {
                    TransactionTimerAction::None
                }
            }

            TimerType::TimerB => {
                // INVITE client timeout
                self.state = TransactionState::InviteClient(InviteClientState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Timeout
            }

            TimerType::TimerD => {
                // INVITE client completed -> terminated
                self.state = TransactionState::InviteClient(InviteClientState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Terminate
            }

            TimerType::TimerE => {
                // Retransmit non-INVITE request
                if let TransactionState::NonInviteClient(NonInviteClientState::Trying) = self.state
                {
                    // Double the interval (exponential backoff)
                    if let Some(timer) = self.timers.iter_mut().find(|t| t.timer_type == TimerType::TimerE) {
                        timer.interval = std::cmp::min(timer.interval * 2, self.sip_timers.t2);
                        timer.expires_at = Instant::now() + timer.interval;
                    }
                    TransactionTimerAction::RetransmitRequest
                } else {
                    TransactionTimerAction::None
                }
            }

            TimerType::TimerF => {
                // Non-INVITE client timeout
                self.state = TransactionState::NonInviteClient(NonInviteClientState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Timeout
            }

            TimerType::TimerG => {
                // Retransmit INVITE response
                if let TransactionState::InviteServer(InviteServerState::Completed) = self.state {
                    // Double the interval
                    if let Some(timer) = self.timers.iter_mut().find(|t| t.timer_type == TimerType::TimerG) {
                        timer.interval = std::cmp::min(timer.interval * 2, self.sip_timers.t2);
                        timer.expires_at = Instant::now() + timer.interval;
                    }
                    TransactionTimerAction::RetransmitResponse
                } else {
                    TransactionTimerAction::None
                }
            }

            TimerType::TimerH => {
                // INVITE server timeout waiting for ACK
                self.state = TransactionState::InviteServer(InviteServerState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Timeout
            }

            TimerType::TimerI => {
                // INVITE server confirmed -> terminated
                self.state = TransactionState::InviteServer(InviteServerState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Terminate
            }

            TimerType::TimerJ => {
                // Non-INVITE server completed -> terminated
                self.state = TransactionState::NonInviteServer(NonInviteServerState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Terminate
            }

            TimerType::TimerK => {
                // Non-INVITE client completed -> terminated
                self.state = TransactionState::NonInviteClient(NonInviteClientState::Terminated);
                self.stop_all_timers();
                TransactionTimerAction::Terminate
            }
        }
    }

    /// Check for expired timers and return actions
    pub fn check_timers(&mut self) -> Vec<(TimerType, TransactionTimerAction)> {
        let now = Instant::now();
        let mut actions = Vec::new();

        // Find expired timers
        let expired: Vec<TimerType> = self
            .timers
            .iter()
            .filter(|t| t.expires_at <= now)
            .map(|t| t.timer_type)
            .collect();

        for timer_type in expired {
            let action = self.handle_timer_fired(timer_type);
            if action != TransactionTimerAction::None {
                actions.push((timer_type, action));
            }
        }

        actions
    }
}

/// Actions that should be taken when a timer fires
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionTimerAction {
    /// No action needed
    None,
    /// Retransmit the request
    RetransmitRequest,
    /// Retransmit the response
    RetransmitResponse,
    /// Transaction timed out
    Timeout,
    /// Terminate the transaction
    Terminate,
}

/// Transaction layer manager
/// Manages all active transactions and handles timer processing
pub struct TransactionLayer {
    /// Active transactions indexed by transaction ID
    transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
    /// SIP timer configuration
    sip_timers: SipTimers,
    /// Background timer task handle
    timer_task: Option<JoinHandle<()>>,
}

impl TransactionLayer {
    /// Create a new transaction layer
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            sip_timers: SipTimers::default(),
            timer_task: None,
        }
    }

    /// Start the transaction layer timer processing
    /// Returns a handle to the background timer task
    pub fn start(&mut self) -> &JoinHandle<()> {
        let transactions = self.transactions.clone();

        let handle = tokio::spawn(async move {
            info!("Transaction layer timer task started");

            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Process timers for all transactions
                let actions = {
                    let mut txns = transactions.write().await;
                    let mut all_actions = Vec::new();

                    for (id, txn) in txns.iter_mut() {
                        let timer_actions = txn.check_timers();
                        for (timer_type, action) in timer_actions {
                            all_actions.push((id.clone(), timer_type, action));
                        }
                    }

                    all_actions
                };

                // Log timer actions
                for (id, timer_type, action) in actions {
                    match action {
                        TransactionTimerAction::RetransmitRequest => {
                            debug!("Transaction {} timer {:?} fired: retransmit request", id.0, timer_type);
                        }
                        TransactionTimerAction::RetransmitResponse => {
                            debug!("Transaction {} timer {:?} fired: retransmit response", id.0, timer_type);
                        }
                        TransactionTimerAction::Timeout => {
                            warn!("Transaction {} timer {:?} fired: timeout", id.0, timer_type);
                        }
                        TransactionTimerAction::Terminate => {
                            debug!("Transaction {} timer {:?} fired: terminate", id.0, timer_type);
                        }
                        TransactionTimerAction::None => {}
                    }
                }

                // Cleanup terminated transactions
                {
                    let mut txns = transactions.write().await;
                    let terminated: Vec<TransactionId> = txns
                        .iter()
                        .filter(|(_, txn)| txn.state.is_terminated())
                        .map(|(id, _)| id.clone())
                        .collect();

                    for id in terminated {
                        debug!("Removing terminated transaction {}", id.0);
                        txns.remove(&id);
                    }
                }
            }
        });

        self.timer_task = Some(handle);
        self.timer_task.as_ref().unwrap()
    }

    /// Stop the transaction layer
    pub fn stop(&mut self) {
        if let Some(handle) = self.timer_task.take() {
            handle.abort();
            info!("Transaction layer timer task stopped");
        }
    }

    /// Create a new client transaction for an outgoing request
    pub async fn create_client_transaction(
        &self,
        request: SipRequest,
        destination: SocketAddr,
        is_reliable: bool,
    ) -> Result<TransactionId, String> {
        // Extract transaction ID from Via branch parameter
        let branch = extract_branch(request.headers())
            .ok_or_else(|| "No branch parameter in Via header".to_string())?;

        let txn_id = TransactionId::from_branch(&branch);

        // Create appropriate transaction type based on method
        let method = request.method()
            .ok_or_else(|| "No method in request".to_string())?;
        let transaction = if method.as_str() == "INVITE" {
            Transaction::new_invite_client(txn_id.clone(), request, destination, is_reliable)
        } else {
            Transaction::new_non_invite_client(txn_id.clone(), request, destination, is_reliable)
        };

        // Store transaction
        let mut txns = self.transactions.write().await;
        info!(
            "Created client transaction {} for {} request to {}",
            txn_id.0, method, destination
        );
        txns.insert(txn_id.clone(), transaction);

        Ok(txn_id)
    }

    /// Create a new server transaction for an incoming request
    pub async fn create_server_transaction(
        &self,
        request: SipRequest,
        source: SocketAddr,
        is_reliable: bool,
    ) -> Result<TransactionId, String> {
        // Extract transaction ID from Via branch parameter
        let branch = extract_branch(request.headers())
            .ok_or_else(|| "No branch parameter in Via header".to_string())?;

        let txn_id = TransactionId::from_branch(&branch);

        // Check if transaction already exists
        {
            let txns = self.transactions.read().await;
            if txns.contains_key(&txn_id) {
                return Ok(txn_id); // Transaction already exists (retransmission)
            }
        }

        // Create appropriate transaction type based on method
        let method = request.method()
            .ok_or_else(|| "No method in request".to_string())?;
        let transaction = if method.as_str() == "INVITE" {
            Transaction::new_invite_server(txn_id.clone(), request, source, is_reliable)
        } else {
            Transaction::new_non_invite_server(txn_id.clone(), request, source, is_reliable)
        };

        // Store transaction
        let mut txns = self.transactions.write().await;
        info!(
            "Created server transaction {} for {} request from {}",
            txn_id.0, method, source
        );
        txns.insert(txn_id.clone(), transaction);

        Ok(txn_id)
    }

    /// Process an incoming response (for client transactions)
    /// Returns the response if it should be passed to the transaction user
    pub async fn process_response(
        &self,
        response: SipResponse,
    ) -> Result<Option<SipResponse>, String> {
        // Extract transaction ID from Via branch parameter
        let branch = extract_branch(response.headers())
            .ok_or_else(|| "No branch parameter in Via header".to_string())?;

        let txn_id = TransactionId::from_branch(&branch);

        // Find and process transaction
        let mut txns = self.transactions.write().await;
        if let Some(txn) = txns.get_mut(&txn_id) {
            let old_state = txn.state;
            txn.process_response(&response)?;
            let new_state = txn.state;

            debug!(
                "Transaction {} processed response {}: {} -> {}",
                txn_id.0,
                response.status_code(),
                old_state.name(),
                new_state.name()
            );

            // Return response if state changed (not a retransmission)
            if old_state != new_state {
                Ok(Some(response))
            } else {
                Ok(None) // Retransmission, already processed
            }
        } else {
            warn!("No transaction found for response: {}", txn_id.0);
            Err(format!("No transaction found for ID: {}", txn_id.0))
        }
    }

    /// Process an incoming ACK (for INVITE server transactions)
    pub async fn process_ack(&self, request: SipRequest) -> Result<(), String> {
        // Extract transaction ID from Via branch parameter
        let branch = extract_branch(request.headers())
            .ok_or_else(|| "No branch parameter in Via header".to_string())?;

        let txn_id = TransactionId::from_branch(&branch);

        // Find and process transaction
        let mut txns = self.transactions.write().await;
        if let Some(txn) = txns.get_mut(&txn_id) {
            let old_state = txn.state;
            txn.process_ack()?;
            let new_state = txn.state;

            debug!(
                "Transaction {} processed ACK: {} -> {}",
                txn_id.0,
                old_state.name(),
                new_state.name()
            );

            Ok(())
        } else {
            warn!("No transaction found for ACK: {}", txn_id.0);
            Ok(()) // ACKs for 2xx responses don't match a transaction
        }
    }

    /// Send a response from a server transaction
    pub async fn send_response(
        &self,
        txn_id: &TransactionId,
        response: SipResponse,
    ) -> Result<(), String> {
        let mut txns = self.transactions.write().await;
        if let Some(txn) = txns.get_mut(txn_id) {
            let old_state = txn.state;
            txn.send_response(response.clone())?;
            let new_state = txn.state;

            debug!(
                "Transaction {} sent response {}: {} -> {}",
                txn_id.0,
                response.status_code(),
                old_state.name(),
                new_state.name()
            );

            Ok(())
        } else {
            Err(format!("Transaction not found: {}", txn_id.0))
        }
    }

    /// Get a transaction by ID (returns a clone)
    pub async fn get_transaction(&self, id: &TransactionId) -> Option<Transaction> {
        let txns = self.transactions.read().await;
        txns.get(id).cloned()
    }

    /// Get the request for a transaction
    pub async fn get_request(&self, id: &TransactionId) -> Option<SipRequest> {
        let txns = self.transactions.read().await;
        txns.get(id).map(|txn| txn.request.clone())
    }

    /// Get the last response for a transaction
    pub async fn get_last_response(&self, id: &TransactionId) -> Option<SipResponse> {
        let txns = self.transactions.read().await;
        txns.get(id).and_then(|txn| txn.last_response.clone())
    }

    /// Get the destination for a transaction
    pub async fn get_destination(&self, id: &TransactionId) -> Option<SocketAddr> {
        let txns = self.transactions.read().await;
        txns.get(id).map(|txn| txn.destination)
    }

    /// Check if a transaction exists
    pub async fn has_transaction(&self, id: &TransactionId) -> bool {
        let txns = self.transactions.read().await;
        txns.contains_key(id)
    }

    /// Get count of active transactions
    pub async fn transaction_count(&self) -> usize {
        let txns = self.transactions.read().await;
        txns.len()
    }

    /// Manually cleanup terminated transactions
    pub async fn cleanup_terminated(&self) -> usize {
        let mut txns = self.transactions.write().await;

        let terminated: Vec<TransactionId> = txns
            .iter()
            .filter(|(_, txn)| txn.state.is_terminated())
            .map(|(id, _)| id.clone())
            .collect();

        for id in &terminated {
            txns.remove(id);
        }

        let removed = terminated.len();
        if removed > 0 {
            info!("Cleaned up {} terminated transactions", removed);
        }

        removed
    }
}

impl Drop for TransactionLayer {
    fn drop(&mut self) {
        self.stop();
    }
}

// Clone implementation for Transaction (needed for get_transaction)
impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            state: self.state,
            request: self.request.clone(),
            destination: self.destination,
            is_reliable: self.is_reliable,
            timers: self.timers.clone(),
            last_response: self.last_response.clone(),
            created_at: self.created_at,
            sip_timers: self.sip_timers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_id() {
        let id1 = TransactionId::from_branch("z9hG4bK776asdhds");
        let id2 = TransactionId::from_branch("z9hG4bK776asdhds");
        assert_eq!(id1, id2);

        let id3 = TransactionId::generate();
        assert!(id3.0.starts_with("z9hG4bK"));
    }

    #[test]
    fn test_timer_durations() {
        let timers = SipTimers::default();

        // UDP timers
        assert_eq!(
            TimerType::TimerA.default_duration(&timers, false),
            Duration::from_millis(500)
        );
        assert_eq!(
            TimerType::TimerB.default_duration(&timers, false),
            Duration::from_secs(32)
        );

        // TCP timers (some should be 0)
        assert_eq!(
            TimerType::TimerD.default_duration(&timers, true),
            Duration::from_secs(0)
        );
        assert_eq!(
            TimerType::TimerI.default_duration(&timers, true),
            Duration::from_secs(0)
        );
    }

    #[test]
    fn test_transaction_state_names() {
        let state = TransactionState::InviteClient(InviteClientState::Calling);
        assert_eq!(state.name(), "ICT-Calling");

        let state = TransactionState::NonInviteServer(NonInviteServerState::Completed);
        assert_eq!(state.name(), "NIST-Completed");
    }

    #[test]
    fn test_transaction_state_terminated() {
        let state = TransactionState::InviteClient(InviteClientState::Terminated);
        assert!(state.is_terminated());

        let state = TransactionState::InviteClient(InviteClientState::Calling);
        assert!(!state.is_terminated());
    }

    // Helper function to create a test request
    fn create_test_request(method: &str) -> SipRequest {
        let request_str = format!(
            "{} sip:bob@example.com SIP/2.0\r\nCall-ID: test-123\r\nCSeq: 1 {}\r\n\r\n",
            method, method
        );
        SipRequest::parse(request_str.as_bytes()).unwrap()
    }

    // Helper function to create a test response
    fn create_test_response(status: u16) -> SipResponse {
        let response_str = format!("SIP/2.0 {} OK\r\nCall-ID: test-123\r\nCSeq: 1 INVITE\r\n\r\n", status);
        SipResponse::parse(response_str.as_bytes()).unwrap()
    }

    #[test]
    fn test_invite_client_transaction_provisional_response() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_invite_client(id, request, dest, false);

        // Initially in Calling state
        assert!(matches!(
            txn.state,
            TransactionState::InviteClient(InviteClientState::Calling)
        ));

        // Receive 180 Ringing
        let response = create_test_response(180);
        txn.process_response(&response).unwrap();

        // Should transition to Proceeding
        assert!(matches!(
            txn.state,
            TransactionState::InviteClient(InviteClientState::Proceeding)
        ));
    }

    #[test]
    fn test_invite_client_transaction_final_response() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-2");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_invite_client(id, request, dest, false);

        // Receive 200 OK
        let response = create_test_response(200);
        txn.process_response(&response).unwrap();

        // Should terminate
        assert!(matches!(
            txn.state,
            TransactionState::InviteClient(InviteClientState::Terminated)
        ));
    }

    #[test]
    fn test_invite_client_transaction_error_response() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-3");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_invite_client(id, request, dest, false);

        // Receive 486 Busy Here
        let response = create_test_response(486);
        txn.process_response(&response).unwrap();

        // Should transition to Completed
        assert!(matches!(
            txn.state,
            TransactionState::InviteClient(InviteClientState::Completed)
        ));

        // Timer D should be started (wait for retransmits)
        assert!(txn.timers.iter().any(|t| t.timer_type == TimerType::TimerD));
    }

    #[test]
    fn test_non_invite_client_transaction() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-4");
        let request = create_test_request("REGISTER");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_non_invite_client(id, request, dest, false);

        // Initially in Trying state
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteClient(NonInviteClientState::Trying)
        ));

        // Receive 200 OK
        let response = create_test_response(200);
        txn.process_response(&response).unwrap();

        // Should transition to Completed
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteClient(NonInviteClientState::Completed)
        ));

        // Timer K should be started
        assert!(txn.timers.iter().any(|t| t.timer_type == TimerType::TimerK));
    }

    #[test]
    fn test_invite_server_transaction_ack() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-5");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_invite_server(id, request, dest, false);

        // Send error response (486)
        let response = create_test_response(486);
        txn.send_response(response).unwrap();

        // Should be in Completed state
        assert!(matches!(
            txn.state,
            TransactionState::InviteServer(InviteServerState::Completed)
        ));

        // Process ACK
        txn.process_ack().unwrap();

        // Should be in Confirmed state
        assert!(matches!(
            txn.state,
            TransactionState::InviteServer(InviteServerState::Confirmed)
        ));

        // Timer I should be started
        assert!(txn.timers.iter().any(|t| t.timer_type == TimerType::TimerI));
    }

    #[test]
    fn test_non_invite_server_transaction() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-6");
        let request = create_test_request("REGISTER");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_non_invite_server(id, request, dest, false);

        // Initially in Trying state
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteServer(NonInviteServerState::Trying)
        ));

        // Send 200 OK
        let response = create_test_response(200);
        txn.send_response(response).unwrap();

        // Should be in Completed state
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteServer(NonInviteServerState::Completed)
        ));

        // Timer J should be started
        assert!(txn.timers.iter().any(|t| t.timer_type == TimerType::TimerJ));
    }

    #[test]
    fn test_timer_expiration_actions() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-branch-7");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let mut txn = Transaction::new_invite_client(id, request, dest, false);

        // Fire Timer A (should retransmit)
        let action = txn.handle_timer_fired(TimerType::TimerA);
        assert_eq!(action, TransactionTimerAction::RetransmitRequest);

        // Fire Timer B (should timeout)
        let action = txn.handle_timer_fired(TimerType::TimerB);
        assert_eq!(action, TransactionTimerAction::Timeout);
        assert!(txn.state.is_terminated());
    }

    #[test]
    fn test_tcp_no_retransmit_timers() {
        use std::net::Ipv4Addr;

        let id = TransactionId::from_branch("test-tcp");
        let request = create_test_request("INVITE");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        // TCP (reliable transport)
        let txn = Transaction::new_invite_client(id, request, dest, true);

        // Timer A should not be started for TCP
        assert!(!txn.timers.iter().any(|t| t.timer_type == TimerType::TimerA));

        // Timer B should still be started
        assert!(txn.timers.iter().any(|t| t.timer_type == TimerType::TimerB));
    }

    // Helper to create request with Via branch
    fn create_request_with_branch(method: &str, branch: &str) -> SipRequest {
        let request_str = format!(
            "{} sip:bob@example.com SIP/2.0\r\n\
            Via: SIP/2.0/UDP 127.0.0.1:5060;branch={}\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>\r\n\
            Call-ID: test-123\r\n\
            CSeq: 1 {}\r\n\
            Contact: <sip:alice@127.0.0.1:5060>\r\n\
            Content-Length: 0\r\n\
            \r\n",
            method, branch, method
        );
        SipRequest::parse(request_str.as_bytes()).unwrap()
    }

    // Helper to create response with Via branch
    fn create_response_with_branch(status: u16, branch: &str) -> SipResponse {
        let response_str = format!(
            "SIP/2.0 {} OK\r\n\
            Via: SIP/2.0/UDP 127.0.0.1:5060;branch={}\r\n\
            From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
            To: Bob <sip:bob@example.com>;tag=987654321\r\n\
            Call-ID: test-123\r\n\
            CSeq: 1 INVITE\r\n\
            Content-Length: 0\r\n\
            \r\n",
            status, branch
        );
        SipResponse::parse(response_str.as_bytes()).unwrap()
    }

    #[tokio::test]
    async fn test_transaction_layer_create_client_transaction() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("INVITE", "z9hG4bK-test-1");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let txn_id = layer
            .create_client_transaction(request, dest, false)
            .await
            .unwrap();

        assert_eq!(txn_id.0, "z9hG4bK-test-1");
        assert!(layer.has_transaction(&txn_id).await);
        assert_eq!(layer.transaction_count().await, 1);
    }

    #[tokio::test]
    async fn test_transaction_layer_create_server_transaction() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("INVITE", "z9hG4bK-test-2");
        let source = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let txn_id = layer
            .create_server_transaction(request.clone(), source, false)
            .await
            .unwrap();

        assert_eq!(txn_id.0, "z9hG4bK-test-2");
        assert!(layer.has_transaction(&txn_id).await);

        // Creating again should return same ID (retransmission)
        let txn_id2 = layer
            .create_server_transaction(request, source, false)
            .await
            .unwrap();

        assert_eq!(txn_id, txn_id2);
        assert_eq!(layer.transaction_count().await, 1);
    }

    #[tokio::test]
    async fn test_transaction_layer_process_response() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("INVITE", "z9hG4bK-test-3");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let _txn_id = layer
            .create_client_transaction(request, dest, false)
            .await
            .unwrap();

        // Send 180 Ringing
        let response = create_response_with_branch(180, "z9hG4bK-test-3");
        let result = layer.process_response(response.clone()).await.unwrap();
        assert!(result.is_some()); // New response, should be passed up

        // Send same 180 again (retransmission)
        let result = layer.process_response(response).await.unwrap();
        assert!(result.is_none()); // Retransmission, should be absorbed

        // Send 200 OK
        let response = create_response_with_branch(200, "z9hG4bK-test-3");
        let result = layer.process_response(response).await.unwrap();
        assert!(result.is_some()); // New response
    }

    #[tokio::test]
    async fn test_transaction_layer_process_ack() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("INVITE", "z9hG4bK-test-4");
        let source = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let txn_id = layer
            .create_server_transaction(request, source, false)
            .await
            .unwrap();

        // Send 486 error response
        let response = create_response_with_branch(486, "z9hG4bK-test-4");
        layer.send_response(&txn_id, response).await.unwrap();

        // Process ACK
        let ack = create_request_with_branch("ACK", "z9hG4bK-test-4");
        layer.process_ack(ack).await.unwrap();

        // Transaction should now be in Confirmed state
        let txn = layer.get_transaction(&txn_id).await.unwrap();
        assert!(matches!(
            txn.state,
            TransactionState::InviteServer(InviteServerState::Confirmed)
        ));
    }

    #[tokio::test]
    async fn test_transaction_layer_get_methods() {
        use std::net::Ipv4Addr;
        use crate::infrastructure::protocols::sip::SipMethod;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("REGISTER", "z9hG4bK-test-5");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        let txn_id = layer
            .create_client_transaction(request.clone(), dest, false)
            .await
            .unwrap();

        // Test get_request
        let req = layer.get_request(&txn_id).await.unwrap();
        assert_eq!(req.method(), Some(SipMethod::Register));

        // Test get_destination
        let destination = layer.get_destination(&txn_id).await.unwrap();
        assert_eq!(destination, dest);

        // Test get_transaction
        let txn = layer.get_transaction(&txn_id).await.unwrap();
        assert_eq!(txn.id, txn_id);

        // Test get_last_response (should be None for client transaction without response)
        let response = layer.get_last_response(&txn_id).await;
        assert!(response.is_none());
    }

    #[tokio::test]
    async fn test_transaction_layer_cleanup_terminated() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        // Create multiple transactions
        let req1 = create_request_with_branch("INVITE", "z9hG4bK-test-6");
        let txn_id1 = layer
            .create_client_transaction(req1, dest, false)
            .await
            .unwrap();

        let req2 = create_request_with_branch("REGISTER", "z9hG4bK-test-7");
        let txn_id2 = layer
            .create_client_transaction(req2, dest, false)
            .await
            .unwrap();

        assert_eq!(layer.transaction_count().await, 2);

        // Terminate first transaction
        let response = create_response_with_branch(200, "z9hG4bK-test-6");
        layer.process_response(response).await.unwrap();

        // First transaction should be terminated
        let txn1 = layer.get_transaction(&txn_id1).await.unwrap();
        assert!(txn1.state.is_terminated());

        // Cleanup
        let removed = layer.cleanup_terminated().await;
        assert_eq!(removed, 1);
        assert_eq!(layer.transaction_count().await, 1);

        // Only second transaction should remain
        assert!(!layer.has_transaction(&txn_id1).await);
        assert!(layer.has_transaction(&txn_id2).await);
    }

    #[tokio::test]
    async fn test_transaction_layer_non_invite_flow() {
        use std::net::Ipv4Addr;

        let layer = TransactionLayer::new();
        let request = create_request_with_branch("REGISTER", "z9hG4bK-test-8");
        let dest = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060);

        // Create client transaction
        let txn_id = layer
            .create_client_transaction(request, dest, false)
            .await
            .unwrap();

        // Should be in Trying state
        let txn = layer.get_transaction(&txn_id).await.unwrap();
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteClient(NonInviteClientState::Trying)
        ));

        // Receive 200 OK
        let response = create_response_with_branch(200, "z9hG4bK-test-8");
        layer.process_response(response).await.unwrap();

        // Should be in Completed state
        let txn = layer.get_transaction(&txn_id).await.unwrap();
        assert!(matches!(
            txn.state,
            TransactionState::NonInviteClient(NonInviteClientState::Completed)
        ));
    }
}

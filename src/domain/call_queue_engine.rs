/// Call Queue Engine for managing queued calls and agent distribution
use crate::domain::call_queue::*;
use crate::domain::audio::{AudioFileManager, StreamingAudioPlayer, SequenceBuilder, PlaybackOptions};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rand::Rng;

/// Call queue engine error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueEngineError {
    /// Queue is full
    QueueFull,
    /// Queue not found
    QueueNotFound,
    /// No available agents
    NoAvailableAgents,
    /// Call not found in queue
    CallNotFound,
    /// Member not found
    MemberNotFound,
    /// Invalid operation
    InvalidOperation(String),
}

impl std::fmt::Display for QueueEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "Queue is full"),
            Self::QueueNotFound => write!(f, "Queue not found"),
            Self::NoAvailableAgents => write!(f, "No available agents"),
            Self::CallNotFound => write!(f, "Call not found"),
            Self::MemberNotFound => write!(f, "Member not found"),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for QueueEngineError {}

/// Queue statistics for monitoring
#[derive(Debug, Clone)]
pub struct QueueStatistics {
    /// Total calls received
    pub total_calls: u64,
    /// Calls currently waiting
    pub calls_waiting: usize,
    /// Calls currently being handled
    pub calls_active: usize,
    /// Total calls answered
    pub calls_answered: u64,
    /// Total calls abandoned (caller hung up)
    pub calls_abandoned: u64,
    /// Total calls overflowed
    pub calls_overflowed: u64,
    /// Average wait time
    pub avg_wait_time: Duration,
    /// Longest wait time
    pub longest_wait_time: Duration,
    /// Service level (% answered within threshold)
    pub service_level: f64,
    /// Service level threshold (e.g., 20 seconds)
    pub service_level_threshold: Duration,
}

impl Default for QueueStatistics {
    fn default() -> Self {
        Self {
            total_calls: 0,
            calls_waiting: 0,
            calls_active: 0,
            calls_answered: 0,
            calls_abandoned: 0,
            calls_overflowed: 0,
            avg_wait_time: Duration::from_secs(0),
            longest_wait_time: Duration::from_secs(0),
            service_level: 0.0,
            service_level_threshold: Duration::from_secs(20),
        }
    }
}

impl QueueStatistics {
    /// Calculate service level percentage
    pub fn calculate_service_level(&mut self, answered_within_threshold: u64) {
        if self.calls_answered > 0 {
            self.service_level = (answered_within_threshold as f64 / self.calls_answered as f64) * 100.0;
        }
    }
}

/// Active queue session
struct QueueSession {
    /// Queue configuration
    queue: CallQueue,
    /// Queue members
    members: HashMap<Uuid, QueueMember>,
    /// Waiting calls
    waiting_calls: VecDeque<QueuedCall>,
    /// Active calls (call_id -> agent_id)
    active_calls: HashMap<String, Uuid>,
    /// Round-robin position for round-robin strategy
    round_robin_position: usize,
    /// Statistics
    statistics: QueueStatistics,
    /// Answered calls within threshold (for SLA)
    answered_within_threshold: u64,
}

impl QueueSession {
    fn new(queue: CallQueue) -> Self {
        Self {
            queue,
            members: HashMap::new(),
            waiting_calls: VecDeque::new(),
            active_calls: HashMap::new(),
            round_robin_position: 0,
            statistics: QueueStatistics::default(),
            answered_within_threshold: 0,
        }
    }

    /// Get available members
    fn available_members(&self) -> Vec<&QueueMember> {
        self.members
            .values()
            .filter(|m| m.is_available())
            .collect()
    }

    /// Select next agent based on strategy
    fn select_agent(&mut self) -> Option<Uuid> {
        let available = self.available_members();
        if available.is_empty() {
            return None;
        }

        match self.queue.strategy {
            QueueStrategy::RingAll => {
                // Return all available agents (we'll return first, caller handles ringing all)
                available.first().map(|m| m.id)
            }
            QueueStrategy::Linear => {
                // Ring in order by join time
                let mut sorted = available.clone();
                sorted.sort_by_key(|m| m.joined_at);
                sorted.first().map(|m| m.id)
            }
            QueueStrategy::LeastRecent => {
                // Agent with oldest last_call_time
                let mut sorted = available.clone();
                sorted.sort_by_key(|m| m.last_call_time.unwrap_or(DateTime::<Utc>::MIN_UTC));
                sorted.first().map(|m| m.id)
            }
            QueueStrategy::FewestCalls => {
                // Agent with lowest total_calls
                let mut sorted = available.clone();
                sorted.sort_by_key(|m| m.total_calls);
                sorted.first().map(|m| m.id)
            }
            QueueStrategy::LeastTalkTime => {
                // Agent with lowest total_talk_time
                let mut sorted = available.clone();
                sorted.sort_by_key(|m| m.total_talk_time);
                sorted.first().map(|m| m.id)
            }
            QueueStrategy::Random => {
                // Random selection
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..available.len());
                available.get(index).map(|m| m.id)
            }
            QueueStrategy::RoundRobin => {
                // Round-robin selection
                if available.is_empty() {
                    return None;
                }

                let mut sorted = available.clone();
                sorted.sort_by_key(|m| m.id); // Consistent ordering

                let agent = sorted.get(self.round_robin_position % sorted.len()).map(|m| m.id);
                self.round_robin_position += 1;
                agent
            }
        }
    }

    /// Update queue positions
    fn update_positions(&mut self) {
        for (index, call) in self.waiting_calls.iter_mut().enumerate() {
            call.position = index + 1;
            call.update_wait_time();
        }
    }

    /// Check if queue is full
    fn is_full(&self) -> bool {
        self.waiting_calls.len() >= self.queue.max_queue_size
    }

    /// Get average wait time
    fn calculate_avg_wait_time(&self) -> Duration {
        if self.waiting_calls.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.waiting_calls
            .iter()
            .map(|c| c.wait_time)
            .sum();

        total / self.waiting_calls.len() as u32
    }
}

/// Call Queue Engine
pub struct CallQueueEngine {
    /// Active queue sessions
    sessions: Arc<Mutex<HashMap<Uuid, QueueSession>>>,
    /// Audio file manager for announcements
    audio_manager: Option<Arc<AudioFileManager>>,
}

impl CallQueueEngine {
    /// Create new call queue engine
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            audio_manager: None,
        }
    }

    /// Set audio manager for announcements
    pub fn set_audio_manager(&mut self, manager: Arc<AudioFileManager>) {
        self.audio_manager = Some(manager);
    }

    /// Start a queue session
    pub fn start_queue(&self, queue: CallQueue) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(queue.id, QueueSession::new(queue));
    }

    /// Stop a queue session
    pub fn stop_queue(&self, queue_id: Uuid) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(&queue_id);
    }

    /// Add member to queue
    pub fn add_member(&self, queue_id: Uuid, member: QueueMember) -> Result<(), QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        session.members.insert(member.id, member);
        Ok(())
    }

    /// Remove member from queue
    pub fn remove_member(&self, queue_id: Uuid, member_id: Uuid) -> Result<(), QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        session.members.remove(&member_id);
        Ok(())
    }

    /// Enqueue a call
    pub fn enqueue_call(
        &self,
        queue_id: Uuid,
        call_id: String,
        caller: String,
        caller_name: Option<String>,
    ) -> Result<QueuedCall, QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        // Check if queue is full
        if session.is_full() {
            session.statistics.calls_overflowed += 1;
            return Err(QueueEngineError::QueueFull);
        }

        // Create queued call
        let mut queued_call = QueuedCall::new(call_id, caller, queue_id);
        if let Some(name) = caller_name {
            queued_call.caller_name = Some(name);
        }

        // Add to queue
        session.waiting_calls.push_back(queued_call.clone());
        session.statistics.total_calls += 1;
        session.update_positions();

        Ok(queued_call)
    }

    /// Get next agent for a call
    pub fn get_next_agent(&self, queue_id: Uuid) -> Result<Option<QueueMember>, QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        let agent_id = session.select_agent();
        Ok(agent_id.and_then(|id| session.members.get(&id).cloned()))
    }

    /// Connect call to agent
    pub fn connect_call(
        &self,
        queue_id: Uuid,
        call_id: &str,
        agent_id: Uuid,
    ) -> Result<(), QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        // Remove from waiting queue
        let call_index = session
            .waiting_calls
            .iter()
            .position(|c| c.call_id == call_id)
            .ok_or(QueueEngineError::CallNotFound)?;

        let queued_call = session.waiting_calls.remove(call_index).unwrap();

        // Update statistics
        session.statistics.calls_answered += 1;
        session.statistics.calls_active += 1;

        // Check if answered within SLA threshold
        if queued_call.wait_time <= session.statistics.service_level_threshold {
            session.answered_within_threshold += 1;
        }

        // Update average wait time
        if session.statistics.calls_answered > 0 {
            let total_wait = session.statistics.avg_wait_time.as_secs() * (session.statistics.calls_answered - 1)
                + queued_call.wait_time.as_secs();
            session.statistics.avg_wait_time = Duration::from_secs(total_wait / session.statistics.calls_answered);
        }

        // Update longest wait time
        if queued_call.wait_time > session.statistics.longest_wait_time {
            session.statistics.longest_wait_time = queued_call.wait_time;
        }

        // Mark agent as busy
        if let Some(agent) = session.members.get_mut(&agent_id) {
            agent.mark_busy();
        }

        // Track active call
        session.active_calls.insert(call_id.to_string(), agent_id);
        session.update_positions();

        // Recalculate service level
        session.statistics.calculate_service_level(session.answered_within_threshold);

        Ok(())
    }

    /// End call
    pub fn end_call(
        &self,
        queue_id: Uuid,
        call_id: &str,
        talk_time: Duration,
    ) -> Result<(), QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        // Remove from active calls
        let agent_id = session
            .active_calls
            .remove(call_id)
            .ok_or(QueueEngineError::CallNotFound)?;

        // Update agent statistics
        if let Some(agent) = session.members.get_mut(&agent_id) {
            agent.record_answered(talk_time);
            agent.mark_available();
        }

        session.statistics.calls_active = session.statistics.calls_active.saturating_sub(1);

        Ok(())
    }

    /// Abandon call (caller hung up while waiting)
    pub fn abandon_call(&self, queue_id: Uuid, call_id: &str) -> Result<(), QueueEngineError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        // Remove from waiting queue
        let call_index = session
            .waiting_calls
            .iter()
            .position(|c| c.call_id == call_id)
            .ok_or(QueueEngineError::CallNotFound)?;

        session.waiting_calls.remove(call_index);
        session.statistics.calls_abandoned += 1;
        session.update_positions();

        Ok(())
    }

    /// Get queue statistics
    pub fn get_statistics(&self, queue_id: Uuid) -> Result<QueueStatistics, QueueEngineError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        let mut stats = session.statistics.clone();
        stats.calls_waiting = session.waiting_calls.len();
        stats.avg_wait_time = session.calculate_avg_wait_time();

        Ok(stats)
    }

    /// Get queue position for a call
    pub fn get_position(&self, queue_id: Uuid, call_id: &str) -> Result<usize, QueueEngineError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        session
            .waiting_calls
            .iter()
            .find(|c| c.call_id == call_id)
            .map(|c| c.position)
            .ok_or(QueueEngineError::CallNotFound)
    }

    /// Get waiting calls
    pub fn get_waiting_calls(&self, queue_id: Uuid) -> Result<Vec<QueuedCall>, QueueEngineError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        Ok(session.waiting_calls.iter().cloned().collect())
    }

    /// Get available agent count
    pub fn get_available_agents(&self, queue_id: Uuid) -> Result<usize, QueueEngineError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        Ok(session.available_members().len())
    }

    /// Create music on hold player for a queue
    pub fn create_moh_player(&self, queue_id: Uuid) -> Result<Option<StreamingAudioPlayer>, QueueEngineError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(&queue_id)
            .ok_or(QueueEngineError::QueueNotFound)?;

        if let Some(ref moh_file) = session.queue.music_on_hold {
            if let Some(ref audio_mgr) = self.audio_manager {
                if let Some(audio) = audio_mgr.get_default(moh_file) {
                    let player = StreamingAudioPlayer::with_options(PlaybackOptions {
                        frame_duration_ms: 20,
                        loop_playback: true,
                        allow_interrupt: false,
                    });
                    player.load(audio);
                    player.play();
                    return Ok(Some(player));
                }
            }
        }

        Ok(None)
    }
}

impl Default for CallQueueEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_queue() -> CallQueue {
        CallQueue {
            id: Uuid::new_v4(),
            name: "Support Queue".to_string(),
            extension: "5000".to_string(),
            strategy: QueueStrategy::RoundRobin,
            max_wait_time: Duration::from_secs(300),
            max_queue_size: 10,
            ring_timeout: Duration::from_secs(20),
            retry_delay: Duration::from_secs(5),
            max_retries: 3,
            wrap_up_time: Duration::from_secs(10),
            announce_position: true,
            announce_wait_time: true,
            music_on_hold: Some("moh_default".to_string()),
            periodic_announce: None,
            periodic_announce_frequency: Duration::from_secs(30),
            overflow_queue_id: None,
            overflow_action: OverflowAction::Voicemail,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let stats = engine.get_statistics(queue.id).unwrap();
        assert_eq!(stats.total_calls, 0);
    }

    #[test]
    fn test_enqueue_call() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let result = engine.enqueue_call(
            queue.id,
            "call-123".to_string(),
            "sip:alice@example.com".to_string(),
            Some("Alice".to_string()),
        );

        assert!(result.is_ok());
        let queued = result.unwrap();
        assert_eq!(queued.caller, "sip:alice@example.com");
        assert_eq!(queued.position, 1);
    }

    #[test]
    fn test_queue_full() {
        let engine = CallQueueEngine::new();
        let mut queue = create_test_queue();
        queue.max_queue_size = 2;
        engine.start_queue(queue.clone());

        // Fill the queue
        engine.enqueue_call(queue.id, "call-1".to_string(), "caller1".to_string(), None).unwrap();
        engine.enqueue_call(queue.id, "call-2".to_string(), "caller2".to_string(), None).unwrap();

        // Try to add one more
        let result = engine.enqueue_call(queue.id, "call-3".to_string(), "caller3".to_string(), None);
        assert_eq!(result, Err(QueueEngineError::QueueFull));
    }

    #[test]
    fn test_add_remove_member() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let member = QueueMember::new(1, "agent1".to_string(), "1001".to_string());
        let member_id = member.id;

        engine.add_member(queue.id, member).unwrap();
        assert_eq!(engine.get_available_agents(queue.id).unwrap(), 1);

        engine.remove_member(queue.id, member_id).unwrap();
        assert_eq!(engine.get_available_agents(queue.id).unwrap(), 0);
    }

    #[test]
    fn test_connect_call() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let member = QueueMember::new(1, "agent1".to_string(), "1001".to_string());
        let member_id = member.id;
        engine.add_member(queue.id, member).unwrap();

        engine.enqueue_call(queue.id, "call-123".to_string(), "caller".to_string(), None).unwrap();

        engine.connect_call(queue.id, "call-123", member_id).unwrap();

        let stats = engine.get_statistics(queue.id).unwrap();
        assert_eq!(stats.calls_answered, 1);
        assert_eq!(stats.calls_active, 1);
        assert_eq!(stats.calls_waiting, 0);
    }

    #[test]
    fn test_abandon_call() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        engine.enqueue_call(queue.id, "call-123".to_string(), "caller".to_string(), None).unwrap();
        engine.abandon_call(queue.id, "call-123").unwrap();

        let stats = engine.get_statistics(queue.id).unwrap();
        assert_eq!(stats.calls_abandoned, 1);
        assert_eq!(stats.calls_waiting, 0);
    }

    #[test]
    fn test_end_call() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let member = QueueMember::new(1, "agent1".to_string(), "1001".to_string());
        let member_id = member.id;
        engine.add_member(queue.id, member).unwrap();

        engine.enqueue_call(queue.id, "call-123".to_string(), "caller".to_string(), None).unwrap();
        engine.connect_call(queue.id, "call-123", member_id).unwrap();

        let talk_time = Duration::from_secs(60);
        engine.end_call(queue.id, "call-123", talk_time).unwrap();

        let stats = engine.get_statistics(queue.id).unwrap();
        assert_eq!(stats.calls_active, 0);
    }

    #[test]
    fn test_get_next_agent_round_robin() {
        let engine = CallQueueEngine::new();
        let queue = create_test_queue();
        engine.start_queue(queue.clone());

        let member1 = QueueMember::new(1, "agent1".to_string(), "1001".to_string());
        let member2 = QueueMember::new(2, "agent2".to_string(), "1002".to_string());
        let id1 = member1.id;
        let id2 = member2.id;

        engine.add_member(queue.id, member1).unwrap();
        engine.add_member(queue.id, member2).unwrap();

        let agent1 = engine.get_next_agent(queue.id).unwrap().unwrap();
        let agent2 = engine.get_next_agent(queue.id).unwrap().unwrap();

        // Should alternate
        assert_ne!(agent1.id, agent2.id);
    }
}

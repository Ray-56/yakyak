/// Call Queue and ACD (Automatic Call Distribution) domain models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use uuid::Uuid;

/// Queue strategy for distributing calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueStrategy {
    /// Ring all available agents simultaneously
    RingAll,
    /// Ring agents in order, one at a time
    Linear,
    /// Ring agent with least recent call
    LeastRecent,
    /// Ring agent with fewest total calls
    FewestCalls,
    /// Ring agent with lowest talk time
    LeastTalkTime,
    /// Random agent selection
    Random,
    /// Round-robin distribution
    RoundRobin,
}

/// Agent status in queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is available to take calls
    Available,
    /// Agent is on a call
    Busy,
    /// Agent is in after-call work
    AfterCallWork,
    /// Agent paused themselves
    Paused,
    /// Agent logged out
    LoggedOut,
}

/// Queue member (agent) in a call queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMember {
    pub id: Uuid,
    pub user_id: i32,
    pub username: String,
    pub extension: String,
    pub status: AgentStatus,
    pub penalty: u32,
    pub paused: bool,
    pub paused_reason: Option<String>,
    pub last_call_time: Option<DateTime<Utc>>,
    pub total_calls: u64,
    pub answered_calls: u64,
    pub missed_calls: u64,
    pub total_talk_time: Duration,
    pub joined_at: DateTime<Utc>,
}

impl QueueMember {
    pub fn new(user_id: i32, username: String, extension: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            username,
            extension,
            status: AgentStatus::Available,
            penalty: 0,
            paused: false,
            paused_reason: None,
            last_call_time: None,
            total_calls: 0,
            answered_calls: 0,
            missed_calls: 0,
            total_talk_time: Duration::from_secs(0),
            joined_at: Utc::now(),
        }
    }

    /// Check if agent is available
    pub fn is_available(&self) -> bool {
        self.status == AgentStatus::Available && !self.paused
    }

    /// Mark agent as busy
    pub fn mark_busy(&mut self) {
        self.status = AgentStatus::Busy;
    }

    /// Mark agent as available
    pub fn mark_available(&mut self) {
        self.status = AgentStatus::Available;
    }

    /// Pause agent
    pub fn pause(&mut self, reason: Option<String>) {
        self.paused = true;
        self.paused_reason = reason;
    }

    /// Unpause agent
    pub fn unpause(&mut self) {
        self.paused = false;
        self.paused_reason = None;
    }

    /// Record call answered
    pub fn record_answered(&mut self, talk_time: Duration) {
        self.total_calls += 1;
        self.answered_calls += 1;
        self.total_talk_time += talk_time;
        self.last_call_time = Some(Utc::now());
    }

    /// Record call missed
    pub fn record_missed(&mut self) {
        self.total_calls += 1;
        self.missed_calls += 1;
    }
}

/// Queued call waiting for an agent
#[derive(Debug, Clone)]
pub struct QueuedCall {
    pub id: Uuid,
    pub call_id: String,
    pub caller: String,
    pub caller_name: Option<String>,
    pub queue_id: Uuid,
    pub enqueued_at: DateTime<Utc>,
    pub position: usize,
    pub wait_time: Duration,
    pub priority: u32,
}

impl QueuedCall {
    pub fn new(call_id: String, caller: String, queue_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            call_id,
            caller,
            caller_name: None,
            queue_id,
            enqueued_at: Utc::now(),
            position: 0,
            wait_time: Duration::from_secs(0),
            priority: 0,
        }
    }

    /// Update wait time
    pub fn update_wait_time(&mut self) {
        self.wait_time = (Utc::now() - self.enqueued_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
    }
}

/// Call queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallQueue {
    pub id: Uuid,
    pub name: String,
    pub extension: String,
    pub strategy: QueueStrategy,
    pub max_wait_time: Duration,
    pub max_queue_size: usize,
    pub ring_timeout: Duration,
    pub retry_delay: Duration,
    pub max_retries: u32,
    pub wrap_up_time: Duration,
    pub announce_position: bool,
    pub announce_wait_time: bool,
    pub music_on_hold: Option<String>,
    pub periodic_announce: Option<String>,
    pub periodic_announce_frequency: Duration,
    pub overflow_queue_id: Option<Uuid>,
    pub overflow_action: OverflowAction,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Action to take when queue overflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverflowAction {
    /// Reject new calls with busy signal
    Busy,
    /// Send to voicemail
    Voicemail,
    /// Forward to another queue
    ForwardToQueue,
    /// Forward to specific extension
    ForwardToExtension,
    /// Play message and hangup
    Announcement,
}

impl CallQueue {
    pub fn new(name: String, extension: String, strategy: QueueStrategy) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            extension,
            strategy,
            max_wait_time: Duration::from_secs(300), // 5 minutes
            max_queue_size: 100,
            ring_timeout: Duration::from_secs(30),
            retry_delay: Duration::from_secs(5),
            max_retries: 3,
            wrap_up_time: Duration::from_secs(30),
            announce_position: true,
            announce_wait_time: true,
            music_on_hold: Some("default_moh.wav".to_string()),
            periodic_announce: None,
            periodic_announce_frequency: Duration::from_secs(60),
            overflow_queue_id: None,
            overflow_action: OverflowAction::Busy,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Call queue state (runtime data)
pub struct CallQueueState {
    pub queue_id: Uuid,
    pub members: HashMap<Uuid, QueueMember>,
    pub waiting_calls: VecDeque<QueuedCall>,
    pub active_calls: HashMap<String, Uuid>, // call_id -> member_id
    pub round_robin_index: usize,
}

impl CallQueueState {
    pub fn new(queue_id: Uuid) -> Self {
        Self {
            queue_id,
            members: HashMap::new(),
            waiting_calls: VecDeque::new(),
            active_calls: HashMap::new(),
            round_robin_index: 0,
        }
    }

    /// Add member to queue
    pub fn add_member(&mut self, member: QueueMember) {
        self.members.insert(member.id, member);
    }

    /// Remove member from queue
    pub fn remove_member(&mut self, member_id: Uuid) -> Option<QueueMember> {
        self.members.remove(&member_id)
    }

    /// Get available members
    pub fn get_available_members(&self) -> Vec<&QueueMember> {
        self.members
            .values()
            .filter(|m| m.is_available())
            .collect()
    }

    /// Enqueue a call
    pub fn enqueue_call(&mut self, call: QueuedCall) -> Result<(), String> {
        // Update positions
        for queued_call in &mut self.waiting_calls {
            queued_call.position += 1;
        }

        self.waiting_calls.push_back(call);
        Ok(())
    }

    /// Dequeue next call
    pub fn dequeue_call(&mut self) -> Option<QueuedCall> {
        let call = self.waiting_calls.pop_front();

        // Update positions
        for (idx, queued_call) in self.waiting_calls.iter_mut().enumerate() {
            queued_call.position = idx + 1;
        }

        call
    }

    /// Get next agent based on strategy
    pub fn get_next_agent(&mut self, strategy: QueueStrategy) -> Option<&mut QueueMember> {
        let available: Vec<Uuid> = self.members
            .iter()
            .filter(|(_, m)| m.is_available())
            .map(|(id, _)| *id)
            .collect();

        if available.is_empty() {
            return None;
        }

        let selected_id = match strategy {
            QueueStrategy::RoundRobin => {
                let id = available[self.round_robin_index % available.len()];
                self.round_robin_index += 1;
                id
            }
            QueueStrategy::LeastRecent => {
                available
                    .iter()
                    .min_by_key(|id| {
                        self.members
                            .get(id)
                            .and_then(|m| m.last_call_time)
                            .unwrap_or(DateTime::<Utc>::MIN_UTC)
                    })
                    .copied()?
            }
            QueueStrategy::FewestCalls => {
                available
                    .iter()
                    .min_by_key(|id| self.members.get(id).map(|m| m.total_calls).unwrap_or(0))
                    .copied()?
            }
            QueueStrategy::LeastTalkTime => {
                available
                    .iter()
                    .min_by_key(|id| {
                        self.members
                            .get(id)
                            .map(|m| m.total_talk_time)
                            .unwrap_or(Duration::from_secs(0))
                    })
                    .copied()?
            }
            QueueStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                available[rng.gen_range(0..available.len())]
            }
            QueueStrategy::Linear | QueueStrategy::RingAll => available[0],
        };

        self.members.get_mut(&selected_id)
    }

    /// Assign call to agent
    pub fn assign_call(&mut self, call_id: String, member_id: Uuid) {
        self.active_calls.insert(call_id, member_id);
        if let Some(member) = self.members.get_mut(&member_id) {
            member.mark_busy();
        }
    }

    /// Complete call
    pub fn complete_call(&mut self, call_id: &str, talk_time: Duration) {
        if let Some(member_id) = self.active_calls.remove(call_id) {
            if let Some(member) = self.members.get_mut(&member_id) {
                member.record_answered(talk_time);
                member.status = AgentStatus::AfterCallWork;
            }
        }
    }

    /// Get queue statistics
    pub fn get_statistics(&self) -> QueueStatistics {
        let total_members = self.members.len();
        let available_members = self.get_available_members().len();
        let busy_members = self.members
            .values()
            .filter(|m| m.status == AgentStatus::Busy)
            .count();

        let calls_waiting = self.waiting_calls.len();
        let longest_wait = self.waiting_calls
            .iter()
            .map(|c| c.wait_time)
            .max()
            .unwrap_or(Duration::from_secs(0));

        QueueStatistics {
            queue_id: self.queue_id,
            total_members,
            available_members,
            busy_members,
            calls_waiting,
            longest_wait,
            active_calls: self.active_calls.len(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize)]
pub struct QueueStatistics {
    pub queue_id: Uuid,
    pub total_members: usize,
    pub available_members: usize,
    pub busy_members: usize,
    pub calls_waiting: usize,
    pub longest_wait: Duration,
    pub active_calls: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_member_creation() {
        let member = QueueMember::new(1, "alice".to_string(), "1001".to_string());
        assert_eq!(member.username, "alice");
        assert_eq!(member.extension, "1001");
        assert_eq!(member.status, AgentStatus::Available);
        assert!(member.is_available());
    }

    #[test]
    fn test_queue_member_pause() {
        let mut member = QueueMember::new(1, "alice".to_string(), "1001".to_string());
        member.pause(Some("Break".to_string()));

        assert!(member.paused);
        assert_eq!(member.paused_reason, Some("Break".to_string()));
        assert!(!member.is_available());

        member.unpause();
        assert!(!member.paused);
        assert!(member.is_available());
    }

    #[test]
    fn test_queue_member_record_calls() {
        let mut member = QueueMember::new(1, "alice".to_string(), "1001".to_string());

        member.record_answered(Duration::from_secs(120));
        assert_eq!(member.answered_calls, 1);
        assert_eq!(member.total_calls, 1);
        assert_eq!(member.total_talk_time, Duration::from_secs(120));

        member.record_missed();
        assert_eq!(member.missed_calls, 1);
        assert_eq!(member.total_calls, 2);
    }

    #[test]
    fn test_call_queue_creation() {
        let queue = CallQueue::new(
            "Support Queue".to_string(),
            "8000".to_string(),
            QueueStrategy::RoundRobin,
        );

        assert_eq!(queue.name, "Support Queue");
        assert_eq!(queue.strategy, QueueStrategy::RoundRobin);
        assert_eq!(queue.max_queue_size, 100);
    }

    #[test]
    fn test_queue_state_enqueue() {
        let queue_id = Uuid::new_v4();
        let mut state = CallQueueState::new(queue_id);

        let call = QueuedCall::new("call-1".to_string(), "alice".to_string(), queue_id);
        state.enqueue_call(call).unwrap();

        assert_eq!(state.waiting_calls.len(), 1);
        assert_eq!(state.waiting_calls[0].position, 0);
    }

    #[test]
    fn test_queue_state_dequeue() {
        let queue_id = Uuid::new_v4();
        let mut state = CallQueueState::new(queue_id);

        let call1 = QueuedCall::new("call-1".to_string(), "alice".to_string(), queue_id);
        let call2 = QueuedCall::new("call-2".to_string(), "bob".to_string(), queue_id);

        state.enqueue_call(call1).unwrap();
        state.enqueue_call(call2).unwrap();

        let dequeued = state.dequeue_call().unwrap();
        assert_eq!(dequeued.caller, "alice");
        assert_eq!(state.waiting_calls.len(), 1);
        assert_eq!(state.waiting_calls[0].position, 1); // Position updated
    }

    #[test]
    fn test_round_robin_selection() {
        let queue_id = Uuid::new_v4();
        let mut state = CallQueueState::new(queue_id);

        let member1 = QueueMember::new(1, "alice".to_string(), "1001".to_string());
        let member2 = QueueMember::new(2, "bob".to_string(), "1002".to_string());

        let id1 = member1.id;
        let id2 = member2.id;

        state.add_member(member1);
        state.add_member(member2);

        let agent1 = state.get_next_agent(QueueStrategy::RoundRobin).unwrap();
        let selected1 = agent1.id;

        let agent2 = state.get_next_agent(QueueStrategy::RoundRobin).unwrap();
        let selected2 = agent2.id;

        // Should alternate
        assert_ne!(selected1, selected2);
    }

    #[test]
    fn test_queue_statistics() {
        let queue_id = Uuid::new_v4();
        let mut state = CallQueueState::new(queue_id);

        let member1 = QueueMember::new(1, "alice".to_string(), "1001".to_string());
        let member2 = QueueMember::new(2, "bob".to_string(), "1002".to_string());

        state.add_member(member1);
        state.add_member(member2);

        let call = QueuedCall::new("call-1".to_string(), "caller".to_string(), queue_id);
        state.enqueue_call(call).unwrap();

        let stats = state.get_statistics();
        assert_eq!(stats.total_members, 2);
        assert_eq!(stats.available_members, 2);
        assert_eq!(stats.calls_waiting, 1);
    }
}

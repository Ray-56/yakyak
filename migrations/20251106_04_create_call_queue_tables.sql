-- Call queues and queue members tables
-- Migration: 20251106_04

-- Call queues table
CREATE TABLE IF NOT EXISTS call_queues (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    extension VARCHAR(20) NOT NULL UNIQUE,
    strategy VARCHAR(50) NOT NULL,  -- QueueStrategy enum
    max_wait_time_secs BIGINT NOT NULL DEFAULT 300,
    max_queue_size INTEGER NOT NULL DEFAULT 100,
    ring_timeout_secs BIGINT NOT NULL DEFAULT 30,
    retry_delay_secs BIGINT NOT NULL DEFAULT 5,
    max_retries INTEGER NOT NULL DEFAULT 3,
    wrap_up_time_secs BIGINT NOT NULL DEFAULT 30,
    announce_position BOOLEAN NOT NULL DEFAULT TRUE,
    announce_wait_time BOOLEAN NOT NULL DEFAULT TRUE,
    music_on_hold VARCHAR(255),
    periodic_announce VARCHAR(255),
    periodic_announce_frequency_secs BIGINT NOT NULL DEFAULT 60,
    overflow_queue_id UUID REFERENCES call_queues(id) ON DELETE SET NULL,
    overflow_action VARCHAR(50) NOT NULL,  -- OverflowAction enum
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE call_queues IS 'Call queue configurations for ACD (Automatic Call Distribution)';
COMMENT ON COLUMN call_queues.id IS 'Unique queue identifier';
COMMENT ON COLUMN call_queues.name IS 'Queue display name';
COMMENT ON COLUMN call_queues.extension IS 'Queue extension number';
COMMENT ON COLUMN call_queues.strategy IS 'Distribution strategy: RingAll, Linear, LeastRecent, FewestCalls, LeastTalkTime, Random, RoundRobin';
COMMENT ON COLUMN call_queues.max_wait_time_secs IS 'Maximum wait time before overflow (seconds)';
COMMENT ON COLUMN call_queues.max_queue_size IS 'Maximum number of calls in queue';
COMMENT ON COLUMN call_queues.ring_timeout_secs IS 'How long to ring each agent (seconds)';
COMMENT ON COLUMN call_queues.retry_delay_secs IS 'Delay before retrying after no answer (seconds)';
COMMENT ON COLUMN call_queues.max_retries IS 'Maximum retry attempts';
COMMENT ON COLUMN call_queues.wrap_up_time_secs IS 'After-call work time (seconds)';
COMMENT ON COLUMN call_queues.announce_position IS 'Announce position in queue to caller';
COMMENT ON COLUMN call_queues.announce_wait_time IS 'Announce estimated wait time to caller';
COMMENT ON COLUMN call_queues.music_on_hold IS 'Music on hold audio file';
COMMENT ON COLUMN call_queues.periodic_announce IS 'Periodic announcement audio file';
COMMENT ON COLUMN call_queues.periodic_announce_frequency_secs IS 'How often to play periodic announcement (seconds)';
COMMENT ON COLUMN call_queues.overflow_queue_id IS 'Queue to forward to on overflow';
COMMENT ON COLUMN call_queues.overflow_action IS 'Overflow action: Busy, Voicemail, ForwardToQueue, ForwardToExtension, Announcement';

-- Queue members (agents) table
CREATE TABLE IF NOT EXISTS queue_members (
    id UUID PRIMARY KEY,
    queue_id UUID NOT NULL REFERENCES call_queues(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    username VARCHAR(100) NOT NULL,
    extension VARCHAR(20) NOT NULL,
    status VARCHAR(50) NOT NULL,  -- AgentStatus enum
    penalty INTEGER NOT NULL DEFAULT 0,
    paused BOOLEAN NOT NULL DEFAULT FALSE,
    paused_reason VARCHAR(255),
    last_call_time TIMESTAMPTZ,
    total_calls BIGINT NOT NULL DEFAULT 0,
    answered_calls BIGINT NOT NULL DEFAULT 0,
    missed_calls BIGINT NOT NULL DEFAULT 0,
    total_talk_time_secs BIGINT NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(queue_id, user_id)
);

COMMENT ON TABLE queue_members IS 'Queue members (agents) assigned to call queues';
COMMENT ON COLUMN queue_members.id IS 'Unique member identifier';
COMMENT ON COLUMN queue_members.queue_id IS 'Queue this member belongs to';
COMMENT ON COLUMN queue_members.user_id IS 'User ID of the agent';
COMMENT ON COLUMN queue_members.username IS 'Username of the agent';
COMMENT ON COLUMN queue_members.extension IS 'Extension number of the agent';
COMMENT ON COLUMN queue_members.status IS 'Agent status: Available, Busy, AfterCallWork, Paused, LoggedOut';
COMMENT ON COLUMN queue_members.penalty IS 'Routing penalty (higher = lower priority)';
COMMENT ON COLUMN queue_members.paused IS 'Whether agent is paused';
COMMENT ON COLUMN queue_members.paused_reason IS 'Reason for pause (break, lunch, etc)';
COMMENT ON COLUMN queue_members.last_call_time IS 'Timestamp of last call handled';
COMMENT ON COLUMN queue_members.total_calls IS 'Total number of calls offered';
COMMENT ON COLUMN queue_members.answered_calls IS 'Number of calls answered';
COMMENT ON COLUMN queue_members.missed_calls IS 'Number of calls missed';
COMMENT ON COLUMN queue_members.total_talk_time_secs IS 'Total talk time in seconds';
COMMENT ON COLUMN queue_members.joined_at IS 'When member joined the queue';

-- Indexes for performance
CREATE INDEX idx_call_queues_extension ON call_queues(extension);
CREATE INDEX idx_queue_members_queue_id ON queue_members(queue_id);
CREATE INDEX idx_queue_members_user_id ON queue_members(user_id);
CREATE INDEX idx_queue_members_status ON queue_members(status);
CREATE INDEX idx_queue_members_paused ON queue_members(paused);
CREATE INDEX idx_queue_members_last_call_time ON queue_members(last_call_time);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_call_queues_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER call_queues_updated_at
    BEFORE UPDATE ON call_queues
    FOR EACH ROW
    EXECUTE FUNCTION update_call_queues_updated_at();

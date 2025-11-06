-- Create call_records table for Call Detail Records (CDR)
-- Migration: 202511050003_create_call_records_table

-- Create call_records table
CREATE TABLE IF NOT EXISTS call_records (
    id UUID PRIMARY KEY,
    call_id VARCHAR(255) NOT NULL,

    -- Caller information
    caller_username VARCHAR(255) NOT NULL,
    caller_uri VARCHAR(512) NOT NULL,
    caller_ip VARCHAR(45) NOT NULL,

    -- Callee information
    callee_username VARCHAR(255) NOT NULL,
    callee_uri VARCHAR(512) NOT NULL,
    callee_ip VARCHAR(45),

    -- Call direction
    direction VARCHAR(20) NOT NULL CHECK (direction IN ('inbound', 'outbound', 'internal')),

    -- Time information
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    answer_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,

    -- Duration in seconds
    setup_duration INTEGER,
    call_duration INTEGER,
    total_duration INTEGER,

    -- Call status and result
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'completed', 'failed', 'busy', 'no_answer', 'cancelled', 'rejected')),
    end_reason TEXT,
    sip_response_code SMALLINT,

    -- Media information
    codec VARCHAR(50),
    rtp_packets_sent BIGINT,
    rtp_packets_received BIGINT,
    rtp_bytes_sent BIGINT,
    rtp_bytes_received BIGINT,

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indices for common queries

-- Index on call_id for fast lookup by SIP Call-ID
CREATE INDEX idx_call_records_call_id ON call_records(call_id);

-- Index on caller_username for filtering by caller
CREATE INDEX idx_call_records_caller_username ON call_records(caller_username);

-- Index on callee_username for filtering by callee
CREATE INDEX idx_call_records_callee_username ON call_records(callee_username);

-- Index on start_time for time-based queries and sorting
CREATE INDEX idx_call_records_start_time ON call_records(start_time DESC);

-- Index on status for filtering by call status
CREATE INDEX idx_call_records_status ON call_records(status);

-- Index on direction for filtering by call direction
CREATE INDEX idx_call_records_direction ON call_records(direction);

-- Composite index for common queries (caller + time range)
CREATE INDEX idx_call_records_caller_time ON call_records(caller_username, start_time DESC);

-- Composite index for common queries (callee + time range)
CREATE INDEX idx_call_records_callee_time ON call_records(callee_username, start_time DESC);

-- Composite index for date range queries with status
CREATE INDEX idx_call_records_time_status ON call_records(start_time DESC, status);

-- Create trigger to auto-update updated_at
CREATE TRIGGER update_call_records_updated_at BEFORE UPDATE ON call_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comments
COMMENT ON TABLE call_records IS 'Call Detail Records (CDR) for billing, auditing, and analytics';
COMMENT ON COLUMN call_records.id IS 'Unique CDR identifier (UUID)';
COMMENT ON COLUMN call_records.call_id IS 'SIP Call-ID header value';
COMMENT ON COLUMN call_records.caller_username IS 'Username of the caller';
COMMENT ON COLUMN call_records.caller_uri IS 'SIP URI of the caller';
COMMENT ON COLUMN call_records.caller_ip IS 'IP address of the caller';
COMMENT ON COLUMN call_records.callee_username IS 'Username of the callee';
COMMENT ON COLUMN call_records.callee_uri IS 'SIP URI of the callee';
COMMENT ON COLUMN call_records.callee_ip IS 'IP address of the callee';
COMMENT ON COLUMN call_records.direction IS 'Call direction: inbound, outbound, or internal';
COMMENT ON COLUMN call_records.start_time IS 'Call initiation time';
COMMENT ON COLUMN call_records.answer_time IS 'Call answer time (when callee picked up)';
COMMENT ON COLUMN call_records.end_time IS 'Call termination time';
COMMENT ON COLUMN call_records.setup_duration IS 'Setup duration in seconds (answer_time - start_time)';
COMMENT ON COLUMN call_records.call_duration IS 'Call duration in seconds (end_time - answer_time)';
COMMENT ON COLUMN call_records.total_duration IS 'Total duration in seconds (end_time - start_time)';
COMMENT ON COLUMN call_records.status IS 'Call status: active, completed, failed, busy, no_answer, cancelled, rejected';
COMMENT ON COLUMN call_records.end_reason IS 'Human-readable end reason';
COMMENT ON COLUMN call_records.sip_response_code IS 'Final SIP response code';
COMMENT ON COLUMN call_records.codec IS 'Audio codec used';
COMMENT ON COLUMN call_records.rtp_packets_sent IS 'Number of RTP packets sent';
COMMENT ON COLUMN call_records.rtp_packets_received IS 'Number of RTP packets received';
COMMENT ON COLUMN call_records.rtp_bytes_sent IS 'Number of bytes sent via RTP';
COMMENT ON COLUMN call_records.rtp_bytes_received IS 'Number of bytes received via RTP';
COMMENT ON COLUMN call_records.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN call_records.updated_at IS 'Last update timestamp';

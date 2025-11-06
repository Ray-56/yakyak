-- Create conference rooms table
CREATE TABLE IF NOT EXISTS conference_rooms (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    pin VARCHAR(20),
    max_participants INTEGER NOT NULL DEFAULT 50,
    state VARCHAR(20) NOT NULL DEFAULT 'Waiting',
    moderator_id UUID,
    recording_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    recording_file TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ
);

-- Create conference participants table
CREATE TABLE IF NOT EXISTS conference_participants (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES conference_rooms(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    call_id VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'Attendee',
    state VARCHAR(20) NOT NULL DEFAULT 'Active',
    is_muted BOOLEAN NOT NULL DEFAULT FALSE,
    volume REAL NOT NULL DEFAULT 1.0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_conference_rooms_state ON conference_rooms(state);
CREATE INDEX IF NOT EXISTS idx_conference_rooms_created_at ON conference_rooms(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_conference_participants_room_id ON conference_participants(room_id);
CREATE INDEX IF NOT EXISTS idx_conference_participants_call_id ON conference_participants(call_id);
CREATE INDEX IF NOT EXISTS idx_conference_participants_joined_at ON conference_participants(joined_at);

-- Comments
COMMENT ON TABLE conference_rooms IS 'Conference rooms for multi-party calls';
COMMENT ON TABLE conference_participants IS 'Participants in conference rooms';
COMMENT ON COLUMN conference_rooms.state IS 'Conference state: Waiting, Active, Locked, Ended';
COMMENT ON COLUMN conference_participants.role IS 'Participant role: Moderator, Presenter, Attendee, Listener';
COMMENT ON COLUMN conference_participants.state IS 'Participant state: Connecting, Active, OnHold, Muted, Disconnected';

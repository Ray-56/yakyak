-- Create voicemail mailboxes table
CREATE TABLE IF NOT EXISTS voicemail_mailboxes (
    mailbox_id VARCHAR(255) PRIMARY KEY,
    user_id INTEGER NOT NULL,
    pin VARCHAR(20),
    greeting_file TEXT,
    max_message_duration INTEGER NOT NULL DEFAULT 180,
    max_messages INTEGER NOT NULL DEFAULT 100,
    email_notification BOOLEAN NOT NULL DEFAULT FALSE,
    email_address VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create voicemail messages table
CREATE TABLE IF NOT EXISTS voicemail_messages (
    id UUID PRIMARY KEY,
    mailbox_id VARCHAR(255) NOT NULL REFERENCES voicemail_mailboxes(mailbox_id) ON DELETE CASCADE,
    caller VARCHAR(255) NOT NULL,
    caller_name VARCHAR(255),
    duration_seconds INTEGER NOT NULL,
    audio_file_path TEXT NOT NULL,
    audio_format VARCHAR(10) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'New',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ,
    saved_at TIMESTAMPTZ
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_voicemail_mailboxes_user_id ON voicemail_mailboxes(user_id);
CREATE INDEX IF NOT EXISTS idx_voicemail_messages_mailbox_id ON voicemail_messages(mailbox_id);
CREATE INDEX IF NOT EXISTS idx_voicemail_messages_status ON voicemail_messages(status);
CREATE INDEX IF NOT EXISTS idx_voicemail_messages_created_at ON voicemail_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_voicemail_messages_caller ON voicemail_messages(caller);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_voicemail_mailbox_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_voicemail_mailbox_updated_at
    BEFORE UPDATE ON voicemail_mailboxes
    FOR EACH ROW
    EXECUTE FUNCTION update_voicemail_mailbox_updated_at();

-- Comments
COMMENT ON TABLE voicemail_mailboxes IS 'Voicemail mailbox configurations';
COMMENT ON TABLE voicemail_messages IS 'Voicemail messages';
COMMENT ON COLUMN voicemail_messages.status IS 'Message status: New, Read, Saved, Deleted';
COMMENT ON COLUMN voicemail_mailboxes.max_message_duration IS 'Maximum message duration in seconds';
COMMENT ON COLUMN voicemail_mailboxes.max_messages IS 'Maximum number of messages per mailbox';

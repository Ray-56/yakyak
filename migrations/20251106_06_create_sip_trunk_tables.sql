-- SIP trunks and trunk statistics tables
-- Migration: 20251106_06

-- SIP trunks table
CREATE TABLE IF NOT EXISTS sip_trunks (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    provider_name VARCHAR(255) NOT NULL,
    trunk_type VARCHAR(50) NOT NULL,  -- TrunkType enum
    sip_server VARCHAR(255) NOT NULL,
    sip_port INTEGER NOT NULL DEFAULT 5060,
    backup_server VARCHAR(255),
    direction VARCHAR(50) NOT NULL,  -- TrunkDirection enum

    -- Authentication
    username VARCHAR(255),
    password VARCHAR(255),
    auth_username VARCHAR(255),
    realm VARCHAR(255),
    allowed_ips TEXT DEFAULT '',  -- Comma-separated IP list

    -- Registration
    register_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    registration_interval BIGINT NOT NULL DEFAULT 3600,  -- seconds
    registration_expires_at TIMESTAMPTZ,
    registered BOOLEAN NOT NULL DEFAULT FALSE,
    last_registration_time TIMESTAMPTZ,

    -- Codecs
    codecs TEXT NOT NULL DEFAULT '',  -- Comma-separated codec:priority pairs
    dtmf_mode VARCHAR(50) NOT NULL DEFAULT 'Rfc2833',

    -- Capacity
    max_concurrent_calls INTEGER NOT NULL DEFAULT 100,
    max_calls_per_second INTEGER NOT NULL DEFAULT 10,

    -- Call routing
    caller_id_number VARCHAR(50),
    caller_id_name VARCHAR(255),
    prefix_strip VARCHAR(50),
    prefix_add VARCHAR(50),

    -- Quality and features
    rtcp_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    t38_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    srtp_enabled BOOLEAN NOT NULL DEFAULT FALSE,

    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE sip_trunks IS 'SIP trunk configurations for carrier connectivity';
COMMENT ON COLUMN sip_trunks.id IS 'Unique trunk identifier';
COMMENT ON COLUMN sip_trunks.name IS 'Trunk name';
COMMENT ON COLUMN sip_trunks.provider_name IS 'Service provider name';
COMMENT ON COLUMN sip_trunks.trunk_type IS 'Trunk type: Register, IpBased, Peer';
COMMENT ON COLUMN sip_trunks.direction IS 'Call direction: Inbound, Outbound, Bidirectional';
COMMENT ON COLUMN sip_trunks.sip_server IS 'SIP server address';
COMMENT ON COLUMN sip_trunks.sip_port IS 'SIP server port';
COMMENT ON COLUMN sip_trunks.allowed_ips IS 'Comma-separated list of allowed IP addresses';
COMMENT ON COLUMN sip_trunks.register_enabled IS 'Whether to register with the provider';
COMMENT ON COLUMN sip_trunks.registration_interval IS 'Registration refresh interval (seconds)';
COMMENT ON COLUMN sip_trunks.codecs IS 'Comma-separated codec:priority pairs';
COMMENT ON COLUMN sip_trunks.dtmf_mode IS 'DTMF mode: Rfc2833, SipInfo, Inband';
COMMENT ON COLUMN sip_trunks.max_concurrent_calls IS 'Maximum concurrent calls allowed';
COMMENT ON COLUMN sip_trunks.max_calls_per_second IS 'Maximum calls per second (CPS)';
COMMENT ON COLUMN sip_trunks.prefix_strip IS 'Prefix to strip from outbound numbers';
COMMENT ON COLUMN sip_trunks.prefix_add IS 'Prefix to add to outbound numbers';
COMMENT ON COLUMN sip_trunks.rtcp_enabled IS 'Enable RTCP';
COMMENT ON COLUMN sip_trunks.t38_enabled IS 'Enable T.38 fax';
COMMENT ON COLUMN sip_trunks.srtp_enabled IS 'Enable SRTP encryption';

-- Trunk statistics table
CREATE TABLE IF NOT EXISTS trunk_statistics (
    trunk_id UUID PRIMARY KEY REFERENCES sip_trunks(id) ON DELETE CASCADE,
    current_calls INTEGER NOT NULL DEFAULT 0,
    total_calls BIGINT NOT NULL DEFAULT 0,
    successful_calls BIGINT NOT NULL DEFAULT 0,
    failed_calls BIGINT NOT NULL DEFAULT 0,
    average_call_duration DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    total_minutes DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    last_call_time TIMESTAMPTZ
);

COMMENT ON TABLE trunk_statistics IS 'Real-time statistics for SIP trunks';
COMMENT ON COLUMN trunk_statistics.trunk_id IS 'Reference to SIP trunk';
COMMENT ON COLUMN trunk_statistics.current_calls IS 'Current active calls';
COMMENT ON COLUMN trunk_statistics.total_calls IS 'Total call attempts';
COMMENT ON COLUMN trunk_statistics.successful_calls IS 'Successfully completed calls';
COMMENT ON COLUMN trunk_statistics.failed_calls IS 'Failed calls';
COMMENT ON COLUMN trunk_statistics.average_call_duration IS 'Average call duration (seconds)';
COMMENT ON COLUMN trunk_statistics.total_minutes IS 'Total call minutes';
COMMENT ON COLUMN trunk_statistics.last_call_time IS 'Last call timestamp';

-- Indexes for performance
CREATE INDEX idx_sip_trunks_name ON sip_trunks(name);
CREATE INDEX idx_sip_trunks_enabled ON sip_trunks(enabled);
CREATE INDEX idx_sip_trunks_trunk_type ON sip_trunks(trunk_type);
CREATE INDEX idx_sip_trunks_direction ON sip_trunks(direction);
CREATE INDEX idx_sip_trunks_registered ON sip_trunks(registered) WHERE register_enabled = TRUE;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_sip_trunks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sip_trunks_updated_at
    BEFORE UPDATE ON sip_trunks
    FOR EACH ROW
    EXECUTE FUNCTION update_sip_trunks_updated_at();

-- Create users table for SIP user management
-- Migration: 20251104_01_create_users_table

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    realm VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    email VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on username for fast lookup
CREATE INDEX idx_users_username ON users(username);

-- Create index on realm for multi-tenant support
CREATE INDEX idx_users_realm ON users(realm);

-- Create index on enabled for filtering
CREATE INDEX idx_users_enabled ON users(enabled);

-- Create function to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to auto-update updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default admin user (password: admin123)
-- Password hash is bcrypt hash of "admin123"
INSERT INTO users (username, password_hash, realm, display_name, email, enabled)
VALUES (
    'admin',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYPqK8P.zHG',
    'localhost',
    'Administrator',
    'admin@localhost',
    true
) ON CONFLICT (username) DO NOTHING;

-- Insert test users
INSERT INTO users (username, password_hash, realm, display_name, email, enabled)
VALUES
    (
        'alice',
        '$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW', -- secret123
        'localhost',
        'Alice',
        'alice@localhost',
        true
    ),
    (
        'bob',
        '$2b$12$h8E9mMZVZCMq3X4z7BYx1O5qKqF7YXz4J0Uq6L.eJXZqXZqXZqXZ1', -- secret456
        'localhost',
        'Bob',
        'bob@localhost',
        true
    )
ON CONFLICT (username) DO NOTHING;

COMMENT ON TABLE users IS 'SIP users for authentication and authorization';
COMMENT ON COLUMN users.id IS 'Primary key';
COMMENT ON COLUMN users.username IS 'SIP username (unique)';
COMMENT ON COLUMN users.password_hash IS 'bcrypt password hash';
COMMENT ON COLUMN users.realm IS 'SIP realm/domain';
COMMENT ON COLUMN users.display_name IS 'Display name for caller ID';
COMMENT ON COLUMN users.email IS 'Email address for notifications';
COMMENT ON COLUMN users.enabled IS 'Whether user account is active';
COMMENT ON COLUMN users.created_at IS 'Account creation timestamp';
COMMENT ON COLUMN users.updated_at IS 'Last update timestamp';

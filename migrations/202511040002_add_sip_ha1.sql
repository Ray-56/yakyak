-- Add SIP HA1 field for SIP Digest Authentication
-- Migration: 20251104_02_add_sip_ha1

-- Add sip_ha1 column to users table
-- SIP HA1 = MD5(username:realm:password)
-- This is needed for SIP Digest Authentication (RFC 2617)
ALTER TABLE users ADD COLUMN IF NOT EXISTS sip_ha1 VARCHAR(32);

-- Create function to calculate SIP HA1
CREATE OR REPLACE FUNCTION calculate_sip_ha1(
    p_username VARCHAR,
    p_realm VARCHAR,
    p_password VARCHAR
) RETURNS VARCHAR AS $$
BEGIN
    RETURN md5(p_username || ':' || p_realm || ':' || p_password);
END;
$$ LANGUAGE plpgsql;

-- Update existing users with their SIP HA1
-- Note: This assumes default passwords for existing test users
UPDATE users
SET sip_ha1 = calculate_sip_ha1(username, realm,
    CASE username
        WHEN 'admin' THEN 'admin123'
        WHEN 'alice' THEN 'secret123'
        WHEN 'bob' THEN 'secret456'
        ELSE 'changeme'
    END
)
WHERE sip_ha1 IS NULL;

COMMENT ON COLUMN users.sip_ha1 IS 'SIP HA1 hash = MD5(username:realm:password) for SIP Digest Authentication';
COMMENT ON FUNCTION calculate_sip_ha1 IS 'Calculate SIP HA1 = MD5(username:realm:password)';

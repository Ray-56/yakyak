-- Create roles table for role-based access control

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions TEXT[] NOT NULL DEFAULT '{}', -- Array of permission strings
    is_system BOOLEAN NOT NULL DEFAULT FALSE, -- System roles cannot be deleted
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create index on role name for faster lookups
CREATE INDEX IF NOT EXISTS idx_roles_name ON roles(name);
CREATE INDEX IF NOT EXISTS idx_roles_is_system ON roles(is_system);

-- Add role_id column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS role_id UUID REFERENCES roles(id) ON DELETE SET NULL;

-- Create index on user role_id for faster joins
CREATE INDEX IF NOT EXISTS idx_users_role_id ON users(role_id);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_roles_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_roles_updated_at_trigger ON roles;
CREATE TRIGGER update_roles_updated_at_trigger
    BEFORE UPDATE ON roles
    FOR EACH ROW
    EXECUTE FUNCTION update_roles_updated_at();

-- Insert default system roles
INSERT INTO roles (id, name, description, permissions, is_system) VALUES
    (
        'a0000000-0000-0000-0000-000000000001'::uuid,
        'administrator',
        'Full system access with all permissions',
        ARRAY[
            'user:read', 'user:create', 'user:update', 'user:delete', 'user:manage_roles',
            'call:read', 'call:create', 'call:terminate', 'call:transfer',
            'cdr:read', 'cdr:export', 'cdr:delete',
            'system:config', 'system:monitor', 'system:audit',
            'conference:create', 'conference:manage', 'conference:moderate',
            'voicemail:access', 'voicemail:manage'
        ],
        true
    ),
    (
        'a0000000-0000-0000-0000-000000000002'::uuid,
        'user',
        'Standard user with basic call permissions',
        ARRAY['call:create', 'call:read', 'voicemail:access'],
        true
    ),
    (
        'a0000000-0000-0000-0000-000000000003'::uuid,
        'operator',
        'Call center operator with call management permissions',
        ARRAY['call:create', 'call:read', 'call:transfer', 'call:terminate', 'user:read', 'cdr:read'],
        true
    )
ON CONFLICT (name) DO NOTHING;

-- Add comments
COMMENT ON TABLE roles IS 'User roles for role-based access control';
COMMENT ON COLUMN roles.id IS 'Unique identifier';
COMMENT ON COLUMN roles.name IS 'Role name (unique)';
COMMENT ON COLUMN roles.description IS 'Role description';
COMMENT ON COLUMN roles.permissions IS 'Array of permission strings';
COMMENT ON COLUMN roles.is_system IS 'System roles cannot be deleted';
COMMENT ON COLUMN users.role_id IS 'User role reference';

-- Tenants and tenant usage tables
-- Migration: 20251106_05

-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    status VARCHAR(50) NOT NULL,  -- TenantStatus enum
    plan VARCHAR(50) NOT NULL,    -- SubscriptionPlan enum
    realm VARCHAR(255) NOT NULL UNIQUE,

    -- Contact information
    contact_email VARCHAR(255) NOT NULL,
    contact_name VARCHAR(255),
    contact_phone VARCHAR(50),
    company_name VARCHAR(255),

    -- Billing information
    billing_email VARCHAR(255),
    billing_address TEXT,

    -- Configuration
    custom_domain VARCHAR(255),
    timezone VARCHAR(100) NOT NULL DEFAULT 'UTC',
    language VARCHAR(10) NOT NULL DEFAULT 'en',

    -- Branding
    logo_url VARCHAR(500),
    primary_color VARCHAR(20),

    -- Quota settings (from TenantQuota)
    max_users INTEGER NOT NULL DEFAULT 5,
    max_concurrent_calls INTEGER NOT NULL DEFAULT 2,
    max_conference_participants INTEGER NOT NULL DEFAULT 3,
    storage_quota_gb INTEGER NOT NULL DEFAULT 1,
    monthly_call_minutes INTEGER NOT NULL DEFAULT 100,
    advanced_features TEXT NOT NULL DEFAULT '',  -- Comma-separated feature list

    -- Trial and suspension
    trial_ends_at TIMESTAMPTZ,
    suspended_reason VARCHAR(500),

    -- Metadata
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE tenants IS 'Multi-tenancy: Tenant organizations';
COMMENT ON COLUMN tenants.id IS 'Unique tenant identifier';
COMMENT ON COLUMN tenants.name IS 'Tenant display name';
COMMENT ON COLUMN tenants.slug IS 'URL-safe identifier';
COMMENT ON COLUMN tenants.status IS 'Tenant status: Active, Suspended, Trial, Deactivated';
COMMENT ON COLUMN tenants.plan IS 'Subscription plan: Free, Starter, Professional, Enterprise, Custom';
COMMENT ON COLUMN tenants.realm IS 'SIP realm for this tenant (for isolation)';
COMMENT ON COLUMN tenants.max_users IS 'Maximum number of users allowed';
COMMENT ON COLUMN tenants.max_concurrent_calls IS 'Maximum concurrent calls allowed';
COMMENT ON COLUMN tenants.max_conference_participants IS 'Maximum participants per conference';
COMMENT ON COLUMN tenants.storage_quota_gb IS 'Storage quota in GB';
COMMENT ON COLUMN tenants.monthly_call_minutes IS 'Monthly call minutes quota';
COMMENT ON COLUMN tenants.advanced_features IS 'Comma-separated list of enabled features';
COMMENT ON COLUMN tenants.trial_ends_at IS 'Trial expiration date';
COMMENT ON COLUMN tenants.suspended_reason IS 'Reason for suspension';
COMMENT ON COLUMN tenants.metadata IS 'Custom metadata JSON';

-- Tenant usage tracking table
CREATE TABLE IF NOT EXISTS tenant_usage (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    current_users INTEGER NOT NULL DEFAULT 0,
    current_calls INTEGER NOT NULL DEFAULT 0,
    minutes_used_this_month DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    storage_used_gb DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE tenant_usage IS 'Real-time usage tracking for tenants';
COMMENT ON COLUMN tenant_usage.tenant_id IS 'Reference to tenant';
COMMENT ON COLUMN tenant_usage.current_users IS 'Current number of active users';
COMMENT ON COLUMN tenant_usage.current_calls IS 'Current number of active calls';
COMMENT ON COLUMN tenant_usage.minutes_used_this_month IS 'Total call minutes used this month';
COMMENT ON COLUMN tenant_usage.storage_used_gb IS 'Storage used in GB';
COMMENT ON COLUMN tenant_usage.last_activity_at IS 'Last activity timestamp';

-- Indexes for performance
CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_realm ON tenants(realm);
CREATE INDEX idx_tenants_status ON tenants(status);
CREATE INDEX idx_tenants_plan ON tenants(plan);
CREATE INDEX idx_tenants_contact_email ON tenants(contact_email);
CREATE INDEX idx_tenants_trial_ends_at ON tenants(trial_ends_at) WHERE trial_ends_at IS NOT NULL;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_tenants_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_tenants_updated_at();

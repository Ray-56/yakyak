/// Multi-tenancy support for isolating customer data
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (billing issues, etc.)
    Suspended,
    /// Tenant is in trial period
    Trial,
    /// Tenant has been deactivated
    Deactivated,
}

/// Tenant subscription plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionPlan {
    /// Free tier with limitations
    Free,
    /// Small business plan
    Starter,
    /// Professional plan
    Professional,
    /// Enterprise plan with full features
    Enterprise,
    /// Custom plan
    Custom(String),
}

/// Resource quotas for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    /// Maximum number of users
    pub max_users: u32,
    /// Maximum concurrent calls
    pub max_concurrent_calls: u32,
    /// Maximum call queue size
    pub max_queue_size: u32,
    /// Maximum conference participants
    pub max_conference_participants: u32,
    /// Storage quota in GB
    pub storage_quota_gb: u32,
    /// Monthly call minutes
    pub monthly_call_minutes: u32,
    /// Recording enabled
    pub recording_enabled: bool,
    /// Advanced features enabled
    pub advanced_features: Vec<String>,
}

impl TenantQuota {
    /// Free tier quotas
    pub fn free_tier() -> Self {
        Self {
            max_users: 5,
            max_concurrent_calls: 2,
            max_queue_size: 10,
            max_conference_participants: 5,
            storage_quota_gb: 1,
            monthly_call_minutes: 100,
            recording_enabled: false,
            advanced_features: vec![],
        }
    }

    /// Starter plan quotas
    pub fn starter() -> Self {
        Self {
            max_users: 25,
            max_concurrent_calls: 10,
            max_queue_size: 50,
            max_conference_participants: 15,
            storage_quota_gb: 10,
            monthly_call_minutes: 1000,
            recording_enabled: true,
            advanced_features: vec!["voicemail".to_string(), "ivr".to_string()],
        }
    }

    /// Professional plan quotas
    pub fn professional() -> Self {
        Self {
            max_users: 100,
            max_concurrent_calls: 50,
            max_queue_size: 200,
            max_conference_participants: 50,
            storage_quota_gb: 50,
            monthly_call_minutes: 5000,
            recording_enabled: true,
            advanced_features: vec![
                "voicemail".to_string(),
                "ivr".to_string(),
                "call_queue".to_string(),
                "analytics".to_string(),
            ],
        }
    }

    /// Enterprise plan quotas (unlimited)
    pub fn enterprise() -> Self {
        Self {
            max_users: u32::MAX,
            max_concurrent_calls: 1000,
            max_queue_size: u32::MAX,
            max_conference_participants: 200,
            storage_quota_gb: 500,
            monthly_call_minutes: u32::MAX,
            recording_enabled: true,
            advanced_features: vec![
                "voicemail".to_string(),
                "ivr".to_string(),
                "call_queue".to_string(),
                "analytics".to_string(),
                "sip_trunk".to_string(),
                "webrtc".to_string(),
                "api_access".to_string(),
            ],
        }
    }

    /// Check if feature is enabled
    pub fn has_feature(&self, feature: &str) -> bool {
        self.advanced_features
            .iter()
            .any(|f| f == feature)
    }
}

/// Tenant (customer/organization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String, // URL-safe identifier
    pub status: TenantStatus,
    pub plan: SubscriptionPlan,
    pub quota: TenantQuota,

    // Contact information
    pub admin_email: String,
    pub admin_name: String,
    pub phone: Option<String>,
    pub company: Option<String>,

    // Billing
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,

    // Configuration
    pub domain: Option<String>, // Custom domain for web interface
    pub realm: String,           // SIP realm for this tenant
    pub timezone: String,
    pub language: String,

    // Branding
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,

    // Metadata
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trial_ends_at: Option<DateTime<Utc>>,
}

impl Tenant {
    /// Create a new tenant
    pub fn new(name: String, slug: String, admin_email: String, admin_name: String) -> Self {
        let now = Utc::now();
        let realm = format!("{}.yakyak.local", slug);

        Self {
            id: Uuid::new_v4(),
            name: name.clone(),
            slug: slug.clone(),
            status: TenantStatus::Trial,
            plan: SubscriptionPlan::Free,
            quota: TenantQuota::free_tier(),
            admin_email,
            admin_name,
            phone: None,
            company: Some(name.clone()),
            billing_email: None,
            billing_address: None,
            domain: None,
            realm,
            timezone: "UTC".to_string(),
            language: "en".to_string(),
            logo_url: None,
            primary_color: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            trial_ends_at: Some(now + chrono::Duration::days(30)),
        }
    }

    /// Upgrade to a new plan
    pub fn upgrade_plan(&mut self, plan: SubscriptionPlan) {
        self.plan = plan.clone();

        // Update quotas based on plan
        self.quota = match plan {
            SubscriptionPlan::Free => TenantQuota::free_tier(),
            SubscriptionPlan::Starter => TenantQuota::starter(),
            SubscriptionPlan::Professional => TenantQuota::professional(),
            SubscriptionPlan::Enterprise => TenantQuota::enterprise(),
            SubscriptionPlan::Custom(_) => self.quota.clone(),
        };

        // If upgrading from trial, clear trial end date
        if self.status == TenantStatus::Trial {
            self.status = TenantStatus::Active;
            self.trial_ends_at = None;
        }

        self.updated_at = Utc::now();
    }

    /// Suspend tenant
    pub fn suspend(&mut self, reason: Option<String>) {
        self.status = TenantStatus::Suspended;
        if let Some(reason) = reason {
            self.metadata.insert("suspension_reason".to_string(), reason);
        }
        self.updated_at = Utc::now();
    }

    /// Reactivate tenant
    pub fn reactivate(&mut self) {
        self.status = TenantStatus::Active;
        self.metadata.remove("suspension_reason");
        self.updated_at = Utc::now();
    }

    /// Check if tenant is active
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active || self.status == TenantStatus::Trial
    }

    /// Check if trial has expired
    pub fn is_trial_expired(&self) -> bool {
        if let Some(trial_end) = self.trial_ends_at {
            Utc::now() > trial_end
        } else {
            false
        }
    }

    /// Check if feature is available for this tenant
    pub fn has_feature(&self, feature: &str) -> bool {
        if !self.is_active() {
            return false;
        }

        self.quota.has_feature(feature)
    }

    /// Check if tenant can create more users
    pub fn can_add_user(&self, current_users: u32) -> bool {
        current_users < self.quota.max_users
    }

    /// Check if tenant can make concurrent call
    pub fn can_make_call(&self, current_calls: u32) -> bool {
        current_calls < self.quota.max_concurrent_calls
    }
}

/// Tenant usage tracking
#[derive(Debug, Clone, Serialize)]
pub struct TenantUsage {
    pub tenant_id: Uuid,
    pub current_users: u32,
    pub current_calls: u32,
    pub total_calls_this_month: u64,
    pub minutes_used_this_month: f64,
    pub storage_used_gb: f64,
    pub last_activity: Option<DateTime<Utc>>,
}

impl TenantUsage {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            current_users: 0,
            current_calls: 0,
            total_calls_this_month: 0,
            minutes_used_this_month: 0.0,
            storage_used_gb: 0.0,
            last_activity: None,
        }
    }

    /// Check if usage is within quotas
    pub fn is_within_quotas(&self, quota: &TenantQuota) -> bool {
        self.current_users <= quota.max_users
            && self.current_calls <= quota.max_concurrent_calls
            && self.minutes_used_this_month <= quota.monthly_call_minutes as f64
            && self.storage_used_gb <= quota.storage_quota_gb as f64
    }

    /// Get usage percentage for display
    pub fn get_usage_percentages(&self, quota: &TenantQuota) -> UsagePercentages {
        UsagePercentages {
            users: (self.current_users as f64 / quota.max_users as f64 * 100.0).min(100.0),
            calls: (self.current_calls as f64 / quota.max_concurrent_calls as f64 * 100.0).min(100.0),
            minutes: (self.minutes_used_this_month / quota.monthly_call_minutes as f64 * 100.0).min(100.0),
            storage: (self.storage_used_gb / quota.storage_quota_gb as f64 * 100.0).min(100.0),
        }
    }
}

/// Usage percentages for quotas
#[derive(Debug, Clone, Serialize)]
pub struct UsagePercentages {
    pub users: f64,
    pub calls: f64,
    pub minutes: f64,
    pub storage: f64,
}

/// Repository trait for tenant persistence
#[async_trait::async_trait]
pub trait TenantRepository: Send + Sync {
    /// Create a new tenant
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, String>;

    /// Get a tenant by ID
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, String>;

    /// Get a tenant by slug
    async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, String>;

    /// Update a tenant
    async fn update_tenant(&self, tenant: &Tenant) -> Result<(), String>;

    /// Delete a tenant
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), String>;

    /// List all tenants
    async fn list_tenants(&self, status: Option<TenantStatus>) -> Result<Vec<Tenant>, String>;

    /// Get or create usage record for a tenant
    async fn get_usage(&self, tenant_id: Uuid) -> Result<Option<TenantUsage>, String>;

    /// Update usage for a tenant
    async fn update_usage(&self, usage: &TenantUsage) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(tenant.name, "Acme Corp");
        assert_eq!(tenant.slug, "acme");
        assert_eq!(tenant.status, TenantStatus::Trial);
        assert!(tenant.trial_ends_at.is_some());
        assert_eq!(tenant.realm, "acme.yakyak.local");
    }

    #[test]
    fn test_plan_upgrade() {
        let mut tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(tenant.plan, SubscriptionPlan::Free);
        assert_eq!(tenant.quota.max_users, 5);

        tenant.upgrade_plan(SubscriptionPlan::Professional);

        assert_eq!(tenant.plan, SubscriptionPlan::Professional);
        assert_eq!(tenant.quota.max_users, 100);
        assert_eq!(tenant.status, TenantStatus::Active);
        assert!(tenant.trial_ends_at.is_none());
    }

    #[test]
    fn test_tenant_suspension() {
        let mut tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        tenant.upgrade_plan(SubscriptionPlan::Starter);
        assert!(tenant.is_active());

        tenant.suspend(Some("Payment failed".to_string()));
        assert_eq!(tenant.status, TenantStatus::Suspended);
        assert!(!tenant.is_active());

        tenant.reactivate();
        assert_eq!(tenant.status, TenantStatus::Active);
        assert!(tenant.is_active());
    }

    #[test]
    fn test_feature_access() {
        let mut tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        // Free tier doesn't have voicemail
        assert!(!tenant.has_feature("voicemail"));

        // Upgrade to professional
        tenant.upgrade_plan(SubscriptionPlan::Professional);
        assert!(tenant.has_feature("voicemail"));
        assert!(tenant.has_feature("ivr"));
        assert!(tenant.has_feature("call_queue"));
    }

    #[test]
    fn test_quota_checks() {
        let tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        // Free tier has max 5 users
        assert!(tenant.can_add_user(4));
        assert!(!tenant.can_add_user(5));

        // Free tier has max 2 concurrent calls
        assert!(tenant.can_make_call(1));
        assert!(!tenant.can_make_call(2));
    }

    #[test]
    fn test_usage_tracking() {
        let tenant_id = Uuid::new_v4();
        let mut usage = TenantUsage::new(tenant_id);
        let quota = TenantQuota::professional();

        usage.current_users = 50;
        usage.minutes_used_this_month = 2500.0;
        usage.storage_used_gb = 25.0;

        assert!(usage.is_within_quotas(&quota));

        let percentages = usage.get_usage_percentages(&quota);
        assert_eq!(percentages.users, 50.0);
        assert_eq!(percentages.minutes, 50.0);
        assert_eq!(percentages.storage, 50.0);
    }

    #[test]
    fn test_trial_expiration() {
        let mut tenant = Tenant::new(
            "Acme Corp".to_string(),
            "acme".to_string(),
            "admin@acme.com".to_string(),
            "John Doe".to_string(),
        );

        // Set trial to expired
        tenant.trial_ends_at = Some(Utc::now() - chrono::Duration::days(1));
        assert!(tenant.is_trial_expired());

        // Upgrade plan clears trial
        tenant.upgrade_plan(SubscriptionPlan::Starter);
        assert!(!tenant.is_trial_expired());
    }
}

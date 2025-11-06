use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// User role with associated permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: HashSet<Permission>,
    pub is_system: bool, // System roles cannot be deleted
}

/// Permission types for role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // User management
    UserRead,
    UserCreate,
    UserUpdate,
    UserDelete,
    UserManageRoles,

    // Call management
    CallRead,
    CallCreate,
    CallTerminate,
    CallTransfer,

    // CDR access
    CdrRead,
    CdrExport,
    CdrDelete,

    // System administration
    SystemConfig,
    SystemMonitor,
    SystemAudit,

    // Conference management
    ConferenceCreate,
    ConferenceManage,
    ConferenceModerate,

    // Voicemail
    VoicemailAccess,
    VoicemailManage,
}

impl Permission {
    /// Get all available permissions
    pub fn all() -> HashSet<Permission> {
        HashSet::from([
            Permission::UserRead,
            Permission::UserCreate,
            Permission::UserUpdate,
            Permission::UserDelete,
            Permission::UserManageRoles,
            Permission::CallRead,
            Permission::CallCreate,
            Permission::CallTerminate,
            Permission::CallTransfer,
            Permission::CdrRead,
            Permission::CdrExport,
            Permission::CdrDelete,
            Permission::SystemConfig,
            Permission::SystemMonitor,
            Permission::SystemAudit,
            Permission::ConferenceCreate,
            Permission::ConferenceManage,
            Permission::ConferenceModerate,
            Permission::VoicemailAccess,
            Permission::VoicemailManage,
        ])
    }

    /// Convert permission to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::UserRead => "user:read",
            Permission::UserCreate => "user:create",
            Permission::UserUpdate => "user:update",
            Permission::UserDelete => "user:delete",
            Permission::UserManageRoles => "user:manage_roles",
            Permission::CallRead => "call:read",
            Permission::CallCreate => "call:create",
            Permission::CallTerminate => "call:terminate",
            Permission::CallTransfer => "call:transfer",
            Permission::CdrRead => "cdr:read",
            Permission::CdrExport => "cdr:export",
            Permission::CdrDelete => "cdr:delete",
            Permission::SystemConfig => "system:config",
            Permission::SystemMonitor => "system:monitor",
            Permission::SystemAudit => "system:audit",
            Permission::ConferenceCreate => "conference:create",
            Permission::ConferenceManage => "conference:manage",
            Permission::ConferenceModerate => "conference:moderate",
            Permission::VoicemailAccess => "voicemail:access",
            Permission::VoicemailManage => "voicemail:manage",
        }
    }

    /// Parse permission from string
    pub fn from_str(s: &str) -> Option<Permission> {
        match s {
            "user:read" => Some(Permission::UserRead),
            "user:create" => Some(Permission::UserCreate),
            "user:update" => Some(Permission::UserUpdate),
            "user:delete" => Some(Permission::UserDelete),
            "user:manage_roles" => Some(Permission::UserManageRoles),
            "call:read" => Some(Permission::CallRead),
            "call:create" => Some(Permission::CallCreate),
            "call:terminate" => Some(Permission::CallTerminate),
            "call:transfer" => Some(Permission::CallTransfer),
            "cdr:read" => Some(Permission::CdrRead),
            "cdr:export" => Some(Permission::CdrExport),
            "cdr:delete" => Some(Permission::CdrDelete),
            "system:config" => Some(Permission::SystemConfig),
            "system:monitor" => Some(Permission::SystemMonitor),
            "system:audit" => Some(Permission::SystemAudit),
            "conference:create" => Some(Permission::ConferenceCreate),
            "conference:manage" => Some(Permission::ConferenceManage),
            "conference:moderate" => Some(Permission::ConferenceModerate),
            "voicemail:access" => Some(Permission::VoicemailAccess),
            "voicemail:manage" => Some(Permission::VoicemailManage),
            _ => None,
        }
    }
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: Option<String>, permissions: HashSet<Permission>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            permissions,
            is_system: false,
        }
    }

    /// Create administrator role with all permissions
    pub fn administrator() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "administrator".to_string(),
            description: Some("Full system access".to_string()),
            permissions: Permission::all(),
            is_system: true,
        }
    }

    /// Create standard user role with basic permissions
    pub fn user() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "user".to_string(),
            description: Some("Standard user with basic call permissions".to_string()),
            permissions: HashSet::from([
                Permission::CallCreate,
                Permission::CallRead,
                Permission::VoicemailAccess,
            ]),
            is_system: true,
        }
    }

    /// Create operator role for call center agents
    pub fn operator() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "operator".to_string(),
            description: Some("Call center operator with call management permissions".to_string()),
            permissions: HashSet::from([
                Permission::CallCreate,
                Permission::CallRead,
                Permission::CallTransfer,
                Permission::CallTerminate,
                Permission::UserRead,
                Permission::CdrRead,
            ]),
            is_system: true,
        }
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if this is a system role (cannot be deleted)
    pub fn is_system_role(&self) -> bool {
        self.is_system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role() {
        let permissions = HashSet::from([Permission::UserRead, Permission::CallCreate]);
        let role = Role::new(
            "test_role".to_string(),
            Some("Test role".to_string()),
            permissions,
        );

        assert_eq!(role.name, "test_role");
        assert_eq!(role.description, Some("Test role".to_string()));
        assert_eq!(role.permissions.len(), 2);
        assert!(!role.is_system);
    }

    #[test]
    fn test_administrator_role() {
        let admin = Role::administrator();

        assert_eq!(admin.name, "administrator");
        assert!(admin.is_system);
        assert!(admin.has_permission(&Permission::UserDelete));
        assert!(admin.has_permission(&Permission::SystemConfig));
        assert_eq!(admin.permissions.len(), Permission::all().len());
    }

    #[test]
    fn test_user_role() {
        let user = Role::user();

        assert_eq!(user.name, "user");
        assert!(user.is_system);
        assert!(user.has_permission(&Permission::CallCreate));
        assert!(!user.has_permission(&Permission::UserDelete));
    }

    #[test]
    fn test_operator_role() {
        let operator = Role::operator();

        assert_eq!(operator.name, "operator");
        assert!(operator.is_system);
        assert!(operator.has_permission(&Permission::CallTransfer));
        assert!(!operator.has_permission(&Permission::UserDelete));
    }

    #[test]
    fn test_permission_management() {
        let mut role = Role::new(
            "test".to_string(),
            None,
            HashSet::new(),
        );

        assert!(!role.has_permission(&Permission::UserRead));

        role.add_permission(Permission::UserRead);
        assert!(role.has_permission(&Permission::UserRead));

        role.remove_permission(&Permission::UserRead);
        assert!(!role.has_permission(&Permission::UserRead));
    }

    #[test]
    fn test_permission_string_conversion() {
        assert_eq!(Permission::UserRead.as_str(), "user:read");
        assert_eq!(Permission::CallCreate.as_str(), "call:create");

        assert_eq!(Permission::from_str("user:read"), Some(Permission::UserRead));
        assert_eq!(Permission::from_str("call:create"), Some(Permission::CallCreate));
        assert_eq!(Permission::from_str("invalid"), None);
    }
}

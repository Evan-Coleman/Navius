use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Role defines user access levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Manager,
    User,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Manager => write!(f, "Manager"),
            Role::User => write!(f, "User"),
        }
    }
}

impl Role {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "manager" => Ok(Role::Manager),
            "user" => Ok(Role::User),
            _ => Err(Error::validation_error(&format!("Invalid role: {}", s))),
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Role::Admin | Role::Manager)
    }

    pub fn can_view_reports(&self) -> bool {
        matches!(self, Role::Admin | Role::Manager)
    }

    pub fn can_manage_system(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingActivation,
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::PendingActivation
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "Active"),
            UserStatus::Inactive => write!(f, "Inactive"),
            UserStatus::Suspended => write!(f, "Suspended"),
            UserStatus::PendingActivation => write!(f, "PendingActivation"),
        }
    }
}

/// User profile information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct UserProfile {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,

    pub avatar_url: Option<String>,

    #[validate(length(max = 500, message = "Bio must be less than 500 characters"))]
    pub bio: Option<String>,

    pub location: Option<String>,

    pub website: Option<String>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            avatar_url: None,
            bio: None,
            location: None,
            website: None,
        }
    }
}

/// User entity representing a system user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,

    #[validate(
        length(
            min = 3,
            max = 50,
            message = "Username must be between 3 and 50 characters"
        ),
        regex(
            path = "USERNAME_REGEX",
            message = "Username contains invalid characters"
        )
    )]
    pub username: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    pub role: Role,

    pub status: UserStatus,

    pub profile: UserProfile,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub last_login_at: Option<DateTime<Utc>>,
}

/// Static regex for username validation
#[allow(dead_code)]
static USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z0-9_\-\.]+$").unwrap());

impl User {
    /// Create a new user with minimal information
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role: Role::default(),
            status: UserStatus::default(),
            profile: UserProfile::default(),
            created_at: now,
            updated_at: now,
            last_login_at: None,
        }
    }

    /// Validate the user data
    pub fn validate(&self) -> Result<()> {
        self.validate_with_message()
            .map_err(|e| Error::validation_error(&e.to_string()))
    }

    /// Check if the user is active
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    /// Activate a pending user
    pub fn activate(&mut self) -> Result<()> {
        match self.status {
            UserStatus::PendingActivation => {
                self.status = UserStatus::Active;
                self.updated_at = Utc::now();
                Ok(())
            }
            UserStatus::Active => Err(Error::validation_error("User is already active")),
            _ => Err(Error::validation_error(
                "User cannot be activated from current status",
            )),
        }
    }

    /// Suspend a user
    pub fn suspend(&mut self) -> Result<()> {
        match self.status {
            UserStatus::Active => {
                self.status = UserStatus::Suspended;
                self.updated_at = Utc::now();
                Ok(())
            }
            UserStatus::Suspended => Err(Error::validation_error("User is already suspended")),
            _ => Err(Error::validation_error(
                "User cannot be suspended from current status",
            )),
        }
    }

    /// Update last login time
    pub fn update_last_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Change user role
    pub fn change_role(&mut self, new_role: Role) {
        self.role = new_role;
        self.updated_at = Utc::now();
    }

    /// Update user profile
    pub fn update_profile(&mut self, profile: UserProfile) -> Result<()> {
        profile
            .validate()
            .map_err(|e| Error::validation_error(&e.to_string()))?;

        self.profile = profile;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Change password
    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }

    /// Check if the user has the given permission
    pub fn has_permission(&self, permission: &str) -> bool {
        match permission {
            "manage_users" => self.role.can_manage_users(),
            "view_reports" => self.role.can_view_reports(),
            "manage_system" => self.role.can_manage_system(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashedpassword".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, Role::User);
        assert_eq!(user.status, UserStatus::PendingActivation);
        assert!(user.last_login_at.is_none());
    }

    #[test]
    fn test_user_validation() {
        let valid_user = User::new(
            "validuser".to_string(),
            "valid@example.com".to_string(),
            "hashedpassword".to_string(),
        );

        assert!(valid_user.validate().is_ok());

        let invalid_email_user = User::new(
            "validuser".to_string(),
            "invalid-email".to_string(),
            "hashedpassword".to_string(),
        );

        assert!(invalid_email_user.validate().is_err());

        let short_username_user = User::new(
            "ab".to_string(), // Too short
            "valid@example.com".to_string(),
            "hashedpassword".to_string(),
        );

        assert!(short_username_user.validate().is_err());
    }

    #[test]
    fn test_user_status_transitions() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashedpassword".to_string(),
        );

        // Initial state is PendingActivation
        assert_eq!(user.status, UserStatus::PendingActivation);

        // Can activate from pending
        assert!(user.activate().is_ok());
        assert_eq!(user.status, UserStatus::Active);

        // Cannot activate already active user
        assert!(user.activate().is_err());

        // Can suspend active user
        assert!(user.suspend().is_ok());
        assert_eq!(user.status, UserStatus::Suspended);

        // Cannot suspend already suspended user
        assert!(user.suspend().is_err());
    }

    #[test]
    fn test_role_permissions() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashedpassword".to_string(),
        );

        // Default role is User
        assert_eq!(user.role, Role::User);
        assert!(!user.has_permission("manage_users"));
        assert!(!user.has_permission("view_reports"));
        assert!(!user.has_permission("manage_system"));

        // Change to Manager
        user.change_role(Role::Manager);
        assert_eq!(user.role, Role::Manager);
        assert!(user.has_permission("manage_users"));
        assert!(user.has_permission("view_reports"));
        assert!(!user.has_permission("manage_system"));

        // Change to Admin
        user.change_role(Role::Admin);
        assert_eq!(user.role, Role::Admin);
        assert!(user.has_permission("manage_users"));
        assert!(user.has_permission("view_reports"));
        assert!(user.has_permission("manage_system"));
    }
}

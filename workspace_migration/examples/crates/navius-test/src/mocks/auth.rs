use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mockall::predicate::*;
use mockall::*;
use uuid::Uuid;

use crate::error::{TestError, TestResult};
use crate::mock::MockRegistry;

/// Error type for authentication operations
#[derive(Debug, thiserror::Error)]
pub enum MockAuthError {
    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Expired token
    #[error("Token has expired")]
    TokenExpired,

    /// Invalid token
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Insufficient permissions
    #[error("Insufficient permissions for: {0}")]
    InsufficientPermissions(String),

    /// Authentication provider error
    #[error("Authentication provider error: {0}")]
    ProviderError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Other error
    #[error("Other error: {0}")]
    OtherError(String),
}

/// User identity
#[derive(Debug, Clone, PartialEq)]
pub struct UserIdentity {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Display name
    pub display_name: String,
    /// Roles
    pub roles: Vec<String>,
    /// Claims
    pub claims: HashMap<String, String>,
    /// Is the user active
    pub is_active: bool,
}

impl UserIdentity {
    /// Create a new user identity
    pub fn new(id: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            email: String::new(),
            display_name: String::new(),
            roles: Vec::new(),
            claims: HashMap::new(),
            is_active: true,
        }
    }

    /// Set the email
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    /// Set the display name
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    /// Add a role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Add a claim
    pub fn with_claim(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.claims.insert(key.into(), value.into());
        self
    }

    /// Set the active state
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Check if the user has a role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if the user has a claim
    pub fn has_claim(&self, key: &str) -> bool {
        self.claims.contains_key(key)
    }

    /// Get a claim value
    pub fn get_claim(&self, key: &str) -> Option<&String> {
        self.claims.get(key)
    }
}

/// Authentication token
#[derive(Debug, Clone, PartialEq)]
pub struct AuthToken {
    /// Token value
    pub value: String,
    /// Token type
    pub token_type: String,
    /// Expiration time in seconds
    pub expires_in: u64,
    /// Refresh token
    pub refresh_token: Option<String>,
}

impl AuthToken {
    /// Create a new authentication token
    pub fn new(value: impl Into<String>, token_type: impl Into<String>, expires_in: u64) -> Self {
        Self {
            value: value.into(),
            token_type: token_type.into(),
            expires_in,
            refresh_token: None,
        }
    }

    /// Set the refresh token
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }
}

/// Permission
#[derive(Debug, Clone, PartialEq)]
pub struct Permission {
    /// Permission ID
    pub id: String,
    /// Permission name
    pub name: String,
    /// Resource
    pub resource: String,
    /// Action
    pub action: String,
    /// Conditions
    pub conditions: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(
        resource: impl Into<String>,
        action: impl Into<String>,
        conditions: Option<String>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let resource_str = resource.into();
        let action_str = action.into();
        let name = format!("{}:{}", resource_str, action_str);

        Self {
            id,
            name,
            resource: resource_str,
            action: action_str,
            conditions,
        }
    }
}

/// Authentication provider
#[automock]
pub trait AuthProvider: Send + Sync {
    /// Authenticate with username and password
    fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(UserIdentity, AuthToken), MockAuthError>;

    /// Verify a token
    fn verify_token(&self, token: &str) -> Result<UserIdentity, MockAuthError>;

    /// Refresh a token
    fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, MockAuthError>;

    /// Get user by ID
    fn get_user(&self, user_id: &str) -> Result<UserIdentity, MockAuthError>;

    /// Check if a user has a permission
    fn has_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, MockAuthError>;

    /// Get all permissions for a user
    fn get_permissions(&self, user_id: &str) -> Result<Vec<Permission>, MockAuthError>;

    /// Logout
    fn logout(&self, token: &str) -> Result<(), MockAuthError>;
}

impl MockAuthProvider {
    /// Create a new mock authentication provider
    pub fn new() -> Self {
        let mock = Self::default();
        mock
    }

    /// Register the mock with the registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);
        registry.register::<dyn AuthProvider, Self>(arc_self.clone())?;
        Ok(arc_self)
    }

    /// Expect authenticate to be called with specific credentials
    pub fn expect_authenticate(
        &self,
        username: &str,
        password: &str,
        result: Result<(UserIdentity, AuthToken), MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let username_clone = username.to_string();
        let password_clone = password.to_string();

        self_mut
            .expect_authenticate()
            .with(predicate::eq(username), predicate::eq(password))
            .return_once(move |_, _| result);
    }

    /// Expect verify_token to be called with a specific token
    pub fn expect_verify_token(&self, token: &str, result: Result<UserIdentity, MockAuthError>) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let token_clone = token.to_string();

        self_mut
            .expect_verify_token()
            .with(predicate::eq(token))
            .return_once(move |_| result);
    }

    /// Expect refresh_token to be called with a specific refresh token
    pub fn expect_refresh_token(
        &self,
        refresh_token: &str,
        result: Result<AuthToken, MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let refresh_token_clone = refresh_token.to_string();

        self_mut
            .expect_refresh_token()
            .with(predicate::eq(refresh_token))
            .return_once(move |_| result);
    }

    /// Expect get_user to be called with a specific user ID
    pub fn expect_get_user(&self, user_id: &str, result: Result<UserIdentity, MockAuthError>) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let user_id_clone = user_id.to_string();

        self_mut
            .expect_get_user()
            .with(predicate::eq(user_id))
            .return_once(move |_| result);
    }

    /// Expect has_permission to be called with specific parameters
    pub fn expect_has_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        result: Result<bool, MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let user_id_clone = user_id.to_string();
        let resource_clone = resource.to_string();
        let action_clone = action.to_string();

        self_mut
            .expect_has_permission()
            .with(
                predicate::eq(user_id),
                predicate::eq(resource),
                predicate::eq(action),
            )
            .return_once(move |_, _, _| result);
    }

    /// Expect get_permissions to be called with a specific user ID
    pub fn expect_get_permissions(
        &self,
        user_id: &str,
        result: Result<Vec<Permission>, MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let user_id_clone = user_id.to_string();

        self_mut
            .expect_get_permissions()
            .with(predicate::eq(user_id))
            .return_once(move |_| result);
    }

    /// Expect logout to be called with a specific token
    pub fn expect_logout(&self, token: &str, result: Result<(), MockAuthError>) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let token_clone = token.to_string();

        self_mut
            .expect_logout()
            .with(predicate::eq(token))
            .return_once(move |_| result);
    }
}

/// Trait for accessing a mock authentication provider in tests
pub trait HasMockAuth {
    /// Get the mock authentication provider
    fn auth_provider(&self) -> Arc<MockAuthProvider>;
}

/// Role-based access control
#[automock]
pub trait RbacProvider: Send + Sync {
    /// Check if a role has a permission
    fn role_has_permission(
        &self,
        role: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, MockAuthError>;

    /// Get all permissions for a role
    fn get_role_permissions(&self, role: &str) -> Result<Vec<Permission>, MockAuthError>;

    /// Assign a permission to a role
    fn assign_permission_to_role(
        &self,
        role: &str,
        permission: &Permission,
    ) -> Result<(), MockAuthError>;

    /// Remove a permission from a role
    fn remove_permission_from_role(
        &self,
        role: &str,
        permission_id: &str,
    ) -> Result<(), MockAuthError>;

    /// Get all roles
    fn get_roles(&self) -> Result<Vec<String>, MockAuthError>;

    /// Create a role
    fn create_role(&self, role: &str) -> Result<(), MockAuthError>;

    /// Delete a role
    fn delete_role(&self, role: &str) -> Result<(), MockAuthError>;
}

impl MockRbacProvider {
    /// Create a new mock RBAC provider
    pub fn new() -> Self {
        let mock = Self::default();
        mock
    }

    /// Register the mock with the registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);
        registry.register::<dyn RbacProvider, Self>(arc_self.clone())?;
        Ok(arc_self)
    }

    /// Expect role_has_permission to be called with specific parameters
    pub fn expect_role_has_permission(
        &self,
        role: &str,
        resource: &str,
        action: &str,
        result: Result<bool, MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let role_clone = role.to_string();
        let resource_clone = resource.to_string();
        let action_clone = action.to_string();

        self_mut
            .expect_role_has_permission()
            .with(
                predicate::eq(role),
                predicate::eq(resource),
                predicate::eq(action),
            )
            .return_once(move |_, _, _| result);
    }

    /// Expect get_role_permissions to be called with a specific role
    pub fn expect_get_role_permissions(
        &self,
        role: &str,
        result: Result<Vec<Permission>, MockAuthError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        let role_clone = role.to_string();

        self_mut
            .expect_get_role_permissions()
            .with(predicate::eq(role))
            .return_once(move |_| result);
    }
}

/// Trait for accessing a mock RBAC provider in tests
pub trait HasMockRbac {
    /// Get the mock RBAC provider
    fn rbac_provider(&self) -> Arc<MockRbacProvider>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_auth_provider() {
        let registry = MockRegistry::new();
        let auth = MockAuthProvider::new().register(&registry).unwrap();

        // Set up expectations
        let user = UserIdentity::new("1", "testuser")
            .with_email("test@example.com")
            .with_role("user");

        let token = AuthToken::new("test-token", "Bearer", 3600);

        auth.expect_authenticate("testuser", "password", Ok((user.clone(), token.clone())));

        auth.expect_verify_token("test-token", Ok(user.clone()));

        // Use the mock
        let (authenticated_user, auth_token) = auth.authenticate("testuser", "password").unwrap();

        assert_eq!(authenticated_user, user);
        assert_eq!(auth_token, token);

        let verified_user = auth.verify_token("test-token").unwrap();
        assert_eq!(verified_user, user);

        // Verify all expectations have been met
        registry.verify().unwrap();
    }

    #[test]
    fn test_user_identity() {
        let user = UserIdentity::new("1", "testuser")
            .with_email("test@example.com")
            .with_display_name("Test User")
            .with_role("user")
            .with_role("admin")
            .with_claim("org", "test-org")
            .with_active(true);

        assert_eq!(user.id, "1");
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert!(user.has_role("user"));
        assert!(user.has_role("admin"));
        assert!(!user.has_role("guest"));
        assert!(user.has_claim("org"));
        assert!(!user.has_claim("department"));
        assert_eq!(user.get_claim("org"), Some(&"test-org".to_string()));
        assert!(user.is_active);
    }

    #[test]
    fn test_auth_token() {
        let token =
            AuthToken::new("test-token", "Bearer", 3600).with_refresh_token("refresh-token");

        assert_eq!(token.value, "test-token");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.expires_in, 3600);
        assert_eq!(token.refresh_token, Some("refresh-token".to_string()));
    }

    #[test]
    fn test_permission() {
        let permission = Permission::new("users", "create", Some("org = 'test-org'".to_string()));

        assert!(!permission.id.is_empty());
        assert_eq!(permission.name, "users:create");
        assert_eq!(permission.resource, "users");
        assert_eq!(permission.action, "create");
        assert_eq!(permission.conditions, Some("org = 'test-org'".to_string()));
    }
}

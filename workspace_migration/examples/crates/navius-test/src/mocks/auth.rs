use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use mockall::predicate::*;
use mockall::*;
use thiserror::Error;
use uuid::Uuid;

use crate::error::{TestError, TestResult};
use crate::mock::MockRegistry;

/// Error returned from mock auth provider
#[derive(Debug, Clone, Error)]
pub enum MockAuthError {
    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Invalid token
    #[error("Invalid token")]
    InvalidToken,

    /// Expired token
    #[error("Token expired")]
    TokenExpired,

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Permission denied
    #[error("Permission denied")]
    PermissionDenied,

    /// Role not found
    #[error("Role not found")]
    RoleNotFound,

    /// Permission not found
    #[error("Permission not found")]
    PermissionNotFound,

    /// Server error
    #[error("Server error: {0}")]
    ServerError(String),

    /// Other error
    #[error("Other error: {0}")]
    OtherError(String),

    /// Not implemented
    #[error("Not implemented")]
    NotImplemented,
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

/// A user token type alias for AuthToken
pub type UserToken = AuthToken;

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

#[derive(Debug, Default)]
pub struct MockAuthProvider {
    authenticate_results: std::sync::Mutex<Vec<Result<(UserIdentity, AuthToken), MockAuthError>>>,
    verify_token_results: std::sync::Mutex<Vec<Result<UserIdentity, MockAuthError>>>,
    refresh_token_results: std::sync::Mutex<Vec<Result<AuthToken, MockAuthError>>>,
    logout_results: std::sync::Mutex<Vec<Result<(), MockAuthError>>>,
    check_permission_results: std::sync::Mutex<Vec<Result<bool, MockAuthError>>>,
    get_user_roles_results: std::sync::Mutex<Vec<Result<Vec<String>, MockAuthError>>>,
    get_user_permissions_results: std::sync::Mutex<Vec<Result<Vec<Permission>, MockAuthError>>>,
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
        registry.register_mock::<dyn AuthProvider>(arc_self.clone());
        Ok(arc_self)
    }

    /// Expect login to be called with specific credentials
    pub fn expect_login(
        &self,
        _username: &str,
        _password: &str,
        result: Result<(UserIdentity, AuthToken), MockAuthError>,
    ) {
        self.authenticate_results.lock().unwrap().push(result);
    }

    /// Expect verify_token to be called with a specific token
    pub fn expect_verify_token(&self, _token: &str, result: Result<UserIdentity, MockAuthError>) {
        self.verify_token_results.lock().unwrap().push(result);
    }

    /// Expect refresh_token to be called with a specific refresh token
    pub fn expect_refresh_token(
        &self,
        _refresh_token: &str,
        result: Result<AuthToken, MockAuthError>,
    ) {
        self.refresh_token_results.lock().unwrap().push(result);
    }

    /// Expect get_user to be called with a specific user ID
    pub fn expect_get_user(&self, _user_id: &str, result: Result<UserIdentity, MockAuthError>) {
        self.verify_token_results.lock().unwrap().push(result);
    }

    /// Expect check_permission to be called with specific parameters
    pub fn expect_check_permission(
        &self,
        _user_id: &str,
        _resource: &str,
        _action: &str,
        result: Result<bool, MockAuthError>,
    ) {
        self.check_permission_results.lock().unwrap().push(result);
    }

    /// Expect get_roles to be called with a specific user ID
    pub fn expect_get_roles(&self, _user_id: &str, result: Result<Vec<String>, MockAuthError>) {
        self.get_user_roles_results.lock().unwrap().push(result);
    }

    /// Expect get_permissions to be called with a specific user ID
    pub fn expect_get_permissions(
        &self,
        _user_id: &str,
        result: Result<Vec<Permission>, MockAuthError>,
    ) {
        self.get_user_permissions_results
            .lock()
            .unwrap()
            .push(result);
    }

    /// Expect logout to be called with a specific user ID
    pub fn expect_logout(&self, _user_id: &str, result: Result<(), MockAuthError>) {
        self.logout_results.lock().unwrap().push(result);
    }
}

impl AuthProvider for MockAuthProvider {
    fn authenticate(
        &self,
        _username: &str,
        _password: &str,
    ) -> Result<(UserIdentity, AuthToken), MockAuthError> {
        if let Some(result) = self.authenticate_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn verify_token(&self, _token: &str) -> Result<UserIdentity, MockAuthError> {
        if let Some(result) = self.verify_token_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn refresh_token(&self, _refresh_token: &str) -> Result<AuthToken, MockAuthError> {
        if let Some(result) = self.refresh_token_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn get_user(&self, _user_id: &str) -> Result<UserIdentity, MockAuthError> {
        if let Some(result) = self.verify_token_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn has_permission(
        &self,
        _user_id: &str,
        _resource: &str,
        _action: &str,
    ) -> Result<bool, MockAuthError> {
        if let Some(result) = self.check_permission_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn get_permissions(&self, _user_id: &str) -> Result<Vec<Permission>, MockAuthError> {
        if let Some(result) = self.get_user_permissions_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn logout(&self, _token: &str) -> Result<(), MockAuthError> {
        if let Some(result) = self.logout_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }
}

/// Trait for accessing a mock authentication provider in tests
pub trait HasMockAuth {
    /// Get the mock authentication provider
    fn auth_provider(&self) -> Arc<MockAuthProvider>;
}

/// Role-based access control
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

#[derive(Debug, Default)]
pub struct MockRbacProvider {
    role_has_permission_results: std::sync::Mutex<Vec<Result<bool, MockAuthError>>>,
    get_role_permissions_results: std::sync::Mutex<Vec<Result<Vec<Permission>, MockAuthError>>>,
    assign_permission_results: std::sync::Mutex<Vec<Result<(), MockAuthError>>>,
    remove_permission_results: std::sync::Mutex<Vec<Result<(), MockAuthError>>>,
    get_roles_results: std::sync::Mutex<Vec<Result<Vec<String>, MockAuthError>>>,
    create_role_results: std::sync::Mutex<Vec<Result<(), MockAuthError>>>,
    delete_role_results: std::sync::Mutex<Vec<Result<(), MockAuthError>>>,
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
        registry.register_mock::<dyn RbacProvider>(arc_self.clone());
        Ok(arc_self)
    }

    /// Expect check_role_permission to be called with specific parameters
    pub fn expect_check_role_permission(
        &self,
        _role: &str,
        _resource: &str,
        _action: &str,
        result: Result<bool, MockAuthError>,
    ) {
        self.role_has_permission_results
            .lock()
            .unwrap()
            .push(result);
    }

    /// Expect get_role_permissions to be called with a specific role
    pub fn expect_get_role_permissions(
        &self,
        _role: &str,
        result: Result<Vec<Permission>, MockAuthError>,
    ) {
        self.get_role_permissions_results
            .lock()
            .unwrap()
            .push(result);
    }

    /// Expect assign_permission_to_role to be called with specific parameters
    pub fn expect_assign_permission_to_role(
        &self,
        _role: &str,
        _permission: &Permission,
        result: Result<(), MockAuthError>,
    ) {
        self.assign_permission_results.lock().unwrap().push(result);
    }

    /// Expect remove_permission_from_role to be called with specific parameters
    pub fn expect_remove_permission_from_role(
        &self,
        _role: &str,
        _permission_id: &str,
        result: Result<(), MockAuthError>,
    ) {
        self.remove_permission_results.lock().unwrap().push(result);
    }

    /// Expect get_roles to be called
    pub fn expect_get_roles(&self, result: Result<Vec<String>, MockAuthError>) {
        self.get_roles_results.lock().unwrap().push(result);
    }

    /// Expect create_role to be called with a specific role
    pub fn expect_create_role(&self, _role: &str, result: Result<(), MockAuthError>) {
        self.create_role_results.lock().unwrap().push(result);
    }

    /// Expect delete_role to be called with a specific role
    pub fn expect_delete_role(&self, _role: &str, result: Result<(), MockAuthError>) {
        self.delete_role_results.lock().unwrap().push(result);
    }
}

impl RbacProvider for MockRbacProvider {
    fn role_has_permission(
        &self,
        _role: &str,
        _resource: &str,
        _action: &str,
    ) -> Result<bool, MockAuthError> {
        if let Some(result) = self.role_has_permission_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn get_role_permissions(&self, _role: &str) -> Result<Vec<Permission>, MockAuthError> {
        if let Some(result) = self.get_role_permissions_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn assign_permission_to_role(
        &self,
        _role: &str,
        _permission: &Permission,
    ) -> Result<(), MockAuthError> {
        if let Some(result) = self.assign_permission_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn remove_permission_from_role(
        &self,
        _role: &str,
        _permission_id: &str,
    ) -> Result<(), MockAuthError> {
        if let Some(result) = self.remove_permission_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn get_roles(&self) -> Result<Vec<String>, MockAuthError> {
        if let Some(result) = self.get_roles_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn create_role(&self, _role: &str) -> Result<(), MockAuthError> {
        if let Some(result) = self.create_role_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }

    fn delete_role(&self, _role: &str) -> Result<(), MockAuthError> {
        if let Some(result) = self.delete_role_results.lock().unwrap().pop() {
            result
        } else {
            Err(MockAuthError::NotImplemented)
        }
    }
}

/// Trait for accessing a mock RBAC provider in tests
pub trait HasMockRbac {
    /// Get the mock RBAC provider
    fn rbac_provider(&self) -> Arc<MockRbacProvider>;
}

/// Create a user token
pub fn create_user_token() -> AuthToken {
    AuthToken {
        value: "test-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: Some("refresh-token".to_string()),
    }
}

/// Create a user identity
pub fn create_user_identity() -> UserIdentity {
    UserIdentity {
        id: "user123".to_string(),
        username: "test_user".to_string(),
        email: "test_user@example.com".to_string(),
        display_name: "Test User".to_string(),
        roles: vec!["user".to_string()],
        claims: Default::default(),
        is_active: true,
    }
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

        auth.expect_login("testuser", "password", Ok((user.clone(), token.clone())));

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

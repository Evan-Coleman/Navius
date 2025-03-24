//! Mock implementations for token client and auth services.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::{
    auth::{
        client::{TokenClient, TokenValidationResult},
        models::{JwtClaims, TokenResponse, UserProfile},
    },
    error::AppError,
};

/// Mock implementation of the TokenClient for testing
#[derive(Debug, Default)]
pub struct MockTokenClient {
    /// Default user ID to return for validation
    pub default_user_id: Option<String>,
    /// Default roles to return for the user
    pub default_roles: Vec<String>,
    /// Default permissions to return for the user
    pub default_permissions: Vec<String>,
    /// Whether validation should succeed or fail
    pub validation_should_succeed: bool,
}

impl MockTokenClient {
    /// Create a new instance with default settings (validation succeeds)
    pub fn new() -> Self {
        Self {
            default_user_id: Some(Uuid::new_v4().to_string()),
            default_roles: vec!["user".to_string()],
            default_permissions: vec!["read:data".to_string()],
            validation_should_succeed: true,
        }
    }

    /// Configure the mock to fail validation
    pub fn with_failed_validation(mut self) -> Self {
        self.validation_should_succeed = false;
        self
    }

    /// Configure the mock with specific user details
    pub fn with_user(
        mut self,
        user_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Self {
        self.default_user_id = Some(user_id);
        self.default_roles = roles;
        self.default_permissions = permissions;
        self
    }
}

#[async_trait]
impl TokenClient for MockTokenClient {
    async fn get_token(&self, _username: &str, _password: &str) -> Result<TokenResponse, AppError> {
        if self.validation_should_succeed {
            Ok(TokenResponse {
                access_token: "mock_access_token".to_string(),
                expires_in: 3600,
                token_type: "Bearer".to_string(),
                refresh_token: Some("mock_refresh_token".to_string()),
            })
        } else {
            Err(AppError::AuthenticationError(
                "Invalid credentials".to_string(),
            ))
        }
    }

    async fn validate_token(&self, _token: &str) -> Result<TokenValidationResult, AppError> {
        if self.validation_should_succeed {
            let user_id = self
                .default_user_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            Ok(TokenValidationResult {
                claims: JwtClaims {
                    sub: user_id.clone(),
                    exp: 0,
                    iat: 0,
                    iss: "mock_issuer".to_string(),
                    aud: vec!["mock_audience".to_string()],
                    roles: self.default_roles.clone(),
                    permissions: self.default_permissions.clone(),
                },
                user_id,
            })
        } else {
            Err(AppError::AuthenticationError("Invalid token".to_string()))
        }
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<TokenResponse, AppError> {
        if self.validation_should_succeed {
            Ok(TokenResponse {
                access_token: "mock_refreshed_access_token".to_string(),
                expires_in: 3600,
                token_type: "Bearer".to_string(),
                refresh_token: Some("mock_refreshed_refresh_token".to_string()),
            })
        } else {
            Err(AppError::AuthenticationError(
                "Invalid refresh token".to_string(),
            ))
        }
    }

    async fn get_user_profile(&self, _token: &str) -> Result<UserProfile, AppError> {
        if self.validation_should_succeed {
            let user_id = self
                .default_user_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            Ok(UserProfile {
                id: user_id,
                email: "mock_user@example.com".to_string(),
                name: "Mock User".to_string(),
                picture: None,
            })
        } else {
            Err(AppError::AuthenticationError(
                "Could not retrieve user profile".to_string(),
            ))
        }
    }
}

/// Create an Arc-wrapped MockTokenClient with default settings
pub fn create_mock_token_client() -> Arc<MockTokenClient> {
    Arc::new(MockTokenClient::new())
}

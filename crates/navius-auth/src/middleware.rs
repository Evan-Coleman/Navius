//! Authentication middleware for Navius Auth.
//!
//! This module provides middleware for integrating authentication
//! with HTTP servers and request handling.

use crate::error::Error;
use crate::providers::AuthProvider;
#[cfg(feature = "http")]
use crate::types::Subject;
#[cfg(feature = "http")]
use axum;
use std::sync::Arc;

/// Authentication layer for HTTP requests.
#[cfg(feature = "http")]
#[derive(Clone)]
pub struct AuthLayer {
    provider: Arc<dyn AuthProvider>,
    config: AuthConfig,
}

/// Configuration for authentication middleware.
#[cfg(feature = "http")]
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Whether authentication is required.
    pub required: bool,
    /// Bearer token header name (default: "Authorization").
    pub header_name: String,
    /// Bearer token prefix (default: "Bearer").
    pub token_prefix: String,
    /// Whether to include the identity in the request extension.
    pub include_identity: bool,
    /// List of paths that do not require authentication.
    pub exempt_paths: Vec<String>,
}

#[cfg(feature = "http")]
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            required: true,
            header_name: "Authorization".to_string(),
            token_prefix: "Bearer".to_string(),
            include_identity: true,
            exempt_paths: vec![],
        }
    }
}

#[cfg(feature = "http")]
impl AuthLayer {
    /// Create a new authentication layer.
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            provider,
            config: AuthConfig::default(),
        }
    }

    /// Create a new authentication layer with custom configuration.
    pub fn with_config(provider: Arc<dyn AuthProvider>, config: AuthConfig) -> Self {
        Self { provider, config }
    }

    /// Create a permissive authentication layer that doesn't require authentication.
    pub fn optional(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            provider,
            config: AuthConfig {
                required: false,
                ..AuthConfig::default()
            },
        }
    }
}

/// Extract the token from the request header.
#[cfg(feature = "http")]
pub fn extract_token<T>(headers: &http::HeaderMap, config: &AuthConfig) -> Option<String> {
    headers
        .get(&config.header_name)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| {
            if header_str.starts_with(&config.token_prefix) {
                Some(
                    header_str
                        .trim_start_matches(&config.token_prefix)
                        .trim_start()
                        .to_string(),
                )
            } else {
                None
            }
        })
}

/// Authorization checker for verifying permissions based on roles.
#[derive(Clone)]
pub struct AuthChecker {
    provider: Arc<dyn AuthProvider>,
}

impl AuthChecker {
    /// Create a new authentication checker.
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self { provider }
    }

    /// Check if a subject is authorized.
    pub async fn check_authorization(
        &self,
        token: &str,
        required_roles: &[String],
    ) -> std::result::Result<Subject, Error> {
        let subject = self.provider.validate_token(token).await?;

        if !required_roles.is_empty()
            && !subject
                .roles
                .iter()
                .any(|role| required_roles.contains(&role.id))
        {
            return Err(Error::authentication_failed(format!(
                "Subject does not have any of the required roles: {:?}",
                required_roles
            )));
        }

        Ok(subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Role, Subject};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio_test::block_on;

    struct MockAuthProvider {
        valid_token: bool,
        subject: Subject,
    }

    impl MockAuthProvider {
        fn new(valid_token: bool) -> Self {
            Self {
                valid_token,
                subject: Subject {
                    id: "test_user".to_string(),
                    subject_type: "user".to_string(),
                    name: "Test User".to_string(),
                    roles: vec![Role {
                        id: "user_role".to_string(),
                        name: "user".to_string(),
                        description: Some("Basic user role".to_string()),
                        permissions: Some(vec!["read".to_string()]),
                    }],
                    attributes: Some(HashMap::from([(
                        "email".to_string(),
                        "test@example.com".to_string(),
                    )])),
                },
            }
        }
    }

    impl AuthProvider for MockAuthProvider {
        fn authenticate(
            &self,
            _credentials: &str,
        ) -> Box<dyn std::future::Future<Output = Result<Identity, Error>> + Send + '_> {
            Box::new(Box::pin(async move {
                if self.valid_token {
                    Ok(Identity {
                        id: "test_user".to_string(),
                        username: "test".to_string(),
                        email: Some("test@example.com".to_string()),
                        display_name: Some("Test User".to_string()),
                        roles: vec![],
                        permissions: None,
                        active: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        provider_id: Some("mock".to_string()),
                        provider_type: Some("mock".to_string()),
                    })
                } else {
                    Err(Error::authentication_failed("Invalid credentials"))
                }
            }))
        }

        fn validate_token(
            &self,
            token: &str,
        ) -> Box<dyn std::future::Future<Output = Result<Subject, Error>> + Send + '_> {
            let subject = self.subject.clone();
            Box::new(Box::pin(async move {
                if token == "valid_token" {
                    Ok(subject)
                } else {
                    Err(Error::token_invalid("Invalid token"))
                }
            }))
        }

        fn create_token(
            &self,
            _subject: &Subject,
        ) -> Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_> {
            Box::new(Box::pin(async move {
                if self.valid_token {
                    Ok("valid_token".to_string())
                } else {
                    Err(Error::token_invalid("Could not create token"))
                }
            }))
        }

        fn revoke_token(
            &self,
            _token: &str,
        ) -> Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_> {
            Box::new(Box::pin(async move { Ok(()) }))
        }

        fn refresh_token(
            &self,
            token: &str,
        ) -> Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_> {
            Box::new(Box::pin(async move {
                if token == "valid_token" {
                    Ok("new_valid_token".to_string())
                } else {
                    Err(Error::token_invalid("Invalid token"))
                }
            }))
        }

        fn provider_type(&self) -> ProviderType {
            ProviderType::Basic
        }

        fn name(&self) -> &str {
            "mock_provider"
        }
    }

    #[test]
    fn test_auth_checker_with_valid_token() {
        let provider = Arc::new(MockAuthProvider::new(true));
        let checker = AuthChecker::new(provider.clone());

        let result = block_on(checker.check_authorization("valid_token", &[]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_checker_with_invalid_token() {
        let provider = Arc::new(MockAuthProvider::new(true));
        let checker = AuthChecker::new(provider.clone());

        let result = block_on(checker.check_authorization("invalid_token", &[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token() {
        let config = AuthConfig::default();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "Authorization",
            http::HeaderValue::from_static("Bearer test_token"),
        );

        let token = extract_token::<()>(&headers, &config);
        assert_eq!(token, Some("test_token".to_string()));
    }

    #[test]
    fn test_extract_token_missing_prefix() {
        let config = AuthConfig::default();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "Authorization",
            http::HeaderValue::from_static("test_token"),
        );

        let token = extract_token::<()>(&headers, &config);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_token_missing_header() {
        let config = AuthConfig::default();
        let headers = http::HeaderMap::new();

        let token = extract_token::<()>(&headers, &config);
        assert_eq!(token, None);
    }
}

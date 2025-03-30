//! Basic authentication provider for Navius Auth.
//!
//! This module provides a simple username/password authentication provider.

use crate::error::{Error, Result};
use crate::providers::{AuthProvider, ProviderType};
use crate::types::{Credentials, Identity, Subject};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tracing::{debug, instrument};
use uuid::Uuid;

/// Configuration for the basic authentication provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasicProviderConfig {
    /// Secret key for token signing.
    pub secret_key: String,
    /// Token expiry in seconds (default: 3600).
    #[serde(default = "default_token_expiry")]
    pub token_expiry: u64,
    /// Whether to hash passwords (default: true).
    #[serde(default = "default_hash_passwords")]
    pub hash_passwords: bool,
    /// Mock user store for development/testing.
    #[serde(default)]
    pub mock_users: Vec<MockUser>,
}

fn default_token_expiry() -> u64 {
    3600 // 1 hour
}

fn default_hash_passwords() -> bool {
    true
}

/// Mock user for development/testing.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockUser {
    /// User ID.
    pub id: String,
    /// Username.
    pub username: String,
    /// Password.
    pub password: String,
    /// Display name.
    pub display_name: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// User roles.
    #[serde(default)]
    pub roles: Vec<String>,
    /// User permissions.
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Basic authentication provider.
pub struct BasicProvider {
    /// Provider name.
    name: String,
    /// Provider configuration.
    config: BasicProviderConfig,
    /// In-memory token store.
    tokens: Arc<RwLock<HashMap<String, TokenInfo>>>,
}

/// Token information.
#[derive(Debug, Clone)]
struct TokenInfo {
    /// Subject ID.
    subject_id: String,
    /// Token expiry.
    expires_at: chrono::DateTime<Utc>,
}

impl BasicProvider {
    /// Create a new basic authentication provider.
    pub fn new(name: String, config: BasicProviderConfig) -> Self {
        Self {
            name,
            config,
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a simple token.
    fn generate_token(&self, subject_id: &str) -> String {
        use rand::{thread_rng, Rng};
        use sha2::{Digest, Sha256};

        let rand_bytes: [u8; 32] = thread_rng().gen();
        let now = Utc::now().timestamp().to_string();
        let data = format!(
            "{}{}{}{}",
            subject_id,
            now,
            self.config.secret_key,
            hex::encode(rand_bytes)
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    /// Find a mock user by username.
    fn find_mock_user(&self, username: &str) -> Option<MockUser> {
        self.config
            .mock_users
            .iter()
            .find(|u| u.username == username)
            .cloned()
    }

    /// Verify a password.
    fn verify_password(&self, password: &str, stored_password: &str) -> bool {
        if self.config.hash_passwords {
            // In a real implementation, we would use a proper password hashing algorithm like bcrypt
            // For simplicity, we're using a basic comparison here
            password == stored_password
        } else {
            password == stored_password
        }
    }

    /// Store a token.
    fn store_token(&self, token: &str, subject_id: &str) -> Result<()> {
        let expires_at = Utc::now() + Duration::seconds(self.config.token_expiry as i64);
        let token_info = TokenInfo {
            subject_id: subject_id.to_string(),
            expires_at,
        };

        let mut tokens = match self.tokens.write() {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(Error::internal(
                    "Failed to acquire write lock on token store",
                ))
            }
        };

        tokens.insert(token.to_string(), token_info);
        Ok(())
    }

    /// Lookup a token.
    fn lookup_token(&self, token: &str) -> Result<Option<TokenInfo>> {
        let tokens = match self.tokens.read() {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(Error::internal(
                    "Failed to acquire read lock on token store",
                ))
            }
        };

        Ok(tokens.get(token).cloned())
    }

    /// Remove a token.
    fn remove_token(&self, token: &str) -> Result<()> {
        let mut tokens = match self.tokens.write() {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(Error::internal(
                    "Failed to acquire write lock on token store",
                ))
            }
        };

        tokens.remove(token);
        Ok(())
    }

    /// Convert a user to an identity.
    pub fn user_to_identity(&self, user: &MockUser) -> Identity {
        Identity {
            id: Uuid::new_v4().to_string(), // Generate a UUID from the string ID
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            roles: user
                .roles
                .iter()
                .map(|r| crate::types::Role {
                    id: Uuid::new_v4().to_string(),
                    name: r.clone(),
                    description: None,
                    permissions: None,
                })
                .collect::<Vec<_>>(),
            permissions: Some(HashSet::new()),
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            provider_id: Some(self.name.clone()),
            provider_type: Some(ProviderType::Basic.to_string()),
        }
    }

    /// Convert a user to a subject.
    pub fn user_to_subject(&self, user: &MockUser) -> Subject {
        Subject {
            id: Uuid::new_v4().to_string(), // Generate a UUID from the string ID
            subject_type: "user".to_string(),
            name: user
                .display_name
                .clone()
                .unwrap_or_else(|| user.username.clone()),
            roles: user
                .roles
                .iter()
                .map(|r| crate::types::Role {
                    id: Uuid::new_v4().to_string(),
                    name: r.clone(),
                    description: None,
                    permissions: None,
                })
                .collect::<Vec<_>>(),
            attributes: Some(HashMap::new()),
        }
    }
}

#[async_trait]
impl AuthProvider for BasicProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Basic
    }

    fn name(&self) -> &str {
        &self.name
    }

    #[instrument(skip(self, credentials), fields(provider = self.name()))]
    async fn authenticate(&self, credentials: &Credentials) -> Result<Identity> {
        debug!("Authenticating user {}", credentials.username);

        // Find the user in mock store
        let user = self
            .find_mock_user(&credentials.username)
            .ok_or_else(|| Error::authentication_failed("Invalid username or password"))?;

        // Verify password
        if !self.verify_password(&credentials.password, &user.password) {
            return Err(Error::authentication_failed("Invalid username or password"));
        }

        // Create identity
        let identity = self.user_to_identity(&user);

        Ok(identity)
    }

    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn validate_token(&self, token: &str) -> Result<Subject> {
        debug!("Validating token");

        // Look up token
        let token_info = match self.lookup_token(token)? {
            Some(info) => info,
            None => return Err(Error::token_invalid("Token not found")),
        };

        // Check expiry
        if token_info.expires_at < Utc::now() {
            return Err(Error::token_expired());
        }

        // Find user by ID
        let user = self
            .config
            .mock_users
            .iter()
            .find(|u| u.id == token_info.subject_id)
            .ok_or_else(|| Error::internal("User not found for valid token"))?;

        // Create subject
        let subject = self.user_to_subject(user);

        Ok(subject)
    }

    #[instrument(skip(self, subject), fields(provider = self.name()))]
    async fn create_token(&self, subject: &Subject) -> Result<String> {
        debug!("Creating token for subject {}", subject.id);

        // Generate token
        let token = self.generate_token(&subject.id.to_string());

        // Store token
        self.store_token(&token, &subject.id.to_string())?;

        Ok(token)
    }

    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn revoke_token(&self, token: &str) -> Result<()> {
        debug!("Revoking token");

        // Remove token
        self.remove_token(token)?;

        Ok(())
    }

    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn refresh_token(&self, token: &str) -> Result<String> {
        debug!("Refreshing token");

        // Validate old token
        let subject = self.validate_token(token).await?;

        // Revoke old token
        self.revoke_token(token).await?;

        // Create new token
        self.create_token(&subject).await
    }
}

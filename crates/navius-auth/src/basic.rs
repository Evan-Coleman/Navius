//! Basic authentication provider for Navius Auth.
//!
//! This module provides a simple username/password authentication provider.

use crate::error::Error;
use crate::providers::{AuthProvider, ProviderType};
use crate::types::{Identity, Subject};
use async_trait::async_trait;
use base64::Engine;
use chrono::{Duration, Utc};
use hex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
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
        let mut rng = rand::thread_rng();
        let rand_bytes: [u8; 32] = rng.random();
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
    fn store_token(&self, token: &str, subject_id: &str) -> Result<(), Error> {
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
    fn lookup_token(&self, token: &str) -> Result<Option<TokenInfo>, Error> {
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
    fn remove_token(&self, token: &str) -> Result<(), Error> {
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

    // Helper method for authenticating with basic auth directly
    pub async fn authenticate_basic(&self, credentials: &str) -> Result<Subject, Error> {
        let token = credentials;
        let subject = self.validate_token(token).await?;

        // Refresh the token
        self.create_token(&subject).await?;

        Ok(subject)
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

    async fn authenticate(&self, credentials: &str) -> Result<Identity, Error> {
        // Use the Engine API instead of deprecated decode function
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(credentials)
            .map_err(|e| Error::authentication_failed(format!("Invalid base64: {}", e)))?;

        let credentials_str = String::from_utf8(decoded)
            .map_err(|e| Error::authentication_failed(format!("Invalid UTF-8: {}", e)))?;
        let parts: Vec<&str> = credentials_str.split(':').collect();
        if parts.len() != 2 {
            return Err(Error::authentication_failed("Invalid credentials format"));
        }
        let username = parts[0];
        let password = parts[1];

        let user = self
            .find_mock_user(username)
            .ok_or_else(|| Error::authentication_failed("User not found"))?;

        if !self.verify_password(password, &user.password) {
            return Err(Error::authentication_failed("Invalid password"));
        }

        Ok(self.user_to_identity(&user))
    }

    async fn validate_token(&self, token: &str) -> Result<Subject, Error> {
        let token_info = self
            .lookup_token(token)?
            .ok_or_else(|| Error::token_invalid("Token not found"))?;

        if token_info.expires_at < Utc::now() {
            return Err(Error::token_expired());
        }

        let user = self
            .find_mock_user(&token_info.subject_id)
            .ok_or_else(|| Error::internal("User not found for valid token"))?;

        Ok(Subject {
            id: token_info.subject_id,
            name: user.username,
            subject_type: "user".to_string(),
            roles: user
                .roles
                .iter()
                .map(|r| crate::types::Role {
                    id: Uuid::new_v4().to_string(),
                    name: r.clone(),
                    description: None,
                    permissions: None,
                })
                .collect(),
            attributes: None,
        })
    }

    async fn create_token(&self, subject: &Subject) -> Result<String, Error> {
        let token = self.generate_token(&subject.id);
        self.store_token(&token, &subject.id)?;
        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<(), Error> {
        self.remove_token(token)
    }

    async fn refresh_token(&self, token: &str) -> Result<String, Error> {
        let subject = self.validate_token(token).await?;
        self.create_token(&subject).await
    }
}

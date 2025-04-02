//! JWT token provider for Navius Auth.
//!
//! This module provides a JWT-based authentication provider.

#[cfg(feature = "jwt")]
use crate::error::Error;
#[cfg(feature = "jwt")]
use crate::providers::{AuthProvider, ProviderType};
#[cfg(feature = "jwt")]
use crate::types::{Claims, Identity, Role, Subject};
#[cfg(feature = "jwt")]
use async_trait::async_trait;
#[cfg(feature = "jwt")]
use chrono::{Duration, Utc};
#[cfg(feature = "jwt")]
use futures::future::FutureExt;
#[cfg(feature = "jwt")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
#[cfg(feature = "jwt")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "jwt")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "jwt")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "jwt")]
use tracing::{debug, error};
#[cfg(feature = "jwt")]
use uuid::Uuid;

#[cfg(feature = "jwt")]
/// Configuration for the JWT token provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenProviderConfig {
    /// Secret key for token signing.
    pub secret_key: String,
    /// Token expiry in seconds (default: 3600).
    #[serde(default = "default_token_expiry")]
    pub token_expiry: u64,
    /// Token issuer (default: "navius").
    #[serde(default = "default_issuer")]
    pub issuer: String,
    /// Token audience (default: "navius-app").
    #[serde(default = "default_audience")]
    pub audience: String,
    /// Mock user store for development/testing.
    #[serde(default)]
    pub mock_users: Vec<MockUser>,
}

#[cfg(feature = "jwt")]
fn default_token_expiry() -> u64 {
    3600 // 1 hour
}

#[cfg(feature = "jwt")]
fn default_issuer() -> String {
    "navius".to_string()
}

#[cfg(feature = "jwt")]
fn default_audience() -> String {
    "navius-app".to_string()
}

#[cfg(feature = "jwt")]
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

#[cfg(feature = "jwt")]
/// JWT authentication provider.
#[derive(Clone)]
pub struct JWTProvider {
    /// Provider name.
    name: String,
    /// Provider configuration.
    config: TokenProviderConfig,
    /// Token blacklist.
    blacklist: Arc<Mutex<HashMap<String, chrono::DateTime<Utc>>>>,
}

#[cfg(feature = "jwt")]
impl JWTProvider {
    /// Create a new JWT authentication provider.
    pub fn new(name: String, config: TokenProviderConfig) -> Self {
        Self {
            name,
            config,
            blacklist: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create claims from a subject.
    fn create_claims(&self, sub: &str, roles: Option<Vec<String>>) -> Claims {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.token_expiry as i64);

        Claims {
            sub: sub.to_string(),
            iss: Some(self.config.issuer.clone()),
            aud: Some(self.config.audience.clone()),
            exp: Some(exp.timestamp() as u64),
            iat: Some(now.timestamp() as u64),
            nbf: None,
            jti: Some(Uuid::new_v4().to_string()),
            roles,
            permissions: None,
            custom: HashMap::new(),
        }
    }

    /// Convert a user to an identity.
    pub fn user_to_identity(&self, user: &MockUser) -> Identity {
        Identity {
            id: Uuid::new_v4().to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            roles: user
                .roles
                .iter()
                .map(|r| Role {
                    id: Uuid::new_v4().to_string(),
                    name: r.clone(),
                    description: None,
                    permissions: None,
                })
                .collect::<Vec<_>>(),
            permissions: Some(HashSet::new()),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            provider_id: Some(self.name.clone()),
            provider_type: Some(ProviderType::JWT.to_string()),
        }
    }

    /// Convert a user to a subject.
    pub fn user_to_subject(&self, user: &MockUser, roles: Vec<Role>) -> Subject {
        Subject {
            id: Uuid::new_v4().to_string(),
            subject_type: "user".to_string(),
            name: user
                .display_name
                .clone()
                .unwrap_or_else(|| user.username.clone()),
            roles,
            attributes: Some(HashMap::new()),
        }
    }

    /// Check if a token is expired.
    fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = Utc::now().timestamp() as u64;
        match claims.exp {
            Some(exp) => exp < now,
            None => false,
        }
    }

    /// Encode JWT claims into a token.
    fn encode_token(&self, claims: &Claims) -> std::result::Result<String, Error> {
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(self.config.secret_key.as_bytes());

        encode(&header, claims, &encoding_key)
            .map_err(|e| Error::internal(format!("Failed to encode JWT: {}", e)))
    }

    /// Decode a JWT token into claims.
    fn decode_token(&self, token: &str) -> std::result::Result<Claims, Error> {
        let decoding_key = DecodingKey::from_secret(self.config.secret_key.as_bytes());
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data =
            decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => Error::token_expired(),
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    Error::token_invalid("Invalid token issuer")
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    Error::token_invalid("Invalid token audience")
                }
                _ => Error::token_invalid(format!("Invalid token: {}", e)),
            })?;

        Ok(token_data.claims)
    }

    /// Find a mock user by username.
    fn find_mock_user_by_username(&self, username: &str) -> Option<MockUser> {
        self.config
            .mock_users
            .iter()
            .find(|u| u.username == username)
            .cloned()
    }

    /// Find a mock user by ID.
    fn find_mock_user_by_id(&self, id: &str) -> Option<MockUser> {
        self.config.mock_users.iter().find(|u| u.id == id).cloned()
    }

    /// Check if a token is blacklisted.
    fn is_blacklisted(&self, token: &str) -> bool {
        let blacklist = self
            .blacklist
            .lock()
            .expect("Failed to acquire lock on token blacklist");
        blacklist.contains_key(token)
    }

    /// Add a token to the blacklist.
    fn blacklist_token(&mut self, token: &str, expiry: chrono::DateTime<Utc>) {
        let mut blacklist = self
            .blacklist
            .lock()
            .expect("Failed to acquire lock on token blacklist");
        blacklist.insert(token.to_string(), expiry);

        // Clean up expired blacklist entries
        let now = Utc::now();
        blacklist.retain(|_, exp| *exp > now);
    }
}

#[cfg(feature = "jwt")]
#[async_trait]
impl AuthProvider for JWTProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::JWT
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn authenticate(&self, _credentials: &str) -> Result<Identity, Error> {
        Err(Error::authentication_failed(
            "JWT provider does not support direct authentication",
        ))
    }

    async fn validate_token(&self, token: &str) -> Result<Subject, Error> {
        // Use the predefined decode_token method which properly handles errors
        let claims = match self.decode_token(token) {
            Ok(claims) => claims,
            Err(e) => return Err(e),
        };

        // Check if the token is blacklisted
        if self.is_blacklisted(token) {
            return Err(Error::token_invalid("Token has been revoked"));
        }

        // Check if the token is expired (as a backup check)
        if self.is_token_expired(&claims) {
            return Err(Error::token_expired());
        }

        // Convert roles from claims to Subject's roles
        let roles = claims
            .roles
            .unwrap_or_default()
            .into_iter()
            .map(|name| Role {
                id: Uuid::new_v4().to_string(),
                name,
                description: None,
                permissions: None,
            })
            .collect();

        Ok(Subject {
            id: claims.sub.clone(),
            name: claims.sub,
            subject_type: "user".to_string(),
            roles,
            attributes: None,
        })
    }

    async fn create_token(&self, subject: &Subject) -> Result<String, Error> {
        let role_names = subject.roles.iter().map(|r| r.name.clone()).collect();
        let claims = Claims {
            sub: subject.id.clone(),
            iss: Some(self.config.issuer.clone()),
            aud: Some(self.config.audience.clone()),
            exp: Some(
                (Utc::now() + Duration::seconds(self.config.token_expiry as i64)).timestamp()
                    as u64,
            ),
            iat: Some(Utc::now().timestamp() as u64),
            nbf: None,
            jti: Some(Uuid::new_v4().to_string()),
            roles: Some(role_names),
            permissions: None,
            custom: HashMap::new(),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret_key.as_bytes()),
        )?;

        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<(), Error> {
        // Attempt to decode the token to get expiry
        let claims = match self.decode_token(token) {
            Ok(claims) => claims,
            Err(e) => {
                // If token is already invalid (expired, etc.), consider it revoked
                match e {
                    Error::TokenExpired { .. } | Error::TokenInvalid { .. } => return Ok(()),
                    _ => return Err(e),
                }
            }
        };

        // Get expiry time or default to 1 hour from now
        let expiry = match claims.exp {
            Some(exp) => chrono::DateTime::from_timestamp(exp as i64, 0)
                .unwrap_or_else(|| Utc::now() + Duration::hours(1)),
            None => Utc::now() + Duration::hours(1),
        };

        // Add to blacklist
        let mut blacklist = self
            .blacklist
            .lock()
            .map_err(|_| Error::internal("Failed to acquire lock on token blacklist"))?;
        blacklist.insert(token.to_string(), expiry);

        // Clean up expired blacklist entries
        let now = Utc::now();
        blacklist.retain(|_, exp| *exp > now);

        Ok(())
    }

    async fn refresh_token(&self, token: &str) -> Result<String, Error> {
        let subject = self.validate_token(token).await?;
        self.create_token(&subject).await
    }
}

#[cfg(feature = "jwt")]
impl JWTProvider {
    pub async fn authenticate_token(&self, token: &str) -> Result<String, Error> {
        debug!(token = %token, "Authenticating with token");

        let subject = self.validate_token(token).await?;

        debug!(subject_id = %subject.id, "Token validated successfully");

        match self.create_token(&subject).await {
            Ok(new_token) => {
                debug!(new_token = %new_token, "New token created successfully");
                Ok(new_token)
            }
            Err(e) => {
                error!(error = %e, "Failed to create new token");
                Err(Error::token_invalid(format!(
                    "Failed to create token: {}",
                    e
                )))
            }
        }
    }
}

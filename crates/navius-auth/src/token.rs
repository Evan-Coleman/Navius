//! JWT token provider for Navius Auth.
//!
//! This module provides a JWT-based authentication provider.

#[cfg(feature = "jwt")]
use crate::error::{Error, Result};
#[cfg(feature = "jwt")]
use crate::providers::{AuthProvider, ProviderType};
#[cfg(feature = "jwt")]
use crate::types::{Claims, Credentials, Identity, Permission, Role, Subject};
#[cfg(feature = "jwt")]
use async_trait::async_trait;
#[cfg(feature = "jwt")]
use chrono::{Duration, Utc};
#[cfg(feature = "jwt")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
#[cfg(feature = "jwt")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "jwt")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "jwt")]
use tracing::{debug, instrument};
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
    blacklist: HashMap<String, chrono::DateTime<Utc>>,
}

#[cfg(feature = "jwt")]
impl JWTProvider {
    /// Create a new JWT authentication provider.
    pub fn new(name: String, config: TokenProviderConfig) -> Self {
        Self {
            name,
            config,
            blacklist: HashMap::new(),
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
    fn encode_token(&self, claims: &Claims) -> Result<String> {
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(self.config.secret_key.as_bytes());

        encode(&header, claims, &encoding_key)
            .map_err(|e| Error::internal(format!("Failed to encode JWT: {}", e)))
    }

    /// Decode a JWT token into claims.
    fn decode_token(&self, token: &str) -> Result<Claims> {
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
        self.blacklist.contains_key(token)
    }

    /// Add a token to the blacklist.
    fn blacklist_token(&mut self, token: &str, expiry: chrono::DateTime<Utc>) {
        self.blacklist.insert(token.to_string(), expiry);

        // Clean up expired blacklist entries
        let now = Utc::now();
        self.blacklist.retain(|_, exp| *exp > now);
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

    /// Authenticate using credentials and return an identity.
    async fn authenticate(&self, credentials: &Credentials) -> Result<Identity> {
        // Find user by username in mock store
        let user = self
            .find_mock_user_by_username(&credentials.username)
            .ok_or_else(|| Error::authentication_failed("Invalid username"))?;

        // Check password
        if user.password != credentials.password {
            return Err(Error::authentication_failed("Invalid password"));
        }

        // Create identity
        let identity = self.user_to_identity(&user);

        Ok(identity)
    }

    /// Create an authentication token for a subject.
    #[instrument(skip(self, subject), fields(provider = self.name()))]
    async fn create_token(&self, subject: &Subject) -> Result<String> {
        // Create claims
        let role_names = subject
            .roles
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        let claims = self.create_claims(&subject.id, Some(role_names));

        // Encode token
        self.encode_token(&claims)
    }

    /// Revoke an authentication token.
    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn revoke_token(&self, token: &str) -> Result<()> {
        // Decode the token to get the claims
        let token_data = self.decode_token(token)?;

        // Add to blacklist until expiry
        let expiry =
            chrono::DateTime::<Utc>::from_timestamp(token_data.exp.unwrap_or_default() as i64, 0)
                .unwrap_or_else(|| Utc::now() + Duration::hours(24));

        // Create a clone for mutability (this would be more efficiently done with a proper thread-safe data structure)
        let mut provider = self.clone();
        provider.blacklist.insert(token.to_string(), expiry);

        Ok(())
    }

    /// Refresh an authentication token.
    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn refresh_token(&self, token: &str) -> Result<String> {
        // Decode the token to validate it
        let decoded = self.decode_token(token)?;

        // Check if the token is expired or will expire soon
        if self.is_token_expired(&decoded) {
            return Err(Error::token_expired());
        }

        // Find the user based on the subject claim
        let user = self
            .find_mock_user_by_id(&decoded.sub)
            .ok_or_else(|| Error::token_invalid("Subject not found"))?;

        // Create a new token
        let roles: Vec<Role> = user
            .roles
            .iter()
            .map(|r| Role {
                id: Uuid::new_v4().to_string(),
                name: r.clone(),
                description: None,
                permissions: None,
            })
            .collect();

        let subject = self.user_to_subject(&user, roles);

        // Create a new token
        self.create_token(&subject).await
    }

    /// Validate an authentication token.
    #[instrument(skip(self, token), fields(provider = self.name()))]
    async fn validate_token(&self, token: &str) -> Result<Subject> {
        // Check if token is blacklisted
        if self.is_blacklisted(token) {
            return Err(Error::token_invalid("Token has been revoked"));
        }

        // Decode token
        let claims = self.decode_token(token)?;

        // Validate claims
        if self.is_token_expired(&claims) {
            return Err(Error::token_expired());
        }

        // Find user by ID
        let user = self
            .find_mock_user_by_id(&claims.sub)
            .ok_or_else(|| Error::internal("User not found for valid token"))?;

        // Create subject
        let roles = claims
            .roles
            .unwrap_or_default()
            .iter()
            .map(|r| Role {
                id: Uuid::new_v4().to_string(),
                name: r.clone(),
                description: None,
                permissions: None,
            })
            .collect::<Vec<_>>();

        let subject = self.user_to_subject(&user, roles);

        Ok(subject)
    }
}

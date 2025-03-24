//! Authentication interfaces and common types
//!
//! This module defines interfaces and types that are used throughout the authentication system.

use async_trait::async_trait;

use crate::core::{
    auth::models::{JwtClaims, TokenResponse, UserProfile},
    error::AppError,
};

/// Result of token validation
#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    /// Parsed JWT claims
    pub claims: JwtClaims,
    /// User ID extracted from the token
    pub user_id: String,
}

/// Interface for token acquisition and validation
#[async_trait]
pub trait TokenClient: Send + Sync {
    /// Get a token using username and password
    async fn get_token(&self, username: &str, password: &str) -> Result<TokenResponse, AppError>;

    /// Validate a token and extract its claims
    async fn validate_token(&self, token: &str) -> Result<TokenValidationResult, AppError>;

    /// Refresh an existing token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AppError>;

    /// Get user profile information
    async fn get_user_profile(&self, token: &str) -> Result<UserProfile, AppError>;
}

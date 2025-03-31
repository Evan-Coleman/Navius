use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::UserService;
use crate::infrastructure::ServiceRegistry;

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub expires_at: i64,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // Subject (user ID)
    exp: i64,     // Expiration time
    iat: i64,     // Issued at
    role: String, // User role
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// JWT token expiration time (24 hours)
const TOKEN_EXPIRATION: i64 = 24 * 60 * 60;
// Refresh token expiration time (7 days)
const REFRESH_TOKEN_EXPIRATION: i64 = 7 * 24 * 60 * 60;
// JWT secret key (in production, this would be loaded from environment variables)
const JWT_SECRET: &[u8] = b"secret-jwt-key-for-development-only";

/// Generate JWT token
fn generate_token(user_id: Uuid, role: &str, expiration: i64) -> Result<String> {
    let now = Utc::now();
    let expires_at = (now + Duration::seconds(expiration)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expires_at,
        iat: now.timestamp(),
        role: role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| Error::internal_server_error(format!("Failed to generate token: {}", e)))
}

/// Login endpoint
pub async fn login(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user_service = registry.user_service();

    // Authenticate user
    let user = user_service
        .authenticate_user(&request.email, &request.password)
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::AuthenticationFailed => {
                Error::unauthorized("Invalid email or password")
            }
            _ => Error::internal_server_error(format!("Authentication error: {}", e.message)),
        })?;

    // Generate access token
    let token = generate_token(user.id, &user.role.to_string(), TOKEN_EXPIRATION)?;

    // Generate refresh token
    let refresh_token = generate_token(user.id, &user.role.to_string(), REFRESH_TOKEN_EXPIRATION)?;

    // Update last login timestamp
    user_service.update_last_login(user.id).await.map_err(|e| {
        Error::internal_server_error(format!("Failed to update last login: {}", e.message))
    })?;

    // Calculate expiration timestamp
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRATION)).timestamp();

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user_id: user.id.to_string(),
        expires_at,
    }))
}

/// Registration endpoint
pub async fn register(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    let user_service = registry.user_service();

    // Create new user
    let user = user_service
        .create_user(
            &request.username,
            &request.email,
            &request.password,
            None, // Default role
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            crate::application::UserServiceErrorKind::EmailAlreadyExists => {
                Error::conflict("Email already in use")
            }
            crate::application::UserServiceErrorKind::UsernameAlreadyExists => {
                Error::conflict("Username already in use")
            }
            _ => Error::internal_server_error(format!("Registration error: {}", e.message)),
        })?;

    // Generate access token
    let token = generate_token(user.id, &user.role.to_string(), TOKEN_EXPIRATION)?;

    // Generate refresh token
    let refresh_token = generate_token(user.id, &user.role.to_string(), REFRESH_TOKEN_EXPIRATION)?;

    // Update last login timestamp
    user_service.update_last_login(user.id).await.map_err(|e| {
        Error::internal_server_error(format!("Failed to update last login: {}", e.message))
    })?;

    // Calculate expiration timestamp
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRATION)).timestamp();

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user_id: user.id.to_string(),
        expires_at,
    }))
}

/// Refresh token endpoint
pub async fn refresh_token(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>> {
    // In a real implementation, we would:
    // 1. Validate the refresh token
    // 2. Check if the refresh token is blacklisted
    // 3. Generate a new access token and refresh token
    // 4. Blacklist the old refresh token

    // Here we'll do a simplified version

    // Parse the token to get the user ID and role
    let token_data = jsonwebtoken::decode::<Claims>(
        &request.refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| Error::unauthorized("Invalid refresh token"))?;

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| Error::unauthorized("Invalid user ID in token"))?;

    let user_service = registry.user_service();

    // Verify the user exists and is active
    let user = user_service
        .get_user(user_id)
        .await
        .map_err(|_| Error::unauthorized("User not found or inactive"))?;

    // Generate new tokens
    let token = generate_token(user.id, &user.role.to_string(), TOKEN_EXPIRATION)?;
    let refresh_token = generate_token(user.id, &user.role.to_string(), REFRESH_TOKEN_EXPIRATION)?;

    // Calculate expiration timestamp
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRATION)).timestamp();

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user_id: user.id.to_string(),
        expires_at,
    }))
}

/// Logout endpoint
pub async fn logout() -> Result<StatusCode> {
    // In a real implementation, we would blacklist the refresh token
    // Since we're using a simplified JWT approach without a token store, we'll just return OK
    // The client is responsible for removing the tokens from local storage

    Ok(StatusCode::OK)
}

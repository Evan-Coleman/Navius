use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use navius_core::error::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Role;
use crate::infrastructure::ServiceRegistry;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at time
    pub role: String, // User role
}

#[derive(Clone)]
pub struct AuthData {
    pub user_id: Uuid,
    pub role: Role,
}

// JWT secret key (in production, this would be loaded from environment variables)
const JWT_SECRET: &[u8] = b"secret-jwt-key-for-development-only";

/// Authentication middleware that verifies the JWT token
pub async fn auth_middleware(
    State(registry): State<Arc<ServiceRegistry>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    // If no Authorization header, return 401
    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate the token
    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Token validation error: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Parse the user ID
    let user_id = match Uuid::parse_str(&token_data.claims.sub) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Extract and convert the role
    let role = match token_data.claims.role.as_str() {
        "admin" => Role::Admin,
        "manager" => Role::Manager,
        _ => Role::User,
    };

    // Create the auth data
    let auth_data = AuthData { user_id, role };

    // Optional: Verify that the user exists and is active
    let user_service = registry.user_service();
    if let Err(_) = user_service.get_user(user_id).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add the auth data to the request extensions
    request.extensions_mut().insert(auth_data);

    // Continue with the request
    Ok(next.run(request).await)
}

/// Middleware factory for requiring authentication
pub fn requires_auth(
    registry: Arc<ServiceRegistry>,
) -> axum::middleware::from_fn_with_state<Arc<ServiceRegistry>, auth_middleware> {
    middleware::from_fn_with_state(registry, auth_middleware)
}

/// Middleware for requiring manager role
pub async fn manager_role_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get the auth data from the request extensions
    let auth_data = request
        .extensions()
        .get::<AuthData>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if the user has manager or admin role
    if !matches!(auth_data.role, Role::Manager | Role::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Continue with the request
    Ok(next.run(request).await)
}

/// Middleware factory for requiring manager role
pub fn requires_manager() -> axum::middleware::from_fn<manager_role_middleware> {
    middleware::from_fn(manager_role_middleware)
}

/// Middleware for requiring admin role
pub async fn admin_role_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get the auth data from the request extensions
    let auth_data = request
        .extensions()
        .get::<AuthData>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if the user has admin role
    if !matches!(auth_data.role, Role::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Continue with the request
    Ok(next.run(request).await)
}

/// Middleware factory for requiring admin role
pub fn requires_admin() -> axum::middleware::from_fn<admin_role_middleware> {
    middleware::from_fn(admin_role_middleware)
}

/// Helper to get the current user ID from the request
pub fn get_current_user(request: &Request) -> Option<Uuid> {
    request
        .extensions()
        .get::<AuthData>()
        .map(|auth_data| auth_data.user_id)
}

/// Extract current user ID from request
pub struct CurrentUser(pub Uuid);

impl axum::extract::FromRequestParts<()> for CurrentUser {
    type Rejection = StatusCode;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 (),
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let auth_data = parts
                .extensions
                .get::<AuthData>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            Ok(CurrentUser(auth_data.user_id))
        })
    }
}

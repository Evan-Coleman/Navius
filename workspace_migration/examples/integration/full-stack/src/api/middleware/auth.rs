use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use navius_auth::jwt::{JwtDecoder, TokenError};
use navius_core::error::Error;

use crate::domain::Role;

#[derive(Clone)]
pub struct AuthData {
    pub user_id: String,
    pub role: Role,
}

/// Authentication middleware that verifies the JWT token
pub async fn auth_middleware(
    State(jwt_decoder): State<Arc<JwtDecoder>>,
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
    let claims = match jwt_decoder.decode(&token) {
        Ok(claims) => claims,
        Err(TokenError::Expired) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Extract and convert the role
    let role = match claims.role.as_str() {
        "admin" => Role::Admin,
        "manager" => Role::Manager,
        _ => Role::User,
    };

    // Create the auth data
    let auth_data = AuthData {
        user_id: claims.sub,
        role,
    };

    // Add the auth data to the request extensions
    request.extensions_mut().insert(auth_data);

    // Continue with the request
    Ok(next.run(request).await)
}

/// Middleware factory for requiring authentication
pub fn requires_auth() -> middleware::from_fn_with_state<Arc<JwtDecoder>, auth_middleware> {
    middleware::from_fn_with_state(
        Arc::new(JwtDecoder::new("dummy_secret".to_string())),
        auth_middleware,
    )
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
pub fn requires_manager() -> axum::middleware::from_fn<auth_middleware> {
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
pub fn requires_admin() -> axum::middleware::from_fn<auth_middleware> {
    middleware::from_fn(admin_role_middleware)
}

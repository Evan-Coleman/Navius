---
title: "Authentication in Navius Applications"
description: "Comprehensive guide to implementing secure authentication in Navius with JWT, role-based access control, middleware integration, and security best practices"
category: examples
tags:
  - authentication
  - security
  - jwt
  - middleware
  - role-based-access
  - authorization
  - tokens
related:
  - 02_examples/rest-api-example.md
  - 02_examples/error-handling-example.md
  - 04_guides/security.md
last_updated: March 31, 2025
version: 1.0
status: stable
---

# Authentication Example

This example demonstrates how to implement secure authentication and authorization in a Navius application, including JWT token handling, role-based access control, and security best practices.

## Overview

Authentication and authorization are critical components of most web applications. This example shows how to:

- Set up a complete JWT-based authentication system
- Implement secure login and token generation
- Create protected routes with middleware
- Implement role-based access control
- Handle authentication errors gracefully
- Store and validate user credentials securely
- Manage token refresh and invalidation

By implementing these patterns, your application will provide secure access control while maintaining good user experience and robust security practices.

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Configuration](#configuration)
  - [Models](#models)
  - [Auth Service](#auth-service)
  - [Middleware](#middleware)
  - [Handlers](#handlers)
  - [Application Setup](#application-setup)
- [Testing the Authentication](#testing-the-authentication)
- [Key Concepts](#key-concepts)
- [Authentication Flow](#authentication-flow)
- [Security Best Practices](#security-best-practices)
- [Common Authentication Patterns](#common-authentication-patterns)
- [Refresh Token Handling](#refresh-token-handling)
- [Integrating with External Auth Providers](#integrating-with-external-auth-providers)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- HTTP and RESTful API concepts
- Basic security concepts (hashing, tokens, etc.)
- JSON Web Tokens (JWT) fundamentals

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- tokio for asynchronous operations
- jsonwebtoken for JWT generation and validation
- bcrypt for password hashing
- serde for serialization/deserialization

## Project Structure

```
authentication-example/
├── Cargo.toml                # Project dependencies
├── config/
│   └── default.yaml          # Configuration with auth settings
└── src/
    ├── main.rs               # Application entry point
    ├── models/
    │   ├── mod.rs            # Module exports
    │   ├── user.rs           # User model
    │   └── auth.rs           # Authentication models
    ├── services/
    │   ├── mod.rs            # Module exports
    │   ├── user_service.rs   # User management 
    │   └── auth_service.rs   # Authentication logic
    ├── middleware/
    │   ├── mod.rs            # Module exports
    │   └── auth.rs           # Auth middleware
    ├── handlers/
    │   ├── mod.rs            # Module exports
    │   ├── auth.rs           # Auth endpoints (login, register)
    │   └── protected.rs      # Protected resource endpoints
    └── error.rs              # Error handling
``` 

## Implementation

### Configuration

First, let's set up the authentication configuration in `config/default.yaml`:

```
server:
  host: "127.0.0.1"
  port: 8080

auth:
  jwt:
    secret: "your-secret-key-here-change-in-production" # Use env vars in production!
    expiration: 3600  # Token expiration in seconds (1 hour)
    refresh_expiration: 2592000  # Refresh token expiration (30 days)
  password:
    min_length: 8
    require_special_chars: true
    require_numbers: true
  endpoints:
    login: "/auth/login"
    register: "/auth/register"
    refresh: "/auth/refresh"
```

### Models

#### `src/models/user.rs`

```
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[validate(custom = "validate_password")]
    pub password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    // Check for at least one number
    if !password.chars().any(|c| c.is_ascii_digit()) {
        let mut err = validator::ValidationError::new("password_number");
        err.message = Some("Password must contain at least one number".into());
        return Err(err);
    }
    
    // Check for at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        let mut err = validator::ValidationError::new("password_special_char");
        err.message = Some("Password must contain at least one special character".into());
        return Err(err);
    }
    
    Ok(())
}
```

#### `src/models/auth.rs`

```
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // Subject (user ID)
    pub email: String,       // User email
    pub roles: Vec<String>,  // User roles
    pub exp: usize,          // Expiration timestamp
    pub iat: usize,          // Issued at timestamp
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
```

### Auth Service

#### `src/services/auth_service.rs`

```
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::user::{User, RegisterRequest, LoginRequest};
use crate::models::auth::{Claims, TokenResponse, RefreshRequest};
use crate::services::user_service::UserService;
use crate::error::AppError;

pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_expiration: i64,
}

pub struct AuthService {
    config: AuthConfig,
    user_service: Arc<UserService>,
}

impl AuthService {
    pub fn new(config: AuthConfig, user_service: Arc<UserService>) -> Self {
        Self {
            config,
            user_service,
        }
    }
    
    pub async fn register(&self, request: RegisterRequest) -> Result<User, AppError> {
        // Check if user already exists
        if self.user_service.find_by_email(&request.email).await.is_some() {
            return Err(AppError::conflict("User with this email already exists"));
        }
        
        // Hash password
        let password_hash = self.hash_password(&request.password)?;
        
        // Create user with default role
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: request.username,
            email: request.email,
            password_hash,
            roles: vec!["USER".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Save user
        self.user_service.create(user.clone()).await?;
        
        // Return user without password hash
        Ok(user)
    }
    
    pub async fn login(&self, request: LoginRequest) -> Result<TokenResponse, AppError> {
        // Find user by email
        let user = self.user_service.find_by_email(&request.email).await
            .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;
        
        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            return Err(AppError::unauthorized("Invalid email or password"));
        }
        
        // Generate tokens
        self.generate_tokens(&user)
    }
    
    pub async fn refresh(&self, request: RefreshRequest) -> Result<TokenResponse, AppError> {
        // Decode and validate refresh token
        let claims = self.decode_token(&request.refresh_token)?;
        
        // Find user
        let user = self.user_service.find_by_id(&claims.sub).await
            .ok_or_else(|| AppError::unauthorized("Invalid token"))?;
        
        // Generate new tokens
        self.generate_tokens(&user)
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        self.decode_token(token)
    }
    
    // Private helper methods
    
    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| AppError::internal_server_error(format!("Failed to hash password: {}", e)))
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        verify(password, hash)
            .map_err(|e| AppError::internal_server_error(format!("Failed to verify password: {}", e)))
    }
    
    fn generate_tokens(&self, user: &User) -> Result<TokenResponse, AppError> {
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        
        // Create access token with short expiration
        let access_exp = (now + Duration::seconds(self.config.jwt_expiration)).timestamp() as usize;
        let access_claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            exp: access_exp,
            iat,
        };
        
        // Create refresh token with longer expiration
        let refresh_exp = (now + Duration::seconds(self.config.refresh_expiration)).timestamp() as usize;
        let refresh_claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: vec![],  // No roles in refresh token for security
            exp: refresh_exp,
            iat,
        };
        
        // Encode tokens
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes())
        ).map_err(|e| AppError::internal_server_error(format!("Failed to create access token: {}", e)))?;
        
        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes())
        ).map_err(|e| AppError::internal_server_error(format!("Failed to create refresh token: {}", e)))?;
        
        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiration as u64,
        })
    }
    
    fn decode_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation
        )
        .map(|data| data.claims)
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::unauthorized("Token has expired")
                },
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::unauthorized("Invalid token")
                },
                _ => AppError::unauthorized(format!("Token validation failed: {}", e))
            }
        })
    }
}
```

### Middleware

#### `src/middleware/auth.rs`

```
use std::sync::Arc;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    extract::State,
};
use axum::extract::TypedHeader;
use axum::headers::{Authorization, Bearer};

use crate::services::auth_service::AuthService;
use crate::error::AppError;
use crate::models::auth::Claims;

// Current authenticated user context that handlers can extract
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
}

// Middleware to verify JWT and extract user information
pub async fn auth_middleware<B>(
    State(auth_service): State<Arc<AuthService>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Validate token and extract claims
    let claims = auth_service.validate_token(auth.token())?;
    
    // Create current user context
    let current_user = CurrentUser {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        roles: claims.roles.clone(),
    };
    
    // Add current user to request extensions
    request.extensions_mut().insert(current_user);
    
    // Continue with the request
    Ok(next.run(request).await)
}

// Middleware to check if user has required role
pub fn require_role(role: &'static str) -> impl FnOnce(CurrentUser) -> Result<CurrentUser, AppError> + Clone {
    move |current_user: CurrentUser| {
        if current_user.roles.contains(&role.to_string()) {
            Ok(current_user)
        } else {
            Err(AppError::forbidden("Insufficient permissions"))
        }
    }
}
```

### Handlers

#### `src/handlers/auth.rs`

```
use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use validator::Validate;

use crate::services::auth_service::AuthService;
use crate::models::user::{RegisterRequest, LoginRequest};
use crate::models::auth::RefreshRequest;
use crate::error::AppError;

pub async fn register(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validate request
    if let Err(errors) = request.validate() {
        return Err(AppError::validation_errors(errors));
    }
    
    // Register user
    let user = auth_service.register(request).await?;
    
    // Return success response
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "roles": user.roles,
            "created_at": user.created_at,
        }))
    ))
}

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate request
    if let Err(errors) = request.validate() {
        return Err(AppError::validation_errors(errors));
    }
    
    // Attempt login
    let token_response = auth_service.login(request).await?;
    
    // Return tokens
    Ok(Json(serde_json::json!({
        "access_token": token_response.access_token,
        "refresh_token": token_response.refresh_token,
        "token_type": token_response.token_type,
        "expires_in": token_response.expires_in,
    })))
}

pub async fn refresh(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Attempt to refresh token
    let token_response = auth_service.refresh(request).await?;
    
    // Return new tokens
    Ok(Json(serde_json::json!({
        "access_token": token_response.access_token,
        "refresh_token": token_response.refresh_token,
        "token_type": token_response.token_type,
        "expires_in": token_response.expires_in,
    })))
}
```

#### `src/handlers/protected.rs`

```
use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;

use crate::services::user_service::UserService;
use crate::middleware::auth::{CurrentUser, require_role};
use crate::error::AppError;

// Handler that requires authentication but no specific role
pub async fn get_profile(
    current_user: CurrentUser,
    State(user_service): State<Arc<UserService>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Look up the full user profile
    let user = user_service.find_by_id(&current_user.user_id).await
        .ok_or_else(|| AppError::not_found("User not found"))?;
    
    Ok(Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "roles": user.roles,
        "created_at": user.created_at,
    })))
}

// Handler that requires admin role
pub async fn get_user_by_id(
    current_user: CurrentUser,
    require_role("ADMIN"): (),
    State(user_service): State<Arc<UserService>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Look up the requested user
    let user = user_service.find_by_id(&user_id).await
        .ok_or_else(|| AppError::not_found(format!("User with ID {} not found", user_id)))?;
    
    Ok(Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "roles": user.roles,
        "created_at": user.created_at,
    })))
}

// Handler that requires multiple roles (admin or moderator)
pub async fn admin_dashboard(
    current_user: CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check if user has either role
    if !current_user.roles.iter().any(|r| r == "ADMIN" || r == "MODERATOR") {
        return Err(AppError::forbidden("Requires admin or moderator role"));
    }
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the admin dashboard",
        "user": {
            "id": current_user.user_id,
            "email": current_user.email,
            "roles": current_user.roles,
        }
    })))
}
```

### Application Setup

#### `src/main.rs`

```
mod models;
mod services;
mod middleware;
mod handlers;
mod error;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    middleware as axum_middleware,
};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

use crate::services::auth_service::{AuthService, AuthConfig};
use crate::services::user_service::UserService;
use crate::middleware::auth::auth_middleware;
use crate::handlers::auth::{register, login, refresh};
use crate::handlers::protected::{get_profile, get_user_by_id, admin_dashboard};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Initialize services
    let user_service = Arc::new(UserService::new());
    
    let auth_config = AuthConfig {
        jwt_secret: "your-secret-key-here-change-in-production".to_string(),
        jwt_expiration: 3600,
        refresh_expiration: 2592000,
    };
    
    let auth_service = Arc::new(AuthService::new(auth_config, user_service.clone()));
    
    // Define public routes that don't require authentication
    let public_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .with_state(auth_service.clone());
    
    // Define protected routes that require authentication
    let protected_routes = Router::new()
        .route("/profile", get(get_profile))
        .route("/users/:id", get(get_user_by_id))
        .route("/admin/dashboard", get(admin_dashboard))
        .route_layer(axum_middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware
        ))
        .with_state(user_service);
    
    // Combine routes and add middleware
    let app = Router::new()
        .merge(public_routes)
        .nest("/api", protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any));
    
    // Start server
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server running on http://127.0.0.1:8080");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Testing the Authentication

### Running the Example

```
cargo run
```

### Testing the Register Endpoint

```
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "Password1!"
  }'
```

Sample response:
```
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "username": "testuser",
  "email": "test@example.com",
  "roles": ["USER"],
  "created_at": "2025-03-27T12:00:00Z"
}
```

### Testing the Login Endpoint

```
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "Password1!"
  }'
```

Sample response:
```
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Testing a Protected Endpoint

```
curl http://localhost:8080/api/profile \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Sample response:
```
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "username": "testuser",
  "email": "test@example.com",
  "roles": ["USER"],
  "created_at": "2025-03-27T12:00:00Z"
}
```

### Testing Token Refresh

```
curl -X POST http://localhost:8080/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

Sample response:
```
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Testing Role-Based Access

```
curl http://localhost:8080/api/admin/dashboard \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

If the user doesn't have the required role:
```
{
  "status": 403,
  "code": "FORBIDDEN",
  "message": "Requires admin or moderator role"
}
```

## Key Concepts

### JWT-Based Authentication

JWT (JSON Web Token) is an open standard for securely transmitting information as a JSON object. In this example, we use JWTs for:

1. **Access Tokens**: Short-lived tokens (1 hour) that grant access to protected resources
2. **Refresh Tokens**: Long-lived tokens (30 days) used to obtain new access tokens
3. **Claims**: Information embedded in the token, such as user ID, roles, and expiration

The token flow works as follows:
1. User logs in with credentials
2. Server validates credentials and issues both access and refresh tokens
3. Client uses access token for API requests
4. When access token expires, client uses refresh token to get a new set of tokens

### Role-Based Access Control (RBAC)

RBAC is implemented through:

1. **User Roles**: Each user has a set of roles (e.g., "USER", "ADMIN", "MODERATOR")
2. **JWT Claims**: Roles are included in the JWT claims
3. **Middleware**: Authentication middleware extracts and verifies the JWT
4. **Role Checks**: Handlers can require specific roles using the `require_role` function

This allows for flexible permission management where different endpoints can require different roles.

### Secure Password Handling

The example demonstrates secure password practices:

1. **Password Hashing**: Using bcrypt to securely hash passwords before storage
2. **Password Validation**: Enforcing password complexity requirements
3. **Password Verification**: Safely comparing passwords against stored hashes

## Authentication Flow

The full authentication flow in this example works as follows:

1. **User Registration**:
   - User submits username, email, and password
   - Server validates input
   - Password is hashed using bcrypt
   - User is created with default "USER" role
   - User details (excluding password hash) are returned

2. **User Login**:
   - User submits email and password
   - Server validates credentials
   - Server generates access and refresh tokens
   - Tokens are returned to the client

3. **Accessing Protected Resources**:
   - Client includes access token in Authorization header
   - Server validates token and extracts user information
   - If token is valid, the request proceeds
   - If token is invalid or expired, the server returns 401 Unauthorized

4. **Token Refresh**:
   - When access token expires, client sends refresh token
   - Server validates refresh token and issues new access/refresh tokens
   - If refresh token is invalid or expired, user must log in again

5. **Role-Based Access Control**:
   - Some endpoints require specific roles
   - Server checks if the authenticated user has the required role
   - If not, the server returns 403 Forbidden

## Security Best Practices

### JWT Security

1. **Use a Strong Secret Key**:
   - In production, use a strong, random secret key
   - Store the secret in environment variables, not in code or config files
   - Consider using different secrets for access and refresh tokens

2. **Keep Tokens Short-Lived**:
   - Access tokens should expire quickly (1 hour or less)
   - Use refresh tokens for obtaining new access tokens

3. **Include Minimal Claims**:
   - Only include necessary information in tokens
   - Avoid including sensitive data
   - Refresh tokens should contain less information than access tokens

4. **Implement Token Revocation**:
   - For high-security applications, maintain a blacklist of revoked tokens
   - Consider using Redis or another in-memory store for this purpose

### Password Security

1. **Use Strong Hashing Algorithms**:
   - bcrypt, Argon2, or scrypt are recommended
   - Never store passwords in plain text or with weak hashing

2. **Implement Password Complexity Rules**:
   - Minimum length (8+ characters)
   - Require a mix of character types
   - Validate both on client and server side

3. **Rate Limiting**:
   - Limit login attempts to prevent brute force attacks
   - Implement progressive delays or account lockouts

### Middleware Security

1. **Proper Error Messages**:
   - Don't reveal too much information in error messages
   - For auth failures, use generic messages like "Invalid email or password"

2. **CORS Configuration**:
   - In production, restrict CORS to specific origins
   - Be careful with allowing credentials in CORS

3. **HTTPS Only**:
   - In production, require HTTPS for all authentication endpoints
   - Set the Secure flag on cookies

## Common Authentication Patterns

### Basic Auth vs. JWT

This example uses JWT, but basic authentication (username/password with each request) is an alternative for simpler applications:

```
async fn basic_auth_middleware<B>(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Validate username and password
    if auth.username() == "admin" && auth.password() == "password" {
        Ok(next.run(request).await)
    } else {
        Err(AppError::unauthorized("Invalid credentials"))
    }
}
```

### Cookie-Based Authentication

For web applications, cookies can be more appropriate than bearer tokens:

```
async fn login_with_cookies(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    let token_response = auth_service.login(request).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        format!(
            "access_token={}; HttpOnly; Path=/; Max-Age={}; SameSite=Strict", 
            token_response.access_token, 
            token_response.expires_in
        ).parse().unwrap(),
    );
    
    Ok((
        StatusCode::OK,
        headers,
        Json(serde_json::json!({"status": "success"}))
    ))
}
```

### Social Login Integration

For OAuth 2.0 / OpenID Connect with providers like Google or GitHub:

```
async fn google_auth_callback(
    State(auth_service): State<Arc<AuthService>>,
    Query(params): Query<GoogleAuthParams>,
) -> Result<Redirect, AppError> {
    // Exchange code for tokens
    let google_tokens = auth_service.exchange_google_code(params.code).await?;
    
    // Get user info from Google
    let google_user = auth_service.get_google_user_info(&google_tokens.access_token).await?;
    
    // Find or create user
    let user = auth_service.find_or_create_social_user(
        "google",
        &google_user.sub,
        &google_user.email,
        &google_user.name,
    ).await?;
    
    // Generate tokens
    let tokens = auth_service.generate_tokens(&user)?;
    
    // Redirect to frontend with tokens
    Ok(Redirect::to(&format!("/auth/callback?token={}", tokens.access_token)))
}
```

## Refresh Token Handling

### Implementing Token Rotation

For enhanced security, implement token rotation where each refresh token can only be used once:

```
pub async fn refresh_with_rotation(&self, refresh_token: &str) -> Result<TokenResponse, AppError> {
    // Decode token
    let claims = self.decode_token(refresh_token)?;
    
    // Find user
    let user = self.user_service.find_by_id(&claims.sub).await?;
    
    // Optional: Check if token has been revoked
    if self.is_token_revoked(refresh_token).await? {
        return Err(AppError::unauthorized("Token has been revoked"));
    }
    
    // Generate new tokens
    let tokens = self.generate_tokens(&user)?;
    
    // Revoke the old refresh token
    self.revoke_token(refresh_token).await?;
    
    Ok(tokens)
}
```

### Handling Refresh Token Theft

Implement detection for potential refresh token theft:

```
pub async fn detect_token_reuse(&self, refresh_token: &str) -> Result<(), AppError> {
    let token_id = self.get_token_id(refresh_token)?;
    
    if self.is_token_used(token_id).await? {
        // Token has been used before - potential theft!
        // Revoke all tokens for this user
        let claims = self.decode_token(refresh_token)?;
        self.revoke_all_tokens_for_user(&claims.sub).await?;
        
        return Err(AppError::unauthorized("Security breach detected"));
    }
    
    // Mark token as used
    self.mark_token_used(token_id).await?;
    
    Ok(())
}
```

## Integrating with External Auth Providers

### OpenID Connect Integration

For integrating with Microsoft Entra ID (formerly Azure AD), Google, or other OIDC providers:

```
pub async fn authenticate_with_oidc(&self, provider: &str, code: &str) -> Result<TokenResponse, AppError> {
    // Get provider configuration
    let config = match provider {
        "google" => self.config.oidc.google.clone(),
        "microsoft" => self.config.oidc.microsoft.clone(),
        _ => return Err(AppError::bad_request("Unsupported provider")),
    };
    
    // Exchange authorization code for tokens
    let token_response = self.http_client
        .post(&config.token_endpoint)
        .form(&[
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .json::<OidcTokenResponse>()
        .await?;
    
    // Verify ID token
    let id_token_claims = self.verify_id_token(&token_response.id_token, &config)?;
    
    // Find or create user
    let user = self.find_or_create_oidc_user(
        provider,
        &id_token_claims.sub,
        &id_token_claims.email,
        &id_token_claims.name,
    ).await?;
    
    // Generate application tokens
    self.generate_tokens(&user)
}
```

### Multi-Factor Authentication

For implementing 2FA with TOTP (Time-based One-Time Password):

```
pub async fn verify_totp(&self, user_id: &str, token: &str) -> Result<bool, AppError> {
    // Get user's TOTP secret
    let user = self.user_service.find_by_id(user_id).await?;
    let totp_secret = user.totp_secret.ok_or_else(|| AppError::bad_request("TOTP not set up"))?;
    
    // Verify TOTP
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        totp_secret.as_bytes(),
    );
    
    // Check if token is valid (allowing one interval before/after for clock skew)
    Ok(totp.check(token, 1))
}
```

## Troubleshooting

### Common Issues

1. **Token Expired**:
   - Ensure client refreshes tokens before they expire
   - Implement token refresh logic on the client side
   - Make sure server and client clocks are synchronized

2. **Invalid Token Format**:
   - Check that the token is being sent correctly in the Authorization header
   - Format should be: `Authorization: Bearer <token>`
   - Ensure the token isn't URL-encoded or truncated

3. **Permission Denied**:
   - Verify the user has the required roles
   - Check role assignment in the database
   - Ensure roles are correctly included in the token claims

4. **CORS Issues**:
   - Configure CORS properly for your frontend
   - Ensure OPTIONS preflight requests are handled
   - Check for credential handling in CORS configuration

### Debugging Tips

1. **Token Inspection**:
   - Use tools like jwt.io to inspect and debug tokens
   - Never paste production tokens into public tools

2. **Logging**:
   - Add detailed logging for auth failures (in development)
   - Log token validation steps for debugging
   - Remove sensitive info from logs in production

3. **Error Handling**:
   - Return descriptive errors during development
   - Switch to generic errors in production
   - Include request IDs for correlation

## Next Steps

After implementing basic authentication:

1. **Add security headers** (Content-Security-Policy, X-XSS-Protection, etc.)
2. **Implement account lockout** for multiple failed login attempts
3. **Add user management features** (password reset, email verification)
4. **Implement multi-factor authentication**
5. **Set up monitoring for suspicious auth activity**
6. **Create a token blacklist** for immediate token revocation

## Related Examples

For more advanced usage and context, check out these related examples:

- [REST API Example](rest-api-example.md) for a complete API implementation
- [Error Handling Example](error-handling-example.md) for robust error handling
- [Custom Service Example](custom-service-example.md) for service architecture

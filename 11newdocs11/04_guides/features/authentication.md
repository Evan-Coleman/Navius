---
title: "Navius Authentication Guide"
description: "A comprehensive guide to implementing secure authentication in Navius applications, including Microsoft Entra integration, session management with Redis, and security best practices"
category: guides
tags:
  - authentication
  - security
  - microsoft-entra
  - redis
  - session-management
  - oauth2
  - jwt
related:
  - ../reference/api/authentication-api.md
  - ../guides/features/api-integration.md
  - ../reference/configuration/environment-variables.md
  - ../guides/deployment/security-checklist.md
last_updated: March 23, 2025
version: 1.0
---
# Navius Authentication Guide

This guide covers the authentication options available in Navius and how to implement them in your application.

## Overview

Navius provides several authentication methods out of the box:

1. **JWT-based authentication**
2. **OAuth2 integration**
3. **API key authentication**
4. **Microsoft Entra (formerly Azure AD) integration**
5. **Custom authentication schemes**

Each method can be configured and combined to suit your application's needs.

## JWT Authentication

JWT (JSON Web Token) authentication is the default method in Navius.

### Configuration

Configure JWT authentication in your `config.yaml` file:

```yaml
auth:
  jwt:
    enabled: true
    secret_key: "${JWT_SECRET}"
    algorithm: "HS256"
    token_expiration_minutes: 60
    refresh_token_expiration_days: 7
    issuer: "naviusframework.dev"
```

### Implementation

1. **Login endpoint**:

```rust
#[post("/login")]
pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate credentials
    let user = state.user_service.authenticate(
        &credentials.username,
        &credentials.password,
    ).await?;
    
    // Generate JWT token
    let token = state.auth_service.generate_token(&user)?;
    let refresh_token = state.auth_service.generate_refresh_token(&user)?;
    
    Ok(Json(AuthResponse {
        access_token: token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    }))
}
```

2. **Protect routes with middleware**:

```rust
// In your router setup
let protected_routes = Router::new()
    .route("/users", get(list_users))
    .route("/users/:id", get(get_user_by_id))
    .layer(JwtAuthLayer::new(
        state.config.auth.jwt.secret_key.clone(),
    ));
```

3. **Access the authenticated user**:

```rust
#[get("/profile")]
pub async fn get_profile(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_by_id(auth_user.id).await?;
    Ok(Json(user))
}
```

## OAuth2 Authentication

Navius supports OAuth2 integration with various providers.

### Configuration

```yaml
auth:
  oauth2:
    enabled: true
    providers:
      google:
        enabled: true
        client_id: "${GOOGLE_CLIENT_ID}"
        client_secret: "${GOOGLE_CLIENT_SECRET}"
        redirect_uri: "https://your-app.com/auth/google/callback"
        scopes:
          - "email"
          - "profile"
      github:
        enabled: true
        client_id: "${GITHUB_CLIENT_ID}"
        client_secret: "${GITHUB_CLIENT_SECRET}"
        redirect_uri: "https://your-app.com/auth/github/callback"
        scopes:
          - "user:email"
```

### Implementation

1. **Add OAuth2 routes**:

```rust
// In your router setup
let auth_routes = Router::new()
    .route("/auth/google/login", get(google_login))
    .route("/auth/google/callback", get(google_callback))
    .route("/auth/github/login", get(github_login))
    .route("/auth/github/callback", get(github_callback));
```

2. **Create provider-specific handlers**:

```rust
#[get("/auth/google/login")]
pub async fn google_login(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let oauth_client = state.oauth_service.get_provider("google");
    let (auth_url, csrf_token) = oauth_client.authorize_url();
    
    // Store CSRF token in cookie
    let cookie = Cookie::build("oauth_csrf", csrf_token.secret().clone())
        .path("/")
        .max_age(time::Duration::minutes(10))
        .http_only(true)
        .secure(true)
        .finish();
    
    (
        StatusCode::FOUND,
        [(header::SET_COOKIE, cookie.to_string())],
        [(header::LOCATION, auth_url.to_string())],
    )
}

#[get("/auth/google/callback")]
pub async fn google_callback(
    Query(params): Query<OAuthCallbackParams>,
    cookies: Cookies,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Validate CSRF token
    let csrf_cookie = cookies.get("oauth_csrf")
        .ok_or(AppError::AuthenticationError("Missing CSRF token".into()))?;
    
    let oauth_client = state.oauth_service.get_provider("google");
    let token = oauth_client.exchange_code(
        params.code,
        csrf_cookie.value(),
    ).await?;
    
    // Get user info
    let user_info = oauth_client.get_user_info(&token).await?;
    
    // Find or create user
    let user = state.user_service.find_or_create_from_oauth(
        "google",
        &user_info.id,
        &user_info.email,
        &user_info.name,
    ).await?;
    
    // Generate JWT token
    let jwt_token = state.auth_service.generate_token(&user)?;
    
    // Redirect to frontend with token
    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, format!("/auth/success?token={}", jwt_token))],
    ))
}
```

## API Key Authentication

For service-to-service or programmatic API access, Navius provides API key authentication.

### Configuration

```yaml
auth:
  api_key:
    enabled: true
    header_name: "X-API-Key"
    query_param_name: "api_key"
```

### Implementation

1. **Create API keys**:

```rust
#[post("/api-keys")]
pub async fn create_api_key(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKey>, AppError> {
    // Ensure user has permission
    if !auth_user.has_permission("api_keys:create") {
        return Err(AppError::PermissionDenied);
    }
    
    // Create API key
    let api_key = state.api_key_service.create(
        auth_user.id,
        &payload.name,
        payload.expiration_days,
    ).await?;
    
    Ok(Json(api_key))
}
```

2. **Apply API key middleware**:

```rust
// In your router setup
let api_routes = Router::new()
    .route("/api/v1/data", get(get_data))
    .layer(ApiKeyLayer::new(state.api_key_service.clone()));
```

## Microsoft Entra (Azure AD) Integration

Navius provides specialized support for Microsoft Entra ID integration.

### Configuration

```yaml
auth:
  microsoft_entra:
    enabled: true
    tenant_id: "${AZURE_TENANT_ID}"
    client_id: "${AZURE_CLIENT_ID}"
    client_secret: "${AZURE_CLIENT_SECRET}"
    redirect_uri: "https://your-app.com/auth/microsoft/callback"
    scopes:
      - "openid"
      - "profile"
      - "email"
    graph_api:
      enabled: true
      scopes:
        - "User.Read"
```

### Implementation

1. **Microsoft login routes**:

```rust
#[get("/auth/microsoft/login")]
pub async fn microsoft_login(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let auth_url = state.microsoft_auth_service.get_authorization_url();
    
    (
        StatusCode::FOUND,
        [(header::LOCATION, auth_url.to_string())],
    )
}

#[get("/auth/microsoft/callback")]
pub async fn microsoft_callback(
    Query(params): Query<MicrosoftAuthCallbackParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Exchange authorization code for token
    let token = state.microsoft_auth_service
        .exchange_code_for_token(&params.code)
        .await?;
    
    // Get user info from Microsoft Graph API
    let user_info = state.microsoft_auth_service
        .get_user_info(&token)
        .await?;
    
    // Find or create user
    let user = state.user_service
        .find_or_create_from_microsoft(&user_info)
        .await?;
    
    // Generate JWT token
    let jwt_token = state.auth_service.generate_token(&user)?;
    
    // Redirect to frontend with token
    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, format!("/auth/success?token={}", jwt_token))],
    ))
}
```

## Custom Authentication Schemes

For specialized authentication needs, Navius allows implementing custom authentication schemes.

### Implementation

1. **Create a custom extractor**:

```rust
pub struct CustomAuthUser {
    pub id: Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

#[async_trait]
impl FromRequestParts<AppState> for CustomAuthUser {
    type Rejection = AppError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Custom authentication logic
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AppError::Unauthorized("Missing authentication".into()))?;
        
        // Parse and validate the header
        let header_value = auth_header.to_str()?;
        
        // Your custom validation logic
        if !header_value.starts_with("Custom ") {
            return Err(AppError::Unauthorized("Invalid auth scheme".into()));
        }
        
        let token = header_value[7..].to_string();
        
        // Validate token and get user
        let user = state.custom_auth_service.validate_token(&token).await?;
        
        Ok(CustomAuthUser {
            id: user.id,
            username: user.username,
            roles: user.roles,
        })
    }
}
```

2. **Use the custom extractor in handlers**:

```rust
#[get("/custom-auth-resource")]
pub async fn get_protected_resource(
    auth: CustomAuthUser,
) -> Result<Json<Resource>, AppError> {
    // Use auth.id, auth.username, auth.roles
    // ...
    
    Ok(Json(resource))
}
```

## Role-Based Access Control

Navius provides a built-in RBAC system that integrates with all authentication methods.

### Configuration

```yaml
auth:
  rbac:
    enabled: true
    default_role: "user"
    roles:
      admin:
        permissions:
          - "users:read"
          - "users:write"
          - "settings:read"
          - "settings:write"
      user:
        permissions:
          - "users:read:self"
          - "settings:read:self"
          - "settings:write:self"
```

### Implementation

1. **Check permissions in handlers**:

```rust
#[get("/admin/settings")]
pub async fn admin_settings(
    auth_user: AuthUser,
) -> Result<Json<Settings>, AppError> {
    // Check if user has the required permission
    if !auth_user.has_permission("settings:read") {
        return Err(AppError::PermissionDenied);
    }
    
    // Proceed with handler logic
    // ...
    
    Ok(Json(settings))
}
```

2. **Or use permission middleware**:

```rust
// In your router setup
let admin_routes = Router::new()
    .route("/admin/users", get(list_users))
    .route("/admin/settings", get(admin_settings))
    .layer(RequirePermissionLayer::new("admin:access"));
```

## Multi-Factor Authentication

Navius supports multi-factor authentication (MFA) via TOTP (Time-based One-Time Password).

### Configuration

```yaml
auth:
  mfa:
    enabled: true
    totp:
      enabled: true
      issuer: "Navius App"
      digits: 6
      period: 30
```

### Implementation

1. **Enable MFA for a user**:

```rust
#[post("/mfa/enable")]
pub async fn enable_mfa(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<MfaSetupResponse>, AppError> {
    // Generate TOTP secret
    let (secret, qr_code) = state.mfa_service.generate_totp_secret(
        &auth_user.username,
    )?;
    
    // Store secret temporarily (not yet activated)
    state.mfa_service.store_pending_secret(auth_user.id, &secret).await?;
    
    Ok(Json(MfaSetupResponse {
        secret,
        qr_code,
    }))
}

#[post("/mfa/verify")]
pub async fn verify_mfa(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<VerifyMfaRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    // Verify TOTP code and activate MFA
    state.mfa_service
        .verify_and_activate(auth_user.id, &payload.code)
        .await?;
    
    Ok(Json(MfaVerifyResponse {
        enabled: true,
    }))
}
```

2. **Login with MFA**:

```rust
#[post("/login")]
pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate credentials
    let user = state.user_service.authenticate(
        &credentials.username,
        &credentials.password,
    ).await?;
    
    // Check if MFA is required
    if user.mfa_enabled {
        // Return challenge response
        return Ok(Json(AuthResponse {
            requires_mfa: true,
            mfa_token: state.auth_service.generate_mfa_token(user.id)?,
            ..Default::default()
        }));
    }
    
    // Generate JWT token
    let token = state.auth_service.generate_token(&user)?;
    let refresh_token = state.auth_service.generate_refresh_token(&user)?;
    
    Ok(Json(AuthResponse {
        access_token: token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        requires_mfa: false,
    }))
}

#[post("/login/mfa")]
pub async fn login_mfa(
    State(state): State<AppState>,
    Json(payload): Json<MfaLoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate MFA token to get user ID
    let user_id = state.auth_service.validate_mfa_token(&payload.mfa_token)?;
    
    // Verify TOTP code
    let valid = state.mfa_service.verify_code(user_id, &payload.code).await?;
    if !valid {
        return Err(AppError::InvalidMfaCode);
    }
    
    // Get user
    let user = state.user_service.get_by_id(user_id).await?;
    
    // Generate JWT token
    let token = state.auth_service.generate_token(&user)?;
    let refresh_token = state.auth_service.generate_refresh_token(&user)?;
    
    Ok(Json(AuthResponse {
        access_token: token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        requires_mfa: false,
    }))
}
```

## Session-Based Authentication

For traditional web applications, Navius also supports session-based authentication.

### Configuration

```yaml
auth:
  session:
    enabled: true
    cookie_name: "navius_session"
    cookie_secure: true
    cookie_http_only: true
    cookie_same_site: "Lax"
    expiry_hours: 24
    redis:
      enabled: true
      url: "${REDIS_URL}"
```

### Implementation

1. **Setup session middleware**:

```rust
// In your main.rs
let session_store = RedisSessionStore::new(
    &config.auth.session.redis.url
).await?;

let app = Router::new()
    // ... routes
    .layer(
        SessionLayer::new(
            session_store,
            &config.auth.session.secret.as_bytes(),
        )
        .with_secure(config.auth.session.cookie_secure)
        .with_http_only(config.auth.session.cookie_http_only)
        .with_same_site(config.auth.session.cookie_same_site)
        .with_expiry(time::Duration::hours(
            config.auth.session.expiry_hours
        ))
    );
```

2. **Session-based login**:

```rust
#[post("/login")]
pub async fn login(
    mut session: Session,
    State(state): State<AppState>,
    Form(credentials): Form<LoginCredentials>,
) -> Result<impl IntoResponse, AppError> {
    // Validate credentials
    let user = state.user_service.authenticate(
        &credentials.username,
        &credentials.password,
    ).await?;
    
    // Store user in session
    session.insert("user_id", user.id)?;
    session.insert("username", user.username.clone())?;
    
    // Redirect to dashboard
    Ok(Redirect::to("/dashboard"))
}
```

3. **Access session data**:

```rust
#[get("/dashboard")]
pub async fn dashboard(
    session: Session,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Get user from session
    let user_id: Uuid = session.get("user_id")
        .ok_or(AppError::Unauthorized("Not logged in".into()))?;
    
    let user = state.user_service.get_by_id(user_id).await?;
    
    Ok(HtmlTemplate::new("dashboard", json!({
        "user": user,
    })))
}
```

## Testing Authentication

Navius provides utilities for testing authenticated endpoints.

```rust
#[tokio::test]
async fn test_protected_endpoint() {
    // Create test app
    let app = TestApp::new().await;
    
    // Create a test user
    let user = app.create_test_user("test@example.com", "password123").await;
    
    // Test with authentication
    let response = app
        .get("/api/profile")
        .with_auth_user(&user) // Helper method to add auth headers
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test without authentication
    let response = app
        .get("/api/profile")
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

## Conclusion

Navius provides a comprehensive and flexible authentication system that can be adapted to various application needs. By combining different authentication methods and access control mechanisms, you can create a secure application with the right balance of security and user experience.

For more advanced use cases, refer to the following resources:
- [Advanced JWT Configuration](advanced_jwt.md)
- [OAuth2 Provider Implementation Guide](oauth2_providers.md)
- [Custom Authentication Schemes](custom_auth.md) 

## Related Documents
- [Installation Guide](../../01_getting_started/installation.md) - How to install the application
- [Development Workflow](../development/development-workflow.md) - Development best practices


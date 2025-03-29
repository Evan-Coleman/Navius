---
title: "Authentication Implementation Guide"
description: "Comprehensive guide for implementing secure authentication in Navius applications, including Microsoft Entra integration and multi-factor authentication"
category: "Guides"
tags: ["security", "authentication", "Microsoft Entra", "MFA", "tokens", "sessions"]
last_updated: "April 6, 2025"
version: "1.0"
---

# Authentication Implementation Guide

## Overview

This guide provides detailed instructions for implementing secure authentication in Navius applications. Authentication is a critical security component that verifies the identity of users before granting access to protected resources.

## Authentication Concepts

### Authentication vs. Authorization

- **Authentication** (covered in this guide) verifies who the user is
- **Authorization** (covered in [Authorization Guide](./authorization-guide.md)) determines what the user can do

### Authentication Factors

Secure authentication typically involves one or more of these factors:

1. **Knowledge** - Something the user knows (password, PIN)
2. **Possession** - Something the user has (mobile device, security key)
3. **Inherence** - Something the user is (fingerprint, facial recognition)

Multi-factor authentication (MFA) combines at least two different factors.

## Authentication Options in Navius

Navius supports multiple authentication providers:

1. **Microsoft Entra ID** (formerly Azure AD) - Primary authentication provider
2. **Local Authentication** - Username/password authentication for development
3. **Custom Providers** - Support for implementing custom authentication logic

## Microsoft Entra Integration

### Configuration

Configure Microsoft Entra in your `config/default.yaml`:

```yaml
auth:
  provider: "entra"
  entra:
    tenant_id: "your-tenant-id"
    client_id: "your-client-id"
    client_secret: "your-client-secret"
    redirect_uri: "https://your-app.com/auth/callback"
    scopes: ["openid", "profile", "email"]
```

### Implementation

Implement the authentication flow:

```rust
use navius::auth::providers::EntraAuthProvider;
use navius::auth::{AuthConfig, AuthProvider};

async fn configure_auth(config: &Config) -> Result<impl AuthProvider, Error> {
    let auth_config = AuthConfig::from_config(config)?;
    let provider = EntraAuthProvider::new(auth_config)?;
    Ok(provider)
}

// In your router setup
async fn configure_routes(app: Router, auth_provider: impl AuthProvider) -> Router {
    app.route("/login", get(login_handler))
       .route("/auth/callback", get(auth_callback_handler))
       .route("/logout", get(logout_handler))
       .with_state(AppState { auth_provider })
}

async fn login_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Redirect to Microsoft Entra login
    let auth_url = state.auth_provider.get_authorization_url()?;
    Redirect::to(&auth_url)
}

async fn auth_callback_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    cookies: Cookies,
) -> impl IntoResponse {
    // Handle auth callback from Microsoft Entra
    let code = params.get("code").ok_or(Error::InvalidAuthRequest)?;
    let token = state.auth_provider.exchange_code_for_token(code).await?;
    
    // Set secure session cookie
    let session = cookies.create_session(&token)?;
    
    Redirect::to("/dashboard")
}
```

### Testing Microsoft Entra Integration

For testing, use the development mode with mock responses:

```yaml
# config/development.yaml
auth:
  provider: "entra"
  entra:
    mock_enabled: true
    mock_users:
      - email: "test@example.com"
        name: "Test User"
        id: "test-user-id"
        roles: ["user"]
```

## Local Authentication

For development or when Microsoft Entra is not available:

```rust
use navius::auth::providers::LocalAuthProvider;

async fn configure_local_auth() -> impl AuthProvider {
    let provider = LocalAuthProvider::new()
        .add_user("admin", "secure-password", vec!["admin"])
        .add_user("user", "user-password", vec!["user"]);
    
    provider
}
```

## Implementing Multi-Factor Authentication

### TOTP (Time-based One-Time Password)

```rust
use navius::auth::mfa::{TotpService, TotpConfig};

// Initialize TOTP service
let totp_service = TotpService::new(TotpConfig {
    issuer: "Your App Name".to_string(),
    digits: 6,
    period: 30,
    algorithm: "SHA1".to_string(),
});

// Generate secret for a user
async fn setup_mfa(user_id: Uuid, totp_service: &TotpService) -> Result<String, Error> {
    let secret = totp_service.generate_secret();
    let provisioning_uri = totp_service.get_provisioning_uri(&user.email, &secret);
    
    // Store secret in database
    store_mfa_secret(user_id, &secret).await?;
    
    // Return provisioning URI for QR code generation
    Ok(provisioning_uri)
}

// Verify TOTP code
async fn verify_totp(user_id: Uuid, code: &str, totp_service: &TotpService) -> Result<bool, Error> {
    let user = get_user(user_id).await?;
    let is_valid = totp_service.verify(&user.mfa_secret, code)?;
    Ok(is_valid)
}
```

### WebAuthn (Passwordless) Support

For implementing WebAuthn (FIDO2) passwordless authentication:

```rust
use navius::auth::webauthn::{WebAuthnService, WebAuthnConfig};

// Initialize WebAuthn service
let webauthn_service = WebAuthnService::new(WebAuthnConfig {
    rp_id: "your-app.com".to_string(),
    rp_name: "Your App Name".to_string(),
    origin: "https://your-app.com".to_string(),
});

// Register a new credential
async fn register_credential(
    user_id: Uuid,
    credential: CredentialCreationResponse,
    webauthn_service: &WebAuthnService,
) -> Result<(), Error> {
    let credential = webauthn_service.register_credential(user_id, credential).await?;
    store_credential(user_id, credential).await?;
    Ok(())
}

// Authenticate with a credential
async fn authenticate(
    credential: CredentialAssertionResponse,
    webauthn_service: &WebAuthnService,
) -> Result<Uuid, Error> {
    let user_id = webauthn_service.authenticate(credential).await?;
    Ok(user_id)
}
```

## Token Management

### Token Types

Navius uses several token types:

1. **ID Token**: Contains user identity information
2. **Access Token**: Grants access to protected resources
3. **Refresh Token**: Used to obtain new access tokens

### Token Storage

Securely store tokens:

```rust
use navius::auth::tokens::{TokenStore, RedisTokenStore};

// Initialize token store
let token_store = RedisTokenStore::new(redis_connection);

// Store a token
async fn store_token(user_id: Uuid, token: &AuthToken, token_store: &impl TokenStore) -> Result<(), Error> {
    token_store.store(user_id, token).await?;
    Ok(())
}

// Retrieve a token
async fn get_token(user_id: Uuid, token_store: &impl TokenStore) -> Result<AuthToken, Error> {
    let token = token_store.get(user_id).await?;
    Ok(token)
}

// Revoke a token
async fn revoke_token(user_id: Uuid, token_store: &impl TokenStore) -> Result<(), Error> {
    token_store.revoke(user_id).await?;
    Ok(())
}
```

### Token Refresh

Implement token refresh to maintain sessions:

```rust
async fn refresh_token(
    user_id: Uuid, 
    refresh_token: &str,
    auth_provider: &impl AuthProvider,
    token_store: &impl TokenStore,
) -> Result<AuthToken, Error> {
    let new_token = auth_provider.refresh_token(refresh_token).await?;
    token_store.store(user_id, &new_token).await?;
    Ok(new_token)
}
```

## Session Management

### Session Configuration

Configure secure sessions:

```rust
use navius::auth::session::{SessionManager, SessionConfig};

let session_manager = SessionManager::new(SessionConfig {
    cookie_name: "session".to_string(),
    cookie_domain: Some("your-app.com".to_string()),
    cookie_path: "/".to_string(),
    cookie_secure: true,
    cookie_http_only: true,
    cookie_same_site: SameSite::Lax,
    expiry: Duration::hours(2),
});
```

### Session Creation and Validation

```rust
// Create a new session
async fn create_session(
    user_id: Uuid,
    token: &AuthToken,
    session_manager: &SessionManager,
) -> Result<Cookie, Error> {
    let session = session_manager.create_session(user_id, token)?;
    Ok(session)
}

// Validate a session
async fn validate_session(
    cookies: &Cookies,
    session_manager: &SessionManager,
) -> Result<Uuid, Error> {
    let user_id = session_manager.validate_session(cookies)?;
    Ok(user_id)
}

// End a session
async fn end_session(
    cookies: &mut Cookies,
    session_manager: &SessionManager,
) -> Result<(), Error> {
    session_manager.end_session(cookies)?;
    Ok(())
}
```

## Authentication Middleware

Create middleware to protect routes:

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    cookies: Cookies,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match state.session_manager.validate_session(&cookies) {
        Ok(user_id) => {
            // Add user ID to request extensions
            let mut req = req;
            req.extensions_mut().insert(UserId(user_id));
            
            // Continue to handler
            Ok(next.run(req).await)
        }
        Err(_) => {
            // Redirect to login page
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// Apply middleware to protected routes
let app = Router::new()
    .route("/", get(public_handler))
    .route("/dashboard", get(dashboard_handler))
    .route_layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware))
    .with_state(app_state);
```

## Security Considerations

### Password Policies

Implement strong password policies:

```rust
use navius::auth::password::{PasswordPolicy, PasswordValidator};

let password_policy = PasswordPolicy {
    min_length: 12,
    require_uppercase: true,
    require_lowercase: true,
    require_digits: true,
    require_special_chars: true,
    max_repeated_chars: 3,
};

let validator = PasswordValidator::new(password_policy);

fn validate_password(password: &str) -> Result<(), String> {
    validator.validate(password)
}
```

### Brute Force Protection

Implement account lockout after failed attempts:

```rust
use navius::auth::protection::{BruteForceProtection, BruteForceConfig};

let protection = BruteForceProtection::new(BruteForceConfig {
    max_attempts: 5,
    lockout_duration: Duration::minutes(30),
    attempt_reset: Duration::hours(24),
});

async fn check_login_attempt(
    username: &str,
    ip_address: &str,
    protection: &BruteForceProtection,
) -> Result<(), Error> {
    protection.check_attempts(username, ip_address).await?;
    Ok(())
}

async fn record_failed_attempt(
    username: &str,
    ip_address: &str,
    protection: &BruteForceProtection,
) -> Result<(), Error> {
    protection.record_failed_attempt(username, ip_address).await?;
    Ok(())
}

async fn reset_attempts(
    username: &str,
    protection: &BruteForceProtection,
) -> Result<(), Error> {
    protection.reset_attempts(username).await?;
    Ok(())
}
```

### Secure Logout

Implement secure logout functionality:

```rust
async fn logout_handler(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    // End session
    state.session_manager.end_session(&cookies)?;
    
    // Revoke token if using OAuth
    if let Some(user_id) = cookies.get_user_id() {
        state.token_store.revoke(user_id).await?;
    }
    
    Redirect::to("/login")
}
```

## Testing Authentication

### Unit Testing

Test authentication components:

```rust
#[tokio::test]
async fn test_token_store() {
    let store = InMemoryTokenStore::new();
    let user_id = Uuid::new_v4();
    let token = AuthToken::new("access", "refresh", "id", 3600);
    
    store.store(user_id, &token).await.unwrap();
    let retrieved = store.get(user_id).await.unwrap();
    
    assert_eq!(token.access_token, retrieved.access_token);
}
```

### Integration Testing

Test the authentication flow:

```rust
#[tokio::test]
async fn test_auth_flow() {
    // Setup test app with mock auth provider
    let app = test_app().await;
    
    // Test login redirect
    let response = app.get("/login").send().await;
    assert_eq!(response.status(), StatusCode::FOUND);
    
    // Test callback with mock code
    let response = app.get("/auth/callback?code=test-code").send().await;
    assert_eq!(response.status(), StatusCode::FOUND);
    
    // Test accessing protected route
    let response = app.get("/dashboard")
        .header("Cookie", "session=test-session")
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}
```

## Implementing Single Sign-On (SSO)

Enable SSO across multiple applications:

```rust
auth:
  provider: "entra"
  entra:
    tenant_id: "your-tenant-id"
    client_id: "your-client-id"
    client_secret: "your-client-secret"
    redirect_uri: "https://your-app.com/auth/callback"
    scopes: ["openid", "profile", "email"]
    enable_sso: true
    sso_domains: ["yourdomain.com"]
```

## Troubleshooting

### Common Issues

1. **Redirect URI Mismatch**: Ensure the redirect URI in your application config exactly matches the one registered in Microsoft Entra.
2. **Token Expiration**: Implement proper token refresh handling.
3. **Clock Skew**: TOTP validation can fail if server clocks are not synchronized.
4. **CORS Issues**: Ensure proper CORS configuration when authenticating from SPAs.

### Debugging Authentication

Enable debug logging for authentication:

```rust
// Initialize logger with auth debug enabled
tracing_subscriber::fmt()
    .with_env_filter("navius::auth=debug")
    .init();
```

## Related Resources

- [Security Best Practices](./security-best-practices.md)
- [Authorization Guide](./authorization-guide.md)
- [Authentication Example](../../02_examples/authentication-example.md)
- [Microsoft Entra Documentation](https://learn.microsoft.com/en-us/entra/identity-platform/)
- [OWASP Authentication Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html) 
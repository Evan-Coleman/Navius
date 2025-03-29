---
title: "Authentication API Reference"
description: "Reference documentation for Navius Authentication API endpoints, request/response formats, and integration patterns"
category: "Reference"
tags: ["api", "authentication", "security", "jwt", "oauth", "endpoints"]
last_updated: "April 9, 2025"
version: "1.0"
---

# Authentication API Reference

## Overview

The Navius Authentication API provides endpoints for user authentication, token management, and session control. It supports multiple authentication methods including JWT-based authentication, OAuth2 with Microsoft Entra, and API key authentication.

This reference document details all endpoints, data structures, and integration patterns for implementing authentication in Navius applications.

## Authentication Methods

Navius supports the following authentication methods:

| Method | Use Case | Security Level | Configuration |
|--------|----------|----------------|--------------|
| JWT | General purpose authentication | High | Required JWT secret |
| Microsoft Entra | Enterprise authentication | Very High | Requires tenant configuration |
| API Keys | Service-to-service | Medium | Requires key management |
| Session Cookies | Web applications | Medium-High | Requires session configuration |

## API Endpoints

### User Authentication

#### `POST /auth/login`

Authenticates a user and returns a JWT token.

**Request Body:**

```json
{
  "username": "user@example.com",
  "password": "secure_password"
}
```

**Response (200 OK):**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

**Error Responses:**

- `401 Unauthorized`: Invalid credentials
- `403 Forbidden`: Account locked or disabled
- `429 Too Many Requests`: Rate limit exceeded

**Curl Example:**

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user@example.com","password":"secure_password"}'
```

**Code Example:**

```rust
// Client-side authentication request
async fn login(client: &Client, username: &str, password: &str) -> Result<TokenResponse> {
    let response = client
        .post("http://localhost:3000/auth/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await?;
        
    if response.status().is_success() {
        Ok(response.json::<TokenResponse>().await?)
    } else {
        Err(format!("Authentication failed: {}", response.status()).into())
    }
}
```

#### `POST /auth/refresh`

Refreshes an expired JWT token using a refresh token.

**Request Body:**

```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (200 OK):**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

**Error Responses:**

- `401 Unauthorized`: Invalid refresh token
- `403 Forbidden`: Refresh token revoked

#### `POST /auth/logout`

Invalidates the current session or token.

**Request Headers:**

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response (200 OK):**

```json
{
  "message": "Successfully logged out"
}
```

### Microsoft Entra Integration

#### `GET /auth/entra/login`

Initiates Microsoft Entra authentication flow.

**Query Parameters:**

- `redirect_uri` (required): URL to redirect after authentication
- `state` (optional): State parameter for CSRF protection

**Response:**

Redirects to Microsoft Entra login page.

#### `GET /auth/entra/callback`

Callback endpoint for Microsoft Entra authentication.

**Query Parameters:**

- `code` (required): Authorization code from Microsoft Entra
- `state` (optional): State parameter for CSRF protection

**Response:**

Redirects to the original `redirect_uri` with token information.

### API Key Management

#### `POST /auth/apikeys`

Creates a new API key for service-to-service authentication.

**Request Headers:**

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Request Body:**

```json
{
  "name": "Service Integration Key",
  "permissions": ["read:users", "write:data"],
  "expiresIn": 2592000  // 30 days in seconds
}
```

**Response (201 Created):**

```json
{
  "id": "api_123456789",
  "key": "sk_live_abcdefghijklmnopqrstuvwxyz123456789",
  "name": "Service Integration Key",
  "permissions": ["read:users", "write:data"],
  "createdAt": "2025-04-09T10:15:30Z",
  "expiresAt": "2025-05-09T10:15:30Z"
}
```

## Data Models

### TokenResponse

```rust
/// Response containing authentication tokens
struct TokenResponse {
    /// JWT access token
    token: String,
    
    /// JWT refresh token
    refresh_token: String,
    
    /// Token validity in seconds
    expires_in: u64,
    
    /// Token type (always "Bearer")
    token_type: String,
}
```

### ApiKey

```rust
/// API key for service-to-service authentication
struct ApiKey {
    /// Unique identifier
    id: String,
    
    /// Secret key (only returned on creation)
    key: Option<String>,
    
    /// Display name
    name: String,
    
    /// List of permissions
    permissions: Vec<String>,
    
    /// Creation timestamp
    created_at: DateTime<Utc>,
    
    /// Expiration timestamp
    expires_at: DateTime<Utc>,
}
```

## Integration Patterns

### JWT Authentication Flow

1. Client calls `/auth/login` with credentials
2. Server validates credentials and returns JWT + refresh token
3. Client stores tokens and includes JWT in subsequent requests
4. When JWT expires, client uses refresh token to get a new JWT
5. For logout, client calls `/auth/logout` and discards tokens

```rust
// Server-side handler for protected routes
async fn protected_route(
    auth: AuthExtractor,
    // other parameters
) -> Result<impl IntoResponse> {
    // Auth middleware extracts and validates JWT automatically
    // If JWT is invalid, route is not reached
    
    // Access authenticated user
    let user_id = auth.user_id;
    
    // Access user permissions
    if !auth.has_permission("read:resource") {
        return Err(AppError::forbidden("Insufficient permissions"));
    }
    
    // Handle the actual request
    Ok(Json(/* response data */))
}
```

### Microsoft Entra (OAuth2) Flow

1. Redirect user to `/auth/entra/login` with appropriate parameters
2. User authenticates with Microsoft
3. Microsoft redirects to `/auth/entra/callback`
4. Server validates the code and issues JWT tokens
5. Application uses the JWT tokens as in the standard JWT flow

```rust
// Client-side Entra redirect
fn redirect_to_entra_login(redirect_uri: &str) -> String {
    format!(
        "/auth/entra/login?redirect_uri={}&state={}", 
        urlencoding::encode(redirect_uri),
        generate_random_state()
    )
}
```

### API Key Authentication

1. Administrator creates API key via `/auth/apikeys`
2. Service includes API key in requests via header
3. Server validates API key and permissions
4. For security, rotate keys periodically

```rust
// Including API key in request header
let response = client
    .get("http://api.example.com/resource")
    .header("X-API-Key", "sk_live_abcdefghijklmnopqrstuvwxyz123456789")
    .send()
    .await?;
```

## Configuration

Authentication can be configured in `config/auth.yaml`:

```yaml
auth:
  jwt:
    secret: "${JWT_SECRET}"
    expiration: 3600  # 1 hour
    refresh_expiration: 2592000  # 30 days
  
  entra:
    tenant_id: "${ENTRA_TENANT_ID}"
    client_id: "${ENTRA_CLIENT_ID}"
    client_secret: "${ENTRA_CLIENT_SECRET}"
    redirect_uri: "http://localhost:3000/auth/entra/callback"
  
  api_keys:
    enabled: true
    max_per_user: 10
    default_expiration: 2592000  # 30 days
```

## Best Practices

### Security Recommendations

1. **Use HTTPS**: Always use HTTPS for all authentication endpoints
2. **Token Storage**: Store tokens securely (use HttpOnly cookies for web apps)
3. **Short Expiration**: Keep JWT tokens short-lived (1 hour or less)
4. **Refresh Token Rotation**: Issue new refresh tokens with each refresh
5. **API Key Handling**: Treat API keys as sensitive credentials
6. **Permission Validation**: Always validate permissions, not just authentication

### Common Pitfalls

1. **JWT in LocalStorage**: Avoid storing JWTs in localStorage (vulnerable to XSS)
2. **Missing CSRF Protection**: Always use state parameter with OAuth flows
3. **Hardcoded Secrets**: Never hardcode secrets in client-side code
4. **Skipping Validation**: Always validate JWTs, even in internal services
5. **Weak Tokens**: Ensure proper entropy in tokens and use proper algorithms

## Troubleshooting

### Common Issues

1. **"Invalid token" errors**: Check token expiration and signature algorithm
2. **CORS errors**: Ensure authentication endpoints have proper CORS configuration
3. **Refresh token not working**: Verify refresh token hasn't expired or been revoked
4. **Rate limiting**: Check if you're hitting rate limits on authentication endpoints

### Debugging

Enable detailed authentication logs by setting:

```
RUST_LOG=navius::auth=debug
```

This will show detailed information about token validation, including reasons for rejection.

## Related Resources

- [Authentication Implementation Guide](../../04_guides/security/authentication-implementation.md)
- [Microsoft Entra Integration Guide](../../04_guides/integrations/microsoft-entra.md)
- [API Security Guide](../../04_guides/security/api-security.md)
- [JWT Standard](https://datatracker.ietf.org/doc/html/rfc7519)

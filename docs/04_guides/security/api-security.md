---
title: "API Security Guide"
description: "Comprehensive guide for securing API endpoints in Navius applications, covering authentication, authorization, input validation, and API-specific security concerns"
category: "Guides"
tags: ["security", "API", "REST", "endpoints", "validation", "rate limiting", "OWASP"]
last_updated: "April 7, 2025"
version: "1.0"
---

# API Security Guide

## Overview

This guide provides detailed instructions for securing API endpoints in Navius applications. APIs are critical entry points to your application that require robust security measures to protect your data and services.

## API Security Fundamentals

### Common API Security Threats

- **Broken Authentication**: Weak authentication allowing unauthorized access
- **Excessive Data Exposure**: Returning excessive data in API responses
- **Broken Object Level Authorization**: Improper access controls for resources
- **Mass Assignment**: Client-provided data modifying sensitive properties
- **Injection Attacks**: SQL, NoSQL, command injection via API inputs
- **Improper Assets Management**: Exposed debug endpoints or outdated APIs
- **API Abuse**: Excessive requests that impact availability

### API Security Principles

1. **Defense in Depth**: Multiple security layers
2. **Least Privilege**: Limit access to necessary resources
3. **Zero Trust**: Verify every request regardless of source
4. **Secure by Default**: Security controls enabled by default
5. **Fail Securely**: Errors default to secure state

## Secure API Authentication

### API Key Authentication

Configure API key authentication:

```yaml
# config/default.yaml
api:
  auth:
    type: "apikey"
    header_name: "X-API-Key"
    key_validation: "database"  # or "static", "redis"
    rate_limiting:
      enabled: true
      limit: 100
      window_seconds: 60
```

Implement API key validation:

```rust
use navius::api::auth::{ApiKeyValidator, DatabaseApiKeyValidator};

// Create API key validator
let api_key_validator = DatabaseApiKeyValidator::new(db_pool).await?;

// API key middleware
async fn api_key_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract API key from header
    let api_key = req.headers()
        .get(state.config.api.auth.header_name.as_str())
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate API key
    let client_info = state.api_key_validator
        .validate(api_key)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add client info to request extensions
    let mut req = req;
    req.extensions_mut().insert(client_info);
    
    // Continue to handler
    Ok(next.run(req).await)
}

// Generate new API key
async fn generate_api_key(
    org_id: Uuid,
    permissions: Vec<String>,
    api_key_service: &ApiKeyService,
) -> Result<ApiKey, Error> {
    let api_key = api_key_service.generate(org_id, permissions).await?;
    Ok(api_key)
}

// Revoke API key
async fn revoke_api_key(
    key_id: Uuid,
    api_key_service: &ApiKeyService,
) -> Result<(), Error> {
    api_key_service.revoke(key_id).await?;
    Ok(())
}
```

### Bearer Token Authentication

Implement JWT-based authentication:

```rust
use navius::api::auth::{JwtValidator, JwtConfig};

// Create JWT validator
let jwt_config = JwtConfig {
    issuer: "navius-api".to_string(),
    audience: "navius-client".to_string(),
    key_id: "current-signing-key".to_string(),
    public_key_path: "/path/to/public.pem".to_string(),
};

let jwt_validator = JwtValidator::new(jwt_config);

// JWT middleware
async fn jwt_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = req.headers()
        .get(HeaderName::from_static("authorization"))
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate JWT
    let claims = state.jwt_validator
        .validate(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add claims to request extensions
    let mut req = req;
    req.extensions_mut().insert(claims);
    
    // Continue to handler
    Ok(next.run(req).await)
}
```

### OAuth 2.0 and OpenID Connect

Configure OAuth 2.0:

```yaml
# config/default.yaml
api:
  auth:
    type: "oauth2"
    provider: "entra"
    entra:
      tenant_id: "your-tenant-id"
      client_id: "your-client-id"
      jwks_uri: "https://login.microsoftonline.com/{tenant_id}/discovery/v2.0/keys"
      issuer: "https://login.microsoftonline.com/{tenant_id}/v2.0"
      audience: "api://your-app-id"
```

Implement OAuth 2.0 validation:

```rust
use navius::api::auth::{OAuth2Validator, OAuth2Config};

// Create OAuth2 validator
let oauth2_config = OAuth2Config::from_config(&config)?;
let oauth2_validator = OAuth2Validator::new(oauth2_config).await?;

// OAuth2 middleware
async fn oauth2_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = req.headers()
        .get(HeaderName::from_static("authorization"))
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate OAuth2 token
    let claims = state.oauth2_validator
        .validate(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add claims to request extensions
    let mut req = req;
    req.extensions_mut().insert(claims);
    
    // Continue to handler
    Ok(next.run(req).await)
}
```

## API Authorization

### Scopes and Permissions

Configure API scopes:

```yaml
# config/default.yaml
api:
  scopes:
    - name: "users:read"
      description: "Read user information"
    - name: "users:write"
      description: "Create or update users"
    - name: "admin"
      description: "Administrative access"
```

Implement scope-based authorization:

```rust
use navius::api::auth::{ScopeValidator, Claims};

// Scope validation middleware
async fn scope_middleware(
    req: Request,
    next: Next,
    required_scopes: Vec<String>,
) -> Result<Response, StatusCode> {
    // Get claims from request extensions
    let claims = req.extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Check if token has required scopes
    let has_scope = required_scopes.iter().any(|scope| {
        claims.scopes.contains(scope)
    });
    
    if !has_scope {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Continue to handler
    Ok(next.run(req).await)
}

// Apply to routes
let app = Router::new()
    .route("/users", get(get_users_handler))
    .route_layer(middleware::from_fn(|req, next| {
        scope_middleware(req, next, vec!["users:read".to_string()])
    }))
    .route("/users", post(create_user_handler))
    .route_layer(middleware::from_fn(|req, next| {
        scope_middleware(req, next, vec!["users:write".to_string()])
    }));
```

### Fine-grained API Permissions

Implement resource-based permissions:

```rust
use navius::api::auth::{PermissionValidator, ResourcePermission};

// Resource permission middleware
async fn resource_permission_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get claims from request extensions
    let claims = req.extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract resource ID from request
    let resource_id = extract_resource_id(&req)?;
    
    // Determine action from method
    let action = match req.method() {
        &Method::GET => "read",
        &Method::POST => "create",
        &Method::PUT | &Method::PATCH => "update",
        &Method::DELETE => "delete",
        _ => "access",
    };
    
    // Check if token has permission for this resource
    let permission = ResourcePermission {
        resource_type: "user",
        resource_id: Some(resource_id),
        action: action.to_string(),
    };
    
    let has_permission = state.permission_validator
        .validate(claims, &permission)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Continue to handler
    Ok(next.run(req).await)
}
```

## Input Validation and Sanitization

### Request Validation

Validate API requests:

```rust
use navius::api::validation::{Validator, ValidationRules};
use serde::{Deserialize, Serialize};

// Define validation schema
#[derive(Debug, Deserialize, Serialize)]
struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    username: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8, max = 100), strong_password)]
    password: String,
    
    #[validate(phone)]
    phone: Option<String>,
}

// Create validator
let validator = Validator::new();

// Request validation middleware
async fn validate_request<T: DeserializeOwned + ValidatedRequest>(
    Json(payload): Json<T>,
    validator: &Validator,
) -> Result<Json<T>, StatusCode> {
    // Validate request
    validator.validate(&payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok(Json(payload))
}

// Use in handler
async fn create_user_handler(
    State(state): State<AppState>,
    validated: ValidatedJson<CreateUserRequest>,
) -> impl IntoResponse {
    // Handler implementation with validated request
    let user = create_user(validated.0).await?;
    
    (StatusCode::CREATED, Json(user))
}
```

### Content Type Validation

Ensure correct content types:

```rust
// Content type validation middleware
async fn content_type_middleware(
    req: Request,
    next: Next,
    allowed_types: Vec<&'static str>,
) -> Result<Response, StatusCode> {
    // Extract content type header
    let content_type = req.headers()
        .get(HeaderName::from_static("content-type"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    // Check if content type is allowed
    let allowed = allowed_types.iter().any(|&t| content_type.starts_with(t));
    
    if !allowed {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    
    // Continue to handler
    Ok(next.run(req).await)
}

// Apply to routes
let app = Router::new()
    .route("/users", post(create_user_handler))
    .route_layer(middleware::from_fn(|req, next| {
        content_type_middleware(req, next, vec!["application/json"])
    }));
```

### API Schema Validation

Validate against OpenAPI schema:

```rust
use navius::api::validation::{OpenApiValidator, OpenApiConfig};

// Create OpenAPI validator
let openapi_validator = OpenApiValidator::new(OpenApiConfig {
    schema_path: "/path/to/openapi.yaml".to_string(),
    validate_requests: true,
    validate_responses: true,
});

// OpenAPI validation middleware
async fn openapi_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Validate request against schema
    state.openapi_validator
        .validate_request(&req)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Call handler
    let response = next.run(req).await;
    
    // Validate response against schema
    state.openapi_validator
        .validate_response(&response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}
```

## Rate Limiting and Throttling

### Rate Limiting Configuration

Configure rate limiting:

```yaml
# config/default.yaml
api:
  rate_limiting:
    enabled: true
    strategies:
      - type: "ip"
        limit: 100
        window_seconds: 60
      - type: "user"
        limit: 1000
        window_seconds: 3600
      - type: "token"
        limit: 5000
        window_seconds: 3600
```

### Rate Limiting Implementation

Implement rate limiting:

```rust
use navius::api::protection::{RateLimiter, RateLimitStrategy, RateLimitConfig};

// Create rate limiter
let rate_limiter = RateLimiter::new(
    RateLimitConfig::from_config(&config)?,
    redis_client,
);

// Rate limiting middleware
async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client identifier (IP, user ID, or token)
    let client_id = get_client_identifier(&req)?;
    
    // Get rate limit strategy based on client
    let strategy = state.rate_limiter.get_strategy_for_client(&client_id);
    
    // Check rate limit
    let result = state.rate_limiter
        .check(client_id, strategy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !result.allowed {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Add rate limit headers to response
    let response = next.run(req).await;
    let response = add_rate_limit_headers(response, result);
    
    Ok(response)
}

// Add rate limit headers to response
fn add_rate_limit_headers(mut response: Response, result: RateLimitResult) -> Response {
    let headers = response.headers_mut();
    
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from(result.limit.to_string()),
    );
    
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from(result.remaining.to_string()),
    );
    
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from(result.reset.to_string()),
    );
    
    response
}
```

### Throttling for Specific Endpoints

Implement endpoint-specific throttling:

```rust
// Endpoint-specific rate limit middleware
async fn endpoint_rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
    endpoint: &str,
    limit: u64,
    window_seconds: u64,
) -> Result<Response, StatusCode> {
    // Get client identifier
    let client_id = get_client_identifier(&req)?;
    
    // Create endpoint-specific key
    let key = format!("{}:{}", endpoint, client_id);
    
    // Check custom rate limit
    let result = state.rate_limiter
        .check_custom(key, limit, window_seconds)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !result.allowed {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Continue to handler
    let response = next.run(req).await;
    let response = add_rate_limit_headers(response, result);
    
    Ok(response)
}

// Apply to specific endpoint
let app = Router::new()
    .route("/password-reset", post(password_reset_handler))
    .route_layer(middleware::from_fn_with_state(app_state.clone(), |req, next, state| {
        endpoint_rate_limit_middleware(State(state), req, next, "password-reset", 5, 3600)
    }));
```

## API Response Security

### Data Minimization

Implement response filtering:

```rust
use navius::api::response::{ResponseFilter, FilterConfig};

// Create response filter
let filter_config = FilterConfig {
    default_fields: vec!["id", "name", "created_at"],
    sensitive_fields: vec!["email", "phone", "address"],
    field_policies: HashMap::from([
        ("users".to_string(), vec!["id", "username", "created_at"]),
        ("orders".to_string(), vec!["id", "status", "items", "total"]),
    ]),
};

let response_filter = ResponseFilter::new(filter_config);

// Filter responses
async fn filter_response<T: Serialize>(
    data: T,
    resource_type: &str,
    fields: Option<Vec<String>>,
    filter: &ResponseFilter,
) -> Result<Json<Value>, StatusCode> {
    let filtered = filter
        .filter_response(data, resource_type, fields)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(filtered))
}

// Use in handler
async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Get user from database
    let user = get_user(user_id).await?;
    
    // Parse fields parameter
    let fields = params.get("fields").map(|f| {
        f.split(',').map(|s| s.trim().to_string()).collect()
    });
    
    // Filter response
    let filtered = filter_response(user, "users", fields, &state.response_filter).await?;
    
    (StatusCode::OK, filtered)
}
```

### Security Headers

Implement security headers:

```rust
use navius::api::security::ApiSecurityHeadersLayer;

// Add API security headers
let app = Router::new()
    .route("/", get(handler))
    .layer(ApiSecurityHeadersLayer::new());

// Headers set:
// - X-Content-Type-Options: nosniff
// - Cache-Control: no-store
// - Content-Security-Policy: default-src 'self'
// - X-Frame-Options: DENY
// - Strict-Transport-Security: max-age=31536000; includeSubDomains
```

### Safe Error Responses

Implement safe error handling:

```rust
use navius::api::error::{ApiError, ApiErrorResponse};

// Create API error
async fn handle_api_error(error: ApiError) -> impl IntoResponse {
    let status = match error.kind {
        ApiErrorKind::NotFound => StatusCode::NOT_FOUND,
        ApiErrorKind::Validation => StatusCode::BAD_REQUEST,
        ApiErrorKind::Authentication => StatusCode::UNAUTHORIZED,
        ApiErrorKind::Authorization => StatusCode::FORBIDDEN,
        ApiErrorKind::RateLimit => StatusCode::TOO_MANY_REQUESTS,
        ApiErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    // Create safe error response
    let response = ApiErrorResponse {
        code: error.code,
        message: error.public_message,
        details: error.public_details,
        request_id: error.request_id,
    };
    
    // Log detailed error information for debugging
    if error.kind == ApiErrorKind::Internal {
        error!(?error, "Internal API error");
    } else {
        debug!(?error, "API error response");
    }
    
    (status, Json(response))
}

// Use in error handler
async fn api_error_handler(error: BoxError) -> impl IntoResponse {
    if let Some(api_error) = error.downcast_ref::<ApiError>() {
        return handle_api_error(api_error.clone()).await;
    }
    
    // Convert other errors to internal API errors
    let api_error = ApiError::internal(
        "unexpected_error",
        "An unexpected error occurred",
        format!("{}", error),
    );
    
    handle_api_error(api_error).await
}
```

## Cross-Origin Resource Sharing (CORS)

### CORS Configuration

Configure CORS:

```yaml
# config/default.yaml
api:
  cors:
    enabled: true
    allow_origins:
      - "https://app.example.com"
      - "https://admin.example.com"
    allow_methods:
      - "GET"
      - "POST"
      - "PUT"
      - "DELETE"
    allow_headers:
      - "Authorization"
      - "Content-Type"
    expose_headers:
      - "X-Request-ID"
    max_age_seconds: 3600
    allow_credentials: true
```

### CORS Implementation

Implement CORS:

```rust
use navius::api::cors::{CorsLayer, CorsConfig};
use tower_http::cors::{CorsLayer as TowerCorsLayer, Any};

// Create CORS layer
let cors_config = CorsConfig::from_config(&config)?;
let cors_layer = if cors_config.enabled {
    let allowed_origins = cors_config.allow_origins
        .iter()
        .map(|origin| origin.parse().unwrap())
        .collect::<Vec<_>>();
    
    let allowed_methods = cors_config.allow_methods
        .iter()
        .map(|method| method.parse().unwrap())
        .collect::<Vec<_>>();
    
    let allowed_headers = cors_config.allow_headers
        .iter()
        .map(|header| header.parse().unwrap())
        .collect::<Vec<_>>();
    
    let exposed_headers = cors_config.expose_headers
        .iter()
        .map(|header| header.parse().unwrap())
        .collect::<Vec<_>>();
    
    Some(
        TowerCorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .expose_headers(exposed_headers)
            .max_age(Duration::from_secs(cors_config.max_age_seconds))
            .allow_credentials(cors_config.allow_credentials)
    )
} else {
    None
};

// Apply CORS layer if enabled
let app = Router::new()
    .route("/", get(handler));

let app = if let Some(cors) = cors_layer {
    app.layer(cors)
} else {
    app
};
```

## API Monitoring and Logging

### Request Logging

Implement API request logging:

```rust
use navius::api::logging::{ApiLogger, LogConfig};

// Create API logger
let log_config = LogConfig {
    request_headers: vec!["user-agent", "content-type", "accept"],
    response_headers: vec!["content-type", "cache-control"],
    log_body: false,
    log_query_params: true,
    mask_sensitive_headers: vec!["authorization", "x-api-key"],
};

let api_logger = ApiLogger::new(log_config);

// Logger middleware
async fn api_logger_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Generate request ID if not present
    let request_id = get_or_generate_request_id(&req);
    
    // Log request
    let start_time = Instant::now();
    state.api_logger.log_request(&req, request_id).await;
    
    // Process request
    let response = next.run(req).await;
    
    // Calculate duration
    let duration = start_time.elapsed();
    
    // Log response
    state.api_logger.log_response(&response, request_id, duration).await;
    
    Ok(response)
}
```

### Error Rate Monitoring

Implement error rate monitoring:

```rust
use navius::api::monitoring::{ErrorMonitor, AlertConfig};

// Create error monitor
let error_monitor = ErrorMonitor::new(
    AlertConfig {
        error_threshold_percent: 5.0,
        window_seconds: 60,
        min_requests: 10,
    },
    metrics_client,
);

// Error monitoring middleware
async fn error_monitor_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Process request
    let response = next.run(req).await;
    
    // Check if response is an error
    let is_error = response.status().is_client_error() || response.status().is_server_error();
    
    // Record request result
    state.error_monitor
        .record(req.uri().path(), is_error)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}
```

### API Metrics

Implement API metrics:

```rust
use navius::api::metrics::{ApiMetrics, MetricsConfig};

// Create API metrics
let api_metrics = ApiMetrics::new(
    MetricsConfig {
        enabled: true,
        endpoint: "/metrics".to_string(),
        namespace: "navius_api".to_string(),
    },
    prometheus_registry,
);

// Metrics middleware
async fn api_metrics_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract path for grouping similar routes
    let path = normalize_path(req.uri().path());
    let method = req.method().clone();
    
    // Start timer
    let start_time = Instant::now();
    
    // Process request
    let response = next.run(req).await;
    
    // Record metrics
    let status = response.status().as_u16();
    let duration = start_time.elapsed();
    
    state.api_metrics.record_request(
        &path,
        method.as_str(),
        status,
        duration.as_secs_f64(),
    );
    
    Ok(response)
}
```

## Security Testing for APIs

### API Security Testing Tools

```rust
use navius::api::testing::{SecurityScanner, ScanConfig};

// Create security scanner
let security_scanner = SecurityScanner::new(
    ScanConfig {
        target_url: "https://api.example.com".to_string(),
        api_schema_path: "/path/to/openapi.yaml".to_string(),
        auth_token: Some("test-token".to_string()),
        scan_types: vec!["injection", "authentication", "authorization"],
    },
);

// Run security scan
async fn run_security_scan(scanner: &SecurityScanner) -> Result<ScanReport, Error> {
    let report = scanner.scan().await?;
    
    // Output scan results
    for vulnerability in &report.vulnerabilities {
        println!(
            "Vulnerability: {} (Severity: {})",
            vulnerability.name, vulnerability.severity
        );
        println!("Endpoint: {}", vulnerability.endpoint);
        println!("Description: {}", vulnerability.description);
        println!("Remediation: {}", vulnerability.remediation);
        println!();
    }
    
    Ok(report)
}
```

### API Fuzz Testing

Implement fuzz testing:

```rust
use navius::api::testing::{FuzzTester, FuzzConfig};

// Create fuzz tester
let fuzz_tester = FuzzTester::new(
    FuzzConfig {
        target_url: "https://api.example.com".to_string(),
        api_schema_path: "/path/to/openapi.yaml".to_string(),
        auth_token: Some("test-token".to_string()),
        iterations: 1000,
        payloads_path: "/path/to/fuzz-payloads.txt".to_string(),
    },
);

// Run fuzz tests
async fn run_fuzz_tests(tester: &FuzzTester) -> Result<FuzzReport, Error> {
    let report = tester.run().await?;
    
    // Output fuzz test results
    for issue in &report.issues {
        println!("Issue: {}", issue.description);
        println!("Endpoint: {}", issue.endpoint);
        println!("Request: {:?}", issue.request);
        println!("Response: {}", issue.response.status);
        println!();
    }
    
    Ok(report)
}
```

## API Security Best Practices

### API Security Checklist

1. **Authentication and Authorization**
   - Implement secure authentication (API keys, JWT, OAuth)
   - Use proper authorization for all endpoints
   - Implement token validation and revocation

2. **Input Validation and Sanitization**
   - Validate all input parameters
   - Sanitize data to prevent injection attacks
   - Validate content types and schemas

3. **Rate Limiting and Resource Protection**
   - Implement rate limiting for all endpoints
   - Set appropriate timeouts for all operations
   - Limit payload sizes

4. **Response Security**
   - Return minimal data in responses
   - Use appropriate security headers
   - Return safe error messages

5. **Transport Security**
   - Enforce HTTPS for all API communications
   - Configure proper TLS settings
   - Implement CORS properly

6. **Logging and Monitoring**
   - Log all API access and errors
   - Monitor for suspicious activity
   - Set up alerts for security incidents

7. **API Lifecycle Management**
   - Version API endpoints
   - Deprecate and retire APIs safely
   - Document security requirements

### Secure API Design Principles

1. **Design for Least Privilege**
   - Each API endpoint should require minimal permissions
   - Scope access tokens to specific resources

2. **Avoid Exposing Implementation Details**
   - Hide internal identifiers when possible
   - Avoid leaking stack traces or internal error messages

3. **Secure Parameter Handling**
   - Always validate query parameters and request bodies
   - Use parameterized queries for database operations

4. **Always Verify on Server**
   - Never trust client-side validation
   - Revalidate all data server-side regardless of client validation

## Troubleshooting API Security Issues

### Common API Security Issues

1. **Authentication Failures**
   - Invalid or expired tokens
   - Missing credentials
   - Incorrect API key format

2. **Authorization Problems**
   - Missing permissions
   - Incorrect scopes
   - Resource access denied

3. **Rate Limiting Issues**
   - Too many requests
   - Inconsistent rate limit application
   - Rate limit bypass attempts

4. **Input Validation Failures**
   - Malformed input data
   - Injection attack attempts
   - Schema validation errors

### Debugging API Security

```rust
// Enable detailed logging for API security components
tracing_subscriber::fmt()
    .with_env_filter("navius::api::auth=debug,navius::api::validation=debug")
    .init();

// Create test tokens for debugging
async fn create_debug_token(
    claims: HashMap<String, Value>,
    jwt_service: &JwtService,
) -> Result<String, Error> {
    let token = jwt_service.create_token(claims).await?;
    Ok(token)
}
```

## Related Resources

- [Authentication Implementation Guide](./authentication-implementation.md)
- [Authorization Guide](./authorization-guide.md)
- [Data Protection Guide](./data-protection.md)
- [Security Best Practices](./security-best-practices.md)
- [OWASP API Security Top 10](https://owasp.org/API-Security/editions/2023/en/0x00-header/) 
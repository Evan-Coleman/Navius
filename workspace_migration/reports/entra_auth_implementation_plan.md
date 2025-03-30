# Microsoft Entra Authentication Provider Implementation Plan

**Date:** March 29, 2025  
**Status:** Planning  
**Priority:** High  
**Target Completion:** April 12, 2025

## Overview

This document outlines the implementation plan for the `navius-auth-entra` crate, which will provide Microsoft Entra ID (formerly Azure AD) authentication support for the Navius framework. This implementation will follow the provider pattern established in the `navius-auth` crate evaluation and build upon the existing Entra provider code.

## Objectives

1. Create a dedicated `navius-auth-entra` crate following the workspace structure
2. Implement the full Entra authentication provider based on the existing code
3. Ensure proper integration with the authentication framework
4. Provide a clean, consistent API following the recommendations from design evaluations
5. Add comprehensive testing for the provider
6. Create detailed documentation for usage

## Implementation Approach

### Crate Structure

```
navius-auth-entra/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Main exports and documentation
│   ├── config.rs            # Configuration structures
│   ├── error.rs             # Error types specific to Entra auth
│   ├── provider.rs          # EntraAuthProvider implementation
│   ├── token.rs             # Token validation and handling
│   ├── jwks.rs              # JWKS cache and management
│   ├── middleware.rs        # HTTP middleware for Entra auth
│   └── types.rs             # Common types and structures
└── tests/
    ├── common/
    │   └── mod.rs           # Test utilities
    ├── integration_test.rs  # Integration tests
    └── provider_test.rs     # Provider implementation tests
```

### Key Components

#### 1. EntraAuthProvider

The core provider implementation that implements the `AuthProvider` trait from `navius-auth`:

```rust
pub struct EntraAuthProvider {
    config: EntraConfig,
    http_client: Client,
    jwks_cache: JwksCache,
    circuit_breaker: CircuitBreaker,
}

impl EntraAuthProvider {
    pub fn new(config: EntraConfig) -> Self {
        // Implementation
    }
    
    pub fn builder() -> EntraAuthProviderBuilder {
        // Return a builder for fluent configuration
    }
    
    async fn validate_token_internal(&self, token: &str) -> Result<Claims, EntraAuthError> {
        // Implementation
    }
    
    async fn refresh_jwks(&self) -> Result<(), EntraAuthError> {
        // Implementation
    }
}

#[async_trait]
impl AuthProvider for EntraAuthProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Entra
    }
    
    fn name(&self) -> &str {
        "entra"
    }
    
    async fn authenticate(&self, credentials: &Credentials) -> Result<Identity, AuthError> {
        // Implementation
    }
    
    async fn validate_token(&self, token: &str) -> Result<Subject, AuthError> {
        // Implementation
    }
    
    async fn create_token(&self, subject: &Subject) -> Result<String, AuthError> {
        // Implementation
    }
    
    async fn revoke_token(&self, token: &str) -> Result<(), AuthError> {
        // Implementation
    }
    
    async fn refresh_token(&self, token: &str) -> Result<String, AuthError> {
        // Implementation
    }
}
```

#### 2. EntraAuthProviderBuilder

A builder pattern implementation for configuring the provider:

```rust
pub struct EntraAuthProviderBuilder {
    config: EntraConfigBuilder,
}

impl EntraAuthProviderBuilder {
    pub fn new() -> Self {
        Self {
            config: EntraConfigBuilder::new(),
        }
    }
    
    pub fn tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.config = self.config.tenant_id(tenant_id);
        self
    }
    
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.config = self.config.client_id(client_id);
        self
    }
    
    pub fn client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.config = self.config.client_secret(client_secret);
        self
    }
    
    // Additional builder methods
    
    pub fn build(self) -> Result<EntraAuthProvider, EntraAuthError> {
        let config = self.config.build()?;
        Ok(EntraAuthProvider::new(config))
    }
}
```

#### 3. Configuration

Type-safe configuration structures:

```rust
#[derive(Debug, Clone)]
pub struct EntraConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub jwks_uri: String,
    pub issuer_url: String,
    pub audience: String,
    pub validation_leeway: Duration,
    pub role_mappings: HashMap<String, Vec<String>>,
    pub debug_validation: bool,
}

pub struct EntraConfigBuilder {
    tenant_id: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    jwks_uri: Option<String>,
    issuer_url: Option<String>,
    audience: Option<String>,
    validation_leeway: Duration,
    role_mappings: HashMap<String, Vec<String>>,
    debug_validation: bool,
}

impl EntraConfigBuilder {
    // Builder methods for configuration
}
```

#### 4. JWKS Cache

A cache for JWKS (JSON Web Key Set) keys with automatic refresh:

```rust
pub struct JwksCache {
    keys: Arc<RwLock<Option<JwksCacheEntry>>>,
    refresh_limiter: RateLimiter,
}

impl JwksCache {
    pub fn new(refresh_rate: Duration) -> Self {
        // Implementation
    }
    
    pub async fn get_keys(&self) -> Result<Vec<Jwk>, EntraAuthError> {
        // Implementation
    }
    
    pub async fn refresh(&self, uri: &str, client: &Client) -> Result<(), EntraAuthError> {
        // Implementation
    }
}
```

#### 5. Circuit Breaker

A circuit breaker for preventing excessive calls to Entra endpoints:

```rust
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: u32,
    retry_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, retry_timeout: Duration) -> Self {
        // Implementation
    }
    
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<EntraAuthError>,
    {
        // Implementation
    }
}
```

### Integration with navius-auth

The `navius-auth-entra` crate will integrate with `navius-auth` as follows:

1. Implement the `AuthProvider` trait from `navius-auth` for `EntraAuthProvider`
2. Register the provider with the `ProviderRegistry` from `navius-auth`
3. Provide middleware extensions for HTTP authentication

```rust
// Example usage in application code
let provider = EntraAuthProvider::builder()
    .tenant_id(config.tenant_id)
    .client_id(config.client_id)
    .client_secret(config.client_secret)
    .build()?;

let auth_registry = AuthRegistry::new()
    .register("entra", provider)
    .build();

let app = Router::new()
    .route("/protected", get(handler))
    .layer(auth_registry.middleware("entra").with_roles(vec!["admin"]).layer());
```

## Testing Strategy

### Unit Tests

1. **Provider Tests**: Test all provider methods with controlled inputs
2. **Token Validation Tests**: Test token validation logic with sample tokens
3. **JWKS Cache Tests**: Test JWKS cache functionality
4. **Circuit Breaker Tests**: Test circuit breaker behavior under various conditions
5. **Builder Tests**: Test configuration builder functionality

### Integration Tests

1. **Mock Server Tests**: Tests using a mock Entra ID server
2. **End-to-End Flow Tests**: Full authentication flow tests
3. **Error Handling Tests**: Tests for various error conditions
4. **Performance Tests**: Tests for performance under load

### Test Utilities

Create comprehensive test utilities:

```rust
pub struct EntraMockServer {
    // Implementation details
}

impl EntraMockServer {
    pub fn new() -> Self {
        // Setup mock server
    }
    
    pub fn with_jwks_endpoint(mut self, keys: Vec<Jwk>) -> Self {
        // Configure JWKS endpoint
        self
    }
    
    pub fn with_token_endpoint(mut self, handler: impl Fn(TokenRequest) -> TokenResponse + 'static) -> Self {
        // Configure token endpoint
        self
    }
    
    pub fn url(&self) -> String {
        // Return server URL
        self.base_url.clone()
    }
    
    pub fn start(self) -> EntraMockServerHandle {
        // Start server and return handle
        EntraMockServerHandle { /* ... */ }
    }
}
```

## Metrics and Monitoring

Implement comprehensive metrics:

1. **Authentication Attempts**: Count of authentication attempts
2. **Successful Authentications**: Count of successful authentications
3. **Failed Authentications**: Count of failed authentications
4. **Token Validations**: Count of token validations
5. **JWKS Refreshes**: Count of JWKS refreshes
6. **Response Times**: Histogram of response times for various operations
7. **Circuit Breaker State**: Gauge of circuit breaker state

## Documentation

Create detailed documentation:

1. **API Documentation**: Comprehensive API documentation with examples
2. **Configuration Guide**: Detailed configuration instructions
3. **Integration Examples**: Examples of integrating with applications
4. **Troubleshooting Guide**: Common issues and solutions

## Implementation Schedule

1. **Week 1 (April 1-5, 2025)**:
   - Set up crate structure
   - Implement core provider functionality
   - Implement token validation

2. **Week 2 (April 6-12, 2025)**:
   - Implement JWKS cache
   - Implement circuit breaker
   - Add metrics and monitoring
   - Create tests
   - Complete documentation

## Conclusion

The `navius-auth-entra` crate will provide a robust, production-ready implementation of Microsoft Entra ID authentication for the Navius framework. By following the established provider pattern and leveraging the existing code, we will ensure consistency with the rest of the ecosystem while providing the specific functionality needed for Entra ID authentication. 
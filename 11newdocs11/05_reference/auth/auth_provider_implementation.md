## Adding a New Provider

1. Create a new module in `src/core/auth/providers/`
2. Implement the `OAuthProvider` trait:
```rust
#[async_trait]
impl OAuthProvider for NewProvider {
    // Required implementations
    async fn validate_token(&self, token: &str) -> Result<StandardClaims, AuthError>;
    async fn refresh_jwks(&self) -> Result<(), AuthError>;
    async fn health_check(&self) -> HealthStatus;
}
```

3. Add provider configuration:
```yaml
auth:
  providers:
    new_provider:
      enabled: true
      client_id: "..."
      jwks_uri: "..."
      issuer_url: "..."
      audience: "..."
```

4. Register the provider in `ProviderRegistry::initialize()`

## Health Check Metrics

The `/actuator/health` endpoint now includes authentication provider status:
```json
{
  "status": "UP",
  "components": {
    "auth_providers": {
      "entra": {
        "ready": true,
        "jwks_valid": true,
        "last_refresh": "2024-03-20T12:34:56Z",
        "error": null
      }
    }
  }
}
```
```
# Authentication Error Reference

## Error Types
- **MissingToken**: No authorization header present
- **InvalidTokenFormat**: Malformed authorization header
- **ValidationFailed**: JWT validation error (expired, invalid signature, etc)
- **AccessDenied**: Insufficient permissions/roles
- **RateLimited**: Too many JWKS refresh requests
- **CircuitOpen**: Provider temporarily disabled due to failures
- **ProviderError**: Provider-specific errors

## Error Handling Guidelines
- Convert provider-specific errors to standard AuthError variants
- Use appropriate HTTP status codes:
  - 401 for authentication failures
  - 403 for authorization failures
  - 429 for rate limiting
  - 503 for circuit breaker open 
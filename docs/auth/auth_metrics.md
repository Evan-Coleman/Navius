# Authentication Metrics

## Token Validation
- `auth_tokens_validated_total`: Counter of validated tokens (tags: provider, status)
- `auth_token_validation_time_seconds`: Histogram of validation times

## JWKS Management
- `auth_jwks_refreshes_total`: Counter of JWKS refresh operations
- `auth_jwks_refresh_time_seconds`: Histogram of refresh durations
- `auth_jwks_valid`: Gauge indicating valid JWKS (0/1)
- `auth_provider_ready`: Gauge indicating provider readiness (0/1)

## Rate Limiting
- `auth_rate_limit_remaining`: Gauge of remaining rate limit capacity 
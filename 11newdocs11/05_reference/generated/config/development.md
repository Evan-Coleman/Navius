# DEVELOPMENT Configuration

```yaml
# Server configuration
server:
  host: "0.0.0.0"
  port: 8080
  
# Logging configuration
logging:
  level: "info"
  format: "json"
```

## Metrics Configuration

```yaml
metrics:
  enabled: true
  endpoint: "/metrics"
  port: 8081
```

## Authentication Configuration

```yaml
auth:
  enabled: true
  jwt_secret: "<your-secret-here>"
  token_expiry: 3600
```

## Caching Configuration

```yaml
caching:
  enabled: true
  redis_url: "redis://localhost:6379"
  default_ttl: 300
```

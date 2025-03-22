# Navius Framework

This directory contains the core functionality of Navius that powers the framework's enterprise-grade features. The core modules provide essential capabilities that should not be modified by users.

## Core Capabilities

- **Route Management**: Essential routes for health checks, metrics, and API documentation
- **Security**: Authentication, authorization, and security middleware
- **Observability**: Metrics collection, tracing, and health monitoring
- **Reliability**: Circuit breakers, rate limiting, timeouts, and retries
- **Configuration**: Environment-aware configuration management
- **Caching**: High-performance caching infrastructure
- **Error Handling**: Structured error management and reporting

## Directory Structure

- `router/`: Core routing infrastructure
- `handlers/`: System-level request handlers
- `auth/`: Authentication and authorization components
- `cache/`: Caching infrastructure
- `config/`: Configuration management
- `database/`: Database connection and transaction management
- `error/`: Error handling framework
- `metrics/`: Metrics collection and reporting
- `reliability/`: Reliability patterns (circuit breakers, retries)
- `utils/`: Core utility functions

## Extending Navius

Navius follows the extension-over-modification pattern. Instead of changing core files:

1. Use `src/app/router.rs` to add custom routes and handlers
2. Create domain-specific services in `src/services/`
3. Implement data access in `src/repository/`
4. Create custom handlers in `src/api/`

## Core Routes

Navius provides these essential routes:

- Public Routes:
  - `/health` - Basic health check
  - `/docs` - API documentation (Swagger UI)

- Actuator Routes (protected when security is enabled):
  - `/actuator/health` - Detailed health check with component status
  - `/actuator/info` - System information
  - `/actuator/metrics` - Prometheus-compatible metrics endpoint

## Performance

The core framework is optimized for maximum performance:

- Asynchronous request handling with Tokio runtime
- Connection pooling for database and external services
- Minimal memory footprint with zero-cost abstractions
- Efficient JSON serialization/deserialization

## Security

Navius implements security best practices by default:

- Automatic TLS configuration
- CORS protection
- Content security headers
- Protection against common web vulnerabilities
- Secure authentication patterns 
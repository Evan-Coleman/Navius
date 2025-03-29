---
title: Navius Environment Variables Reference
description: Comprehensive reference of environment variables for configuring Navius applications
category: reference
tags:
  - configuration
  - environment-variables
  - settings
related:
  - ../architecture/principles.md
  - ../../guides/deployment/production-deployment.md
  - ../../guides/deployment/cloud-deployment.md
last_updated: March 27, 2025
version: 1.0
---

# Navius Environment Variables Reference

## Overview
This reference document provides a comprehensive list of all environment variables supported by the Navius framework. Environment variables are used to configure various aspects of the application without changing code or configuration files, making them ideal for deployment across different environments.

## Core Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `RUN_ENV` | Application environment | `development` | `production` |
| `PORT` | HTTP server port | `3000` | `8080` |
| `HOST` | HTTP server host | `127.0.0.1` | `0.0.0.0` |
| `LOG_LEVEL` | Logging verbosity level | `info` | `debug` |
| `LOG_FORMAT` | Log output format | `text` | `json` |
| `CONFIG_PATH` | Path to configuration directory | `./config` | `/etc/navius/config` |
| `RUST_BACKTRACE` | Enable backtrace on errors | `0` | `1` |
| `RUST_LOG` | Detailed logging configuration | - | `navius=debug,warn` |

## Database Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATABASE_URL` | Database connection string | - | `postgres://user:pass@localhost/dbname` |
| `DATABASE_POOL_SIZE` | Max database connections | `5` | `20` |
| `DATABASE_TIMEOUT_SECONDS` | Query timeout in seconds | `30` | `10` |
| `DATABASE_CONNECT_TIMEOUT_SECONDS` | Connection timeout in seconds | `5` | `3` |
| `DATABASE_IDLE_TIMEOUT_SECONDS` | Idle connection timeout | `300` | `600` |
| `DATABASE_MAX_LIFETIME_SECONDS` | Max connection lifetime | `1800` | `3600` |
| `DATABASE_SSL_MODE` | SSL connection mode | `prefer` | `require` |
| `RUN_MIGRATIONS` | Auto-run migrations on startup | `false` | `true` |

## Cache Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `REDIS_URL` | Redis connection URL | - | `redis://localhost:6379` |
| `REDIS_POOL_SIZE` | Max Redis connections | `5` | `10` |
| `REDIS_TIMEOUT_SECONDS` | Redis command timeout | `5` | `3` |
| `CACHE_TTL_SECONDS` | Default cache TTL | `3600` | `300` |
| `CACHE_PREFIX` | Cache key prefix | `navius:` | `myapp:prod:` |
| `CACHE_ENABLED` | Enable caching | `true` | `false` |

## Security Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `JWT_SECRET` | Secret for JWT tokens | - | `your-jwt-secret-key` |
| `JWT_EXPIRATION_SECONDS` | JWT token expiration | `86400` | `3600` |
| `CORS_ALLOWED_ORIGINS` | Allowed CORS origins | `*` | `https://example.com,https://app.example.com` |
| `CORS_ALLOWED_METHODS` | Allowed CORS methods | `GET,POST,PUT,DELETE` | `GET,POST` |
| `CORS_ALLOWED_HEADERS` | Allowed CORS headers | `Content-Type,Authorization` | `X-API-Key,Authorization` |
| `CORS_MAX_AGE_SECONDS` | CORS preflight cache time | `86400` | `3600` |
| `API_KEY` | Global API key for auth | - | `your-api-key` |
| `TLS_CERT_PATH` | Path to TLS certificate | - | `/etc/certs/server.crt` |
| `TLS_KEY_PATH` | Path to TLS private key | - | `/etc/certs/server.key` |
| `ENABLE_TLS` | Enable TLS encryption | `false` | `true` |

## HTTP Server Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `REQUEST_TIMEOUT_SECONDS` | HTTP request timeout | `30` | `60` |
| `REQUEST_BODY_LIMIT` | Max request body size | `1MB` | `10MB` |
| `ENABLE_COMPRESSION` | Enable response compression | `true` | `false` |
| `KEEP_ALIVE_SECONDS` | Keep-alive connection timeout | `75` | `120` |
| `MAX_CONNECTIONS` | Max concurrent connections | `1024` | `10000` |
| `WORKERS` | Number of worker threads | (cores * 2) | `8` |
| `ENABLE_HEALTH_CHECK` | Enable /health endpoint | `true` | `false` |
| `GRACEFUL_SHUTDOWN_SECONDS` | Graceful shutdown period | `30` | `10` |

## API Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `API_VERSION` | Default API version | `v1` | `v2` |
| `ENABLE_DOCS` | Enable API documentation | `true` | `false` |
| `DOCS_URL_PATH` | Path to API docs | `/docs` | `/api/docs` |
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `false` | `true` |
| `RATE_LIMIT_REQUESTS` | Max requests per window | `100` | `1000` |
| `RATE_LIMIT_WINDOW_SECONDS` | Rate limit time window | `60` | `3600` |
| `API_BASE_PATH` | Base path for all APIs | `/api` | `/api/v1` |

## Monitoring and Telemetry

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `ENABLE_METRICS` | Enable Prometheus metrics | `true` | `false` |
| `METRICS_PATH` | Metrics endpoint path | `/metrics` | `/actuator/metrics` |
| `TRACING_ENABLED` | Enable OpenTelemetry tracing | `false` | `true` |
| `JAEGER_ENDPOINT` | Jaeger collector endpoint | - | `http://jaeger:14268/api/traces` |
| `OTLP_ENDPOINT` | OTLP collector endpoint | - | `http://collector:4317` |
| `SERVICE_NAME` | Service name for telemetry | `navius` | `user-service` |
| `LOG_REQUEST_HEADERS` | Log HTTP request headers | `false` | `true` |
| `HEALTH_CHECK_PATH` | Health check endpoint path | `/health` | `/actuator/health` |

## Integration Settings

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `EMAIL_SMTP_HOST` | SMTP server host | - | `smtp.example.com` |
| `EMAIL_SMTP_PORT` | SMTP server port | `25` | `587` |
| `EMAIL_SMTP_USERNAME` | SMTP authentication user | - | `user@example.com` |
| `EMAIL_SMTP_PASSWORD` | SMTP authentication password | - | `password` |
| `EMAIL_DEFAULT_FROM` | Default sender address | - | `noreply@example.com` |
| `AWS_ACCESS_KEY_ID` | AWS access key | - | `AKIAIOSFODNN7EXAMPLE` |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key | - | `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY` |
| `AWS_REGION` | AWS region | `us-east-1` | `eu-west-1` |
| `S3_BUCKET` | S3 bucket name | - | `my-app-uploads` |
| `S3_URL_EXPIRATION_SECONDS` | S3 presigned URL expiration | `3600` | `300` |

## Resource Limits

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `MEMORY_LIMIT` | Memory limit in MB | - | `512` |
| `CPU_LIMIT` | CPU limit (percentage) | - | `80` |
| `TOKIO_WORKER_THREADS` | Tokio runtime worker threads | (cores) | `8` |
| `BLOCKING_THREADS` | Tokio blocking thread pool size | (cores * 4) | `32` |
| `MAX_TASK_BACKLOG` | Max queued tasks | `10000` | `5000` |

## Feature Flags

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `FEATURE_ADVANCED_SEARCH` | Enable advanced search | `false` | `true` |
| `FEATURE_FILE_UPLOADS` | Enable file uploads | `true` | `false` |
| `FEATURE_WEBSOCKETS` | Enable WebSocket support | `false` | `true` |
| `FEATURE_BATCH_PROCESSING` | Enable batch processing | `false` | `true` |
| `FEATURE_NOTIFICATIONS` | Enable notifications | `true` | `false` |

## Using Environment Variables

Environment variables can be set in various ways:

### 1. In development (`.env` file):

```
# .env
DATABASE_URL=postgres://localhost/navius_dev
LOG_LEVEL=debug
FEATURE_ADVANCED_SEARCH=true
```

### 2. In shell:

```bash
export DATABASE_URL=postgres://localhost/navius_dev
export LOG_LEVEL=debug
./run_dev.sh
```

### 3. In Docker:

```bash
docker run -e DATABASE_URL=postgres://db/navius -e LOG_LEVEL=info navius
```

### 4. In Kubernetes:

```yaml
env:
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: db-credentials
        key: url
  - name: LOG_LEVEL
    value: "info"
```

## Precedence

Environment variables are loaded in this order (later sources override earlier ones):

1. Default values
2. Configuration files (`config/{environment}.toml`)
3. `.env` file
4. Environment variables
5. Command line arguments

## Related Documents

- [Architectural Principles](../architecture/principles.md) - Core architectural principles
- [Production Deployment Guide](../../guides/deployment/production-deployment.md) - Deploying to production
- [Cloud Deployment Guide](../../guides/deployment/cloud-deployment.md) - Cloud-specific deployment 
---
title: "Docker Deployment Guide for Navius"
description: "Comprehensive guide for containerizing and deploying Navius applications using Docker with best practices for configuration, optimization, and security"
category: "guides"
tags:
  - docker
  - deployment
  - containers
  - devops
  - containerization
related:
  - production-deployment.md
  - kubernetes-deployment.md
  - ../../05_reference/configuration/environment-variables.md
  - ../operations/security.md
last_updated: "April 1, 2025"
version: "1.0"
---

# Docker Deployment Guide for Navius

## Overview

This guide provides comprehensive instructions for containerizing and deploying Navius applications using Docker. Navius is particularly well-suited for containerization due to its small footprint, minimal dependencies, and fast startup time.

## Prerequisites

Before containerizing your Navius application, ensure you have:

- Docker installed on your development machine (version 20.10.0 or later)
- A Navius application codebase
- Access to a Docker registry (Docker Hub, GitLab Container Registry, etc.)
- Basic understanding of Docker concepts

## Dockerfile

### Basic Dockerfile

Create a `Dockerfile` in the root of your project with the following content:

```dockerfile
# Build stage
FROM rust:1.72-slim as builder

WORKDIR /usr/src/navius
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/navius/target/release/navius /app/
COPY --from=builder /usr/src/navius/config /app/config

ENV CONFIG_PATH=/app/config

EXPOSE 8080

CMD ["./navius"]
```

### Multi-Stage Build Explanation

This Dockerfile uses a multi-stage build approach:

1. **Builder Stage**:
   - Uses the Rust image to compile the application
   - Installs necessary build dependencies
   - Compiles the application with optimizations

2. **Runtime Stage**:
   - Uses a minimal Debian image for runtime
   - Copies only the compiled binary and configuration files
   - Installs only the runtime dependencies
   - Results in a much smaller final image

## Building the Docker Image

### Basic Build Command

Build your Docker image with:

```bash
docker build -t navius:latest .
```

### Optimized Build

For a production-ready build with additional metadata:

```bash
docker build \
  --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
  --build-arg VERSION=$(git describe --tags --always) \
  --build-arg COMMIT_HASH=$(git rev-parse HEAD) \
  -t navius:latest \
  -t navius:$(git describe --tags --always) \
  .
```

### Cross-Platform Building

To build for multiple platforms using Docker BuildX:

```bash
docker buildx create --name multiplatform --use
docker buildx build --platform linux/amd64,linux/arm64 -t yourregistry/navius:latest --push .
```

## Configuration

### Environment Variables

Navius applications typically use environment variables for configuration. When running in Docker, set these variables using the `-e` flag:

```bash
docker run -e DATABASE_URL=postgres://user:pass@host/db -e RUST_LOG=info navius:latest
```

### Configuration Files

For more complex configuration, mount a configuration directory:

```bash
docker run -v /host/path/to/config:/app/config navius:latest
```

### Docker Compose Setup

For a complete setup with dependencies, create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  navius-api:
    image: navius:latest
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - CONFIG_PATH=/app/config
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/navius
      - REDIS_URL=redis://redis:6379
    volumes:
      - ./config:/app/config
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/actuator/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_USER=postgres
      - POSTGRES_DB=navius
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

## Image Optimization

### Builder Optimization

For faster builds, add a `.dockerignore` file:

```
target/
.git/
.github/
.vscode/
.idea/
tests/
*.md
*.log
```

### Size Optimization

For the smallest possible image, consider using Alpine Linux:

```dockerfile
# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates libssl1.1

WORKDIR /app

COPY --from=builder /usr/src/navius/target/release/navius /app/
COPY --from=builder /usr/src/navius/config /app/config

ENV CONFIG_PATH=/app/config

EXPOSE 8080

CMD ["./navius"]
```

### Security Optimization

For enhanced security, run as a non-root user:

```dockerfile
# Add a navius user and group
RUN addgroup -S navius && adduser -S navius -G navius

# Change ownership
RUN chown -R navius:navius /app

# Switch to navius user
USER navius

EXPOSE 8080

CMD ["./navius"]
```

## Running in Production

### Basic Run Command

Run your containerized Navius application:

```bash
docker run -d -p 8080:8080 --name navius-api navius:latest
```

### Resource Constraints

Set resource limits for production deployments:

```bash
docker run -d -p 8080:8080 \
  --memory=512m \
  --cpus=0.5 \
  --restart=unless-stopped \
  --name navius-api \
  navius:latest
```

### Logging Configuration

Configure logging for production:

```bash
docker run -d -p 8080:8080 \
  -e RUST_LOG=info \
  --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  --name navius-api \
  navius:latest
```

### Health Checks

Use Docker health checks to monitor application health:

```bash
docker run -d -p 8080:8080 \
  --health-cmd "curl -f http://localhost:8080/actuator/health || exit 1" \
  --health-interval=30s \
  --health-timeout=10s \
  --health-retries=3 \
  --health-start-period=10s \
  --name navius-api \
  navius:latest
```

## Docker Registry Integration

### Pushing to a Registry

Push your image to a Docker registry:

```bash
# Tag the image for your registry
docker tag navius:latest registry.example.com/navius:latest

# Push to registry
docker push registry.example.com/navius:latest
```

### Using Private Registries

For private registries, first log in:

```bash
docker login registry.example.com -u username -p password
```

## CI/CD Integration

### GitHub Actions Example

Here's a GitHub Actions workflow for building and publishing your Docker image:

```yaml
name: Build and Publish Docker Image

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Login to DockerHub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: yourusername/navius
          tags: |
            type=semver,pattern={{version}}
            type=ref,event=branch
      
      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Monitoring

### Prometheus Integration

Navius provides Prometheus metrics. Enable with:

```bash
docker run -d -p 8080:8080 -p 9090:9090 \
  -e ENABLE_METRICS=true \
  -e METRICS_PORT=9090 \
  navius:latest
```

### Container Monitoring

To monitor the container itself, consider:

- Docker stats: `docker stats navius-api`
- cAdvisor: A container monitoring tool
- Prometheus Node Exporter with Docker metrics enabled

## Best Practices

1. **Use Multi-Stage Builds** to keep images small
2. **Run as a Non-Root User** for security
3. **Implement Health Checks** for reliability
4. **Pin Dependency Versions** (e.g., `FROM rust:1.72-slim` instead of `FROM rust:latest`)
5. **Keep Images Small** by removing build tools and unused files
6. **Use Docker Compose** for local development with dependencies
7. **Secure Sensitive Data** using Docker secrets or environment variables
8. **Tag Images Properly** for version control (`latest` plus version tags)
9. **Scan Images for Vulnerabilities** using tools like Trivy or Clair
10. **Set Resource Limits** to prevent container resource exhaustion

## Troubleshooting

### Common Issues

1. **Image Too Large**:
   - Use multi-stage builds
   - Minimize layers
   - Use smaller base images like Alpine

2. **Slow Build Times**:
   - Use Docker BuildKit (`DOCKER_BUILDKIT=1 docker build ...`)
   - Optimize `.dockerignore`
   - Use build caching effectively

3. **Container Won't Start**:
   - Check logs: `docker logs navius-api`
   - Verify environment variables
   - Ensure proper permissions on mounted volumes

4. **Permission Issues**:
   - Ensure correct ownership of files
   - Check volume mount permissions
   - Verify the user running the container has necessary permissions

### Debugging Commands

```bash
# Check container logs
docker logs navius-api

# Inspect container details
docker inspect navius-api

# Execute a command inside the container
docker exec -it navius-api /bin/sh

# Check resource usage
docker stats navius-api
```

## Related Resources

- [Production Deployment Guide](production-deployment.md) - General production deployment guidelines
- [Kubernetes Deployment Guide](kubernetes-deployment.md) - Deploying with Kubernetes
- [Environment Variables Reference](../../05_reference/configuration/environment-variables.md) - Configuration options
- [Security Guide](../operations/security.md) - Security considerations

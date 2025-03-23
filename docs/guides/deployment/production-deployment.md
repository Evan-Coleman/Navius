---
title: Navius Production Deployment Guide
description: Comprehensive guide for deploying Navius applications to production environments
category: guides
tags:
  - deployment
  - production
  - docker
  - kubernetes
  - aws
related:
  - cloud-deployment.md
  - ../../reference/configuration/environment-variables.md
  - ../development/testing.md
last_updated: March 23, 2025
version: 1.0
---

# Navius Production Deployment Guide

## Overview
This guide provides comprehensive instructions for deploying Navius applications to production environments, focusing on security, scalability, and reliability. It covers various deployment options, configuration best practices, and performance tuning.

## Prerequisites
Before deploying to production, ensure you have:

- A built and tested Navius application
- Access to your target infrastructure (cloud account, server credentials, etc.)
- CI/CD pipeline configured for automated deployments (recommended)
- Database and cache services ready for production use

## Deployment Options

### Docker Containers

Navius excels in containerized environments, offering minimal resource usage and fast startup times.

```bash
# Build the Docker image
docker build -t navius:latest .

# Run the container
docker run -d \
  -p 8080:8080 \
  -e CONFIG_PATH=/etc/navius/config \
  -e RUST_LOG=info \
  -v /host/path/to/config:/etc/navius/config \
  --name navius-api \
  navius:latest
```

### Kubernetes

Navius applications are ideal for Kubernetes due to their small footprint and rapid startup time.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: navius-api
  labels:
    app: navius-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: navius-api
  template:
    metadata:
      labels:
        app: navius-api
    spec:
      containers:
      - name: api
        image: your-registry/navius:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: CONFIG_PATH
          value: "/etc/navius/config"
        readinessProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          limits:
            cpu: "0.5"
            memory: "512Mi"
          requests:
            cpu: "0.1"
            memory: "128Mi"
```

Service definition:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: navius-api
spec:
  selector:
    app: navius-api
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Bare Metal Deployment

For maximum performance, Navius can be deployed directly to bare metal servers:

```bash
# SSH to your server
ssh user@your-server

# Create directories
sudo mkdir -p /opt/navius/config

# Copy binary and configuration
scp target/release/navius user@server:/opt/navius/
```

Systemd service configuration:

```ini
# /etc/systemd/system/navius.service
[Unit]
Description=Navius API Server
After=network.target

[Service]
User=navius
WorkingDirectory=/opt/navius
ExecStart=/opt/navius/navius
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Production Configuration

### Environment Variables

For production deployments, configure these essential environment variables:

```
# Core settings
RUN_ENV=production
RUST_LOG=info
PORT=3000

# Database settings
DATABASE_URL=postgres://user:password@host:port/db
DATABASE_MAX_CONNECTIONS=20
DATABASE_CONNECT_TIMEOUT_SECONDS=5

# Cache settings
REDIS_URL=redis://user:password@host:port
CACHE_TTL_SECONDS=3600

# Security settings
JWT_SECRET=your-secure-jwt-secret
CORS_ALLOWED_ORIGINS=https://yourdomain.com
```

### Recommended Infrastructure

For production deployments, we recommend:

- **Database**: AWS RDS PostgreSQL or Aurora
- **Cache**: AWS ElastiCache Redis
- **Storage**: AWS S3
- **CDN**: AWS CloudFront
- **Load Balancer**: AWS Application Load Balancer with TLS termination

## Performance Tuning

Navius is designed for high performance, but these optimizations can help in production:

### Thread Pool Sizing

Configure thread pools according to your CPU resources:

```
TOKIO_WORKER_THREADS=number_of_cores * 2
```

### Connection Pool Tuning

Optimize database connection pools:

```
DATABASE_MAX_CONNECTIONS=25
DATABASE_MIN_IDLE=5
DATABASE_IDLE_TIMEOUT_SECONDS=300
```

### Memory Limits

Configure memory limits:

```
RUST_MAX_MEMORY=512m
```

## Monitoring & Observability

### Prometheus Metrics

Metrics are available at the `/metrics` endpoint. Configure Prometheus to scrape this endpoint.

### Health Checks

Health checks are available at:
- `/health` - Basic health check
- `/actuator/health` - Detailed component health

### Logging

Navius uses structured logging with tracing. Configure log aggregation with:

- AWS CloudWatch Logs
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Datadog
- New Relic

Example log configuration:

```
RUST_LOG=info,navius=debug
LOG_FORMAT=json
```

## Scaling Strategies

### Vertical Scaling

Navius is extremely efficient with resources. For many applications, a modest instance size is sufficient:

- AWS: t3.small or t3.medium
- GCP: e2-standard-2
- Azure: Standard_B2s

### Horizontal Scaling

For high-traffic applications, horizontal scaling is recommended:

1. Deploy multiple instances behind a load balancer
2. Configure sticky sessions if using server-side sessions
3. Ensure all state is stored in shared resources (database, Redis)

## Security Best Practices

### TLS Configuration

Always use TLS in production. Configure your load balancer or reverse proxy with:

- TLS 1.2/1.3 only
- Strong cipher suites
- HTTP/2 support
- HSTS headers

### Firewall Rules

Restrict access to your instances:

- Allow only necessary ports
- Implement network segmentation
- Use security groups (AWS) or equivalent

### Regular Updates

Keep your Navius application updated:

```bash
cargo update
cargo build --release
```

## CI/CD Pipeline Integration

### GitHub Actions

```yaml
name: Deploy to Production

on:
  push:
    branches: [ main ]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          
      - name: Build
        run: cargo build --release
        
      - name: Run tests
        run: cargo test
        
      - name: Build Docker image
        run: docker build -t yourdockerhub/navius:${{ github.sha }} .
        
      - name: Push Docker image
        run: |
          docker login -u ${{ secrets.DOCKER_USERNAME }} -p ${{ secrets.DOCKER_PASSWORD }}
          docker push yourdockerhub/navius:${{ github.sha }}
          
      - name: Deploy to ECS
        uses: aws-actions/amazon-ecs-deploy-task-definition@v1
        with:
          task-definition: task-definition.json
          service: navius-service
          cluster: navius-cluster
          image: yourdockerhub/navius:${{ github.sha }}
```

### GitLab CI

```yaml
stages:
  - build
  - test
  - deploy

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/navius

test:
  stage: test
  image: rust:latest
  script:
    - cargo test

deploy:
  stage: deploy
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
    - kubectl set image deployment/navius navius=$CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main
```

## Database Migrations

For database migrations, Navius integrates with SQLx migrations:

```bash
# Create a new migration
cargo sqlx migrate add create_users_table

# Run migrations (automatic during application startup)
cargo sqlx migrate run
```

In production, migrations can be run:
- Automatically on application startup
- Via a dedicated migration job before deployment
- Manually in controlled environments

## Troubleshooting Common Issues

### High Memory Usage

**Symptom**: Memory usage grows over time  
**Solution**: Check for resource leaks, particularly in custom code that holds onto resources

### Slow Startup

**Symptom**: Application takes a long time to start  
**Solution**: Enable the `--release` flag, use the minimal Docker image, or precompile Rust code

### Database Connection Issues

**Symptom**: Application fails to connect to the database  
**Solution**: Verify connection strings, network connectivity, and firewall rules

## Related Documents

- [Cloud Deployment Guide](cloud-deployment.md) - Deploying to specific cloud providers
- [Environment Variables Reference](../../reference/configuration/environment-variables.md) - Complete list of configuration options
- [Testing Guide](../development/testing.md) - Ensuring application quality before deployment 
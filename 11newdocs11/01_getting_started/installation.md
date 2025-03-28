---
title: Navius Installation Guide
description: Comprehensive guide for installing, configuring, and running Navius in different environments
category: getting-started
tags:
  - installation
  - setup
  - configuration
  - prerequisites
  - deployment
related:
  - development-setup.md
  - first-steps.md
  - hello-world.md
  - ../04_guides/deployment/README.md
last_updated: March 27, 2025
version: 1.1
status: active
---

# Navius Installation Guide

## Overview

This guide provides comprehensive instructions for installing, configuring, and running Navius across different environments. It covers prerequisites, installation steps, configuration options, and verification procedures.

## Prerequisites

- **Rust** (1.70.0 or later)
  - Check version with `rustc --version`
  - Install from [rust-lang.org](https://www.rust-lang.org/tools/install)
- **Cargo** (included with Rust)
  - Check version with `cargo --version`
- **Git** (2.30.0 or later)
  - Check version with `git --version`
  - Install from [git-scm.com](https://git-scm.com/downloads)
- **OpenAPI Generator CLI** (for API client generation)
- **PostgreSQL** (optional, for database functionality)
  - Version 14 or later recommended
  - Docker (for containerized setup)
- **Redis** (optional, for caching functionality)

## Quick Start

For those familiar with Rust development, here's the quick setup process:

```shell
# Clone the repository
git clone https://github.com/your-organization/navius.git
cd navius

# Install dependencies
cargo build

# Create environment config
cp .env.example .env

# Run the application in development mode
./run_dev.sh
```

The server will start on http://localhost:3000 by default.

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/your-organization/navius.git
cd navius
```

### 2. Install Dependencies

Install all required dependencies using Cargo:

```bash
cargo build
```

This will download and compile all dependencies specified in the `Cargo.toml` file.

## Configuration

Navius uses a layered configuration approach, providing flexibility across different environments.

### YAML Configuration Files

- `config/default.yaml` - Base configuration for all environments
- `config/development.yaml` - Development-specific settings
- `config/production.yaml` - Production-specific settings
- `config/local.yaml` - Local overrides (not in version control)
- `config/local-{env}.yaml` - Environment-specific local overrides

### Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit with at minimum:

```
# Environment selection
RUN_ENV=development

# Essential environment variables
RUST_LOG=${APP_LOG_LEVEL:-info}

# Database configuration (if needed)
DATABASE_URL=postgres://username:password@localhost:5432/navius

# Secrets (if needed)
JWT_SECRET=your_jwt_secret_here
# API_KEY=your_api_key_here
```

Environment variables can also be used to override any configuration value from the YAML files, providing a secure way to manage sensitive information in production environments.

## Database Setup

### Local Development Database

#### Option 1: Using Docker (Recommended)

For local development, you can use Docker to run a PostgreSQL instance:

```bash
# From the project root:
cd test/resources/docker
docker-compose -f docker-compose.dev.yml up -d
```

This will create a PostgreSQL database accessible at:
- Host: localhost
- Port: 5432
- User: postgres
- Password: postgres
- Database: app

#### Option 2: Direct PostgreSQL Setup

If you prefer to set up PostgreSQL directly:

```bash
psql -c "CREATE DATABASE navius;"
psql -c "CREATE USER navius_user WITH ENCRYPTED PASSWORD 'your_password';"
psql -c "GRANT ALL PRIVILEGES ON DATABASE navius TO navius_user;"
```

### Database Configuration

To use with the application, ensure your `config/development.yaml` has the database section enabled:

```yaml
database:
  enabled: true
  url: "postgres://postgres:postgres@localhost:5432/app"
  max_connections: 10
  connect_timeout_seconds: 30
  idle_timeout_seconds: 300
```

> **Note**: This configuration is for local development only. Production deployments should use a managed database service like AWS RDS with appropriate security settings.

### Run Migrations

Initialize the database schema:

```bash
cargo run --bin migration
```

## Running the Server

Navius provides several ways to run the server, optimized for different scenarios.

### Using the Development Script (Recommended)

For development:

```bash
./run_dev.sh
```

The development script supports several options:

```bash
./run_dev.sh [OPTIONS]
```

Options:
- `--skip-gen` - Skip API model generation
- `--release` - Build and run in release mode
- `--config-dir=DIR` - Use specified config directory (default: config)
- `--env=FILE` - Use specified .env file (default: .env)
- `--environment=ENV` - Use specified environment (default: development)
- `--port=PORT` - Specify server port (default: 3000)
- `--watch` - Restart server on file changes
- `--run-migrations` - Run database migrations before starting
- `--no-health-check` - Skip health check validation after startup
- `--no-hooks` - Skip git hooks setup
- `--help` - Show help message

### Using the Wrapper Script

```bash
# For development (default)
./run.sh

# For production
./run.sh --prod
```

This wrapper script automatically chooses the appropriate environment script based on the `--dev` or `--prod` flag.

### Manual Run

If you prefer to run the server manually (note that this may not include all setup steps performed by the run_dev.sh script):

```bash
cargo run
```

The server will start on http://localhost:3000 by default.

## Verification

To verify that Navius has been installed correctly:

1. Start the application in development mode:

```bash
./run_dev.sh
```

2. Open your browser and navigate to:

```
http://localhost:3000/actuator/health
```

You should see a health check response indicating the application is running.

## Core Endpoints

Navius provides these built-in endpoints:

- `GET /health` - Basic health check endpoint
- `GET /metrics` - Prometheus metrics endpoint
- `GET /actuator/health` - Detailed health check with component status
- `GET /actuator/info` - System information
- `GET /docs` - OpenAPI documentation (Swagger UI)

## API Documentation

API documentation is automatically generated and available at http://localhost:3000/docs when the server is running. The documentation includes:

- All API endpoints with descriptions
- Request/response schemas
- Authentication requirements
- Example requests and responses

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Compiler errors | Ensure you have the correct Rust version (`rustc --version`) |
| Database connection errors | Check your `.env` file and database credentials |
| Port conflicts | Ensure port 3000 is not in use by another application |
| Cargo build failures | Try `cargo clean` followed by `cargo build` |
| Missing dependencies | Install missing system dependencies (e.g., OpenSSL) |

### Database Connection Issues

If you encounter database connection issues:

1. Ensure PostgreSQL is running: `docker ps` or `pg_isready`
2. Verify the database exists: `psql -l`
3. Check your connection string in `.env`
4. Ensure firewall settings allow the connection

## Next Steps

- Continue to [First Steps](first-steps.md) to learn about basic Navius concepts
- Try building a [Hello World Application](hello-world.md)
- Set up your [Development Environment](development-setup.md) for contributing

## Related Documents

- [Development Setup](/docs/getting-started/development-setup.md) - Next steps after installation
- [Development Workflow](/docs/guides/development/development-workflow.md) - Understanding the development process
- [Deployment Guide](/docs/guides/deployment.md) - Production deployment instructions 
# Navius Installation Guide

This guide provides detailed instructions for installing, configuring, and running Navius for different environments.

## Prerequisites

- Rust (latest stable version)
- OpenAPI Generator CLI (for API client generation)
- PostgreSQL (optional, for database functionality)
- Redis (optional, for caching functionality)

## Installation

### Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/evan-coleman/navius.git
   cd navius
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

## Configuration

Navius uses a layered configuration approach, providing flexibility across different environments.

### YAML Configuration Files

- `config/default.yaml` - Base configuration for all environments
- `config/development.yaml` - Development-specific settings
- `config/production.yaml` - Production-specific settings
- `config/local.yaml` - Local overrides (not in version control)
- `config/local-{env}.yaml` - Environment-specific local overrides

### Environment Variables

Create a `.env` file in the project root with at minimum:

```
# Environment selection
RUN_ENV=development

# Essential environment variables
RUST_LOG=${APP_LOG_LEVEL:-info}

# Secrets (if needed)
# API_KEY=your_api_key_here
```

Environment variables can also be used to override any configuration value from the YAML files, providing a secure way to manage sensitive information in production environments.

## Running the Server

Navius provides several ways to run the server, optimized for different scenarios.

### Using the Wrapper Script

```bash
# For development (default)
./run.sh

# For production
./run.sh --prod
```

This wrapper script automatically chooses the appropriate environment script based on the `--dev` or `--prod` flag.

### Development Mode

For development specifically:

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

The script preserves your manual settings in the API registry when generating APIs, ensuring that your customizations to `generate_api` and `generate_handlers` flags remain as you set them.

### Manual Run

If you prefer to run the server manually (note that this may not include all setup steps performed by the run_dev.sh script):

```bash
cargo run
```

The server will start on http://localhost:3000 by default.

## Database Setup

### Local Development Database

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

For detailed implementation of the PostgreSQL connection, see the [PostgreSQL Integration Guide](postgresql_integration.md).

## API Documentation

API documentation is automatically generated and available at http://localhost:3000/docs when the server is running. The documentation includes:

- All API endpoints with descriptions
- Request/response schemas
- Authentication requirements
- Example requests and responses

## Core Endpoints

Navius provides these built-in endpoints:

- `GET /health` - Health check endpoint
- `GET /metrics` - Prometheus metrics endpoint
- `GET /actuator/health` - Detailed health check with component status
- `GET /actuator/info` - System information
- `GET /docs` - OpenAPI documentation (Swagger UI)

## Production Deployment

For production deployment, refer to the [Deployment Guide](deployment.md) which covers:

- AWS infrastructure setup
- Container deployment options
- Security best practices
- Scaling considerations
- Monitoring and observability 
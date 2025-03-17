# Rust Backend

A modular Rust backend application with RESTful API endpoints, OpenAPI documentation, caching, metrics, and more.

## Features

- **RESTful API** using [Axum](https://github.com/tokio-rs/axum)
- **OpenAPI Documentation** with [Utoipa](https://github.com/juhaku/utoipa) and Swagger UI
- **Caching** with [Moka](https://github.com/moka-rs/moka)
- **Metrics Collection** using [metrics](https://github.com/metrics-rs/metrics)
- **Prometheus Integration** for metrics reporting
- **Structured Error Handling**
- **Configuration Management** with YAML files and environment variables
- **Logging** with [tracing](https://github.com/tokio-rs/tracing)

## Project Structure

```
src/
├── app/                  # Application router and state
│   └── router.rs
├── cache/                # Caching functionality
│   └── cache_manager.rs
├── config/               # Configuration management
│   └── app_config.rs
├── error/                # Error handling
│   └── error_types.rs
├── handlers/             # API request handlers
│   ├── data.rs
│   ├── health.rs
│   ├── metrics.rs
│   ├── mod.rs
│   └── pet.rs
├── metrics/              # Metrics collection and reporting
│   └── metrics_service.rs
├── models/               # Data models and schemas
│   ├── mod.rs
│   └── schemas.rs
├── petstore_api/         # Generated Petstore API client
├── lib.rs                # Library module declarations
└── main.rs               # Application entry point

config/
├── default.yaml          # Default configuration
├── development.yaml      # Development environment configuration
├── production.yaml       # Production environment configuration
└── local.yaml            # Local overrides (not in version control)
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- OpenAPI Generator CLI (for API client generation)

### Installation

1. Clone the repository
2. Install dependencies:

```bash
cargo build
```

### Configuration

The application uses a layered configuration approach:

1. **YAML Configuration Files**:
   - `config/default.yaml` - Base configuration for all environments
   - `config/development.yaml` - Development-specific settings
   - `config/production.yaml` - Production-specific settings
   - `config/local.yaml` - Local overrides (not in version control)
   - `config/local-{env}.yaml` - Environment-specific local overrides

2. **Environment Variables**:
   Create a `.env` file in the project root with at minimum:

```
# Environment selection
RUN_ENV=development

# Essential environment variables
RUST_LOG=${APP_LOG_LEVEL:-info}

# Secrets (if needed)
# API_KEY=your_api_key_here
```

Environment variables can also be used to override any configuration value from the YAML files.

### Running the Server

```bash
./run_server.sh
```

The script supports several options:

```bash
./run_server.sh [OPTIONS]
```

Options:
- `--skip-gen` - Skip API model generation
- `--release` - Build and run in release mode
- `--config-dir=DIR` - Use specified config directory (default: config)
- `--env=FILE` - Use specified .env file (default: .env)
- `--environment=ENV` - Use specified environment (default: development)
- `--help` - Show help message

The script always preserves your manual settings in the API registry when generating APIs, ensuring that your customizations to `generate_api` and `generate_handlers` flags remain as you set them.

Or manually (note the run_server.sh script has required steps to run the application so this may not work):

```bash
cargo run
```

The server will start on http://localhost:3000 by default.

## API Documentation

API documentation is available at http://localhost:3000/docs when the server is running.

## Endpoints

- `GET /health` - Health check endpoint
- `GET /metrics` - Prometheus metrics endpoint
- `GET /data` - Sample data endpoint (fetches cat facts)
- `GET /pet/{id}` - Fetch pet by ID from the Petstore API

## API Integration

This project supports easy integration with downstream APIs. To add a new API endpoint:

1. **Automated Method (Recommended)**:
   ```bash
   ./scripts/add_api.sh <api_name> <api_url> <schema_url> [endpoint_path] [param_name]
   ```

   For example:
   ```bash
   ./scripts/add_api.sh jsonplaceholder https://jsonplaceholder.typicode.com https://jsonplaceholder.typicode.com/swagger.json posts id
   ```

2. **Manual Method**:
   See the detailed guide in [API Integration Guide](docs/API_INTEGRATION.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
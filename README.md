# Rust Backend

A modular Rust backend application with RESTful API endpoints, OpenAPI documentation, caching, metrics, and more.

## Features

- **RESTful API** using [Axum](https://github.com/tokio-rs/axum)
- **OpenAPI Documentation** with [Utoipa](https://github.com/juhaku/utoipa) and Swagger UI
- **Caching** with [Moka](https://github.com/moka-rs/moka)
- **Metrics Collection** using [metrics](https://github.com/metrics-rs/metrics)
- **Prometheus Integration** for metrics reporting
- **Structured Error Handling**
- **Configuration Management** with environment variables and config files
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

Create a `.env` file in the project root with the following variables:

```
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
SERVER_TIMEOUT_SECONDS=10
SERVER_MAX_RETRIES=3

# External API endpoints
API_CAT_FACT_URL=https://catfact.ninja/fact
API_PETSTORE_URL=https://petstore3.swagger.io/api/v3

# Application settings
APP_NAME="Petstore API Server"
APP_VERSION="1.0.0"
APP_LOG_LEVEL=info

# Cache settings
CACHE_ENABLED=true
CACHE_TTL_SECONDS=300
CACHE_MAX_CAPACITY=1000
```

### Running the Server

```bash
./run_server.sh
```

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

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
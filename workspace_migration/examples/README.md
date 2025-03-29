# Navius

An enterprise-grade web framework built with Rust, designed as a high-performance alternative to Spring Boot.

## Workspace Structure

Navius is organized as a Rust workspace with multiple crates:

- `navius-core`: Essential, mandatory functionality
- `navius-api`: Main application that ties everything together
- `navius-auth`: Authentication functionality
- `navius-metrics`: Metrics and monitoring capabilities
  - `navius-metrics-prometheus`: Prometheus integration
  - `navius-metrics-dynatrace`: Dynatrace integration
- `navius-database`: Database functionality
  - `navius-database-postgres`: PostgreSQL support
- `navius-cache`: Caching functionality
  - `navius-cache-redis`: Redis support
- `navius-cli`: CLI tools
- `navius-test-utils`: Testing utilities

This modular approach allows you to include only the components you need, resulting in smaller binaries and faster compile times.

## Getting Started

### Installation

Add Navius to your project by including the crates you need:

```toml
[dependencies]
# Core functionality (required)
navius-core = "0.1.0"

# Optional components based on your needs
navius-api = "0.1.0"
navius-auth = "0.1.0"
navius-metrics = "0.1.0"
navius-metrics-prometheus = "0.1.0"
navius-database = "0.1.0"
navius-database-postgres = "0.1.0"
navius-cache = "0.1.0"
navius-cache-redis = "0.1.0"
```

### Basic Usage

```rust
use navius_core::config::Config;
use navius_api::ApiBuilder;

fn main() {
    // Initialize core functionality
    let config = Config::from_env().expect("Failed to load configuration");
    navius_core::init_with_config(config.clone()).expect("Failed to initialize core");
    
    // Build and run the API
    let api = ApiBuilder::new()
        .with_config(config)
        .with_metrics() // Optional: Add metrics support
        .with_auth()    // Optional: Add authentication
        .build()
        .expect("Failed to build API");
        
    api.run().expect("Failed to run API");
}
```

### Using Specific Components

#### Metrics with Prometheus

```rust
use navius_metrics::MetricsBuilder;
use navius_metrics_prometheus::PrometheusRecorder;

// Initialize metrics with Prometheus
let metrics = MetricsBuilder::new()
    .with_recorder(PrometheusRecorder::new())
    .build()
    .expect("Failed to initialize metrics");

// Record a metric
metrics.counter("api.requests.total").increment(1);
```

#### Authentication

```rust
use navius_auth::{Auth, AuthConfig};

// Initialize authentication
let auth = Auth::new(AuthConfig {
    jwt_secret: "your-secret".to_string(),
    token_expiry: 3600,
    ..Default::default()
});

// Create and validate tokens
let token = auth.create_token("user123").expect("Failed to create token");
let claims = auth.validate_token(&token).expect("Invalid token");
```

## Development

### Building from Source

Clone the repository and build all crates:

```bash
git clone https://github.com/Evan-Coleman/Navius.git
cd Navius
cargo build
```

Or build only specific crates:

```bash
cargo build -p navius-core -p navius-api
```

### Running Tests

Run all tests:

```bash
cargo test
```

Or test specific crates:

```bash
cargo test -p navius-core
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to Navius.

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details. 
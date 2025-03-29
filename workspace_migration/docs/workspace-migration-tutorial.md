# Workspace Migration Tutorial

This tutorial provides step-by-step instructions for migrating the Navius project from a feature flag-based approach to a Rust workspace.

## Step 1: Set Up the Workspace Configuration

First, we need to modify the root `Cargo.toml` to define the workspace structure:

```toml
[workspace]
members = [
    "crates/navius-core",
    # Other crates will be added as they are created
]

resolver = "2"

[workspace.dependencies]
# Common dependencies with versions
tokio = { version = "1.44.1", features = ["rt-multi-thread", "macros"] }
tracing = { version = "0.1.41", features = ["log"] }
# ... add other common dependencies
```

## Step 2: Create the Core Crate

The core crate will contain essential functionality used by all other crates.

1. Create the directory structure:

```bash
mkdir -p crates/navius-core/src
```

2. Create `crates/navius-core/Cargo.toml`:

```toml
[package]
name = "navius-core"
version = "0.1.0"
edition = "2024"
description = "Core functionality for the Navius framework"
license = "Apache-2.0"
repository = "https://github.com/Evan-Coleman/Navius"

[dependencies]
tokio = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
# ... other dependencies
```

3. Create `crates/navius-core/src/lib.rs` with the basic module structure:

```rust
//! Core functionality for the Navius framework.

// Re-export core modules
pub mod error;
pub mod config;
pub mod constants;
pub mod types;

// Public exports
pub use error::Error;
pub use config::Config;
```

4. Move core code from the main project into appropriate modules in the core crate:
   - Error types and handling
   - Configuration structures
   - Common utilities and constants
   - Core type definitions

## Step 3: Extract the First Feature Module

Let's extract the metrics module as an example:

1. Create the directory structure:

```bash
mkdir -p crates/navius-metrics/src
```

2. Create `crates/navius-metrics/Cargo.toml` (see the example in the roadmap documents).

3. Identify all metrics-related code in the current codebase:

```bash
grep -r "metrics" --include="*.rs" src/
```

4. Move the metrics code to the new crate, updating imports as needed.

5. Create a clean API in `crates/navius-metrics/src/lib.rs`:

```rust
//! Metrics functionality for the Navius framework.

pub mod recorder;
pub mod counter;
pub mod gauge;
pub mod histogram;

// Public API
pub use recorder::MetricsRecorder;
pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;

/// Initialize the metrics system with default configuration.
pub fn init() -> Result<MetricsRecorder, crate::error::Error> {
    // Implementation
}

/// Initialize the metrics system with custom configuration.
pub fn init_with_config(config: crate::config::MetricsConfig) -> Result<MetricsRecorder, crate::error::Error> {
    // Implementation
}
```

## Step 4: Create a Provider Implementation

For each backend provider (e.g., Prometheus), create a separate crate:

1. Create directory structure:

```bash
mkdir -p crates/navius-metrics-prometheus/src
```

2. Create `crates/navius-metrics-prometheus/Cargo.toml` (see example in roadmap documents).

3. Implement the provider-specific code:

```rust
//! Prometheus implementation for Navius metrics.

use navius_metrics::recorder::MetricsRecorder;

pub struct PrometheusRecorder {
    // Implementation details
}

impl navius_metrics::recorder::MetricsRecorder for PrometheusRecorder {
    // Implementation
}

pub fn init(config: navius_metrics::config::MetricsConfig) -> Result<PrometheusRecorder, navius_metrics::error::Error> {
    // Implementation
}
```

## Step 5: Update the Main Application

Modify the main application to use the new crates:

1. Update `Cargo.toml` to depend on the new crates.

2. Update imports in the main code to use the new namespaces.

3. Replace feature flag conditions with direct imports from the appropriate crates.

## Step 6: Test and Verify

After each module extraction:

1. Run the tests for the extracted module:

```bash
cd crates/navius-metrics
cargo test
```

2. Run the main application tests to ensure everything still works:

```bash
cargo test
```

3. Build the application with different configurations to verify it works as expected.

## Step 7: Document the Changes

1. Update the README.md with instructions on how to use the new workspace structure.

2. Create examples showing how to include different combinations of crates.

3. Update any existing documentation that referenced feature flags.

## Step 8: Repeat for Other Modules

Follow the same process for each major feature module:

1. Authentication
2. Database
3. Caching
4. ...

## Tips for a Successful Migration

1. **Take it step by step** - Don't try to migrate everything at once.

2. **Keep the existing code working** - During the migration, maintain backward compatibility.

3. **Use clear module boundaries** - Define clear interfaces between crates.

4. **Write tests as you go** - Ensure each extracted module has good test coverage.

5. **Update documentation** - Keep documentation up-to-date with the new structure.

6. **Check binary sizes** - Verify that the new structure actually reduces binary sizes for minimal configurations.

7. **Monitor build times** - Use `cargo build -Z timings` to monitor changes in build performance.

## Example: Building with Specific Functionality

Before:
```bash
cargo build --features "metrics,prometheus,database"
```

After:
```bash
# In your application's Cargo.toml
[dependencies]
navius-core = "0.1.0"
navius-metrics = "0.1.0"
navius-metrics-prometheus = "0.1.0"
navius-database = "0.1.0"
```

Then simply:
```bash
cargo build
```

The application will only include the functionality you've explicitly imported. 
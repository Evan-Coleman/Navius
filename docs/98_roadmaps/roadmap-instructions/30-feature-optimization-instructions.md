---
title: "Feature Flag Optimization Guide"
description: "Instructions for implementing binary size and compilation time optimizations through feature flags"
category: implementation-guide
tags:
  - optimization
  - feature-flags
  - dependencies
  - binary-size
last_updated: March 28, 2025
version: 1.0
---

# Feature Flag Optimization Guide

This guide provides detailed instructions for implementing feature flag optimizations to reduce binary size, improve compilation time, and enhance code maintainability.

## Prerequisites

Before starting optimization work:
- Understand the existing feature flag system in `Cargo.toml`
- Familiarize yourself with conditional compilation in Rust (`#[cfg(feature = "...")]`)
- Have a baseline of binary sizes with all features enabled
- Understand module dependencies and relationships

## Analyzing Code for Feature Flag Opportunities

### Step 1: Identify Optional Features

Start by identifying code that should be optional:

1. **Third-party dependencies**: Look for dependencies that are only needed for specific features
2. **Alternate implementations**: Look for multiple implementations of the same interface
3. **Optional functionality**: Identify features that aren't core to the application
4. **Developer utilities**: Find code only used during development

Use this command to find dependencies in your codebase:
```bash
cargo tree --all-features
```

### Step 2: Map Dependencies to Features

Create a dependency matrix to understand relationships:

1. List all dependencies in rows
2. List all features in columns
3. Mark which dependencies are required for each feature
4. Identify opportunities to make dependencies optional

Example dependency matrix format:
```
| Dependency           | default | metrics | database | auth | redis |
|----------------------|---------|---------|----------|------|-------|
| tokio                | ✓       | ✓       | ✓        | ✓    | ✓     |
| axum                 | ✓       | ✓       | ✓        | ✓    | ✓     |
| prometheus           | ✓       | ✓       | -        | -    | -     |
| sqlx                 | ✓       | -       | ✓        | -    | -     |
| redis                | -       | -       | -        | -    | ✓     |
```

### Step 3: Measure Binary Size Impact

Determine which dependencies have the biggest impact:

1. Create a baseline build with all features enabled:
   ```bash
   cargo build --release
   ls -lh target/release/navius
   ```

2. Disable each feature one by one and measure size:
   ```bash
   cargo build --release --no-default-features --features "metrics,database,auth"
   ls -lh target/release/navius
   ```

3. Document findings to prioritize optimization efforts

## Implementing Feature Flag Optimizations

### Step 1: Make Dependencies Optional in Cargo.toml

Convert direct dependencies to optional:

```toml
[dependencies]
# Before
redis = "0.22.1"

# After
redis = { version = "0.22.1", optional = true }
```

Then link the dependency to a feature flag:

```toml
[features]
redis = ["dep:redis"]
```

### Step 2: Gate Imports with Feature Flags

Use conditional compilation for imports:

```rust
// Always imported
use std::sync::Arc;

// Conditionally imported
#[cfg(feature = "redis")]
use redis::{Client, Commands};
```

### Step 3: Conditionally Compile Modules

Gate entire modules based on features:

```rust
// Core modules always available
pub mod core;
pub mod common;

// Optional modules
#[cfg(feature = "redis")]
pub mod redis_cache;

#[cfg(feature = "database")]
pub mod database;
```

### Step 4: Create Conditional Re-exports

Control what's exposed from modules:

```rust
// Always available
pub use self::core::CoreService;

// Conditionally available
#[cfg(feature = "redis")]
pub use self::redis_cache::RedisCache;
```

### Step 5: Implement Feature-Gated Functions and Structs

Use conditional compilation for implementations:

```rust
// Base trait definition (always available)
pub trait CacheProvider {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str) -> Result<(), Error>;
}

// Memory implementation (always available)
pub struct MemoryCache { /* ... */ }
impl CacheProvider for MemoryCache { /* ... */ }

// Redis implementation (conditionally available)
#[cfg(feature = "redis")]
pub struct RedisCache { /* ... */ }

#[cfg(feature = "redis")]
impl CacheProvider for RedisCache { /* ... */ }
```

### Step 6: Handle Entry Points with Feature Variations

Provide different implementations based on available features:

```rust
// Core function always available
pub fn create_default_cache() -> Box<dyn CacheProvider> {
    Box::new(MemoryCache::new())
}

// Enhanced version when redis is available
#[cfg(feature = "redis")]
pub fn create_cache(config: &Config) -> Box<dyn CacheProvider> {
    if config.use_redis {
        Box::new(RedisCache::new(config.redis_url.clone()))
    } else {
        Box::new(MemoryCache::new())
    }
}

// Fallback when redis is not available
#[cfg(not(feature = "redis"))]
pub fn create_cache(_config: &Config) -> Box<dyn CacheProvider> {
    Box::new(MemoryCache::new())
}
```

## Best Practices

### Feature Flag Granularity

1. **Coarse-grained features**: For major subsystems (database, metrics, auth)
2. **Medium-grained features**: For specific implementations (postgres, redis)
3. **Fine-grained features**: For behavior variations (advanced_metrics)

Choose the appropriate level based on:
- Binary size impact
- Compilation time impact
- API complexity impact
- Testing complexity

### Testing Feature Combinations

Ensure code works with different feature combinations:

1. Test with all features enabled
2. Test with minimal features enabled
3. Test each feature individually
4. Test common feature combinations

Add these to CI or create a script:

```bash
#!/bin/bash
# Test common feature combinations
cargo test --no-default-features --features "metrics"
cargo test --no-default-features --features "database"
cargo test --no-default-features --features "metrics,database"
cargo test --all-features
```

### Documentation

Document feature flags thoroughly:

1. In Cargo.toml:
```toml
[features]
# Enable metrics collection and exposition
metrics = ["dep:prometheus", "dep:metrics"]

# Database support with SQLx
database = ["dep:sqlx", "dep:tokio-postgres"]
```

2. In README.md, add a feature flag section:
```markdown
## Feature Flags

This crate provides several feature flags to customize functionality:

- `metrics`: Enables metrics collection using Prometheus
- `database`: Adds database support via SQLx
- `redis`: Enables Redis caching support
- `auth`: Adds authentication and authorization

Example minimal build:
```bash
cargo build --no-default-features --features "database,redis"
```
```

## Real-World Examples

### Example 1: Database Provider Optimization

```rust
// src/database/mod.rs
pub mod interface;
pub mod error;

// Core types always available
pub use self::interface::{DatabaseProvider, QueryResult};
pub use self::error::DatabaseError;

// Provider implementations conditionally available
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use self::postgres::PostgresProvider;

#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlite")]
pub use self::sqlite::SqliteProvider;

// Memory provider always available for testing
pub mod memory;
pub use self::memory::MemoryProvider;

// Factory function with feature-dependent behavior
pub fn create_provider(url: &str) -> Result<Box<dyn DatabaseProvider>, DatabaseError> {
    if url.starts_with("memory:") {
        return Ok(Box::new(MemoryProvider::new()));
    }
    
    #[cfg(feature = "postgres")]
    if url.starts_with("postgres:") {
        return PostgresProvider::create(url).map(|p| Box::new(p) as Box<dyn DatabaseProvider>);
    }
    
    #[cfg(feature = "sqlite")]
    if url.starts_with("sqlite:") {
        return SqliteProvider::create(url).map(|p| Box::new(p) as Box<dyn DatabaseProvider>);
    }
    
    Err(DatabaseError::UnsupportedProvider(format!("No provider available for URL: {}", url)))
}
```

### Example 2: Conditional Service Registration

```rust
// src/app/mod.rs
pub fn register_services(registry: &mut ServiceRegistry, config: &Config) {
    // Core services always registered
    registry.register::<CoreService>(CoreService::new());
    
    // Feature-dependent services
    #[cfg(feature = "metrics")]
    {
        let metrics_service = MetricsService::new(&config.metrics);
        registry.register::<MetricsService>(metrics_service);
    }
    
    #[cfg(feature = "database")]
    {
        let db_provider = create_database_provider(&config.database)
            .expect("Failed to create database provider");
        registry.register_instance::<dyn DatabaseProvider>(db_provider);
    }
}
```

## Measuring Success

After implementing feature flag optimizations, measure the impact:

1. **Binary size**:
   ```bash
   # Before optimization (baseline)
   cargo build --release
   ls -lh target/release/navius
   
   # After optimization (minimal build)
   cargo build --release --no-default-features --features "minimal"
   ls -lh target/release/navius
   ```

2. **Compilation time**:
   ```bash
   # Before optimization
   time cargo build
   
   # After optimization
   time cargo build --no-default-features --features "minimal"
   ```

3. **Dependency count**:
   ```bash
   # Count dependencies before
   cargo tree | wc -l
   
   # Count dependencies after
   cargo tree --no-default-features --features "minimal" | wc -l
   ```

Document these metrics to showcase the optimization's success.

## Additional Resources

- [Rust Cargo Manifest Documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section)
- [The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional Compilation in Rust](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Binary Size Optimization Techniques](https://github.com/johnthagen/min-sized-rust) 
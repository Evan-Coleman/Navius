# Feature Selection

This guide explains how to use feature flags in Navius to selectively enable or disable functionality and customize your application.

## Understanding Feature Flags

Feature flags in Navius allow you to:

1. **Include Optional Dependencies**: Enable or disable dependencies like database drivers
2. **Configure Application Behavior**: Enable or disable entire subsystems
3. **Customize Builds**: Create production, development, or specialized builds
4. **Optimize Binary Size**: Include only what you need
5. **Enable Optional Extensions**: Add extension points without affecting core functionality

## Core Features

Navius defines several core features in its `Cargo.toml`:

```toml
[features]
default = ["postgres", "redis", "metrics"]

# Database options
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]

# Cache options
redis = ["redis-rs"]
memcached = ["memcache"]

# Observability
metrics = ["prometheus"]
tracing = ["opentelemetry"]

# Security
jwt = ["jsonwebtoken"]
oauth = ["oauth2"]

# Server optimizations
production = []
development = ["tracing"]
```

## Enabling Features

Features can be enabled in your project's `Cargo.toml`:

```toml
[dependencies]
navius = { version = "0.1.0", default-features = false, features = ["postgres", "redis", "metrics"] }
```

Or at compile time:

```bash
cargo build --no-default-features --features "postgres redis metrics"
```

## Feature-Gated Code

In your code, use feature flags with `#[cfg(...)]` attributes:

```rust
// Only compile this module when the "postgres" feature is enabled
#[cfg(feature = "postgres")]
pub mod postgres_repository {
    use sqlx::postgres::PgPool;
    
    pub struct PostgresRepository {
        pool: PgPool,
    }
    
    impl PostgresRepository {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
        
        pub async fn get_user(&self, id: &str) -> Result<User, Error> {
            // Implementation for postgres
        }
    }
}

// Only compile this module when the "sqlite" feature is enabled
#[cfg(feature = "sqlite")]
pub mod sqlite_repository {
    use sqlx::sqlite::SqlitePool;
    
    pub struct SqliteRepository {
        pool: SqlitePool,
    }
    
    impl SqliteRepository {
        pub fn new(pool: SqlitePool) -> Self {
            Self { pool }
        }
        
        pub async fn get_user(&self, id: &str) -> Result<User, Error> {
            // Implementation for sqlite
        }
    }
}
```

## Conditional Compilation

Features allow for conditional compilation:

```rust
pub fn initialize_database() -> Box<dyn DatabaseProvider> {
    #[cfg(feature = "postgres")]
    {
        let pool = create_postgres_pool().await;
        return Box::new(postgres_repository::PostgresRepository::new(pool));
    }
    
    #[cfg(feature = "mysql")]
    {
        let pool = create_mysql_pool().await;
        return Box::new(mysql_repository::MySqlRepository::new(pool));
    }
    
    #[cfg(feature = "sqlite")]
    {
        let pool = create_sqlite_pool().await;
        return Box::new(sqlite_repository::SqliteRepository::new(pool));
    }
    
    // Fallback to in-memory implementation
    Box::new(memory_repository::MemoryRepository::new())
}
```

## Feature Dependencies

Features can depend on other features:

```toml
[features]
default = ["web"]
web = ["axum", "tokio"]
api = ["web", "swagger"]
full = ["web", "api", "admin"]
admin = ["web", "authorization"]
authorization = ["jwt"]
```

## Runtime Feature Detection

Check feature flags at runtime:

```rust
pub fn is_development_mode() -> bool {
    cfg!(feature = "development")
}

pub fn is_metrics_enabled() -> bool {
    cfg!(feature = "metrics")
}

pub fn initialize_services() {
    if is_metrics_enabled() {
        println!("Initializing metrics service");
        // Initialize metrics
    }
    
    if is_development_mode() {
        println!("Running in development mode");
        // Enable development features
    }
}
```

## Feature-Specific Configuration

Configure services based on features:

```rust
pub fn configure_logging() {
    let mut log_builder = env_logger::Builder::new();
    
    #[cfg(feature = "development")]
    {
        log_builder.filter_level(log::LevelFilter::Debug);
        log_builder.filter(Some("sqlx"), log::LevelFilter::Info);
    }
    
    #[cfg(not(feature = "development"))]
    {
        log_builder.filter_level(log::LevelFilter::Info);
        log_builder.filter(Some("sqlx"), log::LevelFilter::Warn);
    }
    
    log_builder.init();
}
```

## Common Feature Sets

Navius recommends these feature combinations:

### Basic Application

```toml
navius = { version = "0.1.0", features = ["sqlite"] }
```

### Web API

```toml
navius = { version = "0.1.0", features = ["postgres", "redis", "jwt"] }
```

### Full Production Stack

```toml
navius = { version = "0.1.0", features = ["postgres", "redis", "metrics", "tracing", "production"] }
```

### Development Environment

```toml
navius = { version = "0.1.0", features = ["sqlite", "development"] }
```

## Feature Testing

Test different feature combinations:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(feature = "postgres")]
    fn test_postgres_repository() {
        // Test postgres-specific functionality
    }
    
    #[test]
    #[cfg(feature = "sqlite")]
    fn test_sqlite_repository() {
        // Test sqlite-specific functionality
    }
    
    #[test]
    fn test_common_functionality() {
        // Test functionality available in all feature combinations
    }
}
```

## Using Feature Flags in Your Application

In your application, you can use feature flags for your own features:

```toml
# In your application's Cargo.toml
[features]
default = ["navius/postgres", "user-management", "payments"]
user-management = []
payments = ["stripe"]
admin-panel = []
analytics = ["clickhouse"]
```

## Feature Documentation

Document what each feature enables:

```rust
/// User management module
///
/// This module provides user registration, authentication, and profile management.
///
/// # Features
///
/// - Enabled with the `user-management` feature flag
/// - Requires a database (postgres, mysql, or sqlite)
/// - Provides both synchronous and asynchronous APIs
///
/// # Examples
///
/// ```
/// use navius::user_management::UserService;
///
/// let user_service = UserService::new();
/// let user = user_service.register("user@example.com", "password123").await?;
/// ```
#[cfg(feature = "user-management")]
pub mod user_management {
    // Module implementation
}
```

## Production vs Development Mode

Navius provides specific features for production and development:

```rust
// In src/main.rs
fn main() {
    #[cfg(feature = "development")]
    {
        println!("Running in development mode");
        // Initialize development tools
        initialize_dev_tools();
    }
    
    #[cfg(feature = "production")]
    {
        println!("Running in production mode");
        // Initialize production optimizations
        initialize_production_settings();
    }
    
    // Start the application
    start_application();
}
```

## Best Practices

1. **Design for Modularity**: Structure code with clear feature boundaries
2. **Sensible Defaults**: Make the default feature set cover common needs
3. **Document Features**: Clearly document what each feature enables
4. **Test Feature Combinations**: Test various feature combinations in CI
5. **Use Feature Detection**: Use `cfg!()` for runtime feature checks
6. **Consistent Naming**: Use consistent feature naming schemes
7. **Feature Granularity**: Make features neither too large nor too small
8. **Documentation**: Add feature information to function and module docs

## Common Pitfalls

1. **Feature Combinatorial Explosion**: Too many optional features can lead to many untested combinations
2. **Missing Dependencies**: Forgetting to declare feature dependencies
3. **Redundant Conditionals**: Using runtime checks for compile-time features
4. **Feature Leakage**: Public API depending on features not declared in dependencies
5. **Incomplete Testing**: Not testing all important feature combinations

## Related Guides

- [Configuration](configuration.md) for configuring feature-specific settings
- [Dependency Injection](dependency-injection.md) for working with feature-specific services
- [Testing](testing.md) for testing different feature combinations 
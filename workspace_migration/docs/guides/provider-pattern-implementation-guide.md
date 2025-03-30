# Provider Pattern Implementation Guide

**Version:** 1.0 (DRAFT)  
**Date:** March 29, 2025  
**Status:** Draft  

## Introduction

This guide outlines best practices for implementing the provider pattern in Navius framework crates. The provider pattern is a key architectural approach used throughout the Navius framework to separate interface definitions from their concrete implementations, enabling flexibility, modularity, and extensibility.

Based on findings from the design evaluations of `navius-db` and `navius-cache` crates, this guide aims to standardize the implementation of the provider pattern across the framework to ensure consistency and maintainability.

## Core Principles

The provider pattern in Navius is built on the following key principles:

1. **Interface-Implementation Separation**: Clear separation between the abstract interface and concrete implementations
2. **Feature-Gated Implementation**: Provider-specific implementations are gated behind feature flags
3. **Configuration-Driven Selection**: Providers are selected and configured through structured configuration
4. **Factory Pattern**: Provider instantiation is managed through factory methods
5. **Connection Management**: Consistent connection pooling and lifecycle management
6. **Error Handling**: Unified error handling approach with context preservation
7. **Metrics Integration**: Consistent metrics and observability

## Implementation Guidelines

### 1. Core Interface Definitions

Define the core traits that providers must implement:

```rust
/// Core trait for a cache provider
#[async_trait]
pub trait CacheProvider: Send + Sync + 'static {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;
    
    /// Connect to the cache backend
    async fn connect(&self) -> Result<()>;
    
    /// Get a connection from the provider
    async fn get_connection(&self) -> Result<Box<dyn Connection>>;
    
    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

### 2. Provider Type Enumeration

Create an enumeration of supported provider types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// In-memory implementation
    InMemory,
    /// Redis implementation
    #[cfg(feature = "redis")]
    Redis,
    /// Memcached implementation 
    #[cfg(feature = "memcached")]
    Memcached,
    /// Custom implementation
    Custom,
}
```

### 3. Configuration Structures

Define configuration structures that can be deserialized from various sources:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    /// Provider type identifier
    pub provider_type: ProviderType,
    /// Provider name for identification
    pub name: String,
    /// Provider-specific configuration
    #[serde(flatten)]
    pub config: serde_json::Value,
}
```

### 4. Factory Pattern

Implement a factory pattern for creating provider instances:

```rust
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a new provider from configuration
    pub fn create(config: &ProviderConfig) -> Result<Arc<dyn Provider>> {
        match config.provider_type {
            ProviderType::InMemory => {
                let memory_config = serde_json::from_value::<InMemoryConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(InMemoryProvider::new(
                    config.name.clone(),
                    memory_config,
                )))
            }
            #[cfg(feature = "redis")]
            ProviderType::Redis => {
                let redis_config = serde_json::from_value::<RedisConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(RedisProvider::new(
                    config.name.clone(),
                    redis_config,
                )))
            }
            // Other provider types...
            ProviderType::Custom => Err(Error::configuration(
                "Custom providers must be created manually",
            )),
        }
    }
}
```

### 5. Registry for Provider Management

Implement a registry pattern for managing providers:

```rust
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn Provider>>>,
    default_provider: RwLock<Option<String>>,
}

impl ProviderRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default_provider: RwLock::new(None),
        }
    }
    
    /// Register a provider
    pub async fn register(&self, provider: Arc<dyn Provider>) -> Result<()> {
        let name = provider.name().to_string();
        let mut providers = self.providers.write().await;
        providers.insert(name, provider);
        Ok(())
    }
    
    /// Get a provider by name
    pub async fn get(&self, name: &str) -> Result<Arc<dyn Provider>> {
        let providers = self.providers.read().await;
        providers.get(name)
            .cloned()
            .ok_or_else(|| Error::provider_not_found(name))
    }
    
    /// Set the default provider
    pub async fn set_default(&self, name: &str) -> Result<()> {
        if !self.providers.read().await.contains_key(name) {
            return Err(Error::provider_not_found(name));
        }
        *self.default_provider.write().await = Some(name.to_string());
        Ok(())
    }
    
    /// Get the default provider
    pub async fn default(&self) -> Result<Arc<dyn Provider>> {
        let default_name = self.default_provider.read().await
            .clone()
            .ok_or_else(|| Error::configuration("No default provider configured"))?;
        
        self.get(&default_name).await
    }
}
```

### 6. Feature-Gated Implementation Module

Organize provider implementations under feature gates:

```rust
// In lib.rs
#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "memcached")]
pub mod memcached;

// Always available
pub mod memory;
```

### 7. Provider Implementations

Implement concrete providers that adhere to the interface:

```rust
pub struct RedisProvider {
    name: String,
    config: RedisConfig,
    pool: Mutex<Option<ConnectionPool>>,
}

#[async_trait]
impl Provider for RedisProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn provider_type(&self) -> ProviderType {
        ProviderType::Redis
    }
    
    async fn connect(&self) -> Result<()> {
        // Implementation
    }
    
    async fn get_connection(&self) -> Result<Box<dyn Connection>> {
        // Implementation
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        // Implementation
    }
}
```

## Design Considerations

### Configuration Management

1. **Hierarchical Configuration**: Support both global and provider-specific configuration values
2. **Environment Variables**: Allow configuration via environment variables for deployment flexibility
3. **Validation**: Perform configuration validation during provider creation, with clear error messages
4. **Defaults**: Provide reasonable defaults for non-critical configuration options

Example:
```rust
pub struct RedisConfig {
    /// Connection URL
    pub url: String,
    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// TLS configuration
    #[serde(default)]
    pub tls_enabled: bool,
}

fn default_pool_size() -> usize {
    10
}

fn default_timeout() -> u64 {
    30
}
```

### Error Handling

1. **Error Categorization**: Categorize errors into distinct types (configuration, connection, operation)
2. **Context Preservation**: Include relevant context in error messages
3. **Provider-Specific Errors**: Convert provider-specific errors to framework error types
4. **Status Code Mapping**: Map errors to appropriate HTTP status codes for web applications

Example:
```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Operation error: {0}")]
    Operation(String),
    
    #[error("Provider-specific error: {0}")]
    ProviderError(String),
}

impl Error {
    pub fn configuration<S: ToString>(msg: S) -> Self {
        Error::Configuration(msg.to_string())
    }
    
    pub fn connection<S: ToString>(msg: S) -> Self {
        Error::Connection(msg.to_string())
    }
    
    pub fn provider_not_found<S: ToString>(name: S) -> Self {
        Error::ProviderNotFound(name.to_string())
    }
}
```

### Connection Management

1. **Connection Pooling**: Implement connection pooling for expensive connections
2. **Health Checking**: Include health check logic to validate connections
3. **Graceful Reconnection**: Handle connection failures with retry logic
4. **Cleanup**: Properly close connections during shutdown

Example:
```rust
pub struct ConnectionPool {
    connections: Vec<PooledConnection>,
    max_size: usize,
    connection_timeout: Duration,
}

impl ConnectionPool {
    pub fn new(max_size: usize, connection_timeout: Duration) -> Self {
        Self {
            connections: Vec::with_capacity(max_size),
            max_size,
            connection_timeout,
        }
    }
    
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        // Implementation with timeouts and retries
    }
    
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // Validate connection pool health
    }
}
```

### Metrics and Observability

1. **Standard Metrics**: Define standard metrics across all providers (connections, operations, errors)
2. **Provider-Specific Metrics**: Allow additional provider-specific metrics
3. **Tracing**: Include comprehensive tracing for debugging
4. **Health Information**: Report detailed health information

Example:
```rust
#[cfg(feature = "metrics")]
pub fn record_operation_metrics(
    provider_type: &str,
    operation: &str,
    duration: Duration,
    success: bool,
) {
    metrics::counter!("provider.operations.total", 1, "type" => provider_type, "operation" => operation);
    metrics::histogram!("provider.operations.duration", duration.as_secs_f64(), "type" => provider_type, "operation" => operation);
    
    if !success {
        metrics::counter!("provider.operations.errors", 1, "type" => provider_type, "operation" => operation);
    }
}
```

## Testing Strategy

1. **Interface Testing**: Create tests that work against the interface, not implementations
2. **Mock Providers**: Implement mock providers for testing
3. **Integration Testing**: Test with actual backend services
4. **Configuration Testing**: Test with various configuration combinations

Example:
```rust
pub struct MockProvider {
    name: String,
    operations: RwLock<HashMap<String, Result<Value>>>,
}

impl MockProvider {
    pub fn new(name: String) -> Self {
        Self {
            name,
            operations: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn mock_operation(&self, key: &str, result: Result<Value>) {
        let mut operations = self.operations.write().await;
        operations.insert(key.to_string(), result);
    }
}

#[async_trait]
impl Provider for MockProvider {
    // Implementation that returns mocked results
}
```

## Examples

### Database Provider Example

```rust
// Define the interface
#[async_trait]
pub trait DatabaseProvider: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn provider_type(&self) -> DatabaseProviderType;
    async fn connect(&self) -> Result<()>;
    async fn get_connection(&self) -> Result<Box<dyn DatabaseConnection>>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

// PostgreSQL implementation
pub struct PostgresProvider {
    name: String,
    config: PostgresConfig,
    pool: Mutex<Option<PgPool>>,
}

#[async_trait]
impl DatabaseProvider for PostgresProvider {
    // Implementation details
}

// Usage example
async fn example() -> Result<()> {
    // Create configuration
    let config = ProviderConfig {
        provider_type: DatabaseProviderType::Postgres,
        name: "main-db".to_string(),
        config: serde_json::to_value(PostgresConfig {
            url: "postgres://user:pass@localhost/dbname".to_string(),
            pool_size: 10,
            timeout: 30,
        })?,
    };
    
    // Create provider
    let provider = DatabaseProviderFactory::create(&config)?;
    
    // Connect and use
    provider.connect().await?;
    let conn = provider.get_connection().await?;
    
    // Execute operations
    let result = conn.execute("SELECT * FROM users").await?;
    
    Ok(())
}
```

### Cache Provider Example

```rust
// Define the interface
#[async_trait]
pub trait CacheProvider: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn provider_type(&self) -> CacheProviderType;
    async fn connect(&self) -> Result<()>;
    async fn get_connection(&self) -> Result<Box<dyn CacheConnection>>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

// Redis implementation
pub struct RedisProvider {
    name: String,
    config: RedisConfig,
    pool: Mutex<Option<RedisPool>>,
}

#[async_trait]
impl CacheProvider for RedisProvider {
    // Implementation details
}

// Usage example
async fn example() -> Result<()> {
    // Create configuration
    let config = ProviderConfig {
        provider_type: CacheProviderType::Redis,
        name: "main-cache".to_string(),
        config: serde_json::to_value(RedisConfig {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            timeout: 30,
        })?,
    };
    
    // Create provider
    let provider = CacheProviderFactory::create(&config)?;
    
    // Connect and use
    provider.connect().await?;
    let conn = provider.get_connection().await?;
    
    // Execute operations
    conn.set("key", "value", Some(Duration::from_secs(300))).await?;
    let value = conn.get("key").await?;
    
    Ok(())
}
```

## Conclusion

The provider pattern offers a flexible, extensible approach for Navius framework crates to support multiple backends and implementations while maintaining a consistent interface. By following the guidelines in this document, crates can implement the pattern consistently, making it easier for developers to understand and use the framework.

The pattern has been successfully implemented in the `navius-db` and `navius-cache` crates, providing a solid reference for other crates to follow. As the framework evolves, this guide will be updated with additional examples and best practices.

## References

1. Design Evaluation: navius-db
2. Design Evaluation: navius-cache
3. Cache Provider Guide
4. Database Provider Guide 
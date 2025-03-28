# Provider Architecture

This document outlines the provider architecture used throughout the Navius framework, focusing on the design patterns and implementation details that enable pluggable, interchangeable providers for various capabilities.

## Core Concepts

The provider architecture extends the service architecture with additional concepts:

1. **Provider Traits**: Define capabilities that can be provided by multiple implementations
2. **Provider Implementations**: Concrete implementations of provider traits
3. **Provider Registration**: Process for registering and discovering providers
4. **Provider Selection**: Runtime selection of appropriate providers
5. **Provider Configuration**: Configuration of provider behavior

## Provider vs. Service

While services and providers are similar concepts, they have some key differences:

| Aspect | Services | Providers |
|--------|----------|-----------|
| Focus | Business logic | Infrastructure capabilities |
| Cardinality | Typically one implementation active | Multiple implementations can be active |
| Selection | Fixed at application start | May be selected at runtime |
| Configuration | Application-level | Provider-specific |
| Examples | UserService, OrderService | CacheProvider, DatabaseProvider |

## Provider Organization

```
src/
├── core/
│   ├── providers/
│   │   ├── traits/          # Provider trait definitions
│   │   ├── implementations/  # Core provider implementations
│   │   └── registry.rs      # Provider registry
└── app/
    └── providers/
        └── implementations/  # Application-specific providers
```

## Provider Traits

Provider traits define capabilities that can be provided by multiple implementations:

```rust
// In src/core/providers/traits/cache.rs
pub trait CacheProvider: Send + Sync + 'static {
    fn create(&self, config: &CacheConfig) -> Result<Box<dyn CacheService>, ProviderError>;
    fn supports_type(&self, cache_type: &str) -> bool;
    fn name(&self) -> &'static str;
}
```

Key aspects of provider traits:
- They typically create or configure services
- They may support selection criteria (e.g., `supports_type`)
- They should include identification (e.g., `name`)
- They should include appropriate bounds for threading

## Provider Implementations

Provider implementations implement provider traits:

```rust
// In src/core/providers/implementations/redis_cache.rs
pub struct RedisCacheProvider;

impl RedisCacheProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CacheProvider for RedisCacheProvider {
    fn create(&self, config: &CacheConfig) -> Result<Box<dyn CacheService>, ProviderError> {
        let redis_config = config.redis.as_ref()
            .ok_or_else(|| ProviderError::Configuration("Redis configuration missing".into()))?;
            
        let client = redis::Client::open(redis_config.connection_string.as_str())?;
        let cache_service = RedisCacheService::new(client);
        
        Ok(Box::new(cache_service))
    }
    
    fn supports_type(&self, cache_type: &str) -> bool {
        cache_type.eq_ignore_ascii_case("redis")
    }
    
    fn name(&self) -> &'static str {
        "redis"
    }
}
```

Key aspects of provider implementations:
- They should implement the provider trait fully
- They should handle their own configuration
- They should create and configure the services they provide
- They should properly validate configuration

## Provider Registry

The provider registry manages the available providers:

```rust
// In src/core/providers/registry.rs
pub struct ProviderRegistry<P> {
    providers: Vec<Box<dyn P>>,
}

impl<P: 'static> ProviderRegistry<P> {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    pub fn register(&mut self, provider: Box<dyn P>) {
        self.providers.push(provider);
    }
    
    pub fn get_all(&self) -> &[Box<dyn P>] {
        &self.providers
    }
}

// Specialized implementation for CacheProvider
impl ProviderRegistry<CacheProvider> {
    pub fn find_by_type(&self, cache_type: &str) -> Option<&Box<dyn CacheProvider>> {
        self.providers.iter().find(|p| p.supports_type(cache_type))
    }
    
    pub fn find_by_name(&self, name: &str) -> Option<&Box<dyn CacheProvider>> {
        self.providers.iter().find(|p| p.name().eq_ignore_ascii_case(name))
    }
}
```

## Provider Registration

Providers are typically registered during application startup:

```rust
// In src/app/startup.rs
pub fn register_providers() -> (
    ProviderRegistry<CacheProvider>,
    ProviderRegistry<DatabaseProvider>,
) {
    // Register cache providers
    let mut cache_registry = ProviderRegistry::new();
    cache_registry.register(Box::new(MemoryCacheProvider::new()));
    cache_registry.register(Box::new(RedisCacheProvider::new()));
    
    // Register database providers
    let mut db_registry = ProviderRegistry::new();
    db_registry.register(Box::new(PostgresProvider::new()));
    db_registry.register(Box::new(SqliteProvider::new()));
    
    (cache_registry, db_registry)
}
```

## Provider Selection

Providers can be selected at runtime based on configuration:

```rust
// In src/app/startup.rs
pub fn create_services(
    config: &AppConfig,
    cache_registry: &ProviderRegistry<CacheProvider>,
    db_registry: &ProviderRegistry<DatabaseProvider>,
) -> Result<ServiceRegistry, ProviderError> {
    let mut service_registry = ServiceRegistry::new();
    
    // Select and create database service
    let db_provider = db_registry.find_by_type(&config.database.provider_type)
        .ok_or_else(|| ProviderError::ProviderNotFound(
            format!("Database provider '{}' not found", config.database.provider_type)
        ))?;
        
    let db_service = db_provider.create(&config.database)?;
    service_registry.register::<dyn DatabaseService>(db_service);
    
    // Select and create cache service
    let cache_provider = cache_registry.find_by_type(&config.cache.provider_type)
        .ok_or_else(|| ProviderError::ProviderNotFound(
            format!("Cache provider '{}' not found", config.cache.provider_type)
        ))?;
        
    let cache_service = cache_provider.create(&config.cache)?;
    service_registry.register::<dyn CacheService>(cache_service);
    
    Ok(service_registry)
}
```

## Provider Configuration

Providers typically have their own specific configuration sections:

```yaml
# In config/default.yaml
database:
  provider_type: postgres
  postgres:
    host: localhost
    port: 5432
    username: postgres
    password: password
    database: myapp
  sqlite:
    path: ./data/myapp.db

cache:
  provider_type: redis
  redis:
    connection_string: redis://localhost:6379
  memory:
    max_entries: 10000
    ttl_seconds: 3600
```

## Provider Error Handling

Providers should use a specific error type for provider-related errors:

```rust
// In src/core/providers/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Provider configuration error: {0}")]
    Configuration(String),
    
    #[error("Provider initialization error: {0}")]
    Initialization(String),
    
    #[error("Provider error: {0}")]
    Other(#[from] anyhow::Error),
}
```

## Multiple Active Providers

In some cases, multiple providers of the same type may be active simultaneously:

```rust
// In src/core/services/implementations/multi_cache.rs
pub struct MultiCacheService {
    caches: Vec<Box<dyn CacheService>>,
}

impl MultiCacheService {
    pub fn new(caches: Vec<Box<dyn CacheService>>) -> Self {
        Self { caches }
    }
    
    pub fn from_providers(
        providers: &[Box<dyn CacheProvider>],
        config: &AppConfig,
    ) -> Result<Self, ProviderError> {
        let mut caches = Vec::new();
        
        for provider in providers {
            let cache = provider.create(&config.cache)?;
            caches.push(cache);
        }
        
        Ok(Self { caches })
    }
}

impl CacheService for MultiCacheService {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        for cache in &self.caches {
            if let Some(value) = cache.get(key).await? {
                return Ok(Some(value));
            }
        }
        
        Ok(None)
    }
    
    // Other method implementations...
}
```

## Provider Discovery

In more advanced scenarios, providers can be discovered dynamically:

```rust
// In src/core/providers/discovery.rs
pub fn discover_providers<P: Provider + 'static>() -> ProviderRegistry<P> {
    let mut registry = ProviderRegistry::new();
    
    // Look for providers in a plugins directory
    let plugins_dir = std::path::Path::new("plugins");
    if plugins_dir.exists() {
        for entry in std::fs::read_dir(plugins_dir).unwrap_or_else(|_| return std::fs::ReadDir::empty()) {
            if let Ok(entry) = entry {
                if let Some(provider) = load_provider::<P>(&entry.path()) {
                    registry.register(provider);
                }
            }
        }
    }
    
    registry
}

fn load_provider<P: Provider + 'static>(path: &std::path::Path) -> Option<Box<dyn P>> {
    // Load dynamic library and look for provider factory
    // This is simplified; actual implementation would use libloading
    None
}
```

## Best Practices

### Clearly Separate Provider and Service Concerns

Providers should focus on creating and configuring services, not implementing service functionality directly.

### Support Multiple Provider Types

Applications often need different provider implementations for different environments (development, testing, production).

### Make Provider Selection Configurable

Allow the application to select providers through configuration rather than hardcoding.

### Document Provider Requirements

Clearly document what each provider needs in terms of configuration and external dependencies.

### Test with Multiple Providers

Test the application with each supported provider to ensure consistent behavior.

### Handle Missing Providers Gracefully

The application should provide clear error messages when a requested provider is not available.

## Related Documents

- [Service Architecture](service-architecture.md)
- [Configuration](../guides/configuration.md)
- [Feature Selection](../guides/feature-selection.md) 
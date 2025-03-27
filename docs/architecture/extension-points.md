# Extension Points

This document outlines the extension points available in the Navius framework, which allow for customization and extension of the framework's behavior without modifying its core.

## What Are Extension Points?

Extension points are well-defined interfaces in the Navius framework that allow applications to:

1. **Extend** the framework with custom functionality
2. **Customize** existing behavior
3. **Replace** default implementations with application-specific ones
4. **Integrate** with third-party libraries and systems

Extension points are critical for maintaining a clean separation between framework code and application-specific code.

## Types of Extension Points

Navius provides several types of extension points:

1. **Trait-based Extensions**: Implementing traits to extend functionality
2. **Service Registration**: Registering custom services
3. **Provider Registration**: Registering custom providers
4. **Middleware Extensions**: Adding custom middleware
5. **Event Handlers**: Subscribing to framework events
6. **Configuration Extensions**: Extending configuration

## Trait-based Extensions

The most common extension mechanism in Navius is implementing traits:

```rust
// Create a custom health check by implementing the HealthCheck trait
pub struct DatabaseHealthCheck {
    db_pool: PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &'static str {
        "database"
    }
    
    async fn check(&self) -> HealthStatus {
        match self.db_pool.acquire().await {
            Ok(_) => HealthStatus::up(),
            Err(e) => HealthStatus::down().with_details("Failed to connect to database", e),
        }
    }
}

// Register the custom health check
app.register_health_check(Box::new(DatabaseHealthCheck::new(db_pool)));
```

Common trait-based extension points include:

- `HealthCheck`: Custom health checks
- `AuthenticationProvider`: Custom authentication mechanisms
- `LoggingAdapter`: Custom logging integrations
- `CacheService`: Custom cache implementations
- `EventHandler`: Custom event processing

## Service Registration

Custom services can be registered with the service registry:

```rust
// Define a custom service
pub struct EmailService {
    config: EmailConfig,
    client: reqwest::Client,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError> {
        // Implementation...
        Ok(())
    }
}

// Register the service
let mut registry = ServiceRegistry::new();
let email_service = EmailService::new(config.email.clone());
registry.register::<EmailService>(email_service);

// Use the service later
let email_service = registry.get::<EmailService>()
    .expect("Email service not registered");
    
email_service.send_email("user@example.com", "Hello", "World").await?;
```

## Provider Registration

Custom providers can be registered to create services:

```rust
// Define a custom cache provider
pub struct CloudCacheProvider;

impl CloudCacheProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CacheProvider for CloudCacheProvider {
    fn create(&self, config: &CacheConfig) -> Result<Box<dyn CacheService>, ProviderError> {
        let cloud_config = config.cloud.as_ref()
            .ok_or_else(|| ProviderError::Configuration("Cloud cache configuration missing".into()))?;
            
        let client = CloudCacheClient::new(&cloud_config.connection_string)?;
        let cache_service = CloudCacheService::new(client);
        
        Ok(Box::new(cache_service))
    }
    
    fn supports_type(&self, cache_type: &str) -> bool {
        cache_type.eq_ignore_ascii_case("cloud")
    }
    
    fn name(&self) -> &'static str {
        "cloud-cache"
    }
}

// Register the provider
let mut cache_registry = ProviderRegistry::new();
cache_registry.register(Box::new(CloudCacheProvider::new()));
```

## Middleware Extensions

Custom middleware can be added to the HTTP pipeline:

```rust
// Define custom middleware
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u64) -> Self {
        let limiter = Arc::new(RateLimiter::new(requests_per_minute));
        Self { limiter }
    }
}

impl<S> Layer<S> for RateLimitMiddleware {
    type Service = RateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimitService {
            inner: service,
            limiter: self.limiter.clone(),
        }
    }
}

// Register the middleware
let app = Router::new()
    .route("/api/users", get(list_users))
    .layer(RateLimitMiddleware::new(60));
```

## Event Handlers

Custom event handlers can be registered to respond to framework events:

```rust
// Define a custom event handler
pub struct AuditEventHandler {
    db_pool: PgPool,
}

impl AuditEventHandler {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl EventHandler for AuditEventHandler {
    async fn handle(&self, event: &Event) -> Result<(), EventError> {
        match event {
            Event::UserAuthenticated { user_id, ip_address, timestamp } => {
                sqlx::query!(
                    "INSERT INTO audit_log (event_type, user_id, ip_address, timestamp) VALUES ($1, $2, $3, $4)",
                    "user_authenticated",
                    user_id,
                    ip_address,
                    timestamp
                )
                .execute(&self.db_pool)
                .await?;
            },
            // Handle other events...
            _ => {},
        }
        
        Ok(())
    }
    
    fn supports_event(&self, event_type: &str) -> bool {
        matches!(event_type, "user_authenticated" | "user_created" | "user_deleted")
    }
}

// Register the event handler
let mut event_bus = EventBus::new();
event_bus.register_handler(Box::new(AuditEventHandler::new(db_pool)));
```

## Configuration Extensions

The configuration system can be extended with custom sections:

```rust
// Define a custom configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
}

// Extend the application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // Standard configuration...
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    
    // Custom configuration...
    pub twilio: TwilioConfig,
}

// Use the custom configuration
let config = ConfigBuilder::new()
    .add_file("config/default.toml")
    .build::<AppConfig>()?;
    
let twilio_service = TwilioService::new(config.twilio);
```

## Extension Point Best Practices

### Make Extension Points Explicit

Clearly document which parts of the framework are intended for extension. Use traits with well-defined methods rather than relying on inheriting from concrete classes.

### Follow the Principle of Least Surprise

Extension points should behave in predictable ways. Avoid hidden behaviors or side effects that might surprise developers using the extension point.

### Use Composition Over Inheritance

Favor composition patterns (like middleware) over inheritance hierarchies for extensions. This provides more flexibility and avoids many common inheritance problems.

### Provide Sensible Defaults

Every extension point should have a reasonable default implementation. Users should only need to implement custom extensions when they want to change the default behavior.

### Document Extension Requirements

Clearly document what is required to implement an extension point, including:
- Required methods and their semantics
- Threading and lifetime requirements
- Error handling expectations
- Performance considerations

### Test Extensions Thoroughly

Provide testing utilities and examples to help users test their extensions. Extension points should be designed with testability in mind.

## Core Extension Points Reference

### HealthCheck Trait

```rust
pub trait HealthCheck: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    async fn check(&self) -> HealthStatus;
}
```

### CacheService Trait

```rust
pub trait CacheService: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;
}
```

### DatabaseService Trait

```rust
pub trait DatabaseService: Send + Sync + 'static {
    async fn execute(&self, query: &str, params: &[Value]) -> Result<u64, DatabaseError>;
    async fn query_one(&self, query: &str, params: &[Value]) -> Result<Row, DatabaseError>;
    async fn query_all(&self, query: &str, params: &[Value]) -> Result<Vec<Row>, DatabaseError>;
    async fn transaction<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&dyn Transaction) -> Future<Output = Result<R, DatabaseError>> + Send,
        R: Send + 'static;
}
```

### AuthenticationProvider Trait

```rust
pub trait AuthenticationProvider: Send + Sync + 'static {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Option<User>, AuthError>;
    async fn validate_token(&self, token: &str) -> Result<Option<User>, AuthError>;
    async fn refresh_token(&self, token: &str) -> Result<Option<String>, AuthError>;
}
```

### EventHandler Trait

```rust
pub trait EventHandler: Send + Sync + 'static {
    async fn handle(&self, event: &Event) -> Result<(), EventError>;
    fn supports_event(&self, event_type: &str) -> bool;
}
```

## Related Documents

- [Service Architecture](service-architecture.md)
- [Provider Architecture](provider-architecture.md)
- [Dependency Injection](../guides/dependency-injection.md)
- [Middleware](../guides/middleware.md) 
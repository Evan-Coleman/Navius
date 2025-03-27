---
title: "Application API Reference"
description: "Comprehensive reference for the Navius Application API"
category: api
tags:
  - api
  - application
  - framework
related:
  - reference/api/configuration-api.md
  - examples/basic-application-example.md
last_updated: March 26, 2024
version: 1.0
---

# Application API Reference

## Overview

The Application API in Navius provides the core interfaces and structures for building applications. It offers a standardized approach to application lifecycle, dependency injection, routing, middleware, and error handling.

## Core Components

### Application

The `Application` struct represents a Navius application:

```rust
pub struct Application {
    name: String,
    router: Router,
    service_registry: Arc<ServiceRegistry>,
    config: Arc<dyn ConfigService>,
    state: AppState,
}

impl Application {
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn config(&self) -> &Arc<dyn ConfigService> {
        &self.config
    }
    
    pub fn router(&self) -> &Router {
        &self.router
    }
    
    pub fn service_registry(&self) -> &Arc<ServiceRegistry> {
        &self.service_registry
    }
    
    pub fn state(&self) -> &AppState {
        &self.state
    }
    
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}
```

### Application Builder

The `ApplicationBuilder` provides a fluent API for configuring and building a Navius application:

```rust
pub struct ApplicationBuilder {
    name: String,
    router_config: RouterConfig,
    service_registry: Arc<ServiceRegistry>,
    config: Option<Arc<dyn ConfigService>>,
    state: AppState,
    startup_hooks: Vec<Box<dyn ApplicationHook>>,
    shutdown_hooks: Vec<Box<dyn ApplicationHook>>,
}

impl ApplicationBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        // Initialize with defaults
    }
    
    pub fn with_config(mut self, config: Arc<dyn ConfigService>) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_router_config(mut self, config: RouterConfig) -> Self {
        self.router_config = config;
        self
    }
    
    pub fn with_service<S: 'static + Send + Sync>(mut self, service: S) -> Result<Self, AppError> {
        self.service_registry.register(service)?;
        Ok(self)
    }
    
    pub fn with_startup_hook(mut self, hook: Box<dyn ApplicationHook>) -> Self {
        self.startup_hooks.push(hook);
        self
    }
    
    pub fn with_shutdown_hook(mut self, hook: Box<dyn ApplicationHook>) -> Self {
        self.shutdown_hooks.push(hook);
        self
    }
    
    pub fn build(self) -> Result<Application, AppError> {
        // Build the application with all configured components
    }
}
```

### Application Hook

The `ApplicationHook` trait defines hooks that are called during the application lifecycle:

```rust
pub trait ApplicationHook: Send + Sync {
    fn execute(&self, app: &mut Application) -> Result<(), AppError>;
}
```

### AppState

The `AppState` struct holds application-wide state:

```rust
pub struct AppState {
    values: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn insert<T: 'static + Send + Sync>(&self, value: T) -> Result<(), AppError> {
        let mut values = self.values.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock on app state")
        })?;
        
        let type_id = TypeId::of::<T>();
        values.insert(type_id, Box::new(value));
        
        Ok(())
    }
    
    pub fn get<T: 'static + Clone + Send + Sync>(&self) -> Result<T, AppError> {
        let values = self.values.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock on app state")
        })?;
        
        let type_id = TypeId::of::<T>();
        
        match values.get(&type_id) {
            Some(value) => {
                if let Some(value_ref) = value.downcast_ref::<T>() {
                    Ok(value_ref.clone())
                } else {
                    Err(AppError::internal_server_error(
                        format!("Value of type {:?} exists but could not be downcast", type_id)
                    ))
                }
            },
            None => Err(AppError::not_found(
                format!("No value of type {:?} found in app state", type_id)
            )),
        }
    }
}
```

## Application Lifecycle

### Building an Application

```rust
// Create an application builder
let builder = ApplicationBuilder::new("my-app")
    .with_config(config_service.clone())?
    .with_service(database_service.clone())?
    .with_service(cache_service.clone())?
    .with_startup_hook(Box::new(DatabaseMigrationHook::new()))?
    .with_shutdown_hook(Box::new(ResourceCleanupHook::new()))?;

// Build the application
let app = builder.build()?;
```

### Starting an Application

```rust
// Create and build an application
let app = ApplicationBuilder::new("my-app")
    // Configure the application
    .build()?;

// Run the application
app.run().await?;
```

### Graceful Shutdown

```rust
// Create a shutdown signal handler
let shutdown_signal = async {
    match signal::ctrl_c().await {
        Ok(()) => {
            log::info!("Shutdown signal received, starting graceful shutdown");
        },
        Err(err) => {
            log::error!("Error listening for shutdown signal: {}", err);
        }
    }
};

// Run the application with shutdown signal
app.run_until(shutdown_signal).await?;
```

## Routing

### Defining Routes

```rust
// Create a router
let router = Router::new()
    .route("/", get(index_handler))
    .route("/users", get(get_users).post(create_user))
    .route("/users/:id", get(get_user_by_id))
    .nest("/api/v1", api_v1_router())
    .layer(middleware::from_fn(auth_middleware));

// Create an application with the router
let app = ApplicationBuilder::new("my-app")
    .with_router(router)
    .build()?;
```

### Route Groups

```rust
// Create a grouped router
let api_router = Router::new()
    .route("/users", get(get_users).post(create_user))
    .route("/users/:id", get(get_user_by_id).put(update_user).delete(delete_user))
    .route("/health", get(health_check));

// Add authentication middleware to the group
let authenticated_api = api_router.layer(middleware::from_fn(auth_middleware));

// Add the grouped router to the main router
let router = Router::new()
    .nest("/api/v1", authenticated_api)
    .route("/", get(index_handler));
```

### Route Parameters

```rust
// Handler function with route parameter
async fn get_user_by_id(
    Path(id): Path<String>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<User>, AppError> {
    let user_service = registry.get::<UserService>()?;
    let user = user_service.get_user(&id).await?;
    Ok(Json(user))
}
```

### Query Parameters

```rust
#[derive(Debug, Deserialize)]
struct UserQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
}

async fn get_users(
    Query(query): Query<UserQuery>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<Vec<User>>, AppError> {
    let user_service = registry.get::<UserService>()?;
    
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    let sort_by = query.sort_by.unwrap_or_else(|| "created_at".to_string());
    
    let users = user_service.list_users(limit, offset, &sort_by).await?;
    Ok(Json(users))
}
```

## Request Handling

### Request Extractors

Navius provides several extractors for handling requests:

| Extractor | Description | Example |
|-----------|-------------|---------|
| `State<T>` | Extract shared state | `State(registry): State<Arc<ServiceRegistry>>` |
| `Path<T>` | Extract path parameters | `Path(id): Path<String>` |
| `Query<T>` | Extract query parameters | `Query(params): Query<Pagination>` |
| `Json<T>` | Extract JSON request body | `Json(user): Json<CreateUser>` |
| `Form<T>` | Extract form data | `Form(login): Form<LoginForm>` |
| `Extension<T>` | Extract request extensions | `Extension(user): Extension<CurrentUser>` |

### Handler Function

```rust
async fn create_user(
    Json(user_data): Json<CreateUserRequest>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user_service = registry.get::<UserService>()?;
    
    // Validate input
    user_data.validate()?;
    
    // Create user
    let user = user_service.create_user(user_data.into()).await?;
    
    // Return created user with 201 status
    Ok((StatusCode::CREATED, Json(user)))
}
```

### Service Access

```rust
async fn process_order(
    Json(order_data): Json<CreateOrderRequest>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<Order>, AppError> {
    // Get services from registry
    let order_service = registry.get::<OrderService>()?;
    let payment_service = registry.get::<PaymentService>()?;
    let notification_service = registry.get::<NotificationService>()?;
    
    // Process the order
    let order = order_service.create_order(order_data).await?;
    
    // Process payment
    let payment = payment_service.process_payment(&order).await?;
    
    // Update order with payment information
    let updated_order = order_service.update_order_payment(&order.id, &payment.id).await?;
    
    // Send notification
    notification_service.send_order_confirmation(&updated_order).await?;
    
    Ok(Json(updated_order))
}
```

## Middleware

### Creating Middleware

```rust
async fn logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let response = next.run(req).await?;
    
    let duration = start.elapsed();
    log::info!("{} {} - {} - {:?}", method, uri, response.status(), duration);
    
    Ok(response)
}
```

### Authentication Middleware

```rust
async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract authorization header
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Missing authorization header"))?;
    
    // Validate token
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::unauthorized("Invalid authorization format"));
    }
    
    let token = &auth_header["Bearer ".len()..];
    
    // Get auth service from request extensions
    let registry = req.extensions()
        .get::<Arc<ServiceRegistry>>()
        .ok_or_else(|| AppError::internal_server_error("Service registry not found"))?;
    
    let auth_service = registry.get::<AuthService>()?;
    
    // Verify token
    let user = auth_service.verify_token(token).await?;
    
    // Add user to request extensions
    let mut req = req;
    req.extensions_mut().insert(user);
    
    // Continue with the request
    next.run(req).await
}
```

### CORS Middleware

```rust
fn configure_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(["https://example.com".parse::<HeaderValue>().unwrap()])
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

// Add to router
let router = Router::new()
    .route("/users", get(get_users))
    .layer(configure_cors());
```

### Rate Limiting Middleware

```rust
fn configure_rate_limit() -> RateLimitLayer {
    RateLimitLayer::new(
        100, // requests
        Duration::from_secs(60), // per minute
    )
}

// Add to router
let router = Router::new()
    .route("/api/v1/users", get(get_users))
    .layer(configure_rate_limit());
```

## Error Handling

### AppError

The `AppError` struct represents application errors:

```rust
pub struct AppError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AppError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            details: None,
            source: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }
    
    // Convenience methods for common errors
    
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", message)
    }
    
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message)
    }
    
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "FORBIDDEN", message)
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message)
    }
    
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR", message)
    }
}
```

### Error Response Format

```json
{
  "status": 400,
  "code": "VALIDATION_ERROR",
  "message": "Invalid request data",
  "details": {
    "errors": [
      {
        "field": "email",
        "message": "Invalid email format"
      },
      {
        "field": "password",
        "message": "Password must be at least 8 characters"
      }
    ]
  }
}
```

### Error Handler Middleware

```rust
async fn error_handler(err: AppError) -> Response {
    let status = err.status;
    
    let body = serde_json::json!({
        "status": status.as_u16(),
        "code": err.code,
        "message": err.message,
        "details": err.details,
    });
    
    // Log the error
    if status.is_server_error() {
        log::error!("Server error: {:?}", err);
        if let Some(source) = &err.source {
            log::error!("Caused by: {:?}", source);
        }
    } else {
        log::debug!("Client error: {:?}", err);
    }
    
    // Create response
    let mut response = Response::new(Body::from(serde_json::to_vec(&body).unwrap_or_default()));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    
    response
}
```

### Validation Error Handling

```rust
#[derive(Debug, Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
    
    #[validate(length(min = 1, max = 100))]
    name: String,
}

impl CreateUserRequest {
    fn validate(&self) -> Result<(), AppError> {
        match <Self as Validate>::validate(self) {
            Ok(_) => Ok(()),
            Err(validation_errors) => {
                let mut errors = Vec::new();
                
                for (field, field_errors) in validation_errors.field_errors() {
                    for error in field_errors {
                        errors.push(serde_json::json!({
                            "field": field,
                            "message": error.message.clone().unwrap_or_else(|| "Invalid value".into()),
                        }));
                    }
                }
                
                Err(AppError::bad_request("Validation failed")
                    .with_details(serde_json::json!({ "errors": errors })))
            }
        }
    }
}
```

## Configuration

### Application Configuration

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub environment: String,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout: Option<u64>,
}

// Load configuration
let config = ConfigManager::new(vec![
    Box::new(FileConfigLoader::new("config")?),
    Box::new(EnvConfigLoader::new("APP_")?),
])
.load()?;

// Create application with configuration
let app = ApplicationBuilder::new("my-app")
    .with_config(Arc::new(config))
    .build()?;
```

### Loading Configuration from Files

```yaml
# config/default.yaml
name: "my-app"
environment: "development"
server:
  host: "127.0.0.1"
  port: 8080
  request_timeout: 30
logging:
  level: "debug"
  format: "json"
database:
  url: "postgres://localhost:5432/myapp"
  max_connections: 10
```

## State Management

### Application State

```rust
// Define application state
pub struct AppMetrics {
    pub request_count: AtomicU64,
    pub error_count: AtomicU64,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
    
    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::SeqCst)
    }
    
    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::SeqCst)
    }
}

// Add state to application
let metrics = AppMetrics::new();
let app = ApplicationBuilder::new("my-app")
    .with_state(metrics)?
    .build()?;
```

### Accessing State in Handlers

```rust
async fn metrics_handler(
    State(app): State<Arc<Application>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let metrics = app.state().get::<AppMetrics>()?;
    
    let response = serde_json::json!({
        "request_count": metrics.get_request_count(),
        "error_count": metrics.get_error_count(),
    });
    
    Ok(Json(response))
}
```

## Request Validation

### Request Validation Middleware

```rust
async fn validate_request<T, B>(req: Request<B>, next: Next<B>) -> Result<Response, AppError>
where
    T: DeserializeOwned + Validate,
    B: Body + Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    B::Data: Send,
{
    // Extract and validate the body
    let (parts, body) = req.into_parts();
    let bytes = hyper::body::to_bytes(body).await.map_err(|err| {
        AppError::bad_request("Failed to read request body").with_source(err)
    })?;
    
    let value: T = serde_json::from_slice(&bytes).map_err(|err| {
        AppError::bad_request("Invalid JSON").with_source(err)
    })?;
    
    // Validate the request
    if let Err(validation_errors) = value.validate() {
        let mut errors = Vec::new();
        
        for (field, field_errors) in validation_errors.field_errors() {
            for error in field_errors {
                errors.push(serde_json::json!({
                    "field": field,
                    "message": error.message.clone().unwrap_or_else(|| "Invalid value".into()),
                }));
            }
        }
        
        return Err(AppError::bad_request("Validation failed")
            .with_details(serde_json::json!({ "errors": errors })));
    }
    
    // Reconstruct the request
    let body = Body::from(bytes);
    let req = Request::from_parts(parts, body);
    
    // Continue with the request
    next.run(req).await
}
```

## Health Checks

### Health Check Handler

```rust
async fn health_check(
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let health_service = registry.get::<HealthService>()?;
    
    // Perform health check
    let health_status = health_service.check().await?;
    
    // Return health status
    Ok(Json(health_status))
}
```

### Health Service

```rust
pub struct HealthService {
    checkers: Vec<Box<dyn HealthChecker>>,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
        }
    }
    
    pub fn add_checker(&mut self, checker: Box<dyn HealthChecker>) {
        self.checkers.push(checker);
    }
    
    pub async fn check(&self) -> Result<serde_json::Value, AppError> {
        let mut status = "UP";
        let mut components = HashMap::new();
        
        for checker in &self.checkers {
            let result = checker.check().await;
            
            let component_status = match &result {
                Ok(_) => "UP",
                Err(_) => {
                    status = "DOWN";
                    "DOWN"
                }
            };
            
            let details = match result {
                Ok(details) => details,
                Err(err) => {
                    serde_json::json!({
                        "error": err.to_string()
                    })
                }
            };
            
            components.insert(checker.name(), serde_json::json!({
                "status": component_status,
                "details": details
            }));
        }
        
        Ok(serde_json::json!({
            "status": status,
            "components": components,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
```

## Related Examples

- [Basic Application Example](../../examples/basic-application-example.md)
- [Configuration Example](../../examples/configuration-example.md)
- [Custom Service Example](../../examples/custom-service-example.md) 
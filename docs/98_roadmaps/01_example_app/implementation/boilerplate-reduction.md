# Boilerplate Reduction: Before and After

This document demonstrates the dramatic reduction in user code that will be achieved through the Zero Boilerplate Initiative.

## Application Setup

### Before

```rust
// main.rs
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create API router
    let api_router = Router::new()
        .route("/hello", get(hello_world))
        .route("/context", get(context_info));

    // Create Web Plugin with custom settings
    let web = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(3000)
        .with_router(
            Router::new().nest("/api/v1", api_router)
        );

    // Build and run application with plugin
    let app = App::new()
        .register_plugin(web)
        .build();

    app.run().await;
}
```

### After

```rust
#[navius_app]
async fn main() {
    // That's it! All configuration is handled via convention
    // and/or configuration files
}

// Handlers are defined in dedicated handler files for proper separation of concerns
// src/handlers/api_handler.rs
pub async fn hello_world() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, world!" }))
}

// Routes reference these handlers using attribute macros
// src/main.rs
#[nest(prefix = "/api/v1")]
mod api {
    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        api_handler::hello_world().await
    }
}

#[route(GET, "/api/v1/context")]
async fn context_info(#[inject] app: &App) -> impl IntoResponse {
    format!("Component types: {:?}", app.registry().component_types())
}
```

## Web Plugin Configuration

### Before

```rust
// web_plugin.rs
pub struct WebPlugin {
    base: PluginBase,
    host: String,
    port: u16,
    router: Option<Router>,
    lifecycle_stage: LifecycleStage,
}

impl WebPlugin {
    pub fn new() -> Self {
        Self {
            base: PluginBase::new("WebPlugin"),
            host: "0.0.0.0".to_string(),
            port: 8080,
            router: None,
            lifecycle_stage: LifecycleStage::Created,
        }
    }

    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    async fn start_server(&self) -> Result<(), anyhow::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let addr = addr.parse()?;

        if let Some(router) = &self.router {
            let app = router.clone();
            tracing::info!("Starting web server on {}", addr);
            tokio::task::spawn(async move {
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            });
        } else {
            tracing::warn!("No router provided, server will not be started");
        }

        Ok(())
    }
}

impl PluginLifecycle for WebPlugin {
    fn initialize(&mut self, app: &App) -> Result<(), anyhow::Error> {
        self.lifecycle_stage = LifecycleStage::Initialized;
        Ok(())
    }

    fn start(&mut self, app: &App) -> Result<(), anyhow::Error> {
        self.lifecycle_stage = LifecycleStage::Starting;
        tokio::runtime::Handle::current().block_on(async {
            self.start_server().await
        })?;
        self.lifecycle_stage = LifecycleStage::Running;
        Ok(())
    }

    fn stop(&mut self, app: &App) -> Result<(), anyhow::Error> {
        self.lifecycle_stage = LifecycleStage::Stopping;
        // Shutdown logic would go here
        self.lifecycle_stage = LifecycleStage::Stopped;
        Ok(())
    }

    fn health_check(&self) -> Result<bool, anyhow::Error> {
        Ok(self.lifecycle_stage == LifecycleStage::Running)
    }
}
```

### After

```rust
// No user code required for standard web plugin configuration!
// Everything is configured via configuration files or environment variables

// For custom configuration, a minimal setup is possible:
#[navius_web_config]
fn configure_web(builder: WebBuilder) -> WebBuilder {
    builder
        .with_cors(
            CorsOptions::default()
                .allow_origin("https://example.com")
                .allow_methods(vec!["GET", "POST"])
        )
        .with_middleware(custom_middleware)
}
```

## Dependency Injection

### Before

```rust
// Handlers need to use State extraction explicitly
async fn user_service(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_repo = state.app
        .registry()
        .resolve::<Box<dyn UserRepository>>()
        .expect("Failed to resolve UserRepository");
    
    // Use user_repo
}

// Service registration is verbose
fn register_services(app: &mut App) {
    app.registry_mut().register::<Box<dyn UserRepository>>(
        Box::new(UserRepositoryImpl::new())
    );
    app.registry_mut().register::<Box<dyn AuthService>>(
        Box::new(AuthServiceImpl::new())
    );
}
```

### After

```rust
// Dependency injection via attribute macro
#[route(GET, "/api/v1/users")]
async fn user_service(
    #[inject] user_repo: &dyn UserRepository,
    #[inject] auth: &dyn AuthService
) -> impl IntoResponse {
    // Use injected dependencies directly
}

// Service registration via attributes
#[service]
impl UserRepository for UserRepositoryImpl {
    // Implementation
}

#[service]
impl AuthService for AuthServiceImpl {
    // Implementation
}
```

## Route Definition

### Before

```rust
// Router creation is manual and verbose
let router = Router::new()
    .route("/users", get(list_users))
    .route("/users/:id", get(get_user))
    .route("/users", post(create_user))
    .route("/users/:id", put(update_user))
    .route("/users/:id", delete(delete_user));
```

### After

```rust
// Routes are defined directly on handler functions
#[route(GET, "/users")]
async fn list_users() -> impl IntoResponse { /* ... */ }

#[route(GET, "/users/:id")]
async fn get_user(#[param] id: String) -> impl IntoResponse { /* ... */ }

#[route(POST, "/users")]
async fn create_user(#[body] user: User) -> impl IntoResponse { /* ... */ }

#[route(PUT, "/users/:id")]
async fn update_user(
    #[param] id: String,
    #[body] user: User
) -> impl IntoResponse { /* ... */ }

#[route(DELETE, "/users/:id")]
async fn delete_user(#[param] id: String) -> impl IntoResponse { /* ... */ }
```

## Configuration Management

### Before

```rust
// Manual configuration loading and validation
fn load_config() -> Config {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.toml".to_string());
    
    let config_str = std::fs::read_to_string(config_path)
        .expect("Failed to read config file");
    
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config");
    
    config.validate().expect("Invalid configuration");
    
    config
}

// Manual environment variable overrides
fn override_with_env(config: &mut Config) {
    if let Ok(host) = std::env::var("WEB_HOST") {
        config.web.host = host;
    }
    
    if let Ok(port_str) = std::env::var("WEB_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            config.web.port = port;
        }
    }
    
    // Many more overrides...
}
```

### After

```rust
// Automatic configuration loading with validation
#[derive(Deserialize, Config)]
struct AppConfig {
    web: WebConfig,
    database: DatabaseConfig,
    // Other configuration sections
}

// Configuration is automatically:
// 1. Loaded from standard locations
// 2. Overridden by environment variables
// 3. Validated before use
// 4. Available for injection

#[route(GET, "/config")]
async fn show_config(#[inject] config: &AppConfig) -> impl IntoResponse {
    format!("Web port: {}", config.web.port)
}
```

## Summary of Improvements

| Feature | Before (LOC) | After (LOC) | Reduction |
|---------|--------------|-------------|-----------|
| Main application setup | 20+ | 3 | 85% |
| Web plugin configuration | 70+ | 0-10 | 85-100% |
| Route definition | 15-30 | 5-10 | 66% |
| Dependency injection | 20+ | 3-5 | 75-85% |
| Configuration management | 40+ | 5-10 | 75-87% |
| **Total** | **165+** | **16-28** | **83-90%** |

These improvements will dramatically reduce the amount of code needed to build Navius applications, allowing developers to focus on business logic rather than plumbing code. 
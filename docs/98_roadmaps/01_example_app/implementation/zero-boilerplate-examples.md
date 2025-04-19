# Zero Boilerplate Examples

This document contains concrete examples of boilerplate code that will be eliminated through the Zero Boilerplate Initiative.

## Current Boilerplate in Example App

### 1. Main Entry Point

Currently, the `main.rs` file contains significant setup code that should be handled by the framework:

```rust
// src/main.rs (current state)
#[tokio::main]
async fn main() -> navius_core::error::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Load configuration
    let configurator = WebConfigurator::new();
    let config = configurator.load_config().map_err(|e| {
        navius_core::error::Error::internal(format!("Failed to load configuration: {}", e))
    })?;

    // Create API router
    let api_router = Router::new().route("/hello", get(handlers::api::hello_world));

    // Get server host and port from config
    let host = config
        .get_string("server.host")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = config.get_int("server.port").unwrap_or(3000) as u16;

    // Create a web plugin with settings from config
    let web_plugin = plugins::WebPlugin::new()
        .with_port(port)
        .with_host(host)
        .with_router(Router::new().nest("/api/v1", api_router));

    // Build and run our application
    App::new().add_web_plugin(web_plugin).run().await
}
```

### 2. WebPlugin Implementation

The `WebPlugin` contains complex server setup logic that shouldn't be user's responsibility:

```rust
// src/plugins/web_plugin.rs (partial)
impl WebPlugin {
    pub fn new() -> Self {
        Self {
            base: Arc::new(BasePlugin::new()),
            host: "127.0.0.1".to_string(),
            port: 3000,
            router: Router::new(),
            lifecycle_stage: LifecycleStage::Created,
        }
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    async fn start_server(&self) -> Result<(), Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let socket_addr = addr.parse().map_err(|e| {
            Error::internal(format!("Failed to parse server address: {}", e))
        })?;

        tracing::info!("Starting web server on {}", addr);

        let app = self.router.clone();

        tokio::spawn(async move {
            axum::Server::bind(&socket_addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        Ok(())
    }
}
```

### 3. Configuration Boilerplate

Configuration management includes significant boilerplate:

```rust
// src/config/web_configurator.rs (partial)
pub struct WebConfigurator {
    pub sources: Vec<Box<dyn ConfigSource>>,
}

impl WebConfigurator {
    pub fn new() -> Self {
        let env_source = Box::new(EnvConfigSource::new("NAVIUS_"));
        let file_source = Box::new(FileConfigSource::new("config/default"));

        Self {
            sources: vec![file_source, env_source],
        }
    }

    pub fn load_config(&self) -> Result<Config, ConfigError> {
        let mut config = Config::new();
        
        for source in &self.sources {
            let values = source.load()?;
            for (key, value) in values {
                config.set(&key, value)?;
            }
        }

        Ok(config)
    }
}
```

### 4. App Builder Boilerplate

The `App` implementation contains complex initialization code:

```rust
// src/app.rs (partial)
impl App {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            registry: Arc::new(ComponentRegistry::new()),
        }
    }

    pub fn add_web_plugin(mut self, plugin: WebPlugin) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub async fn run(self) -> Result<(), Error> {
        // Initialize all plugins
        for plugin in &self.plugins {
            plugin.initialize(&self.registry).await?;
        }

        // Start all plugins
        for plugin in &self.plugins {
            plugin.start().await?;
        }

        // Keep the application running
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let _ = rx.await;

        // Stop all plugins
        for plugin in self.plugins.iter().rev() {
            plugin.stop().await?;
        }

        Ok(())
    }
}
```

## After Zero Boilerplate Initiative

### 1. Simplified Main Entry Point

```rust
// src/main.rs (after)
#[navius_app]
async fn main() {
    // Empty! All configuration handled by framework
}

#[get("/api/v1/hello")]
async fn hello_world() -> impl IntoResponse {
    Json(json!({ "message": "Hello, world!" }))
}

#[get("/api/v1/context")]
async fn context_info(State(state): State<AppState>) -> impl IntoResponse {
    format!("Component types: {:?}", state.app.registry().component_types())
}
```

### 2. No WebPlugin Required

User code no longer needs to implement or copy WebPlugin - it's provided by the framework.

### 3. Declarative Configuration

```rust
// config/app.toml
[server]
host = "127.0.0.1"
port = 3000

[logging]
level = "debug"

# Configuration automatically loaded by the framework
```

### 4. No App Builder Boilerplate

The App Builder is fully abstracted by the framework and the `#[navius_app]` macro.

## Line Count Comparison

| Component | Before | After | Reduction |
|-----------|--------|-------|-----------|
| main.rs | 51 | 10 | 80% |
| web_plugin.rs | 169 | 0 | 100% |
| web_configurator.rs | 87 | 0 | 100% |
| app.rs | 120 | 0 | 100% |
| **Total** | **427** | **10** | **98%** |

## Benefits

1. **Dramatically reduced code** - 98% less infrastructure code
2. **Focus on domain logic** - Developers write only business-specific code
3. **Reduced learning curve** - New developers can be productive faster
4. **Standardized patterns** - Consistent application structure
5. **Automatic best practices** - Framework applies optimal defaults

## Next Steps

See the [Zero Boilerplate Implementation Plan](./zero-boilerplate-plan.md) for detailed phases and timeline. 
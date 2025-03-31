//! Full-Stack Integration Example
//!
//! This example demonstrates the integration of various Navius components
//! in a complete application. It implements a task management system with
//! user authentication, task tracking, notifications, and reporting.

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod plugins;

// Re-export core application components
pub use config::AppConfig;

/// Application bootstrap function to initialize the app
pub async fn bootstrap() -> Result<(), navius_core::error::Error> {
    use crate::config::load_config;

    // Load application configuration
    let config = load_config().await?;

    // Initialize logger
    init_logger(&config);

    // Initialize the application
    let app = init_application(config).await?;

    // Start the application
    app.start().await
}

/// Initialize the logger
fn init_logger(config: &config::AppConfig) {
    use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan, prelude::*, registry};

    let env_filter = if config.environment.as_str() == "development" {
        EnvFilter::new(format!(
            "full_stack_integration_example={},navius=debug,tower_http=debug",
            config.log_level
        ))
    } else {
        EnvFilter::new(format!(
            "full_stack_integration_example={},navius=info",
            config.log_level
        ))
    };

    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true);

    registry().with(env_filter).with(fmt_layer).init();
}

/// Initialize the application
async fn init_application(
    config: std::sync::Arc<config::AppConfig>,
) -> Result<Application, navius_core::error::Error> {
    use navius_core::di::ServiceProvider;
    use navius_http::server::HttpServer;

    // Create service provider
    let service_provider = create_service_provider(config.clone()).await?;

    // Create HTTP server
    let server = create_http_server(config.clone(), service_provider.clone()).await?;

    // Create application instance
    let app = Application::new(config, service_provider, server);

    Ok(app)
}

/// Create the service provider with all dependencies
async fn create_service_provider(
    config: std::sync::Arc<config::AppConfig>,
) -> Result<std::sync::Arc<dyn navius_core::di::ServiceProvider>, navius_core::error::Error> {
    use navius_core::di::{Container, ServiceCollection};

    // Create service collection
    let mut services = ServiceCollection::new();

    // Register config
    services.add_singleton_value(config.clone());

    // Register infrastructure services
    infrastructure::register_services(&mut services, &config).await?;

    // Register application services
    application::register_services(&mut services)?;

    // Register plugins
    plugins::register_services(&mut services)?;

    // Build the container
    let provider = services.build()?;

    Ok(std::sync::Arc::new(provider))
}

/// Create the HTTP server
async fn create_http_server(
    config: std::sync::Arc<config::AppConfig>,
    service_provider: std::sync::Arc<dyn navius_core::di::ServiceProvider>,
) -> Result<navius_http::server::HttpServer, navius_core::error::Error> {
    // Configure and build the HTTP server
    let server = api::configure_server(config, service_provider).await?;

    Ok(server)
}

/// Main application struct
pub struct Application {
    config: std::sync::Arc<config::AppConfig>,
    service_provider: std::sync::Arc<dyn navius_core::di::ServiceProvider>,
    http_server: navius_http::server::HttpServer,
}

impl Application {
    /// Create a new application instance
    pub fn new(
        config: std::sync::Arc<config::AppConfig>,
        service_provider: std::sync::Arc<dyn navius_core::di::ServiceProvider>,
        http_server: navius_http::server::HttpServer,
    ) -> Self {
        Self {
            config,
            service_provider,
            http_server,
        }
    }

    /// Start the application
    pub async fn start(&self) -> Result<(), navius_core::error::Error> {
        // Log application startup
        tracing::info!("Starting Full-Stack Integration Example application");
        tracing::info!("Environment: {}", self.config.environment);

        // Start the HTTP server
        self.http_server.start().await?;

        Ok(())
    }
}

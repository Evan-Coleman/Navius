use async_trait::async_trait;
use axum::{Router, routing::get};
use navius_core::{di::Application, error::Result};
use navius_plugin::plugin::{MessageHandler, PluginLifecycle, PluginLifecycleStage};
use navius_plugin::{BasePlugin, PluginConfig, PluginHealth, PluginMetadata, PluginResult};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

/// WebPlugin provides web server capabilities to Navius applications
#[derive(Debug)]
pub struct WebPlugin {
    base: BasePlugin,
    host: String,
    port: u16,
    router: Option<Router>,
    lifecycle_stage: PluginLifecycleStage,
}

impl WebPlugin {
    /// Create a new WebPlugin with default settings
    pub fn new() -> Self {
        Self {
            base: BasePlugin::new(PluginMetadata::new(
                "web",
                "0.1.0",
                "Web Server Plugin",
                "Provides web server capabilities to Navius applications",
                "Navius Team",
            )),
            host: "127.0.0.1".to_string(),
            port: 3001,
            router: None,
            lifecycle_stage: PluginLifecycleStage::Created,
        }
    }

    /// Set the host address
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the port number
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Get the host address
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port number
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Set the router for the web server
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Start the web server with the configured settings
    pub async fn start_server<T: Clone + Send + Sync + 'static>(&self, app_state: T) -> Result<()> {
        // Create router with explicit state
        let router = if let Some(router) = &self.router {
            // Convert the router to a service
            router
                .clone()
                .into_make_service_with_connect_info::<std::net::SocketAddr>()
        } else {
            // Default router
            Router::new()
                .route("/", get(|| async { "Welcome to Navius Web Plugin!" }))
                .with_state(app_state)
                .into_make_service_with_connect_info::<std::net::SocketAddr>()
        };

        let addr = format!("{}:{}", self.host, self.port)
            .parse::<SocketAddr>()
            .map_err(|e| navius_core::error::Error::internal(format!("Invalid address: {}", e)))?;

        info!("Starting web server on {}", addr);

        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            navius_core::error::Error::internal(format!("Failed to bind to address: {}", e))
        })?;

        info!("Server listening on http://{}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| navius_core::error::Error::internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}

/// Define a simple application state that contains the application context
#[derive(Clone, Debug)]
pub struct AppState {
    pub app: Arc<Application>,
}

#[async_trait]
impl PluginLifecycle for WebPlugin {
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Extract configuration values if present
        if let Some(host) = config.get_string("host") {
            self.host = host;
        }

        if let Some(port) = config.get_number("port") {
            self.port = port as u16;
        }

        self.lifecycle_stage = PluginLifecycleStage::Initialized;
        info!(
            "Initialized WebPlugin with host: {}, port: {}",
            self.host, self.port
        );
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        self.lifecycle_stage = PluginLifecycleStage::Started;
        info!("Started WebPlugin - server will be started when needed");
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        self.lifecycle_stage = PluginLifecycleStage::Stopped;
        info!("Stopped WebPlugin");
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        PluginHealth::Healthy
    }
}

#[async_trait]
impl MessageHandler for WebPlugin {
    async fn handle_message(&self, _message: Value) -> PluginResult<Option<Value>> {
        // No message handling for now
        Ok(None)
    }
}

impl navius_plugin::Plugin for WebPlugin {
    fn id(&self) -> &str {
        self.base.metadata().id.as_str()
    }

    fn version(&self) -> &str {
        self.base.metadata().version.as_str()
    }

    fn metadata(&self) -> &PluginMetadata {
        self.base.metadata()
    }

    fn lifecycle_stage(&self) -> PluginLifecycleStage {
        self.lifecycle_stage
    }

    fn capabilities(&self) -> HashMap<String, Arc<dyn std::any::Any + Send + Sync>> {
        self.base.capabilities()
    }
}

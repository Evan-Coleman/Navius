use crate::plugins::web_plugin::AppState;
use axum::Router;
use navius_core::{
    di::{Application, ApplicationBuilder},
    error::Result,
};
use navius_plugin::{Plugin, PluginResult, plugin::PluginLifecycle};
use std::sync::Arc;
use tracing::info;

/// Application builder for Navius applications
pub struct App {
    builder: ApplicationBuilder,
    plugins: Vec<Box<dyn Plugin>>,
    web_plugin: Option<crate::plugins::WebPlugin>, // Store the WebPlugin separately
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            builder: ApplicationBuilder::new(),
            plugins: Vec::new(),
            web_plugin: None,
        }
    }

    /// Add a plugin to the application
    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        info!("Adding plugin: {}", plugin.id());
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Add a web plugin to the application
    pub fn add_web_plugin(mut self, plugin: crate::plugins::WebPlugin) -> Self {
        info!("Adding web plugin: {}", plugin.id());
        // Store the web plugin separately
        self.web_plugin = Some(plugin);
        self
    }

    /// Get a reference to the application builder
    pub fn builder(&self) -> &ApplicationBuilder {
        &self.builder
    }

    /// Get a mutable reference to the application builder
    pub fn builder_mut(&mut self) -> &mut ApplicationBuilder {
        &mut self.builder
    }

    /// Run the application
    pub async fn run(mut self) -> Result<()> {
        // Build the application
        let app: Application = self.builder.build();
        let app_arc = Arc::new(app);

        // Initialize plugins
        for plugin in &mut self.plugins {
            let result: PluginResult<()> = plugin.initialize(Default::default()).await;
            if let Err(e) = result {
                return Err(navius_core::error::Error::internal(format!(
                    "Failed to initialize plugin {}: {}",
                    plugin.id(),
                    e
                )));
            }
        }

        // Start plugins
        for plugin in &mut self.plugins {
            let result: PluginResult<()> = plugin.start().await;
            if let Err(e) = result {
                return Err(navius_core::error::Error::internal(format!(
                    "Failed to start plugin {}: {}",
                    plugin.id(),
                    e
                )));
            }
        }

        // Handle web plugin if present
        if let Some(mut web_plugin) = self.web_plugin {
            // Initialize web plugin
            let result = web_plugin.initialize(Default::default()).await;
            if let Err(e) = result {
                return Err(navius_core::error::Error::internal(format!(
                    "Failed to initialize web plugin: {}",
                    e
                )));
            }

            // Start web plugin
            let result = web_plugin.start().await;
            if let Err(e) = result {
                return Err(navius_core::error::Error::internal(format!(
                    "Failed to start web plugin: {}",
                    e
                )));
            }

            // Create application state
            let app_state = AppState {
                app: app_arc.clone(),
            };

            // Start the web server
            return web_plugin.start_server(app_state).await;
        }

        // No web plugin found
        Err(navius_core::error::Error::internal(
            "No web plugin found. Add a WebPlugin with add_web_plugin() to serve HTTP requests.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_plugin::{BasePlugin, PluginBuilder, PluginLifecycleStage, PluginMetadata};

    // Create a proper test plugin
    #[derive(Debug)]
    struct TestPlugin {
        base: BasePlugin,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                base: BasePlugin::builder()
                    .id("test_plugin")
                    .name("Test Plugin")
                    .version("1.0.0")
                    .description("A simple test plugin")
                    .author("Test Author")
                    .build(),
            }
        }
    }

    #[async_trait::async_trait]
    impl navius_plugin::MessageHandler for TestPlugin {
        async fn handle_message(
            &self,
            _message: serde_json::Value,
        ) -> navius_plugin::PluginResult<Option<serde_json::Value>> {
            Ok(None)
        }
    }

    #[async_trait::async_trait]
    impl navius_plugin::PluginLifecycle for TestPlugin {
        async fn initialize(
            &mut self,
            _config: navius_plugin::PluginConfig,
        ) -> navius_plugin::PluginResult<()> {
            self.base
                .set_lifecycle_stage(navius_plugin::PluginLifecycleStage::Initialized);
            Ok(())
        }

        async fn start(&mut self) -> navius_plugin::PluginResult<()> {
            self.base
                .set_lifecycle_stage(navius_plugin::PluginLifecycleStage::Started);
            Ok(())
        }

        async fn stop(&mut self) -> navius_plugin::PluginResult<()> {
            self.base
                .set_lifecycle_stage(navius_plugin::PluginLifecycleStage::Stopped);
            Ok(())
        }

        async fn health_check(&self) -> navius_plugin::PluginHealth {
            navius_plugin::PluginHealth::Up
        }
    }

    impl Plugin for TestPlugin {
        fn id(&self) -> &str {
            self.base.id()
        }

        fn version(&self) -> &str {
            self.base.version()
        }

        fn metadata(&self) -> &PluginMetadata {
            self.base.metadata()
        }

        fn lifecycle_stage(&self) -> PluginLifecycleStage {
            self.base.lifecycle_stage()
        }
    }

    #[tokio::test]
    async fn test_app_builder() {
        // Create a new App instance
        let app = App::new().add_plugin(TestPlugin::new()).build();

        // Should not panic
        assert!(true);
    }

    #[tokio::test]
    async fn test_app_run() {
        // Create a new App instance and run it
        let result = App::new().add_plugin(TestPlugin::new()).run().await;

        // Should run without error
        assert!(result.is_ok());
    }
}

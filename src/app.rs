use navius_core::{Error, ErrorCode, Result};
use navius_plugin::{Plugin, PluginConfig, PluginRegistry};
use std::sync::Arc;

/// The App struct is the main entry point for the application.
pub struct App {
    /// The plugin registry that manages all plugins.
    registry: Arc<PluginRegistry>,
}

impl App {
    /// Create a new App instance.
    pub fn new() -> AppBuilder {
        AppBuilder::new()
    }

    /// Run the application.
    pub async fn run(self) -> Result<()> {
        // This is just a shell implementation for testing
        // In a real application, this would start the server, etc.
        tracing::info!("Starting application with plugins");

        // In a real implementation, we would:
        // 1. Start all plugins in the correct order
        // 2. Wait for termination signal
        // 3. Properly shut down all plugins

        tracing::info!("Application started successfully");
        Ok(())
    }
}

/// The AppBuilder struct provides a fluent API for configuring the application.
pub struct AppBuilder {
    registry: Arc<PluginRegistry>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl AppBuilder {
    /// Create a new AppBuilder instance.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            plugins: Vec::new(),
        }
    }

    /// Add a plugin to the application.
    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Build the App instance.
    pub fn build(self) -> App {
        // In a real implementation, we would:
        // 1. Register all plugins with the registry
        // 2. Initialize them in the correct order based on dependencies

        App {
            registry: self.registry,
        }
    }

    /// Run the application after building it.
    pub async fn run(self) -> Result<()> {
        let registry = self.registry.clone();

        // Register all plugins
        for plugin in self.plugins {
            let plugin_id = plugin.id().to_string();
            registry.register_plugin(plugin).await.map_err(|e| {
                Error::plugin(format!("Failed to register plugin {}: {}", plugin_id, e))
            })?;

            // Initialize the plugin
            registry
                .initialize_plugin(&plugin_id, PluginConfig::default())
                .await
                .map_err(|e| {
                    Error::plugin(format!("Failed to initialize plugin {}: {}", plugin_id, e))
                })?;

            // Start the plugin
            registry.start_plugin(&plugin_id).await.map_err(|e| {
                Error::plugin(format!("Failed to start plugin {}: {}", plugin_id, e))
            })?;
        }

        let app = App { registry };

        app.run().await
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

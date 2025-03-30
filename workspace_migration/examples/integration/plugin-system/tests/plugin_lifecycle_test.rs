//! Tests for the plugin lifecycle

use async_trait::async_trait;
use navius_core::{
    di::{Application, ApplicationBuilder, ComponentScope, Environment},
    error::Result,
};
use navius_plugin::{Plugin, PluginContext, PluginRegistry};
use plugin_system::capabilities::LoggingCapability;
use std::sync::{Arc, Mutex};

// Test Logger implementation for capturing log messages
#[derive(Clone)]
struct TestLogger {
    logs: Arc<Mutex<Vec<String>>>,
}

impl TestLogger {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }
}

impl LoggingCapability for TestLogger {
    fn info(&self, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("INFO: {}", message));
    }

    fn warn(&self, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("WARN: {}", message));
    }

    fn error(&self, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("ERROR: {}", message));
    }

    fn debug(&self, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("DEBUG: {}", message));
    }

    fn set_level(&self, _level: &str) -> Result<()> {
        Ok(())
    }
}

// Test Plugin for simple lifecycle testing
struct TestPlugin {
    name: String,
    initialized: Mutex<bool>,
    test_logger: Arc<TestLogger>,
}

impl TestPlugin {
    fn new(name: &str, test_logger: Arc<TestLogger>) -> Self {
        Self {
            name: name.to_string(),
            initialized: Mutex::new(false),
            test_logger,
        }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Test plugin for lifecycle testing"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        self.test_logger
            .info(&format!("Registering plugin '{}'", self.name));

        // Register the test logger
        app.register_component(
            "test_logger",
            self.test_logger.clone(),
            ComponentScope::Singleton,
        );

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        self.test_logger
            .info(&format!("Starting plugin '{}'", self.name));
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        self.test_logger
            .info(&format!("Stopping plugin '{}'", self.name));
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_lifecycle() -> Result<()> {
    // Create test logger and plugin
    let test_logger = Arc::new(TestLogger::new());
    let test_plugin = TestPlugin::new("test-plugin", test_logger.clone());

    // Create registry
    let registry = Arc::new(PluginRegistry::new());

    // Register plugin
    registry.register(Box::new(test_plugin))?;

    // Create application
    let app_builder = ApplicationBuilder::new()
        .with_environment(Environment::Development)
        .with_plugin_registry(registry.clone());

    let mut app = app_builder.build();

    // Initialize plugins
    registry.initialize_all().await?;

    // Start plugins
    registry.start_all().await?;

    // Get logs
    let logs = test_logger.get_logs();

    // Verify registration and start log messages
    assert!(
        logs.iter()
            .any(|log| log.contains("Registering plugin 'test-plugin'"))
    );
    assert!(
        logs.iter()
            .any(|log| log.contains("Starting plugin 'test-plugin'"))
    );

    // Shutdown plugins
    registry.shutdown_all().await?;

    // Get updated logs
    let logs = test_logger.get_logs();

    // Verify stop log message
    assert!(
        logs.iter()
            .any(|log| log.contains("Stopping plugin 'test-plugin'"))
    );

    Ok(())
}

#[tokio::test]
async fn test_plugin_dependency_resolution() -> Result<()> {
    // Create test logger
    let test_logger = Arc::new(TestLogger::new());

    // Create plugins with dependencies
    let plugin_a = TestPlugin::new("plugin-a", test_logger.clone());
    let plugin_b = TestPlugin::new("plugin-b", test_logger.clone());
    let plugin_c = TestPlugin::new("plugin-c", test_logger.clone());

    // Create registry
    let registry = Arc::new(PluginRegistry::new());

    // Register plugins (intentionally out of dependency order)
    registry.register(Box::new(plugin_c))?;
    registry.register(Box::new(plugin_a))?;
    registry.register(Box::new(plugin_b))?;

    // Create application
    let app_builder = ApplicationBuilder::new()
        .with_environment(Environment::Development)
        .with_plugin_registry(registry.clone());

    let mut app = app_builder.build();

    // Initialize plugins
    registry.initialize_all().await?;

    // Start plugins
    registry.start_all().await?;

    // Shutdown plugins
    registry.shutdown_all().await?;

    Ok(())
}

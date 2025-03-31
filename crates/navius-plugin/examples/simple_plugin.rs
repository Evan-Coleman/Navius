use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use navius_plugin::{
    Capability, ConfigurationCapability, HealthCheckCapability, LoggingCapability, PluginRegistry,
    PluginRegistryManager, impl_capability, plugin,
};

// Define a simple logging capability
#[derive(Debug)]
struct SimpleLogger {
    prefix: String,
}

impl SimpleLogger {
    fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

// Implement the Capability trait for SimpleLogger
impl_capability!(SimpleLogger, "simple_logger");

// Implement the LoggingCapability trait for SimpleLogger
impl LoggingCapability for SimpleLogger {
    fn debug(&self, message: &str) {
        println!("[DEBUG] {}: {}", self.prefix, message);
    }

    fn info(&self, message: &str) {
        println!("[INFO] {}: {}", self.prefix, message);
    }

    fn warn(&self, message: &str) {
        println!("[WARN] {}: {}", self.prefix, message);
    }

    fn error(&self, message: &str) {
        println!("[ERROR] {}: {}", self.prefix, message);
    }

    fn log_event(&self, event_type: &str, data: serde_json::Value) {
        println!("[EVENT] {}: {} - {}", self.prefix, event_type, data);
    }
}

// Define a simple health check capability
#[derive(Debug)]
struct SimpleHealthCheck {
    healthy: bool,
    details: HashMap<String, String>,
}

impl SimpleHealthCheck {
    fn new() -> Self {
        let mut details = HashMap::new();
        details.insert("version".to_string(), "1.0.0".to_string());
        details.insert("uptime".to_string(), "0".to_string());

        Self {
            healthy: true,
            details,
        }
    }

    fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }

    fn add_detail(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.details.insert(key.into(), value.into());
    }
}

// Implement the Capability trait for SimpleHealthCheck
impl_capability!(SimpleHealthCheck, "simple_health_check");

// Implement the HealthCheckCapability trait for SimpleHealthCheck
#[async_trait::async_trait]
impl HealthCheckCapability for SimpleHealthCheck {
    async fn check_health(&self) -> bool {
        self.healthy
    }

    async fn health_details(&self) -> serde_json::Value {
        json!({
            "healthy": self.healthy,
            "details": self.details
        })
    }
}

// Define a simple configuration capability
#[derive(Debug)]
struct SimpleConfig {
    values: HashMap<String, String>,
}

impl SimpleConfig {
    fn new() -> Self {
        let mut values = HashMap::new();
        values.insert("log_level".to_string(), "info".to_string());
        values.insert("max_connections".to_string(), "10".to_string());

        Self { values }
    }
}

// Implement the Capability trait for SimpleConfig
impl_capability!(SimpleConfig, "simple_config");

// Implement the ConfigurationCapability trait for SimpleConfig
impl ConfigurationCapability for SimpleConfig {
    fn get_config_keys(&self) -> Vec<&'static str> {
        vec!["log_level", "max_connections"]
    }

    fn set_config(&mut self, key: &str, value: &str) -> bool {
        if self.get_config_keys().contains(&key) {
            self.values.insert(key.to_string(), value.to_string());
            true
        } else {
            false
        }
    }

    fn get_config(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a plugin registry
    let registry = Arc::new(PluginRegistry::new());

    // Create a simple plugin
    let simple_plugin = plugin!(
        "SimplePlugin",
        "1.0.0",
        "A simple example plugin",
        "Navius Team"
    )
    .with_tag("example")
    .with_tag("simple")
    .with_capability(SimpleLogger::new("SimplePlugin"))
    .with_capability(SimpleHealthCheck::new())
    .with_capability(SimpleConfig::new())
    .build();

    // Register the plugin
    let plugin_id = registry.register_plugin(simple_plugin).await?;
    println!("Registered plugin with ID: {}", plugin_id);

    // Start all plugins
    registry.start_all_plugins().await?;

    // Get the plugin
    let plugin = registry.get_plugin(plugin_id).unwrap();
    println!("Got plugin: {}", plugin.metadata().name);

    // Check if the plugin has the logger capability
    if plugin.has_capability("simple_logger") {
        println!("Plugin has logger capability");

        // Find plugins with the logger capability
        let plugins_with_logger = registry.find_plugins_by_capability("simple_logger");
        println!(
            "Found {} plugins with logger capability",
            plugins_with_logger.len()
        );

        // Use the logger capability
        for p in plugins_with_logger {
            if let Some(logger) = p.get_capability("simple_logger") {
                if let Some(logger) = logger.downcast_ref::<Box<dyn LoggingCapability>>() {
                    logger.info("Hello from the plugin system!");
                    logger.log_event(
                        "example_event",
                        json!({
                            "time": "now",
                            "value": 42
                        }),
                    );
                }
            }
        }
    }

    // Check plugin health
    let health_results = registry.health_check_all().await;
    for (name, health) in health_results {
        println!("Plugin '{}' health: {}", name, health);
    }

    // Stop all plugins
    registry.stop_all_plugins().await?;
    println!("Stopped all plugins");

    Ok(())
}

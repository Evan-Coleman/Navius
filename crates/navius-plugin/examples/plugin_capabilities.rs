use navius_plugin::{
    PluginRegistry,
    capability::{ConfigurationCapability, HealthCheckCapability, LoggingCapability},
    error::{PluginHealth, PluginResult},
    impl_capability, plugin,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Define a custom logger capability
#[derive(Debug, Clone)]
struct SimpleLogger {
    name: String,
    log_count: Arc<Mutex<u32>>,
}

impl SimpleLogger {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            log_count: Arc::new(Mutex::new(0)),
        }
    }

    // Helper to increment log count
    fn increment_log_count(&self) {
        if let Ok(mut count) = self.log_count.lock() {
            *count += 1;
        }
    }

    // Get the log count
    fn get_log_count(&self) -> u32 {
        if let Ok(count) = self.log_count.lock() {
            *count
        } else {
            0
        }
    }
}

// Implement the Capability trait
impl_capability!(SimpleLogger, "simple_logger");

// Implement the LoggingCapability trait
impl LoggingCapability for SimpleLogger {
    fn debug(&self, message: &str) {
        println!("[DEBUG] {}: {}", self.name, message);
        self.increment_log_count();
    }

    fn info(&self, message: &str) {
        println!("[INFO] {}: {}", self.name, message);
        self.increment_log_count();
    }

    fn warn(&self, message: &str) {
        println!("[WARN] {}: {}", self.name, message);
        self.increment_log_count();
    }

    fn error(&self, message: &str) {
        println!("[ERROR] {}: {}", self.name, message);
        self.increment_log_count();
    }

    fn log_event(&self, event_type: &str, data: HashMap<String, Value>) {
        println!("[EVENT][{}] {}: {:?}", event_type, self.name, data);
        self.increment_log_count();
    }
}

// Define a health check capability
#[derive(Debug, Clone)]
struct SimpleHealthCheck {
    name: String,
    healthy: bool,
    details: HashMap<String, Value>,
}

impl SimpleHealthCheck {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            healthy: true,
            details: HashMap::new(),
        }
    }

    fn mark_unhealthy(&mut self, reason: &str) {
        self.healthy = false;
        self.details
            .insert("reason".to_string(), Value::String(reason.to_string()));
    }

    fn mark_healthy(&mut self) {
        self.healthy = true;
        self.details.remove("reason");
    }

    fn add_detail(&mut self, key: &str, value: Value) {
        self.details.insert(key.to_string(), value);
    }
}

// Implement the Capability trait
impl_capability!(SimpleHealthCheck, "simple_health");

// Implement the HealthCheckCapability trait
impl HealthCheckCapability for SimpleHealthCheck {
    async fn check_health(&self) -> PluginHealth {
        if self.healthy {
            PluginHealth::Healthy
        } else if let Some(reason) = self.details.get("reason") {
            if let Some(reason_str) = reason.as_str() {
                PluginHealth::Unhealthy {
                    message: reason_str.to_string(),
                }
            } else {
                PluginHealth::Unhealthy {
                    message: "Unknown reason".to_string(),
                }
            }
        } else {
            PluginHealth::Unhealthy {
                message: "No reason provided".to_string(),
            }
        }
    }

    async fn health_details(&self) -> HashMap<String, Value> {
        self.details.clone()
    }
}

// Define a configuration capability
#[derive(Debug, Clone)]
struct SimpleConfig {
    config: HashMap<String, Value>,
}

impl SimpleConfig {
    fn new() -> Self {
        let mut config = HashMap::new();
        config.insert("version".to_string(), Value::String("1.0.0".to_string()));

        Self { config }
    }
}

// Implement the Capability trait
impl_capability!(SimpleConfig, "simple_config");

// Implement the ConfigurationCapability trait
impl ConfigurationCapability for SimpleConfig {
    fn get_config_keys(&self) -> Vec<String> {
        self.config.keys().cloned().collect()
    }

    fn get_config_string(&self, key: &str) -> Option<String> {
        self.config
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    fn get_config_number(&self, key: &str) -> Option<f64> {
        self.config.get(key).and_then(|v| v.as_f64())
    }

    fn get_config_bool(&self, key: &str) -> Option<bool> {
        self.config.get(key).and_then(|v| v.as_bool())
    }

    fn get_config_value(&self, key: &str) -> Option<Value> {
        self.config.get(key).cloned()
    }

    fn set_config<T: serde::Serialize>(&mut self, key: &str, value: T) -> PluginResult<()> {
        match serde_json::to_value(value) {
            Ok(json_value) => {
                self.config.insert(key.to_string(), json_value);
                Ok(())
            }
            Err(e) => Err(navius_plugin::error::PluginError::ConfigurationError(
                format!("Failed to serialize value: {}", e),
            )),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Plugin Capabilities Example");
    println!("==========================");

    // Create a plugin registry
    let registry = PluginRegistry::new();

    // Create logger capability
    let logger = SimpleLogger::new("CapabilityPlugin");

    // Create health check capability
    let mut health_check = SimpleHealthCheck::new("CapabilityPlugin");
    health_check.add_detail("version", Value::String("1.0.0".to_string()));
    health_check.add_detail("uptime", Value::Number(0.into()));

    // Create config capability
    let mut config = SimpleConfig::new();
    config.set_config("debug_mode", true)?;
    config.set_config("max_connections", 100)?;

    // Create a plugin with capabilities
    let plugin = plugin!(
        "CapabilityPlugin",
        "1.0.0",
        "A plugin with capabilities",
        "Navius Team"
    )
    .with_tag("example")
    .with_named_capability("logger", logger.clone())
    .with_named_capability("health", health_check.clone())
    .with_named_capability("config", config)
    .build();

    // Register the plugin
    println!("\nRegistering plugin...");
    let plugin_id = registry.register_plugin(plugin).await?;
    println!("Registered plugin: {}", plugin_id);

    // Initialize and start the plugin
    println!("\nInitializing and starting plugin...");
    let config = navius_plugin::PluginConfig::default();
    registry.initialize_plugin(&plugin_id, config).await?;
    registry.start_plugin(&plugin_id).await?;

    // Get and use the logger capability
    println!("\nUsing logger capability...");
    let logger_capability = registry.get_plugin_capability(&plugin_id, "logger")?;

    // Downcast to the specific capability type
    let logger = logger_capability
        .downcast_ref::<SimpleLogger>()
        .expect("Failed to downcast to SimpleLogger");

    logger.debug("This is a debug message");
    logger.info("This is an info message");
    logger.warn("This is a warning message");
    logger.error("This is an error message");

    let mut event_data = HashMap::new();
    event_data.insert("event_id".to_string(), Value::String("12345".to_string()));
    event_data.insert(
        "timestamp".to_string(),
        Value::String("2025-03-29T12:00:00Z".to_string()),
    );

    logger.log_event("SYSTEM", event_data);

    println!("Total log entries: {}", logger.get_log_count());

    // Get and use the configuration capability
    println!("\nUsing configuration capability...");
    let config_capability = registry.get_plugin_capability(&plugin_id, "config")?;

    // Downcast to the specific capability type
    let config = config_capability
        .downcast_ref::<SimpleConfig>()
        .expect("Failed to downcast to SimpleConfig");

    println!("Available config keys: {:?}", config.get_config_keys());
    println!("Version: {:?}", config.get_config_string("version"));
    println!("Debug mode: {:?}", config.get_config_bool("debug_mode"));
    println!(
        "Max connections: {:?}",
        config.get_config_number("max_connections")
    );

    // Get and use the health check capability
    println!("\nUsing health check capability...");
    let health_capability = registry.get_plugin_capability(&plugin_id, "health")?;

    // Downcast to the specific capability type
    let health = health_capability
        .downcast_ref::<SimpleHealthCheck>()
        .expect("Failed to downcast to SimpleHealthCheck");

    println!("Current health: {:?}", health.check_health().await);
    println!("Health details: {:?}", health.health_details().await);

    // Modify the health (this would be unsafe in a real application since
    // it requires mutable access to a shared reference, but it's just for demonstration)
    let health_mut =
        unsafe { &mut *(health as *const SimpleHealthCheck as *mut SimpleHealthCheck) };

    // Mark the plugin as unhealthy
    health_mut.mark_unhealthy("Memory usage is too high");

    // Check health again
    println!("\nHealth after marking unhealthy:");
    println!("Current health: {:?}", health.check_health().await);

    // Stop the plugin
    println!("\nStopping plugin...");
    registry.stop_plugin(&plugin_id).await?;

    println!("\nPlugin capabilities example completed");

    Ok(())
}

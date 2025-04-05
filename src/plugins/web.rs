use navius_core::{Error, Result};
use navius_plugin::{
    BasePlugin, MessageHandler, Plugin, PluginConfig, PluginHealth, PluginLifecycle,
    PluginLifecycleStage, PluginMetadata, PluginResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Configuration for the WebPlugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPluginConfig {
    /// Default CORS origin pattern
    pub default_cors_origin: String,
    /// Maximum payload size in bytes
    pub max_payload_size: usize,
}

impl Default for WebPluginConfig {
    fn default() -> Self {
        Self {
            default_cors_origin: "*".to_string(),
            max_payload_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// The WebPlugin provides web server functionality using axum.
#[derive(Debug)]
pub struct WebPlugin {
    base: BasePlugin,
    config: Arc<WebPluginConfig>,
}

impl WebPlugin {
    /// Create a new WebPlugin instance.
    pub fn new() -> Self {
        Self {
            base: BasePlugin::builder()
                .id("web_plugin")
                .name("Web Server Plugin")
                .version("1.0.0")
                .description("Web server functionality using Axum")
                .author("Navius Team")
                .build(),
            config: Arc::new(WebPluginConfig::default()),
        }
    }

    /// Create a new WebPlugin instance with the specified configuration.
    pub fn with_config(config: WebPluginConfig) -> Self {
        Self {
            base: BasePlugin::builder()
                .id("web_plugin")
                .name("Web Server Plugin")
                .version("1.0.0")
                .description("Web server functionality using Axum")
                .author("Navius Team")
                .build(),
            config: Arc::new(config),
        }
    }

    /// Get the plugin configuration
    pub fn config(&self) -> &WebPluginConfig {
        &self.config
    }
}

impl Default for WebPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MessageHandler for WebPlugin {
    async fn handle_message(&self, message: Value) -> PluginResult<Option<Value>> {
        tracing::debug!("WebPlugin received message: {:?}", message);
        // In a real implementation, this would handle various types of messages
        Ok(None)
    }
}

#[async_trait::async_trait]
impl PluginLifecycle for WebPlugin {
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Parse config and update the plugin configuration
        if let Some(config_value) = config.settings.get("web") {
            match serde_json::from_value::<WebPluginConfig>(config_value.clone()) {
                Ok(parsed_config) => {
                    self.config = Arc::new(parsed_config);
                    tracing::info!("WebPlugin initialized with custom configuration");
                }
                Err(e) => {
                    return Err(navius_plugin::PluginError::ConfigurationError(format!(
                        "Failed to parse WebPlugin configuration: {}",
                        e
                    )));
                }
            }
        }

        self.base
            .set_lifecycle_stage(PluginLifecycleStage::Initialized);
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        // In a real implementation, this would start the web server
        tracing::info!(
            "WebPlugin started with CORS origin: {} and max payload size: {} bytes",
            self.config.default_cors_origin,
            self.config.max_payload_size
        );

        self.base.set_lifecycle_stage(PluginLifecycleStage::Started);
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        // In a real implementation, this would stop the web server
        tracing::info!("WebPlugin stopped");

        self.base.set_lifecycle_stage(PluginLifecycleStage::Stopped);
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        // In a real implementation, this would check the health of the web server
        PluginHealth::Up
    }
}

impl Plugin for WebPlugin {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let plugin = WebPlugin::new();
        assert_eq!(plugin.config().default_cors_origin, "*");
        assert_eq!(plugin.config().max_payload_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_custom_config() {
        let config = WebPluginConfig {
            default_cors_origin: "https://example.com".to_string(),
            max_payload_size: 5 * 1024 * 1024,
        };
        let plugin = WebPlugin::with_config(config.clone());
        assert_eq!(plugin.config().default_cors_origin, "https://example.com");
        assert_eq!(plugin.config().max_payload_size, 5 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut plugin = WebPlugin::new();
        let mut config = PluginConfig::default();
        config.set(
            "web",
            json!({
                "default_cors_origin": "https://test.com",
                "max_payload_size": 2097152
            }),
        );

        let result = plugin.initialize(config).await;
        assert!(result.is_ok());
        assert_eq!(plugin.config().default_cors_origin, "https://test.com");
        assert_eq!(plugin.config().max_payload_size, 2097152);
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut plugin = WebPlugin::new();
        let mut config = PluginConfig::default();
        config.set("web", "invalid");

        let result = plugin.initialize(config).await;
        assert!(result.is_err());
    }
}

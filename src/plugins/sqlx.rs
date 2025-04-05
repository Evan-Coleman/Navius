use navius_core::Result;
use navius_plugin::{
    BasePlugin, MessageHandler, Plugin, PluginConfig, PluginHealth, PluginLifecycle,
    PluginLifecycleStage, PluginMetadata, PluginResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Configuration for the SqlxPlugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlxPluginConfig {
    /// Use database migrations
    pub use_migrations: bool,
    /// Path to migration directory
    pub migration_dir: String,
    /// Use connection pool
    pub use_connection_pool: bool,
}

impl Default for SqlxPluginConfig {
    fn default() -> Self {
        Self {
            use_migrations: true,
            migration_dir: "migrations".to_string(),
            use_connection_pool: true,
        }
    }
}

/// The SqlxPlugin provides database connectivity using sqlx.
#[derive(Debug)]
pub struct SqlxPlugin {
    base: BasePlugin,
    config: Arc<SqlxPluginConfig>,
}

impl SqlxPlugin {
    /// Create a new SqlxPlugin instance.
    pub fn new() -> Self {
        Self {
            base: BasePlugin::builder()
                .id("sqlx_plugin")
                .name("Sqlx Database Plugin")
                .version("1.0.0")
                .description("Database connectivity with Sqlx")
                .author("Navius Team")
                .build(),
            config: Arc::new(SqlxPluginConfig::default()),
        }
    }

    /// Create a new SqlxPlugin instance with a custom configuration.
    pub fn with_config(config: SqlxPluginConfig) -> Self {
        Self {
            base: BasePlugin::builder()
                .id("sqlx_plugin")
                .name("Sqlx Database Plugin")
                .version("1.0.0")
                .description("Database connectivity with Sqlx")
                .author("Navius Team")
                .build(),
            config: Arc::new(config),
        }
    }

    /// Get the plugin configuration
    pub fn config(&self) -> &SqlxPluginConfig {
        &self.config
    }
}

impl Default for SqlxPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MessageHandler for SqlxPlugin {
    async fn handle_message(&self, message: Value) -> PluginResult<Option<Value>> {
        tracing::debug!("SqlxPlugin received message: {:?}", message);
        // In a real implementation, this would handle database-related messages
        Ok(None)
    }
}

#[async_trait::async_trait]
impl PluginLifecycle for SqlxPlugin {
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Parse config and update the plugin configuration
        if let Some(config_value) = config.settings.get("sqlx") {
            match serde_json::from_value::<SqlxPluginConfig>(config_value.clone()) {
                Ok(parsed_config) => {
                    self.config = Arc::new(parsed_config);
                    tracing::info!("SqlxPlugin initialized with custom configuration");
                }
                Err(e) => {
                    return Err(navius_plugin::PluginError::ConfigurationError(format!(
                        "Failed to parse SqlxPlugin configuration: {}",
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
        // In a real implementation, this would start database connections
        tracing::info!(
            "SqlxPlugin started with migrations: {}, connection pool: {}",
            self.config.use_migrations,
            self.config.use_connection_pool
        );

        self.base.set_lifecycle_stage(PluginLifecycleStage::Started);
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        // In a real implementation, this would close database connections
        tracing::info!("SqlxPlugin stopped");

        self.base.set_lifecycle_stage(PluginLifecycleStage::Stopped);
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        // In a real implementation, this would check the database connectivity
        PluginHealth::Up
    }
}

impl Plugin for SqlxPlugin {
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
        let plugin = SqlxPlugin::new();
        assert!(plugin.config().use_migrations);
        assert_eq!(plugin.config().migration_dir, "migrations");
        assert!(plugin.config().use_connection_pool);
    }

    #[test]
    fn test_custom_config() {
        let config = SqlxPluginConfig {
            use_migrations: false,
            migration_dir: "custom_migrations".to_string(),
            use_connection_pool: false,
        };
        let plugin = SqlxPlugin::with_config(config);
        assert!(!plugin.config().use_migrations);
        assert_eq!(plugin.config().migration_dir, "custom_migrations");
        assert!(!plugin.config().use_connection_pool);
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut plugin = SqlxPlugin::new();
        let mut config = PluginConfig::default();
        config.set(
            "sqlx",
            json!({
                "use_migrations": false,
                "migration_dir": "test_migrations",
                "use_connection_pool": true
            }),
        );

        let result = plugin.initialize(config).await;
        assert!(result.is_ok());
        assert!(!plugin.config().use_migrations);
        assert_eq!(plugin.config().migration_dir, "test_migrations");
        assert!(plugin.config().use_connection_pool);
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut plugin = SqlxPlugin::new();
        let mut config = PluginConfig::default();
        config.set("sqlx", "invalid");

        let result = plugin.initialize(config).await;
        assert!(result.is_err());
    }
}

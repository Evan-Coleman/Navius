//! Plugin System Components
//!
//! This module contains plugin definitions and registration functions.

use async_trait::async_trait;
use navius_core::{
    di::{Application, ComponentScope},
    error::{Error, Result},
};
use navius_plugin::{Plugin, PluginContext, PluginRegistry};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

use crate::capabilities::{ConfigCapability, LoggingCapability, StorageCapability};
use crate::services::{ConfigService, LoggerService, StorageService};

/// Register all plugins with the plugin registry
pub fn register_all_plugins(registry: &Arc<PluginRegistry>) -> Result<()> {
    // Register base plugins first
    registry.register(Box::new(LoggingPlugin::new()))?;
    registry.register(Box::new(ConfigPlugin::new()))?;

    // Then feature-dependent plugins
    registry.register(Box::new(StoragePlugin::new()))?;
    registry.register(Box::new(ApiPlugin::new()))?;

    // Finally, the application plugins that depend on the others
    registry.register(Box::new(UserPlugin::new()))?;
    registry.register(Box::new(AnalyticsPlugin::new()))?;

    Ok(())
}

//
// Base Plugins
//

/// Logging Plugin provides logging capabilities to the application
pub struct LoggingPlugin {
    initialized: Mutex<bool>,
    logger: Option<Arc<dyn LoggingCapability>>,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            logger: None,
        }
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }

    fn description(&self) -> &str {
        "Provides logging capabilities to the application"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        Vec::new() // No dependencies
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering LoggingPlugin");

        // Create the logger service
        let logger = Arc::new(LoggerService::new("main", "debug"));

        // Store a reference
        let log_plugin = unsafe { &mut *(self as *const Self as *mut Self) };
        log_plugin.logger = Some(logger.clone());

        // Register with the application
        app.register_component("logger", logger, ComponentScope::Singleton);

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        if let Some(logger) = &self.logger {
            logger.info("LoggingPlugin started");
        }
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        if let Some(logger) = &self.logger {
            logger.info("LoggingPlugin stopped");
        }
        Ok(())
    }
}

/// Config Plugin provides configuration capabilities to the application
pub struct ConfigPlugin {
    initialized: Mutex<bool>,
    config_service: Option<Arc<dyn ConfigCapability>>,
}

impl ConfigPlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            config_service: None,
        }
    }
}

#[async_trait]
impl Plugin for ConfigPlugin {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "Provides configuration capabilities to the application"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["logging"] // Depends on logging
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering ConfigPlugin");

        // Get dependencies
        let logger = app.get_component::<Arc<dyn LoggingCapability>>("logger")?;

        // Create the config service
        let config_service = Arc::new(ConfigService::new(logger.clone()));

        // Store a reference
        let config_plugin = unsafe { &mut *(self as *const Self as *mut Self) };
        config_plugin.config_service = Some(config_service.clone());

        // Register with the application
        app.register_component("config", config_service, ComponentScope::Singleton);

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        if let Some(config) = &self.config_service {
            config.load()?;
        }
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        Ok(())
    }
}

/// Storage Plugin provides data storage capabilities to the application
pub struct StoragePlugin {
    initialized: Mutex<bool>,
    storage_service: Option<Arc<dyn StorageCapability>>,
}

impl StoragePlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            storage_service: None,
        }
    }
}

#[async_trait]
impl Plugin for StoragePlugin {
    fn name(&self) -> &str {
        "storage"
    }

    fn description(&self) -> &str {
        "Provides data storage capabilities to the application"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["logging", "config"] // Depends on logging and config
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering StoragePlugin");

        // Get dependencies
        let logger = app.get_component::<Arc<dyn LoggingCapability>>("logger")?;
        let config = app.get_component::<Arc<dyn ConfigCapability>>("config")?;

        // Create the storage service with dependencies
        let storage_service = Arc::new(StorageService::new(logger.clone(), config.clone()));

        // Store a reference
        let storage_plugin = unsafe { &mut *(self as *const Self as *mut Self) };
        storage_plugin.storage_service = Some(storage_service.clone());

        // Register with the application
        app.register_component("storage", storage_service, ComponentScope::Singleton);

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        if let Some(storage) = &self.storage_service {
            storage.initialize().await?;
        }
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        if let Some(storage) = &self.storage_service {
            storage.shutdown().await?;
        }
        Ok(())
    }
}

/// API Plugin provides HTTP API capabilities to the application
pub struct ApiPlugin {
    initialized: Mutex<bool>,
}

impl ApiPlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
        }
    }
}

#[async_trait]
impl Plugin for ApiPlugin {
    fn name(&self) -> &str {
        "api"
    }

    fn description(&self) -> &str {
        "Provides HTTP API capabilities to the application"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["logging", "config"] // Depends on logging and config
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering ApiPlugin");

        // API setup would typically happen here
        // For this example, we just mark as initialized

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        info!("ApiPlugin started");
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        info!("ApiPlugin stopped");
        Ok(())
    }
}

//
// Application Plugins
//

/// User Plugin provides user management capabilities
pub struct UserPlugin {
    initialized: Mutex<bool>,
}

impl UserPlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
        }
    }
}

#[async_trait]
impl Plugin for UserPlugin {
    fn name(&self) -> &str {
        "user"
    }

    fn description(&self) -> &str {
        "Provides user management capabilities"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["storage", "api"] // Depends on storage and api
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering UserPlugin");

        // User plugin setup would typically happen here
        // For this example, we just mark as initialized

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        info!("UserPlugin started");
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        info!("UserPlugin stopped");
        Ok(())
    }
}

/// Analytics Plugin provides analytics capabilities
pub struct AnalyticsPlugin {
    initialized: Mutex<bool>,
}

impl AnalyticsPlugin {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
        }
    }
}

#[async_trait]
impl Plugin for AnalyticsPlugin {
    fn name(&self) -> &str {
        "analytics"
    }

    fn description(&self) -> &str {
        "Provides analytics capabilities"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["storage", "api", "user"] // Depends on storage, api, and user
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering AnalyticsPlugin");

        // Analytics plugin setup would typically happen here
        // For this example, we just mark as initialized

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        info!("AnalyticsPlugin started");
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        info!("AnalyticsPlugin stopped");
        Ok(())
    }
}

//! Dynamic Plugin Example
//!
//! This is an example of a dynamically loaded plugin for the Navius framework.

use async_trait::async_trait;
use navius_core::{
    di::{Application, ComponentScope},
    error::{Error, Result},
};
use navius_plugin::{
    Capability, LoggingCapability, Plugin, PluginContext, PluginMetadata, impl_capability, plugin,
};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// The dynamic plugin implementation
pub struct DynamicPlugin {
    name: String,
    version: String,
    description: String,
    initialized: Mutex<bool>,
    logger: Option<Arc<dyn LoggingCapability>>,
}

impl DynamicPlugin {
    /// Create a new dynamic plugin
    pub fn new() -> Self {
        Self {
            name: "dynamic".to_string(),
            version: "0.1.0".to_string(),
            description: "A dynamically loaded plugin".to_string(),
            initialized: Mutex::new(false),
            logger: None,
        }
    }
}

#[async_trait]
impl Plugin for DynamicPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["logging"] // Depends on logging
    }

    async fn on_register(&self, app: &mut dyn Application, _: &PluginContext) -> Result<()> {
        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        info!("Registering DynamicPlugin");

        // Get dependencies
        let logger = app.get_component::<Arc<dyn LoggingCapability>>("logger")?;

        // Store a reference
        let dynamic_plugin = unsafe { &mut *(self as *const Self as *mut Self) };
        dynamic_plugin.logger = Some(logger.clone());

        // Create a capability provided by this plugin
        let dynamic_capability = Arc::new(DynamicCapability::new("Dynamic capability"));

        // Register with the application
        app.register_component(
            "dynamic_capability",
            dynamic_capability,
            ComponentScope::Singleton,
        );

        *initialized = true;
        Ok(())
    }

    async fn on_start(&self, _: &PluginContext) -> Result<()> {
        if let Some(logger) = &self.logger {
            logger.info("DynamicPlugin started");
        }
        Ok(())
    }

    async fn on_stop(&self, _: &PluginContext) -> Result<()> {
        if let Some(logger) = &self.logger {
            logger.info("DynamicPlugin stopped");
        }
        Ok(())
    }
}

/// A capability provided by the dynamic plugin
pub struct DynamicCapability {
    name: String,
}

impl DynamicCapability {
    /// Create a new dynamic capability
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Get the capability name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Do something with the capability
    pub fn perform_action(&self, action: &str) -> Result<String> {
        Ok(format!(
            "Performed action '{}' with capability '{}'",
            action, self.name
        ))
    }
}

impl_capability!(DynamicCapability, "dynamic_capability");

/// Required export function to create the plugin
/// This is the entry point used by the plugin loader
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = DynamicPlugin::new();
    Box::into_raw(Box::new(plugin))
}

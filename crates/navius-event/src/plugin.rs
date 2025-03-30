//! Plugin integration for the event system

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use navius_plugin::error::{IntoPluginError, PluginResult};
use navius_plugin::{ComponentRegistry, Plugin};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::bus::EventBus;
use crate::error::{EventError, EventResult};
use crate::handler::EventHandler;

/// Plugin for registering event handlers with the event bus.
pub struct EventPlugin {
    /// Plugin ID
    id: String,
    /// Plugin description
    description: String,
    /// Event bus instance.
    event_bus: EventBus,
    /// Plug state
    state: Arc<Mutex<PluginState>>,
}

/// Stores mutable plugin state
struct PluginState {
    /// Whether the plugin has been initialized.
    initialized: bool,
    /// Registered handler IDs for cleanup during shutdown.
    handler_ids: Vec<Uuid>,
}

impl EventPlugin {
    /// Creates a new event plugin with the given event bus.
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            id: "event".to_string(),
            description: "Event system plugin for publishing and subscribing to events".to_string(),
            event_bus,
            state: Arc::new(Mutex::new(PluginState {
                initialized: false,
                handler_ids: Vec::new(),
            })),
        }
    }

    /// Creates a new event plugin with a default event bus.
    pub fn default() -> Self {
        Self::new(EventBus::new())
    }

    /// Returns a reference to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}

#[async_trait]
impl Plugin for EventPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    #[instrument(skip(self, registry))]
    async fn initialize(&self, registry: &ComponentRegistry) -> PluginResult<()> {
        {
            let mut state = self.state.lock().unwrap();
            if state.initialized {
                return Ok(());
            }
        }

        info!("Initializing event plugin");

        // Register the event bus in the component registry
        registry
            .register_instance(None, self.event_bus.clone())
            .map_err(|e| e.into_plugin_error("Failed to register event bus"))?;

        // Mark as initialized
        {
            let mut state = self.state.lock().unwrap();
            state.initialized = true;
        }

        debug!("Event plugin initialized successfully");
        Ok(())
    }

    #[instrument(skip(self, _registry))]
    async fn shutdown(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        {
            let state = self.state.lock().unwrap();
            if !state.initialized {
                return Ok(());
            }
        }

        info!("Shutting down event plugin");

        // Unregister all handlers
        let handler_ids = {
            let mut state = self.state.lock().unwrap();
            std::mem::take(&mut state.handler_ids)
        };

        for id in handler_ids {
            if let Err(err) = self.event_bus.unsubscribe(id).await {
                debug!(
                    error = %err,
                    handler_id = %id,
                    "Failed to unsubscribe handler during shutdown"
                );
            }
        }

        // Shutdown the event bus
        if let Err(err) = self.event_bus.shutdown().await {
            debug!(
                error = %err,
                "Failed to shutdown event bus"
            );
        }

        // Mark as not initialized
        {
            let mut state = self.state.lock().unwrap();
            state.initialized = false;
        }

        debug!("Event plugin shutdown successfully");
        Ok(())
    }
}

/// Extension trait for using the EventPlugin.
#[async_trait]
pub trait EventPluginExt {
    /// Registers an event handler with the event plugin.
    async fn register_handler<H: EventHandler>(&self, handler: H) -> EventResult<uuid::Uuid>;
}

#[async_trait]
impl EventPluginExt for EventPlugin {
    #[instrument(skip(self, handler))]
    async fn register_handler<H: EventHandler>(&self, handler: H) -> EventResult<uuid::Uuid> {
        debug!("Registering event handler");

        // Subscribe the handler to the event bus
        let handler_id = self.event_bus.subscribe(handler).await?;

        // Store the handler ID for cleanup during shutdown
        {
            let mut state = self.state.lock().unwrap();
            state.handler_ids.push(handler_id);
        }

        Ok(handler_id)
    }
}

//! Event System Integration Example Library
//!
//! This crate demonstrates the integration of event system components
//! in the Navius framework, showcasing event-driven architecture patterns.

pub mod api;
pub mod broker;
pub mod events;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

use async_trait::async_trait;
use navius_core::{
    di::{Application, ApplicationBuilder, ComponentScope, Environment},
    error::Result,
    plugin::{Plugin, PluginContext},
};
use navius_db::connection::DbConfig;
use navius_db_postgres::connection::PostgresConnectionManager;
use std::sync::Arc;
use tracing::info;

use crate::api::AppServer;
use crate::events::{OrderCreatedEvent, OrderShippedEvent, SystemAlertEvent};
use crate::handlers::{AnalyticsHandler, AuditLogHandler, InventoryHandler, NotificationHandler};
use crate::models::{AlertLevel, OrderItem};
use crate::repository::EventRepository;
use crate::services::EventBusService;

/// EventSystemPlugin provides a way to add event system capabilities
/// to any Navius application.
pub struct EventSystemPlugin {
    db_url: String,
}

impl EventSystemPlugin {
    /// Create a new EventSystemPlugin with the specified database URL
    pub fn new(db_url: &str) -> Self {
        Self {
            db_url: db_url.to_string(),
        }
    }
}

#[async_trait]
impl Plugin for EventSystemPlugin {
    async fn on_register(&self, app: &mut dyn Application, _context: &PluginContext) -> Result<()> {
        info!("Registering EventSystemPlugin");

        // Configure database
        let db_config = DbConfig::new(&self.db_url);
        let pg_connection_manager = PostgresConnectionManager::new(db_config);

        // Create event repository
        let event_repository = Arc::new(EventRepository::new(pg_connection_manager.clone()));

        // Initialize database schema
        event_repository.initialize_schema().await?;

        // Create event bus
        let event_bus = EventBusService::new();

        // Create event handlers
        let notification_handler = Arc::new(NotificationHandler::new("EmailNotifier"));
        let analytics_handler = Arc::new(AnalyticsHandler::new("EventAnalytics"));
        let audit_log_handler = Arc::new(AuditLogHandler::new(
            "DatabaseAuditLog",
            event_repository.clone(),
        ));
        let inventory_handler =
            Arc::new(InventoryHandler::new("ProductInventory", event_bus.clone()));

        // Register event handlers with event bus
        event_bus.subscribe("OrderCreated", notification_handler.clone())?;
        event_bus.subscribe("OrderShipped", notification_handler.clone())?;
        event_bus.subscribe("SystemAlert", notification_handler.clone())?;

        event_bus.subscribe("OrderCreated", analytics_handler.clone())?;
        event_bus.subscribe("OrderShipped", analytics_handler.clone())?;
        event_bus.subscribe("InventoryUpdated", analytics_handler.clone())?;
        event_bus.subscribe("SystemAlert", analytics_handler.clone())?;

        event_bus.subscribe("OrderCreated", audit_log_handler.clone())?;
        event_bus.subscribe("OrderShipped", audit_log_handler.clone())?;
        event_bus.subscribe("InventoryUpdated", audit_log_handler.clone())?;
        event_bus.subscribe("SystemAlert", audit_log_handler.clone())?;

        event_bus.subscribe("OrderCreated", inventory_handler.clone())?;

        // Create and register the AppServer
        let mut app_server = AppServer::new(
            event_bus.clone(),
            event_repository.clone(),
            analytics_handler.clone(),
            inventory_handler.clone(),
        );
        app_server.configure_routes()?;

        // Register components with the application
        app.register_component("eventBus", event_bus.clone(), ComponentScope::Singleton);
        app.register_component(
            "eventRepository",
            event_repository,
            ComponentScope::Singleton,
        );
        app.register_component(
            "analyticsHandler",
            analytics_handler,
            ComponentScope::Singleton,
        );
        app.register_component(
            "inventoryHandler",
            inventory_handler,
            ComponentScope::Singleton,
        );
        app.register_component("appServer", app_server, ComponentScope::Singleton);

        info!("EventSystemPlugin registered successfully");
        Ok(())
    }

    fn name(&self) -> &str {
        "EventSystemPlugin"
    }

    fn description(&self) -> &str {
        "Provides event system capabilities for Navius applications"
    }
}

/// Create and configure a standalone event system application
pub async fn create_event_system_app(db_url: &str) -> Result<ApplicationBuilder> {
    let plugin = EventSystemPlugin::new(db_url);

    let mut app_builder = ApplicationBuilder::new().with_environment(Environment::Development);

    app_builder.register_plugin(Box::new(plugin)).await?;

    Ok(app_builder)
}

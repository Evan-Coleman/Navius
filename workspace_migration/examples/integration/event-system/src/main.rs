//! Event System Integration Example
//!
//! This example demonstrates the integration of event system components
//! in the Navius framework, showcasing event-driven architecture patterns.

mod api;
mod events;
mod handlers;
mod models;
mod repository;
mod services;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use navius_core::{
    config::Config,
    di::{Application, ApplicationBuilder, AsyncLifecycle, ComponentScope, Environment, Lifecycle},
    error::{Error, Result},
};
use navius_db::{
    connection::{ConnectionManager, DbConfig},
    repository::Repository,
    transaction::{Transaction, TransactionManager},
};
use navius_db_postgres::{connection::PostgresConnectionManager, query::PostgresQuery};
use navius_event::{Event, EventBus, EventHandler, EventPublisher, EventSubscriber};
use navius_http::{
    response::Response,
    routing::{Route, Router},
    server::{HttpServer, ServerConfig},
};
use navius_plugin::{Plugin, PluginRegistry};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::AppServer;
use crate::events::{OrderCreatedEvent, OrderShippedEvent, SystemAlertEvent};
use crate::handlers::{AnalyticsHandler, AuditLogHandler, InventoryHandler, NotificationHandler};
use crate::models::{AlertLevel, OrderItem};
use crate::repository::EventRepository;
use crate::services::EventBusService;

// ====================== Event Definitions ======================

/// Base trait for all domain events
#[async_trait]
trait DomainEvent: Send + Sync {
    fn event_type(&self) -> String;
    fn event_id(&self) -> Uuid;
    fn timestamp(&self) -> DateTime<Utc>;
    fn to_json(&self) -> Result<String>;
}

/// Base event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventMetadata {
    event_id: Uuid,
    event_type: String,
    timestamp: DateTime<Utc>,
    source: String,
}

// Order Created Event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderCreatedEvent {
    metadata: EventMetadata,
    order_id: Uuid,
    customer_id: String,
    items: Vec<OrderItem>,
    total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderItem {
    product_id: String,
    quantity: i32,
    price: f64,
}

impl OrderCreatedEvent {
    fn new(order_id: Uuid, customer_id: &str, items: Vec<OrderItem>, total_amount: f64) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "OrderCreated".to_string(),
                timestamp: Utc::now(),
                source: "OrderService".to_string(),
            },
            order_id,
            customer_id: customer_id.to_string(),
            items,
            total_amount,
        }
    }
}

impl DomainEvent for OrderCreatedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

// Order Shipped Event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderShippedEvent {
    metadata: EventMetadata,
    order_id: Uuid,
    tracking_number: String,
    shipping_method: String,
    shipped_at: DateTime<Utc>,
    estimated_delivery: DateTime<Utc>,
}

impl OrderShippedEvent {
    fn new(order_id: Uuid, tracking_number: &str, shipping_method: &str) -> Self {
        let now = Utc::now();
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "OrderShipped".to_string(),
                timestamp: now,
                source: "ShippingService".to_string(),
            },
            order_id,
            tracking_number: tracking_number.to_string(),
            shipping_method: shipping_method.to_string(),
            shipped_at: now,
            estimated_delivery: now + chrono::Duration::days(3),
        }
    }
}

impl DomainEvent for OrderShippedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

// Inventory Updated Event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InventoryUpdatedEvent {
    metadata: EventMetadata,
    product_id: String,
    previous_quantity: i32,
    new_quantity: i32,
    warehouse_id: String,
    reason: String,
}

impl InventoryUpdatedEvent {
    fn new(
        product_id: &str,
        previous_quantity: i32,
        new_quantity: i32,
        warehouse_id: &str,
        reason: &str,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "InventoryUpdated".to_string(),
                timestamp: Utc::now(),
                source: "InventoryService".to_string(),
            },
            product_id: product_id.to_string(),
            previous_quantity,
            new_quantity,
            warehouse_id: warehouse_id.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl DomainEvent for InventoryUpdatedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

// System Alert Event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemAlertEvent {
    metadata: EventMetadata,
    alert_level: AlertLevel,
    message: String,
    service: String,
    requires_action: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl SystemAlertEvent {
    fn new(alert_level: AlertLevel, message: &str, service: &str, requires_action: bool) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "SystemAlert".to_string(),
                timestamp: Utc::now(),
                source: "MonitoringService".to_string(),
            },
            alert_level,
            message: message.to_string(),
            service: service.to_string(),
            requires_action,
        }
    }
}

impl DomainEvent for SystemAlertEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

// ====================== Event Bus Implementation ======================

// Simple in-memory event bus implementation
#[derive(Clone)]
struct InMemoryEventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Arc<Mutex<Vec<EventRecord>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EventRecord {
    id: Uuid,
    event_type: String,
    payload: String,
    timestamp: DateTime<Utc>,
    source: String,
}

impl InMemoryEventBus {
    fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            event_store: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_event_history(&self) -> Vec<EventRecord> {
        let store = self.event_store.lock().unwrap();
        store.clone()
    }

    fn record_event(&self, event_type: &str, payload: &str, source: &str) -> Result<()> {
        let record = EventRecord {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
        };

        let mut store = self.event_store.lock().unwrap();
        store.push(record);
        Ok(())
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish<E>(&self, event: E) -> Result<()>
    where
        E: DomainEvent + Send + Sync + 'static,
    {
        let event_type = event.event_type();
        let event_json = event.to_json()?;

        // Store the event for history
        self.record_event(&event_type, &event_json, "system")?;

        // Execute all handlers for this event type
        let handlers = {
            let handlers_map = self.handlers.lock().unwrap();
            handlers_map
                .get(&event_type)
                .cloned()
                .unwrap_or_else(Vec::new)
        };

        info!(
            "Publishing event: {} to {} handlers",
            event_type,
            handlers.len()
        );

        for handler in handlers {
            if let Err(e) = handler.handle(&event_type, &event_json).await {
                error!("Error handling event {}: {}", event_type, e);
                // Continue processing other handlers despite errors
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventSubscriber for InMemoryEventBus {
    fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers_map = self.handlers.lock().unwrap();
        let handlers = handlers_map
            .entry(event_type.to_string())
            .or_insert_with(Vec::new);

        handlers.push(handler);
        info!("Subscribed handler to event type: {}", event_type);

        Ok(())
    }

    fn unsubscribe(&self, event_type: &str, handler_id: &str) -> Result<()> {
        let mut handlers_map = self.handlers.lock().unwrap();

        if let Some(handlers) = handlers_map.get_mut(event_type) {
            // Filter out the handler with the matching ID
            // In a real implementation, handler would have a unique ID
            // For this example, we'll assume handler_id is not used
            info!("Unsubscribed handler from event type: {}", event_type);
        }

        Ok(())
    }
}

// ====================== Event Handlers ======================

// Notification Handler - sends notifications for events
struct NotificationHandler {
    name: String,
}

impl NotificationHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for NotificationHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        info!("[{}] Notification for event: {}", self.name, event_type);

        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                info!(
                    "[{}] Order created notification: Order #{} with {} items for customer {}",
                    self.name,
                    event.order_id,
                    event.items.len(),
                    event.customer_id
                );
            }
            "OrderShipped" => {
                let event: OrderShippedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderShippedEvent: {}", e))
                })?;

                info!(
                    "[{}] Order shipped notification: Order #{} shipped via {} with tracking number {}",
                    self.name, event.order_id, event.shipping_method, event.tracking_number
                );
            }
            "SystemAlert" => {
                let event: SystemAlertEvent = serde_json::from_str(event_data)
                    .map_err(|e| Error::new(&format!("Failed to parse SystemAlertEvent: {}", e)))?;

                let level = match event.alert_level {
                    AlertLevel::Info => "INFO",
                    AlertLevel::Warning => "WARNING",
                    AlertLevel::Error => "ERROR",
                    AlertLevel::Critical => "CRITICAL",
                };

                info!(
                    "[{}] System alert notification: [{}] {} - Service: {}",
                    self.name, level, event.message, event.service
                );
            }
            _ => {
                info!(
                    "[{}] Generic notification for event: {}",
                    self.name, event_type
                );
            }
        }

        Ok(())
    }
}

// Analytics Handler - records metrics about events
struct AnalyticsHandler {
    name: String,
    metrics: Arc<Mutex<HashMap<String, i32>>>,
}

impl AnalyticsHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_metrics(&self) -> HashMap<String, i32> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

#[async_trait]
impl EventHandler for AnalyticsHandler {
    async fn handle(&self, event_type: &str, _event_data: &str) -> Result<()> {
        let mut metrics = self.metrics.lock().unwrap();
        let count = metrics.entry(event_type.to_string()).or_insert(0);
        *count += 1;

        info!(
            "[{}] Analytics recorded for event: {} (count: {})",
            self.name, event_type, *count
        );

        Ok(())
    }
}

// Inventory Handler - updates inventory based on order events
struct InventoryHandler {
    name: String,
    inventory: Arc<Mutex<HashMap<String, i32>>>,
    event_bus: InMemoryEventBus,
}

impl InventoryHandler {
    fn new(name: &str, event_bus: InMemoryEventBus) -> Self {
        // Initialize with some sample inventory
        let mut inventory = HashMap::new();
        inventory.insert("PROD-001".to_string(), 100);
        inventory.insert("PROD-002".to_string(), 50);
        inventory.insert("PROD-003".to_string(), 75);

        Self {
            name: name.to_string(),
            inventory: Arc::new(Mutex::new(inventory)),
            event_bus,
        }
    }

    fn get_inventory(&self) -> HashMap<String, i32> {
        let inventory = self.inventory.lock().unwrap();
        inventory.clone()
    }
}

#[async_trait]
impl EventHandler for InventoryHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                info!(
                    "[{}] Processing inventory updates for order: {}",
                    self.name, event.order_id
                );

                let mut inventory = self.inventory.lock().unwrap();

                // Update inventory for each item in the order
                for item in &event.items {
                    if let Some(current_quantity) = inventory.get_mut(&item.product_id) {
                        let previous_quantity = *current_quantity;
                        *current_quantity = (*current_quantity - item.quantity).max(0);

                        info!(
                            "[{}] Updated inventory for product {}: {} -> {}",
                            self.name, item.product_id, previous_quantity, *current_quantity
                        );

                        // Publish inventory updated event
                        let inventory_event = InventoryUpdatedEvent::new(
                            &item.product_id,
                            previous_quantity,
                            *current_quantity,
                            "WAREHOUSE-001",
                            &format!("Order: {}", event.order_id),
                        );

                        // Drop the mutex before publishing to avoid deadlocks
                        drop(inventory);

                        if let Err(e) = self.event_bus.publish(inventory_event).await {
                            error!(
                                "[{}] Failed to publish inventory update event: {}",
                                self.name, e
                            );
                        }

                        // Re-acquire the mutex
                        inventory = self.inventory.lock().unwrap();
                    } else {
                        warn!(
                            "[{}] Product not found in inventory: {}",
                            self.name, item.product_id
                        );
                    }
                }

                // Check for low inventory and generate alerts if needed
                for (product_id, quantity) in inventory.iter() {
                    if *quantity < 10 {
                        // Generate a system alert for low inventory
                        let alert = SystemAlertEvent::new(
                            AlertLevel::Warning,
                            &format!(
                                "Low inventory for product {}: {} units remaining",
                                product_id, quantity
                            ),
                            "InventoryService",
                            true,
                        );

                        // Drop the mutex before publishing
                        drop(inventory);

                        if let Err(e) = self.event_bus.publish(alert).await {
                            error!(
                                "[{}] Failed to publish low inventory alert: {}",
                                self.name, e
                            );
                        }

                        break; // To avoid multiple alerts, we'll just trigger one for now
                    }
                }
            }
            _ => {
                // Other event types not handled by this handler
            }
        }

        Ok(())
    }
}

// ====================== Event Persistence ======================

// Event Repository for persisting events to the database
struct EventRepository {
    connection_manager: PostgresConnectionManager,
}

impl EventRepository {
    fn new(connection_manager: PostgresConnectionManager) -> Self {
        Self { connection_manager }
    }

    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection_manager.get_connection().await?;

        // Create events table
        let create_table_query = PostgresQuery::new(
            "CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                event_type VARCHAR(255) NOT NULL,
                payload TEXT NOT NULL,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                source VARCHAR(255) NOT NULL
            )",
        );

        create_table_query.execute_update(&conn, &[]).await?;
        info!("Event database schema initialized successfully");

        Ok(())
    }

    async fn save_event(&self, record: &EventRecord) -> Result<()> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "INSERT INTO events (id, event_type, payload, timestamp, source) 
             VALUES ($1, $2, $3, $4, $5)",
        );

        query
            .execute_update(
                &conn,
                &[
                    &record.id,
                    &record.event_type,
                    &record.payload,
                    &record.timestamp,
                    &record.source,
                ],
            )
            .await?;

        Ok(())
    }

    async fn get_events(&self, limit: i32) -> Result<Vec<EventRecord>> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "SELECT id, event_type, payload, timestamp, source 
             FROM events 
             ORDER BY timestamp DESC 
             LIMIT $1",
        );

        let rows = query.execute_query(&conn, &[&limit]).await?;

        let events = rows
            .into_iter()
            .map(|row| EventRecord {
                id: row.get("id"),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                timestamp: row.get("timestamp"),
                source: row.get("source"),
            })
            .collect();

        Ok(events)
    }

    async fn get_events_by_type(&self, event_type: &str, limit: i32) -> Result<Vec<EventRecord>> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "SELECT id, event_type, payload, timestamp, source 
             FROM events 
             WHERE event_type = $1
             ORDER BY timestamp DESC 
             LIMIT $2",
        );

        let rows = query.execute_query(&conn, &[&event_type, &limit]).await?;

        let events = rows
            .into_iter()
            .map(|row| EventRecord {
                id: row.get("id"),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                timestamp: row.get("timestamp"),
                source: row.get("source"),
            })
            .collect();

        Ok(events)
    }
}

// Audit Log Handler - persists events to the database
struct AuditLogHandler {
    name: String,
    event_repository: Arc<EventRepository>,
}

impl AuditLogHandler {
    fn new(name: &str, event_repository: Arc<EventRepository>) -> Self {
        Self {
            name: name.to_string(),
            event_repository,
        }
    }
}

#[async_trait]
impl EventHandler for AuditLogHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        info!(
            "[{}] Persisting event to audit log: {}",
            self.name, event_type
        );

        let record = EventRecord {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload: event_data.to_string(),
            timestamp: Utc::now(),
            source: "AuditLog".to_string(),
        };

        if let Err(e) = self.event_repository.save_event(&record).await {
            error!("[{}] Failed to persist event to database: {}", self.name, e);
            return Err(e);
        }

        info!("[{}] Event successfully persisted to audit log", self.name);
        Ok(())
    }
}

// ====================== HTTP Integration ======================

// AppServer integrates all components with HTTP endpoints
struct AppServer {
    server: HttpServer,
    event_bus: InMemoryEventBus,
    event_repository: Arc<EventRepository>,
    analytics_handler: Arc<AnalyticsHandler>,
    inventory_handler: Arc<InventoryHandler>,
}

impl AppServer {
    fn new(
        event_bus: InMemoryEventBus,
        event_repository: Arc<EventRepository>,
        analytics_handler: Arc<AnalyticsHandler>,
        inventory_handler: Arc<InventoryHandler>,
    ) -> Self {
        let config = ServerConfig::default().with_address("127.0.0.1:8080".to_string());
        let server = HttpServer::new(config);

        Self {
            server,
            event_bus,
            event_repository,
            analytics_handler,
            inventory_handler,
        }
    }

    fn configure_routes(&mut self) -> Result<()> {
        let mut router = Router::new();

        // Health check endpoint
        router.add_route(Route::get("/health", |_req| {
            Ok(format!(
                "{{\"status\": \"OK\", \"timestamp\": \"{}\"}}",
                Utc::now()
            ))
        }));

        // Trigger sample events endpoint
        let event_bus = self.event_bus.clone();
        router.add_route(Route::post("/api/events/trigger/:type", move |req| {
            let event_bus = event_bus.clone();

            async move {
                let event_type = req
                    .param("type")
                    .ok_or_else(|| Error::new("Missing event type"))?;

                match event_type {
                    "order-created" => {
                        // Create a sample order
                        let items = vec![
                            OrderItem {
                                product_id: "PROD-001".to_string(),
                                quantity: 2,
                                price: 29.99,
                            },
                            OrderItem {
                                product_id: "PROD-002".to_string(),
                                quantity: 1,
                                price: 49.99,
                            },
                        ];

                        let total = items
                            .iter()
                            .map(|item| item.price * item.quantity as f64)
                            .sum();
                        let event =
                            OrderCreatedEvent::new(Uuid::new_v4(), "CUST-123", items, total);

                        event_bus.publish(event).await?;
                        Ok("Order created event triggered successfully".to_string())
                    }
                    "order-shipped" => {
                        // Create a sample shipping event
                        let event =
                            OrderShippedEvent::new(Uuid::new_v4(), "TRK12345678", "Express");

                        event_bus.publish(event).await?;
                        Ok("Order shipped event triggered successfully".to_string())
                    }
                    "system-alert" => {
                        // Create a sample system alert
                        let event = SystemAlertEvent::new(
                            AlertLevel::Warning,
                            "Disk usage above 80%",
                            "StorageService",
                            true,
                        );

                        event_bus.publish(event).await?;
                        Ok("System alert event triggered successfully".to_string())
                    }
                    _ => Err(Error::new(&format!(
                        "Unsupported event type: {}",
                        event_type
                    ))),
                }
            }
        }));

        // Event history endpoint
        let event_repository = self.event_repository.clone();
        router.add_route(Route::get("/api/events/history", move |req| {
            let event_repository = event_repository.clone();

            async move {
                // Get optional query parameters
                let limit = req
                    .query("limit")
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(50);

                let event_type = req.query("type");

                let events = if let Some(event_type) = event_type {
                    event_repository
                        .get_events_by_type(event_type, limit)
                        .await?
                } else {
                    event_repository.get_events(limit).await?
                };

                Ok(serde_json::to_string(&events)?)
            }
        }));

        // Event metrics endpoint
        let analytics_handler = self.analytics_handler.clone();
        router.add_route(Route::get("/api/events/metrics", move |_req| {
            let metrics = analytics_handler.get_metrics();
            Ok(serde_json::to_string(&metrics).unwrap())
        }));

        // Inventory status endpoint
        let inventory_handler = self.inventory_handler.clone();
        router.add_route(Route::get("/api/inventory", move |_req| {
            let inventory = inventory_handler.get_inventory();
            Ok(serde_json::to_string(&inventory).unwrap())
        }));

        // SSE endpoint for event streaming
        let event_bus = self.event_bus.clone();
        router.add_route(Route::get("/api/events/stream", move |_req| {
            let event_bus = event_bus.clone();

            async move {
                // In a real implementation, this would set up an SSE connection
                // and stream events in real-time. For this example, we'll just return
                // the recent events.
                let events = event_bus.get_event_history();

                // Set SSE headers
                let mut response = Response::new(200);
                response.set_header("Content-Type", "text/event-stream");
                response.set_header("Cache-Control", "no-cache");
                response.set_header("Connection", "keep-alive");

                // Format events as SSE data
                let mut body = String::new();
                for event in events {
                    body.push_str(&format!("event: {}\n", event.event_type));
                    body.push_str(&format!("id: {}\n", event.id));
                    body.push_str(&format!("data: {}\n\n", event.payload));
                }

                response.set_body(body);
                Ok(response)
            }
        }));

        self.server.set_router(router);
        Ok(())
    }
}

#[async_trait]
impl AsyncLifecycle for AppServer {
    async fn on_initialize_async(&self) -> Result<()> {
        info!("Starting AppServer");
        self.server.start().await?;
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        info!("Stopping AppServer");
        self.server.stop().await?;
        Ok(())
    }
}

// ====================== Main Application ======================

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();

    info!("Starting Event System Integration Example");

    // Create the application
    let mut app = ApplicationBuilder::new()
        .with_environment(Environment::Development)
        .build();

    // Configure database
    let db_config = DbConfig::new("postgres://navius:navius_password@localhost:5433/navius_events");
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
    let inventory_handler = Arc::new(InventoryHandler::new("ProductInventory", event_bus.clone()));

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
        event_repository,
        analytics_handler,
        inventory_handler,
    );
    app_server.configure_routes()?;

    app.register_component("appServer", app_server, ComponentScope::Singleton);

    // Start the application
    app.start().await?;

    // Generate some sample events to demonstrate functionality
    info!("Generating sample events to demonstrate functionality...");

    // Create a sample order
    let items = vec![
        OrderItem {
            product_id: "PROD-001".to_string(),
            quantity: 2,
            price: 29.99,
        },
        OrderItem {
            product_id: "PROD-002".to_string(),
            quantity: 1,
            price: 49.99,
        },
    ];

    let total = items
        .iter()
        .map(|item| item.price * item.quantity as f64)
        .sum();
    let order_event = OrderCreatedEvent::new(uuid::Uuid::new_v4(), "CUST-123", items, total);

    event_bus.publish(order_event).await?;

    // Create a sample shipping event
    let shipping_event = OrderShippedEvent::new(uuid::Uuid::new_v4(), "TRK12345678", "Express");

    event_bus.publish(shipping_event).await?;

    // Create a sample system alert
    let alert_event = SystemAlertEvent::new(
        AlertLevel::Warning,
        "Disk usage above 80%",
        "StorageService",
        true,
    );

    event_bus.publish(alert_event).await?;

    // Wait for user to terminate
    info!("Application started successfully. Press Ctrl+C to stop.");
    info!("Server running at http://127.0.0.1:8080");
    info!("Try the following endpoints:");
    info!("- GET /health - Health check");
    info!("- POST /api/events/trigger/order-created - Trigger an order created event");
    info!("- POST /api/events/trigger/order-shipped - Trigger an order shipped event");
    info!("- POST /api/events/trigger/system-alert - Trigger a system alert event");
    info!("- GET /api/events/history - View event history");
    info!("- GET /api/events/metrics - View event metrics");
    info!("- GET /api/inventory - View inventory status");
    info!("- GET /api/events/stream - Stream events (SSE)");

    // Keep the application running
    tokio::signal::ctrl_c().await?;

    // Shutdown the application
    info!("Shutting down the application...");
    app.stop().await?;

    info!("Application shutdown complete");
    Ok(())
}

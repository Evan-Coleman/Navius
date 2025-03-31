//! Integration tests for the Event System example

use chrono::Utc;
use navius_core::{
    di::{ApplicationBuilder, ComponentScope, Environment},
    error::Result,
};
use navius_db::connection::DbConfig;
use navius_db_postgres::connection::PostgresConnectionManager;
use navius_event::EventBus;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::sleep;
use uuid::Uuid;

use event_system::{
    events::{DomainEvent, OrderCreatedEvent, SystemAlertEvent},
    handlers::{AnalyticsHandler, InventoryHandler, NotificationHandler},
    models::{AlertLevel, OrderItem},
    repository::EventRepository,
    services::EventBusService,
};

#[tokio::test]
async fn test_event_bus_publish_subscribe() -> Result<()> {
    // Create event bus
    let event_bus = EventBusService::new();

    // Create handlers
    let notification_handler = Arc::new(NotificationHandler::new("TestNotifier"));
    let analytics_handler = Arc::new(AnalyticsHandler::new("TestAnalytics"));

    // Subscribe handlers
    event_bus.subscribe("OrderCreated", notification_handler.clone())?;
    event_bus.subscribe("OrderCreated", analytics_handler.clone())?;

    // Create and publish an event
    let items = vec![OrderItem {
        product_id: "TEST-001".to_string(),
        quantity: 1,
        price: 9.99,
    }];

    let event = OrderCreatedEvent::new(Uuid::new_v4(), "CUST-TEST", items, 9.99);

    event_bus.publish(event).await?;

    // Wait a moment for async processing
    sleep(Duration::from_millis(100)).await;

    // Verify analytics handler received the event
    let metrics = analytics_handler.get_metrics();
    assert_eq!(metrics.get("OrderCreated"), Some(&1));

    Ok(())
}

#[tokio::test]
async fn test_inventory_updates_on_order() -> Result<()> {
    // Create event bus
    let event_bus = EventBusService::new();

    // Create inventory handler
    let inventory_handler = Arc::new(InventoryHandler::new("TestInventory", event_bus.clone()));

    // Subscribe handler
    event_bus.subscribe("OrderCreated", inventory_handler.clone())?;

    // Get initial inventory
    let initial_inventory = inventory_handler.get_inventory();
    let initial_prod1_qty = *initial_inventory.get("PROD-001").unwrap();

    // Create and publish an order event for product PROD-001
    let items = vec![OrderItem {
        product_id: "PROD-001".to_string(),
        quantity: 5,
        price: 29.99,
    }];

    let event = OrderCreatedEvent::new(Uuid::new_v4(), "CUST-TEST", items, 29.99 * 5.0);

    event_bus.publish(event).await?;

    // Wait a moment for async processing
    sleep(Duration::from_millis(100)).await;

    // Verify inventory was updated
    let updated_inventory = inventory_handler.get_inventory();
    let updated_prod1_qty = *updated_inventory.get("PROD-001").unwrap();

    assert_eq!(updated_prod1_qty, initial_prod1_qty - 5);

    Ok(())
}

#[tokio::test]
async fn test_system_alert_event() -> Result<()> {
    // Create event bus
    let event_bus = EventBusService::new();

    // Create handlers
    let notification_handler = Arc::new(NotificationHandler::new("TestNotifier"));
    let analytics_handler = Arc::new(AnalyticsHandler::new("TestAnalytics"));

    // Subscribe handlers
    event_bus.subscribe("SystemAlert", notification_handler.clone())?;
    event_bus.subscribe("SystemAlert", analytics_handler.clone())?;

    // Create and publish a system alert
    let alert = SystemAlertEvent::new(
        AlertLevel::Critical,
        "Test critical alert",
        "TestService",
        true,
    );

    event_bus.publish(alert).await?;

    // Wait a moment for async processing
    sleep(Duration::from_millis(100)).await;

    // Verify analytics handler received the event
    let metrics = analytics_handler.get_metrics();
    assert_eq!(metrics.get("SystemAlert"), Some(&1));

    Ok(())
}

#[tokio::test]
#[ignore] // Requires database, so ignore by default
async fn test_event_repository() -> Result<()> {
    // Set up test database connection
    let db_config =
        DbConfig::new("postgres://navius:navius_password@localhost:5433/navius_events_test");
    let pg_connection_manager = PostgresConnectionManager::new(db_config);

    // Create event repository
    let event_repository = Arc::new(EventRepository::new(pg_connection_manager.clone()));

    // Initialize schema
    event_repository.initialize_schema().await?;

    // Create a test event record
    let test_event = event_system::models::EventRecord {
        id: Uuid::new_v4(),
        event_type: "TestEvent".to_string(),
        payload: r#"{"test":"value"}"#.to_string(),
        timestamp: Utc::now(),
        source: "TestSource".to_string(),
    };

    // Save the event
    event_repository.save_event(&test_event).await?;

    // Retrieve events
    let events = event_repository.get_events(10).await?;

    // Verify the event was saved
    assert!(!events.is_empty());

    // Retrieve events by type
    let typed_events = event_repository.get_events_by_type("TestEvent", 10).await?;

    // Verify the event was retrieved
    assert!(!typed_events.is_empty());
    assert_eq!(typed_events[0].event_type, "TestEvent");

    Ok(())
}

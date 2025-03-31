//! Real-Time Dashboard Example
//!
//! This example demonstrates how to implement a real-time monitoring dashboard
//! using the Event System integration, showcasing event aggregation, metrics,
//! and status tracking capabilities.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use event_system::{
    events::{DomainEvent, InventoryUpdatedEvent, OrderCreatedEvent, SystemAlertEvent},
    models::{AlertLevel, EventMetricsSummary},
    services::EventBusService,
};
use navius_core::{
    di::{ApplicationBuilder, Environment},
    error::{Error, Result},
};
use navius_event::EventHandler;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Dashboard status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum StatusIndicator {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// System component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComponentHealth {
    name: String,
    status: StatusIndicator,
    last_updated: DateTime<Utc>,
    message: Option<String>,
}

/// Dashboard metrics and status aggregator
struct DashboardAggregator {
    name: String,
    component_status: Arc<Mutex<HashMap<String, ComponentHealth>>>,
    event_counts: Arc<Mutex<HashMap<String, i32>>>,
    period_start: Arc<Mutex<DateTime<Utc>>>,
    alert_channel: broadcast::Sender<SystemAlertEvent>,
    recent_alerts: Arc<Mutex<Vec<SystemAlertEvent>>>,
}

impl DashboardAggregator {
    fn new(name: &str) -> Self {
        // Set up initial component health status
        let mut components = HashMap::new();
        components.insert(
            "OrderService".to_string(),
            ComponentHealth {
                name: "OrderService".to_string(),
                status: StatusIndicator::Unknown,
                last_updated: Utc::now(),
                message: Some("Initializing".to_string()),
            },
        );
        components.insert(
            "InventoryService".to_string(),
            ComponentHealth {
                name: "InventoryService".to_string(),
                status: StatusIndicator::Unknown,
                last_updated: Utc::now(),
                message: Some("Initializing".to_string()),
            },
        );
        components.insert(
            "NotificationService".to_string(),
            ComponentHealth {
                name: "NotificationService".to_string(),
                status: StatusIndicator::Unknown,
                last_updated: Utc::now(),
                message: Some("Initializing".to_string()),
            },
        );

        // Create alert broadcast channel (capacity 100)
        let (tx, _) = broadcast::channel(100);

        Self {
            name: name.to_string(),
            component_status: Arc::new(Mutex::new(components)),
            event_counts: Arc::new(Mutex::new(HashMap::new())),
            period_start: Arc::new(Mutex::new(Utc::now())),
            alert_channel: tx,
            recent_alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a snapshot of all component health statuses
    fn get_component_health(&self) -> HashMap<String, ComponentHealth> {
        let status = self.component_status.lock().unwrap();
        status.clone()
    }

    /// Get event metrics for the current period
    fn get_metrics(&self) -> EventMetricsSummary {
        let counts = self.event_counts.lock().unwrap();
        let period_start = *self.period_start.lock().unwrap();

        EventMetricsSummary {
            event_counts: counts.clone(),
            total_events: counts.values().sum(),
            period_start,
            period_end: Utc::now(),
        }
    }

    /// Get a subscriber to receive alerts in real-time
    fn subscribe_to_alerts(&self) -> broadcast::Receiver<SystemAlertEvent> {
        self.alert_channel.subscribe()
    }

    /// Get recent alerts
    fn get_recent_alerts(&self) -> Vec<SystemAlertEvent> {
        let alerts = self.recent_alerts.lock().unwrap();
        alerts.clone()
    }

    /// Reset metrics for a new period
    fn reset_metrics(&self) {
        let mut counts = self.event_counts.lock().unwrap();
        counts.clear();

        let mut period_start = self.period_start.lock().unwrap();
        *period_start = Utc::now();

        info!("[{}] Metrics reset for new period", self.name);
    }

    /// Update a component's health status
    fn update_component_status(
        &self,
        component: &str,
        status: StatusIndicator,
        message: Option<String>,
    ) {
        let mut components = self.component_status.lock().unwrap();

        components.insert(
            component.to_string(),
            ComponentHealth {
                name: component.to_string(),
                status,
                last_updated: Utc::now(),
                message,
            },
        );

        info!(
            "[{}] Component '{}' status updated to {:?}",
            self.name, component, status
        );
    }
}

#[async_trait]
impl EventHandler for DashboardAggregator {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        // Update event counts
        {
            let mut counts = self.event_counts.lock().unwrap();
            let count = counts.entry(event_type.to_string()).or_insert(0);
            *count += 1;
        }

        // Process specific event types
        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                // Update OrderService status to healthy
                self.update_component_status(
                    "OrderService",
                    StatusIndicator::Healthy,
                    Some(format!("Processing order #{}", event.order_id)),
                );

                debug!(
                    "[{}] Processed order created: {} with {} items",
                    self.name,
                    event.order_id,
                    event.items.len()
                );
            }
            "InventoryUpdated" => {
                let event: InventoryUpdatedEvent =
                    serde_json::from_str(event_data).map_err(|e| {
                        Error::new(&format!("Failed to parse InventoryUpdatedEvent: {}", e))
                    })?;

                // Update InventoryService status based on stock level
                let status = if event.new_quantity <= 0 {
                    StatusIndicator::Warning
                } else {
                    StatusIndicator::Healthy
                };

                self.update_component_status(
                    "InventoryService",
                    status,
                    Some(format!(
                        "Product {} inventory changed: {} → {}",
                        event.product_id, event.previous_quantity, event.new_quantity
                    )),
                );
            }
            "SystemAlert" => {
                let event: SystemAlertEvent = serde_json::from_str(event_data)
                    .map_err(|e| Error::new(&format!("Failed to parse SystemAlertEvent: {}", e)))?;

                // Update component status based on alert level
                let status = match event.alert_level {
                    AlertLevel::Info => StatusIndicator::Healthy,
                    AlertLevel::Warning => StatusIndicator::Warning,
                    AlertLevel::Error | AlertLevel::Critical => StatusIndicator::Critical,
                };

                self.update_component_status(&event.service, status, Some(event.message.clone()));

                // Store recent alert
                {
                    let mut alerts = self.recent_alerts.lock().unwrap();
                    alerts.push(event.clone());

                    // Keep only the 10 most recent alerts
                    if alerts.len() > 10 {
                        alerts.remove(0);
                    }
                }

                // Broadcast the alert to any subscribers
                let _ = self.alert_channel.send(event);
            }
            _ => {}
        }

        Ok(())
    }
}

/// Generate random test events for the dashboard
async fn generate_test_events(event_bus: EventBusService) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
    let start_time = Instant::now();

    // Run for 30 seconds
    while start_time.elapsed().as_secs() < 30 {
        interval.tick().await;

        // Randomly choose an event type to generate
        let event_type = rand::random::<u8>() % 3;

        match event_type {
            0 => {
                // Generate OrderCreated event
                let items = vec![event_system::models::OrderItem {
                    product_id: format!("PROD-{:03}", rand::random::<u8>() % 10),
                    quantity: (rand::random::<u8>() % 5) + 1,
                    price: 19.99 + (rand::random::<u8>() % 10) as f64,
                }];

                let total = items.iter().map(|i| i.price * i.quantity as f64).sum();
                let order_event = OrderCreatedEvent::new(
                    Uuid::new_v4(),
                    &format!("CUST-{:03}", rand::random::<u16>() % 100),
                    items,
                    total,
                );

                event_bus.publish(order_event).await?;
            }
            1 => {
                // Generate InventoryUpdated event
                let product_id = format!("PROD-{:03}", rand::random::<u8>() % 10);
                let prev_qty = (rand::random::<u8>() % 20) + 5;
                let new_qty = if rand::random::<bool>() {
                    prev_qty.saturating_sub(rand::random::<u8>() % 5)
                } else {
                    prev_qty + (rand::random::<u8>() % 5)
                };

                let inventory_event = InventoryUpdatedEvent::new(
                    &product_id,
                    prev_qty as i32,
                    new_qty as i32,
                    "WAREHOUSE-001",
                    "Inventory adjustment",
                );

                event_bus.publish(inventory_event).await?;
            }
            _ => {
                // Generate SystemAlert event
                let services = [
                    "OrderService",
                    "InventoryService",
                    "NotificationService",
                    "StorageService",
                ];
                let service = services[rand::random::<usize>() % services.len()];

                let level_num = rand::random::<u8>() % 10;
                let level = match level_num {
                    0..=6 => AlertLevel::Info,
                    7..=8 => AlertLevel::Warning,
                    _ => AlertLevel::Error,
                };

                let messages = match level {
                    AlertLevel::Info => [
                        "Service started successfully",
                        "Configuration reloaded",
                        "Cache cleared",
                        "Maintenance completed",
                    ],
                    AlertLevel::Warning => [
                        "High CPU usage detected",
                        "Memory usage above 80%",
                        "Slow response time detected",
                        "Rate limit approaching threshold",
                    ],
                    AlertLevel::Error => [
                        "Database connection failed",
                        "API endpoint timeout",
                        "Service crashed",
                        "Authentication service unavailable",
                    ],
                    AlertLevel::Critical => [
                        "System down",
                        "Data corruption detected",
                        "Security breach detected",
                        "Cluster quorum lost",
                    ],
                };

                let message = messages[rand::random::<usize>() % messages.len()];

                let alert_event = SystemAlertEvent::new(
                    level,
                    message,
                    service,
                    level == AlertLevel::Error || level == AlertLevel::Critical,
                );

                event_bus.publish(alert_event).await?;
            }
        }
    }

    Ok(())
}

/// Monitor real-time alerts
async fn monitor_alerts(dashboard: Arc<DashboardAggregator>) -> Result<()> {
    let mut rx = dashboard.subscribe_to_alerts();

    info!("Alert monitor started");

    while let Ok(alert) = rx.recv().await {
        let level = match alert.alert_level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Error => "ERROR",
            AlertLevel::Critical => "CRITICAL",
        };

        // Log alerts differently based on level
        match alert.alert_level {
            AlertLevel::Info => {
                info!("📢 [{}] {} - {}", level, alert.service, alert.message);
            }
            AlertLevel::Warning => {
                warn!("⚠️ [{}] {} - {}", level, alert.service, alert.message);
            }
            AlertLevel::Error | AlertLevel::Critical => {
                error!("🚨 [{}] {} - {}", level, alert.service, alert.message);
            }
        }
    }

    Ok(())
}

/// Display dashboard status periodically
async fn display_dashboard(dashboard: Arc<DashboardAggregator>) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    while interval.tick().await.elapsed() < tokio::time::Duration::from_secs(30) {
        // Get current component status
        let components = dashboard.get_component_health();
        let metrics = dashboard.get_metrics();

        println!("\n--- SYSTEM DASHBOARD ({}) ---", Utc::now());

        // Display component status
        println!("Component Status:");
        for (_, component) in components {
            let status_icon = match component.status {
                StatusIndicator::Healthy => "✅",
                StatusIndicator::Warning => "⚠️",
                StatusIndicator::Critical => "❌",
                StatusIndicator::Unknown => "❓",
            };

            println!(
                "{} {} - {} (updated: {})",
                status_icon,
                component.name,
                component
                    .message
                    .unwrap_or_else(|| "No message".to_string()),
                component.last_updated.format("%H:%M:%S")
            );
        }

        // Display event metrics
        println!("\nEvent Activity:");
        println!(
            "Period: {} to {}",
            metrics.period_start.format("%H:%M:%S"),
            metrics.period_end.format("%H:%M:%S")
        );
        println!("Total Events: {}", metrics.total_events);

        for (event_type, count) in metrics.event_counts {
            println!("- {}: {}", event_type, count);
        }

        // Display recent alerts
        let alerts = dashboard.get_recent_alerts();
        if !alerts.is_empty() {
            println!("\nRecent Alerts:");
            for alert in alerts.iter().rev().take(5) {
                let level = match alert.alert_level {
                    AlertLevel::Info => "INFO",
                    AlertLevel::Warning => "WARNING",
                    AlertLevel::Error => "ERROR",
                    AlertLevel::Critical => "CRITICAL",
                };

                println!(
                    "[{}] {} - {} ({})",
                    level,
                    alert.service,
                    alert.message,
                    alert.metadata.timestamp.format("%H:%M:%S")
                );
            }
        }

        println!("-------------------------------\n");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Create application
    let app = event_system::create_event_system_app(
        "postgres://navius:navius_password@localhost:5433/navius_events",
    )
    .await?;
    let mut app = app.build();

    // Start the application
    app.start().await?;

    // Retrieve the event bus component
    let event_bus = app
        .get_component::<EventBusService>("eventBus")
        .expect("Event bus component not found");

    // Create and register our dashboard aggregator
    let dashboard = Arc::new(DashboardAggregator::new("SystemDashboard"));

    // Subscribe to relevant events
    event_bus.subscribe("OrderCreated", dashboard.clone())?;
    event_bus.subscribe("OrderShipped", dashboard.clone())?;
    event_bus.subscribe("InventoryUpdated", dashboard.clone())?;
    event_bus.subscribe("SystemAlert", dashboard.clone())?;

    info!("Dashboard aggregator registered with event bus");

    // Start monitoring tasks
    let event_generator = tokio::spawn(generate_test_events(event_bus.clone()));
    let alert_monitor = tokio::spawn(monitor_alerts(dashboard.clone()));
    let dashboard_display = tokio::spawn(display_dashboard(dashboard.clone()));

    info!("Real-time dashboard example started");
    info!("Running for 30 seconds with sample events...");

    // Reset metrics every 10 seconds
    let dashboard_clone = dashboard.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        for _ in 0..3 {
            interval.tick().await;
            dashboard_clone.reset_metrics();
        }
    });

    // Wait for display task to complete
    let _ = tokio::join!(event_generator, alert_monitor, dashboard_display);

    // Shutdown
    app.stop().await?;

    info!("Real-time dashboard example completed");
    Ok(())
}

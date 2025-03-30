//! Event System API
//!
//! This file contains the HTTP API implementation for the Event System integration example.

use std::sync::Arc;

use navius_core::error::{Error, Result};
use navius_http::{
    response::Response,
    routing::{Route, Router},
    server::{HttpServer, ServerConfig},
};
use tracing::info;
use uuid::Uuid;

use crate::events::{OrderCreatedEvent, OrderShippedEvent, SystemAlertEvent};
use crate::handlers::{AnalyticsHandler, InventoryHandler};
use crate::models::{AlertLevel, ApiResponse, OrderItem};
use crate::repository::EventRepository;
use crate::services::EventBusService;

/// AppServer integrates all components with HTTP endpoints
pub struct AppServer {
    server: HttpServer,
    event_bus: EventBusService,
    event_repository: Arc<EventRepository>,
    analytics_handler: Arc<AnalyticsHandler>,
    inventory_handler: Arc<InventoryHandler>,
}

impl AppServer {
    pub fn new(
        event_bus: EventBusService,
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

    pub fn configure_routes(&mut self) -> Result<()> {
        let mut router = Router::new();

        // Health check endpoint
        router.add_route(Route::get("/health", |_req| {
            Ok(serde_json::to_string(&ApiResponse::success("OK")).unwrap())
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
                        Ok(serde_json::to_string(&ApiResponse::success(
                            "Order created event triggered",
                        ))
                        .unwrap())
                    }
                    "order-shipped" => {
                        // Create a sample shipping event
                        let event =
                            OrderShippedEvent::new(Uuid::new_v4(), "TRK12345678", "Express");

                        event_bus.publish(event).await?;
                        Ok(serde_json::to_string(&ApiResponse::success(
                            "Order shipped event triggered",
                        ))
                        .unwrap())
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
                        Ok(serde_json::to_string(&ApiResponse::success(
                            "System alert event triggered",
                        ))
                        .unwrap())
                    }
                    _ => {
                        let error_msg = format!("Unsupported event type: {}", event_type);
                        Ok(serde_json::to_string(&ApiResponse::<()>::error(&error_msg)).unwrap())
                    }
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

                let events = match event_type {
                    Some(event_type) => {
                        event_repository
                            .get_events_by_type(event_type, limit)
                            .await?
                    }
                    None => event_repository.get_events(limit).await?,
                };

                Ok(serde_json::to_string(&ApiResponse::success(events)).unwrap())
            }
        }));

        // Event metrics endpoint
        let analytics_handler = self.analytics_handler.clone();
        router.add_route(Route::get("/api/events/metrics", move |_req| {
            let metrics = analytics_handler.get_metrics();
            Ok(serde_json::to_string(&ApiResponse::success(metrics)).unwrap())
        }));

        // Inventory status endpoint
        let inventory_handler = self.inventory_handler.clone();
        router.add_route(Route::get("/api/inventory", move |_req| {
            let inventory = inventory_handler.get_inventory();
            Ok(serde_json::to_string(&ApiResponse::success(inventory)).unwrap())
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

    pub async fn start(&self) -> Result<()> {
        info!("Starting AppServer");
        self.server.start().await
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping AppServer");
        self.server.stop().await
    }
}

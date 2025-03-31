//! External Message Broker Integration
//!
//! This module provides adapters for integrating with external message brokers
//! like Kafka and RabbitMQ. It enables publishing events to external systems
//! and subscribing to events from external sources.

use async_trait::async_trait;
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::events::DomainEvent;
use crate::services::EventBusService;

/// Message broker connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    /// Type of message broker (kafka, rabbitmq)
    pub broker_type: BrokerType,
    /// Connection string for the broker
    pub connection_string: String,
    /// Client ID or application name
    pub client_id: String,
    /// Authentication credentials (optional)
    pub credentials: Option<BrokerCredentials>,
    /// Additional configuration options
    pub options: serde_json::Value,
}

/// Supported message broker types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrokerType {
    Kafka,
    RabbitMQ,
}

/// Authentication credentials for message brokers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerCredentials {
    pub username: String,
    pub password: String,
    pub mechanism: String,
}

/// External message broker adapter trait
#[async_trait]
pub trait MessageBrokerAdapter: Send + Sync {
    /// Connect to the message broker
    async fn connect(&self) -> Result<()>;

    /// Disconnect from the message broker
    async fn disconnect(&self) -> Result<()>;

    /// Publish an event to the message broker
    async fn publish_event<E>(&self, event: E, topic: &str) -> Result<()>
    where
        E: DomainEvent + Send + Sync + 'static;

    /// Subscribe to events from a topic
    async fn subscribe(&self, topic: &str, event_bus: EventBusService) -> Result<()>;

    /// Check if the connection is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get the broker type
    fn broker_type(&self) -> BrokerType;

    /// Get connection status information
    fn connection_info(&self) -> serde_json::Value;
}

/// Factory for creating message broker adapters
pub struct MessageBrokerFactory;

impl MessageBrokerFactory {
    /// Create a new message broker adapter based on configuration
    pub fn create(config: BrokerConfig) -> Result<Arc<dyn MessageBrokerAdapter>> {
        match config.broker_type {
            BrokerType::Kafka => Ok(Arc::new(KafkaAdapter::new(config))),
            BrokerType::RabbitMQ => Ok(Arc::new(RabbitMqAdapter::new(config))),
        }
    }
}

//=============================================================================
// Kafka Adapter Implementation
//=============================================================================

/// Kafka message broker adapter
pub struct KafkaAdapter {
    config: BrokerConfig,
    is_connected: std::sync::atomic::AtomicBool,
    error_count: std::sync::atomic::AtomicUsize,
}

impl KafkaAdapter {
    /// Create a new Kafka adapter with the specified configuration
    pub fn new(config: BrokerConfig) -> Self {
        Self {
            config,
            is_connected: std::sync::atomic::AtomicBool::new(false),
            error_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Simulate publishing to Kafka
    async fn simulate_kafka_publish(&self, topic: &str, payload: &str) -> Result<()> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::new("Not connected to Kafka"));
        }

        // In a real implementation, this would use a Kafka client library
        debug!(
            "Publishing to Kafka topic '{}': {} bytes",
            topic,
            payload.len()
        );

        // Simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Occasional simulated errors for realistic behavior
        if rand::random::<f32>() < 0.05 {
            self.error_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            warn!("Kafka publish error simulation (1 in 20 chance)");
            return Err(Error::new("Simulated Kafka publish error"));
        }

        info!("Successfully published event to Kafka topic '{}'", topic);
        Ok(())
    }

    /// Simulate Kafka subscription with a background task
    async fn start_kafka_consumer(&self, topic: String, event_bus: EventBusService) -> Result<()> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::new("Not connected to Kafka"));
        }

        // Create a channel for the consumer task to send events back
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn a background task to simulate Kafka consumer
        let is_connected = self.is_connected.clone();
        tokio::spawn(async move {
            info!("Started Kafka consumer for topic '{}'", topic);

            // Keep consuming while connected
            while is_connected.load(std::sync::atomic::Ordering::Relaxed) {
                // Simulate receiving a message every few seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // Only send a message occasionally to simulate random traffic
                if rand::random::<f32>() < 0.3 {
                    // Create a test message
                    let event_type = match rand::random::<u8>() % 3 {
                        0 => "ExternalOrderCreated",
                        1 => "ExternalInventoryUpdate",
                        _ => "ExternalSystemAlert",
                    };

                    let payload = match event_type {
                        "ExternalOrderCreated" => format!(
                            r#"{{"event_type":"{}","order_id":"EXT-{}","timestamp":"{}","customer_id":"CUST-EXT-{}","items":[{{"product_id":"PROD-001","quantity":2,"price":29.99}}]}}"#,
                            event_type,
                            uuid::Uuid::new_v4(),
                            chrono::Utc::now().to_rfc3339(),
                            rand::random::<u16>(),
                        ),
                        "ExternalInventoryUpdate" => format!(
                            r#"{{"event_type":"{}","product_id":"PROD-EXT-{}","prev_qty":{},"new_qty":{},"timestamp":"{}","warehouse":"WAREHOUSE-EXT"}}"#,
                            event_type,
                            rand::random::<u16>(),
                            rand::random::<u8>() + 10,
                            rand::random::<u8>(),
                            chrono::Utc::now().to_rfc3339(),
                        ),
                        _ => format!(
                            r#"{{"event_type":"{}","level":"WARNING","message":"External system alert","source":"external-system","timestamp":"{}"}}"#,
                            event_type,
                            chrono::Utc::now().to_rfc3339(),
                        ),
                    };

                    // Send the message to the channel
                    if tx.send((event_type.to_string(), payload)).await.is_err() {
                        error!("Failed to send event to channel - receiver dropped");
                        break;
                    }
                }
            }

            info!("Stopped Kafka consumer for topic '{}'", topic);
        });

        // Start a task to process received events
        let event_bus_clone = event_bus.clone();
        tokio::spawn(async move {
            while let Some((event_type, event_data)) = rx.recv().await {
                info!("Received external event from Kafka: {}", event_type);

                // Record the external event in the event bus history
                if let Err(e) =
                    event_bus_clone.record_event(&event_type, &event_data, "kafka-external")
                {
                    error!("Failed to record external event: {}", e);
                    continue;
                }

                // Process the external event using the appropriate handlers
                match event_bus_clone
                    .process_external_event(&event_type, &event_data)
                    .await
                {
                    Ok(_) => debug!("Successfully processed external event"),
                    Err(e) => error!("Failed to process external event: {}", e),
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl MessageBrokerAdapter for KafkaAdapter {
    async fn connect(&self) -> Result<()> {
        info!("Connecting to Kafka: {}", self.config.connection_string);

        // Simulate connection process
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Set connected state
        self.is_connected
            .store(true, std::sync::atomic::Ordering::Relaxed);

        info!("Connected to Kafka successfully");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from Kafka");

        // Set disconnected state
        self.is_connected
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Simulate disconnection process
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        info!("Disconnected from Kafka successfully");
        Ok(())
    }

    async fn publish_event<E>(&self, event: E, topic: &str) -> Result<()>
    where
        E: DomainEvent + Send + Sync + 'static,
    {
        // Convert the event to JSON
        let payload = event.to_json()?;

        // Publish to Kafka
        self.simulate_kafka_publish(topic, &payload).await
    }

    async fn subscribe(&self, topic: &str, event_bus: EventBusService) -> Result<()> {
        info!("Subscribing to Kafka topic: {}", topic);

        // Start the Kafka consumer
        self.start_kafka_consumer(topic.to_string(), event_bus)
            .await
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if connected
        let is_healthy = self.is_connected.load(std::sync::atomic::Ordering::Relaxed);

        if is_healthy {
            // Check error count - if too many errors, consider unhealthy
            let errors = self.error_count.load(std::sync::atomic::Ordering::Relaxed);
            return Ok(errors < 10);
        }

        Ok(false)
    }

    fn broker_type(&self) -> BrokerType {
        BrokerType::Kafka
    }

    fn connection_info(&self) -> serde_json::Value {
        serde_json::json!({
            "broker_type": "Kafka",
            "connection_string": self.config.connection_string,
            "client_id": self.config.client_id,
            "connected": self.is_connected.load(std::sync::atomic::Ordering::Relaxed),
            "error_count": self.error_count.load(std::sync::atomic::Ordering::Relaxed),
        })
    }
}

//=============================================================================
// RabbitMQ Adapter Implementation
//=============================================================================

/// RabbitMQ message broker adapter
pub struct RabbitMqAdapter {
    config: BrokerConfig,
    is_connected: std::sync::atomic::AtomicBool,
    channels: std::sync::Mutex<Vec<String>>,
}

impl RabbitMqAdapter {
    /// Create a new RabbitMQ adapter with the specified configuration
    pub fn new(config: BrokerConfig) -> Self {
        Self {
            config,
            is_connected: std::sync::atomic::AtomicBool::new(false),
            channels: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Simulate publishing to RabbitMQ
    async fn simulate_rabbitmq_publish(&self, queue: &str, payload: &str) -> Result<()> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::new("Not connected to RabbitMQ"));
        }

        // In a real implementation, this would use a RabbitMQ client library
        debug!(
            "Publishing to RabbitMQ queue '{}': {} bytes",
            queue,
            payload.len()
        );

        // Simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

        info!("Successfully published event to RabbitMQ queue '{}'", queue);
        Ok(())
    }

    /// Simulate RabbitMQ subscription with a background task
    async fn start_rabbitmq_consumer(
        &self,
        queue: String,
        event_bus: EventBusService,
    ) -> Result<()> {
        if !self.is_connected.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::new("Not connected to RabbitMQ"));
        }

        // Add the queue to the list of channels
        {
            let mut channels = self.channels.lock().unwrap();
            channels.push(queue.clone());
        }

        // Create a channel for the consumer task to send events back
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn a background task to simulate RabbitMQ consumer
        let is_connected = self.is_connected.clone();
        tokio::spawn(async move {
            info!("Started RabbitMQ consumer for queue '{}'", queue);

            // Keep consuming while connected
            while is_connected.load(std::sync::atomic::Ordering::Relaxed) {
                // Simulate receiving a message every few seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Only send a message occasionally to simulate random traffic
                if rand::random::<f32>() < 0.2 {
                    // Create a test message
                    let event_type = match rand::random::<u8>() % 2 {
                        0 => "ExternalPaymentProcessed",
                        _ => "ExternalShipmentUpdate",
                    };

                    let payload = match event_type {
                        "ExternalPaymentProcessed" => format!(
                            r#"{{"event_type":"{}","payment_id":"PAY-{}","order_id":"ORD-{}","amount":{},"status":"completed","timestamp":"{}"}}"#,
                            event_type,
                            uuid::Uuid::new_v4(),
                            uuid::Uuid::new_v4(),
                            (rand::random::<f32>() * 100.0).round() / 100.0 + 10.0,
                            chrono::Utc::now().to_rfc3339(),
                        ),
                        _ => format!(
                            r#"{{"event_type":"{}","shipment_id":"SHP-{}","carrier":"External Logistics","status":"in_transit","location":"Distribution Center","timestamp":"{}"}}"#,
                            event_type,
                            uuid::Uuid::new_v4(),
                            chrono::Utc::now().to_rfc3339(),
                        ),
                    };

                    // Send the message to the channel
                    if tx.send((event_type.to_string(), payload)).await.is_err() {
                        error!("Failed to send event to channel - receiver dropped");
                        break;
                    }
                }
            }

            info!("Stopped RabbitMQ consumer for queue '{}'", queue);
        });

        // Start a task to process received events
        let event_bus_clone = event_bus.clone();
        tokio::spawn(async move {
            while let Some((event_type, event_data)) = rx.recv().await {
                info!("Received external event from RabbitMQ: {}", event_type);

                // Record the external event in the event bus history
                if let Err(e) =
                    event_bus_clone.record_event(&event_type, &event_data, "rabbitmq-external")
                {
                    error!("Failed to record external event: {}", e);
                    continue;
                }

                // Process the external event using the appropriate handlers
                match event_bus_clone
                    .process_external_event(&event_type, &event_data)
                    .await
                {
                    Ok(_) => debug!("Successfully processed external event"),
                    Err(e) => error!("Failed to process external event: {}", e),
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl MessageBrokerAdapter for RabbitMqAdapter {
    async fn connect(&self) -> Result<()> {
        info!("Connecting to RabbitMQ: {}", self.config.connection_string);

        // Simulate connection process
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // Set connected state
        self.is_connected
            .store(true, std::sync::atomic::Ordering::Relaxed);

        info!("Connected to RabbitMQ successfully");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from RabbitMQ");

        // Set disconnected state
        self.is_connected
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Clear channels
        {
            let mut channels = self.channels.lock().unwrap();
            channels.clear();
        }

        // Simulate disconnection process
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        info!("Disconnected from RabbitMQ successfully");
        Ok(())
    }

    async fn publish_event<E>(&self, event: E, queue: &str) -> Result<()>
    where
        E: DomainEvent + Send + Sync + 'static,
    {
        // Convert the event to JSON
        let payload = event.to_json()?;

        // Publish to RabbitMQ
        self.simulate_rabbitmq_publish(queue, &payload).await
    }

    async fn subscribe(&self, queue: &str, event_bus: EventBusService) -> Result<()> {
        info!("Subscribing to RabbitMQ queue: {}", queue);

        // Start the RabbitMQ consumer
        self.start_rabbitmq_consumer(queue.to_string(), event_bus)
            .await
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if connected
        Ok(self.is_connected.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn broker_type(&self) -> BrokerType {
        BrokerType::RabbitMQ
    }

    fn connection_info(&self) -> serde_json::Value {
        let channels = self.channels.lock().unwrap();

        serde_json::json!({
            "broker_type": "RabbitMQ",
            "connection_string": self.config.connection_string,
            "client_id": self.config.client_id,
            "connected": self.is_connected.load(std::sync::atomic::Ordering::Relaxed),
            "channels": channels,
        })
    }
}

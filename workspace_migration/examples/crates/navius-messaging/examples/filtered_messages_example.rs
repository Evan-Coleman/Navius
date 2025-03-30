use std::sync::Arc;
use std::time::Duration;

use navius_messaging::broker::TopologyBuilder;
use navius_messaging::config::BrokerConfig;
use navius_messaging::consumer::ConsumerOptions;
use navius_messaging::message::{
    Message, MessageAcknowledgment, MessageFilter, MessageProcessingResult,
};
use navius_messaging::publisher::PublishOptions;
use navius_messaging::serialization::JsonSerializer;
use navius_messaging::util::{generate_correlation_id, generate_id};

// Our example message payload
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct LogEvent {
    id: String,
    timestamp: u64,
    level: String,
    source: String,
    component: String,
    message: String,
    tags: Vec<String>,
    context: Option<serde_json::Value>,
}

// Custom filter implementation for log events
struct LogEventFilter {
    min_level: String,
    source_pattern: Option<String>,
    required_tags: Vec<String>,
    component: Option<String>,
}

impl LogEventFilter {
    fn new(
        min_level: &str,
        source_pattern: Option<&str>,
        required_tags: Vec<&str>,
        component: Option<&str>,
    ) -> Self {
        Self {
            min_level: min_level.to_string(),
            source_pattern: source_pattern.map(|s| s.to_string()),
            required_tags: required_tags.iter().map(|s| s.to_string()).collect(),
            component: component.map(|s| s.to_string()),
        }
    }

    fn level_priority(&self, level: &str) -> u8 {
        match level.to_lowercase().as_str() {
            "trace" => 1,
            "debug" => 2,
            "info" => 3,
            "warn" | "warning" => 4,
            "error" => 5,
            "fatal" | "critical" => 6,
            _ => 0,
        }
    }
}

impl MessageFilter<LogEvent> for LogEventFilter {
    fn matches(&self, message: &Message<LogEvent>) -> bool {
        let log = &message.payload;

        // Check minimum log level
        let message_level = self.level_priority(&log.level);
        let min_level = self.level_priority(&self.min_level);

        if message_level < min_level {
            return false;
        }

        // Check source pattern if specified
        if let Some(pattern) = &self.source_pattern {
            if !log.source.contains(pattern) {
                return false;
            }
        }

        // Check component if specified
        if let Some(component) = &self.component {
            if &log.component != component {
                return false;
            }
        }

        // Check if all required tags are present
        if !self.required_tags.is_empty() {
            for required_tag in &self.required_tags {
                if !log.tags.contains(required_tag) {
                    return false;
                }
            }
        }

        true
    }
}

// Use the in-memory broker from the basic example
mod memory_broker {
    // For brevity, reuse the same in-memory broker implementation
    // from the basic_example.rs file
    // In a real application, you would import or include the implementation here
    include!("basic_example.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a broker config
    let config = BrokerConfig::new("in-memory-broker", "InMemory Broker Example", "in-memory");

    // Create a broker factory
    let factory = memory_broker::InMemoryBrokerFactory;

    // Create the broker
    let broker = factory.create_broker(config).await?;

    // Connect to the broker
    println!("Connecting to the broker...");
    broker.connect().await?;
    println!("Connected!");

    // Create the basic topology
    println!("Setting up messaging topology...");
    let builder = TopologyBuilder::new(broker.clone());

    // Create a fanout exchange for logs
    let exchange = "logs";
    builder
        .declare_exchange(
            exchange,
            navius_messaging::topology::ExchangeType::Fanout,
            true,
            false,
            None,
        )
        .await?;

    // Create the logs queue
    let queue = "application-logs";
    builder
        .declare_queue(queue, true, false, false, None)
        .await?;

    // Bind the queue to the exchange
    builder.bind_queue(queue, exchange, "", None).await?;

    // Create filters
    let error_filter = LogEventFilter::new("error", None, vec![], None);
    let db_filter = LogEventFilter::new("info", Some("database"), vec![], None);
    let security_filter = LogEventFilter::new("info", None, vec!["security"], None);

    // Create handlers
    let error_handler = |message: &navius_messaging::message::ReceivedMessage<LogEvent>| -> MessageProcessingResult {
        let log = &message.message.payload;
        println!(
            "[ERROR HANDLER] {} - {}: {} - {}",
            log.timestamp, log.level.to_uppercase(), log.component, log.message
        );
        Ok(MessageAcknowledgment::Ack)
    };

    let db_handler = |message: &navius_messaging::message::ReceivedMessage<LogEvent>| -> MessageProcessingResult {
        let log = &message.message.payload;
        println!(
            "[DATABASE HANDLER] {} - {}: {} - {}",
            log.timestamp, log.level.to_uppercase(), log.component, log.message
        );
        Ok(MessageAcknowledgment::Ack)
    };

    let security_handler = |message: &navius_messaging::message::ReceivedMessage<LogEvent>| -> MessageProcessingResult {
        let log = &message.message.payload;
        println!(
            "[SECURITY HANDLER] {} - {}: {} - {}",
            log.timestamp, log.level.to_uppercase(), log.component, log.message
        );
        Ok(MessageAcknowledgment::Ack)
    };

    // Subscribe with filters
    println!("Setting up filtered consumers...");

    let error_consumer = broker
        .subscribe_filtered(
            queue,
            error_handler,
            error_filter,
            Some(ConsumerOptions::default().with_consumer_tag("error-consumer")),
        )
        .await?;

    let db_consumer = broker
        .subscribe_filtered(
            queue,
            db_handler,
            db_filter,
            Some(ConsumerOptions::default().with_consumer_tag("db-consumer")),
        )
        .await?;

    let security_consumer = broker
        .subscribe_filtered(
            queue,
            security_handler,
            security_filter,
            Some(ConsumerOptions::default().with_consumer_tag("security-consumer")),
        )
        .await?;

    println!("Consumers ready!");

    // Publish some test log events
    println!("Publishing test log events...");

    let log_events = vec![
        // Standard info log - should be filtered out
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "info".to_string(),
            source: "app".to_string(),
            component: "web".to_string(),
            message: "Application started successfully".to_string(),
            tags: vec!["startup".to_string()],
            context: None,
        },
        // Error log - should be caught by error filter
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "error".to_string(),
            source: "app".to_string(),
            component: "api".to_string(),
            message: "Failed to connect to external service".to_string(),
            tags: vec!["external".to_string(), "connectivity".to_string()],
            context: Some(serde_json::json!({
                "service": "payment-gateway",
                "timeout": 5000
            })),
        },
        // Database info log - should be caught by db filter
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "info".to_string(),
            source: "database".to_string(),
            component: "postgres".to_string(),
            message: "Database migration completed successfully".to_string(),
            tags: vec!["migration".to_string(), "performance".to_string()],
            context: Some(serde_json::json!({
                "version": "v2.3.0",
                "duration_ms": 1250
            })),
        },
        // Security warning log - should be caught by security filter
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "warn".to_string(),
            source: "auth".to_string(),
            component: "login".to_string(),
            message: "Multiple failed login attempts detected".to_string(),
            tags: vec!["security".to_string(), "auth".to_string()],
            context: Some(serde_json::json!({
                "username": "admin",
                "ip": "203.0.113.42",
                "attempts": 5
            })),
        },
        // Database error log - should be caught by both error and db filters
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "error".to_string(),
            source: "database".to_string(),
            component: "postgres".to_string(),
            message: "Query timeout detected".to_string(),
            tags: vec!["performance".to_string(), "timeout".to_string()],
            context: Some(serde_json::json!({
                "query_id": "q-29471",
                "duration_ms": 30000,
                "table": "large_transactions"
            })),
        },
        // Security error log - should be caught by both error and security filters
        LogEvent {
            id: generate_id(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            level: "error".to_string(),
            source: "auth".to_string(),
            component: "permissions".to_string(),
            message: "Unauthorized access attempt to restricted resource".to_string(),
            tags: vec![
                "security".to_string(),
                "auth".to_string(),
                "violation".to_string(),
            ],
            context: Some(serde_json::json!({
                "user_id": "user-591",
                "resource": "/admin/system-settings",
                "required_role": "admin"
            })),
        },
    ];

    // Publish all log events
    for log_event in log_events {
        let message =
            Message::new(log_event.clone(), "logs").with_correlation_id(generate_correlation_id());

        let publish_options = PublishOptions::new(exchange);

        broker.publish(&message, Some(publish_options)).await?;
        println!("Published log: {} - {}", log_event.level, log_event.message);
    }

    // Wait a bit for messages to be processed
    println!("Waiting for messages to be processed...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Cancel the consumers
    println!("Cancelling consumers...");
    error_consumer.cancel().await?;
    db_consumer.cancel().await?;
    security_consumer.cancel().await?;

    // Get metrics
    let metrics = broker.metrics().await;
    println!("Broker metrics:");
    println!("- Messages published: {}", metrics.published_messages);
    println!("- Messages consumed: {}", metrics.consumed_messages);
    println!("- Messages acknowledged: {}", metrics.acknowledged_messages);

    // Disconnect from the broker
    println!("Disconnecting from the broker...");
    broker.disconnect().await?;
    println!("Disconnected!");

    println!("Example completed successfully!");
    Ok(())
}

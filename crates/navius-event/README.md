# Navius Event System

A lightweight, flexible, and type-safe event system for building event-driven applications with the Navius framework.

## Features

- **Type-safe event publishing and subscribing** with full serialization support
- **Filtered event subscriptions** based on event type, priority, source, and metadata
- **Topic-based routing** for organizing event streams
- **Priority levels** for important events
- **Correlation IDs** for tracking related events
- **Metadata support** for adding context to events
- **In-memory implementation** with configurable retention
- **Async-first design** with Tokio integration
- **Backpressure handling** with configurable buffer sizes

## Quick Start

```rust
use navius_event::{create_memory_broker, Event, SubscriptionOptions};
use serde::{Deserialize, Serialize};

// Define your event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCreatedEvent {
    user_id: String,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory event broker
    let broker = create_memory_broker().await?;
    
    // Create a topic
    broker.create_topic("user.events").await?;
    
    // Subscribe to events
    let options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("user-subscription");
        
    let (subscription_id, mut event_stream) = 
        broker.subscribe::<UserCreatedEvent>("user.events", options).await?;
    
    // Spawn a task to handle events
    tokio::spawn(async move {
        while let Some(result) = event_stream.next().await {
            match result {
                Ok(event) => {
                    println!("Received user created event:");
                    println!("  User ID: {}", event.payload.user_id);
                    println!("  Username: {}", event.payload.username);
                    println!("  Email: {}", event.payload.email);
                }
                Err(e) => eprintln!("Error receiving event: {}", e),
            }
        }
    });
    
    // Publish an event
    let event = Event::new(
        "user.created", 
        "user.events", 
        "user-service",
        UserCreatedEvent {
            user_id: "123".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        },
    );
    
    broker.publish(event).await?;
    
    // When done, unsubscribe
    broker.unsubscribe(&subscription_id).await?;
    
    Ok(())
}
```

## Core Components

### Event

The `Event<T>` struct represents an event with a typed payload:

```rust
let event = Event::new(
    "user.created",           // Event type
    "user.events",            // Topic
    "user-service",           // Source
    UserCreatedEvent { ... }, // Payload
)
.with_correlation_id("signup-flow-123")
.with_metadata("region", "us-west");
```

### EventBroker

The `EventBroker` trait defines the interface for event brokers:

```rust
// Create topics
broker.create_topic("orders").await?;

// Publish events
broker.publish(event).await?;

// Subscribe to events
let (subscription_id, event_stream) = 
    broker.subscribe::<OrderEvent>("orders", options).await?;

// Unsubscribe
broker.unsubscribe(&subscription_id).await?;
```

### Subscription Options

Configure how you receive events:

```rust
let options = SubscriptionOptions::new()
    .with_buffer_size(100)                 // Configure backpressure buffer
    .with_name("audit-subscription")       // Name for debugging
    .with_historical_events(true)          // Get events from before subscription
    .with_filter(filter)                   // Apply filtering
    .with_max_retries(5);                  // Configure delivery retries
```

### Event Filtering

Filter events based on various criteria:

```rust
let filter = EventFilterConfig::new()
    .with_event_types(vec!["order.created", "order.updated"])
    .with_min_priority(EventPriority::High)
    .with_sources(vec!["order-service"])
    .with_correlation_id("batch-process-123")
    .with_metadata("region", "us-east");

let options = SubscriptionOptions::new()
    .with_filter(filter);
```

## Advanced Usage

### JSON Events

Work with dynamically typed JSON events:

```rust
// Publish a JSON event
broker.publish_json(
    "config.updated",
    "system.events",
    "config-service",
    serde_json::json!({
        "component": "database",
        "key": "max_connections",
        "value": 100
    }),
).await?;

// Subscribe to JSON events
let (sub_id, json_stream) = broker.subscribe_json(
    "system.events", 
    options
).await?;
```

### Event Priorities

Prioritize critical events:

```rust
let event = Event::with_priority(
    "system.alert",
    "system.events",
    "monitoring-service",
    AlertEvent { ... },
    EventPriority::Critical,
);
```

### Working with Recent Events

Retrieve or clear recent events:

```rust
// Get recent events
let recent_events = broker.get_recent_events("orders", 10).await?;

// Apply filters to recent events
let filtered_events = broker.filter_events("orders", filter, 10).await?;

// Clear events
broker.clear_events("orders").await?;
```

### Broker Health Checks

Verify broker health:

```rust
let healthy = broker.health_check().await?;
```

## Examples

The `examples/` directory contains several usage examples:

- **basic_events.rs**: Demonstrates basic event publishing and subscribing
- **filtered_events.rs**: Shows how to use event filtering

To run an example:

```bash
cargo run --example basic_events
```

## Performance Considerations

- Configure appropriate buffer sizes in `SubscriptionOptions` to handle backpressure
- Use event filtering to reduce unnecessary event processing
- Set appropriate retention policies in `EventBrokerConfig`

## License

MIT OR Apache-2.0 
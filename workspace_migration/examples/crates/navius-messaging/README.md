# Navius Messaging

A flexible, high-performance messaging system for distributed applications built on Rust.

## Features

- **Broker-agnostic interface**: Consistent API regardless of the underlying message broker.
- **Multiple messaging patterns**: Support for pub/sub, request/reply, and work queues.
- **Type-safe messaging**: Strongly typed message payloads with serialization support.
- **Topic-based routing**: Advanced topic pattern matching for efficient message delivery.
- **Message filtering**: Client-side filtering options for selective message processing.
- **Flexible subscriptions**: Advanced subscription options with priority handling and correlation.
- **Connection management**: Automatic connection handling with retry capabilities.
- **Topology management**: Declarative or programmatic definition of message exchanges and queues.
- **Performance metrics**: Built-in metrics collection for monitoring message throughput.
- **Error handling**: Comprehensive error handling and recovery mechanisms.
- **Async-first**: Built from the ground up for Rust's async/await ecosystem.

## Architecture

The crate is designed around the following components:

- **MessageBroker**: Core trait defining operations for message brokers.
- **Message**: Type for wrapping payloads with metadata and correlation information.
- **Publisher**: Interface for publishing messages with various delivery options.
- **Consumer**: Interface for consuming messages with filtering capabilities.
- **Topology**: Components for defining message routing infrastructure.
- **Serialization**: Tools for converting between message types and serialized formats.
- **Error**: Comprehensive error types for graceful error handling.

## Usage Examples

### Basic Publishing and Consuming

```rust
use std::time::Duration;
use navius_messaging::broker::TopologyBuilder;
use navius_messaging::config::BrokerConfig;
use navius_messaging::consumer::ConsumerOptions;
use navius_messaging::message::{Message, MessageAcknowledgment};
use navius_messaging::publisher::PublishOptions;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct UserEvent {
    id: String,
    user_id: String,
    event_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and connect to a broker
    let config = BrokerConfig::new("my-broker", "My Broker", "in-memory");
    let factory = InMemoryBrokerFactory;
    let broker = factory.create_broker(config).await?;
    broker.connect().await?;
    
    // Set up the messaging topology
    let builder = TopologyBuilder::new(broker.clone());
    let topics = vec![("user-events", "user.#")];
    builder.build_topic("user-events-exchange", &topics).await?;
    
    // Create a message handler
    let user_event_handler = |message: &ReceivedMessage<UserEvent>| {
        let event = &message.message.payload;
        println!("Received event: {}", event.event_type);
        Ok(MessageAcknowledgment::Ack)
    };
    
    // Subscribe to messages
    let consumer_options = ConsumerOptions::default().with_consumer_tag("user-consumer");
    let consumer = broker
        .subscribe("user-events", user_event_handler, Some(consumer_options))
        .await?;
    
    // Publish a message
    let event = UserEvent {
        id: "evt-123".to_string(),
        user_id: "user-456".to_string(),
        event_type: "login".to_string(),
    };
    
    let message = Message::new(event, "user.login");
    let options = PublishOptions::new("user-events-exchange");
    broker.publish(&message, Some(options)).await?;
    
    // Wait for processing
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Clean up
    consumer.cancel().await?;
    broker.disconnect().await?;
    
    Ok(())
}
```

### Request/Reply Pattern

```rust
use navius_messaging::util::RequestReply;

// Create a request/reply client
let request_reply = RequestReply::new(
    broker.clone(),
    "requests-queue",
    Duration::from_secs(10),
);

// Send a request and wait for response
let request = MyRequest { id: "req-123", data: "test" };
let response: MyResponse = request_reply.request(
    &request, 
    "my.request.topic",
    None
).await?;
```

### Message Filtering

```rust
// Create a custom filter
struct PriorityFilter {
    min_priority: u8,
}

impl MessageFilter<MyMessage> for PriorityFilter {
    fn matches(&self, message: &Message<MyMessage>) -> bool {
        message.payload.priority >= self.min_priority
    }
}

// Create and use the filter
let filter = PriorityFilter { min_priority: 3 };

broker.subscribe_filtered(
    "high-priority-queue",
    high_priority_handler,
    filter,
    Some(ConsumerOptions::default())
).await?;
```

## Broker Implementations

The crate provides or will provide implementations for common message brokers:

- In-memory broker (included)
- RabbitMQ (upcoming)
- NATS (upcoming)
- Kafka (planned)
- Redis Streams (planned)

## Advanced Features

### Batched Publishing

```rust
use navius_messaging::util::MessageBuffer;

let buffer = MessageBuffer::new(10, Duration::from_millis(500));

buffer.add(message1, options1).await;
buffer.add(message2, options2).await;

// Messages will be published when either:
// - The buffer reaches 10 messages
// - 500ms has elapsed since the first unbatched message
```

### Rate Limiting

```rust
use navius_messaging::util::RateLimiter;

let limiter = RateLimiter::new(100, Duration::from_secs(1)); // 100 msgs/sec

if limiter.check() {
    broker.publish(&message, None).await?;
}
```

### Deduplication

```rust
use navius_messaging::util::DeduplicationFilter;

let deduper = DeduplicationFilter::new(Duration::from_secs(60));

if !deduper.has_seen(&message_id) {
    process_message(&message);
    deduper.mark_seen(message_id);
}
```

## Metrics and Monitoring

The crate provides built-in metrics collection:

```rust
let metrics = broker.metrics().await;
println!("Messages published: {}", metrics.published_messages);
println!("Messages consumed: {}", metrics.consumed_messages);
println!("Active consumers: {}", metrics.active_consumers);
```

## Configuration

Broker configuration supports various options:

```rust
let config = BrokerConfig::new("my-broker", "My Broker", "rabbitmq")
    .with_connection_string("amqp://guest:guest@localhost:5672/%2f")
    .with_retry_strategy(RetryStrategy::Exponential {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        max_retries: 5,
    })
    .with_heartbeat(Duration::from_secs(30))
    .with_connection_timeout(Duration::from_secs(5));
```

## Testing

The in-memory broker implementation is particularly useful for testing:

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_message_processing() {
        let broker = create_in_memory_test_broker().await;
        
        // Test your messaging logic without external dependencies
        let result = send_and_receive_test_message(broker).await;
        
        assert!(result.is_ok());
    }
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
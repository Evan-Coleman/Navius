# Navius Event

A comprehensive event handling and notification system for the Navius framework.

## Features

- **Event Dispatching**: Type-safe event publishing and subscribing
- **Async Event Handling**: Process events asynchronously with Tokio
- **Topic-Based Routing**: Subscribe to specific event topics
- **Plugin Integration**: Register event handlers through plugins
- **Filtering**: Filter events based on custom predicates
- **Error Handling**: Robust error handling for event processing
- **Priority Queues**: Handle critical events first

## Basic Usage

```rust
use navius_event::{Event, EventBus, EventHandler, EventSubscription, EventResult};
use async_trait::async_trait;
use std::sync::Arc;

// Define an event
#[derive(Debug, Clone)]
struct UserCreatedEvent {
    user_id: String,
    username: String,
}

impl Event for UserCreatedEvent {
    fn event_type(&self) -> &'static str {
        "user_created"
    }

    fn topic(&self) -> &'static str {
        "users"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Define an event handler
#[derive(Debug)]
struct EmailNotificationHandler;

#[async_trait]
impl EventHandler for EmailNotificationHandler {
    fn subscriptions(&self) -> Vec<EventSubscription> {
        vec![EventSubscription::new("users")]
    }

    async fn handle(&self, event: EventEnvelope) -> EventResult<()> {
        if let Some(user_event) = event.downcast_payload::<UserCreatedEvent>() {
            println!("Sending welcome email to {}", user_event.username);
            // Send welcome email logic here
        }
        Ok(())
    }
}

// Use the event bus
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an event bus
    let event_bus = EventBus::new();
    
    // Subscribe a handler
    let handler = EmailNotificationHandler;
    let handler_id = event_bus.subscribe(handler).await?;
    
    // Publish an event
    let event = UserCreatedEvent {
        user_id: "12345".to_string(),
        username: "new_user".to_string(),
    };
    
    event_bus.publish(event).await?;
    
    // Let the events process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Cleanup
    event_bus.unsubscribe(handler_id).await?;
    event_bus.shutdown().await?;
    
    Ok(())
}
```

## Plugin Integration

You can use the event system as a plugin with the Navius plugin system:

```rust
use navius_event::{EventPlugin, EventPluginExt};
use navius_plugin::{PluginRegistry, ComponentRegistry};

// Create a plugin registry
let registry = ComponentRegistry::new();

// Create and register the event plugin
let event_plugin = EventPlugin::default();
event_plugin.initialize(&registry).await?;

// Register event handlers
let handler = MyEventHandler::new();
event_plugin.register_handler(handler).await?;

// Get the event bus from the registry
let event_bus = registry.get_by_type::<EventBus>(None)?;

// Use the event bus
event_bus.instance().publish(my_event).await?;
```

## Filtering Events

You can filter events based on custom predicates:

```rust
use navius_event::{EventSubscription, EventHandler};

#[async_trait]
impl EventHandler for AdminNotificationHandler {
    fn subscriptions(&self) -> Vec<EventSubscription> {
        vec![
            EventSubscription::new("users")
                .with_filter(|event| {
                    // Only handle events for admin users
                    if let Some(user_event) = event.downcast_payload::<UserCreatedEvent>() {
                        user_event.username.contains("admin")
                    } else {
                        false
                    }
                })
        ]
    }
    
    async fn handle(&self, event: EventEnvelope) -> EventResult<()> {
        // Handle admin user events
        // ...
        Ok(())
    }
}
```

## Event Priorities

Events can have different priorities:

```rust
use navius_event::{Event, EventPriority};

#[derive(Debug, Clone)]
struct SystemAlertEvent {
    message: String,
    severity: String,
}

impl Event for SystemAlertEvent {
    fn event_type(&self) -> &'static str {
        "system_alert"
    }

    fn topic(&self) -> &'static str {
        "system"
    }
    
    fn priority(&self) -> EventPriority {
        if self.severity == "critical" {
            EventPriority::Critical
        } else if self.severity == "high" {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

## License

MIT OR Apache-2.0 
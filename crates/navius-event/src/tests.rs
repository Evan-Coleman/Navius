use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::future::join_all;
use navius_plugin::Plugin;
use tokio::sync::mpsc;

use crate::bus::EventBus;
use crate::error::EventResult;
use crate::event::{Event, EventEnvelope, EventPriority};
use crate::handler::{EventHandler, EventSubscription};
use crate::plugin::{EventPlugin, EventPluginExt};

// Test events
#[derive(Debug, Clone)]
struct TestEvent {
    message: String,
}

#[derive(Debug, Clone)]
struct HighPriorityEvent {
    message: String,
}

// Test handlers
#[derive(Debug, Clone)]
struct TestHandler {
    received: Arc<Mutex<Vec<String>>>,
    topic: String,
}

impl TestHandler {
    fn new(topic: &str) -> Self {
        Self {
            received: Arc::new(Mutex::new(Vec::new())),
            topic: topic.to_string(),
        }
    }

    fn received_events(&self) -> Vec<String> {
        self.received.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventHandler for TestHandler {
    fn subscriptions(&self) -> Vec<EventSubscription> {
        vec![EventSubscription::new(&self.topic)]
    }

    async fn handle(&self, event: EventEnvelope) -> EventResult<()> {
        // Try to downcast to TestEvent
        if let Some(test_event) = event.downcast_payload::<TestEvent>() {
            self.received
                .lock()
                .unwrap()
                .push(test_event.message.clone());
        } else if let Some(high_event) = event.downcast_payload::<HighPriorityEvent>() {
            self.received
                .lock()
                .unwrap()
                .push(high_event.message.clone());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct FilteredHandler {
    received: Arc<Mutex<Vec<String>>>,
    message_filter: String,
}

impl FilteredHandler {
    fn new(filter: &str) -> Self {
        Self {
            received: Arc::new(Mutex::new(Vec::new())),
            message_filter: filter.to_string(),
        }
    }

    fn received_events(&self) -> Vec<String> {
        self.received.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventHandler for FilteredHandler {
    fn subscriptions(&self) -> Vec<EventSubscription> {
        let message_filter = self.message_filter.clone();
        vec![EventSubscription::new("test").with_filter(move |event| {
            if let Some(test_event) = event.downcast_payload::<TestEvent>() {
                test_event.message.contains(&message_filter)
            } else {
                false
            }
        })]
    }

    async fn handle(&self, event: EventEnvelope) -> EventResult<()> {
        if let Some(test_event) = event.downcast_payload::<TestEvent>() {
            self.received
                .lock()
                .unwrap()
                .push(test_event.message.clone());
        }
        Ok(())
    }
}

// Add explicit topic implementation for TestEvent
impl Event for TestEvent {
    fn event_type(&self) -> &'static str {
        "test_event"
    }

    fn topic(&self) -> &'static str {
        "test"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

// Add explicit topic implementation for HighPriorityEvent
impl Event for HighPriorityEvent {
    fn event_type(&self) -> &'static str {
        "high_priority_event"
    }

    fn topic(&self) -> &'static str {
        "test"
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[tokio::test]
async fn test_basic_publish_subscribe() {
    // Create an event bus
    let event_bus = EventBus::new();

    // Create a handler
    let handler = TestHandler::new("test");

    // Subscribe the handler
    let handler_id = event_bus.subscribe(handler.clone()).await.unwrap();

    // Publish an event
    let event = TestEvent {
        message: "Hello, world!".to_string(),
    };

    event_bus.publish(event).await.unwrap();

    // Give some time for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check that the handler received the event
    let received = handler.received_events();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0], "Hello, world!");

    // Unsubscribe
    event_bus.unsubscribe(handler_id).await.unwrap();

    // Shutdown
    event_bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_filtered_subscription() {
    // Create an event bus
    let event_bus = EventBus::new();

    // Create a filtered handler
    let handler = FilteredHandler::new("important");

    // Subscribe the handler
    let handler_id = event_bus.subscribe(handler.clone()).await.unwrap();

    // Publish some events
    let events = vec![
        TestEvent {
            message: "This is important".to_string(),
        },
        TestEvent {
            message: "This is not filtered".to_string(),
        },
        TestEvent {
            message: "Another important message".to_string(),
        },
    ];

    for event in events {
        event_bus.publish(event).await.unwrap();
    }

    // Give some time for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check that the handler only received the filtered events
    let received = handler.received_events();
    assert_eq!(received.len(), 2);
    assert!(received.contains(&"This is important".to_string()));
    assert!(received.contains(&"Another important message".to_string()));

    // Unsubscribe
    event_bus.unsubscribe(handler_id).await.unwrap();

    // Shutdown
    event_bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_priority_handling() {
    // Create an event bus
    let event_bus = EventBus::new();

    // Create a handler
    let handler = TestHandler::new("test");

    // Subscribe the handler
    let handler_id = event_bus.subscribe(handler.clone()).await.unwrap();

    // Create a channel to signal when events are published
    let (tx, mut rx) = mpsc::channel::<()>(1);
    let tx = Arc::new(tx);

    // Spawn a task to publish events with different priorities
    let event_bus_clone = event_bus.clone();
    tokio::spawn(async move {
        // Publish 5 normal priority events
        for i in 1..=5 {
            let event = TestEvent {
                message: format!("Normal {}", i),
            };
            event_bus_clone.publish(event).await.unwrap();
        }

        // Publish a high priority event
        let event = HighPriorityEvent {
            message: "High Priority".to_string(),
        };
        event_bus_clone.publish(event).await.unwrap();

        // Publish more normal priority events
        for i in 6..=10 {
            let event = TestEvent {
                message: format!("Normal {}", i),
            };
            event_bus_clone.publish(event).await.unwrap();
        }

        // Signal that all events are published
        let _ = tx.send(()).await;
    });

    // Wait for all events to be published
    let _ = rx.recv().await;

    // Give some time for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Check that the handler received all events
    let received = handler.received_events();
    assert_eq!(received.len(), 11);

    // Unsubscribe
    event_bus.unsubscribe(handler_id).await.unwrap();

    // Shutdown
    event_bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_event_plugin() {
    // Create a plugin registry
    let registry = navius_plugin::ComponentRegistry::new();

    // Create an event plugin
    let event_plugin = EventPlugin::default();

    // Initialize the plugin
    event_plugin.initialize(&registry).await.unwrap();

    // Create a handler
    let handler = TestHandler::new("test");

    // Register the handler with the plugin
    let handler_id = event_plugin
        .register_handler(handler.clone())
        .await
        .unwrap();

    // Get the event bus from the registry
    let event_bus = registry.get_by_type::<EventBus>(None).unwrap();

    // Publish an event
    let event = TestEvent {
        message: "Hello from plugin!".to_string(),
    };

    event_bus.instance().publish(event).await.unwrap();

    // Give some time for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check that the handler received the event
    let received = handler.received_events();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0], "Hello from plugin!");

    // Shutdown the plugin
    event_plugin.shutdown(&registry).await.unwrap();
}

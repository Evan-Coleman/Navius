use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::events::{Event, EventError, EventPublisher, EventSubscriber};

type SubscriberMap = HashMap<String, Vec<Arc<dyn EventSubscriber>>>;

pub struct InMemoryEventPublisher {
    subscribers: Arc<Mutex<SubscriberMap>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish_event<E: Event>(&self, event: E) -> Result<(), EventError> {
        let event_type = event.event_type();
        let subscribers = self
            .subscribers
            .lock()
            .map_err(|e| EventError::internal(format!("Failed to acquire lock: {}", e)))?;

        let event_subscribers = subscribers.get(&event_type).cloned().unwrap_or_default();

        for subscriber in event_subscribers {
            if let Err(e) = subscriber.handle_event(&event).await {
                // Log error but continue with other subscribers
                eprintln!("Error handling event: {}", e);
            }
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        event_type: &str,
        subscriber: Arc<dyn EventSubscriber>,
    ) -> Result<(), EventError> {
        let mut subscribers = self
            .subscribers
            .lock()
            .map_err(|e| EventError::internal(format!("Failed to acquire lock: {}", e)))?;

        let event_subscribers = subscribers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new);
        event_subscribers.push(subscriber);

        Ok(())
    }

    async fn unsubscribe(&self, event_type: &str, subscriber_id: Uuid) -> Result<(), EventError> {
        let mut subscribers = self
            .subscribers
            .lock()
            .map_err(|e| EventError::internal(format!("Failed to acquire lock: {}", e)))?;

        if let Some(event_subscribers) = subscribers.get_mut(event_type) {
            event_subscribers.retain(|s| s.id() != subscriber_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::EventType;

    #[derive(Debug, Clone)]
    struct TestEvent {
        id: Uuid,
        test_data: String,
    }

    impl Event for TestEvent {
        fn id(&self) -> Uuid {
            self.id
        }

        fn event_type(&self) -> String {
            "test.event".to_string()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn event_name(&self) -> &'static str {
            "TestEvent"
        }
    }

    struct TestSubscriber {
        id: Uuid,
        received_events: Arc<Mutex<Vec<String>>>,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                id: Uuid::new_v4(),
                received_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn received_events(&self) -> Vec<String> {
            self.received_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventSubscriber for TestSubscriber {
        fn id(&self) -> Uuid {
            self.id
        }

        async fn handle_event<E: Event>(&self, event: &E) -> Result<(), EventError> {
            let test_event = event.as_any().downcast_ref::<TestEvent>();

            if let Some(e) = test_event {
                let mut received = self.received_events.lock().unwrap();
                received.push(e.test_data.clone());
            }

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        // Setup
        let publisher = InMemoryEventPublisher::new();
        let subscriber = Arc::new(TestSubscriber::new());

        // Subscribe
        publisher
            .subscribe("test.event", subscriber.clone())
            .await
            .unwrap();

        // Publish
        let event = TestEvent {
            id: Uuid::new_v4(),
            test_data: "Test message".to_string(),
        };

        publisher.publish_event(event).await.unwrap();

        // Verify
        let received = subscriber.received_events();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], "Test message");
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        // Setup
        let publisher = InMemoryEventPublisher::new();
        let subscriber = Arc::new(TestSubscriber::new());

        // Subscribe
        publisher
            .subscribe("test.event", subscriber.clone())
            .await
            .unwrap();

        // Unsubscribe
        publisher
            .unsubscribe("test.event", subscriber.id())
            .await
            .unwrap();

        // Publish
        let event = TestEvent {
            id: Uuid::new_v4(),
            test_data: "Test message".to_string(),
        };

        publisher.publish_event(event).await.unwrap();

        // Verify
        let received = subscriber.received_events();
        assert_eq!(received.len(), 0);
    }
}

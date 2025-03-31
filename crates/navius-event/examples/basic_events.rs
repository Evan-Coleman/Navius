use navius_event::{
    DeliveryStatus, Event, EventBrokerConfig, EventFilterConfig, EventPriority,
    SubscriptionOptions, create_memory_broker,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;

/// Example user event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserEvent {
    user_id: String,
    action: String,
    details: serde_json::Value,
}

/// Example system event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemEvent {
    component: String,
    status: String,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Navius Event System Example");
    println!("==========================\n");

    // Create an event broker
    let broker_config = EventBrokerConfig::new()
        .with_max_topics(100)
        .with_max_subscribers_per_topic(50)
        .with_event_retention_seconds(3600);

    println!("Creating event broker...");
    let broker = create_memory_broker().await?;

    // Create topics
    println!("Creating topics...");
    broker.create_topic("user.events").await?;
    broker.create_topic("system.events").await?;

    println!("Listing topics...");
    let topics = broker.list_topics().await?;
    for topic in &topics {
        println!("  - {}: {} subscribers", topic.name, topic.subscriber_count);
    }

    // Subscribe to user events
    println!("\nSubscribing to user events...");
    let user_subscription_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("user-events-subscription")
        .with_filter(
            EventFilterConfig::new()
                .with_min_priority(EventPriority::Normal)
                .with_sources(vec!["user-service"]),
        );

    let (user_sub_id, mut user_events) = broker
        .subscribe::<UserEvent>("user.events", user_subscription_options)
        .await?;

    println!("  Subscribed with ID: {}", user_sub_id);

    // Subscribe to system events
    println!("\nSubscribing to system events...");
    let system_subscription_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("system-events-subscription");

    let (system_sub_id, mut system_events) = broker
        .subscribe::<SystemEvent>("system.events", system_subscription_options)
        .await?;

    println!("  Subscribed with ID: {}", system_sub_id);

    // Subscription infos
    println!("\nSubscription information:");
    let user_sub_info = broker.get_subscription_info(&user_sub_id).await?;
    println!("  User subscription: {}", user_sub_info.id);
    println!("    Topic: {}", user_sub_info.topic);
    println!("    Created: {}", user_sub_info.created_at);

    let system_sub_info = broker.get_subscription_info(&system_sub_id).await?;
    println!("  System subscription: {}", system_sub_info.id);
    println!("    Topic: {}", system_sub_info.topic);
    println!("    Created: {}", system_sub_info.created_at);

    // Publish a user event
    println!("\nPublishing user event...");
    let user_event = Event::new(
        "user.login",
        "user.events",
        "user-service",
        UserEvent {
            user_id: "user123".to_string(),
            action: "login".to_string(),
            details: serde_json::json!({
                "ip": "192.168.1.1",
                "device": "mobile",
                "success": true,
            }),
        },
    )
    .with_correlation_id("session-abc-123")
    .with_metadata("region", "us-west")
    .with_metadata("app_version", "1.2.3");

    let delivery_status = broker.publish(user_event).await?;
    println!("  Delivery status: {:?}", delivery_status);

    // Publish a system event
    println!("\nPublishing system event...");
    let system_event = Event::with_priority(
        "system.startup",
        "system.events",
        "auth-service",
        SystemEvent {
            component: "auth-service".to_string(),
            status: "started".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        EventPriority::High,
    );

    let delivery_status = broker.publish(system_event).await?;
    println!("  Delivery status: {:?}", delivery_status);

    // Publish a JSON event
    println!("\nPublishing JSON event...");
    let json_status = broker
        .publish_json(
            "config.update",
            "system.events",
            "config-service",
            serde_json::json!({
                "component": "database",
                "status": "updated",
                "timestamp": chrono::Utc::now().timestamp(),
            }),
        )
        .await?;
    println!("  JSON event delivery status: {:?}", json_status);

    // Receive events
    println!("\nWaiting for events...");

    // Start task to consume user events
    let user_events_task = tokio::spawn(async move {
        while let Some(event_result) = user_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\nReceived user event:");
                    println!("  ID: {}", event.id);
                    println!("  Type: {}", event.event_type);
                    println!("  Priority: {:?}", event.priority);
                    println!("  User ID: {}", event.payload.user_id);
                    println!("  Action: {}", event.payload.action);
                    println!("  Details: {}", event.payload.details);
                    if let Some(corr_id) = &event.correlation_id {
                        println!("  Correlation ID: {}", corr_id);
                    }
                    println!("  Metadata:");
                    for (key, value) in &event.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving user event: {}", e);
                }
            }
        }
    });

    // Start task to consume system events
    let system_events_task = tokio::spawn(async move {
        while let Some(event_result) = system_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\nReceived system event:");
                    println!("  ID: {}", event.id);
                    println!("  Type: {}", event.event_type);
                    println!("  Priority: {:?}", event.priority);
                    println!("  Component: {}", event.payload.component);
                    println!("  Status: {}", event.payload.status);
                    println!("  Timestamp: {}", event.payload.timestamp);
                }
                Err(e) => {
                    eprintln!("Error receiving system event: {}", e);
                }
            }
        }
    });

    // Give time for events to be processed
    time::sleep(Duration::from_millis(300)).await;

    // Get recent events
    println!("\nGetting recent events for user.events:");
    let recent_user_events = broker.get_recent_events("user.events", 10).await?;
    println!("  Found {} events", recent_user_events.len());
    for event in &recent_user_events {
        println!("  - ID: {} Type: {}", event.id, event.event_type);
    }

    // Getting recent events with filter
    let filter = EventFilterConfig::new()
        .with_event_types(vec!["user.login".to_string()])
        .with_metadata("region", "us-west");

    println!("\nGetting filtered events for user.events:");
    let filtered_events = broker.filter_events("user.events", filter, 10).await?;
    println!("  Found {} filtered events", filtered_events.len());
    for event in &filtered_events {
        println!("  - ID: {} Type: {}", event.id, event.event_type);
    }

    // Broker health check
    println!("\nChecking broker health...");
    let health = broker.health_check().await?;
    println!("  Broker health: {}", health);

    // Give time for tasks to finish
    time::sleep(Duration::from_millis(500)).await;

    // Unsubscribe
    println!("\nUnsubscribing from events...");
    broker.unsubscribe(&user_sub_id).await?;
    broker.unsubscribe(&system_sub_id).await?;

    // Abort the event receiver tasks
    user_events_task.abort();
    system_events_task.abort();

    println!("\nExample complete!");

    Ok(())
}

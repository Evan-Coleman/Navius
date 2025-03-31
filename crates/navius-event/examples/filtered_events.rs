use navius_event::{
    Event, EventBrokerConfig, EventFilterConfig, EventPriority, SubscriptionOptions,
    create_memory_broker,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{task, time};

/// Notification event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationEvent {
    notification_type: String,
    title: String,
    message: String,
    user_ids: Vec<String>,
    channel: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Navius Event System - Filtered Events Example");
    println!("============================================\n");

    // Create an event broker
    println!("Creating event broker...");
    let broker = create_memory_broker().await?;

    // Create topics
    println!("Creating notification topic...");
    broker.create_topic("notifications").await?;

    // Subscribe with different filters
    println!("\nCreating subscriptions with different filters...");

    // Subscription 1: High priority email notifications only
    let high_priority_email_filter = EventFilterConfig::new()
        .with_min_priority(EventPriority::High)
        .with_event_types(vec!["notification.email"]);

    let email_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("high-priority-email-subscription")
        .with_filter(high_priority_email_filter);

    let (email_sub_id, mut email_events) = broker
        .subscribe::<NotificationEvent>("notifications", email_options)
        .await?;

    println!(
        "  - Created high priority email subscription: {}",
        email_sub_id
    );

    // Subscription 2: User-specific notifications
    let user_specific_filter = EventFilterConfig::new().with_metadata("target_user", "user123");

    let user_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("user123-subscription")
        .with_filter(user_specific_filter);

    let (user_sub_id, mut user_events) = broker
        .subscribe::<NotificationEvent>("notifications", user_options)
        .await?;

    println!("  - Created user-specific subscription: {}", user_sub_id);

    // Subscription 3: All push notifications (regardless of priority)
    let push_filter = EventFilterConfig::new().with_event_types(vec!["notification.push"]);

    let push_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("push-subscription")
        .with_filter(push_filter);

    let (push_sub_id, mut push_events) = broker
        .subscribe::<NotificationEvent>("notifications", push_options)
        .await?;

    println!(
        "  - Created push notification subscription: {}",
        push_sub_id
    );

    // Subscription 4: Marketing notifications with correlation tracking
    let marketing_filter = EventFilterConfig::new()
        .with_event_types(vec!["notification.marketing"])
        .with_correlation_id("campaign-2025-spring");

    let marketing_options = SubscriptionOptions::new()
        .with_buffer_size(10)
        .with_name("marketing-subscription")
        .with_filter(marketing_filter);

    let (marketing_sub_id, mut marketing_events) = broker
        .subscribe::<NotificationEvent>("notifications", marketing_options)
        .await?;

    println!(
        "  - Created marketing campaign subscription: {}",
        marketing_sub_id
    );

    // Spawn tasks to handle each subscription stream
    let email_task = task::spawn(async move {
        println!("\nListening for high priority email notifications...");
        while let Some(event_result) = email_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\n🔔 HIGH PRIORITY EMAIL NOTIFICATION:");
                    println!("  Title: {}", event.payload.title);
                    println!("  Message: {}", event.payload.message);
                    println!("  Channel: {}", event.payload.channel);
                    println!("  Users: {:?}", event.payload.user_ids);
                }
                Err(e) => eprintln!("Error receiving email event: {}", e),
            }
        }
    });

    let user_task = task::spawn(async move {
        println!("\nListening for user123 notifications...");
        while let Some(event_result) = user_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\n👤 USER-SPECIFIC NOTIFICATION:");
                    println!("  Type: {}", event.payload.notification_type);
                    println!("  Title: {}", event.payload.title);
                    println!("  Message: {}", event.payload.message);
                    println!("  Channel: {}", event.payload.channel);
                }
                Err(e) => eprintln!("Error receiving user event: {}", e),
            }
        }
    });

    let push_task = task::spawn(async move {
        println!("\nListening for all push notifications...");
        while let Some(event_result) = push_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\n📱 PUSH NOTIFICATION:");
                    println!("  Priority: {:?}", event.priority);
                    println!("  Title: {}", event.payload.title);
                    println!("  Message: {}", event.payload.message);
                    println!("  Users: {:?}", event.payload.user_ids);
                }
                Err(e) => eprintln!("Error receiving push event: {}", e),
            }
        }
    });

    let marketing_task = task::spawn(async move {
        println!("\nListening for spring campaign marketing notifications...");
        while let Some(event_result) = marketing_events.next().await {
            match event_result {
                Ok(event) => {
                    println!("\n🔊 MARKETING NOTIFICATION (SPRING CAMPAIGN):");
                    println!("  Title: {}", event.payload.title);
                    println!("  Message: {}", event.payload.message);
                    println!("  Channel: {}", event.payload.channel);
                    if let Some(corr_id) = &event.correlation_id {
                        println!("  Campaign ID: {}", corr_id);
                    }
                }
                Err(e) => eprintln!("Error receiving marketing event: {}", e),
            }
        }
    });

    // Give some time for subscribers to set up
    time::sleep(Duration::from_millis(100)).await;

    // Publish some events
    println!("\nPublishing notification events with different characteristics...");

    // Event 1: High priority email notification
    let event1 = Event::with_priority(
        "notification.email",
        "notifications",
        "notification-service",
        NotificationEvent {
            notification_type: "account_security".to_string(),
            title: "Security Alert".to_string(),
            message: "Your account password was changed".to_string(),
            user_ids: vec!["user456".to_string()],
            channel: "email".to_string(),
        },
        EventPriority::High,
    );
    broker.publish(event1).await?;
    println!("  ✓ Published high priority email notification");

    // Event 2: User-specific push notification with normal priority
    let event2 = Event::new(
        "notification.push",
        "notifications",
        "notification-service",
        NotificationEvent {
            notification_type: "chat_message".to_string(),
            title: "New Message".to_string(),
            message: "You have a new message from Alice".to_string(),
            user_ids: vec!["user123".to_string()],
            channel: "push".to_string(),
        },
    )
    .with_metadata("target_user", "user123");
    broker.publish(event2).await?;
    println!("  ✓ Published user-specific push notification");

    // Event 3: Normal priority email notification (won't match the high priority filter)
    let event3 = Event::new(
        "notification.email",
        "notifications",
        "notification-service",
        NotificationEvent {
            notification_type: "newsletter".to_string(),
            title: "Weekly Newsletter".to_string(),
            message: "Check out what's new this week".to_string(),
            user_ids: vec!["user123".to_string(), "user456".to_string()],
            channel: "email".to_string(),
        },
    );
    broker.publish(event3).await?;
    println!("  ✓ Published normal priority email notification");

    // Event 4: Marketing notification with correlation ID
    let event4 = Event::new(
        "notification.marketing",
        "notifications",
        "marketing-service",
        NotificationEvent {
            notification_type: "promotion".to_string(),
            title: "Spring Sale".to_string(),
            message: "Save 20% on all products".to_string(),
            user_ids: vec!["user123".to_string(), "user456".to_string()],
            channel: "email".to_string(),
        },
    )
    .with_correlation_id("campaign-2025-spring");
    broker.publish(event4).await?;
    println!("  ✓ Published spring campaign marketing notification");

    // Event 5: Marketing notification with different correlation ID (won't match filter)
    let event5 = Event::new(
        "notification.marketing",
        "notifications",
        "marketing-service",
        NotificationEvent {
            notification_type: "promotion".to_string(),
            title: "New User Offer".to_string(),
            message: "Welcome gift for new users".to_string(),
            user_ids: vec!["user789".to_string()],
            channel: "email".to_string(),
        },
    )
    .with_correlation_id("campaign-new-users");
    broker.publish(event5).await?;
    println!("  ✓ Published new user campaign marketing notification");

    // Wait for all events to be processed
    println!("\nWaiting for event processing...");
    time::sleep(Duration::from_secs(1)).await;

    // Unsubscribe and clean up
    println!("\nCleaning up subscriptions...");
    broker.unsubscribe(&email_sub_id).await?;
    broker.unsubscribe(&user_sub_id).await?;
    broker.unsubscribe(&push_sub_id).await?;
    broker.unsubscribe(&marketing_sub_id).await?;

    // Abort all tasks
    email_task.abort();
    user_task.abort();
    push_task.abort();
    marketing_task.abort();

    println!("\nExample complete!");

    Ok(())
}

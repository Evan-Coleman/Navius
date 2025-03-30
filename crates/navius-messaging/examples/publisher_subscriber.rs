use navius_messaging::MessageEnvelope;
use navius_messaging::config::{BrokerConfig, MessagingConfig};
use navius_messaging::helpers::init_messaging;
use navius_messaging::publisher::PublishOptions;
use navius_messaging::subscriber::SubscribeOptions;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// This example demonstrates a complete messaging cycle with both publishing and subscribing.
/// It creates a publisher and subscriber, subscribes to a topic, publishes messages, and
/// processes the received messages.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("Starting complete messaging example");

    // Create a broker configuration
    let broker_config = BrokerConfig {
        provider: "memory".to_string(),
        connection_string: "memory://local".to_string(),
        ..Default::default()
    };

    // Create the messaging configuration
    let mut config = MessagingConfig::default();
    config.default = broker_config;

    // Initialize the messaging manager
    let manager = Arc::new(init_messaging(config));

    // Register a memory provider
    let provider = navius_messaging_memory::MemoryProvider::new();
    manager.register_provider(provider);

    // Create a channel to communicate received messages
    let (tx, mut rx) = mpsc::channel(100);

    // Create a subscriber and publisher
    let subscriber = manager.create_subscriber("default").await?;
    let publisher = manager.create_publisher("default").await?;

    // Subscribe options
    let subscribe_options = SubscribeOptions::default()
        .with_exchange("examples")
        .with_routing_key("complete.#")
        .with_queue("complete-queue")
        .with_durable(true);

    println!("Subscribing to messages with routing key 'complete.#'");

    // Clone the sender for use in the subscription callback
    let tx_clone = tx.clone();

    // Subscribe to messages
    let subscription = subscriber
        .subscribe_fn(&subscribe_options, move |message, ack, _reject| {
            let tx = tx_clone.clone();
            async move {
                // Send the message to the channel
                if let Err(e) = tx.send(message.clone()).await {
                    eprintln!("Failed to send message to channel: {}", e);
                }

                // Acknowledge the message
                ack().await
            }
        })
        .await?;

    println!("Subscription set up successfully");

    // Publish options
    let publish_options = PublishOptions::default()
        .with_exchange("examples")
        .with_routing_key("complete.test");

    // Spawn a task for publishing messages
    let publisher_clone = publisher.clone();
    let publish_options_clone = publish_options.clone();

    let publisher_task = tokio::spawn(async move {
        for i in 1..=5 {
            // Create a message envelope
            let payload = format!("Test message {}", i);
            let properties = navius_messaging::MessageProperties::new()
                .with_content_type("text/plain")
                .with_message_type("test")
                .with_header("sequence", i.to_string())
                .with_persistent(true);

            let envelope =
                MessageEnvelope::with_properties(payload.as_bytes().to_vec(), properties);

            // Publish the message
            match publisher_clone
                .publish_envelope(envelope, &publish_options_clone)
                .await
            {
                Ok(_) => println!("Published message {}", i),
                Err(e) => eprintln!("Failed to publish message {}: {}", i, e),
            }

            // Wait a bit between messages
            sleep(Duration::from_millis(200)).await;
        }

        // Signal completion with a final message
        let final_message = "All messages sent!";
        match publisher_clone
            .publish(final_message.as_bytes().to_vec(), &publish_options_clone)
            .await
        {
            Ok(_) => println!("Published final message"),
            Err(e) => eprintln!("Failed to publish final message: {}", e),
        }
    });

    // Spawn a task for receiving messages
    let receiver_task = tokio::spawn(async move {
        let mut count = 0;

        // Receive messages with a timeout
        while let Ok(message) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
            match message {
                Some(msg) => {
                    count += 1;

                    // Convert payload to string
                    let payload = String::from_utf8_lossy(&msg.payload);

                    println!("Received message {}:", count);
                    println!("  ID: {}", msg.id);
                    println!("  Payload: {}", payload);

                    // Print headers if any
                    if !msg.properties.headers.is_empty() {
                        println!("  Headers:");
                        for (key, value) in &msg.properties.headers {
                            println!("    {}: {}", key, value);
                        }
                    }

                    if payload == "All messages sent!" {
                        println!("Received final message, exiting");
                        break;
                    }
                }
                None => {
                    println!("Channel closed, exiting");
                    break;
                }
            }
        }

        println!("Received {} messages in total", count);
    });

    // Wait for both tasks to complete
    publisher_task.await?;
    receiver_task.await?;

    println!("All messages processed");

    // Clean up
    subscriber.unsubscribe(subscription).await?;
    subscriber.close().await?;
    publisher.close().await?;

    // Shutdown the manager
    manager.shutdown().await?;
    println!("Messaging manager shut down successfully");

    Ok(())
}

// Note: This example requires the `navius-messaging-memory` crate, which is a memory-based
// implementation of the messaging provider interface. You would replace it with a real provider
// like `navius-messaging-rabbitmq` in a production environment.

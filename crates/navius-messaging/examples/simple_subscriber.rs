use navius_messaging::config::{BrokerConfig, MessagingConfig};
use navius_messaging::helpers::init_messaging;
use navius_messaging::subscriber::SubscribeOptions;
use std::error::Error;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;

/// A simple example that demonstrates how to create a subscriber and process messages.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

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
    let manager = init_messaging(config);

    // Register a memory provider
    let provider = navius_messaging_memory::MemoryProvider::new();
    manager.register_provider(provider);

    // Create a subscriber
    let subscriber = manager.create_subscriber("default").await?;

    // Subscribe to messages
    let options = SubscribeOptions::default()
        .with_exchange("examples")
        .with_routing_key("simple")
        .with_queue("simple-queue")
        .with_durable(true);

    println!("Subscribing to exchange 'examples' with routing key 'simple'");

    let subscription = subscriber
        .subscribe_fn(&options, |message, ack, reject| async move {
            // Extract message payload as a string
            let payload = if let Ok(text) = String::from_utf8(message.payload.clone()) {
                text
            } else {
                let hex_payload = message
                    .payload
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join("");
                format!("<binary: {}>", hex_payload)
            };

            println!("Received message: {}", payload);
            println!("Message ID: {}", message.id);
            println!("Created at: {}", message.created_at);

            // Print message properties if any
            if !message.properties.headers.is_empty() {
                println!("Headers:");
                for (key, value) in &message.properties.headers {
                    println!("  {}: {}", key, value);
                }
            }

            if let Some(content_type) = &message.properties.content_type {
                println!("Content-Type: {}", content_type);
            }

            if let Some(message_type) = &message.properties.message_type {
                println!("Message-Type: {}", message_type);
            }

            // Acknowledge the message
            println!("Acknowledging message");
            ack().await
        })
        .await?;

    println!("Subscription established, waiting for messages...");
    println!("Press Ctrl+C to exit");

    // Wait for Ctrl+C signal
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("Ctrl+C received, shutting down");
        }
        Err(err) => {
            eprintln!("Error listening for Ctrl+C: {}", err);
        }
    }

    // Unsubscribe and close
    subscriber.unsubscribe(subscription).await?;
    subscriber.close().await?;

    // Shutdown the manager
    manager.shutdown().await?;
    println!("Messaging manager shut down successfully");

    Ok(())
}

// Note: This example requires the `navius-messaging-memory` crate, which is a memory-based
// implementation of the messaging provider interface. You would replace it with a real provider
// like `navius-messaging-rabbitmq` in a production environment.

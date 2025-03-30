use navius_messaging::config::{BrokerConfig, MessagingConfig};
use navius_messaging::helpers::init_messaging;
use navius_messaging::publisher::PublishOptions;
use std::error::Error;

/// A simple example that demonstrates how to create a publisher and publish messages.
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

    // Get a broker
    let broker = manager.get_broker("default").await?;

    // Create a publisher
    let publisher = manager.create_publisher("default").await?;

    // Publish a message with default options
    let options = PublishOptions::default()
        .with_exchange("examples")
        .with_routing_key("simple");

    let payload = "Hello, Navius Messaging!".as_bytes().to_vec();
    publisher.publish(payload, &options).await?;

    println!("Published message to exchange 'examples' with routing key 'simple'");

    // Create a message envelope with properties and publish it
    let properties = navius_messaging::MessageProperties::new()
        .with_content_type("text/plain")
        .with_message_type("greeting")
        .with_header("app", "example")
        .with_persistent(true);

    let envelope = navius_messaging::MessageEnvelope::with_properties(
        "Hello with properties!".as_bytes().to_vec(),
        properties,
    );

    publisher.publish_envelope(envelope, &options).await?;

    println!("Published message envelope with properties");

    // Close the publisher and shut down
    publisher.close().await?;
    manager.shutdown().await?;

    println!("Messaging manager shut down successfully");

    Ok(())
}

// Note: This example requires the `navius-messaging-memory` crate, which is a memory-based
// implementation of the messaging provider interface. You would replace it with a real provider
// like `navius-messaging-rabbitmq` in a production environment.

use std::sync::Arc;
use std::time::Duration;

use navius_messaging::broker::TopologyBuilder;
use navius_messaging::config::BrokerConfig;
use navius_messaging::consumer::ConsumerOptions;
use navius_messaging::message::{Message, MessageAcknowledgment, MessageProcessingResult};
use navius_messaging::publisher::PublishOptions;
use navius_messaging::serialization::JsonSerializer;
use navius_messaging::util::{RequestReply, generate_correlation_id, generate_id};

// Our example message payload for request
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct UserInfoRequest {
    user_id: String,
    request_id: String,
    fields: Vec<String>,
}

// Our example message payload for response
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct UserInfoResponse {
    user_id: String,
    request_id: String,
    data: serde_json::Value,
    success: bool,
    error: Option<String>,
}

// Use the in-memory broker from the basic example
mod memory_broker {
    // For brevity, reuse the same in-memory broker implementation
    // from the basic_example.rs file
    // In a real application, you would import or include the implementation here
    include!("basic_example.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a broker config
    let config = BrokerConfig::new("in-memory-broker", "InMemory Broker Example", "in-memory");

    // Create a broker factory
    let factory = memory_broker::InMemoryBrokerFactory;

    // Create the broker
    let broker = factory.create_broker(config).await?;

    // Connect to the broker
    println!("Connecting to the broker...");
    broker.connect().await?;
    println!("Connected!");

    // Create the basic topology
    println!("Setting up messaging topology...");
    let builder = TopologyBuilder::new(broker.clone());

    // Create request and reply queues
    builder
        .declare_queue("user-info-requests", false, true, false, None)
        .await?;

    // Create request/reply client
    let request_reply = RequestReply::new(
        broker.clone(),
        "user-info-requests",
        Duration::from_secs(10),
    );

    // Start the service (in a separate task)
    tokio::spawn({
        let broker = broker.clone();
        async move {
            // Define handler for user info requests
            let user_info_handler = |message: &navius_messaging::message::ReceivedMessage<
                UserInfoRequest,
            >|
             -> MessageProcessingResult {
                let request = &message.message.payload;
                println!(
                    "Service received request: user_id={}, request_id={}, fields={:?}",
                    request.user_id, request.request_id, request.fields
                );

                // Build user data (in a real app, would fetch from database)
                let user_data = match request.user_id.as_str() {
                    "user123" => {
                        let mut data = serde_json::Map::new();

                        // Only include requested fields
                        if request.fields.contains(&"name".to_string()) {
                            data.insert(
                                "name".to_string(),
                                serde_json::Value::String("John Doe".to_string()),
                            );
                        }

                        if request.fields.contains(&"email".to_string()) {
                            data.insert(
                                "email".to_string(),
                                serde_json::Value::String("john@example.com".to_string()),
                            );
                        }

                        if request.fields.contains(&"role".to_string()) {
                            data.insert(
                                "role".to_string(),
                                serde_json::Value::String("admin".to_string()),
                            );
                        }

                        serde_json::Value::Object(data)
                    }
                    "user456" => {
                        let mut data = serde_json::Map::new();

                        // Only include requested fields
                        if request.fields.contains(&"name".to_string()) {
                            data.insert(
                                "name".to_string(),
                                serde_json::Value::String("Jane Smith".to_string()),
                            );
                        }

                        if request.fields.contains(&"email".to_string()) {
                            data.insert(
                                "email".to_string(),
                                serde_json::Value::String("jane@example.com".to_string()),
                            );
                        }

                        if request.fields.contains(&"role".to_string()) {
                            data.insert(
                                "role".to_string(),
                                serde_json::Value::String("user".to_string()),
                            );
                        }

                        serde_json::Value::Object(data)
                    }
                    _ => {
                        // User not found
                        serde_json::Value::Null
                    }
                };

                // Create the response
                let response = UserInfoResponse {
                    user_id: request.user_id.clone(),
                    request_id: request.request_id.clone(),
                    data: user_data.clone(),
                    success: user_data != serde_json::Value::Null,
                    error: if user_data == serde_json::Value::Null {
                        Some("User not found".to_string())
                    } else {
                        None
                    },
                };

                // Create the response message
                let response_message = Message::new(response, "user-info-responses")
                    .with_correlation_id(
                        message.message.correlation_id.clone().unwrap_or_default(),
                    );

                // Get the reply-to topic from the original message
                let reply_to = message.message.reply_to.clone();

                if let Some(reply_to) = reply_to {
                    // Publish the response
                    let publish_options = PublishOptions::new("").with_routing_key(reply_to);

                    // Publish with confirmation
                    match broker
                        .publish_with_confirm(
                            &response_message,
                            Some(publish_options),
                            Some(Duration::from_secs(5)),
                        )
                        .await
                    {
                        Ok(_) => {
                            println!(
                                "Service sent response for request_id={}",
                                request.request_id
                            );
                        }
                        Err(e) => {
                            eprintln!("Error sending response: {}", e);
                        }
                    }
                } else {
                    eprintln!("No reply_to in the request, cannot respond!");
                }

                // Acknowledge the request message
                Ok(MessageAcknowledgment::Ack)
            };

            // Subscribe to process user info requests
            let consumer_options =
                ConsumerOptions::default().with_consumer_tag("user-info-service");

            let consumer_handle = broker
                .subscribe(
                    "user-info-requests",
                    user_info_handler,
                    Some(consumer_options),
                )
                .await
                .expect("Failed to create consumer");

            println!("Service started and listening for requests");

            // Keep the service running
            tokio::signal::ctrl_c().await.ok();

            // Clean shutdown
            consumer_handle.cancel().await.ok();
        }
    });

    // Wait for service to initialize
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send requests as a client
    println!("\nSending requests as client...");

    // Request 1: Get name and email for existing user
    let request1 = UserInfoRequest {
        user_id: "user123".to_string(),
        request_id: generate_id(),
        fields: vec!["name".to_string(), "email".to_string()],
    };

    println!(
        "Sending request for user_id={} (fields: {:?})",
        request1.user_id, request1.fields
    );
    let response1: UserInfoResponse = request_reply.request(&request1, "user.info", None).await?;

    println!("Received response for user_id={}", response1.user_id);
    println!("  Success: {}", response1.success);
    println!("  Data: {}", response1.data);
    if let Some(error) = response1.error {
        println!("  Error: {}", error);
    }

    // Request 2: Get role for another existing user
    let request2 = UserInfoRequest {
        user_id: "user456".to_string(),
        request_id: generate_id(),
        fields: vec!["role".to_string()],
    };

    println!(
        "\nSending request for user_id={} (fields: {:?})",
        request2.user_id, request2.fields
    );
    let response2: UserInfoResponse = request_reply.request(&request2, "user.info", None).await?;

    println!("Received response for user_id={}", response2.user_id);
    println!("  Success: {}", response2.success);
    println!("  Data: {}", response2.data);
    if let Some(error) = response2.error {
        println!("  Error: {}", error);
    }

    // Request 3: Get info for non-existent user
    let request3 = UserInfoRequest {
        user_id: "invalid-user".to_string(),
        request_id: generate_id(),
        fields: vec!["name".to_string(), "email".to_string(), "role".to_string()],
    };

    println!(
        "\nSending request for user_id={} (fields: {:?})",
        request3.user_id, request3.fields
    );
    let response3: UserInfoResponse = request_reply.request(&request3, "user.info", None).await?;

    println!("Received response for user_id={}", response3.user_id);
    println!("  Success: {}", response3.success);
    println!("  Data: {}", response3.data);
    if let Some(error) = response3.error {
        println!("  Error: {}", error);
    }

    // Get metrics
    let metrics = broker.metrics().await;
    println!("\nBroker metrics:");
    println!("- Messages published: {}", metrics.published_messages);
    println!("- Messages consumed: {}", metrics.consumed_messages);
    println!("- Messages acknowledged: {}", metrics.acknowledged_messages);

    // Disconnect from the broker
    println!("\nDisconnecting from the broker...");
    broker.disconnect().await?;
    println!("Disconnected!");

    println!("Example completed successfully!");
    Ok(())
}

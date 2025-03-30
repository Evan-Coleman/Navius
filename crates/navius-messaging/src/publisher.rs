use crate::error::MessagingResult;
use crate::message::{Message, MessageEnvelope, MessageProperties};
use async_trait::async_trait;
use std::time::Duration;

/// Options for publishing messages.
#[derive(Debug, Clone)]
pub struct PublishOptions {
    /// The exchange or topic to publish to
    pub exchange: String,

    /// The routing key for the message
    pub routing_key: String,

    /// Whether the message should be persistent
    pub persistent: bool,

    /// Whether to wait for server confirmation
    pub confirm: bool,

    /// Timeout for the publish operation
    pub timeout: Option<Duration>,

    /// Whether to publish with mandatory flag (AMQP)
    pub mandatory: bool,

    /// Whether to publish with immediate flag (AMQP)
    pub immediate: bool,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            exchange: "".to_string(), // Default exchange
            routing_key: "".to_string(),
            persistent: true,
            confirm: true,
            timeout: None,
            mandatory: false,
            immediate: false,
        }
    }
}

impl PublishOptions {
    /// Create new publish options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the exchange to publish to
    pub fn with_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = exchange.into();
        self
    }

    /// Set the routing key
    pub fn with_routing_key(mut self, routing_key: impl Into<String>) -> Self {
        self.routing_key = routing_key.into();
        self
    }

    /// Set whether the message should be persistent
    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// Set whether to wait for server confirmation
    pub fn with_confirm(mut self, confirm: bool) -> Self {
        self.confirm = confirm;
        self
    }

    /// Set the timeout for the publish operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to publish with mandatory flag
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Set whether to publish with immediate flag
    pub fn with_immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }
}

/// A trait for publishing messages.
#[async_trait]
pub trait MessagePublisher: Send + Sync {
    /// Publish a message with the given options.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    /// * `options` - Options for publishing the message
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn publish<T: Message>(&self, message: T, options: PublishOptions)
    -> MessagingResult<()>;

    /// Publish a message envelope with the given options.
    ///
    /// # Arguments
    /// * `envelope` - The message envelope to publish
    /// * `options` - Options for publishing the message
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn publish_envelope<T: Message>(
        &self,
        envelope: MessageEnvelope<T>,
        options: PublishOptions,
    ) -> MessagingResult<()>;

    /// Publish a message with default options.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    /// * `routing_key` - The routing key for the message
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn publish_simple<T: Message>(
        &self,
        message: T,
        routing_key: impl Into<String>,
    ) -> MessagingResult<()> {
        let options = PublishOptions::default().with_routing_key(routing_key);
        self.publish(message, options).await
    }

    /// Create a message envelope from the given message and publish it with default options.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    /// * `routing_key` - The routing key for the message
    /// * `properties` - The message properties
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn publish_with_properties<T: Message>(
        &self,
        message: T,
        routing_key: impl Into<String>,
        properties: MessageProperties,
    ) -> MessagingResult<()> {
        let envelope = MessageEnvelope::with_properties(message, properties);
        let options = PublishOptions::default().with_routing_key(routing_key);
        self.publish_envelope(envelope, options).await
    }

    /// Close the publisher and release any resources.
    async fn close(&self) -> MessagingResult<()>;
}

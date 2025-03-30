use crate::error::MessagingResult;
use crate::message::{Message, MessageEnvelope, RawMessage};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Options for subscribing to messages.
#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    /// The exchange or topic to subscribe to
    pub exchange: String,

    /// The routing key pattern for the subscription
    pub routing_key: String,

    /// The queue name to use
    pub queue: Option<String>,

    /// Whether to use a durable queue
    pub durable: bool,

    /// Whether to auto-delete the queue when no longer used
    pub auto_delete: bool,

    /// Whether to use an exclusive queue
    pub exclusive: bool,

    /// The prefetch count
    pub prefetch_count: Option<u16>,

    /// Whether to auto-acknowledge messages
    pub auto_ack: bool,

    /// Additional queue arguments
    pub arguments: std::collections::HashMap<String, String>,

    /// Timeout for the subscribe operation
    pub timeout: Option<Duration>,
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            exchange: "".to_string(),
            routing_key: "#".to_string(),
            queue: None,
            durable: true,
            auto_delete: false,
            exclusive: false,
            prefetch_count: None,
            auto_ack: false,
            arguments: std::collections::HashMap::new(),
            timeout: None,
        }
    }
}

impl SubscribeOptions {
    /// Create new subscribe options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the exchange to subscribe to
    pub fn with_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = exchange.into();
        self
    }

    /// Set the routing key pattern
    pub fn with_routing_key(mut self, routing_key: impl Into<String>) -> Self {
        self.routing_key = routing_key.into();
        self
    }

    /// Set the queue name
    pub fn with_queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = Some(queue.into());
        self
    }

    /// Set whether to use a durable queue
    pub fn with_durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set whether to auto-delete the queue
    pub fn with_auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Set whether to use an exclusive queue
    pub fn with_exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    /// Set the prefetch count
    pub fn with_prefetch_count(mut self, prefetch_count: u16) -> Self {
        self.prefetch_count = Some(prefetch_count);
        self
    }

    /// Set whether to auto-acknowledge messages
    pub fn with_auto_ack(mut self, auto_ack: bool) -> Self {
        self.auto_ack = auto_ack;
        self
    }

    /// Add a queue argument
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set the timeout for the subscribe operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Type for message acknowledgment results.
pub type MessageAckResult = Result<(), MessageAckError>;

/// Error types for message acknowledgment.
#[derive(Debug, thiserror::Error)]
pub enum MessageAckError {
    /// Failed to acknowledge a message.
    #[error("Failed to acknowledge message: {0}")]
    AckFailed(String),

    /// Failed to reject a message.
    #[error("Failed to reject message: {0}")]
    RejectFailed(String),
}

/// A trait for handlers that process messages.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// The message type this handler can process.
    type Message: Message;

    /// Handle a message.
    ///
    /// # Arguments
    /// * `message` - The message to handle
    /// * `ack` - Function to acknowledge the message
    /// * `reject` - Function to reject the message
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn handle(
        &self,
        message: MessageEnvelope<Self::Message>,
        ack: Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
        reject: Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
    ) -> MessagingResult<()>;
}

/// Create a message handler from an async function.
pub fn handler_fn<T, F, Fut>(f: F) -> MessageHandlerFn<T, F>
where
    T: Message,
    F: Fn(
            MessageEnvelope<T>,
            Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
            Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
        ) -> Fut
        + Send
        + Sync,
    Fut: Future<Output = MessagingResult<()>> + Send,
{
    MessageHandlerFn {
        f,
        _marker: std::marker::PhantomData,
    }
}

/// A message handler implemented as a function.
pub struct MessageHandlerFn<T, F> {
    f: F,
    _marker: std::marker::PhantomData<fn(T)>,
}

#[async_trait]
impl<T, F, Fut> MessageHandler for MessageHandlerFn<T, F>
where
    T: Message,
    F: Fn(
            MessageEnvelope<T>,
            Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
            Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
        ) -> Fut
        + Send
        + Sync,
    Fut: Future<Output = MessagingResult<()>> + Send,
{
    type Message = T;

    async fn handle(
        &self,
        message: MessageEnvelope<Self::Message>,
        ack: Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
        reject: Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
    ) -> MessagingResult<()> {
        (self.f)(message, ack, reject).await
    }
}

/// A trait for subscribing to messages.
#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    /// Subscribe to messages with a handler.
    ///
    /// # Arguments
    /// * `handler` - The handler to process messages
    /// * `options` - Options for subscribing to messages
    ///
    /// # Returns
    /// A result with the subscription ID if successful
    async fn subscribe<H, T>(
        &self,
        handler: H,
        options: SubscribeOptions,
    ) -> MessagingResult<String>
    where
        H: MessageHandler<Message = T> + 'static,
        T: Message + 'static;

    /// Subscribe to messages with a closure.
    ///
    /// # Arguments
    /// * `handler` - The closure to process messages
    /// * `options` - Options for subscribing to messages
    ///
    /// # Returns
    /// A result with the subscription ID if successful
    async fn subscribe_fn<F, Fut, T>(
        &self,
        handler: F,
        options: SubscribeOptions,
    ) -> MessagingResult<String>
    where
        F: Fn(
                MessageEnvelope<T>,
                Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
                Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
            ) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = MessagingResult<()>> + Send,
        T: Message + 'static;

    /// Subscribe to raw messages.
    ///
    /// # Arguments
    /// * `handler` - The function to process raw messages
    /// * `options` - Options for subscribing to messages
    ///
    /// # Returns
    /// A result with the subscription ID if successful
    async fn subscribe_raw<F, Fut>(
        &self,
        handler: F,
        options: SubscribeOptions,
    ) -> MessagingResult<String>
    where
        F: Fn(
                RawMessage,
                Box<dyn FnOnce() -> MessageAckResult + Send + Sync>,
                Box<dyn FnOnce(bool) -> MessageAckResult + Send + Sync>,
            ) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = MessagingResult<()>> + Send;

    /// Subscribe to messages with simple acknowledge semantics.
    ///
    /// In this variant, the message is automatically acknowledged when the handler
    /// returns successfully, and rejected when it returns an error.
    ///
    /// # Arguments
    /// * `handler` - The function to process messages
    /// * `options` - Options for subscribing to messages
    ///
    /// # Returns
    /// A result with the subscription ID if successful
    async fn subscribe_auto<F, Fut, T>(
        &self,
        handler: F,
        options: SubscribeOptions,
    ) -> MessagingResult<String>
    where
        F: Fn(MessageEnvelope<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = MessagingResult<()>> + Send,
        T: Message + 'static;

    /// Unsubscribe from a subscription.
    ///
    /// # Arguments
    /// * `subscription_id` - The ID of the subscription to unsubscribe from
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn unsubscribe(&self, subscription_id: &str) -> MessagingResult<()>;

    /// Close the subscriber and release any resources.
    async fn close(&self) -> MessagingResult<()>;
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::broker::MessageBroker;
use crate::error::{DeliveryMode, MessagingError, MessagingResult};
use crate::message::Message;

/// Options for publishing messages
#[derive(Debug, Clone)]
pub struct PublishOptions {
    /// Exchange to publish to
    pub exchange: String,

    /// Routing key to use
    pub routing_key: Option<String>,

    /// Mandatory flag (message must be routable)
    pub mandatory: bool,

    /// Immediate flag (message must be deliverable immediately)
    pub immediate: bool,

    /// Message delivery mode
    pub delivery_mode: DeliveryMode,

    /// Message expiration in milliseconds
    pub expiration_ms: Option<u64>,

    /// Message priority (0-255, higher is more priority)
    pub priority: Option<u8>,

    /// Wait for confirmation
    pub wait_for_confirm: bool,

    /// Confirmation timeout
    pub confirm_timeout: Option<Duration>,

    /// Additional headers to include
    pub headers: HashMap<String, String>,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            exchange: "".to_string(), // Default exchange
            routing_key: None,
            mandatory: false,
            immediate: false,
            delivery_mode: DeliveryMode::Persistent,
            expiration_ms: None,
            priority: None,
            wait_for_confirm: false,
            confirm_timeout: Some(Duration::from_secs(5)),
            headers: HashMap::new(),
        }
    }
}

impl PublishOptions {
    /// Create new publish options with the given exchange
    pub fn new(exchange: impl Into<String>) -> Self {
        Self {
            exchange: exchange.into(),
            ..Default::default()
        }
    }

    /// Set the routing key
    pub fn with_routing_key(mut self, routing_key: impl Into<String>) -> Self {
        self.routing_key = Some(routing_key.into());
        self
    }

    /// Set the mandatory flag
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Set the immediate flag
    pub fn with_immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }

    /// Set the delivery mode
    pub fn with_delivery_mode(mut self, mode: DeliveryMode) -> Self {
        self.delivery_mode = mode;
        self
    }

    /// Set message expiration
    pub fn with_expiration(mut self, expiration_ms: u64) -> Self {
        self.expiration_ms = Some(expiration_ms);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set confirmation options
    pub fn with_confirm(mut self, wait_for_confirm: bool, timeout: Option<Duration>) -> Self {
        self.wait_for_confirm = wait_for_confirm;
        self.confirm_timeout = timeout;
        self
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add multiple headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }
}

/// Result of a message publish operation
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// Whether the message was successfully published
    pub success: bool,

    /// Message ID
    pub message_id: String,

    /// Error message, if any
    pub error: Option<String>,

    /// Confirmation ID from the broker, if available
    pub confirmation_id: Option<String>,

    /// Time taken to publish
    pub publish_time: Duration,
}

/// Trait for message publishers
#[async_trait]
pub trait MessagePublisher: Send + Sync {
    /// Publish a message
    async fn publish<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult>;

    /// Publish a message and wait for confirmation
    async fn publish_with_confirm<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult>;
}

/// A publisher that uses a message broker
pub struct BrokerPublisher {
    /// Broker to publish to
    broker: Arc<dyn MessageBroker>,

    /// Default publish options
    default_options: PublishOptions,
}

impl BrokerPublisher {
    /// Create a new broker publisher
    pub fn new(broker: Arc<dyn MessageBroker>) -> Self {
        let default_options = PublishOptions::default();
        Self {
            broker,
            default_options,
        }
    }

    /// Create a new broker publisher with default options
    pub fn with_options(broker: Arc<dyn MessageBroker>, options: PublishOptions) -> Self {
        Self {
            broker,
            default_options: options,
        }
    }

    /// Set default options
    pub fn set_default_options(&mut self, options: PublishOptions) {
        self.default_options = options;
    }

    /// Merge provided options with defaults
    fn merge_options(&self, options: Option<PublishOptions>) -> PublishOptions {
        match options {
            Some(opts) => {
                let mut merged = self.default_options.clone();

                // Override defaults with provided options
                if !opts.exchange.is_empty() {
                    merged.exchange = opts.exchange;
                }

                if opts.routing_key.is_some() {
                    merged.routing_key = opts.routing_key;
                }

                merged.mandatory = opts.mandatory;
                merged.immediate = opts.immediate;
                merged.delivery_mode = opts.delivery_mode;

                if opts.expiration_ms.is_some() {
                    merged.expiration_ms = opts.expiration_ms;
                }

                if opts.priority.is_some() {
                    merged.priority = opts.priority;
                }

                merged.wait_for_confirm = opts.wait_for_confirm;

                if opts.confirm_timeout.is_some() {
                    merged.confirm_timeout = opts.confirm_timeout;
                }

                // Merge headers
                for (k, v) in opts.headers {
                    merged.headers.insert(k, v);
                }

                merged
            }
            None => self.default_options.clone(),
        }
    }
}

#[async_trait]
impl MessagePublisher for BrokerPublisher {
    async fn publish<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult> {
        let opts = self.merge_options(options);
        let start = std::time::Instant::now();

        let result = if opts.wait_for_confirm {
            self.broker
                .publish_with_confirm(message, Some(opts.clone()), opts.confirm_timeout)
                .await
        } else {
            self.broker.publish(message, Some(opts.clone())).await
        };

        let elapsed = start.elapsed();

        match result {
            Ok(()) => Ok(PublishResult {
                success: true,
                message_id: message.id.clone(),
                error: None,
                confirmation_id: None, // Broker doesn't provide this
                publish_time: elapsed,
            }),
            Err(err) => Ok(PublishResult {
                success: false,
                message_id: message.id.clone(),
                error: Some(err.to_string()),
                confirmation_id: None,
                publish_time: elapsed,
            }),
        }
    }

    async fn publish_with_confirm<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult> {
        let mut opts = self.merge_options(options);
        opts.wait_for_confirm = true;

        if let Some(t) = timeout {
            opts.confirm_timeout = Some(t);
        }

        self.publish(message, Some(opts)).await
    }
}

/// A publisher that can batch messages
pub struct BatchPublisher {
    /// Inner publisher
    inner: Arc<dyn MessagePublisher>,

    /// Batch size
    batch_size: usize,

    /// Batch timeout
    batch_timeout: Duration,

    /// Message queue
    queue_tx: mpsc::Sender<BatchMessage>,
}

/// A message in the batch
enum BatchMessage {
    /// Message to publish
    Message(Box<dyn BatchItem>),

    /// Flush the batch
    Flush,
}

/// Trait for items that can be batched
#[async_trait]
trait BatchItem: Send + Sync {
    /// Publish the item
    async fn publish(&self, publisher: &dyn MessagePublisher) -> MessagingResult<PublishResult>;
}

/// A message to be batched
struct BatchableMessage<T: serde::Serialize + Send + Sync + 'static> {
    /// The message
    message: Message<T>,

    /// Publish options
    options: Option<PublishOptions>,

    /// Result sender
    result_tx: Option<mpsc::Sender<MessagingResult<PublishResult>>>,
}

#[async_trait]
impl<T: serde::Serialize + Send + Sync + 'static> BatchItem for BatchableMessage<T> {
    async fn publish(&self, publisher: &dyn MessagePublisher) -> MessagingResult<PublishResult> {
        publisher.publish(&self.message, self.options.clone()).await
    }
}

impl BatchPublisher {
    /// Create a new batch publisher
    pub fn new(
        inner: Arc<dyn MessagePublisher>,
        batch_size: usize,
        batch_timeout: Duration,
    ) -> Self {
        let (queue_tx, mut queue_rx) = mpsc::channel::<BatchMessage>(1000);
        let inner_clone = inner.clone();

        // Spawn background task to process batches
        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut timeout = tokio::time::interval(batch_timeout);

            loop {
                tokio::select! {
                    // Process incoming messages
                    msg = queue_rx.recv() => {
                        match msg {
                            Some(BatchMessage::Message(item)) => {
                                batch.push(item);

                                // If batch is full, publish it
                                if batch.len() >= batch_size {
                                    Self::publish_batch(&inner_clone, &mut batch).await;
                                }
                            }
                            Some(BatchMessage::Flush) => {
                                // Publish any pending messages
                                if !batch.is_empty() {
                                    Self::publish_batch(&inner_clone, &mut batch).await;
                                }
                            }
                            None => {
                                // Channel closed, exit
                                break;
                            }
                        }
                    }

                    // Timeout - publish any pending messages
                    _ = timeout.tick() => {
                        if !batch.is_empty() {
                            Self::publish_batch(&inner_clone, &mut batch).await;
                        }
                    }
                }
            }
        });

        Self {
            inner,
            batch_size,
            batch_timeout,
            queue_tx,
        }
    }

    /// Publish a batch of messages
    async fn publish_batch(
        publisher: &Arc<dyn MessagePublisher>,
        batch: &mut Vec<Box<dyn BatchItem>>,
    ) {
        for item in batch.drain(..) {
            // Ignore errors, they're handled through the result channel
            let _ = item.publish(publisher.as_ref()).await;
        }
    }

    /// Publish a message
    pub async fn publish<T: serde::Serialize + Send + Sync + 'static>(
        &self,
        message: Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult> {
        let (result_tx, mut result_rx) = mpsc::channel(1);

        let batch_item = BatchableMessage {
            message,
            options,
            result_tx: Some(result_tx),
        };

        self.queue_tx
            .send(BatchMessage::Message(Box::new(batch_item)))
            .await
            .map_err(|_| {
                MessagingError::PublishError("Failed to queue message for batch".into())
            })?;

        // Wait for result
        result_rx
            .recv()
            .await
            .ok_or_else(|| MessagingError::PublishError("No result received from batch".into()))?
    }

    /// Flush any pending messages
    pub async fn flush(&self) -> MessagingResult<()> {
        self.queue_tx
            .send(BatchMessage::Flush)
            .await
            .map_err(|_| MessagingError::PublishError("Failed to send flush command".into()))?;
        Ok(())
    }
}

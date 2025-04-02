use std::{collections::HashMap, marker::PhantomData, sync::Arc, time::Duration};

use async_trait::async_trait;
use erased_serde;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::mpsc;

use crate::{
    broker::MessageBroker,
    error::{DeliveryMode, MessagingError, MessagingResult},
    message::Message,
};

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
    async fn publish_any(
        &self,
        message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult>;

    /// Publish a message and wait for confirmation
    async fn publish_any_with_confirm(
        &self,
        message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult>;
}

/// Extension trait for type-safe publishing
#[async_trait]
pub trait TypedMessagePublisher<T: serde::Serialize + Send + Sync + Clone + 'static>:
    MessagePublisher
{
    /// Publish a message with a specific type
    async fn publish(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult>;

    /// Publish a message with a specific type and wait for confirmation
    async fn publish_with_confirm(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult>;
}

/// A publisher that uses a message broker
pub struct BrokerPublisher {
    /// Broker to publish to
    #[allow(dead_code)]
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
    async fn publish_any(
        &self,
        _message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult> {
        let _opts = self.merge_options(options);
        // Implementation would go here
        unimplemented!()
    }

    async fn publish_any_with_confirm(
        &self,
        _message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult> {
        let _opts = self.merge_options(options);
        let _timeout = timeout.unwrap_or_else(|| Duration::from_secs(5));
        // Implementation would go here
        unimplemented!()
    }
}

/// A publisher that can batch messages
pub struct BatchPublisher {
    /// Inner publisher
    #[allow(dead_code)]
    inner: Arc<dyn MessagePublisher>,

    /// Batch size
    #[allow(dead_code)]
    batch_size: usize,

    /// Batch timeout
    #[allow(dead_code)]
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

/// A specialized batch message implementation that works with boxed payloads
struct BoxedBatchMessage {
    /// The message
    message: Message<Box<dyn erased_serde::Serialize + Send + Sync>>,

    /// Publish options
    options: Option<PublishOptions>,

    /// Result sender
    #[allow(dead_code)]
    result_tx: Option<mpsc::Sender<MessagingResult<PublishResult>>>,
}

#[async_trait]
impl BatchItem for BoxedBatchMessage {
    async fn publish(&self, publisher: &dyn MessagePublisher) -> MessagingResult<PublishResult> {
        publisher
            .publish_any(&self.message, self.options.clone())
            .await
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

    /// Publish a message to a topic without waiting for confirmation
    pub async fn publish<T>(&self, message: &Message<T>) -> MessagingResult<PublishResult>
    where
        T: Serialize + Clone + Send + Sync + 'static,
    {
        // Create a boxed payload as an erased type
        let payload: Box<dyn erased_serde::Serialize + Send + Sync> =
            Box::new(message.payload.clone());

        // Create a new message with the boxed payload
        let boxed_message = Message {
            id: message.id.clone(),
            topic: message.topic.clone(),
            payload,
            headers: message.headers.clone(),
            timestamp: message.timestamp,
            expiration: message.expiration,
            priority: message.priority,
            delivery_mode: message.delivery_mode,
            correlation_id: message.correlation_id.clone(),
            reply_to: message.reply_to.clone(),
            reply_to_id: message.reply_to_id.clone(),
        };

        // Create a batch message using our specialized type
        let batch_message = Box::new(BoxedBatchMessage {
            message: boxed_message,
            options: None,
            result_tx: None,
        });

        // Queue the message
        self.queue_tx
            .send(BatchMessage::Message(batch_message))
            .await
            .map_err(|e| {
                MessagingError::ChannelSendError(format!("Failed to queue message: {:?}", e))
            })?;

        // Return a success result
        Ok(PublishResult {
            success: true,
            message_id: message.id.clone(),
            error: None,
            confirmation_id: None,
            publish_time: Duration::from_secs(0),
        })
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

/// A typed publisher implementation
pub struct TypedPublisherImpl<T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone> {
    inner: Arc<dyn MessagePublisher>,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone> TypedPublisherImpl<T> {
    /// Create a new typed publisher
    pub fn new(inner: Arc<dyn MessagePublisher>) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone> MessagePublisher
    for TypedPublisherImpl<T>
{
    async fn publish_any(
        &self,
        message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult> {
        self.inner.publish_any(message, options).await
    }

    async fn publish_any_with_confirm(
        &self,
        message: &Message<Box<dyn erased_serde::Serialize + Send + Sync>>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult> {
        self.inner
            .publish_any_with_confirm(message, options, timeout)
            .await
    }
}

#[async_trait]
impl<T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone> TypedMessagePublisher<T>
    for TypedPublisherImpl<T>
{
    async fn publish(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<PublishResult> {
        // Create a new message with the same properties but boxed payload
        let payload: Box<dyn erased_serde::Serialize + Send + Sync> =
            Box::new(message.payload.clone());
        let boxed_message = Message {
            id: message.id.clone(),
            topic: message.topic.clone(),
            payload, // Properly boxed payload
            headers: message.headers.clone(),
            timestamp: message.timestamp,
            expiration: message.expiration,
            priority: message.priority,
            delivery_mode: message.delivery_mode,
            correlation_id: message.correlation_id.clone(),
            reply_to: message.reply_to.clone(),
            reply_to_id: message.reply_to_id.clone(),
        };

        // Publish using any publishing
        self.inner.publish_any(&boxed_message, options).await
    }

    async fn publish_with_confirm(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<PublishResult> {
        // Create a new message with the same properties but boxed payload
        let payload: Box<dyn erased_serde::Serialize + Send + Sync> =
            Box::new(message.payload.clone());
        let boxed_message = Message {
            id: message.id.clone(),
            topic: message.topic.clone(),
            payload, // Properly boxed payload
            headers: message.headers.clone(),
            timestamp: message.timestamp,
            expiration: message.expiration,
            priority: message.priority,
            delivery_mode: message.delivery_mode,
            correlation_id: message.correlation_id.clone(),
            reply_to: message.reply_to.clone(),
            reply_to_id: message.reply_to_id.clone(),
        };

        // Publish using any publishing with confirm
        self.inner
            .publish_any_with_confirm(&boxed_message, options, timeout)
            .await
    }
}

/// Create a typed publisher from a message broker
pub fn create_typed_publisher<T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone>(
    broker: &Arc<dyn MessageBroker>,
) -> Arc<dyn TypedMessagePublisher<T>> {
    let publisher = BrokerPublisher::new(broker.clone());
    Arc::new(TypedPublisherImpl::new(Arc::new(publisher)))
}

#[async_trait]
impl<T: serde::Serialize + Send + Sync + Clone + 'static> BatchItem for BatchableMessage<T> {
    async fn publish(&self, publisher: &dyn MessagePublisher) -> MessagingResult<PublishResult> {
        // Clone the payload to create a boxed version
        let payload_clone = self.message.payload.clone();
        let boxed_payload: Box<dyn erased_serde::Serialize + Send + Sync> = Box::new(payload_clone);

        // Create a new message with the boxed payload
        let boxed_message = Message {
            id: self.message.id.clone(),
            topic: self.message.topic.clone(),
            payload: boxed_payload,
            headers: self.message.headers.clone(),
            timestamp: self.message.timestamp,
            expiration: self.message.expiration,
            priority: self.message.priority,
            delivery_mode: self.message.delivery_mode,
            correlation_id: self.message.correlation_id.clone(),
            reply_to: self.message.reply_to.clone(),
            reply_to_id: self.message.reply_to_id.clone(),
        };

        // Use the publisher with the boxed message
        publisher
            .publish_any(&boxed_message, self.options.clone())
            .await
    }
}

/// A message to be batched
#[allow(dead_code)]
struct BatchableMessage<T: serde::Serialize + Send + Sync + Clone + 'static> {
    /// The message
    message: Message<T>,

    /// Publish options
    options: Option<PublishOptions>,

    /// Result sender
    result_tx: Option<mpsc::Sender<MessagingResult<PublishResult>>>,
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::AcknowledgmentMode;
use crate::message::MessageProcessingResult;

/// Options for message consumers
#[derive(Debug, Clone)]
pub struct ConsumerOptions {
    /// Consumer tag
    pub consumer_tag: Option<String>,

    /// Whether this is an exclusive consumer
    pub exclusive: bool,

    /// Whether the consumer should be auto-deleted when the channel is closed
    pub auto_delete: bool,

    /// Prefetch count (QoS)
    pub prefetch_count: u16,

    /// Acknowledgment mode
    pub ack_mode: AcknowledgmentMode,

    /// Additional arguments to pass to the broker
    pub arguments: HashMap<String, String>,

    /// Offset to start consuming from (for brokers that support it)
    pub offset: Option<MessageOffset>,

    /// Maximum number of times to retry message processing
    pub max_retries: u32,

    /// Backoff strategy for retries
    pub retry_strategy: RetryStrategy,

    /// Whether to forward errors to a dead letter queue
    pub forward_errors: bool,

    /// Whether to use nolocal option (don't receive messages published by this connection)
    pub no_local: bool,

    /// Consumer timeout
    pub timeout: Option<Duration>,

    /// Whether to auto-start consuming
    pub auto_start: bool,

    /// Consumer priority (higher values have higher priority, if the broker supports it)
    pub priority: Option<u8>,
}

impl Default for ConsumerOptions {
    fn default() -> Self {
        Self {
            consumer_tag: None,
            exclusive: false,
            auto_delete: false,
            prefetch_count: 10,
            ack_mode: AcknowledgmentMode::Manual,
            arguments: HashMap::new(),
            offset: None,
            max_retries: 3,
            retry_strategy: RetryStrategy::Exponential {
                initial_delay: Duration::from_millis(100),
                multiplier: 2.0,
                max_delay: Duration::from_secs(30),
                jitter: true,
            },
            forward_errors: true,
            no_local: false,
            timeout: Some(Duration::from_secs(30)),
            auto_start: true,
            priority: None,
        }
    }
}

impl ConsumerOptions {
    /// Create a new set of consumer options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the consumer tag
    pub fn with_consumer_tag(mut self, tag: impl Into<String>) -> Self {
        self.consumer_tag = Some(tag.into());
        self
    }

    /// Set whether this is an exclusive consumer
    pub fn with_exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    /// Set whether the consumer should auto-delete
    pub fn with_auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Set the prefetch count
    pub fn with_prefetch_count(mut self, prefetch: u16) -> Self {
        self.prefetch_count = prefetch;
        self
    }

    /// Set the acknowledgment mode
    pub fn with_ack_mode(mut self, mode: AcknowledgmentMode) -> Self {
        self.ack_mode = mode;
        self
    }

    /// Add an argument
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set the offset to start consuming from
    pub fn with_offset(mut self, offset: MessageOffset) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set retry options
    pub fn with_retry(mut self, max_retries: u32, strategy: RetryStrategy) -> Self {
        self.max_retries = max_retries;
        self.retry_strategy = strategy;
        self
    }

    /// Set whether to forward errors to a dead letter queue
    pub fn with_forward_errors(mut self, forward: bool) -> Self {
        self.forward_errors = forward;
        self
    }

    /// Set whether to use the no_local option
    pub fn with_no_local(mut self, no_local: bool) -> Self {
        self.no_local = no_local;
        self
    }

    /// Set the consumer timeout
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set whether to auto-start consuming
    pub fn with_auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = auto_start;
        self
    }

    /// Set the consumer priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// Starting offset for message consumption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageOffset {
    /// Start from the beginning of the queue/topic
    Beginning,

    /// Start from the end of the queue/topic (new messages only)
    End,

    /// Start from a specific offset
    Offset(u64),

    /// Start from a timestamp (milliseconds since epoch)
    Timestamp(i64),
}

/// Retry strategy for failed message processing
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// Retry immediately without backoff
    Immediate,

    /// Fixed delay between retries
    Fixed {
        /// Delay between retries
        delay: Duration,
    },

    /// Exponential backoff
    Exponential {
        /// Initial delay
        initial_delay: Duration,

        /// Multiplier for each retry
        multiplier: f64,

        /// Maximum delay
        max_delay: Duration,

        /// Whether to add jitter
        jitter: bool,
    },

    /// Custom retry strategy
    Custom {
        /// Function to calculate delay for a given retry attempt
        calculator: Arc<dyn Fn(u32) -> Duration + Send + Sync>,
    },
}

impl RetryStrategy {
    /// Calculate the delay for a specific retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            RetryStrategy::Immediate => Duration::from_millis(0),

            RetryStrategy::Fixed { delay } => *delay,

            RetryStrategy::Exponential {
                initial_delay,
                multiplier,
                max_delay,
                jitter,
            } => {
                let base_delay_ms =
                    initial_delay.as_millis() as f64 * multiplier.powf(attempt as f64);
                let mut delay_ms = base_delay_ms.min(max_delay.as_millis() as f64);

                if *jitter {
                    // Add up to 10% jitter
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let random = (now % 1000) as f64 / 1000.0;
                    let jitter_amount = delay_ms * 0.1 * random;
                    delay_ms += jitter_amount;
                }

                Duration::from_millis(delay_ms as u64)
            }

            RetryStrategy::Custom { calculator } => calculator(attempt),
        }
    }
}

/// Information about a consumer
#[derive(Debug, Clone)]
pub struct ConsumerInfo {
    /// Consumer tag
    pub tag: String,

    /// Queue being consumed
    pub queue: String,

    /// Consumer options
    pub options: ConsumerOptions,

    /// Time the consumer was created
    pub created_at: std::time::SystemTime,

    /// Number of messages delivered
    pub messages_delivered: u64,

    /// Number of messages acknowledged
    pub messages_acknowledged: u64,

    /// Number of messages rejected
    pub messages_rejected: u64,

    /// Consumer status
    pub status: ConsumerStatus,
}

/// Status of a consumer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerStatus {
    /// Consumer is active and processing messages
    Active,

    /// Consumer is paused
    Paused,

    /// Consumer is cancelled
    Cancelled,

    /// Consumer encountered an error
    Error,
}

/// A handler that retries message processing up to a maximum number of times
pub struct RetryingHandler<T, H> {
    /// Inner handler
    inner: H,

    /// Maximum retry attempts
    max_retries: u32,

    /// Retry strategy
    retry_strategy: RetryStrategy,

    /// Phantom data
    _marker: std::marker::PhantomData<T>,
}

impl<T, H> RetryingHandler<T, H> {
    /// Create a new retrying handler
    pub fn new(inner: H, max_retries: u32, retry_strategy: RetryStrategy) -> Self {
        Self {
            inner,
            max_retries,
            retry_strategy,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, H> RetryingHandler<T, H>
where
    H: crate::message::MessageHandler<T>,
    T: Clone,
{
    /// Process a message with retries
    pub async fn process(
        &self,
        message: &crate::message::ReceivedMessage<T>,
    ) -> MessageProcessingResult {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= self.max_retries {
            match self.inner.handle(message) {
                Ok(ack) => return Ok(ack),
                Err(err) => {
                    attempt += 1;
                    last_error = Some(err);

                    if attempt <= self.max_retries {
                        let delay = self.retry_strategy.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // If we exhausted all retries, return the last error
        Err(last_error.unwrap_or_else(|| {
            crate::error::MessagingError::ConsumerError("Maximum retries exceeded".into())
        }))
    }
}

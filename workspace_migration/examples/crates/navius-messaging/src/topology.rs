use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Exchange types supported by message brokers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeType {
    /// Direct exchange (exact match on routing key)
    Direct,

    /// Fanout exchange (broadcasts to all bound queues)
    Fanout,

    /// Topic exchange (pattern matching on routing key)
    Topic,

    /// Headers exchange (matches based on message headers)
    Headers,

    /// Consistent hash exchange (distributes based on routing keys using consistent hashing)
    ConsistentHash,
}

impl fmt::Display for ExchangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeType::Direct => write!(f, "direct"),
            ExchangeType::Fanout => write!(f, "fanout"),
            ExchangeType::Topic => write!(f, "topic"),
            ExchangeType::Headers => write!(f, "headers"),
            ExchangeType::ConsistentHash => write!(f, "x-consistent-hash"),
        }
    }
}

impl Default for ExchangeType {
    fn default() -> Self {
        ExchangeType::Direct
    }
}

/// Definition of a message exchange
#[derive(Debug, Clone)]
pub struct Exchange {
    /// Exchange name
    pub name: String,

    /// Exchange type
    pub kind: ExchangeType,

    /// Whether the exchange is durable (survives broker restart)
    pub durable: bool,

    /// Whether the exchange is auto-deleted when no longer in use
    pub auto_delete: bool,

    /// Whether the exchange is internal (can't be published to directly)
    pub internal: bool,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

impl Exchange {
    /// Create a new exchange with the given name and type
    pub fn new(name: impl Into<String>, kind: ExchangeType) -> Self {
        Self {
            name: name.into(),
            kind,
            durable: true,
            auto_delete: false,
            internal: false,
            arguments: HashMap::new(),
        }
    }

    /// Create a direct exchange
    pub fn direct(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Direct)
    }

    /// Create a fanout exchange
    pub fn fanout(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Fanout)
    }

    /// Create a topic exchange
    pub fn topic(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Topic)
    }

    /// Create a headers exchange
    pub fn headers(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Headers)
    }

    /// Set durability
    pub fn durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set auto-delete
    pub fn auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Set internal flag
    pub fn internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }

    /// Add an argument
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Add alternate exchange
    pub fn with_alternate_exchange(mut self, alt_exchange: impl Into<String>) -> Self {
        self.arguments
            .insert("alternate-exchange".into(), alt_exchange.into());
        self
    }
}

/// Definition of a message queue
#[derive(Debug, Clone)]
pub struct Queue {
    /// Queue name
    pub name: String,

    /// Whether the queue is durable (survives broker restart)
    pub durable: bool,

    /// Whether the queue is exclusive to one connection
    pub exclusive: bool,

    /// Whether the queue is auto-deleted when no longer in use
    pub auto_delete: bool,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

impl Queue {
    /// Create a new queue with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            exclusive: false,
            auto_delete: false,
            arguments: HashMap::new(),
        }
    }

    /// Set durability
    pub fn durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set exclusive flag
    pub fn exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    /// Set auto-delete flag
    pub fn auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Add an argument
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set the message TTL (time-to-live)
    pub fn with_message_ttl(mut self, ttl: Duration) -> Self {
        self.arguments
            .insert("x-message-ttl".into(), ttl.as_millis().to_string());
        self
    }

    /// Set the queue TTL (time-to-live)
    pub fn with_queue_ttl(mut self, ttl: Duration) -> Self {
        self.arguments
            .insert("x-expires".into(), ttl.as_millis().to_string());
        self
    }

    /// Set the dead letter exchange and routing key
    pub fn with_dead_letter(
        mut self,
        exchange: impl Into<String>,
        routing_key: Option<impl Into<String>>,
    ) -> Self {
        self.arguments
            .insert("x-dead-letter-exchange".into(), exchange.into());

        if let Some(key) = routing_key {
            self.arguments
                .insert("x-dead-letter-routing-key".into(), key.into());
        }

        self
    }

    /// Set the maximum length of the queue
    pub fn with_max_length(mut self, max_length: u32) -> Self {
        self.arguments
            .insert("x-max-length".into(), max_length.to_string());
        self
    }

    /// Set the maximum size of the queue in bytes
    pub fn with_max_length_bytes(mut self, max_bytes: u32) -> Self {
        self.arguments
            .insert("x-max-length-bytes".into(), max_bytes.to_string());
        self
    }

    /// Set queue overflow behavior
    pub fn with_overflow(mut self, behavior: QueueOverflowBehavior) -> Self {
        self.arguments
            .insert("x-overflow".into(), behavior.to_string());
        self
    }

    /// Set the maximum priority for the queue
    pub fn with_max_priority(mut self, max_priority: u8) -> Self {
        self.arguments
            .insert("x-max-priority".into(), max_priority.to_string());
        self
    }

    /// Is this queue a temporary/server-generated queue
    pub fn is_temporary(&self) -> bool {
        self.exclusive && self.auto_delete && !self.durable
    }
}

/// Behavior when a queue overflows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueOverflowBehavior {
    /// Drop messages at the head of the queue (oldest)
    DropHead,

    /// Reject new messages
    RejectPublish,

    /// Reject new messages and return with a publisher confirm
    RejectPublishDlx,
}

impl fmt::Display for QueueOverflowBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueOverflowBehavior::DropHead => write!(f, "drop-head"),
            QueueOverflowBehavior::RejectPublish => write!(f, "reject-publish"),
            QueueOverflowBehavior::RejectPublishDlx => write!(f, "reject-publish-dlx"),
        }
    }
}

/// A binding between an exchange and a queue or another exchange
#[derive(Debug, Clone)]
pub struct Binding {
    /// Source exchange
    pub source: String,

    /// Destination (queue or exchange)
    pub destination: BindingDestination,

    /// Routing key
    pub routing_key: String,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

/// Binding destination type
#[derive(Debug, Clone)]
pub enum BindingDestination {
    /// Binding to a queue
    Queue(String),

    /// Binding to an exchange
    Exchange(String),
}

impl Binding {
    /// Create a new binding to a queue
    pub fn queue_binding(
        source: impl Into<String>,
        queue: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            destination: BindingDestination::Queue(queue.into()),
            routing_key: routing_key.into(),
            arguments: HashMap::new(),
        }
    }

    /// Create a new binding to an exchange
    pub fn exchange_binding(
        source: impl Into<String>,
        destination: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            destination: BindingDestination::Exchange(destination.into()),
            routing_key: routing_key.into(),
            arguments: HashMap::new(),
        }
    }

    /// Add an argument
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Add a header match for headers exchange
    pub fn with_header_match(
        mut self,
        header: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.arguments.insert(header.into(), value.into());
        self
    }

    /// Set match type for headers exchange (any or all)
    pub fn with_match_type(mut self, match_all: bool) -> Self {
        self.arguments.insert(
            "x-match".into(),
            if match_all { "all" } else { "any" }.into(),
        );
        self
    }
}

/// Information about a topology element
#[derive(Debug, Clone)]
pub struct TopologyInfo {
    /// Exchange information
    pub exchanges: Vec<ExchangeInfo>,

    /// Queue information
    pub queues: Vec<QueueInfo>,

    /// Binding information
    pub bindings: Vec<BindingInfo>,
}

/// Information about an exchange
#[derive(Debug, Clone)]
pub struct ExchangeInfo {
    /// Exchange name
    pub name: String,

    /// Exchange type
    pub kind: ExchangeType,

    /// Whether the exchange is durable
    pub durable: bool,

    /// Whether the exchange is auto-deleted
    pub auto_delete: bool,

    /// Whether the exchange is internal
    pub internal: bool,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

/// Information about a queue
#[derive(Debug, Clone)]
pub struct QueueInfo {
    /// Queue name
    pub name: String,

    /// Whether the queue is durable
    pub durable: bool,

    /// Whether the queue is exclusive
    pub exclusive: bool,

    /// Whether the queue is auto-deleted
    pub auto_delete: bool,

    /// Additional arguments
    pub arguments: HashMap<String, String>,

    /// Number of messages in the queue
    pub messages: u32,

    /// Number of consumers
    pub consumers: u32,
}

/// Information about a binding
#[derive(Debug, Clone)]
pub struct BindingInfo {
    /// Source exchange
    pub source: String,

    /// Destination (queue or exchange)
    pub destination: String,

    /// Whether the destination is a queue or exchange
    pub destination_type: BindingDestinationType,

    /// Routing key
    pub routing_key: String,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

/// Type of binding destination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingDestinationType {
    /// Queue destination
    Queue,

    /// Exchange destination
    Exchange,
}

impl fmt::Display for BindingDestinationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingDestinationType::Queue => write!(f, "queue"),
            BindingDestinationType::Exchange => write!(f, "exchange"),
        }
    }
}

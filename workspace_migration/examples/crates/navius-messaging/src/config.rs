use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use crate::error::AcknowledgmentMode;

/// Configuration for message brokers
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    /// Unique identifier for this broker
    pub id: String,

    /// Broker name for identification
    pub name: String,

    /// Broker type (e.g., "rabbitmq", "kafka", "redis")
    pub broker_type: String,

    /// Connection configuration
    pub connection: ConnectionConfig,

    /// Default channel/connection pool configuration
    pub pool: PoolConfig,

    /// Topology recovery configuration
    pub recovery: RecoveryConfig,

    /// Default consumer configuration
    pub consumer: ConsumerDefaultConfig,

    /// Default publisher configuration
    pub publisher: PublisherDefaultConfig,

    /// Default exchange name
    pub default_exchange: Option<String>,

    /// Event publishing configuration
    pub events: EventConfig,

    /// Additional broker-specific options
    pub options: HashMap<String, String>,
}

impl BrokerConfig {
    /// Create a new broker configuration with default values
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        broker_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            broker_type: broker_type.into(),
            connection: ConnectionConfig::default(),
            pool: PoolConfig::default(),
            recovery: RecoveryConfig::default(),
            consumer: ConsumerDefaultConfig::default(),
            publisher: PublisherDefaultConfig::default(),
            default_exchange: None,
            events: EventConfig::default(),
            options: HashMap::new(),
        }
    }

    /// Set the connection configuration
    pub fn with_connection(mut self, connection: ConnectionConfig) -> Self {
        self.connection = connection;
        self
    }

    /// Set the pool configuration
    pub fn with_pool(mut self, pool: PoolConfig) -> Self {
        self.pool = pool;
        self
    }

    /// Set the recovery configuration
    pub fn with_recovery(mut self, recovery: RecoveryConfig) -> Self {
        self.recovery = recovery;
        self
    }

    /// Set the consumer configuration
    pub fn with_consumer(mut self, consumer: ConsumerDefaultConfig) -> Self {
        self.consumer = consumer;
        self
    }

    /// Set the publisher configuration
    pub fn with_publisher(mut self, publisher: PublisherDefaultConfig) -> Self {
        self.publisher = publisher;
        self
    }

    /// Set the default exchange
    pub fn with_default_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.default_exchange = Some(exchange.into());
        self
    }

    /// Set the event configuration
    pub fn with_events(mut self, events: EventConfig) -> Self {
        self.events = events;
        self
    }

    /// Add a broker-specific option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Add multiple broker-specific options
    pub fn with_options(mut self, options: HashMap<String, String>) -> Self {
        self.options.extend(options);
        self
    }
}

/// Configuration for broker connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection URI/URL
    pub uri: String,

    /// Authentication username
    pub username: Option<String>,

    /// Authentication password
    pub password: Option<String>,

    /// Virtual host or namespace
    pub vhost: Option<String>,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,

    /// Heartbeat interval in seconds (0 to disable)
    pub heartbeat_secs: u16,

    /// SSL/TLS configuration
    pub tls: Option<TlsConfig>,

    /// Connection retry configuration
    pub retry: RetryConfig,

    /// Additional connection properties
    pub properties: HashMap<String, String>,
}

impl ConnectionConfig {
    /// Create a new connection configuration with the specified URI
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            username: None,
            password: None,
            vhost: None,
            timeout_ms: 5000,
            heartbeat_secs: 30,
            tls: None,
            retry: RetryConfig::default(),
            properties: HashMap::new(),
        }
    }

    /// Set authentication credentials
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Set virtual host
    pub fn with_vhost(mut self, vhost: impl Into<String>) -> Self {
        self.vhost = Some(vhost.into());
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set heartbeat interval
    pub fn with_heartbeat(mut self, heartbeat_secs: u16) -> Self {
        self.heartbeat_secs = heartbeat_secs;
        self
    }

    /// Set TLS configuration
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    /// Add a connection property
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            uri: "amqp://localhost:5672".to_string(),
            username: None,
            password: None,
            vhost: None,
            timeout_ms: 5000,
            heartbeat_secs: 30,
            tls: None,
            retry: RetryConfig::default(),
            properties: HashMap::new(),
        }
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to CA certificate file
    pub ca_cert_path: Option<String>,

    /// Path to client certificate file
    pub client_cert_path: Option<String>,

    /// Path to client key file
    pub client_key_path: Option<String>,

    /// Verify peer certificate
    pub verify_peer: bool,

    /// Enable revocation checking
    pub verify_hostname: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            verify_peer: true,
            verify_hostname: true,
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration with the given CA certificate path
    pub fn new(ca_cert_path: impl Into<String>) -> Self {
        Self {
            ca_cert_path: Some(ca_cert_path.into()),
            client_cert_path: None,
            client_key_path: None,
            verify_peer: true,
            verify_hostname: true,
        }
    }

    /// Set client certificate and key
    pub fn with_client_cert(
        mut self,
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.client_cert_path = Some(cert_path.into());
        self.client_key_path = Some(key_path.into());
        self
    }

    /// Set verify peer option
    pub fn with_verify_peer(mut self, verify: bool) -> Self {
        self.verify_peer = verify;
        self
    }

    /// Set verify hostname option
    pub fn with_verify_hostname(mut self, verify: bool) -> Self {
        self.verify_hostname = verify;
        self
    }
}

/// Connection retry configuration
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Maximum number of connection attempts (0 means unlimited)
    pub max_attempts: u32,

    /// Initial backoff delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum backoff delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier for each retry
    pub multiplier: f64,

    /// Maximum jitter percentage (0.0 to 1.0)
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 0, // Unlimited
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with the given parameters
    pub fn new(
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            multiplier,
            jitter: 0.1,
        }
    }

    /// Set the jitter factor
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.max(0.0).min(1.0);
        self
    }

    /// Calculate the delay for a specific retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = self.initial_delay_ms as f64 * self.multiplier.powf(attempt as f64);

        // Apply jitter
        if self.jitter > 0.0 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let random = (now % 1000) as f64 / 1000.0;
            let jitter_amount = delay * self.jitter * random;
            delay += jitter_amount;
        }

        // Cap at max delay
        let delay_ms = delay.min(self.max_delay_ms as f64) as u64;

        Duration::from_millis(delay_ms)
    }
}

/// Channel/connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_size: usize,

    /// Maximum number of connections to allow
    pub max_size: usize,

    /// Maximum time to wait for a connection in milliseconds
    pub acquire_timeout_ms: u64,

    /// Connection idle timeout in milliseconds (0 means no timeout)
    pub idle_timeout_ms: u64,

    /// Maximum lifetime of a connection in milliseconds (0 means no maximum)
    pub max_lifetime_ms: u64,

    /// Test connections before returning them to client
    pub test_on_acquire: bool,

    /// Test connections before they're returned to the pool
    pub test_on_release: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 20,
            acquire_timeout_ms: 30000,
            idle_timeout_ms: 300000,  // 5 minutes
            max_lifetime_ms: 3600000, // 1 hour
            test_on_acquire: true,
            test_on_release: false,
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration with the specified min and max sizes
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            max_size,
            acquire_timeout_ms: 30000,
            idle_timeout_ms: 300000,
            max_lifetime_ms: 3600000,
            test_on_acquire: true,
            test_on_release: false,
        }
    }

    /// Set the acquire timeout
    pub fn with_acquire_timeout(mut self, timeout_ms: u64) -> Self {
        self.acquire_timeout_ms = timeout_ms;
        self
    }

    /// Set the idle timeout
    pub fn with_idle_timeout(mut self, timeout_ms: u64) -> Self {
        self.idle_timeout_ms = timeout_ms;
        self
    }

    /// Set the maximum lifetime
    pub fn with_max_lifetime(mut self, timeout_ms: u64) -> Self {
        self.max_lifetime_ms = timeout_ms;
        self
    }

    /// Set testing policies
    pub fn with_testing(mut self, test_on_acquire: bool, test_on_release: bool) -> Self {
        self.test_on_acquire = test_on_acquire;
        self.test_on_release = test_on_release;
        self
    }
}

/// Topology recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Enable topology recovery
    pub enabled: bool,

    /// Recover exchanges
    pub recover_exchanges: bool,

    /// Recover queues
    pub recover_queues: bool,

    /// Recover bindings
    pub recover_bindings: bool,

    /// Recover consumers
    pub recover_consumers: bool,

    /// Maximum recovery attempts
    pub max_attempts: u32,

    /// Backoff strategy for recovery attempts
    pub backoff: RetryConfig,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            recover_exchanges: true,
            recover_queues: true,
            recover_bindings: true,
            recover_consumers: true,
            max_attempts: 10,
            backoff: RetryConfig::default(),
        }
    }
}

impl RecoveryConfig {
    /// Create a new recovery configuration
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            recover_exchanges: true,
            recover_queues: true,
            recover_bindings: true,
            recover_consumers: true,
            max_attempts: 10,
            backoff: RetryConfig::default(),
        }
    }

    /// Set what to recover
    pub fn with_recovery_types(
        mut self,
        exchanges: bool,
        queues: bool,
        bindings: bool,
        consumers: bool,
    ) -> Self {
        self.recover_exchanges = exchanges;
        self.recover_queues = queues;
        self.recover_bindings = bindings;
        self.recover_consumers = consumers;
        self
    }

    /// Set max attempts
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set backoff strategy
    pub fn with_backoff(mut self, backoff: RetryConfig) -> Self {
        self.backoff = backoff;
        self
    }
}

/// Default consumer configuration
#[derive(Debug, Clone)]
pub struct ConsumerDefaultConfig {
    /// Default prefetch count
    pub prefetch_count: u16,

    /// Default acknowledgment mode
    pub ack_mode: AcknowledgmentMode,

    /// Default consumer tag prefix
    pub tag_prefix: String,

    /// Default consumer timeout in milliseconds
    pub timeout_ms: u64,

    /// Whether to auto-recover consumers on reconnection
    pub auto_recover: bool,

    /// Maximum retry attempts after receiving an error
    pub max_retries: u32,

    /// Retry backoff in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ConsumerDefaultConfig {
    fn default() -> Self {
        Self {
            prefetch_count: 10,
            ack_mode: AcknowledgmentMode::Manual,
            tag_prefix: "navius-consumer".to_string(),
            timeout_ms: 30000,
            auto_recover: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl ConsumerDefaultConfig {
    /// Create a new consumer configuration with the given prefetch count
    pub fn new(prefetch_count: u16) -> Self {
        Self {
            prefetch_count,
            ack_mode: AcknowledgmentMode::Manual,
            tag_prefix: "navius-consumer".to_string(),
            timeout_ms: 30000,
            auto_recover: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }

    /// Set acknowledgment mode
    pub fn with_ack_mode(mut self, ack_mode: AcknowledgmentMode) -> Self {
        self.ack_mode = ack_mode;
        self
    }

    /// Set consumer tag prefix
    pub fn with_tag_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tag_prefix = prefix.into();
        self
    }

    /// Set consumer timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set auto-recovery
    pub fn with_auto_recover(mut self, auto_recover: bool) -> Self {
        self.auto_recover = auto_recover;
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, max_retries: u32, retry_delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = retry_delay_ms;
        self
    }
}

/// Default publisher configuration
#[derive(Debug, Clone)]
pub struct PublisherDefaultConfig {
    /// Default exchange name to use
    pub default_exchange: String,

    /// Whether to use publisher confirms
    pub confirms: bool,

    /// Confirm timeout in milliseconds
    pub confirm_timeout_ms: u64,

    /// Whether to use mandatory flag by default
    pub mandatory: bool,

    /// Whether to use immediate flag by default
    pub immediate: bool,

    /// Default expiration for messages in milliseconds (0 = no expiration)
    pub expiration_ms: u64,

    /// Default message priority (0-255, higher is more priority)
    pub priority: Option<u8>,

    /// Maximum retry attempts after a publish failure
    pub max_retries: u32,

    /// Retry backoff in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for PublisherDefaultConfig {
    fn default() -> Self {
        Self {
            default_exchange: "".to_string(), // Default exchange
            confirms: true,
            confirm_timeout_ms: 5000,
            mandatory: false,
            immediate: false,
            expiration_ms: 0,
            priority: None,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

impl PublisherDefaultConfig {
    /// Create a new publisher configuration with the given exchange
    pub fn new(default_exchange: impl Into<String>) -> Self {
        Self {
            default_exchange: default_exchange.into(),
            confirms: true,
            confirm_timeout_ms: 5000,
            mandatory: false,
            immediate: false,
            expiration_ms: 0,
            priority: None,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }

    /// Set publisher confirms
    pub fn with_confirms(mut self, confirms: bool, timeout_ms: u64) -> Self {
        self.confirms = confirms;
        self.confirm_timeout_ms = timeout_ms;
        self
    }

    /// Set mandatory flag
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Set immediate flag
    pub fn with_immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }

    /// Set message expiration
    pub fn with_expiration(mut self, expiration_ms: u64) -> Self {
        self.expiration_ms = expiration_ms;
        self
    }

    /// Set default priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, max_retries: u32, retry_delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = retry_delay_ms;
        self
    }
}

/// Event publishing configuration
#[derive(Debug, Clone)]
pub struct EventConfig {
    /// Enable publishing events
    pub enabled: bool,

    /// Topic for publishing events
    pub topic: String,

    /// Event types to publish
    pub event_types: Vec<MessagingEventType>,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            topic: "messaging".to_string(),
            event_types: vec![
                MessagingEventType::ConnectionStatus,
                MessagingEventType::ConsumerStatus,
                MessagingEventType::PublisherConfirms,
            ],
        }
    }
}

impl EventConfig {
    /// Create a new event configuration
    pub fn new(enabled: bool, topic: impl Into<String>) -> Self {
        Self {
            enabled,
            topic: topic.into(),
            event_types: vec![
                MessagingEventType::ConnectionStatus,
                MessagingEventType::ConsumerStatus,
                MessagingEventType::PublisherConfirms,
            ],
        }
    }

    /// Set event types to publish
    pub fn with_event_types(mut self, event_types: Vec<MessagingEventType>) -> Self {
        self.event_types = event_types;
        self
    }
}

/// Types of messaging events that can be published
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessagingEventType {
    /// Connection status events
    ConnectionStatus,

    /// Consumer status events
    ConsumerStatus,

    /// Publisher confirm events
    PublisherConfirms,

    /// Message received events
    MessageReceived,

    /// Message published events
    MessagePublished,

    /// Message rejected events
    MessageRejected,

    /// Error events
    Error,
}

impl fmt::Display for MessagingEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessagingEventType::ConnectionStatus => write!(f, "connection.status"),
            MessagingEventType::ConsumerStatus => write!(f, "consumer.status"),
            MessagingEventType::PublisherConfirms => write!(f, "publisher.confirms"),
            MessagingEventType::MessageReceived => write!(f, "message.received"),
            MessagingEventType::MessagePublished => write!(f, "message.published"),
            MessagingEventType::MessageRejected => write!(f, "message.rejected"),
            MessagingEventType::Error => write!(f, "error"),
        }
    }
}

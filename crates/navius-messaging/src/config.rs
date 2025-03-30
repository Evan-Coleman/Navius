use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the messaging system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Default broker to use
    #[serde(default = "default_broker")]
    pub default_broker: String,

    /// Broker-specific configurations
    pub brokers: HashMap<String, BrokerConfig>,

    /// Client configuration
    #[serde(default)]
    pub client: ClientConfig,

    /// Connection retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
}

fn default_broker() -> String {
    "default".to_string()
}

impl Default for MessagingConfig {
    fn default() -> Self {
        let mut brokers = HashMap::new();
        brokers.insert("default".to_string(), BrokerConfig::default());

        Self {
            default_broker: "default".to_string(),
            brokers,
            client: ClientConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl MessagingConfig {
    /// Create a new messaging configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the default broker configuration
    pub fn default_broker_config(&self) -> &BrokerConfig {
        self.brokers.get(&self.default_broker).unwrap_or_else(|| {
            // Return the first broker if the default doesn't exist
            self.brokers
                .values()
                .next()
                .expect("No broker configurations available")
        })
    }

    /// Get a broker configuration by name
    pub fn get_broker(&self, name: &str) -> Option<&BrokerConfig> {
        self.brokers.get(name)
    }

    /// Add a broker configuration
    pub fn add_broker(&mut self, name: String, config: BrokerConfig) -> &mut Self {
        self.brokers.insert(name, config);
        self
    }

    /// Set the default broker
    pub fn set_default_broker(&mut self, name: String) -> &mut Self {
        self.default_broker = name;
        self
    }
}

/// Configuration for a specific broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    /// Provider type (e.g., "rabbitmq", "kafka", "in-memory")
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Connection URI
    pub uri: Option<String>,

    /// Hostname
    pub host: Option<String>,

    /// Port
    pub port: Option<u16>,

    /// Virtual host (for AMQP brokers)
    pub virtual_host: Option<String>,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    /// Whether to use TLS
    #[serde(default)]
    pub use_tls: bool,

    /// Path to client certificate
    pub client_cert: Option<String>,

    /// Path to client key
    pub client_key: Option<String>,

    /// Path to CA certificate
    pub ca_cert: Option<String>,

    /// Whether to verify server certificate
    #[serde(default = "default_true")]
    pub verify_server: bool,

    /// Additional provider-specific options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

fn default_provider() -> String {
    "rabbitmq".to_string()
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            uri: None,
            host: Some("localhost".to_string()),
            port: Some(5672),
            virtual_host: Some("/".to_string()),
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            connection_timeout_secs: default_connection_timeout(),
            use_tls: false,
            client_cert: None,
            client_key: None,
            ca_cert: None,
            verify_server: true,
            options: HashMap::new(),
        }
    }
}

impl BrokerConfig {
    /// Create a new broker configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the provider type
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set the connection URI
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the connection host and port
    pub fn with_host_port(mut self, host: impl Into<String>, port: u16) -> Self {
        self.host = Some(host.into());
        self.port = Some(port);
        self
    }

    /// Set the virtual host
    pub fn with_virtual_host(mut self, virtual_host: impl Into<String>) -> Self {
        self.virtual_host = Some(virtual_host.into());
        self
    }

    /// Set the authentication credentials
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, timeout_secs: u64) -> Self {
        self.connection_timeout_secs = timeout_secs;
        self
    }

    /// Enable TLS with certificates
    pub fn with_tls(
        mut self,
        client_cert: Option<String>,
        client_key: Option<String>,
        ca_cert: Option<String>,
        verify_server: bool,
    ) -> Self {
        self.use_tls = true;
        self.client_cert = client_cert;
        self.client_key = client_key;
        self.ca_cert = ca_cert;
        self.verify_server = verify_server;
        self
    }

    /// Add a provider-specific option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Get the connection timeout as a Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }

    /// Build a connection string based on the configuration
    pub fn connection_string(&self) -> String {
        if let Some(uri) = &self.uri {
            return uri.clone();
        }

        let protocol = if self.use_tls { "amqps" } else { "amqp" };
        let host = self.host.as_deref().unwrap_or("localhost");
        let port = self.port.unwrap_or(if self.use_tls { 5671 } else { 5672 });
        let virtual_host = self.virtual_host.as_deref().unwrap_or("/");

        // URL encode the virtual host
        let virtual_host = urlencoding::encode(virtual_host);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            // URL encode the username and password
            let username = urlencoding::encode(username);
            let password = urlencoding::encode(password);

            format!(
                "{}://{}:{}@{}:{}/{}",
                protocol, username, password, host, port, virtual_host
            )
        } else {
            format!("{}://{}:{}/{}", protocol, host, port, virtual_host)
        }
    }
}

/// Client-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Size of the channel buffer for receiving messages
    #[serde(default = "default_channel_buffer")]
    pub channel_buffer_size: usize,

    /// Default prefetch count for consumers
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,

    /// Enable automatic acknowledgment of messages
    #[serde(default)]
    pub auto_ack: bool,

    /// Default timeout for publish operations in milliseconds
    #[serde(default = "default_publish_timeout")]
    pub publish_timeout_ms: u64,

    /// Default timeout for subscribe operations in milliseconds
    #[serde(default = "default_subscribe_timeout")]
    pub subscribe_timeout_ms: u64,
}

fn default_channel_buffer() -> usize {
    100
}

fn default_prefetch_count() -> u16 {
    10
}

fn default_publish_timeout() -> u64 {
    5000
}

fn default_subscribe_timeout() -> u64 {
    10000
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: default_channel_buffer(),
            prefetch_count: default_prefetch_count(),
            auto_ack: false,
            publish_timeout_ms: default_publish_timeout(),
            subscribe_timeout_ms: default_subscribe_timeout(),
        }
    }
}

impl ClientConfig {
    /// Get the publish timeout as a Duration
    pub fn publish_timeout(&self) -> Duration {
        Duration::from_millis(self.publish_timeout_ms)
    }

    /// Get the subscribe timeout as a Duration
    pub fn subscribe_timeout(&self) -> Duration {
        Duration::from_millis(self.subscribe_timeout_ms)
    }
}

/// Configuration for connection retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Initial delay before retrying (in milliseconds)
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    /// Maximum delay between retries (in milliseconds)
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// Multiplier for backoff strategy
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// Whether to use random jitter in retry delays
    #[serde(default = "default_use_jitter")]
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            initial_delay_ms: default_initial_delay(),
            max_delay_ms: default_max_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            use_jitter: default_use_jitter(),
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a specific retry attempt.
    ///
    /// # Arguments
    /// * `attempt` - The current attempt number (starting from 1)
    ///
    /// # Returns
    /// The delay duration for this attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let attempt = attempt.min(self.max_attempts);
        let base_delay = (self.initial_delay_ms as f64
            * self.backoff_multiplier.powi(attempt as i32 - 1)) as u64;
        let delay = base_delay.min(self.max_delay_ms);

        if self.use_jitter {
            // Apply random jitter (±10%)
            let jitter_range = delay / 10;
            let jitter = if jitter_range > 0 {
                let mut rng = rand::thread_rng();
                use rand::Rng;
                rng.gen_range(0..=jitter_range * 2) as i64 - jitter_range as i64
            } else {
                0
            };

            let final_delay = (delay as i64 + jitter).max(0) as u64;
            Duration::from_millis(final_delay)
        } else {
            Duration::from_millis(delay)
        }
    }
}

fn default_max_attempts() -> u32 {
    5
}
fn default_initial_delay() -> u64 {
    500
}
fn default_max_delay() -> u64 {
    30000
}
fn default_backoff_multiplier() -> f64 {
    2.0
}
fn default_use_jitter() -> bool {
    true
}

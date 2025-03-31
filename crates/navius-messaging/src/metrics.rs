use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::sync::RwLock;

/// Metrics collector for messaging operations
pub struct MessagingMetrics {
    /// Start time of the metrics collection
    start_time: Instant,

    /// Total messages published
    pub messages_published: AtomicU64,

    /// Total messages consumed
    pub messages_consumed: AtomicU64,

    /// Total messages acknowledged
    pub messages_acknowledged: AtomicU64,

    /// Total messages rejected
    pub messages_rejected: AtomicU64,

    /// Total connection errors
    pub connection_errors: AtomicU64,

    /// Total publish errors
    pub publish_errors: AtomicU64,

    /// Total consume errors
    pub consume_errors: AtomicU64,

    /// Active connections
    pub active_connections: AtomicU64,

    /// Active channels
    pub active_channels: AtomicU64,

    /// Active publishers
    pub active_publishers: AtomicU64,

    /// Active consumers
    pub active_consumers: AtomicU64,

    /// Publish latencies (in microseconds)
    publish_latencies: DashMap<String, Vec<u64>>,

    /// Consume latencies (in microseconds)
    consume_latencies: DashMap<String, Vec<u64>>,

    /// Custom metrics
    custom_metrics: DashMap<String, AtomicU64>,

    /// Queue metrics
    queue_metrics: RwLock<HashMap<String, QueueMetrics>>,

    /// Exchange metrics
    exchange_metrics: RwLock<HashMap<String, ExchangeMetrics>>,
}

impl Default for MessagingMetrics {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            messages_published: AtomicU64::new(0),
            messages_consumed: AtomicU64::new(0),
            messages_acknowledged: AtomicU64::new(0),
            messages_rejected: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            publish_errors: AtomicU64::new(0),
            consume_errors: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            active_channels: AtomicU64::new(0),
            active_publishers: AtomicU64::new(0),
            active_consumers: AtomicU64::new(0),
            publish_latencies: DashMap::new(),
            consume_latencies: DashMap::new(),
            custom_metrics: DashMap::new(),
            queue_metrics: RwLock::new(HashMap::new()),
            exchange_metrics: RwLock::new(HashMap::new()),
        }
    }
}

impl MessagingMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a shared metrics collector
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Get the uptime of the metrics collector
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Record a message publication
    pub fn record_publish(&self, exchange: &str, latency_us: u64) {
        self.messages_published.fetch_add(1, Ordering::Relaxed);

        self.publish_latencies
            .entry(exchange.to_string())
            .or_default()
            .push(latency_us);

        // Keep only the last 1000 latencies
        if self.publish_latencies.get(exchange).unwrap().len() > 1000 {
            self.publish_latencies.get_mut(exchange).unwrap().remove(0);
        }
    }

    /// Record a message consumption
    pub fn record_consume(&self, queue: &str, latency_us: u64) {
        self.messages_consumed.fetch_add(1, Ordering::Relaxed);

        self.consume_latencies
            .entry(queue.to_string())
            .or_default()
            .push(latency_us);

        // Keep only the last 1000 latencies
        if self.consume_latencies.get(queue).unwrap().len() > 1000 {
            self.consume_latencies.get_mut(queue).unwrap().remove(0);
        }
    }

    /// Record a message acknowledgment
    pub fn record_ack(&self) {
        self.messages_acknowledged.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a message rejection
    pub fn record_reject(&self) {
        self.messages_rejected.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection error
    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a publish error
    pub fn record_publish_error(&self) {
        self.publish_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a consume error
    pub fn record_consume_error(&self) {
        self.consume_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active connections count
    pub fn set_active_connections(&self, count: u64) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    /// Update active channels count
    pub fn set_active_channels(&self, count: u64) {
        self.active_channels.store(count, Ordering::Relaxed);
    }

    /// Update active publishers count
    pub fn set_active_publishers(&self, count: u64) {
        self.active_publishers.store(count, Ordering::Relaxed);
    }

    /// Update active consumers count
    pub fn set_active_consumers(&self, count: u64) {
        self.active_consumers.store(count, Ordering::Relaxed);
    }

    /// Increment a custom metric
    pub fn increment_custom(&self, name: &str, value: u64) {
        self.custom_metrics
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }

    /// Get a custom metric value
    pub fn get_custom(&self, name: &str) -> u64 {
        self.custom_metrics
            .get(name)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Set a custom metric value
    pub fn set_custom(&self, name: &str, value: u64) {
        self.custom_metrics
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::Relaxed);
    }

    /// Get average publish latency for an exchange
    pub fn avg_publish_latency(&self, exchange: &str) -> Option<f64> {
        self.publish_latencies.get(exchange).map(|latencies| {
            if latencies.is_empty() {
                0.0
            } else {
                latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
            }
        })
    }

    /// Get average consume latency for a queue
    pub fn avg_consume_latency(&self, queue: &str) -> Option<f64> {
        self.consume_latencies.get(queue).map(|latencies| {
            if latencies.is_empty() {
                0.0
            } else {
                latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
            }
        })
    }

    /// Update queue metrics
    pub async fn update_queue_metrics(&self, name: &str, metrics: QueueMetrics) {
        let mut queue_metrics = self.queue_metrics.write().await;
        queue_metrics.insert(name.to_string(), metrics);
    }

    /// Get queue metrics
    pub async fn get_queue_metrics(&self, name: &str) -> Option<QueueMetrics> {
        let queue_metrics = self.queue_metrics.read().await;
        queue_metrics.get(name).cloned()
    }

    /// Update exchange metrics
    pub async fn update_exchange_metrics(&self, name: &str, metrics: ExchangeMetrics) {
        let mut exchange_metrics = self.exchange_metrics.write().await;
        exchange_metrics.insert(name.to_string(), metrics);
    }

    /// Get exchange metrics
    pub async fn get_exchange_metrics(&self, name: &str) -> Option<ExchangeMetrics> {
        let exchange_metrics = self.exchange_metrics.read().await;
        exchange_metrics.get(name).cloned()
    }

    /// Get a summary of all metrics
    pub async fn summary(&self) -> MetricsSummary {
        let mut queue_summaries = HashMap::new();
        for (queue, metrics) in self.queue_metrics.read().await.iter() {
            queue_summaries.insert(queue.clone(), metrics.clone());
        }

        let mut exchange_summaries = HashMap::new();
        for (exchange, metrics) in self.exchange_metrics.read().await.iter() {
            exchange_summaries.insert(exchange.clone(), metrics.clone());
        }

        MetricsSummary {
            uptime: self.uptime(),
            messages_published: self.messages_published.load(Ordering::Relaxed),
            messages_consumed: self.messages_consumed.load(Ordering::Relaxed),
            messages_acknowledged: self.messages_acknowledged.load(Ordering::Relaxed),
            messages_rejected: self.messages_rejected.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            publish_errors: self.publish_errors.load(Ordering::Relaxed),
            consume_errors: self.consume_errors.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            active_channels: self.active_channels.load(Ordering::Relaxed),
            active_publishers: self.active_publishers.load(Ordering::Relaxed),
            active_consumers: self.active_consumers.load(Ordering::Relaxed),
            queue_metrics: queue_summaries,
            exchange_metrics: exchange_summaries,
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.messages_published.store(0, Ordering::Relaxed);
        self.messages_consumed.store(0, Ordering::Relaxed);
        self.messages_acknowledged.store(0, Ordering::Relaxed);
        self.messages_rejected.store(0, Ordering::Relaxed);
        self.connection_errors.store(0, Ordering::Relaxed);
        self.publish_errors.store(0, Ordering::Relaxed);
        self.consume_errors.store(0, Ordering::Relaxed);

        self.publish_latencies.clear();
        self.consume_latencies.clear();
        self.custom_metrics.clear();

        let mut queue_metrics = self.queue_metrics.write().await;
        queue_metrics.clear();

        let mut exchange_metrics = self.exchange_metrics.write().await;
        exchange_metrics.clear();
    }
}

/// Metrics for a queue
#[derive(Debug, Clone)]
pub struct QueueMetrics {
    /// Queue name
    pub name: String,

    /// Number of messages in the queue
    pub message_count: u64,

    /// Number of consumers
    pub consumer_count: u64,

    /// Number of messages published to the queue
    pub messages_published: u64,

    /// Number of messages consumed from the queue
    pub messages_consumed: u64,

    /// Number of messages acknowledged
    pub messages_acknowledged: u64,

    /// Number of messages rejected
    pub messages_rejected: u64,

    /// Average message size in bytes
    pub avg_message_size: Option<f64>,

    /// Queue memory usage in bytes
    pub memory_usage: Option<u64>,
}

/// Metrics for an exchange
#[derive(Debug, Clone)]
pub struct ExchangeMetrics {
    /// Exchange name
    pub name: String,

    /// Exchange type
    pub kind: String,

    /// Number of bindings
    pub binding_count: u64,

    /// Number of messages published
    pub messages_published: u64,

    /// Number of messages routed
    pub messages_routed: u64,

    /// Number of messages dropped (not routed)
    pub messages_dropped: u64,
}

/// Summary of all metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Uptime of the metrics collector
    pub uptime: Duration,

    /// Total messages published
    pub messages_published: u64,

    /// Total messages consumed
    pub messages_consumed: u64,

    /// Total messages acknowledged
    pub messages_acknowledged: u64,

    /// Total messages rejected
    pub messages_rejected: u64,

    /// Total connection errors
    pub connection_errors: u64,

    /// Total publish errors
    pub publish_errors: u64,

    /// Total consume errors
    pub consume_errors: u64,

    /// Active connections
    pub active_connections: u64,

    /// Active channels
    pub active_channels: u64,

    /// Active publishers
    pub active_publishers: u64,

    /// Active consumers
    pub active_consumers: u64,

    /// Queue metrics
    pub queue_metrics: HashMap<String, QueueMetrics>,

    /// Exchange metrics
    pub exchange_metrics: HashMap<String, ExchangeMetrics>,
}

impl MetricsSummary {
    /// Calculate the message throughput (messages per second)
    pub fn calculate_throughput(&self) -> f64 {
        if self.uptime.as_secs() == 0 {
            0.0
        } else {
            (self.messages_published + self.messages_consumed) as f64 / self.uptime.as_secs_f64()
        }
    }

    /// Calculate the error rate (errors per second)
    pub fn calculate_error_rate(&self) -> f64 {
        if self.uptime.as_secs() == 0 {
            0.0
        } else {
            (self.connection_errors + self.publish_errors + self.consume_errors) as f64
                / self.uptime.as_secs_f64()
        }
    }

    /// Calculate the success rate (percentage)
    pub fn calculate_success_rate(&self) -> f64 {
        let total_messages = self.messages_published + self.messages_consumed;
        let total_errors = self.publish_errors + self.consume_errors;

        if total_messages == 0 {
            100.0
        } else {
            ((total_messages - total_errors) as f64 / total_messages as f64) * 100.0
        }
    }

    /// Format the summary as a string
    pub fn format(&self) -> String {
        let throughput = self.calculate_throughput();
        let error_rate = self.calculate_error_rate();
        let success_rate = self.calculate_success_rate();

        format!(
            "Messaging Metrics Summary:
- Uptime: {:?}
- Messages: published={}, consumed={}, acked={}, rejected={}
- Errors: connection={}, publish={}, consume={}
- Active: connections={}, channels={}, publishers={}, consumers={}
- Performance: throughput={:.2} msgs/sec, error_rate={:.2} errors/sec, success_rate={:.2}%",
            self.uptime,
            self.messages_published,
            self.messages_consumed,
            self.messages_acknowledged,
            self.messages_rejected,
            self.connection_errors,
            self.publish_errors,
            self.consume_errors,
            self.active_connections,
            self.active_channels,
            self.active_publishers,
            self.active_consumers,
            throughput,
            error_rate,
            success_rate
        )
    }
}

/// Helper struct for measuring operation latency
pub struct LatencyMeasurer {
    /// Start time of the operation
    start: Instant,
}

impl LatencyMeasurer {
    /// Create a new latency measurer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// End the measurement and get the elapsed time in microseconds
    pub fn end(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

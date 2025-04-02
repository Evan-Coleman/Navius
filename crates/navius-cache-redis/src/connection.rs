use crate::metrics;
use crate::{
    config::RedisCacheConfig,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
};
use navius_cache::CacheKey;
use navius_core::health::HealthCheck;
use redis::aio::ConnectionManager;
use redis::{
    aio::ConnectionLike, aio::MultiplexedConnection, Client, Cmd, FromRedisValue, RedisError,
};
use std::future::Future;
use std::{
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::timeout};
use tracing::{debug, error, info, instrument, warn};

/// Statistics for connection pool monitoring
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections created
    pub total_connections_created: usize,
    /// Total number of connections closed
    pub total_connections_closed: usize,
    /// Total number of successful connection acquisitions
    pub total_acquires: usize,
    /// Total number of failed connection acquisitions
    pub total_acquire_failures: usize,
    /// Total number of timeouts during connection acquisition
    pub total_acquire_timeouts: usize,
    /// Current number of active connections
    pub current_active_connections: usize,
    /// Current number of idle connections in the pool
    pub current_idle_connections: usize,
}

/// Health status of a connection
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionHealth {
    /// Connection is healthy
    Healthy,
    /// Connection is degraded (still usable but with issues)
    Degraded(String),
    /// Connection is unhealthy
    Unhealthy(String),
}

/// A connection with metadata for pool management
#[derive(Debug)]
struct PooledConnection {
    /// The actual Redis connection
    connection: MultiplexedConnection,
    /// When the connection was created
    created_at: Instant,
    /// When the connection was last used
    last_used: Instant,
    /// Number of times this connection has been used
    use_count: usize,
}

impl PooledConnection {
    /// Create a new pooled connection
    fn new(connection: MultiplexedConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    /// Update the last used timestamp
    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }

    /// Check if the connection has expired based on max lifetime
    fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    /// Check if the connection has been idle for too long
    fn is_idle_timeout(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }
}

/// Redis connection manager for handling connection pooling
#[derive(Clone)]
pub struct RedisConnectionManager<T> {
    /// Redis client
    client: Client,
    /// Cache configuration
    config: Arc<RedisCacheConfig>,
    /// Connection pool
    pool: Arc<Mutex<Vec<MultiplexedConnection>>>,
    /// Connection pool statistics
    stats: Arc<Mutex<PoolStats>>,
    /// Circuit breaker - tracks consecutive failures
    consecutive_failures: Arc<Mutex<usize>>,
    /// Circuit breaker - last failure time
    last_failure: Arc<Mutex<Option<Instant>>>,
    /// Circuit breaker - is open (allowing connections)
    circuit_open: Arc<Mutex<bool>>,
    /// Maximum number of connections
    max_connections: usize,
    /// Connection timeout
    connection_timeout: Duration,
    /// Key prefix
    key_prefix: Option<String>,
    /// Phantom data for type parameter
    _phantom: PhantomData<T>,
}

impl<T: 'static + Send + FromRedisValue> RedisConnectionManager<T> {
    /// Create a new connection manager
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|err| RedisCacheError::Connection(err.to_string()))?;

        let manager = Self {
            client,
            config: Arc::new(config),
            pool: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            consecutive_failures: Arc::new(Mutex::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            circuit_open: Arc::new(Mutex::new(true)),
            max_connections: config.max_connections as usize,
            connection_timeout: config.connection_timeout(),
            key_prefix: Some(config.key_prefix.clone()),
            _phantom: PhantomData,
        };

        // Create initial connection
        manager.wait_for_available_connection().await?;

        Ok(manager)
    }

    /// Get a connection from the pool
    pub async fn get_connection(&mut self) -> RedisCacheResult<&mut MultiplexedConnection> {
        // Try to get an existing connection
        if let Some(conn) = self.get_available_connection().await {
            return Ok(conn);
        }

        // Create a new connection if we haven't reached the limit
        if self.connections.len() < self.max_connections {
            let conn = self.create_connection().await?;
            self.connections.push(conn);
            return Ok(self.connections.last_mut().unwrap());
        }

        // Wait for a connection to become available
        let start = Instant::now();
        while start.elapsed() < self.connection_timeout {
            if let Some(conn) = self.get_available_connection().await {
                return Ok(conn);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(RedisCacheError::Timeout(format!(
            "Timed out waiting for connection after {}ms",
            self.connection_timeout.as_millis()
        )))
    }

    /// Add prefix to key
    pub fn prefixed_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.key_prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }

    /// Get the key prefix
    pub fn key_prefix(&self) -> &str {
        self.key_prefix.as_deref().unwrap_or("")
    }

    /// Check if Redis is available
    pub async fn ping(&self) -> RedisCacheResult<()> {
        self.execute_command("ping", "PING", |mut conn| async move {
            redis::cmd("PING").query_async(&mut conn).await
        })
        .await
        .map(|_: String| ())
    }

    /// Get the Redis configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Get connection pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let stats = self.stats.lock().await;

        // Record pool stats whenever stats are requested
        metrics::record_connection_pool_stats(
            stats.total_connections_created - stats.total_connections_closed,
            stats.current_idle_connections,
            stats.current_active_connections,
        );

        stats.clone()
    }

    /// Check the health of a connection
    async fn check_connection_health(&self, conn: &mut MultiplexedConnection) -> ConnectionHealth {
        match timeout(
            self.connection_timeout,
            redis::cmd("PING").query_async::<_, String>(conn),
        )
        .await
        {
            Ok(Ok(_)) => ConnectionHealth::Healthy,
            Ok(Err(err)) => ConnectionHealth::Degraded(err.to_string()),
            Err(_) => ConnectionHealth::Unhealthy("Connection timeout".to_string()),
        }
    }

    /// Wait for an available connection
    async fn wait_for_available_connection(&self) -> RedisCacheResult<MultiplexedConnection> {
        let start = Instant::now();
        while start.elapsed() < self.connection_timeout {
            // Check circuit breaker
            if !*self.circuit_open.lock().await {
                if let Some(last_failure) = *self.last_failure.lock().await {
                    if last_failure.elapsed() > Duration::from_secs(30) {
                        self.reset_circuit_breaker().await;
                    } else {
                        return Err(RedisCacheError::Connection(
                            "Circuit breaker is open, Redis connections temporarily disabled"
                                .to_string(),
                        ));
                    }
                }
            }

            // Try to get a connection from the pool
            let mut pool = self.pool.lock().await;
            if let Some(conn) = pool.pop() {
                let mut stats = self.stats.lock().await;
                stats.total_acquires += 1;
                stats.current_active_connections += 1;
                stats.current_idle_connections -= 1;
                return Ok(conn);
            }

            // Create a new connection if pool is not full
            if pool.len() < self.max_connections {
                match timeout(
                    self.connection_timeout,
                    self.client.get_multiplexed_tokio_connection(),
                )
                .await
                {
                    Ok(Ok(conn)) => {
                        let mut stats = self.stats.lock().await;
                        stats.total_connections_created += 1;
                        stats.total_acquires += 1;
                        stats.current_active_connections += 1;
                        return Ok(conn);
                    }
                    Ok(Err(err)) => {
                        self.record_connection_failure().await;
                        return Err(RedisCacheError::Connection(format!(
                            "Failed to create Redis connection: {}",
                            err
                        )));
                    }
                    Err(_) => {
                        self.record_connection_failure().await;
                        return Err(RedisCacheError::Timeout(format!(
                            "Redis connection creation timed out after {}ms",
                            self.connection_timeout.as_millis()
                        )));
                    }
                }
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Could not get a connection after all retries
        Err(RedisCacheError::Connection(
            "Could not acquire a Redis connection after waiting".to_string(),
        ))
    }

    /// Record a connection failure for circuit breaker logic
    async fn record_connection_failure(&self) {
        let mut failures = self.consecutive_failures.lock().await;
        *failures += 1;

        // If we reach the threshold, open the circuit breaker
        let circuit_threshold = 5;
        if *failures >= circuit_threshold {
            debug!(
                "Opening circuit breaker after {} consecutive failures",
                *failures
            );
            *self.circuit_open.lock().await = false;
            *self.last_failure.lock().await = Some(Instant::now());
        }
    }

    /// Reset the circuit breaker state after a successful operation
    async fn reset_circuit_breaker(&self) {
        let mut circuit_open = self.circuit_open.lock().await;
        if !*circuit_open {
            debug!("Resetting circuit breaker due to successful operation");
            *circuit_open = true;
            *self.consecutive_failures.lock().await = 0;
            *self.last_failure.lock().await = None;
        }
    }

    /// Start background tasks for connection pool maintenance
    fn start_pool_maintenance(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs(30);
            loop {
                tokio::time::sleep(interval).await;
                this.clean_expired_connections().await;
            }
        });
    }

    /// Clean expired or idle connections from the pool
    async fn clean_expired_connections(&self) {
        let mut pool = self.pool.lock().await;
        let mut stats = self.stats.lock().await;

        let max_lifetime = Duration::from_secs(self.config.command_timeout_seconds * 10);
        let idle_timeout = Duration::from_secs(self.config.command_timeout_seconds * 5);

        // Find and remove expired connections
        let before_count = pool.len();
        pool.retain(|conn| {
            let keep = !conn.is_expired(max_lifetime) && !conn.is_idle_timeout(idle_timeout);
            if !keep {
                stats.total_connections_closed += 1;
                stats.current_idle_connections -= 1;
            }
            keep
        });

        let removed = before_count - pool.len();
        if removed > 0 {
            debug!("Removed {} expired connections from pool", removed);
        }

        // Check if we need to add more connections to maintain minimum
        let min_connections = std::cmp::max(1, self.config.max_connections / 5) as usize;
        if pool.len() < min_connections {
            let to_add = min_connections - pool.len();
            debug!(
                "Adding {} new connections to maintain minimum pool size",
                to_add
            );

            for _ in 0..to_add {
                if let Ok(conn) = self.client.get_multiplexed_async_connection().await {
                    pool.push(conn);
                    stats.total_connections_created += 1;
                    stats.current_idle_connections += 1;
                } else {
                    // Stop if we can't create more connections
                    break;
                }
            }
        }
    }

    pub async fn get(&self) -> Result<ConnectionManager, RedisError> {
        self.client.get_connection_manager().await
    }

    /// Execute a command with automatic connection management and error handling
    pub async fn execute_command<F, Fut, R>(
        &self,
        operation: &str,
        command: &str,
        f: F,
    ) -> RedisCacheResult<R>
    where
        F: FnOnce(&mut MultiplexedConnection) -> Fut,
        Fut: Future<Output = RedisResult<R>>,
    {
        let start = Instant::now();
        let mut retries = 0;
        let max_retries = 3;

        loop {
            let mut conn = self.wait_for_available_connection().await?;

            match timeout(self.connection_timeout, f(&mut conn)).await {
                Ok(Ok(result)) => {
                    metrics::record_operation_success::<T>(command);
                    metrics::record_operation_duration::<T>(command, start.elapsed());
                    return Ok(result);
                }
                Ok(Err(err)) => {
                    metrics::record_operation_error::<T>(
                        command,
                        &RedisCacheError::Redis(err.to_string()),
                    );
                    if retries >= max_retries {
                        return Err(RedisCacheError::Redis(format!(
                            "Command {} failed after {} retries: {}",
                            command, max_retries, err
                        )));
                    }
                    retries += 1;
                    continue;
                }
                Err(_) => {
                    metrics::record_operation_error::<T>(
                        command,
                        &RedisCacheError::Timeout(format!(
                            "Command {} timed out after {}ms",
                            command,
                            self.connection_timeout.as_millis()
                        )),
                    );
                    if retries >= max_retries {
                        return Err(RedisCacheError::Timeout(format!(
                            "Command {} timed out after {} retries",
                            command, max_retries
                        )));
                    }
                    retries += 1;
                    continue;
                }
            }
        }
    }
}

impl<T: 'static + Send + FromRedisValue> ConnectionLike for RedisConnectionManager<T> {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        Box::pin(async move {
            let mut conn = self.get().await?;
            conn.req_packed_command(cmd).await
        })
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        pipeline: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        Box::pin(async move {
            let mut conn = self.get().await?;
            conn.req_packed_commands(pipeline, offset, count).await
        })
    }

    fn get_db(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisCacheConfig;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_pool() {
        // Skip test if Redis is not available
        let config = RedisCacheConfig {
            url: "127.0.0.1:6379".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(60),
            max_connections: 5,
            min_connections: 2,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 2,
            command_timeout_seconds: 1,
            idle_timeout_seconds: 60,
            max_lifetime_seconds: 300,
            retry_commands: true,
            max_retries: 3,
            health_check_interval_seconds: 30,
            circuit_breaker_threshold: 5,
            circuit_reset_timeout_seconds: 5,
            enable_metrics: false,
        };

        let connection_manager = match RedisConnectionManager::new(config).await {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test_connection_pool - Redis not available");
                return;
            }
        };

        // Get connection stats before
        let before_stats = connection_manager.get_stats().await;

        // Get multiple connections
        let mut connections = Vec::new();
        for i in 0..3 {
            match connection_manager.get_connection().await {
                Ok(conn) => {
                    println!("Got connection {}", i);
                    connections.push(conn);
                }
                Err(e) => panic!("Failed to get connection {}: {}", i, e),
            }
        }

        // Check stats after getting connections
        let after_stats = connection_manager.get_stats().await;
        assert!(after_stats.total_acquires >= before_stats.total_acquires + 3);
        assert!(after_stats.current_active_connections >= 3);

        // Return some connections
        for conn in connections {
            connection_manager.return_connection(conn).await;
        }

        // Check stats after returning
        let final_stats = connection_manager.get_stats().await;
        assert!(final_stats.current_idle_connections >= before_stats.current_idle_connections);

        // Test ping
        connection_manager
            .ping()
            .await
            .expect("Ping should succeed");
    }
}

impl<T: 'static + Send + FromRedisValue> RedisConnectionManager<T> {
    #[instrument(skip(self, filled_pipeline))]
    pub async fn execute_pipeline_with_retry<T: FromRedisValue>(
        &self,
        mut filled_pipeline: redis::Pipeline,
        timeout_duration: Duration,
        retry_count: u32,
    ) -> RedisCacheResult<T> {
        let mut attempts = 0;
        loop {
            match self
                .execute_pipeline_once(filled_pipeline.clone(), timeout_duration)
                .await
            {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempts += 1;
                    if attempts >= retry_count {
                        return Err(err);
                    }
                    if let RedisCacheError::Redis(ref redis_err) = err {
                        match redis_err.kind() {
                            redis::ErrorKind::IoError | redis::ErrorKind::ReadTimeout => {
                                continue;
                            }
                            _ => return Err(err),
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    #[instrument(skip(self, filled_pipeline))]
    async fn execute_pipeline_once(
        &self,
        filled_pipeline: redis::Pipeline,
        timeout_duration: Duration,
    ) -> RedisCacheResult<T> {
        let mut conn = self.get_connection().await?;
        let execution_future = filled_pipeline.query_async(&mut conn);
        match timeout(timeout_duration, execution_future).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => {
                metrics::record_operation_error::<T>(metrics::names::PIPELINE_EXECUTE, &err);
                match err.kind() {
                    redis::ErrorKind::IoError | redis::ErrorKind::ReadTimeout => {
                        Err(RedisCacheError::Redis(err))
                    }
                    _ => Err(RedisCacheError::Redis(err)),
                }
            }
            Err(_) => {
                metrics::record_operation_error::<T>(
                    metrics::names::PIPELINE_EXECUTE_TIMEOUT,
                    &RedisError::from(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Pipeline execution timed out",
                    )),
                );
                Err(RedisCacheError::Timeout)
            }
        }
    }
}

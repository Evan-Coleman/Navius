use crate::metrics;
use crate::{
    config::RedisCacheConfig,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
};
use redis::{aio::ConnectionManager, Client, Connection, RedisError};
use std::{
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
    connection: Connection,
    /// When the connection was created
    created_at: Instant,
    /// When the connection was last used
    last_used: Instant,
    /// Number of times this connection has been used
    use_count: usize,
}

impl PooledConnection {
    /// Create a new pooled connection
    fn new(connection: Connection) -> Self {
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
pub struct RedisConnectionManager {
    /// Redis client
    client: Arc<Client>,
    /// Cache configuration
    config: RedisCacheConfig,
    /// Connection pool
    pool: Arc<Mutex<Vec<PooledConnection>>>,
    /// Connection pool statistics
    stats: Arc<Mutex<PoolStats>>,
    /// Circuit breaker - tracks consecutive failures
    consecutive_failures: Arc<Mutex<usize>>,
    /// Circuit breaker - last failure time
    last_failure: Arc<Mutex<Option<Instant>>>,
    /// Circuit breaker - is open (allowing connections)
    circuit_open: Arc<Mutex<bool>>,
}

impl RedisConnectionManager {
    /// Create a new Redis connection manager
    #[instrument(skip(config), level = "debug")]
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        info!("Initializing Redis connection manager");

        let redis_url = if config.url.starts_with("redis://") {
            config.url.clone()
        } else {
            format!("redis://{}", config.url)
        };

        debug!("Connecting to Redis at: {}", redis_url);

        let client = match Client::open(redis_url) {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to create Redis client: {}", err);
                return Err(RedisCacheError::ConnectionError(err.to_string()));
            }
        };

        // Create initial connections up to min_connections (or 1 as minimum for testing)
        let min_connections = std::cmp::max(1, config.max_connections / 5);
        let mut initial_connections = Vec::with_capacity(min_connections as usize);

        // Initialize with at least one verified connection
        match client.get_connection() {
            Ok(conn) => {
                initial_connections.push(PooledConnection::new(conn));
            }
            Err(err) => {
                error!("Failed to establish initial Redis connection: {}", err);
                return Err(RedisCacheError::ConnectionError(format!(
                    "Failed to connect to Redis: {}",
                    err
                )));
            }
        }

        // Try to establish the minimum connections (don't fail if we can't get them all)
        for _ in 1..min_connections {
            if let Ok(conn) = client.get_connection() {
                initial_connections.push(PooledConnection::new(conn));
            } else {
                // We already have at least one connection, so just warn
                warn!("Could not establish all initial Redis connections");
                break;
            }
        }

        let stats = PoolStats {
            total_connections_created: initial_connections.len(),
            current_idle_connections: initial_connections.len(),
            ..Default::default()
        };

        let manager = Self {
            client: Arc::new(client),
            config,
            pool: Arc::new(Mutex::new(initial_connections)),
            stats: Arc::new(Mutex::new(stats)),
            consecutive_failures: Arc::new(Mutex::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            circuit_open: Arc::new(Mutex::new(true)),
        };

        // Start the background tasks for pool maintenance
        manager.start_pool_maintenance();

        debug!(
            "Redis connection manager initialized with {} connections",
            min_connections
        );
        Ok(manager)
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_connection(&self) -> RedisCacheResult<Connection> {
        let start_time = Instant::now();

        // Check if circuit breaker is open
        if !*self.circuit_open.lock().await {
            let mut last_failure = self.last_failure.lock().await;
            if let Some(time) = *last_failure {
                // Reset circuit breaker after circuit_reset_timeout
                if time.elapsed() > Duration::from_secs(5) {
                    debug!("Resetting circuit breaker after timeout");
                    *self.circuit_open.lock().await = true;
                    *self.consecutive_failures.lock().await = 0;
                    *last_failure = None;
                } else {
                    let err = RedisCacheError::ConnectionError(
                        "Circuit breaker is open, Redis connections temporarily disabled"
                            .to_string(),
                    );

                    // Record the error in metrics
                    metrics::record_operation_error(metrics::names::CONNECTION_ACQUIRE, &err);

                    return Err(err);
                }
            }
        }

        // First try to get a connection from the pool
        let mut pool = self.pool.lock().await;

        // Find a non-expired connection
        let max_lifetime = Duration::from_secs(self.config.command_timeout_seconds * 10);
        let idle_timeout = Duration::from_secs(self.config.command_timeout_seconds * 5);

        let conn_index = pool
            .iter()
            .position(|conn| !conn.is_expired(max_lifetime) && !conn.is_idle_timeout(idle_timeout));

        if let Some(index) = conn_index {
            // Get and remove the connection from the pool
            let mut conn = pool.remove(index);

            // Perform a quick health check before returning
            let health = self.check_connection_health(&conn.connection).await;

            // Record health in metrics
            metrics::record_connection_health(&health);

            match health {
                ConnectionHealth::Healthy => {
                    // Mark connection as used
                    conn.mark_used();

                    // Update stats
                    let mut stats = self.stats.lock().await;
                    stats.total_acquires += 1;
                    stats.current_active_connections += 1;
                    stats.current_idle_connections -= 1;

                    // Record pool stats
                    metrics::record_connection_pool_stats(
                        stats.total_connections_created - stats.total_connections_closed,
                        stats.current_idle_connections,
                        stats.current_active_connections,
                    );

                    // Record connection acquisition time
                    metrics::record_connection_acquisition(start_time.elapsed());

                    // Record successful operation
                    metrics::record_operation_success(metrics::names::CONNECTION_ACQUIRE);

                    return Ok(conn.connection);
                }
                ConnectionHealth::Degraded(reason) | ConnectionHealth::Unhealthy(reason) => {
                    debug!("Discarding unhealthy connection from pool: {}", reason);
                    let mut stats = self.stats.lock().await;
                    stats.total_connections_closed += 1;
                    stats.current_idle_connections -= 1;

                    // Record pool stats
                    metrics::record_connection_pool_stats(
                        stats.total_connections_created - stats.total_connections_closed,
                        stats.current_idle_connections,
                        stats.current_active_connections,
                    );

                    // Try to create a new connection instead
                    drop(stats);
                    drop(pool);

                    // Record the error
                    let err = RedisCacheError::ConnectionError(reason);
                    metrics::record_operation_error(metrics::names::CONNECTION_ACQUIRE, &err);

                    return self.create_connection().await;
                }
            }
        }

        // No suitable connection found in the pool, check if we can create a new one
        let stats = {
            let stats = self.stats.lock().await;
            *stats
        };

        let active_connections = stats.current_active_connections;
        let idle_connections = stats.current_idle_connections;
        let total_connections = active_connections + idle_connections;
        let max_connections = self.config.max_connections as usize;

        if total_connections >= max_connections {
            // We've reached the max connections limit
            // Wait for a connection to be returned or create a new one if timeout
            debug!(
                "Connection pool at capacity ({}/{}), waiting for available connection",
                total_connections, max_connections
            );

            drop(pool);
            return self.wait_for_available_connection().await;
        }

        // Create a new connection
        drop(pool);
        let result = self.create_connection().await;

        // Record acquisition time
        metrics::record_connection_acquisition(start_time.elapsed());

        // Record result
        match &result {
            Ok(_) => metrics::record_operation_success(metrics::names::CONNECTION_ACQUIRE),
            Err(err) => metrics::record_operation_error(metrics::names::CONNECTION_ACQUIRE, err),
        }

        result
    }

    /// Create a new connection
    async fn create_connection(&self) -> RedisCacheResult<Connection> {
        let timer = metrics::TimedOperation::new(metrics::names::CONNECTION_ACQUIRE);

        let conn_timeout = Duration::from_secs(self.config.connect_timeout_seconds);
        let conn_result = timeout(conn_timeout, self.client.get_async_connection()).await;

        match conn_result {
            Ok(Ok(conn)) => {
                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.total_connections_created += 1;
                    stats.total_acquires += 1;
                    stats.current_active_connections += 1;

                    // Record pool stats
                    metrics::record_connection_pool_stats(
                        stats.total_connections_created - stats.total_connections_closed,
                        stats.current_idle_connections,
                        stats.current_active_connections,
                    );
                }

                // Reset consecutive failures on success
                *self.consecutive_failures.lock().await = 0;

                timer.record_success();
                Ok(conn)
            }
            Ok(Err(err)) => {
                // Redis error
                self.record_connection_failure().await;

                let error = RedisCacheError::ConnectionError(format!(
                    "Failed to create Redis connection: {}",
                    err
                ));

                timer.record_error(&error);
                Err(error)
            }
            Err(_) => {
                // Timeout error
                self.record_connection_failure().await;

                let error =
                    RedisCacheError::Timeout("Redis connection creation timed out".to_string());

                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.total_acquire_timeouts += 1;
                }

                timer.record_error(&error);
                Err(error)
            }
        }
    }

    /// Return a connection to the pool
    pub async fn return_connection(&self, conn: Connection) {
        let health = self.check_connection_health(&conn).await;

        // Record health in metrics
        metrics::record_connection_health(&health);

        let mut pool = self.pool.lock().await;
        let mut stats = self.stats.lock().await;
        stats.current_active_connections -= 1;

        match health {
            ConnectionHealth::Healthy => {
                pool.push(PooledConnection::new(conn));
                stats.current_idle_connections += 1;
            }
            ConnectionHealth::Degraded(reason) | ConnectionHealth::Unhealthy(reason) => {
                debug!("Not returning unhealthy connection to pool: {}", reason);
                stats.total_connections_closed += 1;
            }
        }

        // Record pool stats
        metrics::record_connection_pool_stats(
            stats.total_connections_created - stats.total_connections_closed,
            stats.current_idle_connections,
            stats.current_active_connections,
        );
    }

    /// Execute a Redis command and handle errors
    pub async fn execute_command<F, T>(
        &self,
        key: &str,
        operation: &str,
        func: F,
    ) -> RedisCacheResult<T>
    where
        F: FnOnce(Connection) -> Result<T, RedisError>,
    {
        let conn = self.get_connection().await?;

        // Track start time for operation
        let start = Instant::now();

        // Execute the command with timeout
        let command_timeout = Duration::from_secs(self.config.command_timeout_seconds);
        let operation_future = tokio::task::spawn_blocking(move || func(conn));

        let result = match timeout(command_timeout, operation_future).await {
            Ok(task_result) => match task_result {
                Ok(cmd_result) => cmd_result,
                Err(join_err) => {
                    error!("Redis operation task failed: {}", join_err);
                    Err(RedisError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Task execution error: {}", join_err),
                    )))
                }
            },
            Err(_) => {
                error!("Redis operation timed out after {:?}", command_timeout);
                Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "Operation '{}' timed out after {:?}",
                        operation, command_timeout
                    ),
                )))
            }
        };

        // Calculate operation duration
        let duration = start.elapsed();
        if duration > Duration::from_millis(100) {
            warn!("Redis operation '{}' took {:?}", operation, duration);
        }

        match result {
            Ok(value) => {
                // Return a new connection to the pool
                if let Ok(new_conn) = self.client.get_connection() {
                    self.return_connection(new_conn).await;
                }
                Ok(value)
            }
            Err(err) => {
                // Record failure for circuit breaker
                self.record_connection_failure().await;

                error!(
                    "Redis operation '{}' failed on key '{}': {}",
                    operation, key, err
                );
                Err(RedisCacheError::OperationError(format!(
                    "Redis operation '{}' failed: {}",
                    operation, err
                )))
            }
        }
    }

    /// Execute a pipeline command on a Redis connection
    ///
    /// This method is similar to execute_command but is optimized for pipeline operations
    /// which may operate on multiple keys.
    #[instrument(skip(self, func), level = "debug")]
    pub async fn execute_pipeline_command<F, T>(
        &self,
        operation: &str,
        func: F,
    ) -> RedisCacheResult<T>
    where
        F: FnOnce(Connection) -> Result<T, RedisError>,
    {
        let start_time = Instant::now();

        // Get a connection from the pool
        let connection = match self.get_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(
                    "Failed to get connection for pipeline operation {}: {}",
                    operation, e
                );
                metrics::record_operation_error(
                    &format!("{}_pipeline", metrics::names::CONNECTION_ERROR),
                    &e,
                );
                return Err(e);
            }
        };

        // Execute the command with timeout
        let timeout_duration = Duration::from_secs(self.config.command_timeout_seconds);
        let result = match timeout(timeout_duration, async { func(connection) }).await {
            Ok(Ok(result)) => {
                // Record successful operation
                debug!("Pipeline operation {} completed successfully", operation);
                Ok(result)
            }
            Ok(Err(redis_error)) => {
                // Record Redis error
                error!(
                    "Redis error during pipeline operation {}: {}",
                    operation, redis_error
                );
                self.record_connection_failure().await;
                metrics::record_operation_error(
                    operation,
                    &RedisCacheError::from(redis_error.clone()),
                );
                Err(redis_error.into())
            }
            Err(timeout_error) => {
                // Record timeout error
                error!(
                    "Timeout during pipeline operation {}: {}",
                    operation, timeout_error
                );
                self.record_connection_failure().await;
                let error = RedisCacheError::Timeout(format!(
                    "Operation timed out after {:?}: {}",
                    timeout_duration, timeout_error
                ));
                metrics::record_operation_error(operation, &error);
                Err(error)
            }
        };

        // Record operation time
        let elapsed = start_time.elapsed();
        if let Ok(ref result_val) = result {
            metrics::record_operation_duration(operation, elapsed);
        }

        result
    }

    /// Create a prefixed key
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        if self.config.key_prefix.is_empty() {
            key.as_ref().to_string()
        } else {
            format!("{}:{}", self.config.key_prefix, key.as_ref())
        }
    }

    /// Get the key prefix
    pub fn key_prefix(&self) -> &str {
        &self.config.key_prefix
    }

    /// Check if Redis is available
    pub async fn ping(&self) -> RedisCacheResult<()> {
        self.execute_command("ping", "PING", |mut conn| {
            redis::cmd("PING").query::<String>(&mut conn).map(|_| ())
        })
        .await
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
    async fn check_connection_health(&self, conn: &Connection) -> ConnectionHealth {
        let timeout_duration = Duration::from_millis(500); // Quick health check timeout

        match timeout(
            timeout_duration,
            conn.clone().req_command(&redis::cmd("PING")),
        )
        .await
        {
            Ok(Ok(_)) => ConnectionHealth::Healthy,
            Ok(Err(err)) => ConnectionHealth::Unhealthy(format!("Ping failed: {}", err)),
            Err(_) => ConnectionHealth::Unhealthy(String::from("Health check timed out")),
        }
    }

    /// Wait for an available connection
    async fn wait_for_available_connection(&self) -> RedisCacheResult<Connection> {
        let retry_interval = Duration::from_millis(50);
        let max_retries = (self.config.connection_timeout_seconds * 1000) / 50;

        for _ in 0..max_retries {
            // Try to get a connection from the pool
            let mut pool = self.pool.lock().await;
            if !pool.is_empty() {
                // There's a connection in the pool, use it
                let conn = pool.remove(0);

                // Update stats
                let mut stats = self.stats.lock().await;
                stats.total_acquires += 1;
                stats.current_idle_connections -= 1;
                stats.current_active_connections += 1;

                return Ok(conn.connection);
            }

            // No connection available, release lock and wait
            drop(pool);
            tokio::time::sleep(retry_interval).await;
        }

        // Could not get a connection after all retries
        Err(RedisCacheError::ConnectionError(
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
                if let Ok(conn) = self.client.get_connection() {
                    pool.push(PooledConnection::new(conn));
                    stats.total_connections_created += 1;
                    stats.current_idle_connections += 1;
                } else {
                    // Stop if we can't create more connections
                    break;
                }
            }
        }
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

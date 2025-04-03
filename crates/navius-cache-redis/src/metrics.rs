use crate::error::RedisCacheError;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Metric names for Redis cache operations
pub struct MetricNames;

impl MetricNames {
    // Basic operations
    /// Metric for get operations
    pub const GET: &'static str = "cache_redis_get";
    /// Metric for set operations
    pub const SET: &'static str = "cache_redis_set";
    /// Metric for delete operations
    pub const DELETE: &'static str = "cache_redis_delete";
    /// Metric for exists operations
    pub const EXISTS: &'static str = "cache_redis_exists";
    /// Metric for expire operations
    pub const EXPIRE: &'static str = "cache_redis_expire";
    /// Metric for increment operations
    pub const INCREMENT: &'static str = "cache_redis_increment";
    /// Metric for clear operations
    pub const CLEAR: &'static str = "redis_cache_clear";

    // List operations
    /// Metric for list push operations
    pub const LIST_PUSH: &'static str = "redis_cache_list_push";
    /// Metric for list pop operations
    pub const LIST_POP: &'static str = "redis_cache_list_pop";
    /// Metric for list range operations
    pub const LIST_RANGE: &'static str = "redis_cache_list_range";
    /// Metric for list length operations
    pub const LIST_LENGTH: &'static str = "redis_cache_list_length";

    // Hash operations
    /// Metric for hash get operations
    pub const HASH_GET: &'static str = "redis_cache_hash_get";
    /// Metric for hash set operations
    pub const HASH_SET: &'static str = "redis_cache_hash_set";
    /// Metric for hash delete operations
    pub const HASH_DELETE: &'static str = "redis_cache_hash_delete";

    // Connection metrics
    /// Metric for connection pool size
    pub const POOL_SIZE: &'static str = "cache_redis_pool_size";
    /// Metric for active connections
    pub const ACTIVE_CONNECTIONS: &'static str = "redis_cache_active_connections";
    /// Metric for idle connections
    pub const IDLE_CONNECTIONS: &'static str = "redis_cache_idle_connections";

    // Result metrics
    /// Metric for cache hits
    pub const CACHE_HIT: &'static str = "cache_redis_hit";
    /// Metric for cache misses
    pub const CACHE_MISS: &'static str = "cache_redis_miss";
    /// Metric for cache errors
    pub const CACHE_ERROR: &'static str = "redis_cache_error";

    // New constants
    pub const PING: &'static str = "cache_redis_ping";
    pub const SCAN: &'static str = "cache_redis_scan";
    pub const FLUSH: &'static str = "cache_redis_flush";
    pub const CACHE: &'static str = "cache_redis";
}

/// Record operation timing metrics
pub fn record_operation_timing(operation: &str, duration: Duration) {
    if cfg!(feature = "metrics") {
        // Implementation with real metrics library would go here
        // For now, we'll just log the timing
        if duration > Duration::from_millis(100) {
            warn!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "Slow Redis operation"
            );
        } else {
            debug!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "Redis operation timing"
            );
        }
    }
}

/// Record cache hit metrics
pub fn record_cache_hit(operation: &str) {
    if cfg!(feature = "metrics") {
        // Implementation with real metrics library would go here
        // For now, we'll just log the hit
        debug!(operation = operation, "Cache hit");

        #[cfg(feature = "metrics")]
        {
            metrics::counter!(MetricNames::CACHE_HIT);
        }
    }
}

/// Record cache miss metrics
pub fn record_cache_miss(operation: &str) {
    if cfg!(feature = "metrics") {
        // Implementation with real metrics library would go here
        // For now, we'll just log the miss
        debug!(operation = operation, "Cache miss");

        #[cfg(feature = "metrics")]
        {
            metrics::counter!(MetricNames::CACHE_MISS);
        }
    }
}

/// Record cache error metrics
pub fn record_cache_error(operation: &str, error: &RedisCacheError) {
    if cfg!(feature = "metrics") {
        // Implementation with real metrics library would go here
        // For now, we'll just log the error
        warn!(
            operation = operation,
            error = %error,
            "Cache operation error"
        );

        #[cfg(feature = "metrics")]
        {
            metrics::counter!(MetricNames::CACHE_ERROR);
        }
    }
}

/// Record connection pool metrics
pub(crate) fn record_pool_metrics(pool_size: u32, active_connections: u32, idle_connections: u32) {
    #[cfg(feature = "metrics")]
    {
        let pool_size_key = format!("{}.pool_size", MetricNames::CACHE);
        metrics::counter!(pool_size_key);
        let active_key = format!("{}.active_connections", MetricNames::CACHE);
        metrics::counter!(active_key);
        let idle_key = format!("{}.idle_connections", MetricNames::CACHE);
        metrics::counter!(idle_key);

        // Log the metrics values for now
        debug!(
            pool_size = pool_size,
            active_connections = active_connections,
            idle_connections = idle_connections,
            "Connection pool stats"
        );
    }
}

/// Timer for measuring operation durations
pub struct OperationTimer {
    /// Start time of the operation
    start: Instant,
    /// Name of the operation
    operation: String,
}

impl OperationTimer {
    /// Create a new timer for the given operation
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    /// Record the timing of the operation and log it if it was slow
    pub fn record(&self) {
        let duration = self.start.elapsed();
        record_operation_timing(&self.operation, duration);

        #[cfg(feature = "metrics")]
        {
            metrics::counter!(format!("{}_duration", self.operation.clone()));
        }
    }

    /// Record the timing of the operation with success status
    pub fn record_success(&self) {
        self.record();

        #[cfg(feature = "metrics")]
        {
            metrics::counter!(self.operation.clone());
        }
    }

    /// Record the timing of the operation with error status
    pub fn record_error(&self, error: &RedisCacheError) {
        self.record();
        record_cache_error(&self.operation, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RedisCacheError;

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new(MetricNames::GET);
        timer.record();

        // Add small delay to test timing
        std::thread::sleep(Duration::from_millis(10));

        let timer = OperationTimer::new(MetricNames::SET);
        timer.record_success();

        let timer = OperationTimer::new(MetricNames::DELETE);
        timer.record_error(&RedisCacheError::Timeout("Test timeout".to_string()));
    }
}

// Redis cache metrics implementation
//
// This module provides metrics collection and reporting for the Redis cache implementation.
// It works with the metrics crate to record various performance and operational metrics.

use crate::connection::ConnectionHealth;
use crate::error::RedisCacheResult;
use metrics::{counter, gauge, histogram};
use std::time::{Duration, Instant};

/// Metric names for Redis cache operations
pub mod names {
    // Operation metrics
    pub const GET: &str = "redis_cache_get";
    pub const SET: &str = "redis_cache_set";
    pub const DELETE: &str = "redis_cache_delete";
    pub const EXISTS: &str = "redis_cache_exists";
    pub const EXPIRE: &str = "redis_cache_expire";
    pub const TTL: &str = "redis_cache_ttl";
    pub const INVALIDATE: &str = "redis_cache_invalidate";
    pub const GET_MANY: &str = "redis_cache_get_many";
    pub const SET_MANY: &str = "redis_cache_set_many";
    pub const DELETE_MANY: &str = "redis_cache_delete_many";

    // Collection operation metrics
    pub const LIST_PUSH: &str = "redis_cache_list_push";
    pub const LIST_POP: &str = "redis_cache_list_pop";
    pub const LIST_RANGE: &str = "redis_cache_list_range";
    pub const LIST_LENGTH: &str = "redis_cache_list_length";
    pub const SET_ADD: &str = "redis_cache_set_add";
    pub const SET_REMOVE: &str = "redis_cache_set_remove";
    pub const SET_MEMBERS: &str = "redis_cache_set_members";
    pub const HASH_SET: &str = "redis_cache_hash_set";
    pub const HASH_GET: &str = "redis_cache_hash_get";
    pub const HASH_DELETE: &str = "redis_cache_hash_delete";

    // Advanced operation metrics
    pub const PIPELINE_EXECUTE: &str = "redis_cache_pipeline_execute";
    pub const SCRIPT_EXECUTE: &str = "redis_cache_script_execute";

    // Connection metrics
    pub const CONNECTION_ACQUIRE: &str = "redis_cache_connection_acquire";
    pub const CONNECTION_RELEASE: &str = "redis_cache_connection_release";
    pub const CONNECTION_ERROR: &str = "redis_cache_connection_error";
    pub const CONNECTION_TIMEOUT: &str = "redis_cache_connection_timeout";
    pub const CONNECTION_POOL_SIZE: &str = "redis_cache_connection_pool_size";
    pub const CONNECTION_POOL_IDLE: &str = "redis_cache_connection_pool_idle";
    pub const CONNECTION_POOL_USED: &str = "redis_cache_connection_pool_used";

    // Health metrics
    pub const HEALTH_CHECK: &str = "redis_cache_health_check";
}

/// Prefix used for all Redis cache metrics
const METRIC_PREFIX: &str = "navius_";

/// Create a prefixed metric name
fn create_key(name: &str) -> String {
    format!("{}{}", METRIC_PREFIX, name)
}

/// Record a Redis operation with timing and result
pub fn record_operation<T>(name: &str, start: Instant, result: &RedisCacheResult<T>) {
    let duration = start.elapsed();
    record_operation_duration(name, duration);

    match result {
        Ok(_) => record_operation_success(name),
        Err(err) => record_operation_error(name, err),
    }
}

/// Record operation timing
pub fn record_operation_duration(name: &str, duration: Duration) {
    let key = create_key(&format!("{}_duration_ms", name));
    let duration_ms = duration.as_secs_f64() * 1000.0;
    histogram!(key).record(duration_ms);
}

/// Record successful operation
pub fn record_operation_success(name: &str) {
    let key = create_key(&format!("{}_success", name));
    counter!(key).increment(1);
}

/// Record operation error
pub fn record_operation_error(name: &str, error: &crate::error::RedisCacheError) {
    let key = create_key(&format!("{}_error", name));
    counter!(key).increment(1);

    // Record specific error types based on defined RedisCacheError variants
    let error_type = get_error_type(error);

    let error_key = create_key(&format!("{}_error_{}", name, error_type));
    counter!(error_key).increment(1);
}

fn get_error_type(error: &crate::error::RedisCacheError) -> &'static str {
    match error {
        crate::error::RedisCacheError::ConnectionError(_) => "connection",
        crate::error::RedisCacheError::OperationError(_) => "operation",
        crate::error::RedisCacheError::SerializationError(_) => "serialization",
        crate::error::RedisCacheError::DeserializationError(_) => "deserialization",
        crate::error::RedisCacheError::Timeout(_) => "timeout",
        crate::error::RedisCacheError::UnsupportedOperation(_) => "unsupported_operation",
    }
}

/// Record connection pool stats
pub fn record_connection_pool_stats(size: usize, idle: usize, used: usize) {
    gauge!(create_key(names::CONNECTION_POOL_SIZE)).set(size as f64);
    gauge!(create_key(names::CONNECTION_POOL_IDLE)).set(idle as f64);
    gauge!(create_key(names::CONNECTION_POOL_USED)).set(used as f64);
}

/// Record connection acquisition time
pub fn record_connection_acquisition(duration: Duration) {
    let key = create_key(names::CONNECTION_ACQUIRE);
    let duration_ms = duration.as_secs_f64() * 1000.0;
    histogram!(key).record(duration_ms);
}

/// Record connection health
pub fn record_connection_health(health: &ConnectionHealth) {
    let key = create_key(names::HEALTH_CHECK);

    match health {
        ConnectionHealth::Healthy => {
            counter!(format!("{}_healthy", key)).increment(1);
            gauge!(format!("{}_status", key)).set(1.0);
        }
        ConnectionHealth::Degraded(reason) => {
            counter!(format!("{}_degraded", key)).increment(1);
            counter!(format!(
                "{}_degraded_{}",
                key,
                reason.to_lowercase().replace(' ', "_")
            ))
            .increment(1);
            gauge!(format!("{}_status", key)).set(0.5);
        }
        ConnectionHealth::Unhealthy(reason) => {
            counter!(format!("{}_unhealthy", key)).increment(1);
            counter!(format!(
                "{}_unhealthy_{}",
                key,
                reason.to_lowercase().replace(' ', "_")
            ))
            .increment(1);
            gauge!(format!("{}_status", key)).set(0.0);
        }
    }
}

/// A utility struct to time operations and record metrics automatically
pub struct TimedOperation {
    name: String,
    start: Instant,
}

impl TimedOperation {
    /// Create a new timed operation
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    /// Record the result of the operation
    pub fn record<T>(&self, result: &RedisCacheResult<T>) {
        record_operation(&self.name, self.start, result);
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        record_operation_duration(&self.name, self.start.elapsed());
        record_operation_success(&self.name);
    }

    /// Record a failed operation
    pub fn record_error(&self, error: &crate::error::RedisCacheError) {
        record_operation_duration(&self.name, self.start.elapsed());
        record_operation_error(&self.name, error);
    }
}

/// Increment operation count
pub fn increment_operation_count(name: &str) {
    let key = create_key(&format!("{}_count", name));
    counter!(key).increment(1);
}

/// Increment error count for an operation
pub fn increment_error_count(name: &str) {
    let key = create_key(&format!("{}_error", name));
    counter!(key).increment(1);
}

/// Increment timeout count for an operation
pub fn increment_timeout_count(name: &str) {
    let key = create_key(&format!("{}_timeout", name));
    counter!(key).increment(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RedisCacheError;

    #[test]
    fn test_create_key() {
        let key = create_key("test_metric");
        assert_eq!(key, "navius_test_metric");
    }

    #[test]
    fn test_timed_operation() {
        let timer = TimedOperation::new(names::GET);

        // Test success case
        timer.record_success();

        // Test error case
        let error = RedisCacheError::NoResult;
        timer.record_error(&error);

        // Test result case
        let result: RedisCacheResult<()> = Ok(());
        timer.record(&result);

        let error_result: RedisCacheResult<()> = Err(RedisCacheError::NoResult);
        timer.record(&error_result);
    }
}

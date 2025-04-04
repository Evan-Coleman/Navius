//! Metrics implementation for cache operations
//!
//! This module provides metrics tracking for cache operations when the `metrics` feature is enabled.
//! It uses the `metrics` crate to record and report cache operation statistics like hits, misses,
//! latency, and error rates.

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};
use std::time::{Duration, Instant};

/// Types of cache operations for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOperation {
    /// Get a single value
    Get,
    /// Get multiple values
    GetMany,
    /// Set a single value
    Set,
    /// Set multiple values
    SetMany,
    /// Delete a single value
    Delete,
    /// Delete multiple values
    DeleteMany,
    /// Check if a key exists
    Exists,
    /// Increment a counter
    Increment,
    /// Set expiry for a key
    Expire,
    /// Clear the cache
    Clear,
    /// Health check
    HealthCheck,
}

impl CacheOperation {
    /// Convert the operation to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::GetMany => "get_many",
            Self::Set => "set",
            Self::SetMany => "set_many",
            Self::Delete => "delete",
            Self::DeleteMany => "delete_many",
            Self::Exists => "exists",
            Self::Increment => "increment",
            Self::Expire => "expire",
            Self::Clear => "clear",
            Self::HealthCheck => "health_check",
        }
    }
}

/// Result of a cache operation for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheResult {
    /// Operation succeeded
    Success,
    /// Operation failed
    Error,
    /// Cache miss (for get operations)
    Miss,
    /// Cache hit (for get operations)
    Hit,
}

impl CacheResult {
    /// Convert the result to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Miss => "miss",
            Self::Hit => "hit",
        }
    }
}

/// Record cache metrics
///
/// When the metrics feature is enabled, this records metrics for cache operations.
/// When disabled, this is a no-op.
#[cfg(feature = "metrics")]
pub fn record_cache_metric(
    operation: CacheOperation,
    backend: &str,
    result: CacheResult,
    duration: Duration,
) {
    // Increment operation counter
    counter!(
        "navius_cache_operations_total",
        "operation" => operation.as_str().to_string(),
        "backend" => backend.to_string(),
        "result" => result.as_str().to_string()
    )
    .increment(1);

    // Record operation duration
    histogram!(
        "navius_cache_operation_duration_seconds",
        "operation" => operation.as_str().to_string(),
        "backend" => backend.to_string()
    )
    .record(duration.as_secs_f64());
}

/// No-op implementation when metrics are disabled
#[cfg(not(feature = "metrics"))]
pub fn record_cache_metric(
    _operation: CacheOperation,
    _backend: &str,
    _result: CacheResult,
    _duration: Duration,
) {
    // No-op when metrics are disabled
}

/// Timer for measuring cache operation duration
#[derive(Debug)]
pub struct CacheTimer {
    /// Start time of the operation
    start: Instant,
    /// Operation being timed
    operation: CacheOperation,
    /// Backend being used
    backend: String,
}

impl CacheTimer {
    /// Create a new timer for a cache operation
    pub fn new(operation: CacheOperation, backend: &str) -> Self {
        Self {
            start: Instant::now(),
            operation,
            backend: backend.to_string(),
        }
    }

    /// Record a successful operation
    pub fn success(self) {
        let duration = self.start.elapsed();
        record_cache_metric(
            self.operation,
            &self.backend,
            CacheResult::Success,
            duration,
        );
    }

    /// Record a failed operation
    pub fn error(self) {
        let duration = self.start.elapsed();
        record_cache_metric(self.operation, &self.backend, CacheResult::Error, duration);
    }

    /// Record a cache hit
    pub fn hit(self) {
        let duration = self.start.elapsed();
        record_cache_metric(self.operation, &self.backend, CacheResult::Hit, duration);
    }

    /// Record a cache miss
    pub fn miss(self) {
        let duration = self.start.elapsed();
        record_cache_metric(self.operation, &self.backend, CacheResult::Miss, duration);
    }
}

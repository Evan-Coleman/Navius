//! User-defined concurrency limiting components
//!
//! This module allows you to define custom concurrency limits
//! for your application. The core implementation is provided by
//! crate::core::reliability::concurrency.

use crate::core::reliability::concurrency::*;

// Example of custom concurrency limiters:
//
// /// Concurrency limiter for database operations
// pub fn database_concurrency_limiter<S>(service: S) -> ConcurrencyLimitService<S> {
//     ConcurrencyLimitLayer::new(20)  // Max 20 concurrent requests
//         .layer(service)
// }
//
// /// Concurrency limiter for file operations
// pub fn file_operations_concurrency_limiter<S>(service: S) -> ConcurrencyLimitService<S> {
//     ConcurrencyLimitLayer::new(5)   // Max 5 concurrent requests
//         .layer(service)
// }
//
// /// Concurrency limiter for API endpoints
// pub fn api_concurrency_limiter<S>(service: S) -> ConcurrencyLimitService<S> {
//     ConcurrencyLimitLayer::new(50)  // Max 50 concurrent requests
//         .layer(service)
// }

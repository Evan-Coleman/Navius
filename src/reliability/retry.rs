//! User-defined retry components
//!
//! This module allows you to define custom retry strategies
//! for your application. The core implementation is provided by
//! crate::core::reliability::retry.

use crate::core::reliability::retry::*;
use std::time::Duration;

// Example of a custom retry strategy for the database service:
//
// /// Retry strategy for database operations
// pub fn database_retry<S>(service: S) -> RetryService<S> {
//     RetryLayer::new(
//         3,                           // Maximum 3 retry attempts
//         Duration::from_millis(100),  // Start with 100ms delay
//         Duration::from_secs(1),      // Max 1 second delay
//         true,                        // Use exponential backoff
//         vec![500, 503]               // Only retry on these status codes
//     ).layer(service)
// }
//
// /// Aggressive retry for critical services
// pub fn critical_service_retry<S>(service: S) -> RetryService<S> {
//     RetryLayer::new(
//         5,                           // Maximum 5 retry attempts
//         Duration::from_millis(50),   // Start with 50ms delay
//         Duration::from_millis(500),  // Max 500ms delay
//         true,                        // Use exponential backoff
//         vec![500, 502, 503, 504]     // Retry on all server errors
//     ).layer(service)
// }

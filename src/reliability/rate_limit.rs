//! User-defined rate limiting components
//!
//! This module allows you to define custom rate limiting strategies
//! for your application. The core implementation is provided by
//! crate::core::reliability::rate_limit.

use crate::core::reliability::rate_limit::*;
use std::time::Duration;

// Example of custom rate limiting configurations:
//
// /// Rate limiter for authentication endpoints
// pub fn auth_rate_limiter<S>(service: S) -> RateLimitService<S> {
//     RateLimitLayer::new(
//         20,                         // 20 requests allowed
//         Duration::from_secs(60),    // Per 60 second window
//         true                        // Per-client (IP-based) limiting
//     ).layer(service)
// }
//
// /// Rate limiter for public API endpoints
// pub fn public_api_rate_limiter<S>(service: S) -> RateLimitService<S> {
//     RateLimitLayer::new(
//         100,                        // 100 requests allowed
//         Duration::from_secs(60),    // Per 60 second window
//         true                        // Per-client (IP-based) limiting
//     ).layer(service)
// }
//
// /// Global rate limiter for the entire service
// pub fn global_rate_limiter<S>(service: S) -> RateLimitService<S> {
//     RateLimitLayer::new(
//         1000,                       // 1000 requests allowed
//         Duration::from_secs(60),    // Per 60 second window
//         false                       // Global limiting (not per-client)
//     ).layer(service)
// }

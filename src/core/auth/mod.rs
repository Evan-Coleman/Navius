//! Authentication core functionality
//!
//! This module provides authentication and authorization functionality:
//! - Middleware for validating incoming bearer tokens (protect our API)
//! - Client for acquiring tokens for downstream API calls

pub mod client;
pub mod middleware;

pub use client::EntraTokenClient;
pub use middleware::EntraAuthLayer;

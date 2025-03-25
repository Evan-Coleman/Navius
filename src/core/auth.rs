//! Authentication core functionality
//!
//! This module provides authentication and authorization functionality:
//! - Middleware for validating incoming bearer tokens (protect our API)
//! - Client for acquiring tokens for downstream API calls

pub mod error;
pub mod middleware;
pub mod providers;

pub use self::{claims::StandardClaims, error::AuthError};

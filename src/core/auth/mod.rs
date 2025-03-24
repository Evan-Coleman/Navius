//! Authentication core functionality
//!
//! This module provides authentication and authorization functionality:
//! - Middleware for validating incoming bearer tokens (protect our API)
//! - Client for acquiring tokens for downstream API calls

pub mod client;
pub mod middleware;
pub mod mock;
pub mod models;

pub use client::EntraTokenClient;
pub use middleware::{
    Permission, PermissionRequirement, Role, RoleRequirement, auth_middleware, require_auth,
    require_roles, role_from_string,
};
pub use mock::MockTokenClient;

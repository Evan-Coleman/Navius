// User-extensible health check handlers
// This file allows you to add custom health checks while using the core functionality

// Re-export all core health check functionality
pub use crate::core::handlers::health::*;

// Add your custom health check endpoints below
// Example:
// pub async fn custom_health_check() -> impl axum::response::IntoResponse {
//     // Your implementation here
// }

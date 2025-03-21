// User-extensible actuator handlers
// This file allows you to add custom actuator endpoints while using the core functionality

// Re-export all core actuator functionality
pub use crate::core::handlers::actuator::*;

// Add your custom actuator endpoints below
// Example:
// pub async fn custom_actuator() -> impl axum::response::IntoResponse {
//     // Your implementation here
// }

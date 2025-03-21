// User-extensible logging handlers
// This file allows you to add custom logging functionality while using the core implementation

// Re-export all core logging functionality
pub use crate::core::handlers::logging::*;

// Add your custom logging functions below
// Example:
// pub async fn custom_logging_middleware<B>(
//     req: Request<B>,
//     next: Next<B>,
// ) -> impl IntoResponse {
//     // Your implementation here
// }

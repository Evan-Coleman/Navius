// User-extensible documentation handlers
// This file allows you to add custom documentation endpoints while using the core functionality

// Re-export all core documentation functionality
pub use crate::core::handlers::docs::*;

// Add your custom documentation endpoints below
// Example:
// pub async fn custom_docs_handler() -> impl axum::response::IntoResponse {
//     // Your implementation here
// }

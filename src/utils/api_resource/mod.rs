//! # API Resource Abstraction
//!
//! This module provides a user-friendly interface for working with API resources.
//! The core implementation is in the core/utils/api_resource module.

#[cfg(test)]
mod tests;

// Re-export core functionality
pub use crate::core::utils::api_resource::*;

// Add your custom API resource utilities below
//
// Example:
//
// /// Create a custom health check for API resources
// pub fn create_custom_health_check<T: ApiResource>(
//     api_url: String
// ) -> impl Fn(&Arc<AppState>) -> futures::future::BoxFuture<'static, DependencyStatus> + Send + Sync + 'static {
//     move |state: &Arc<AppState>| {
//         let api_url = api_url.clone();
//         Box::pin(async move {
//             // Custom health check implementation
//             DependencyStatus {
//                 name: format!("{} API", T::api_name()),
//                 status: "healthy".to_string(),
//                 details: Some(serde_json::json!({ "url": api_url })),
//             }
//         })
//     }
// }

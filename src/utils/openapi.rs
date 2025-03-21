//! # OpenAPI Utilities
//!
//! This module provides user-extensible utilities for working with OpenAPI.
//! The core implementation is in the core/utils/openapi module.

// Re-export core functionality
pub use crate::core::utils::openapi::*;

// Add your custom OpenAPI utilities below
//
// Example:
//
// use axum::{
//     extract::{Path, State},
//     http::{StatusCode, header},
//     response::IntoResponse,
// };
// use std::sync::Arc;
// use std::path::PathBuf;
//
// use crate::core::router::AppState;
//
// /// Serve a custom OpenAPI document
// pub async fn serve_custom_openapi_doc(
//     State(state): State<Arc<AppState>>,
//     Path(doc_name): Path<String>
// ) -> impl IntoResponse {
//     let base_path = state.config.openapi_spec_path();
//     let parent_dir = PathBuf::from(&base_path).parent().unwrap_or(PathBuf::from("").as_path());
//     let custom_path = parent_dir.join(format!("{}.yaml", doc_name));
//
//     match std::fs::read_to_string(custom_path) {
//         Ok(content) => {
//             (
//                 StatusCode::OK,
//                 [(header::CONTENT_TYPE, "text/yaml")],
//                 content,
//             )
//         }
//         Err(_) => {
//             (
//                 StatusCode::NOT_FOUND,
//                 [(header::CONTENT_TYPE, "text/plain")],
//                 format!("Custom OpenAPI doc '{}' not found", doc_name),
//             )
//         }
//     }
// }

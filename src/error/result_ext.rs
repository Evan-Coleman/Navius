//! # Result Extensions
//!
//! This module allows users to extend the core Result extension functionality.

// Re-export everything from core
pub use crate::core::error::result_ext::*;

// Add custom result extensions below:

// Example:
// ```
// pub trait CustomResultExt<T, E> {
//     fn with_custom_context(self, context: impl Fn() -> String) -> Result<T>;
// }
//
// impl<T, E> CustomResultExt<T, E> for std::result::Result<T, E>
// where
//     E: std::fmt::Debug,
// {
//     fn with_custom_context(self, context: impl Fn() -> String) -> Result<T> {
//         self.map_err(|e| {
//             let ctx = context();
//             AppError::internal(format!("{}: {:?}", ctx, e))
//         })
//     }
// }
// ```

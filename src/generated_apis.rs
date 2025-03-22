//! Generated API modules
//!
//! This file serves as a bridge to the generated API code in the /generated directory.
//! It uses the #[path] attribute to reference files outside of the src directory.

#[path = "../generated/petstore_api/src/lib.rs"]
pub mod petstore_api;

// Re-export from petstore_api
pub use petstore_api::Upet;

//! Generated API modules
//!
//! This file serves as a bridge to the generated API code in the target/generated directory.

#[path = "../target/generated/petstore_api/src/lib.rs"]
pub mod petstore_api;

// Re-export from petstore_api
pub use petstore_api::Upet;


//! Navius Job Processing System
//!
//! This crate provides job processing and background task execution for the Navius framework.
//!
//! ## Main Components
//!
//! - `JobProvider`: Core trait for job processing systems
//! - `Job`: Generic job definition with typed payloads
//! - `JobEnvelope`: Transport representation of a job
//!
//! ## Example Usage:
//!
//! ```rust
//! use navius_job::{create_memory_provider, Job, JobProvider, JobProviderExt};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create an in-memory job provider
//!     let provider = create_memory_provider();
//!     
//!     // Start the provider
//!     provider.start().await.unwrap();
//!     
//!     // Create and schedule a job
//!     let job = Job::new("email", "default", "system", "Hello, World!");
//!     let job_id = provider.schedule(job).await.unwrap();
//!     
//!     println!("Scheduled job: {}", job_id);
//! }
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Job error types
pub mod error;

/// Core job structures
pub mod job;

/// Job provider interfaces
pub mod provider;

/// In-memory job queue implementation
pub mod memory;

// Re-export primary types from job
pub use job::Job;

// Re-export primary types from error
pub use error::{JobError, JobResult, JobStatus};

// Re-export primary types from provider
pub use provider::{JobProvider, JobProviderConfig, JobProviderExt, JobProviderFactory};

// Re-export memory implementation
pub use memory::InMemoryJobProvider;

/// Create a new in-memory job provider
pub fn create_memory_provider() -> InMemoryJobProvider {
    InMemoryJobProvider::new(JobProviderConfig::default())
}

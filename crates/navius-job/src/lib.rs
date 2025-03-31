// Navius Job Processing System
//
// This crate provides job processing and background task execution for the Navius framework.

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
pub use job::{Job, JobContext, JobId, JobOptions, JobResult, JobStatus};

// Re-export primary types from error
pub use error::{JobError, JobErrorCode};

// Re-export primary types from provider
pub use provider::{JobProvider, JobProviderFactory, JobQueue, JobWorker};

// Re-export memory implementation
pub use memory::{InMemoryJobProvider, InMemoryJobQueue, InMemoryWorker};

/// Create a new in-memory job provider
pub fn create_memory_provider() -> Box<dyn JobProvider> {
    Box::new(InMemoryJobProvider::new())
}

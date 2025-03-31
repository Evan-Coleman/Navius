use std::error::Error;
use std::fmt;
use std::io;

/// Result type for job operations
pub type JobResult<T> = Result<T, JobError>;

/// Represents errors that can occur in the job system
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    /// Error scheduling a job
    #[error("Failed to schedule job: {0}")]
    ScheduleError(String),

    /// Error executing a job
    #[error("Failed to execute job: {0}")]
    ExecutionError(String),

    /// Error cancelling a job
    #[error("Failed to cancel job: {0}")]
    CancellationError(String),

    /// Queue not found
    #[error("Queue '{0}' does not exist")]
    QueueNotFound(String),

    /// Invalid queue name
    #[error("Invalid queue name: {0}")]
    InvalidQueue(String),

    /// Job not found
    #[error("Job '{0}' not found")]
    JobNotFound(String),

    /// Error serializing or deserializing job payload
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error delivering a job
    #[error("Job delivery error: {0}")]
    DeliveryError(String),

    /// Maximum jobs reached for queue
    #[error("Maximum jobs reached for queue '{0}'")]
    MaxJobsReached(String),

    /// Job backpressure error (too many pending jobs)
    #[error("Job backpressure error: {0}")]
    BackpressureError(String),

    /// Timeout waiting for job completion
    #[error("Timeout waiting for job completion")]
    TimeoutError,

    /// Error parsing a cron expression
    #[error("Error parsing cron expression: {0}")]
    CronParseError(String),

    /// Job provider error
    #[error("Job provider error: {0}")]
    ProviderError(String),

    /// Worker error
    #[error("Worker error: {0}")]
    WorkerError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Error from event system
    #[error("Event system error: {0}")]
    EventError(#[from] navius_event::EventError),

    /// Other error
    #[error("Job error: {0}")]
    Other(String),
}

/// Job priority levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum JobPriority {
    /// Low priority jobs
    Low = 0,

    /// Normal priority jobs (default)
    Normal = 1,

    /// High priority jobs
    High = 2,

    /// Critical priority jobs (processed before all others)
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JobStatus {
    /// Job is pending execution
    Pending,

    /// Job is currently executing
    Running,

    /// Job has completed successfully
    Completed,

    /// Job has failed
    Failed,

    /// Job has been cancelled
    Cancelled,

    /// Job has been scheduled for future execution
    Scheduled,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Cancelled => write!(f, "Cancelled"),
            JobStatus::Scheduled => write!(f, "Scheduled"),
        }
    }
}

/// Job execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JobExecutionResult {
    /// Job completed successfully
    Success(serde_json::Value),

    /// Job failed with an error
    Failure {
        /// Error message
        message: String,
        /// Optional error details
        details: Option<serde_json::Value>,
    },
}

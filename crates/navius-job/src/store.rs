use crate::error::{JobResult, JobStatus};
use crate::job::Job;
use tokio::sync::broadcast;

/// Type alias for job ID
pub type JobId = String;

/// Configuration for job stores
#[derive(Debug, Clone)]
pub struct JobStoreConfig {
    /// Maximum number of jobs to store per queue
    pub max_jobs_per_queue: Option<usize>,

    /// Maximum retention period for completed jobs in seconds
    pub retention_period_seconds: u64,

    /// Whether to validate job queues against known queues
    pub validate_queues: bool,

    /// Known valid queue names if validation is enabled
    pub known_queues: Vec<String>,
}

impl Default for JobStoreConfig {
    fn default() -> Self {
        Self {
            max_jobs_per_queue: None,
            retention_period_seconds: 86400, // 24 hours
            validate_queues: false,
            known_queues: Vec::new(),
        }
    }
}

/// Job store interface for storing and retrieving jobs
#[::async_trait::async_trait]
pub trait JobStore: Send + Sync {
    /// Create a new job in the store
    async fn create(&self, job: Job<serde_json::Value>) -> JobResult<JobId>;

    /// Get a job by ID
    async fn get(&self, id: &str) -> JobResult<Option<Job<serde_json::Value>>>;

    /// Update an existing job
    async fn update(&self, id: &str, job: Job<serde_json::Value>) -> JobResult<()>;

    /// Delete a job from the store
    async fn delete(&self, id: &str) -> JobResult<()>;

    /// List all jobs in the store
    async fn list(&self) -> JobResult<Vec<Job<serde_json::Value>>>;

    /// Subscribe to job status updates
    async fn subscribe(&self) -> broadcast::Receiver<(JobId, JobStatus)>;

    /// Publish a job status update
    async fn publish_status(&self, id: &str, status: JobStatus) -> JobResult<()>;
}

use crate::error::{JobExecutionResult, JobResult};
use crate::job::{Job, JobEnvelope, JobFilterConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// Configuration for a job provider
#[derive(Debug, Clone)]
pub struct JobProviderConfig {
    /// Maximum number of queues
    pub max_queues: Option<usize>,

    /// Maximum jobs per queue
    pub max_jobs_per_queue: Option<usize>,

    /// Maximum job retention period in seconds (0 = no retention)
    pub job_retention_seconds: u64,

    /// Maximum jobs to retain per queue (0 = unlimited)
    pub max_retained_jobs_per_queue: usize,

    /// Whether to validate job queues against a known list
    pub validate_queues: bool,

    /// Known queues (if validation is enabled)
    pub known_queues: Vec<String>,

    /// Queue namespace prefix
    pub queue_namespace: Option<String>,

    /// Default maximum retries for jobs
    pub default_max_retries: u32,

    /// Default job timeout in seconds
    pub default_timeout_seconds: Option<u64>,

    /// Default backoff strategy for retries (in seconds)
    pub default_retry_backoff: Option<Vec<u32>>,

    /// Number of worker threads (0 = number of CPUs)
    pub worker_threads: usize,

    /// Whether to publish job events to the event system
    pub publish_events: bool,

    /// Event topic for job events
    pub event_topic: Option<String>,
}

impl JobProviderConfig {
    /// Create a new job provider configuration with default values
    pub fn new() -> Self {
        Self {
            max_queues: None,
            max_jobs_per_queue: None,
            job_retention_seconds: 86400, // 24 hours by default
            max_retained_jobs_per_queue: 10000,
            validate_queues: false,
            known_queues: Vec::new(),
            queue_namespace: None,
            default_max_retries: 3,
            default_timeout_seconds: Some(300), // 5 minutes
            default_retry_backoff: Some(vec![1, 5, 30, 60, 300]), // 1s, 5s, 30s, 1m, 5m
            worker_threads: 0,
            publish_events: true,
            event_topic: Some("jobs".to_string()),
        }
    }

    /// Set the maximum number of queues
    pub fn with_max_queues(mut self, max_queues: usize) -> Self {
        self.max_queues = Some(max_queues);
        self
    }

    /// Set the maximum jobs per queue
    pub fn with_max_jobs_per_queue(mut self, max_jobs: usize) -> Self {
        self.max_jobs_per_queue = Some(max_jobs);
        self
    }

    /// Set the job retention period in seconds
    pub fn with_job_retention_seconds(mut self, seconds: u64) -> Self {
        self.job_retention_seconds = seconds;
        self
    }

    /// Set the maximum retained jobs per queue
    pub fn with_max_retained_jobs(mut self, max_jobs: usize) -> Self {
        self.max_retained_jobs_per_queue = max_jobs;
        self
    }

    /// Enable queue validation and set known queues
    pub fn with_queue_validation(mut self, queues: Vec<impl Into<String>>) -> Self {
        self.validate_queues = true;
        self.known_queues = queues.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Set the queue namespace prefix
    pub fn with_queue_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.queue_namespace = Some(namespace.into());
        self
    }

    /// Set the default maximum retries for jobs
    pub fn with_default_max_retries(mut self, max_retries: u32) -> Self {
        self.default_max_retries = max_retries;
        self
    }

    /// Set the default job timeout in seconds
    pub fn with_default_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.default_timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Set the default backoff strategy for retries
    pub fn with_default_retry_backoff(mut self, backoff_seconds: Vec<u32>) -> Self {
        self.default_retry_backoff = Some(backoff_seconds);
        self
    }

    /// Set the number of worker threads
    pub fn with_worker_threads(mut self, worker_threads: usize) -> Self {
        self.worker_threads = worker_threads;
        self
    }

    /// Set whether to publish job events to the event system
    pub fn with_publish_events(mut self, publish_events: bool) -> Self {
        self.publish_events = publish_events;
        self
    }

    /// Set the event topic for job events
    pub fn with_event_topic(mut self, event_topic: impl Into<String>) -> Self {
        self.event_topic = Some(event_topic.into());
        self
    }
}

impl Default for JobProviderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about the job provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Unique identifier for the provider
    pub id: String,

    /// Name of the provider
    pub name: String,

    /// Number of active queues
    pub queue_count: usize,

    /// Number of active jobs
    pub job_count: usize,

    /// Number of worker threads
    pub worker_count: usize,

    /// Implementation type
    pub implementation_type: String,
}

/// Information about a job queue
#[derive(Debug, Clone)]
pub struct QueueInfo {
    /// Name of the queue
    pub name: String,

    /// Number of jobs in the queue
    pub job_count: usize,

    /// Number of running jobs
    pub running_job_count: usize,

    /// Number of scheduled jobs
    pub scheduled_job_count: usize,

    /// Number of completed jobs
    pub completed_job_count: usize,

    /// Number of failed jobs
    pub failed_job_count: usize,

    /// Number of recurring jobs
    pub recurring_job_count: usize,
}

/// Job handler function trait
pub trait JobHandlerFn: Send + Sync + 'static {
    /// Execute the handler
    fn execute(&self, envelope: JobEnvelope) -> JobResult<JobExecutionResult>;
}

impl<F> JobHandlerFn for F
where
    F: Fn(JobEnvelope) -> JobResult<JobExecutionResult> + Send + Sync + 'static,
{
    fn execute(&self, envelope: JobEnvelope) -> JobResult<JobExecutionResult> {
        self(envelope)
    }
}

/// Job handler function
pub type JobHandler<T> =
    Box<dyn Fn(Job<T>) -> JobResult<JobExecutionResult> + Send + Sync + 'static>;

/// Scheduling options for jobs
#[derive(Debug, Clone)]
pub struct SchedulingOptions {
    /// Maximum number of retries
    pub max_retries: Option<u32>,

    /// Retry backoff strategy (in seconds)
    pub retry_backoff: Option<Vec<u32>>,

    /// Job timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Job priority
    pub priority: Option<crate::error::JobPriority>,

    /// Job metadata
    pub metadata: Option<HashMap<String, String>>,

    /// Correlation ID
    pub correlation_id: Option<String>,
}

impl SchedulingOptions {
    /// Create new scheduling options with default values
    pub fn new() -> Self {
        Self {
            max_retries: None,
            retry_backoff: None,
            timeout_seconds: None,
            priority: None,
            metadata: None,
            correlation_id: None,
        }
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the retry backoff strategy
    pub fn with_retry_backoff(mut self, retry_backoff: Vec<u32>) -> Self {
        self.retry_backoff = Some(retry_backoff);
        self
    }

    /// Set the job timeout in seconds
    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Set the job priority
    pub fn with_priority(mut self, priority: crate::error::JobPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Add metadata to the job
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }

        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.into(), value.into());
        }

        self
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

impl Default for SchedulingOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Core trait defining the job provider interface
#[async_trait]
pub trait JobProvider: Send + Sync {
    /// Get provider information
    async fn get_info(&self) -> JobResult<ProviderInfo>;

    /// Get information about all queues
    async fn list_queues(&self) -> JobResult<Vec<QueueInfo>>;

    /// Check if a queue exists
    async fn queue_exists(&self, queue: &str) -> JobResult<bool>;

    /// Create a new queue
    async fn create_queue(&self, queue: &str) -> JobResult<()>;

    /// Delete a queue and all its jobs
    async fn delete_queue(&self, queue: &str) -> JobResult<()>;

    /// Start the job provider
    async fn start(&self) -> JobResult<()>;

    /// Stop the job provider
    async fn stop(&self) -> JobResult<()>;

    /// Pause job processing
    async fn pause(&self) -> JobResult<()>;

    /// Resume job processing
    async fn resume(&self) -> JobResult<()>;

    /// Get the status of the job provider
    async fn is_running(&self) -> JobResult<bool>;

    /// Schedule a job (type-erased version)
    async fn schedule_raw(&self, job: JobEnvelope) -> JobResult<Uuid>;

    /// Cancel a job
    async fn cancel(&self, job_id: &Uuid) -> JobResult<bool>;

    /// Register a job handler by type and serialized handler
    async fn register_handler_raw(
        &self,
        job_type: &str,
        handler: Box<dyn JobHandlerFn>,
    ) -> JobResult<()>;

    /// Unregister a job handler
    async fn unregister_handler(&self, job_type: &str) -> JobResult<bool>;

    /// Get a job by ID
    async fn get_job(&self, job_id: &Uuid) -> JobResult<Option<JobEnvelope>>;

    /// Get jobs that match a filter
    async fn get_jobs(
        &self,
        queue: &str,
        filter: &JobFilterConfig,
        limit: usize,
    ) -> JobResult<Vec<JobEnvelope>>;

    /// Get completed jobs
    async fn get_completed_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>>;

    /// Get failed jobs
    async fn get_failed_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>>;

    /// Get scheduled jobs
    async fn get_scheduled_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>>;

    /// Get recurring jobs
    async fn get_recurring_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>>;

    /// Retry a failed job
    async fn retry_job(&self, job_id: &Uuid) -> JobResult<bool>;

    /// Retry all failed jobs in a queue
    async fn retry_all_failed_jobs(&self, queue: &str) -> JobResult<usize>;

    /// Check provider health
    async fn health_check(&self) -> JobResult<bool>;
}

/// Type-safe extension trait for JobProvider
#[async_trait]
pub trait JobProviderExt: JobProvider {
    /// Schedule a job
    async fn schedule<T>(&self, job: Job<T>) -> JobResult<Uuid>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let envelope = JobEnvelope::from_job(&job)?;
        self.schedule_raw(envelope).await
    }

    /// Schedule a job with options
    async fn schedule_with_options<T>(
        &self,
        job_type: &str,
        queue: &str,
        payload: T,
        options: SchedulingOptions,
    ) -> JobResult<Uuid>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        // Create the job
        let mut job = Job::new(job_type, queue, "job-provider", payload);

        // Apply options
        if let Some(max_retries) = options.max_retries {
            job.max_retries = max_retries;
        }

        if let Some(retry_backoff) = options.retry_backoff {
            job.retry_backoff = Some(retry_backoff);
        }

        if let Some(timeout_seconds) = options.timeout_seconds {
            job.timeout_seconds = Some(timeout_seconds);
        }

        if let Some(priority) = options.priority {
            job.priority = priority;
        }

        if let Some(correlation_id) = options.correlation_id {
            job.correlation_id = Some(correlation_id);
        }

        if let Some(metadata) = options.metadata {
            job.metadata.extend(metadata);
        }

        // Schedule the job
        self.schedule(job).await
    }

    /// Schedule a job to run at a specific time
    async fn schedule_at<T>(
        &self,
        job_type: &str,
        queue: &str,
        payload: T,
        scheduled_time: DateTime<Utc>,
        options: SchedulingOptions,
    ) -> JobResult<Uuid>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        // Create the job
        let mut job = Job::new(job_type, queue, "job-provider", payload);
        job.scheduled_for = Some(scheduled_time);

        // Apply options
        if let Some(max_retries) = options.max_retries {
            job.max_retries = max_retries;
        }

        if let Some(retry_backoff) = options.retry_backoff {
            job.retry_backoff = Some(retry_backoff);
        }

        if let Some(timeout_seconds) = options.timeout_seconds {
            job.timeout_seconds = Some(timeout_seconds);
        }

        if let Some(priority) = options.priority {
            job.priority = priority;
        }

        if let Some(correlation_id) = options.correlation_id {
            job.correlation_id = Some(correlation_id);
        }

        if let Some(metadata) = options.metadata {
            job.metadata.extend(metadata);
        }

        // Schedule the job
        self.schedule(job).await
    }

    /// Schedule a recurring job with a cron expression
    async fn schedule_recurring<T>(
        &self,
        job_type: &str,
        queue: &str,
        payload: T,
        cron_expression: &str,
        options: SchedulingOptions,
    ) -> JobResult<Uuid>
    where
        T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    {
        // Create the job
        let mut job = Job::new(job_type, queue, "job-provider", payload);
        job.cron_expression = Some(cron_expression.to_string());

        // Apply options
        if let Some(max_retries) = options.max_retries {
            job.max_retries = max_retries;
        }

        if let Some(retry_backoff) = options.retry_backoff {
            job.retry_backoff = Some(retry_backoff);
        }

        if let Some(timeout_seconds) = options.timeout_seconds {
            job.timeout_seconds = Some(timeout_seconds);
        }

        if let Some(priority) = options.priority {
            job.priority = priority;
        }

        if let Some(correlation_id) = options.correlation_id {
            job.correlation_id = Some(correlation_id);
        }

        if let Some(metadata) = options.metadata {
            job.metadata.extend(metadata);
        }

        // Schedule the job
        self.schedule(job).await
    }

    /// Register a job handler
    async fn register_handler<T>(
        &self,
        job_type: &str,
        handler: Box<
            dyn Fn(T, JobEnvelope) -> JobResult<JobExecutionResult> + Send + Sync + 'static,
        >,
    ) -> JobResult<()>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        // Create a type-erased handler function
        let handler_fn = Box::new(
            move |envelope: JobEnvelope| -> JobResult<JobExecutionResult> {
                let payload = envelope.payload.clone();
                let payload_json = serde_json::from_value::<T>(payload)?;
                handler(payload_json, envelope)
            },
        );

        self.register_handler_raw(job_type, handler_fn).await
    }
}

// Implement the extension trait for all JobProvider implementors
impl<P: JobProvider> JobProviderExt for P {}

/// Factory for creating job providers
#[async_trait]
pub trait JobProviderFactory: Send + Sync {
    /// Type of job provider created by this factory
    type Provider: JobProvider;

    /// Create a new job provider
    async fn create_provider(&self, config: JobProviderConfig) -> JobResult<Arc<Self::Provider>>;
}

/// Trait for types that can be used as job payloads
pub trait JobPayload: Serialize + DeserializeOwned + Send + Sync + 'static {}
impl<T> JobPayload for T where T: Serialize + DeserializeOwned + Send + Sync + 'static {}

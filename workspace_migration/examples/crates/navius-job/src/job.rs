use crate::error::{JobExecutionResult, JobPriority, JobStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use uuid::Uuid;

/// Represents a job in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job<T> {
    /// Unique identifier for the job
    pub id: Uuid,

    /// Type of the job
    pub job_type: String,

    /// Queue the job belongs to
    pub queue: String,

    /// Time the job was created
    pub created_at: DateTime<Utc>,

    /// Time the job should be executed (None for immediate execution)
    pub scheduled_for: Option<DateTime<Utc>>,

    /// Time when job started execution (if started)
    pub started_at: Option<DateTime<Utc>>,

    /// Time when job completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,

    /// Priority of the job
    pub priority: JobPriority,

    /// Current status of the job
    pub status: JobStatus,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Current retry count
    pub retry_count: u32,

    /// Backoff strategy for retries (in seconds)
    pub retry_backoff: Option<Vec<u32>>,

    /// Source of the job (typically service or component name)
    pub source: String,

    /// Optional correlation ID for tracing related jobs
    pub correlation_id: Option<String>,

    /// Metadata associated with the job
    pub metadata: HashMap<String, String>,

    /// Optional timeout for the job execution (in seconds)
    pub timeout_seconds: Option<u64>,

    /// Optional cron expression for recurring jobs
    pub cron_expression: Option<String>,

    /// Payload of the job
    pub payload: T,

    /// Execution result (if completed or failed)
    #[serde(skip)]
    pub result: Option<JobExecutionResult>,
}

impl<T> Job<T> {
    /// Create a new job
    pub fn new(
        job_type: impl Into<String>,
        queue: impl Into<String>,
        source: impl Into<String>,
        payload: T,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type: job_type.into(),
            queue: queue.into(),
            created_at: Utc::now(),
            scheduled_for: None,
            started_at: None,
            completed_at: None,
            priority: JobPriority::Normal,
            status: JobStatus::Pending,
            max_retries: 0,
            retry_count: 0,
            retry_backoff: None,
            source: source.into(),
            correlation_id: None,
            metadata: HashMap::new(),
            timeout_seconds: None,
            cron_expression: None,
            payload,
            result: None,
        }
    }

    /// Create a new scheduled job
    pub fn scheduled(
        job_type: impl Into<String>,
        queue: impl Into<String>,
        source: impl Into<String>,
        payload: T,
        scheduled_for: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type: job_type.into(),
            queue: queue.into(),
            created_at: Utc::now(),
            scheduled_for: Some(scheduled_for),
            started_at: None,
            completed_at: None,
            priority: JobPriority::Normal,
            status: JobStatus::Scheduled,
            max_retries: 0,
            retry_count: 0,
            retry_backoff: None,
            source: source.into(),
            correlation_id: None,
            metadata: HashMap::new(),
            timeout_seconds: None,
            cron_expression: None,
            payload,
            result: None,
        }
    }

    /// Create a new recurring job with a cron expression
    pub fn recurring(
        job_type: impl Into<String>,
        queue: impl Into<String>,
        source: impl Into<String>,
        payload: T,
        cron_expression: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type: job_type.into(),
            queue: queue.into(),
            created_at: Utc::now(),
            scheduled_for: None,
            started_at: None,
            completed_at: None,
            priority: JobPriority::Normal,
            status: JobStatus::Scheduled,
            max_retries: 0,
            retry_count: 0,
            retry_backoff: None,
            source: source.into(),
            correlation_id: None,
            metadata: HashMap::new(),
            timeout_seconds: None,
            cron_expression: Some(cron_expression.into()),
            payload,
            result: None,
        }
    }

    /// Set the priority for the job
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the correlation ID for the job
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Add metadata to the job
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries to the job
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Set the timeout for the job
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_seconds = Some(timeout.as_secs());
        self
    }

    /// Configure retry behavior for the job
    pub fn with_retries(mut self, max_retries: u32, backoff_seconds: Option<Vec<u32>>) -> Self {
        self.max_retries = max_retries;
        self.retry_backoff = backoff_seconds;
        self
    }

    /// Check if the job is ready to be executed
    pub fn is_ready(&self) -> bool {
        match (self.status, self.scheduled_for) {
            (JobStatus::Pending, _) => true,
            (JobStatus::Scheduled, Some(time)) => time <= Utc::now(),
            _ => false,
        }
    }

    /// Check if the job is a recurring job
    pub fn is_recurring(&self) -> bool {
        self.cron_expression.is_some()
    }
}

/// An envelope containing a job with its payload serialized to JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEnvelope {
    /// Unique identifier for the job
    pub id: Uuid,

    /// Type of the job
    pub job_type: String,

    /// Queue the job belongs to
    pub queue: String,

    /// Time the job was created
    pub created_at: DateTime<Utc>,

    /// Time the job should be executed (None for immediate execution)
    pub scheduled_for: Option<DateTime<Utc>>,

    /// Time when job started execution (if started)
    pub started_at: Option<DateTime<Utc>>,

    /// Time when job completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,

    /// Priority of the job
    pub priority: JobPriority,

    /// Current status of the job
    pub status: JobStatus,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Current retry count
    pub retry_count: u32,

    /// Backoff strategy for retries (in seconds)
    pub retry_backoff: Option<Vec<u32>>,

    /// Source of the job (typically service or component name)
    pub source: String,

    /// Optional correlation ID for tracing related jobs
    pub correlation_id: Option<String>,

    /// Metadata associated with the job
    pub metadata: HashMap<String, String>,

    /// Optional timeout for the job execution (in seconds)
    pub timeout_seconds: Option<u64>,

    /// Optional cron expression for recurring jobs
    pub cron_expression: Option<String>,

    /// JSON serialized payload
    pub payload: serde_json::Value,

    /// Execution result (if completed or failed)
    pub result: Option<JobExecutionResult>,
}

impl JobEnvelope {
    /// Convert a job to an envelope
    pub fn from_job<T>(job: &Job<T>) -> crate::error::JobResult<Self>
    where
        T: Serialize,
    {
        let payload = match serde_json::to_value(&job.payload) {
            Ok(value) => value,
            Err(err) => {
                return Err(crate::error::JobError::SerializationError(format!(
                    "Failed to serialize job payload: {}",
                    err
                )));
            }
        };

        Ok(Self {
            id: job.id,
            job_type: job.job_type.clone(),
            queue: job.queue.clone(),
            created_at: job.created_at,
            scheduled_for: job.scheduled_for,
            started_at: job.started_at,
            completed_at: job.completed_at,
            priority: job.priority,
            status: job.status,
            max_retries: job.max_retries,
            retry_count: job.retry_count,
            retry_backoff: job.retry_backoff.clone(),
            source: job.source.clone(),
            correlation_id: job.correlation_id.clone(),
            metadata: job.metadata.clone(),
            timeout_seconds: job.timeout_seconds,
            cron_expression: job.cron_expression.clone(),
            payload,
            result: job.result.clone(),
        })
    }

    /// Try to deserialize the payload into the specified type
    pub fn try_into_job<T>(&self) -> crate::error::JobResult<Job<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let payload = match serde_json::from_value(self.payload.clone()) {
            Ok(value) => value,
            Err(err) => {
                return Err(crate::error::JobError::SerializationError(format!(
                    "Failed to deserialize job payload: {}",
                    err
                )));
            }
        };

        Ok(Job {
            id: self.id,
            job_type: self.job_type.clone(),
            queue: self.queue.clone(),
            created_at: self.created_at,
            scheduled_for: self.scheduled_for,
            started_at: self.started_at,
            completed_at: self.completed_at,
            priority: self.priority,
            status: self.status,
            max_retries: self.max_retries,
            retry_count: self.retry_count,
            retry_backoff: self.retry_backoff.clone(),
            source: self.source.clone(),
            correlation_id: self.correlation_id.clone(),
            metadata: self.metadata.clone(),
            timeout_seconds: self.timeout_seconds,
            cron_expression: self.cron_expression.clone(),
            payload,
            result: self.result.clone(),
        })
    }
}

/// Job filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFilterConfig {
    /// Filter jobs by type (if specified)
    pub job_types: Option<Vec<String>>,

    /// Filter jobs by queue (if specified)
    pub queues: Option<Vec<String>>,

    /// Filter jobs by source (if specified)
    pub sources: Option<Vec<String>>,

    /// Filter jobs by minimum priority (if specified)
    pub min_priority: Option<JobPriority>,

    /// Filter jobs by status (if specified)
    pub statuses: Option<Vec<JobStatus>>,

    /// Filter jobs by correlation ID (if specified)
    pub correlation_id: Option<String>,

    /// Filter jobs by metadata key-value pairs (if specified)
    pub metadata: Option<HashMap<String, String>>,

    /// Filter jobs by whether they are recurring (if specified)
    pub is_recurring: Option<bool>,
}

impl JobFilterConfig {
    /// Create a new empty job filter configuration
    pub fn new() -> Self {
        Self {
            job_types: None,
            queues: None,
            sources: None,
            min_priority: None,
            statuses: None,
            correlation_id: None,
            metadata: None,
            is_recurring: None,
        }
    }

    /// Filter jobs by type
    pub fn with_job_types(mut self, job_types: Vec<impl Into<String>>) -> Self {
        self.job_types = Some(job_types.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Filter jobs by queue
    pub fn with_queues(mut self, queues: Vec<impl Into<String>>) -> Self {
        self.queues = Some(queues.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Filter jobs by source
    pub fn with_sources(mut self, sources: Vec<impl Into<String>>) -> Self {
        self.sources = Some(sources.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Filter jobs by minimum priority
    pub fn with_min_priority(mut self, min_priority: JobPriority) -> Self {
        self.min_priority = Some(min_priority);
        self
    }

    /// Filter jobs by status
    pub fn with_statuses(mut self, statuses: Vec<JobStatus>) -> Self {
        self.statuses = Some(statuses);
        self
    }

    /// Filter jobs by correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Filter jobs by metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }

        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.into(), value.into());
        }

        self
    }

    /// Filter jobs by whether they are recurring
    pub fn with_recurring(mut self, is_recurring: bool) -> Self {
        self.is_recurring = Some(is_recurring);
        self
    }

    /// Apply the filter to a job envelope
    pub fn matches(&self, job: &JobEnvelope) -> bool {
        // Check job type filter
        if let Some(job_types) = &self.job_types {
            if !job_types.contains(&job.job_type) {
                return false;
            }
        }

        // Check queue filter
        if let Some(queues) = &self.queues {
            if !queues.contains(&job.queue) {
                return false;
            }
        }

        // Check source filter
        if let Some(sources) = &self.sources {
            if !sources.contains(&job.source) {
                return false;
            }
        }

        // Check priority filter
        if let Some(min_priority) = &self.min_priority {
            if job.priority < *min_priority {
                return false;
            }
        }

        // Check status filter
        if let Some(statuses) = &self.statuses {
            if !statuses.contains(&job.status) {
                return false;
            }
        }

        // Check correlation ID filter
        if let Some(correlation_id) = &self.correlation_id {
            if let Some(job_correlation_id) = &job.correlation_id {
                if job_correlation_id != correlation_id {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check metadata filter
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                if !job.metadata.contains_key(key) || job.metadata.get(key).unwrap() != value {
                    return false;
                }
            }
        }

        // Check recurring filter
        if let Some(is_recurring) = self.is_recurring {
            if is_recurring != job.cron_expression.is_some() {
                return false;
            }
        }

        true
    }
}

impl Default for JobFilterConfig {
    fn default() -> Self {
        Self::new()
    }
}

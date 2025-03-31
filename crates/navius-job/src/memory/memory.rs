use crate::error::{JobError, JobExecutionResult, JobPriority, JobResult, JobStatus};
use crate::job::{Job, JobEnvelope, JobFilterConfig};
use crate::provider::{
    JobHandlerFn, JobProvider, JobProviderConfig, JobProviderFactory, ProviderInfo, QueueInfo,
    SchedulingOptions,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use navius_event::broker::EventBroker;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time;
use uuid::Uuid;

mod queue;
mod worker;

use queue::InMemoryQueue;
use worker::{JobCompletionNotification, JobExecution, Worker, WorkerCommand};

/// In-memory job provider implementation
pub struct InMemoryJobProvider {
    /// Provider ID
    id: String,

    /// Provider name
    name: String,

    /// Provider configuration
    config: JobProviderConfig,

    /// Queues managed by this provider
    queues: Arc<RwLock<HashMap<String, Arc<InMemoryQueue>>>>,

    /// Running workers
    workers: Arc<Mutex<HashMap<String, Worker>>>,

    /// Type handlers by job type
    handlers: Arc<RwLock<HashMap<String, Box<dyn JobHandlerFn>>>>,

    /// Is the provider running
    running: Arc<RwLock<bool>>,

    /// Worker join handles
    worker_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,

    /// Event publisher for job events (if enabled)
    event_publisher: Option<Arc<navius_event::InMemoryEventBroker>>,
}

impl InMemoryJobProvider {
    /// Create a new in-memory job provider
    pub fn new(config: JobProviderConfig) -> Self {
        let id = Uuid::new_v4().to_string();
        let name = format!("in-memory-job-provider-{}", id);

        let provider = Self {
            id,
            name,
            config,
            queues: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            worker_handles: Arc::new(Mutex::new(Vec::new())),
            event_publisher: None,
        };

        provider
    }

    /// Create a new in-memory job provider with event publisher
    pub async fn with_event_publisher(
        config: JobProviderConfig,
        event_broker: Arc<navius_event::InMemoryEventBroker>,
    ) -> JobResult<Self> {
        let mut provider = Self::new(config);
        provider.event_publisher = Some(event_broker);

        // Create event topic if it doesn't exist
        if let Some(event_broker) = &provider.event_publisher {
            if let Some(topic) = &provider.config.event_topic {
                if !event_broker.topic_exists(topic).await? {
                    event_broker.create_topic(topic).await?;
                }
            }
        }

        Ok(provider)
    }

    /// Start the cleanup task for job retention
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let queues = self.queues.clone();
        let retention_seconds = self.config.job_retention_seconds;
        let max_retained_jobs = self.config.max_retained_jobs_per_queue;

        tokio::spawn(async move {
            let cleanup_interval = time::Duration::from_secs(300); // 5 minutes

            loop {
                time::sleep(cleanup_interval).await;

                // Skip cleanup if retention is disabled
                if retention_seconds == 0 && max_retained_jobs == 0 {
                    continue;
                }

                let now = Utc::now();
                let retention_cutoff = now - chrono::Duration::seconds(retention_seconds as i64);

                let queue_map = queues.read().await;
                for queue in queue_map.values() {
                    // Remove old completed/failed jobs based on retention period
                    if retention_seconds > 0 {
                        queue.cleanup_old_jobs(retention_cutoff).await;
                    }

                    // Enforce maximum jobs per queue
                    if max_retained_jobs > 0 {
                        queue.enforce_max_jobs(max_retained_jobs).await;
                    }
                }
            }
        })
    }

    /// Publish a job event
    async fn publish_event(&self, event_type: &str, job: &JobEnvelope) -> JobResult<()> {
        if !self.config.publish_events {
            return Ok(());
        }

        if let Some(event_broker) = &self.event_publisher {
            if let Some(topic) = &self.config.event_topic {
                let event = navius_event::Event::new(
                    event_type,
                    topic,
                    "job-provider",
                    serde_json::to_value(job)?,
                )
                .with_correlation_id(job.id.to_string());

                let _ = event_broker
                    .publish_json(
                        event_type,
                        topic,
                        "job-provider",
                        serde_json::to_value(job)?,
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Handle job completion notification
    async fn handle_job_completion(
        &self,
        notification: JobCompletionNotification,
    ) -> JobResult<()> {
        let queue_name = notification.queue.clone();
        let job_id = notification.job_id;

        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(&queue_name)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue_name)))?;

        // Update job status based on result
        match notification.result {
            JobExecution::Success(result) => {
                // Mark job as completed
                queue.mark_job_completed(&job_id, result).await?;

                // Get the updated job and publish event
                if let Some(job) = queue.get_job(&job_id).await? {
                    self.publish_event("job.completed", &job).await?;
                }
            }
            JobExecution::Failure { message, details } => {
                // Mark job as failed
                queue.mark_job_failed(&job_id, message, details).await?;

                // Get the updated job and publish event
                if let Some(job) = queue.get_job(&job_id).await? {
                    self.publish_event("job.failed", &job).await?;
                }
            }
        }

        Ok(())
    }

    /// Process scheduled and recurring jobs
    async fn process_scheduled_jobs(&self) -> JobResult<()> {
        let queue_names = {
            let queues = self.queues.read().await;
            queues.keys().cloned().collect::<Vec<_>>()
        };

        // Process each queue
        for queue_name in queue_names {
            let queue = {
                let queues = self.queues.read().await;
                queues.get(&queue_name).cloned()
            };

            if let Some(queue) = queue {
                // Process due jobs
                let due_jobs = queue.get_due_jobs().await?;
                for job in due_jobs {
                    // Send job to a worker
                    self.process_job(job).await?;
                }

                // Process recurring jobs
                // TODO: Implement recurring job processing
            }
        }

        Ok(())
    }

    /// Process a job by finding a worker
    async fn process_job(&self, job: JobEnvelope) -> JobResult<()> {
        // Find a worker to handle the job
        let workers = self.workers.lock().await;
        if workers.is_empty() {
            return Err(JobError::NoAvailableWorkers(
                "No workers available".to_string(),
            ));
        }

        // Find a worker (simple round-robin for now)
        let worker_id = workers.keys().next().unwrap().clone();
        let worker = workers.get(&worker_id).unwrap();

        // Send job to worker for processing
        if let Some(tx) = &worker.command_tx {
            tx.send(WorkerCommand::ProcessJob(job)).await.map_err(|e| {
                JobError::WorkerError(format!("Failed to send job to worker: {}", e))
            })?;
        } else {
            return Err(JobError::WorkerError(
                "Worker command channel not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Start worker threads
    async fn start_workers(&self) -> JobResult<()> {
        let worker_count = if self.config.worker_threads == 0 {
            // Default to number of CPUs
            num_cpus::get()
        } else {
            self.config.worker_threads
        };

        let (completion_tx, mut completion_rx) = tokio::sync::mpsc::channel(100);

        // Start each worker
        for i in 0..worker_count {
            let worker_id = format!("worker-{}", i);
            let worker_name = format!("worker-{}", i);

            let mut worker = Worker::new(worker_id.clone(), worker_name);
            worker.start(self.handlers.clone(), completion_tx.clone())?;

            let mut workers = self.workers.lock().await;
            workers.insert(worker_id, worker);
        }

        // Start job completion handler
        let provider = Arc::new(self.clone());
        let handle = tokio::spawn(async move {
            while let Some(notification) = completion_rx.recv().await {
                if let Err(e) = provider.handle_job_completion(notification).await {
                    tracing::error!(error = %e, "Failed to handle job completion");
                }
            }
        });

        let mut handles = self.worker_handles.lock().await;
        handles.push(handle);

        Ok(())
    }

    /// Start the scheduled job processor
    fn start_scheduler(&self) -> tokio::task::JoinHandle<()> {
        let provider = Arc::new(self.clone());

        tokio::spawn(async move {
            let scheduler_interval = Duration::from_secs(1);

            loop {
                time::sleep(scheduler_interval).await;

                // Check if provider is still running
                let running = *provider.running.read().await;
                if !running {
                    break;
                }

                // Process scheduled jobs
                if let Err(e) = provider.process_scheduled_jobs().await {
                    tracing::error!(error = %e, "Error processing scheduled jobs");
                }
            }
        })
    }

    /// Process job if provider is running and job is ready for execution
    fn is_job_ready(job: &JobEnvelope) -> bool {
        match job.status {
            JobStatus::Pending => true,
            JobStatus::Scheduled => {
                if let Some(scheduled_time) = job.scheduled_for {
                    Utc::now() >= scheduled_time
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

/// Clone implementation for InMemoryJobProvider
impl Clone for InMemoryJobProvider {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            config: self.config.clone(),
            queues: self.queues.clone(),
            workers: self.workers.clone(),
            handlers: self.handlers.clone(),
            running: self.running.clone(),
            worker_handles: self.worker_handles.clone(),
            event_publisher: self.event_publisher.clone(),
        }
    }
}

#[async_trait]
impl JobProvider for InMemoryJobProvider {
    async fn get_info(&self) -> JobResult<ProviderInfo> {
        let queues = self.queues.read().await;
        let workers = self.workers.lock().await;

        Ok(ProviderInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            queue_count: queues.len(),
            job_count: 0, // TODO: Calculate total job count
            worker_count: workers.len(),
            implementation_type: "in-memory".to_string(),
        })
    }

    async fn list_queues(&self) -> JobResult<Vec<QueueInfo>> {
        let queues = self.queues.read().await;
        let mut result = Vec::with_capacity(queues.len());

        for queue in queues.values() {
            result.push(queue.get_queue_info().await?);
        }

        Ok(result)
    }

    async fn queue_exists(&self, queue: &str) -> JobResult<bool> {
        let queues = self.queues.read().await;
        Ok(queues.contains_key(queue))
    }

    async fn create_queue(&self, queue: &str) -> JobResult<()> {
        // Check if queue already exists
        let exists = self.queue_exists(queue).await?;
        if exists {
            return Ok(());
        }

        // Check queue name constraints
        if queue.is_empty() {
            return Err(JobError::InvalidQueue(
                "Queue name cannot be empty".to_string(),
            ));
        }

        // Validate queue name if validation is enabled
        if self.config.validate_queues && !self.config.known_queues.contains(&queue.to_string()) {
            return Err(JobError::InvalidQueue(format!(
                "Queue '{}' is not in the list of known queues",
                queue
            )));
        }

        // Check max queues constraint
        if let Some(max_queues) = self.config.max_queues {
            let queues = self.queues.read().await;
            if queues.len() >= max_queues {
                return Err(JobError::ProviderError(format!(
                    "Maximum number of queues ({}) reached",
                    max_queues
                )));
            }
        }

        // Create the queue
        let queue = InMemoryQueue::new(queue);
        let mut queues = self.queues.write().await;
        queues.insert(queue.name.clone(), Arc::new(queue));

        Ok(())
    }

    async fn delete_queue(&self, queue: &str) -> JobResult<()> {
        let mut queues = self.queues.write().await;
        queues.remove(queue);
        Ok(())
    }

    async fn start(&self) -> JobResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Start worker threads
        self.start_workers().await?;

        // Start the cleanup task
        let cleanup_handle = self.start_cleanup_task();
        let mut handles = self.worker_handles.lock().await;
        handles.push(cleanup_handle);

        // Start the scheduler
        let scheduler_handle = self.start_scheduler();
        handles.push(scheduler_handle);

        *running = true;
        Ok(())
    }

    async fn stop(&self) -> JobResult<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        // Stop all workers
        let mut workers = self.workers.lock().await;
        for (_, worker) in workers.iter_mut() {
            worker.stop().await?;
        }
        workers.clear();

        // Abort all background tasks
        let mut handles = self.worker_handles.lock().await;
        for handle in handles.iter() {
            handle.abort();
        }
        handles.clear();

        *running = false;
        Ok(())
    }

    async fn pause(&self) -> JobResult<()> {
        let running = self.running.read().await;
        if !*running {
            return Err(JobError::ProviderError(
                "Provider is not running".to_string(),
            ));
        }

        // Pause all workers
        let workers = self.workers.lock().await;
        for (_, worker) in workers.iter() {
            if let Some(tx) = &worker.command_tx {
                let _ = tx.send(WorkerCommand::Pause).await;
            }
        }

        Ok(())
    }

    async fn resume(&self) -> JobResult<()> {
        let running = self.running.read().await;
        if !*running {
            return Err(JobError::ProviderError(
                "Provider is not running".to_string(),
            ));
        }

        // Resume all workers
        let workers = self.workers.lock().await;
        for (_, worker) in workers.iter() {
            if let Some(tx) = &worker.command_tx {
                let _ = tx.send(WorkerCommand::Resume).await;
            }
        }

        Ok(())
    }

    async fn is_running(&self) -> JobResult<bool> {
        let running = self.running.read().await;
        Ok(*running)
    }

    async fn schedule_raw(&self, envelope: JobEnvelope) -> JobResult<Uuid> {
        // Store job ID for return
        let job_id = envelope.id;
        let queue_name = envelope.queue.clone();

        // Create queue if it doesn't exist
        if !self.queue_exists(&queue_name).await? {
            self.create_queue(&queue_name).await?;
        }

        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(&queue_name)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue_name)))?;

        // Add job to queue
        queue.add_job(envelope.clone()).await?;

        // Publish event
        self.publish_event("job.scheduled", &envelope).await?;

        // Process job if provider is running and job is ready
        let running = self.running.read().await;
        if *running && Self::is_job_ready(&envelope) {
            // Process immediately
            self.process_job(envelope).await?;
        }

        Ok(job_id)
    }

    async fn register_handler_raw(
        &self,
        job_type: &str,
        handler: Box<dyn JobHandlerFn>,
    ) -> JobResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(job_type.to_string(), handler);
        Ok(())
    }

    async fn unregister_handler(&self, job_type: &str) -> JobResult<bool> {
        let mut handlers = self.handlers.write().await;
        Ok(handlers.remove(job_type).is_some())
    }

    async fn cancel(&self, job_id: &Uuid) -> JobResult<bool> {
        // Find the queue that contains the job
        let queues = self.queues.read().await;
        for queue in queues.values() {
            if let Some(job) = queue.get_job(job_id).await? {
                // Cancel the job
                let result = queue.cancel_job(job_id).await?;
                if result {
                    // Publish event
                    self.publish_event("job.cancelled", &job).await?;
                }
                return Ok(result);
            }
        }

        // Job not found
        Ok(false)
    }

    async fn get_job(&self, job_id: &Uuid) -> JobResult<Option<JobEnvelope>> {
        // Find the queue that contains the job
        let queues = self.queues.read().await;
        for queue in queues.values() {
            if let Some(job) = queue.get_job(job_id).await? {
                return Ok(Some(job));
            }
        }

        // Job not found
        Ok(None)
    }

    async fn get_jobs(
        &self,
        queue: &str,
        filter: &JobFilterConfig,
        limit: usize,
    ) -> JobResult<Vec<JobEnvelope>> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Get jobs by filter
        queue.get_jobs_by_filter(filter, limit).await
    }

    async fn get_completed_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Get completed jobs
        queue.get_completed_jobs(limit).await
    }

    async fn get_failed_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Get failed jobs
        queue.get_failed_jobs(limit).await
    }

    async fn get_scheduled_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Get scheduled jobs
        queue.get_scheduled_jobs(limit).await
    }

    async fn get_recurring_jobs(&self, queue: &str, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Get recurring jobs
        queue.get_recurring_jobs(limit).await
    }

    async fn retry_job(&self, job_id: &Uuid) -> JobResult<bool> {
        // Find the queue that contains the job
        let queues = self.queues.read().await;
        for queue in queues.values() {
            if let Some(job) = queue.get_job(job_id).await? {
                if job.status == JobStatus::Failed {
                    // Retry the job
                    let result = queue.retry_job(job_id).await?;
                    if result {
                        // Get the updated job and publish event
                        if let Some(updated_job) = queue.get_job(job_id).await? {
                            self.publish_event("job.retry", &updated_job).await?;
                        }
                    }
                    return Ok(result);
                }
                // Job found but not failed
                return Ok(false);
            }
        }

        // Job not found
        Ok(false)
    }

    async fn retry_all_failed_jobs(&self, queue: &str) -> JobResult<usize> {
        // Get the queue
        let queues = self.queues.read().await;
        let queue = queues
            .get(queue)
            .ok_or_else(|| JobError::QueueNotFound(format!("Queue not found: {}", queue)))?;

        // Retry all failed jobs
        queue.retry_all_failed_jobs().await
    }

    async fn health_check(&self) -> JobResult<bool> {
        let running = self.running.read().await;
        Ok(*running)
    }
}

/// Implementation of the JobProviderFactory for the in-memory provider
#[derive(Clone)]
pub struct InMemoryJobProviderFactory {}

impl InMemoryJobProviderFactory {
    /// Create a new in-memory job provider factory
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InMemoryJobProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobProviderFactory for InMemoryJobProviderFactory {
    type Provider = InMemoryJobProvider;

    async fn create_provider(&self, config: JobProviderConfig) -> JobResult<Arc<Self::Provider>> {
        let provider = InMemoryJobProvider::new(config);
        Ok(Arc::new(provider))
    }
}

/// Execute a job asynchronously
async fn execute_job(
    job: JobEnvelope,
    handlers: &HashMap<String, Box<dyn JobHandlerFn>>,
) -> JobResult<JobExecutionResult> {
    // Get the handler
    let handler = handlers.get(&job.job_type).ok_or_else(|| {
        JobError::HandlerNotFound(format!("No handler found for job type: {}", job.job_type))
    })?;

    // Execute the handler
    handler.execute(job)
}

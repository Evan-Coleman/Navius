use crate::error::{JobError, JobExecutionResult, JobResult, JobStatus};
use crate::job::{JobEnvelope, JobFilterConfig};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// In-memory queue for jobs
pub struct InMemoryQueue {
    /// Queue name
    pub name: String,

    /// Jobs in the queue
    jobs: Mutex<HashMap<Uuid, JobEnvelope>>,

    /// Pending jobs queue
    pending_jobs: Mutex<VecDeque<Uuid>>,

    /// Scheduled jobs (job_id -> scheduled_time)
    scheduled_jobs: Mutex<HashMap<Uuid, DateTime<Utc>>>,

    /// Running jobs
    running_jobs: Mutex<HashMap<Uuid, JobEnvelope>>,

    /// Completed jobs
    completed_jobs: Mutex<VecDeque<JobEnvelope>>,

    /// Failed jobs
    failed_jobs: Mutex<VecDeque<JobEnvelope>>,

    /// Recurring jobs
    recurring_jobs: Mutex<HashMap<Uuid, JobEnvelope>>,
}

impl InMemoryQueue {
    /// Create a new in-memory queue
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            jobs: Mutex::new(HashMap::new()),
            pending_jobs: Mutex::new(VecDeque::new()),
            scheduled_jobs: Mutex::new(HashMap::new()),
            running_jobs: Mutex::new(HashMap::new()),
            completed_jobs: Mutex::new(VecDeque::new()),
            failed_jobs: Mutex::new(VecDeque::new()),
            recurring_jobs: Mutex::new(HashMap::new()),
        }
    }

    /// Add a job to the queue
    pub async fn add_job(&self, job: JobEnvelope) -> JobResult<()> {
        let job_id = job.id;

        // Store the job
        {
            let mut jobs = self.jobs.lock().await;
            jobs.insert(job_id, job.clone());
        }

        // Add to appropriate queue based on status and schedule
        match (job.status, job.scheduled_for, job.cron_expression.is_some()) {
            // Pending job
            (JobStatus::Pending, _, _) => {
                let mut pending = self.pending_jobs.lock().await;
                pending.push_back(job_id);
            }

            // Scheduled job
            (JobStatus::Scheduled, Some(time), false) => {
                let mut scheduled = self.scheduled_jobs.lock().await;
                scheduled.insert(job_id, time);
            }

            // Recurring job
            (_, _, true) => {
                let mut recurring = self.recurring_jobs.lock().await;
                recurring.insert(job_id, job);
            }

            // Handle other cases (completed, failed, etc.)
            (JobStatus::Completed, _, _) => {
                let mut completed = self.completed_jobs.lock().await;
                completed.push_back(job);
            }

            (JobStatus::Failed, _, _) => {
                let mut failed = self.failed_jobs.lock().await;
                failed.push_back(job);
            }

            _ => {
                // Unknown status, treat as pending
                let mut pending = self.pending_jobs.lock().await;
                pending.push_back(job_id);
            }
        }

        Ok(())
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &Uuid) -> JobResult<Option<JobEnvelope>> {
        let jobs = self.jobs.lock().await;
        Ok(jobs.get(job_id).cloned())
    }

    /// Get a pending job
    pub async fn get_pending_job(&self) -> JobResult<Option<JobEnvelope>> {
        let mut pending = self.pending_jobs.lock().await;
        if let Some(job_id) = pending.pop_front() {
            let jobs = self.jobs.lock().await;
            return Ok(jobs.get(&job_id).cloned());
        }
        Ok(None)
    }

    /// Get jobs that are due to be executed
    pub async fn get_due_jobs(&self) -> JobResult<Vec<JobEnvelope>> {
        let now = Utc::now();
        let mut due_jobs = Vec::new();

        // Check scheduled jobs
        let mut scheduled = self.scheduled_jobs.lock().await;
        let mut job_ids_to_move = Vec::new();

        for (job_id, scheduled_time) in scheduled.iter() {
            if *scheduled_time <= now {
                job_ids_to_move.push(*job_id);
            }
        }

        // Get the jobs and remove from scheduled list
        if !job_ids_to_move.is_empty() {
            let jobs = self.jobs.lock().await;
            for job_id in job_ids_to_move {
                if let Some(job) = jobs.get(&job_id) {
                    due_jobs.push(job.clone());
                }
                scheduled.remove(&job_id);
            }
        }

        // Return any due jobs
        Ok(due_jobs)
    }

    /// Mark a job as running
    pub async fn mark_job_running(&self, job_id: &Uuid) -> JobResult<bool> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Running;
            job.started_at = Some(Utc::now());

            // Add to running jobs
            let mut running = self.running_jobs.lock().await;
            running.insert(*job_id, job.clone());

            return Ok(true);
        }
        Ok(false)
    }

    /// Mark a job as completed
    pub async fn mark_job_completed(
        &self,
        job_id: &Uuid,
        result: JobExecutionResult,
    ) -> JobResult<bool> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Completed;
            job.completed_at = Some(Utc::now());
            job.result = Some(result);

            // Remove from running jobs
            let mut running = self.running_jobs.lock().await;
            running.remove(job_id);

            // Add to completed jobs
            let mut completed = self.completed_jobs.lock().await;
            completed.push_back(job.clone());

            return Ok(true);
        }
        Ok(false)
    }

    /// Mark a job as failed
    pub async fn mark_job_failed(
        &self,
        job_id: &Uuid,
        error: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> JobResult<bool> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Failed;
            job.completed_at = Some(Utc::now());
            job.result = Some(crate::error::JobExecutionResult::Failure {
                message: error.into(),
                details,
            });

            // Increment retry count if applicable
            job.retry_count += 1;

            // Remove from running jobs
            let mut running = self.running_jobs.lock().await;
            running.remove(job_id);

            // Check if the job should be retried
            if job.retry_count < job.max_retries {
                // Calculate backoff time
                let backoff_seconds = if let Some(backoff) = &job.retry_backoff {
                    let retry_index = job.retry_count as usize - 1;
                    if retry_index < backoff.len() {
                        backoff[retry_index]
                    } else {
                        backoff.last().copied().unwrap_or(60) // Default to last value or 60s
                    }
                } else {
                    // Default exponential backoff: 1s, 5s, 25s, 125s, ...
                    (5_u32.pow(job.retry_count)).min(300) // Cap at 5 minutes
                };

                // Schedule for retry
                job.status = JobStatus::Scheduled;
                job.scheduled_for =
                    Some(Utc::now() + chrono::Duration::seconds(backoff_seconds as i64));

                // Add to scheduled jobs
                let mut scheduled = self.scheduled_jobs.lock().await;
                scheduled.insert(*job_id, job.scheduled_for.unwrap());
            } else {
                // Add to failed jobs if no more retries
                let mut failed = self.failed_jobs.lock().await;
                failed.push_back(job.clone());
            }

            return Ok(true);
        }
        Ok(false)
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: &Uuid) -> JobResult<bool> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            // Only cancel if not already completed or failed
            if job.status != JobStatus::Completed && job.status != JobStatus::Failed {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(Utc::now());

                // Remove from pending queue if present
                let mut pending = self.pending_jobs.lock().await;
                let position = pending.iter().position(|id| id == job_id);
                if let Some(pos) = position {
                    pending.remove(pos);
                }

                // Remove from scheduled queue if present
                let mut scheduled = self.scheduled_jobs.lock().await;
                scheduled.remove(job_id);

                // Remove from running jobs if present
                let mut running = self.running_jobs.lock().await;
                running.remove(job_id);

                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get jobs that match a filter
    pub async fn get_jobs_by_filter(
        &self,
        filter: &JobFilterConfig,
        limit: usize,
    ) -> JobResult<Vec<JobEnvelope>> {
        let jobs = self.jobs.lock().await;
        let mut filtered_jobs = Vec::new();

        for job in jobs.values() {
            if filter.matches(job) {
                filtered_jobs.push(job.clone());
                if filtered_jobs.len() >= limit {
                    break;
                }
            }
        }

        Ok(filtered_jobs)
    }

    /// Get completed jobs
    pub async fn get_completed_jobs(&self, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        let completed = self.completed_jobs.lock().await;
        let mut result = Vec::new();
        for job in completed.iter().rev().take(limit) {
            result.push(job.clone());
        }
        Ok(result)
    }

    /// Get failed jobs
    pub async fn get_failed_jobs(&self, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        let failed = self.failed_jobs.lock().await;
        let mut result = Vec::new();
        for job in failed.iter().rev().take(limit) {
            result.push(job.clone());
        }
        Ok(result)
    }

    /// Get scheduled jobs
    pub async fn get_scheduled_jobs(&self, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        let scheduled_ids = self.scheduled_jobs.lock().await;
        let jobs = self.jobs.lock().await;
        let mut result = Vec::new();

        for job_id in scheduled_ids.keys().take(limit) {
            if let Some(job) = jobs.get(job_id) {
                result.push(job.clone());
            }
        }

        Ok(result)
    }

    /// Get recurring jobs
    pub async fn get_recurring_jobs(&self, limit: usize) -> JobResult<Vec<JobEnvelope>> {
        let recurring = self.recurring_jobs.lock().await;
        let mut result = Vec::new();
        for job in recurring.values().take(limit) {
            result.push(job.clone());
        }
        Ok(result)
    }

    /// Get queue information
    pub async fn get_queue_info(&self) -> JobResult<crate::provider::QueueInfo> {
        let jobs = self.jobs.lock().await;
        let pending = self.pending_jobs.lock().await;
        let running = self.running_jobs.lock().await;
        let scheduled = self.scheduled_jobs.lock().await;
        let completed = self.completed_jobs.lock().await;
        let failed = self.failed_jobs.lock().await;
        let recurring = self.recurring_jobs.lock().await;

        Ok(crate::provider::QueueInfo {
            name: self.name.clone(),
            job_count: jobs.len(),
            running_job_count: running.len(),
            scheduled_job_count: scheduled.len(),
            completed_job_count: completed.len(),
            failed_job_count: failed.len(),
            recurring_job_count: recurring.len(),
        })
    }

    /// Clean up old jobs based on a cutoff time
    pub async fn cleanup_old_jobs(&self, cutoff: DateTime<Utc>) -> usize {
        let mut cleaned_up = 0;

        // Clean up completed jobs
        {
            let mut completed = self.completed_jobs.lock().await;
            let initial_len = completed.len();
            completed.retain(|job| job.completed_at.unwrap_or(job.created_at) > cutoff);
            cleaned_up += initial_len - completed.len();
        }

        // Clean up failed jobs
        {
            let mut failed = self.failed_jobs.lock().await;
            let initial_len = failed.len();
            failed.retain(|job| job.completed_at.unwrap_or(job.created_at) > cutoff);
            cleaned_up += initial_len - failed.len();
        }

        // Clean up from main jobs map (but only if they're completed or failed)
        {
            let mut jobs = self.jobs.lock().await;
            let initial_len = jobs.len();
            jobs.retain(|_, job| {
                job.status != JobStatus::Completed && job.status != JobStatus::Failed
                    || job.completed_at.unwrap_or(job.created_at) > cutoff
            });
            cleaned_up += initial_len - jobs.len();
        }

        cleaned_up
    }

    /// Enforce maximum jobs per queue
    pub async fn enforce_max_jobs(&self, max_jobs: usize) -> usize {
        let mut removed = 0;

        // First reduce completed jobs
        {
            let mut completed = self.completed_jobs.lock().await;
            if completed.len() > max_jobs / 2 {
                let to_remove = completed.len() - max_jobs / 2;
                for _ in 0..to_remove {
                    completed.pop_front();
                }
                removed += to_remove;
            }
        }

        // Then reduce failed jobs
        {
            let mut failed = self.failed_jobs.lock().await;
            if failed.len() > max_jobs / 2 {
                let to_remove = failed.len() - max_jobs / 2;
                for _ in 0..to_remove {
                    failed.pop_front();
                }
                removed += to_remove;
            }
        }

        removed
    }

    /// Retry a failed job
    pub async fn retry_job(&self, job_id: &Uuid) -> JobResult<bool> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            if job.status == JobStatus::Failed {
                // Reset job status
                job.status = JobStatus::Pending;
                job.started_at = None;
                job.completed_at = None;
                job.result = None;

                // Remove from failed jobs
                let mut failed = self.failed_jobs.lock().await;
                let position = failed.iter().position(|j| j.id == *job_id);
                if let Some(pos) = position {
                    failed.remove(pos);
                }

                // Add to pending queue
                let mut pending = self.pending_jobs.lock().await;
                pending.push_back(*job_id);

                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Retry all failed jobs
    pub async fn retry_all_failed_jobs(&self) -> JobResult<usize> {
        let mut retried = 0;
        let mut failed_job_ids = Vec::new();

        // Get all failed job IDs
        {
            let failed = self.failed_jobs.lock().await;
            for job in failed.iter() {
                failed_job_ids.push(job.id);
            }
        }

        // Retry each job
        for job_id in failed_job_ids {
            if self.retry_job(&job_id).await? {
                retried += 1;
            }
        }

        Ok(retried)
    }
}

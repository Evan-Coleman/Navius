use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::{JobError, JobResult};
use crate::job::{Job, JobId, JobStatus};
use crate::store::{JobStore, JobStoreConfig};

/// In-memory job store implementation
pub struct MemoryJobStore {
    jobs: Arc<Mutex<HashMap<JobId, Job>>>,
    status_tx: broadcast::Sender<(JobId, JobStatus)>,
}

impl MemoryJobStore {
    /// Create a new memory job store
    pub fn new(_config: JobStoreConfig) -> Self {
        let (status_tx, _) = broadcast::channel(100);
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            status_tx,
        }
    }
}

#[async_trait::async_trait]
impl JobStore for MemoryJobStore {
    async fn create(&self, job: Job) -> JobResult<JobId> {
        let id = Uuid::new_v4().to_string();
        let mut jobs = self.jobs.lock().unwrap();
        jobs.insert(id.clone(), job);
        Ok(id)
    }

    async fn get(&self, id: &str) -> JobResult<Option<Job>> {
        let jobs = self.jobs.lock().unwrap();
        Ok(jobs.get(id).cloned())
    }

    async fn update(&self, id: &str, job: Job) -> JobResult<()> {
        let mut jobs = self.jobs.lock().unwrap();
        if !jobs.contains_key(id) {
            return Err(JobError::NotFound(id.to_string()));
        }
        jobs.insert(id.to_string(), job);
        Ok(())
    }

    async fn delete(&self, id: &str) -> JobResult<()> {
        let mut jobs = self.jobs.lock().unwrap();
        if jobs.remove(id).is_none() {
            return Err(JobError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn list(&self) -> JobResult<Vec<Job>> {
        let jobs = self.jobs.lock().unwrap();
        Ok(jobs.values().cloned().collect())
    }

    async fn subscribe(&self) -> broadcast::Receiver<(JobId, JobStatus)> {
        self.status_tx.subscribe()
    }

    async fn publish_status(&self, id: &str, status: JobStatus) -> JobResult<()> {
        self.status_tx.send((id.to_string(), status)).map_err(|e| {
            JobError::Other(format!("Failed to publish job status: {}", e))
        })?;
 
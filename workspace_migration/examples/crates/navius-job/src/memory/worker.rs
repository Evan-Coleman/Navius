use crate::error::{JobError, JobExecutionResult, JobResult};
use crate::job::JobEnvelope;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Worker for processing jobs
pub struct Worker {
    /// Worker ID
    pub id: String,

    /// Worker name
    pub name: String,

    /// Whether the worker is running
    pub running: bool,

    /// Channel for worker control commands
    pub command_tx: Option<Sender<WorkerCommand>>,

    /// Join handle for the worker task
    pub join_handle: Option<JoinHandle<()>>,
}

/// Commands for controlling a worker
pub enum WorkerCommand {
    /// Process a job
    ProcessJob(JobEnvelope),

    /// Stop the worker
    Stop,

    /// Pause the worker
    Pause,

    /// Resume the worker
    Resume,
}

/// Job completion notification
pub struct JobCompletionNotification {
    /// Job ID
    pub job_id: Uuid,

    /// Queue name
    pub queue: String,

    /// Result of job execution
    pub result: JobExecution,
}

/// Result of job execution
pub enum JobExecution {
    /// Job completed successfully
    Success(JobExecutionResult),

    /// Job failed
    Failure {
        /// Error message
        message: String,
        /// Error details
        details: Option<serde_json::Value>,
    },
}

impl Worker {
    /// Create a new worker
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            running: false,
            command_tx: None,
            join_handle: None,
        }
    }

    /// Start the worker
    pub fn start(
        &mut self,
        handlers: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    String,
                    Box<
                        dyn Fn(JobEnvelope) -> JobResult<JobExecutionResult>
                            + Send
                            + Sync
                            + 'static,
                    >,
                >,
            >,
        >,
        completion_tx: Sender<JobCompletionNotification>,
    ) -> JobResult<()> {
        if self.running {
            return Ok(());
        }

        let (command_tx, command_rx) = tokio::sync::mpsc::channel(100);
        self.command_tx = Some(command_tx);

        let worker_id = self.id.clone();
        let worker_name = self.name.clone();

        let handle = tokio::spawn(async move {
            Self::worker_loop(worker_id, worker_name, command_rx, handlers, completion_tx).await;
        });

        self.join_handle = Some(handle);
        self.running = true;

        Ok(())
    }

    /// Stop the worker
    pub async fn stop(&mut self) -> JobResult<()> {
        if !self.running {
            return Ok(());
        }

        if let Some(tx) = &self.command_tx {
            let _ = tx.send(WorkerCommand::Stop).await;
        }

        if let Some(handle) = self.join_handle.take() {
            let _ = handle.await;
        }

        self.running = false;
        self.command_tx = None;

        Ok(())
    }

    /// Worker processing loop
    async fn worker_loop(
        worker_id: String,
        worker_name: String,
        mut command_rx: Receiver<WorkerCommand>,
        handlers: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    String,
                    Box<
                        dyn Fn(JobEnvelope) -> JobResult<JobExecutionResult>
                            + Send
                            + Sync
                            + 'static,
                    >,
                >,
            >,
        >,
        completion_tx: Sender<JobCompletionNotification>,
    ) {
        tracing::info!(worker_id = %worker_id, worker_name = %worker_name, "Worker started");

        let mut paused = false;

        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                WorkerCommand::ProcessJob(job) if !paused => {
                    // Process the job
                    let job_id = job.id;
                    let queue = job.queue.clone();
                    let job_type = job.job_type.clone();

                    tracing::debug!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        job_type = %job_type,
                        queue = %queue,
                        "Processing job"
                    );

                    // Try to find a handler for this job type
                    let result = {
                        let handlers = handlers.read().await;
                        if let Some(handler) = handlers.get(&job_type) {
                            // Execute the handler with the job
                            match handler(job.clone()) {
                                Ok(result) => JobExecution::Success(result),
                                Err(e) => JobExecution::Failure {
                                    message: e.to_string(),
                                    details: None,
                                },
                            }
                        } else {
                            // No handler found for this job type
                            JobExecution::Failure {
                                message: format!(
                                    "No handler registered for job type: {}",
                                    job_type
                                ),
                                details: None,
                            }
                        }
                    };

                    // Send job completion notification
                    let notification = JobCompletionNotification {
                        job_id,
                        queue,
                        result,
                    };

                    if let Err(e) = completion_tx.send(notification).await {
                        tracing::error!(
                            worker_id = %worker_id,
                            error = %e,
                            "Failed to send job completion notification"
                        );
                    }
                }

                WorkerCommand::ProcessJob(_) if paused => {
                    // Worker is paused, do nothing
                    tracing::debug!(
                        worker_id = %worker_id,
                        "Worker is paused, ignoring job"
                    );
                }

                WorkerCommand::Stop => {
                    tracing::info!(
                        worker_id = %worker_id,
                        "Worker stopping"
                    );
                    break;
                }

                WorkerCommand::Pause => {
                    paused = true;
                    tracing::info!(
                        worker_id = %worker_id,
                        "Worker paused"
                    );
                }

                WorkerCommand::Resume => {
                    paused = false;
                    tracing::info!(
                        worker_id = %worker_id,
                        "Worker resumed"
                    );
                }
            }
        }

        tracing::info!(
            worker_id = %worker_id,
            "Worker stopped"
        );
    }
}

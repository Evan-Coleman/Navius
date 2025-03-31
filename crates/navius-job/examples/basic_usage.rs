use navius_job::error::{JobError, JobExecutionResult, JobResult};
use navius_job::job::Job;
use navius_job::memory::InMemoryJobProviderFactory;
use navius_job::provider::{JobProvider, JobProviderConfig, JobProviderFactory, SchedulingOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

// Define job payloads
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EmailJob {
    to: String,
    subject: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProcessDataJob {
    data_id: String,
    priority: u32,
}

// Example of a job handler function
async fn handle_email_job(job: Job<EmailJob>) -> JobResult<JobExecutionResult> {
    println!(
        "Sending email to {} with subject: {}",
        job.payload.to, job.payload.subject
    );
    println!("Email body: {}", job.payload.body);

    // Simulate email sending
    time::sleep(Duration::from_millis(500)).await;

    // Return success result with some metadata
    Ok(JobExecutionResult::success().with_metadata("sent_at", chrono::Utc::now().to_rfc3339()))
}

async fn handle_process_data_job(job: Job<ProcessDataJob>) -> JobResult<JobExecutionResult> {
    println!(
        "Processing data with ID: {} (priority: {})",
        job.payload.data_id, job.payload.priority
    );

    // Simulate processing based on priority
    let processing_time = match job.payload.priority {
        0..=3 => 1000, // Low priority: longer processing time
        4..=7 => 500,  // Medium priority
        _ => 200,      // High priority: shorter processing time
    };

    time::sleep(Duration::from_millis(processing_time)).await;

    // Randomly fail some jobs to demonstrate error handling
    if job.payload.data_id.ends_with("3") {
        return Err(JobError::ProcessingError(
            "Data processing failed due to corrupt data".to_string(),
        ));
    }

    // Return success with some metrics
    Ok(JobExecutionResult::success()
        .with_metadata("processing_time_ms", processing_time.to_string())
        .with_metadata("processed_at", chrono::Utc::now().to_rfc3339()))
}

// Function to demonstrate job cancellation
async fn cancel_random_job(provider: &Arc<dyn JobProvider>, job_ids: &[Uuid]) -> JobResult<()> {
    // Cancel a random job
    if !job_ids.is_empty() {
        let index = rand::random::<usize>() % job_ids.len();
        let job_id = job_ids[index];

        println!("Attempting to cancel job with ID: {}", job_id);
        let cancelled = provider.cancel(&job_id).await?;

        if cancelled {
            println!("Job cancelled successfully");
        } else {
            println!("Job could not be cancelled (might be already completed or running)");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Navius Job System - Basic Example");
    println!("=================================");

    // Create a provider configuration
    let config = JobProviderConfig::default()
        .with_worker_threads(2) // Use 2 worker threads
        .with_job_retention_seconds(3600) // Keep completed jobs for 1 hour
        .with_max_retained_jobs_per_queue(100) // Keep at most 100 jobs per queue
        .with_default_max_retries(3) // Retry failed jobs up to 3 times
        .with_default_timeout_seconds(Some(30)); // 30 second timeout for jobs

    // Create a provider factory and a provider
    let factory = InMemoryJobProviderFactory::new();
    let provider = factory.create_provider(config).await?;

    println!("\nCreating queues...");
    provider.create_queue("emails").await?;
    provider.create_queue("data-processing").await?;

    // Register job handlers
    println!("\nRegistering job handlers...");
    provider
        .register_handler("send-email", handle_email_job)
        .await?;
    provider
        .register_handler("process-data", handle_process_data_job)
        .await?;

    // Start the provider
    println!("\nStarting job provider...");
    provider.start().await?;

    // Create some email jobs
    println!("\nScheduling email jobs...");
    let mut job_ids = Vec::new();

    // Schedule a regular email job
    let email_job = EmailJob {
        to: "user@example.com".to_string(),
        subject: "Welcome to Navius".to_string(),
        body: "Thank you for joining Navius!".to_string(),
    };
    let job_id = provider
        .schedule(Job::new("send-email", "emails", "example", email_job))
        .await?;
    job_ids.push(job_id);
    println!("Scheduled welcome email job with ID: {}", job_id);

    // Schedule an email job with options
    let options = SchedulingOptions::default()
        .with_priority(5) // Medium priority
        .with_correlation_id("user-123".to_string())
        .with_metadata("campaign", "onboarding".to_string());

    let email_job = EmailJob {
        to: "another@example.com".to_string(),
        subject: "Your account setup".to_string(),
        body: "Please complete your account setup...".to_string(),
    };

    let job_id = provider
        .schedule_with_options("send-email", "emails", email_job, options)
        .await?;
    job_ids.push(job_id);
    println!("Scheduled setup email job with ID: {}", job_id);

    // Schedule a delayed email job
    let scheduled_time = chrono::Utc::now() + chrono::Duration::seconds(5);

    let email_job = EmailJob {
        to: "delayed@example.com".to_string(),
        subject: "Delayed notification".to_string(),
        body: "This email was scheduled for later delivery.".to_string(),
    };

    let job_id = provider
        .schedule_at(
            "send-email",
            "emails",
            email_job,
            scheduled_time,
            SchedulingOptions::default(),
        )
        .await?;
    job_ids.push(job_id);
    println!(
        "Scheduled delayed email job with ID: {} (runs at {})",
        job_id, scheduled_time
    );

    // Schedule some data processing jobs
    println!("\nScheduling data processing jobs...");
    for i in 0..5 {
        let priority = i * 2; // 0, 2, 4, 6, 8

        let data_job = ProcessDataJob {
            data_id: format!("data-{}", i),
            priority,
        };

        let options = SchedulingOptions::default()
            .with_priority(priority)
            .with_metadata("source", "example-loader".to_string());

        let job_id = provider
            .schedule_with_options("process-data", "data-processing", data_job, options)
            .await?;

        job_ids.push(job_id);
        println!(
            "Scheduled data processing job with ID: {} (priority: {})",
            job_id, priority
        );
    }

    // Schedule a recurring job
    println!("\nScheduling a recurring job...");
    let recurring_job = ProcessDataJob {
        data_id: "recurring-data".to_string(),
        priority: 1,
    };

    let job_id = provider
        .schedule_recurring(
            "process-data",
            "data-processing",
            recurring_job,
            "*/10 * * * * *", // Run every 10 seconds
            SchedulingOptions::default(),
        )
        .await?;

    println!("Scheduled recurring job with ID: {}", job_id);
    job_ids.push(job_id);

    // Cancel a random job
    time::sleep(Duration::from_secs(1)).await;
    cancel_random_job(&provider, &job_ids).await?;

    // Wait for jobs to be processed
    println!("\nWaiting for jobs to be processed...");
    time::sleep(Duration::from_secs(10)).await;

    // Retrieve and display job information
    println!("\nJob Information:");
    for job_id in &job_ids {
        if let Some(job) = provider.get_job(job_id).await? {
            println!("{}: {} - {}", job_id, job.job_type, job.status);
        } else {
            println!("{}: Job not found", job_id);
        }
    }

    // Display queue information
    println!("\nQueue Information:");
    let queues = provider.list_queues().await?;
    for queue in queues {
        println!(
            "{}: {} jobs (pending: {}, completed: {}, failed: {})",
            queue.name,
            queue.job_count,
            queue.pending_count,
            queue.completed_count,
            queue.failed_count
        );
    }

    // Retrieve and display failed jobs
    println!("\nFailed Jobs:");
    let failed_jobs = provider.get_failed_jobs("data-processing", 10).await?;
    for job in failed_jobs {
        println!(
            "{}: {} - Failed: {}",
            job.id,
            job.job_type,
            job.failure_reason.unwrap_or_default()
        );
    }

    // Retry failed jobs
    if !failed_jobs.is_empty() {
        println!("\nRetrying failed jobs...");
        let retry_count = provider.retry_all_failed_jobs("data-processing").await?;
        println!("Retried {} failed jobs", retry_count);

        // Wait for retry processing
        time::sleep(Duration::from_secs(2)).await;
    }

    // Check provider health
    println!(
        "\nProvider Health Check: {}",
        provider.health_check().await?
    );

    // Stop the provider
    println!("\nStopping job provider...");
    provider.stop().await?;

    println!("\nExample completed.");
    Ok(())
}

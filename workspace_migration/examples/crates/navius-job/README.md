# Navius Job System

A flexible, typed job processing system for Rust applications with support for scheduling, prioritization, retries, and more.

## Features

- **Type-safe job definitions**: Define job payloads with Rust types.
- **Multiple providers**: In-memory provider available with planned support for persistent providers.
- **Job scheduling**: Schedule jobs for immediate or future execution.
- **Recurring jobs**: Schedule jobs to run on a recurring basis using cron expressions.
- **Job prioritization**: Prioritize jobs within queues.
- **Automatic retries**: Configure retry logic with backoff policies.
- **Job cancellation**: Cancel pending or scheduled jobs.
- **Job filtering**: Retrieve jobs by status, type, or custom filters.
- **Job timeouts**: Set execution timeouts for jobs.
- **Correlation tracking**: Track related jobs using correlation IDs.
- **Event integration**: Optional integration with the Navius event system.
- **Pluggable architecture**: Easily extend with custom job providers.

## Quick Start

```rust
use navius_job::error::{JobExecutionResult, JobResult};
use navius_job::job::Job;
use navius_job::provider::{JobProvider, JobProviderConfig, JobProviderFactory};
use navius_job::memory::InMemoryJobProviderFactory;
use serde::{Deserialize, Serialize};

// 1. Define a job payload
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EmailJob {
    to: String,
    subject: String,
    body: String,
}

// 2. Define a job handler function
async fn handle_email_job(job: Job<EmailJob>) -> JobResult<JobExecutionResult> {
    println!("Sending email to: {}", job.payload.to);
    // Implement email sending logic...
    Ok(JobExecutionResult::success())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 3. Create a job provider
    let factory = InMemoryJobProviderFactory::new();
    let provider = factory.create_provider(JobProviderConfig::default()).await?;
    
    // 4. Register the job handler
    provider.create_queue("emails").await?;
    provider.register_handler("send-email", handle_email_job).await?;
    
    // 5. Start the provider
    provider.start().await?;
    
    // 6. Schedule a job
    let email_job = EmailJob {
        to: "user@example.com".to_string(),
        subject: "Hello".to_string(),
        body: "This is a test email".to_string(),
    };
    
    let job_id = provider.schedule(
        Job::new("send-email", "emails", "app", email_job)
    ).await?;
    
    println!("Scheduled job with ID: {}", job_id);
    
    // Wait for job to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 7. Cleanup
    provider.stop().await?;
    
    Ok(())
}
```

## Core Concepts

### Jobs

A job consists of:

- **Job Type**: String identifier for the job type
- **Queue**: The queue the job belongs to
- **Payload**: The typed data for the job
- **Status**: Current job status (pending, running, completed, failed, etc.)
- **Priority**: Numerical priority (higher values = higher priority)
- **Metadata**: Key-value pairs for additional information
- **Scheduling Information**: For delayed or recurring jobs

### Job Providers

Providers are responsible for:

- Managing job queues
- Scheduling and executing jobs
- Handling retries
- Tracking job status

The crate includes an in-memory provider for development and testing.

### Job Handlers

Job handlers are functions that process jobs:

```rust
async fn my_handler(job: Job<MyPayload>) -> JobResult<JobExecutionResult> {
    // Process the job...
    Ok(JobExecutionResult::success())
}
```

## Advanced Usage

### Scheduling Options

```rust
// Schedule with options
let options = SchedulingOptions::default()
    .with_priority(5)
    .with_max_retries(3)
    .with_retry_backoff(RetryBackoff::ExponentialSeconds(1, 30))
    .with_timeout_seconds(Some(60))
    .with_correlation_id("user-123".to_string())
    .with_metadata("source", "api".to_string());

provider.schedule_with_options(
    "job-type",
    "queue-name",
    my_payload,
    options
).await?;
```

### Scheduled Jobs

```rust
// Schedule a job to run in the future
let scheduled_time = chrono::Utc::now() + chrono::Duration::minutes(30);

provider.schedule_at(
    "send-reminder",
    "notifications",
    reminder_payload,
    scheduled_time,
    SchedulingOptions::default()
).await?;
```

### Recurring Jobs

```rust
// Schedule a job to run every day at 8am
provider.schedule_recurring(
    "daily-report",
    "reports",
    report_payload,
    "0 8 * * *", // Cron expression
    SchedulingOptions::default()
).await?;
```

### Job Filtering

```rust
// Filter configuration
let filter = JobFilterConfig {
    status: Some(JobStatus::Failed),
    job_type: Some("send-email".to_string()),
    before: Some(chrono::Utc::now()),
    after: Some(chrono::Utc::now() - chrono::Duration::days(1)),
    ..Default::default()
};

// Get filtered jobs
let jobs = provider.get_jobs("my-queue", &filter, 10).await?;
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 
# scheduled_jobs API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- async-trait
- chrono
- cron
- futures
- serde
- serde_json
- thiserror
- tokio
- tokio-stream
- tracing
- uuid
- navius-event
- navius-core

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Job<T> | job.rs | 11 | ⚠️ Partial |
| JobEnvelope | job.rs | 218 | ⚠️ Partial |
| JobFilterConfig | job.rs | 357 | ⚠️ Partial |
| Worker | worker.rs | 9 | ⚠️ Partial |
| JobCompletionNotification | worker.rs | 42 | ✅ Complete |
| InMemoryQueue | queue.rs | 10 | ⚠️ Partial |
| InMemoryJobProvider | mod.rs | 27 | ⚠️ Partial |
| InMemoryJobProviderFactory | mod.rs | 910 | ⚠️ Partial |
| JobProviderConfig | provider.rs | 13 | ⚠️ Partial |
| ProviderInfo | provider.rs | 156 | ⚠️ Partial |
| QueueInfo | provider.rs | 178 | ✅ Complete |
| SchedulingOptions | provider.rs | 207 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| WorkerCommand | worker.rs | 27 | ✅ Complete |
| JobExecution | worker.rs | 54 | ✅ Complete |
| JobError | error.rs | 10 | ⚠️ Partial |
| JobPriority | error.rs | 84 | ⚠️ Partial |
| JobStatus | error.rs | 106 | ⚠️ Partial |
| JobExecutionResult | error.rs | 141 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| JobProvider: | provider.rs | 292 | ⚠️ Partial |
| JobProviderFactory: | provider.rs | 409 | ✅ Complete |
| JobPayload: | provider.rs | 415 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new( | job.rs | 73 | ⚠️ Partial |
| scheduled( | job.rs | 103 | ⚠️ Partial |
| recurring( | job.rs | 134 | ⚠️ Partial |
| with_priority(mut | job.rs | 165 | ⚠️ Partial |
| with_correlation_id(mut | job.rs | 171 | ⚠️ Partial |
| with_metadata(mut | job.rs | 177 | ⚠️ Partial |
| with_metadata_map(mut | job.rs | 183 | ⚠️ Partial |
| with_timeout(mut | job.rs | 189 | ⚠️ Partial |
| with_retries(mut | job.rs | 195 | ⚠️ Partial |
| is_ready(&self) | job.rs | 202 | ⚠️ Partial |
| is_recurring(&self) | job.rs | 211 | ⚠️ Partial |
| from_job<T>(job: | job.rs | 279 | ✅ Complete |
| try_into_job<T>(&self) | job.rs | 317 | ⚠️ Partial |
| new() | job.rs | 385 | ✅ Complete |
| with_job_types(mut | job.rs | 399 | ⚠️ Partial |
| with_queues(mut | job.rs | 405 | ⚠️ Partial |
| with_sources(mut | job.rs | 411 | ⚠️ Partial |
| with_min_priority(mut | job.rs | 417 | ⚠️ Partial |
| with_statuses(mut | job.rs | 423 | ⚠️ Partial |
| with_correlation_id(mut | job.rs | 429 | ⚠️ Partial |
| with_metadata(mut | job.rs | 435 | ⚠️ Partial |
| with_recurring(mut | job.rs | 448 | ⚠️ Partial |
| matches(&self, | job.rs | 454 | ⚠️ Partial |
| new(id: | worker.rs | 69 | ✅ Complete |
| start( | worker.rs | 80 | ⚠️ Partial |
| fn | worker.rs | 118 | ⚠️ Partial |
| new(name: | queue.rs | 38 | ✅ Complete |
| fn | queue.rs | 52 | ⚠️ Partial |
| fn | queue.rs | 103 | ⚠️ Partial |
| fn | queue.rs | 109 | ⚠️ Partial |
| fn | queue.rs | 119 | ⚠️ Partial |
| fn | queue.rs | 149 | ⚠️ Partial |
| fn | queue.rs | 165 | ⚠️ Partial |
| fn | queue.rs | 190 | ⚠️ Partial |
| fn | queue.rs | 247 | ⚠️ Partial |
| fn | queue.rs | 277 | ⚠️ Partial |
| fn | queue.rs | 298 | ⚠️ Partial |
| fn | queue.rs | 308 | ⚠️ Partial |
| fn | queue.rs | 318 | ⚠️ Partial |
| fn | queue.rs | 333 | ⚠️ Partial |
| fn | queue.rs | 343 | ⚠️ Partial |
| fn | queue.rs | 364 | ⚠️ Partial |
| fn | queue.rs | 398 | ⚠️ Partial |
| fn | queue.rs | 429 | ⚠️ Partial |
| fn | queue.rs | 457 | ⚠️ Partial |
| new(config: | mod.rs | 65 | ✅ Complete |
| fn | mod.rs | 85 | ⚠️ Partial |
| start_cleanup_task(&self) | mod.rs | 105 | ⚠️ Partial |
| new() | mod.rs | 914 | ⚠️ Partial |
| new() | provider.rs | 56 | ✅ Complete |
| with_max_queues(mut | provider.rs | 75 | ⚠️ Partial |
| with_max_jobs_per_queue(mut | provider.rs | 81 | ⚠️ Partial |
| with_job_retention_seconds(mut | provider.rs | 87 | ⚠️ Partial |
| with_max_retained_jobs(mut | provider.rs | 93 | ⚠️ Partial |
| with_queue_validation(mut | provider.rs | 99 | ⚠️ Partial |
| with_queue_namespace(mut | provider.rs | 106 | ⚠️ Partial |
| with_default_max_retries(mut | provider.rs | 112 | ⚠️ Partial |
| with_default_timeout_seconds(mut | provider.rs | 118 | ⚠️ Partial |
| with_default_retry_backoff(mut | provider.rs | 124 | ⚠️ Partial |
| with_worker_threads(mut | provider.rs | 130 | ⚠️ Partial |
| with_publish_events(mut | provider.rs | 136 | ⚠️ Partial |
| with_event_topic(mut | provider.rs | 142 | ⚠️ Partial |
| new() | provider.rs | 229 | ✅ Complete |
| with_max_retries(mut | provider.rs | 241 | ⚠️ Partial |
| with_retry_backoff(mut | provider.rs | 247 | ⚠️ Partial |
| with_timeout_seconds(mut | provider.rs | 253 | ⚠️ Partial |
| with_priority(mut | provider.rs | 259 | ⚠️ Partial |
| with_metadata(mut | provider.rs | 265 | ⚠️ Partial |
| with_correlation_id(mut | provider.rs | 278 | ⚠️ Partial |


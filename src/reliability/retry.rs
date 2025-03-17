use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::http::StatusCode;
use axum::response::Response;
use futures::future::BoxFuture;
use pin_project::pin_project;
use rand::Rng;
use tower::{Layer, Service};
use tracing::{debug, info, warn, error};

/// Layer for adding retry capability to services
#[derive(Clone, Debug)]
pub struct RetryLayer {
    /// Maximum number of retry attempts
    max_attempts: u32,
    /// Base delay between retries
    base_delay: Duration,
    /// Maximum delay between retries
    max_delay: Duration,
    /// Whether to use exponential backoff
    use_exponential_backoff: bool,
    /// Status codes that should trigger a retry
    retry_status_codes: Vec<u16>,
}

impl RetryLayer {
    /// Create a new retry layer
    pub fn new(
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        use_exponential_backoff: bool,
        retry_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            use_exponential_backoff,
            retry_status_codes,
        }
    }
}

impl<S> Layer<S> for RetryLayer {
    type Service = RetryService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RetryService {
            inner: service,
            max_attempts: self.max_attempts,
            base_delay: self.base_delay,
            max_delay: self.max_delay,
            use_exponential_backoff: self.use_exponential_backoff,
            retry_status_codes: self.retry_status_codes.clone(),
        }
    }
}

/// Service implementing retry logic
#[derive(Clone, Debug)]
pub struct RetryService<S> {
    inner: S,
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    use_exponential_backoff: bool,
    retry_status_codes: Vec<u16>,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for RetryService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static + Clone,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = RetryFuture<S, ReqBody, ResBody>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Clone the service and request to allow for retries
        let clone_service = self.inner.clone();
        let service = std::mem::replace(&mut self.inner, clone_service);

        RetryFuture {
            state: RetryState::Initial {
                service,
                request: req,
                attempt: 1,
                max_attempts: self.max_attempts,
                base_delay: self.base_delay,
                max_delay: self.max_delay,
                use_exponential_backoff: self.use_exponential_backoff,
                retry_status_codes: self.retry_status_codes.clone(),
            },
        }
    }
}

/// State machine for the retry process
enum RetryState<S, ReqBody, ResBody> {
    /// Initial state - first attempt
    Initial {
        service: S,
        request: axum::http::Request<ReqBody>,
        attempt: u32,
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        use_exponential_backoff: bool,
        retry_status_codes: Vec<u16>,
    },
    /// Service call is in progress
    Running(BoxFuture<'static, Result<Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>>),
    /// Waiting for the retry delay
    Waiting {
        service: S,
        request: axum::http::Request<ReqBody>,
        attempt: u32,
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        use_exponential_backoff: bool,
        retry_status_codes: Vec<u16>,
        delay_future: Pin<Box<tokio::time::Sleep>>,
    },
    /// Terminal state for Poll
    Empty,
}

/// Future that drives the retry process
#[pin_project]
pub struct RetryFuture<S, ReqBody, ResBody> {
    #[pin]
    state: RetryState<S, ReqBody, ResBody>,
}

impl<S, ReqBody, ResBody> Future for RetryFuture<S, ReqBody, ResBody>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + Clone + 'static,
    ResBody: Send + 'static,
{
    type Output = Result<Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut state = std::mem::replace(this.state, RetryState::Empty);

        loop {
            match state {
                RetryState::Initial {
                    mut service,
                    request,
                    attempt,
                    max_attempts,
                    base_delay,
                    max_delay,
                    use_exponential_backoff,
                    retry_status_codes,
                } => {
                    debug!("Starting request attempt {}/{}", attempt, max_attempts);
                    match service.poll_ready(cx) {
                        Poll::Ready(Ok(())) => {
                            let cloned_req = request.clone();
                            let future = Box::pin(async move {
                                service.call(cloned_req).await.map_err(Into::into)
                            });
                            state = RetryState::Running(future);
                            continue;
                        }
                        Poll::Ready(Err(e)) => {
                            return Poll::Ready(Err(e));
                        }
                        Poll::Pending => {
                            *this.state = RetryState::Initial {
                                service,
                                request,
                                attempt,
                                max_attempts,
                                base_delay,
                                max_delay,
                                use_exponential_backoff,
                                retry_status_codes,
                            };
                            return Poll::Pending;
                        }
                    }
                }
                RetryState::Running(mut future) => {
                    match future.as_mut().poll(cx) {
                        Poll::Ready(Ok(response)) => {
                            // Check if we should retry based on status code
                            let status = response.status();
                            let status_code = status.as_u16();
                            
                            // We're done if the response is success or if we don't retry for this status
                            if status.is_success() || !retry_status_codes.contains(&status_code) {
                                return Poll::Ready(Ok(response));
                            }
                            
                            // Extract state from the response to prepare for retry
                            if let RetryState::Empty = *this.state {
                                return Poll::Ready(Ok(response));
                            }
                            
                            if let RetryState::Initial {
                                service,
                                request,
                                attempt,
                                max_attempts,
                                base_delay,
                                max_delay,
                                use_exponential_backoff,
                                retry_status_codes,
                            } = std::mem::replace(this.state, RetryState::Empty)
                            {
                                if attempt >= max_attempts {
                                    warn!("Request failed with status {}, reached max retries ({})", status, max_attempts);
                                    return Poll::Ready(Ok(response));
                                }
                                
                                let next_attempt = attempt + 1;
                                
                                // Calculate delay with optional exponential backoff and jitter
                                let delay = if use_exponential_backoff {
                                    let backoff_factor = 2_u32.pow(next_attempt - 1) as f64;
                                    let delay_millis = base_delay.as_millis() as f64 * backoff_factor;
                                    let max_millis = max_delay.as_millis() as f64;
                                    let capped_millis = delay_millis.min(max_millis);
                                    
                                    // Add jitter (±10%)
                                    let jitter_factor = 0.9 + rand::thread_rng().gen::<f64>() * 0.2;
                                    let jittered_millis = capped_millis * jitter_factor;
                                    
                                    Duration::from_millis(jittered_millis as u64)
                                } else {
                                    base_delay
                                };
                                
                                info!(
                                    "Request failed with status {}. Retrying ({}/{}) after {:?}",
                                    status, next_attempt, max_attempts, delay
                                );
                                
                                let delay_future = Box::pin(tokio::time::sleep(delay));
                                
                                state = RetryState::Waiting {
                                    service,
                                    request,
                                    attempt: next_attempt,
                                    max_attempts,
                                    base_delay,
                                    max_delay,
                                    use_exponential_backoff,
                                    retry_status_codes,
                                    delay_future,
                                };
                                continue;
                            } else {
                                return Poll::Ready(Ok(response));
                            }
                        }
                        Poll::Ready(Err(err)) => {
                            // Extract state to prepare for retry after error
                            if let RetryState::Empty = *this.state {
                                return Poll::Ready(Err(err));
                            }
                            
                            if let RetryState::Initial {
                                service,
                                request,
                                attempt,
                                max_attempts,
                                base_delay,
                                max_delay,
                                use_exponential_backoff,
                                retry_status_codes,
                            } = std::mem::replace(this.state, RetryState::Empty)
                            {
                                if attempt >= max_attempts {
                                    error!("Request failed with error, reached max retries ({}): {:?}", max_attempts, err);
                                    return Poll::Ready(Err(err));
                                }
                                
                                let next_attempt = attempt + 1;
                                
                                // Calculate delay with exponential backoff and jitter
                                let delay = if use_exponential_backoff {
                                    let backoff_factor = 2_u32.pow(next_attempt - 1) as f64;
                                    let delay_millis = base_delay.as_millis() as f64 * backoff_factor;
                                    let max_millis = max_delay.as_millis() as f64;
                                    let capped_millis = delay_millis.min(max_millis);
                                    
                                    // Add jitter (±10%)
                                    let jitter_factor = 0.9 + rand::thread_rng().gen::<f64>() * 0.2;
                                    let jittered_millis = capped_millis * jitter_factor;
                                    
                                    Duration::from_millis(jittered_millis as u64)
                                } else {
                                    base_delay
                                };
                                
                                warn!(
                                    "Request failed with error. Retrying ({}/{}) after {:?}: {:?}",
                                    next_attempt, max_attempts, delay, err
                                );
                                
                                let delay_future = Box::pin(tokio::time::sleep(delay));
                                
                                state = RetryState::Waiting {
                                    service,
                                    request,
                                    attempt: next_attempt,
                                    max_attempts,
                                    base_delay,
                                    max_delay,
                                    use_exponential_backoff,
                                    retry_status_codes,
                                    delay_future,
                                };
                                continue;
                            } else {
                                return Poll::Ready(Err(err));
                            }
                        }
                        Poll::Pending => {
                            *this.state = RetryState::Running(future);
                            return Poll::Pending;
                        }
                    }
                }
                RetryState::Waiting {
                    service,
                    request,
                    attempt,
                    max_attempts,
                    base_delay,
                    max_delay,
                    use_exponential_backoff,
                    retry_status_codes,
                    mut delay_future,
                } => {
                    match delay_future.as_mut().poll(cx) {
                        Poll::Ready(()) => {
                            debug!("Retry delay complete, starting attempt {}/{}", attempt, max_attempts);
                            state = RetryState::Initial {
                                service,
                                request,
                                attempt,
                                max_attempts,
                                base_delay,
                                max_delay,
                                use_exponential_backoff,
                                retry_status_codes,
                            };
                            continue;
                        }
                        Poll::Pending => {
                            *this.state = RetryState::Waiting {
                                service,
                                request,
                                attempt,
                                max_attempts,
                                base_delay,
                                max_delay,
                                use_exponential_backoff,
                                retry_status_codes,
                                delay_future,
                            };
                            return Poll::Pending;
                        }
                    }
                }
                RetryState::Empty => {
                    panic!("RetryFuture polled after completion");
                }
            }
        }
    }
} 
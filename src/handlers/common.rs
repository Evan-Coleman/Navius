// Common handlers for the Navius framework

/// A basic example handler to demonstrate routing.
pub async fn hello_world() -> &'static str {
    "Hello from Navius!"
}

/// A health check handler for monitoring.
pub async fn health_check() -> &'static str {
    "OK"
}

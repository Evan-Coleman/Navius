use crate::core::error::AppError;

// Database health check removed for stability
// Instead, we return a simple success response
pub async fn check_health() -> Result<(), AppError> {
    // No database to check, so always report success
    Ok(())
}

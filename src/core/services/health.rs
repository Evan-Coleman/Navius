use crate::core::error::AppError;

// Instead, we return a simple success response
pub async fn check_health() -> Result<(), AppError> {
    Ok(())
}

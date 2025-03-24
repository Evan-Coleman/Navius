use crate::core::database::DatabaseConnection;
use crate::core::error::AppError;
use std::sync::Arc;

pub async fn check_health(db: Arc<dyn DatabaseConnection>) -> Result<(), AppError> {
    db.ping().await
}

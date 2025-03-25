use crate::core::database::PgPool;
use crate::core::error::AppError;
use std::sync::Arc;

pub async fn check_health(db: Arc<dyn PgPool>) -> Result<(), AppError> {
    db.ping().await
}

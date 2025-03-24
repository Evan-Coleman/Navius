use crate::core::error::error_types::AppError;

#[async_trait]
impl MockPool {
    pub async fn ping(&self) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn begin(&self) -> Result<MockTransaction, AppError> {
        Ok(MockTransaction {})
    }
}

#[async_trait]
impl MockTransaction {
    pub async fn commit(self) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), AppError> {
        Ok(())
    }
}

use crate::app::database::repositories::pet_repository::Pet;
use crate::app::services::dto::{CreatePetDto, UpdatePetDto};
use crate::core::error::AppError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait PetService: Send + Sync {
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, AppError>;
    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, AppError>;
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, AppError>;
    async fn delete_pet(&self, id: Uuid) -> Result<(), AppError>;
}

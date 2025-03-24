use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::core::database::repositories::{Pet, PetRepository};
use crate::services::error::ServiceError;

// DTOs for pets
#[derive(Debug)]
pub struct CreatePetDto {
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

#[derive(Debug)]
pub struct UpdatePetDto {
    pub name: Option<String>,
    pub pet_type: Option<String>,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

#[async_trait]
pub trait IPetService: Send + Sync {
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError>;
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError>;
    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError>;
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError>;
    async fn delete_pet(&self, id: Uuid) -> Result<(), ServiceError>;
}

pub struct PetService {
    pet_repository: Arc<dyn PetRepository>,
}

impl PetService {
    pub fn new(pet_repository: Arc<dyn PetRepository>) -> Self {
        Self { pet_repository }
    }
}

#[async_trait]
impl IPetService for PetService {
    #[instrument(skip(self))]
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError> {
        debug!("Getting all pets");
        let pets = self
            .pet_repository
            .find_all()
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(pets)
    }

    #[instrument(skip(self))]
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError> {
        debug!("Getting pet by id: {}", id);
        let pet = self
            .pet_repository
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        match pet {
            Some(pet) => Ok(pet),
            None => Err(ServiceError::PetNotFound),
        }
    }

    #[instrument(skip(self, dto), fields(pet_name = %dto.name, pet_type = %dto.pet_type))]
    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError> {
        debug!("Creating pet: {} ({})", dto.name, dto.pet_type);

        // Create pet in the repository
        let pet = self
            .pet_repository
            .create(dto.name, dto.pet_type, dto.breed, dto.age)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(pet)
    }

    #[instrument(skip(self, dto), fields(pet_id = %id))]
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError> {
        debug!("Updating pet with id: {}", id);

        // Get existing pet to ensure it exists
        let _ = self.get_pet_by_id(id).await?;

        // Update the pet in the repository
        let updated_pet = self
            .pet_repository
            .update(
                id,
                dto.name.unwrap_or_default(),
                dto.pet_type.unwrap_or_default(),
                dto.breed,
                dto.age,
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        match updated_pet {
            Some(pet) => Ok(pet),
            None => Err(ServiceError::PetNotFound),
        }
    }

    #[instrument(skip(self))]
    async fn delete_pet(&self, id: Uuid) -> Result<(), ServiceError> {
        debug!("Deleting pet with id: {}", id);

        // Delete pet
        let deleted = self
            .pet_repository
            .delete(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::PetNotFound)
        }
    }
}

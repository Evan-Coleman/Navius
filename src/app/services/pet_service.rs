use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::app::database::repositories::pet_repository::{Pet as RepositoryPet, PetRepository};
use crate::app::services::dto::{CreatePetDto, UpdatePetDto};
use crate::app::services::error::ServiceError;
use crate::core::database::connection::DatabaseConnection;
use crate::core::{
    database::{models::pet::Pet as CorePet, repositories::PetRepository as CorePetRepository},
    error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trait for Pet service operations
#[async_trait]
pub trait IPetService: Send + Sync {
    /// Get all pets
    async fn get_all_pets(&self) -> Result<Vec<RepositoryPet>, AppError>;

    /// Get a pet by ID
    async fn get_pet_by_id(&self, id: Uuid) -> Result<RepositoryPet, AppError>;

    /// Create a new pet
    async fn create_pet(&self, dto: CreatePetDto) -> Result<RepositoryPet, AppError>;

    /// Update an existing pet
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<RepositoryPet, AppError>;

    /// Delete a pet
    async fn delete_pet(&self, id: Uuid) -> Result<(), AppError>;
}

/// Service for managing pets
pub struct PetService<R: PetRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: PetRepository + ?Sized> PetService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: PetRepository + ?Sized + Send + Sync> IPetService for PetService<R> {
    async fn get_all_pets(&self) -> Result<Vec<RepositoryPet>, AppError> {
        self.repository.get_pets().await
    }

    async fn get_pet_by_id(&self, id: Uuid) -> Result<RepositoryPet, AppError> {
        self.repository.get_pet_by_id(id).await
    }

    async fn create_pet(&self, dto: CreatePetDto) -> Result<RepositoryPet, AppError> {
        let pet: RepositoryPet = dto.into();
        self.repository.create_pet(pet).await
    }

    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<RepositoryPet, AppError> {
        let pet: RepositoryPet = dto.into();
        self.repository.update_pet(id, pet).await
    }

    async fn delete_pet(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete_pet(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::database::repositories::pet_repository::tests::MockPetRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_all_pets() {
        let repository = Arc::new(MockPetRepository::new(vec![]));
        let service = PetService::new(repository);
        let result = service.get_all_pets().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pet_by_id() {
        let id = Uuid::new_v4();
        let pet = RepositoryPet {
            id,
            name: "Test Pet".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repository = Arc::new(MockPetRepository::new(vec![pet.clone()]));
        let service = PetService::new(repository);
        let result = service.get_pet_by_id(id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, pet.id);
    }

    #[tokio::test]
    async fn test_create_pet() {
        let repository = Arc::new(MockPetRepository::new(vec![]));
        let service = PetService::new(repository);
        let dto = CreatePetDto {
            name: "Test Pet".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some(3),
        };
        let result = service.create_pet(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_pet() {
        let id = Uuid::new_v4();
        let pet = RepositoryPet {
            id,
            name: "Test Pet".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repository = Arc::new(MockPetRepository::new(vec![pet.clone()]));
        let service = PetService::new(repository);
        let dto = UpdatePetDto {
            name: Some("Updated Pet".to_string()),
            pet_type: None,
            breed: None,
            age: None,
        };
        let result = service.update_pet(id, dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_pet() {
        let id = Uuid::new_v4();
        let pet = RepositoryPet {
            id,
            name: "Test Pet".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repository = Arc::new(MockPetRepository::new(vec![pet]));
        let service = PetService::new(repository);
        let result = service.delete_pet(id).await;
        assert!(result.is_ok());
    }
}

// Add conversion implementations to address the type mismatches
impl From<RepositoryPet> for Pet {
    fn from(repo_pet: RepositoryPet) -> Self {
        Self {
            id: repo_pet.id,
            name: repo_pet.name,
            pet_type: repo_pet.pet_type,
            breed: repo_pet.breed,
            age: repo_pet.age,
            created_at: repo_pet.created_at,
            updated_at: repo_pet.updated_at,
        }
    }
}

impl From<Pet> for RepositoryPet {
    fn from(svc_pet: Pet) -> Self {
        Self {
            id: svc_pet.id,
            name: svc_pet.name,
            pet_type: svc_pet.pet_type,
            breed: svc_pet.breed,
            age: svc_pet.age,
            created_at: svc_pet.created_at,
            updated_at: svc_pet.updated_at,
        }
    }
}

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
    database::{models::Pet, repositories::PetRepository},
    error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trait for Pet service operations
#[async_trait]
pub trait IPetService: Send + Sync {
    /// Get all pets
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError>;

    /// Get a pet by ID
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError>;

    /// Create a new pet
    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError>;

    /// Update an existing pet
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError>;

    /// Delete a pet
    async fn delete_pet(&self, id: Uuid) -> Result<bool, AppError>;
}

/// Service for managing pets
pub struct PetService<R: PetRepository + ?Sized> {
    repository: Arc<R>,
}

impl<R: PetRepository + ?Sized> PetService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
        let repo_pets = self.repository.find_all().await?;
        Ok(repo_pets.into_iter().map(Pet::from).collect())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Pet, AppError> {
        let repo_pet = self.repository.find_by_id(id).await?;
        Ok(Pet::from(repo_pet))
    }

    pub async fn create(&self, pet: Pet) -> Result<Pet, AppError> {
        let repo_pet = self.repository.create(pet.into()).await?;
        Ok(Pet::from(repo_pet))
    }

    pub async fn update(&self, pet: Pet) -> Result<Pet, AppError> {
        let repo_pet = self.repository.update(pet.into()).await?;
        Ok(Pet::from(repo_pet))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        self.repository.delete(id).await
    }
}

#[async_trait]
impl<R: PetRepository + ?Sized> IPetService for PetService<R> {
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError> {
        self.find_all().await
    }

    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError> {
        self.find_by_id(id).await
    }

    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError> {
        self.create(Pet {
            id: Uuid::new_v4(),
            name: dto.name,
            pet_type: dto.pet_type,
            breed: dto.breed,
            age: dto.age.unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
    }

    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError> {
        self.update(Pet {
            id,
            name: dto.name,
            pet_type: dto.pet_type,
            breed: dto.breed,
            age: dto.age.unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
    }

    async fn delete_pet(&self, id: Uuid) -> Result<bool, AppError> {
        self.delete(id).await
    }
}

pub struct MockPetRepository {
    pets: Arc<RwLock<Vec<Pet>>>,
}

impl MockPetRepository {
    pub fn new(pets: Vec<Pet>) -> Self {
        Self {
            pets: Arc::new(RwLock::new(pets)),
        }
    }
}

impl Default for MockPetRepository {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[async_trait]
impl PetRepository for MockPetRepository {
    async fn find_all(&self) -> Result<Vec<RepositoryPet>, AppError> {
        let pets = self.pets.read().unwrap();
        let repo_pets = pets
            .iter()
            .map(|p| RepositoryPet::from(p.clone()))
            .collect();
        Ok(repo_pets)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<RepositoryPet, AppError> {
        self.pets
            .read()
            .unwrap()
            .iter()
            .find(|p| p.id == id)
            .map(|p| RepositoryPet::from(p.clone()))
            .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", id)))
    }

    async fn create(&self, pet: RepositoryPet) -> Result<RepositoryPet, AppError> {
        let svc_pet = Pet::from(pet.clone());
        let mut pets = self.pets.write().unwrap();
        pets.push(svc_pet);
        Ok(pet)
    }

    async fn update(&self, pet: RepositoryPet) -> Result<RepositoryPet, AppError> {
        let svc_pet = Pet::from(pet.clone());
        let mut pets = self.pets.write().unwrap();
        if let Some(index) = pets.iter().position(|p| p.id == svc_pet.id) {
            pets[index] = svc_pet;
            Ok(pet)
        } else {
            Err(AppError::NotFound(format!(
                "Pet with id {} not found",
                pet.id
            )))
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let mut pets = self.pets.write().unwrap();
        if let Some(index) = pets.iter().position(|p| p.id == id) {
            pets.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_pet_service() {
        let pet = Pet {
            id: Uuid::new_v4(),
            name: "Fluffy".to_string(),
            pet_type: "Cat".to_string(),
            breed: Some("Persian".to_string()),
            age: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repository = Arc::new(MockPetRepository::new(vec![pet.clone()]));
        let service = PetService::new(repository);

        // Test find_all
        let pets = service.get_all_pets().await.unwrap();
        assert_eq!(pets.len(), 1);
        assert_eq!(pets[0].name, "Fluffy");

        // Test find_by_id
        let found_pet = service.get_pet_by_id(pet.id).await.unwrap();
        assert_eq!(found_pet.id, pet.id);

        // Test create
        let new_pet = Pet {
            id: Uuid::new_v4(),
            name: "Buddy".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Golden Retriever".to_string()),
            age: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let created_pet = service
            .create_pet(CreatePetDto {
                name: new_pet.name.clone(),
                pet_type: Some(new_pet.pet_type.clone()),
                breed: new_pet.breed.clone(),
                age: Some(new_pet.age),
            })
            .await
            .unwrap();
        assert_eq!(created_pet.name, "Buddy");

        // Test update
        let mut updated_pet = created_pet.clone();
        updated_pet.name = "Buddy Jr.".to_string();
        let result = service
            .update_pet(
                updated_pet.id,
                UpdatePetDto {
                    name: Some(updated_pet.name.clone()),
                    pet_type: Some(updated_pet.pet_type.clone()),
                    breed: updated_pet.breed.clone(),
                    age: Some(updated_pet.age),
                },
            )
            .await
            .unwrap();
        assert_eq!(result.name, "Buddy Jr.");

        // Test delete
        let deleted = service.delete_pet(created_pet.id).await.unwrap();
        assert!(deleted);
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

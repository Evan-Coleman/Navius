use crate::app::api::pet_core::{CreatePetRequest, UpdatePetRequest};
use crate::app::database::repositories::pet_repository::Pet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a pet
#[derive(Debug, Clone)]
pub struct CreatePetDto {
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

/// DTO for updating a pet
#[derive(Debug, Clone)]
pub struct UpdatePetDto {
    pub name: Option<String>,
    pub pet_type: Option<String>,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

impl From<CreatePetRequest> for CreatePetDto {
    fn from(req: CreatePetRequest) -> Self {
        Self {
            name: req.name,
            pet_type: req.pet_type,
            breed: req.breed,
            age: req.age,
        }
    }
}

impl From<UpdatePetRequest> for UpdatePetDto {
    fn from(req: UpdatePetRequest) -> Self {
        Self {
            name: req.name,
            pet_type: req.pet_type,
            breed: req.breed,
            age: req.age,
        }
    }
}

impl From<CreatePetDto> for Pet {
    fn from(dto: CreatePetDto) -> Self {
        Self {
            id: Uuid::new_v4(), // Generate a new UUID
            name: dto.name,
            pet_type: dto.pet_type,
            breed: dto.breed,
            age: dto.age,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl From<UpdatePetDto> for Pet {
    fn from(dto: UpdatePetDto) -> Self {
        Self {
            id: Uuid::new_v4(), // Will be set by caller
            name: dto.name.unwrap_or_default(),
            pet_type: dto.pet_type.unwrap_or_default(),
            breed: dto.breed,
            age: dto.age,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

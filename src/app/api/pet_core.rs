/// This file implements the core Pet database API.
/// It provides the main functionality for managing pets in the application.
/// This is a CORE implementation and should not be removed.
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    app::{
        database::repositories::pet_repository::{
            Pet as RepositoryPet, PetRepository, PgPetRepository as DefaultPetRepository,
        },
        services::{
            CreatePetDto, ServiceError, UpdatePetDto,
            pet_service::{IPetService, Pet as ServicePet, PetService},
        },
    },
    core::services::Services,
    core::{error::AppError, error::Result, router::AppState},
};

// Helper module for UUID serialization
mod uuid_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

// API response for pet operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PetResponse {
    pub id: Uuid,
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ServicePet> for PetResponse {
    fn from(pet: ServicePet) -> Self {
        Self {
            id: pet.id,
            name: pet.name,
            pet_type: pet.pet_type.unwrap_or_default(),
            breed: pet.breed,
            age: Some(pet.age),
            created_at: "".to_string(), // Service Pet doesn't have timestamp fields
            updated_at: "".to_string(),
        }
    }
}

impl From<RepositoryPet> for PetResponse {
    fn from(pet: RepositoryPet) -> Self {
        Self {
            id: pet.id,
            name: pet.name,
            pet_type: pet.pet_type,
            breed: Some(pet.breed),
            age: Some(pet.age),
            created_at: pet.created_at.to_rfc3339(),
            updated_at: pet.updated_at.to_rfc3339(),
        }
    }
}

// DTO for creating a new pet
#[derive(Debug, Deserialize)]
pub struct CreatePetRequest {
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

impl From<CreatePetRequest> for CreatePetDto {
    fn from(req: CreatePetRequest) -> Self {
        Self {
            name: req.name,
            pet_type: Some(req.pet_type),
            breed: req.breed,
            age: req.age,
        }
    }
}

// DTO for updating an existing pet
#[derive(Debug, Deserialize)]
pub struct UpdatePetRequest {
    pub name: Option<String>,
    pub pet_type: Option<String>,
    pub breed: Option<String>,
    pub age: Option<i32>,
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

// Configure pet API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/pets", get(get_all_pets))
        .route("/pets", post(create_pet))
        .route("/pets/:id", get(get_pet))
        .route("/pets/:id", put(update_pet))
        .route("/pets/:id", delete(delete_pet))
}

// Get all pets endpoint
#[instrument(skip(state))]
async fn get_all_pets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<PetResponse>>> {
    info!("Fetching all pets");

    let pet_service = get_pet_service(state)?;

    let pets = pet_service.get_all_pets().await.map_err(|e| {
        error!("Failed to fetch pets: {}", e);
        AppError::from(e)
    })?;

    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok());

    let pet_responses: Vec<PetResponse> = pets.into_iter().map(PetResponse::from).collect();

    let limited_responses = match limit {
        Some(limit) if limit > 0 => pet_responses.into_iter().take(limit).collect(),
        _ => pet_responses,
    };

    info!("Successfully fetched {} pets", limited_responses.len());
    Ok(Json(limited_responses))
}

// Get pet by ID endpoint
#[instrument(skip(state), fields(pet_id = %id))]
pub async fn get_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PetResponse>> {
    info!("Fetching pet with ID: {}", id);

    let pet_service = get_pet_service(state)?;

    let pet = pet_service.get_pet_by_id(id).await.map_err(|e| {
        error!("Failed to get pet with ID {}: {}", id, e);
        AppError::from(e)
    })?;

    let response = PetResponse::from(pet);
    info!("Successfully fetched pet with ID: {}", id);
    Ok(Json(response))
}

// Create pet endpoint
#[instrument(skip(state, request), fields(pet_name = %request.name))]
pub async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePetRequest>,
) -> Result<Json<PetResponse>> {
    info!("Creating new pet: {}", request.name);

    let pet_service = get_pet_service(state)?;
    let dto = CreatePetDto::from(request);

    let pet = pet_service.create_pet(dto).await.map_err(|e| {
        error!("Failed to create pet: {}", e);
        AppError::from(e)
    })?;

    let response = PetResponse::from(pet);
    info!("Successfully created pet with ID: {}", response.id);
    Ok(Json(response))
}

// Update pet endpoint
#[instrument(skip(state, request), fields(pet_id = %id))]
pub async fn update_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdatePetRequest>,
) -> Result<Json<PetResponse>> {
    info!("Updating pet with ID: {}", id);

    let pet_service = get_pet_service(state)?;
    let dto = UpdatePetDto::from(request);

    let pet = pet_service.update_pet(id, dto).await.map_err(|e| {
        error!("Failed to update pet with ID {}: {}", id, e);
        AppError::from(e)
    })?;

    let response = PetResponse::from(pet);
    info!("Successfully updated pet with ID: {}", id);
    Ok(Json(response))
}

// Delete pet endpoint
#[instrument(skip(state), fields(pet_id = %id))]
pub async fn delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    info!("Deleting pet with ID: {}", id);

    let pet_service = get_pet_service(state)?;

    pet_service.delete_pet(id).await.map_err(|e| {
        error!("Failed to delete pet with ID {}: {}", id, e);
        AppError::from(e)
    })?;

    info!("Successfully deleted pet with ID: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

// Helper function to get pet service from state
fn get_pet_service(state: Arc<AppState>) -> Result<Arc<PetService<dyn PetRepository>>> {
    // Try to get the service from the registry first
    if let Some(service) = state
        .service_registry
        .get::<PetService<dyn PetRepository>>("pet_service")
    {
        return Ok(service);
    }

    // If not in registry, use the one from services if available
    match &state.services {
        Some(services) => {
            if let Some(pet_service) = services.get_pet_service() {
                // We need to downcast the IPetService trait object to the concrete PetService
                return Ok(Arc::new(PetService::new(pet_service.db_pool.clone())));
            }
        }
        None => {}
    }

    // As fallback, create a new pet service if we have a database pool
    if let Some(db_pool) = &state.db_pool {
        return Ok(Arc::new(PetService::new(db_pool.clone())));
    }

    Err(AppError::internal_server_error("Pet service not available"))
}

/// This file provides an alternative implementation of the Pet database API.
/// It will eventually replace the implementation in pet_core.rs.
/// This is a CORE implementation and should not be removed.
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::app::database::repositories::pet_repository::{
    Pet, PetRepository, PgPetRepository as DefaultPetRepository,
};
use crate::app::services::{CreatePetDto, IPetService, PetService, UpdatePetDto};
use crate::core::error::{AppError, Result};
use crate::core::router::AppState;
use crate::core::services::ServiceRegistry;

// DTO for returning pet data in API responses
#[derive(Debug, Serialize)]
pub struct PetResponse {
    pub id: Uuid,
    pub name: String,
    pub pet_type: String,
    pub breed: String,
    pub age: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<pet_service::Pet> for PetResponse {
    fn from(pet: pet_service::Pet) -> Self {
        Self {
            id: pet.id,
            name: pet.name,
            pet_type: pet.pet_type.unwrap_or_default(),
            breed: pet.breed.unwrap_or_default(),
            age: pet.age.unwrap_or_default(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl From<pet_repository::Pet> for PetResponse {
    fn from(pet: pet_repository::Pet) -> Self {
        Self {
            id: pet.id,
            name: pet.name,
            pet_type: pet.pet_type,
            breed: pet.breed,
            age: pet.age,
            created_at: format_timestamp(pet.created_at),
            updated_at: format_timestamp(pet.updated_at),
        }
    }
}

// Helper to format timestamps consistently
fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
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

// Configure pet database routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/petdb", get(get_pets))
        .route("/petdb", post(create_pet))
        .route("/petdb/:id", get(get_pet))
        .route("/petdb/:id", put(update_pet))
        .route("/petdb/:id", delete(delete_pet))
}

// Get all pets endpoint
#[instrument(skip(state))]
pub async fn get_pets(State(state): State<Arc<AppState>>) -> Result<Json<Vec<PetResponse>>> {
    info!("Fetching all pets from pet_db API");

    let pet_service = get_pet_service(state)?;

    let pets = pet_service.get_all_pets().await.map_err(|e| {
        error!("Failed to fetch pets: {}", e);
        AppError::from(e)
    })?;

    let responses = pets.into_iter().map(|p| p.into()).collect();
    info!("Successfully retrieved pets from database");

    Ok(Json(responses))
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
#[instrument(skip(state, dto), fields(pet_name = %dto.name))]
pub async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreatePetRequest>,
) -> Result<Json<PetResponse>> {
    info!("Creating new pet: {}", dto.name);

    let pet_service = get_pet_service(state)?;

    let pet = pet_service.create_pet(dto.into()).await.map_err(|e| {
        error!("Failed to create pet: {}", e);
        AppError::from(e)
    })?;

    info!("Successfully created pet with ID: {}", pet.id);
    Ok(Json(pet.into()))
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

// Helper function to get pet service
fn get_pet_service(state: Arc<AppState>) -> Result<Arc<PetService<dyn PetRepository>>> {
    // Try to get the service from the registry first
    if let Some(service) = state
        .service_registry
        .get::<PetService<dyn PetRepository>>("pet_service")
    {
        return Ok(service);
    }

    // Get the database connection
    let db_pool = state.db_pool.clone().ok_or_else(|| {
        error!("Database connection not available");
        AppError::internal_server_error("Database connection not available")
    })?;

    // Create and return the service directly with the database pool
    let service = Arc::new(PetService::new(db_pool.as_ref().clone()));
    info!("Created new PetService instance for pet_db API");
    Ok(service)
}

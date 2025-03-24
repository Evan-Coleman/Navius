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
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::app::database::repositories::pet_repository::{
    Pet, PetRepository, PgPetRepository as DefaultPetRepository,
};
use crate::app::services::pet_service::{CreatePetDto, IPetService, PetService, UpdatePetDto};
use crate::core::error::AppError;
use crate::core::router::AppState;
use crate::core::services::ServiceRegistry;

// DTO for returning pet data in API responses
#[derive(Debug, Serialize)]
pub struct PetResponse {
    pub id: String,
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Pet> for PetResponse {
    fn from(pet: Pet) -> Self {
        Self {
            id: pet.id.to_string(),
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
            pet_type: req.pet_type,
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
pub async fn get_pets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PetResponse>>, AppError> {
    let pet_service = get_pet_service(state)
        .map_err(|(status, msg)| AppError::internal_server_error(format!("{}: {}", status, msg)))?;

    match pet_service.get_all_pets().await {
        Ok(pets) => Ok(Json(pets.into_iter().map(|p| p.into()).collect())),
        Err(e) => Err(AppError::from(e)),
    }
}

// Get pet by ID endpoint
#[instrument(skip(state))]
pub async fn get_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = get_pet_service(state)
        .map_err(|(status, msg)| AppError::internal_server_error(format!("{}: {}", status, msg)))?;

    let id = Uuid::parse_str(&id).map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    match pet_service.get_pet_by_id(id).await {
        Ok(pet) => Ok(Json(pet.into())),
        Err(e) => Err(AppError::from(e)),
    }
}

// Create pet endpoint
#[instrument(skip(state))]
pub async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreatePetRequest>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = get_pet_service(state)
        .map_err(|(status, msg)| AppError::internal_server_error(format!("{}: {}", status, msg)))?;

    match pet_service.create_pet(dto.into()).await {
        Ok(pet) => Ok(Json(pet.into())),
        Err(e) => Err(AppError::from(e)),
    }
}

// Update pet endpoint
#[instrument(skip(state))]
pub async fn update_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(dto): Json<UpdatePetRequest>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = get_pet_service(state)
        .map_err(|(status, msg)| AppError::internal_server_error(format!("{}: {}", status, msg)))?;

    let id = Uuid::parse_str(&id).map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    match pet_service.update_pet(id, dto.into()).await {
        Ok(pet) => Ok(Json(pet.into())),
        Err(e) => Err(AppError::from(e)),
    }
}

// Delete pet endpoint
#[instrument(skip(state))]
pub async fn delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let pet_service = get_pet_service(state)
        .map_err(|(status, msg)| AppError::internal_server_error(format!("{}: {}", status, msg)))?;

    let id = Uuid::parse_str(&id).map_err(|_| AppError::bad_request("Invalid UUID format"))?;

    match pet_service.delete_pet(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(AppError::from(e)),
    }
}

// Helper function to get pet service
fn get_pet_service(state: Arc<AppState>) -> Result<Arc<PetService>, (StatusCode, String)> {
    if let Some(service) = state.service_registry.get::<PetService>("pet_service") {
        return Ok(service.clone());
    }

    // Get the database pool
    let db_pool = match &state.db_pool {
        Some(pool) => pool.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database pool not found".to_string(),
            ));
        }
    };

    // Create a repository from the pool
    let pet_repository =
        Arc::new(DefaultPetRepository::new(db_pool.clone())) as Arc<dyn PetRepository>;

    // Create and return the service
    Ok(Arc::new(PetService::new(pet_repository)))
}

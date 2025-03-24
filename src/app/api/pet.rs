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
use tracing::instrument;
use uuid::Uuid;

use crate::{
    app::{
        database::repositories::pet_repository::{
            Pet, PetRepository, PgPetRepository as DefaultPetRepository,
        },
        services::{
            CreatePetDto, ServiceError, UpdatePetDto,
            pet_service::{IPetService, PetService},
        },
    },
    core::{error::AppError, router::AppState},
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

impl From<Pet> for PetResponse {
    fn from(pet: Pet) -> Self {
        Self {
            id: pet.id,
            name: pet.name,
            pet_type: pet.pet_type,
            breed: pet.breed,
            age: pet.age,
            created_at: pet.created_at.to_rfc3339(),
            updated_at: pet.updated_at.to_rfc3339(),
        }
    }
}

// Create pet request
#[derive(Debug, Deserialize, Serialize)]
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

// Update pet request
#[derive(Debug, Deserialize, Serialize)]
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

// Configure pet routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/petdb", get(get_all_pets))
        .route("/petdb", post(create_pet))
        .route("/petdb/{id}", get(get_pet))
        .route("/petdb/{id}", put(update_pet))
        .route("/petdb/{id}", delete(delete_pet))
}

// Map service errors to HTTP status codes
fn map_service_error(err: ServiceError) -> (StatusCode, String) {
    match err {
        ServiceError::PetNotFound => (StatusCode::NOT_FOUND, "Pet not found".to_string()),
        ServiceError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
        ServiceError::DatabaseError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", msg),
        ),
        ServiceError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
        ServiceError::UsernameExists => {
            (StatusCode::CONFLICT, "Username already exists".to_string())
        }
        ServiceError::EmailExists => (StatusCode::CONFLICT, "Email already exists".to_string()),
    }
}

// Get all pets
#[instrument(skip(state))]
async fn get_all_pets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<PetResponse>>, (StatusCode, String)> {
    // Get pet service
    let pet_service = get_pet_service(state)?;

    // Check for species query parameter
    if let Some(species) = params.get("species") {
        // Find by species
        let pets = pet_service
            .find_by_species(species)
            .await
            .map_err(map_service_error)?;

        // Convert to response format
        let responses = pets.into_iter().map(PetResponse::from).collect();
        return Ok(Json(responses));
    }

    // Get all pets
    let pets = pet_service
        .get_all_pets()
        .await
        .map_err(map_service_error)?;

    // Convert to response format
    let responses = pets.into_iter().map(PetResponse::from).collect();

    Ok(Json(responses))
}

// Get a pet by ID
#[instrument(skip(state))]
async fn get_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<Json<PetResponse>, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };

    // Get pet service
    let pet_service = get_pet_service(state)?;

    // Get pet by ID
    let pet = pet_service
        .get_pet_by_id(id)
        .await
        .map_err(map_service_error)?;

    // Convert to response format
    let response = PetResponse::from(pet);

    Ok(Json(response))
}

// Create a new pet
#[instrument(skip(state, request))]
async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePetRequest>,
) -> Result<(StatusCode, Json<PetResponse>), (StatusCode, String)> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if request.pet_type.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Pet type cannot be empty".to_string(),
        ));
    }

    if let Some(age) = request.age {
        if age < 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Age cannot be negative".to_string(),
            ));
        }
    }

    // Get pet service
    let pet_service = get_pet_service(state)?;

    // Create DTO
    let create_dto = CreatePetDto::from(request);

    // Create pet
    let pet = pet_service
        .create_pet(create_dto)
        .await
        .map_err(map_service_error)?;

    // Convert to response format
    let response = PetResponse::from(pet);

    Ok((StatusCode::CREATED, Json(response)))
}

// Update a pet
#[instrument(skip(state, request))]
async fn update_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdatePetRequest>,
) -> Result<Json<PetResponse>, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
        }
    }

    if let Some(ref pet_type) = request.pet_type {
        if pet_type.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Pet type cannot be empty".to_string(),
            ));
        }
    }

    if let Some(age) = request.age {
        if age < 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Age cannot be negative".to_string(),
            ));
        }
    }

    // Get pet service
    let pet_service = get_pet_service(state)?;

    // Create DTO
    let update_dto = UpdatePetDto::from(request);

    // Update pet
    let pet = pet_service
        .update_pet(id, update_dto)
        .await
        .map_err(map_service_error)?;

    // Convert to response format
    let response = PetResponse::from(pet);

    Ok(Json(response))
}

// Delete a pet
#[instrument(skip(state))]
async fn delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };

    // Get pet service
    let pet_service = get_pet_service(state)?;

    // Delete pet
    pet_service
        .delete_pet(id)
        .await
        .map_err(map_service_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// Helper function to get pet service
fn get_pet_service(state: Arc<AppState>) -> Result<Arc<PetService>, (StatusCode, String)> {
    // If the pet service is already set in the state, return it
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

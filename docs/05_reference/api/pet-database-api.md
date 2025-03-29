---
title: "Pet Database API Reference"
description: "Reference documentation for the Pet Database API, including CRUD operations and architecture details"
category: reference
tags:
  - api
  - database
  - pets
  - repository
related:
  - database-api.md
  - ../patterns/repository-pattern.md
  - ../../02_examples/database-service-example.md
last_updated: April 9, 2025
version: 1.0
---

# Pet Database API

## Overview

The Pet Database API provides a complete set of CRUD (Create, Read, Update, Delete) operations for managing pet records in the database. This API is built following clean architecture principles, with proper separation between database abstractions in the core layer and pet-specific implementations in the application layer.

This reference document details all endpoints, data structures, request/response formats, and integration patterns for working with pet data in Navius applications.

## Endpoints

### Get All Pets

Retrieves a list of all pets in the database.

**URL**: `/petdb`

**Method**: `GET`

**Query Parameters**:
- `limit` (optional): Maximum number of records to return (default: 100)
- `offset` (optional): Number of records to skip (default: 0)
- `species` (optional): Filter by pet species
- `sort` (optional): Sort field (e.g., `name`, `age`, `created_at`)
- `order` (optional): Sort order (`asc` or `desc`, default: `asc`)

**Authentication**: Public

**Response**: 

```json
{
  "data": [
    {
      "id": "uuid-string",
      "name": "Pet Name",
      "species": "Pet Species",
      "age": 5,
      "created_at": "2024-06-01T12:00:00.000Z",
      "updated_at": "2024-06-01T12:00:00.000Z"
    },
    ...
  ],
  "pagination": {
    "total": 150,
    "limit": 100,
    "offset": 0,
    "next_offset": 100
  }
}
```

**Status Codes**:
- `200 OK`: Successfully retrieved the list of pets
- `400 Bad Request`: Invalid query parameters
- `500 Internal Server Error`: Server encountered an error

**Curl Example**:

```bash
# Get all pets
curl -X GET http://localhost:3000/petdb

# Get pets with pagination and filtering
curl -X GET "http://localhost:3000/petdb?limit=10&offset=20&species=dog&sort=age&order=desc"
```

**Code Example**:

```rust
// Client-side request to get all pets
async fn get_all_pets(client: &Client, limit: Option<u32>, species: Option<&str>) -> Result<PetListResponse> {
    let mut req = client.get("http://localhost:3000/petdb");
    
    if let Some(limit) = limit {
        req = req.query(&[("limit", limit.to_string())]);
    }
    
    if let Some(species) = species {
        req = req.query(&[("species", species)]);
    }
    
    let response = req.send().await?;
    
    if response.status().is_success() {
        Ok(response.json::<PetListResponse>().await?)
    } else {
        Err(format!("Failed to get pets: {}", response.status()).into())
    }
}
```

### Get Pet by ID

Retrieves a specific pet by its unique identifier.

**URL**: `/petdb/:id`

**Method**: `GET`

**URL Parameters**:
- `id`: UUID of the pet to retrieve

**Authentication**: Public

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T12:00:00.000Z"
}
```

**Status Codes**:
- `200 OK`: Successfully retrieved the pet
- `400 Bad Request`: Invalid UUID format
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

**Curl Example**:

```bash
curl -X GET http://localhost:3000/petdb/550e8400-e29b-41d4-a716-446655440000
```

**Code Example**:

```rust
// Client-side request to get a pet by ID
async fn get_pet_by_id(client: &Client, id: &str) -> Result<Pet> {
    let response = client
        .get(&format!("http://localhost:3000/petdb/{}", id))
        .send()
        .await?;
    
    match response.status() {
        StatusCode::OK => Ok(response.json::<Pet>().await?),
        StatusCode::NOT_FOUND => Err("Pet not found".into()),
        _ => Err(format!("Failed to get pet: {}", response.status()).into())
    }
}
```

### Create Pet

Creates a new pet in the database.

**URL**: `/petdb`

**Method**: `POST`

**Authentication**: Required

**Request Headers**:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body**:

```json
{
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5
}
```

**Validation Rules**:
- `name`: Required, cannot be empty, maximum 50 characters
- `species`: Required, cannot be empty
- `age`: Required, must be non-negative, must be realistic (0-100)

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T12:00:00.000Z"
}
```

**Status Codes**:
- `201 Created`: Successfully created the pet
- `400 Bad Request`: Validation error in the request data
- `401 Unauthorized`: Authentication required
- `500 Internal Server Error`: Server encountered an error

**Curl Example**:

```bash
curl -X POST http://localhost:3000/petdb \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{"name":"Fluffy","species":"cat","age":3}'
```

**Code Example**:

```rust
// Client-side request to create a pet
async fn create_pet(client: &Client, token: &str, pet: &CreatePetRequest) -> Result<Pet> {
    let response = client
        .post("http://localhost:3000/petdb")
        .header("Authorization", format!("Bearer {}", token))
        .json(pet)
        .send()
        .await?;
    
    match response.status() {
        StatusCode::CREATED => Ok(response.json::<Pet>().await?),
        StatusCode::BAD_REQUEST => {
            let error = response.json::<ErrorResponse>().await?;
            Err(format!("Validation error: {}", error.message).into())
        },
        _ => Err(format!("Failed to create pet: {}", response.status()).into())
    }
}
```

### Update Pet

Updates an existing pet in the database.

**URL**: `/petdb/:id`

**Method**: `PUT`

**URL Parameters**:
- `id`: UUID of the pet to update

**Authentication**: Required

**Request Headers**:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body**:

```json
{
  "name": "Updated Name",      // Optional
  "species": "Updated Species", // Optional
  "age": 6                     // Optional
}
```

**Validation Rules**:
- `name` (if provided): Cannot be empty, maximum 50 characters
- `species` (if provided): Cannot be empty
- `age` (if provided): Must be non-negative, must be realistic (0-100)

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Updated Name",
  "species": "Updated Species",
  "age": 6,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T13:00:00.000Z"
}
```

**Status Codes**:
- `200 OK`: Successfully updated the pet
- `400 Bad Request`: Invalid UUID format or validation error
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

**Curl Example**:

```bash
curl -X PUT http://localhost:3000/petdb/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{"name":"Fluffy Jr.","age":4}'
```

**Code Example**:

```rust
// Client-side request to update a pet
async fn update_pet(client: &Client, token: &str, id: &str, update: &UpdatePetRequest) -> Result<Pet> {
    let response = client
        .put(&format!("http://localhost:3000/petdb/{}", id))
        .header("Authorization", format!("Bearer {}", token))
        .json(update)
        .send()
        .await?;
    
    match response.status() {
        StatusCode::OK => Ok(response.json::<Pet>().await?),
        StatusCode::NOT_FOUND => Err("Pet not found".into()),
        StatusCode::BAD_REQUEST => {
            let error = response.json::<ErrorResponse>().await?;
            Err(format!("Validation error: {}", error.message).into())
        },
        _ => Err(format!("Failed to update pet: {}", response.status()).into())
    }
}
```

### Delete Pet

Deletes a pet from the database.

**URL**: `/petdb/:id`

**Method**: `DELETE`

**URL Parameters**:
- `id`: UUID of the pet to delete

**Authentication**: Required

**Request Headers**:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response**: No content

**Status Codes**:
- `204 No Content`: Successfully deleted the pet
- `400 Bad Request`: Invalid UUID format
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

**Curl Example**:

```bash
curl -X DELETE http://localhost:3000/petdb/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Code Example**:

```rust
// Client-side request to delete a pet
async fn delete_pet(client: &Client, token: &str, id: &str) -> Result<()> {
    let response = client
        .delete(&format!("http://localhost:3000/petdb/{}", id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),
        StatusCode::NOT_FOUND => Err("Pet not found".into()),
        _ => Err(format!("Failed to delete pet: {}", response.status()).into())
    }
}
```

## Data Models

### Pet

```rust
/// Represents a pet entity in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    /// Unique identifier
    pub id: String,
    
    /// Pet name
    pub name: String,
    
    /// Pet species
    pub species: String,
    
    /// Pet age in years
    pub age: u32,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}
```

### CreatePetRequest

```rust
/// Request to create a new pet
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePetRequest {
    /// Pet name
    #[validate(required, length(min = 1, max = 50))]
    pub name: String,
    
    /// Pet species
    #[validate(required, length(min = 1))]
    pub species: String,
    
    /// Pet age in years
    #[validate(range(min = 0, max = 100))]
    pub age: u32,
}
```

### UpdatePetRequest

```rust
/// Request to update an existing pet
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePetRequest {
    /// Optional updated pet name
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    
    /// Optional updated pet species
    #[validate(length(min = 1))]
    pub species: Option<String>,
    
    /// Optional updated pet age
    #[validate(range(min = 0, max = 100))]
    pub age: Option<u32>,
}
```

### PetListResponse

```rust
/// Response containing a list of pets with pagination
#[derive(Debug, Serialize, Deserialize)]
pub struct PetListResponse {
    /// List of pet records
    pub data: Vec<Pet>,
    
    /// Pagination information
    pub pagination: Pagination,
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    /// Total number of records
    pub total: u64,
    
    /// Number of records in current page
    pub limit: u32,
    
    /// Starting offset
    pub offset: u32,
    
    /// Next page offset (null if last page)
    pub next_offset: Option<u32>,
}
```

## Architecture

The Pet API follows a clean architecture approach with the following layers:

### Core Layer

- **EntityRepository**: Generic repository interface in `core/database/repository.rs` that defines standard CRUD operations
- **Database Utilities**: Common database functions in `core/database/utils.rs` for transaction management and error handling

```rust
/// Generic repository interface for database entities
pub trait EntityRepository<T, ID, C, U> {
    /// Create a new entity
    async fn create(&self, data: &C) -> Result<T>;
    
    /// Find an entity by ID
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    
    /// Find all entities matching criteria
    async fn find_all(&self, criteria: &HashMap<String, Value>) -> Result<Vec<T>>;
    
    /// Update an entity
    async fn update(&self, id: &ID, data: &U) -> Result<T>;
    
    /// Delete an entity
    async fn delete(&self, id: &ID) -> Result<bool>;
}
```

### Application Layer

- **PetRepository**: Implementation of the `EntityRepository` for Pet entities in `app/database/repositories/pet_repository.rs`
- **PetService**: Business logic and validation in `app/services/pet_service.rs`
- **API Endpoints**: HTTP handlers in `app/api/pet_db.rs` that expose the functionality via REST API

```rust
/// Implementation of the Pet repository
pub struct PetRepository {
    pool: Pool<Postgres>,
}

impl PetRepository {
    /// Create a new PetRepository
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl EntityRepository<Pet, String, CreatePetRequest, UpdatePetRequest> for PetRepository {
    async fn create(&self, data: &CreatePetRequest) -> Result<Pet> {
        // Implementation...
    }
    
    async fn find_by_id(&self, id: &String) -> Result<Option<Pet>> {
        // Implementation...
    }
    
    // Other implementations...
}
```

```rust
/// Pet service handling business logic
pub struct PetService {
    repository: Arc<dyn EntityRepository<Pet, String, CreatePetRequest, UpdatePetRequest>>,
}

impl PetService {
    pub fn new(repository: Arc<dyn EntityRepository<Pet, String, CreatePetRequest, UpdatePetRequest>>) -> Self {
        Self { repository }
    }
    
    /// Create a new pet with validation
    pub async fn create_pet(&self, data: CreatePetRequest) -> Result<Pet> {
        // Validate data
        data.validate()?;
        
        // Create pet in database
        self.repository.create(&data).await
    }
    
    // Other methods...
}
```

This separation allows for clear responsibilities:
- **Core Layer**: Generic interfaces and abstractions
- **App Layer**: Pet-specific implementations

## Implementation

### HTTP Handlers

```rust
/// Pet database API endpoints
pub fn pet_routes() -> Router {
    Router::new()
        .route("/", get(get_all_pets).post(create_pet))
        .route("/:id", get(get_pet_by_id).put(update_pet).delete(delete_pet))
}

/// Handler for GET /petdb
async fn get_all_pets(
    State(state): State<AppState>,
    Query(params): Query<GetAllPetsParams>,
) -> Result<Json<PetListResponse>, AppError> {
    let pet_service = &state.pet_service;
    
    let criteria = HashMap::new();
    // Convert query params to criteria...
    
    let pets = pet_service.find_all_pets(criteria, params.limit, params.offset).await?;
    Ok(Json(pets))
}

/// Handler for POST /petdb
async fn create_pet(
    State(state): State<AppState>,
    auth: AuthExtractor,
    Json(data): Json<CreatePetRequest>,
) -> Result<(StatusCode, Json<Pet>), AppError> {
    // Verify permissions
    if !auth.has_permission("create:pets") {
        return Err(AppError::forbidden("Insufficient permissions"));
    }
    
    let pet_service = &state.pet_service;
    let pet = pet_service.create_pet(data).await?;
    
    Ok((StatusCode::CREATED, Json(pet)))
}

// Other handlers...
```

### Integration with Router

```rust
// In app/api/router.rs
pub fn api_routes() -> Router {
    Router::new()
        // Other routes...
        .nest("/petdb", pet_routes())
        // Other routes...
}
```

## Error Handling

The API follows a consistent error handling approach:

- **400 Bad Request**: Input validation errors (invalid data, format issues)
- **401 Unauthorized**: Authentication issues
- **404 Not Found**: Resource not found
- **500 Internal Server Error**: Database or server-side errors

All error responses include:
- HTTP status code
- Error message
- Error type

Example error response:

```json
{
  "code": 400,
  "message": "Pet name cannot be empty",
  "error_type": "validation_error",
  "details": [
    {
      "field": "name",
      "error": "Cannot be empty"
    }
  ]
}
```

### Error Types

```rust
/// Application error types
pub enum AppErrorType {
    /// Invalid input data
    ValidationError,
    
    /// Resource not found
    NotFound,
    
    /// Authentication error
    Unauthorized,
    
    /// Permission error
    Forbidden,
    
    /// Database error
    DatabaseError,
    
    /// Server error
    InternalError,
}

/// Application error structure
pub struct AppError {
    /// HTTP status code
    pub code: StatusCode,
    
    /// Error message
    pub message: String,
    
    /// Error type
    pub error_type: AppErrorType,
    
    /// Additional error details
    pub details: Option<Vec<ErrorDetail>>,
}

/// Detailed error information
pub struct ErrorDetail {
    /// Field name (for validation errors)
    pub field: String,
    
    /// Error description
    pub error: String,
}
```

### Error Conversion

```rust
// Example of converting validation errors
impl From<ValidationError> for AppError {
    fn from(error: ValidationError) -> Self {
        let details = error.field_errors().iter()
            .map(|(field, errors)| {
                ErrorDetail {
                    field: field.to_string(),
                    error: errors[0].message.clone().unwrap_or_default(),
                }
            })
            .collect();
            
        AppError {
            code: StatusCode::BAD_REQUEST,
            message: "Validation failed".to_string(),
            error_type: AppErrorType::ValidationError,
            details: Some(details),
        }
    }
}
```

## Testing

The API includes comprehensive tests:

- **Unit Tests**: For core database abstractions and PetService business logic
- **API Endpoint Tests**: Testing the HTTP layer and response handling

### Example Unit Test

```rust
#[tokio::test]
async fn test_pet_service_create() {
    // Setup mock repository
    let mock_repo = MockPetRepository::new();
    mock_repo.expect_create()
        .with(predicate::function(|req: &CreatePetRequest| {
            req.name == "Fluffy" && req.species == "cat" && req.age == 3
        }))
        .returning(|_| {
            Ok(Pet {
                id: "test-uuid".to_string(),
                name: "Fluffy".to_string(),
                species: "cat".to_string(),
                age: 3,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        });
    
    let service = PetService::new(Arc::new(mock_repo));
    
    // Call service method
    let create_req = CreatePetRequest {
        name: "Fluffy".to_string(),
        species: "cat".to_string(),
        age: 3,
    };
    
    let result = service.create_pet(create_req).await;
    
    // Verify result
    assert!(result.is_ok());
    let pet = result.unwrap();
    assert_eq!(pet.name, "Fluffy");
    assert_eq!(pet.species, "cat");
    assert_eq!(pet.age, 3);
}
```

### Example API Test

```rust
#[tokio::test]
async fn test_create_pet_endpoint() {
    // Setup test app with mocked dependencies
    let app = test_app().await;
    
    // Test valid request
    let response = app
        .client
        .post("/petdb")
        .header("Authorization", "Bearer test-token")
        .json(&json!({
            "name": "Fluffy",
            "species": "cat",
            "age": 3
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let pet: Pet = response.json().await.unwrap();
    assert_eq!(pet.name, "Fluffy");
    
    // Test invalid request
    let response = app
        .client
        .post("/petdb")
        .header("Authorization", "Bearer test-token")
        .json(&json!({
            "name": "",  // Empty name (invalid)
            "species": "cat",
            "age": 3
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

## Related Resources

- [Repository Pattern Guide](../patterns/repository-pattern.md)
- [Database API Reference](./database-api.md)
- [API Resource Implementation](./api-resource.md)
- [Database Service Example](../../02_examples/database-service-example.md)

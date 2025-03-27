---
title: "REST API Example"
description: "Building RESTful APIs with Navius"
category: examples
tags:
  - examples
  - rest
  - api
  - http
related:
  - examples/basic-application-example.md
  - examples/error-handling-example.md
  - reference/api/application-api.md
last_updated: March 26, 2024
version: 1.0
---

# REST API Example

This example demonstrates how to build a RESTful API using Navius, including proper resource modeling, request handling, response formatting, and error handling.

## Project Structure

```
rest-api-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs                  # Application entry point
    ├── api/                     # API handlers
    │   ├── mod.rs
    │   └── products.rs          # Product resource handlers
    ├── models/                  # Domain models
    │   ├── mod.rs
    │   └── product.rs           # Product model
    ├── repositories/            # Data access
    │   ├── mod.rs
    │   └── product_repository.rs # Product repository
    ├── services/                # Business logic
    │   ├── mod.rs
    │   └── product_service.rs   # Product service
    └── utils/                   # Utilities
        ├── mod.rs
        └── error.rs             # Error handling
```

## Implementation

### Cargo.toml

```toml
[package]
name = "rest-api-example"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = "0.1.0"
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.3", features = ["v4", "serde"] }
async-trait = "0.1"
tracing = "0.1"
validator = { version = "0.16", features = ["derive"] }
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

### src/models/product.rs

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub stock: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    
    #[validate(range(min = 0.01))]
    pub price: f64,
    
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    
    #[validate(range(min = 0))]
    pub stock: i32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    
    #[validate(length(min = 1, max = 1000))]
    pub description: Option<String>,
    
    #[validate(range(min = 0.01))]
    pub price: Option<f64>,
    
    #[validate(length(min = 1, max = 50))]
    pub category: Option<String>,
    
    #[validate(range(min = 0))]
    pub stock: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub in_stock: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl From<CreateProductRequest> for Product {
    fn from(req: CreateProductRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: req.name,
            description: req.description,
            price: req.price,
            category: req.category,
            stock: req.stock,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### src/repositories/product_repository.rs

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::models::product::{Product, ProductQuery};
use crate::utils::error::AppError;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(&self, query: &ProductQuery) -> Result<Vec<Product>, AppError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Product, AppError>;
    async fn create(&self, product: Product) -> Result<Product, AppError>;
    async fn update(&self, id: &Uuid, product: Product) -> Result<Product, AppError>;
    async fn delete(&self, id: &Uuid) -> Result<(), AppError>;
}

// In-memory implementation for the example
pub struct InMemoryProductRepository {
    products: RwLock<HashMap<Uuid, Product>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self {
            products: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn find_all(&self, query: &ProductQuery) -> Result<Vec<Product>, AppError> {
        let products = self.products.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock")
        })?;
        
        let mut result: Vec<Product> = products.values().cloned().collect();
        
        // Apply filters
        if let Some(category) = &query.category {
            result.retain(|p| p.category == *category);
        }
        
        if let Some(min_price) = query.min_price {
            result.retain(|p| p.price >= min_price);
        }
        
        if let Some(max_price) = query.max_price {
            result.retain(|p| p.price <= max_price);
        }
        
        if let Some(in_stock) = query.in_stock {
            if in_stock {
                result.retain(|p| p.stock > 0);
            } else {
                result.retain(|p| p.stock == 0);
            }
        }
        
        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(result.len());
        
        let end = (offset + limit).min(result.len());
        if offset < end {
            Ok(result[offset..end].to_vec())
        } else {
            Ok(vec![])
        }
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Product, AppError> {
        let products = self.products.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock")
        })?;
        
        products.get(id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("Product with ID {} not found", id)))
    }
    
    async fn create(&self, product: Product) -> Result<Product, AppError> {
        let mut products = self.products.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock")
        })?;
        
        let product_clone = product.clone();
        products.insert(product.id, product);
        
        Ok(product_clone)
    }
    
    async fn update(&self, id: &Uuid, product: Product) -> Result<Product, AppError> {
        let mut products = self.products.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock")
        })?;
        
        if !products.contains_key(id) {
            return Err(AppError::not_found(format!("Product with ID {} not found", id)));
        }
        
        let product_clone = product.clone();
        products.insert(*id, product);
        
        Ok(product_clone)
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), AppError> {
        let mut products = self.products.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock")
        })?;
        
        if products.remove(id).is_none() {
            return Err(AppError::not_found(format!("Product with ID {} not found", id)));
        }
        
        Ok(())
    }
}
```

### src/services/product_service.rs

```rust
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::models::product::{Product, CreateProductRequest, UpdateProductRequest, ProductQuery};
use crate::repositories::product_repository::ProductRepository;
use crate::utils::error::AppError;

pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub fn new(repository: Arc<dyn ProductRepository>) -> Self {
        Self {
            repository,
        }
    }
    
    pub async fn get_products(&self, query: ProductQuery) -> Result<Vec<Product>, AppError> {
        self.repository.find_all(&query).await
    }
    
    pub async fn get_product(&self, id: &Uuid) -> Result<Product, AppError> {
        self.repository.find_by_id(id).await
    }
    
    pub async fn create_product(&self, req: CreateProductRequest) -> Result<Product, AppError> {
        // Convert request to model
        let product = Product::from(req);
        
        // Save to repository
        self.repository.create(product).await
    }
    
    pub async fn update_product(&self, id: &Uuid, req: UpdateProductRequest) -> Result<Product, AppError> {
        // Get existing product
        let mut product = self.repository.find_by_id(id).await?;
        
        // Update fields if provided
        if let Some(name) = req.name {
            product.name = name;
        }
        
        if let Some(description) = req.description {
            product.description = description;
        }
        
        if let Some(price) = req.price {
            product.price = price;
        }
        
        if let Some(category) = req.category {
            product.category = category;
        }
        
        if let Some(stock) = req.stock {
            product.stock = stock;
        }
        
        // Update timestamp
        product.updated_at = Utc::now();
        
        // Save updated product
        self.repository.update(id, product).await
    }
    
    pub async fn delete_product(&self, id: &Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
```

### src/utils/error.rs

```rust
use navius::http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "status": self.status_code().as_u16(),
            "code": self.error_code(),
            "message": self.to_string(),
        })
    }
    
    // Helper functions to create errors
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }
    
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }
    
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
    
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError(message.into())
    }
}

impl Into<navius::http::Response> for AppError {
    fn into(self) -> navius::http::Response {
        let status = self.status_code();
        let body = self.to_json().to_string();
        
        let mut response = navius::http::Response::new(body.into());
        *response.status_mut() = status;
        response.headers_mut().insert(
            navius::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        
        response
    }
}
```

### src/api/products.rs

```rust
use std::sync::Arc;
use navius::http::{Request, Response, StatusCode};
use navius::web::{Path, Query, Json, State};
use uuid::Uuid;

use crate::models::product::{CreateProductRequest, UpdateProductRequest, ProductQuery};
use crate::services::product_service::ProductService;
use crate::utils::error::AppError;

pub async fn get_products(
    Query(query): Query<ProductQuery>,
    State(product_service): State<Arc<ProductService>>,
) -> Result<Json<Vec<crate::models::product::Product>>, AppError> {
    let products = product_service.get_products(query).await?;
    Ok(Json(products))
}

pub async fn get_product(
    Path(id): Path<Uuid>,
    State(product_service): State<Arc<ProductService>>,
) -> Result<Json<crate::models::product::Product>, AppError> {
    let product = product_service.get_product(&id).await?;
    Ok(Json(product))
}

pub async fn create_product(
    Json(create_req): Json<CreateProductRequest>,
    State(product_service): State<Arc<ProductService>>,
) -> Result<(StatusCode, Json<crate::models::product::Product>), AppError> {
    // Validate request
    if let Err(errors) = validator::Validate::validate(&create_req) {
        return Err(AppError::bad_request(format!("Validation error: {:?}", errors)));
    }
    
    // Create product
    let product = product_service.create_product(create_req).await?;
    
    // Return 201 Created with the product
    Ok((StatusCode::CREATED, Json(product)))
}

pub async fn update_product(
    Path(id): Path<Uuid>,
    Json(update_req): Json<UpdateProductRequest>,
    State(product_service): State<Arc<ProductService>>,
) -> Result<Json<crate::models::product::Product>, AppError> {
    // Validate request
    if let Err(errors) = validator::Validate::validate(&update_req) {
        return Err(AppError::bad_request(format!("Validation error: {:?}", errors)));
    }
    
    // Update product
    let product = product_service.update_product(&id, update_req).await?;
    
    Ok(Json(product))
}

pub async fn delete_product(
    Path(id): Path<Uuid>,
    State(product_service): State<Arc<ProductService>>,
) -> Result<StatusCode, AppError> {
    // Delete product
    product_service.delete_product(&id).await?;
    
    // Return 204 No Content
    Ok(StatusCode::NO_CONTENT)
}
```

### src/api/mod.rs

```rust
pub mod products;

use navius::routing::{Router, get, post, put, delete};
use std::sync::Arc;

use crate::services::product_service::ProductService;

pub fn api_router(product_service: Arc<ProductService>) -> Router {
    Router::new()
        .nest("/api/v1", 
            Router::new()
                .nest("/products", products_router(product_service))
        )
}

fn products_router(product_service: Arc<ProductService>) -> Router {
    Router::new()
        .route("/", get(products::get_products).post(products::create_product))
        .route("/:id", get(products::get_product).put(products::update_product).delete(products::delete_product))
        .with_state(product_service)
}
```

### src/main.rs

```rust
mod api;
mod models;
mod repositories;
mod services;
mod utils;

use std::sync::Arc;
use navius::Application;

use repositories::product_repository::InMemoryProductRepository;
use services::product_service::ProductService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    tracing_subscriber::fmt::init();
    
    // Create repository
    let product_repository = Arc::new(InMemoryProductRepository::new());
    
    // Create service
    let product_service = Arc::new(ProductService::new(product_repository));
    
    // Set up the router
    let router = api::api_router(product_service);
    
    // Create and run the application
    let app = Application::builder()
        .with_router(router)
        .build()?;
    
    println!("REST API server running at http://127.0.0.1:8080");
    app.run("127.0.0.1:8080").await?;
    
    Ok(())
}
```

### config/default.yaml

```yaml
server:
  host: "127.0.0.1"
  port: 8080
  
logging:
  level: "debug"
  format: "json"
```

## Running the Example

To run this REST API example:

1. Clone the repository:
   ```bash
   git clone https://github.com/navius/examples.git
   cd examples/rest-api-example
   ```

2. Build and run the application:
   ```bash
   cargo run
   ```

3. The server will start on `http://127.0.0.1:8080`

## Testing the API Endpoints

### Create a Product

```bash
curl -X POST http://127.0.0.1:8080/api/v1/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Ergonomic Keyboard",
    "description": "Comfortable ergonomic keyboard for long coding sessions",
    "price": 129.99,
    "category": "computer accessories",
    "stock": 50
  }'
```

### Get All Products

```bash
curl http://127.0.0.1:8080/api/v1/products
```

### Get Products with Filters

```bash
curl "http://127.0.0.1:8080/api/v1/products?category=computer%20accessories&min_price=100&in_stock=true"
```

### Get a Product by ID

```bash
curl http://127.0.0.1:8080/api/v1/products/[product-id]
```

### Update a Product

```bash
curl -X PUT http://127.0.0.1:8080/api/v1/products/[product-id] \
  -H "Content-Type: application/json" \
  -d '{
    "price": 119.99,
    "stock": 45
  }'
```

### Delete a Product

```bash
curl -X DELETE http://127.0.0.1:8080/api/v1/products/[product-id]
```

## Key Concepts Demonstrated

1. **Resource-Oriented Design**: The API is structured around product resources with standard CRUD operations.

2. **Request Validation**: Input validation using the `validator` crate to ensure data integrity.

3. **Error Handling**: Comprehensive error handling with meaningful error messages and appropriate HTTP status codes.

4. **Query Parameters**: Supporting filtering, sorting, and pagination through query parameters.

5. **Proper HTTP Status Codes**:
   - `200 OK` for successful GET, PUT operations
   - `201 Created` for successful POST operations
   - `204 No Content` for successful DELETE operations
   - `400 Bad Request` for validation errors
   - `404 Not Found` for resources that don't exist

6. **Service Layer Architecture**:
   - API handlers: Handle HTTP requests and responses
   - Services: Contain business logic
   - Repositories: Manage data access
   - Models: Define data structures

7. **Dependency Injection**: Services and repositories are injected as dependencies.

## Best Practices

1. **API Versioning**: Routes are prefixed with `/api/v1/` to support future versioning.

2. **Response Formatting**: Consistent JSON response structure.

3. **Validation**: Input data is validated before processing.

4. **Separation of Concerns**: Clear separation between API handling, business logic, and data access.

5. **Error Messages**: Descriptive error messages that don't expose internal details.

6. **Status Codes**: Appropriate HTTP status codes for different situations.

## Next Steps

- Implement authentication and authorization
- Add OpenAPI documentation
- Implement a real database connection
- Add pagination metadata to responses
- Implement request/response logging middleware

## Related Documentation

- [Application API Reference](../reference/api/application-api.md)
- [Error Handling Example](./error-handling-example.md)
- [Basic Application Example](./basic-application-example.md) 
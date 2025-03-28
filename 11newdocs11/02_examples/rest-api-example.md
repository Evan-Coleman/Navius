---
title: "Building RESTful APIs with Navius"
description: "Comprehensive guide to implementing RESTful APIs using Navius, including resource modeling, validation, pagination, error handling, and testing"
category: examples
tags:
  - rest
  - api
  - http
  - crud
  - controllers
  - validation
  - pagination
  - error-handling
related:
  - 02_examples/graphql-example.md
  - 02_examples/dependency-injection-example.md
  - 04_guides/api-design.md
last_updated: March 27, 2025
version: 1.1
status: stable
---

# REST API Example

This example demonstrates how to build a RESTful API using Navius, including proper resource modeling, request handling, response formatting, and error handling.

## Overview

REST (Representational State Transfer) is an architectural style for designing networked applications. Navius provides a clean, idiomatic way to build REST APIs in Rust with features like request validation, route handlers, error handling, and JSON serialization/deserialization.

This example builds a product catalog API that demonstrates:
- RESTful resource modeling with CRUD operations
- Request validation and sanitization
- Response formatting and status codes
- Query parameter handling and pagination
- Error handling and appropriate HTTP status codes
- Clean architecture with separation of concerns

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Models](#srcmodelsproductrs)
  - [Repository](#srcrepositoriesproduct_repositoryrs)
  - [Service Layer](#srcservicesproduct_servicers)
  - [API Handlers](#srcapiproductsrs)
  - [Error Handling](#srcutilserrorrs)
  - [Application Entry Point](#srcmainrs)
- [API Endpoints](#api-endpoints)
- [Testing the API](#testing-the-api)
- [Best Practices](#best-practices)
- [Common Patterns](#common-patterns)
- [Performance Considerations](#performance-considerations)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- HTTP and REST API fundamentals
- JSON serialization/deserialization
- Basic error handling patterns
- Navius framework basics

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- tokio for asynchronous operations
- serde for serialization/deserialization
- validator for request validation
- uuid for unique identifiers

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
name = "navius-rest-api-example"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = "0.1.0"
tokio = { version = "1.28.0", features = ["full"] }
serde = { version = "1.0.160", features = ["derive"] }
serde_json = "1.0.96"
validator = { version = "0.16.0", features = ["derive"] }
thiserror = "1.0.40"
async-trait = "0.1.68"
uuid = { version = "1.3.1", features = ["v4", "serde"] }
tracing = "0.1.37"
chrono = { version = "0.4.24", features = ["serde"] }
```

### config/default.yaml

```yaml
server:
  host: "127.0.0.1"
  port: 8080
  
logging:
  level: debug
  format: json

cors:
  allowed_origins: ["*"]
  allowed_methods: ["GET", "POST", "PUT", "DELETE"]
  max_age_secs: 86400
```

### src/models/product.rs

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub sku: String,
    pub stock: i32,
    pub category: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,
    
    #[validate(length(min = 10, max = 1000, message = "Description must be between 10 and 1000 characters"))]
    pub description: String,
    
    #[validate(range(min = 0.0, message = "Price must be positive"))]
    pub price: f64,
    
    #[validate(length(min = 3, max = 20, message = "SKU must be between 3 and 20 characters"))]
    pub sku: String,
    
    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: i32,
    
    #[validate(length(min = 1, max = 50, message = "Category must be between 1 and 50 characters"))]
    pub category: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: Option<String>,
    
    #[validate(length(min = 10, max = 1000, message = "Description must be between 10 and 1000 characters"))]
    pub description: Option<String>,
    
    #[validate(range(min = 0.0, message = "Price must be positive"))]
    pub price: Option<f64>,
    
    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: Option<i32>,
    
    #[validate(length(min = 1, max = 50, message = "Category must be between 1 and 50 characters"))]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}
```

### src/repositories/product_repository.rs

```rust
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::product::{CreateProductRequest, Product, QueryParams, UpdateProductRequest};
use crate::utils::error::ApiError;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(&self, params: &QueryParams) -> Result<Vec<Product>, ApiError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError>;
    async fn create(&self, product: CreateProductRequest) -> Result<Product, ApiError>;
    async fn update(&self, id: Uuid, product: UpdateProductRequest) -> Result<Product, ApiError>;
    async fn delete(&self, id: Uuid) -> Result<(), ApiError>;
}

// For this example, we're using an in-memory repository
pub struct InMemoryProductRepository {
    products: Arc<Mutex<HashMap<Uuid, Product>>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self {
            products: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn find_all(&self, params: &QueryParams) -> Result<Vec<Product>, ApiError> {
        let products = self.products.lock().unwrap();
        
        let mut result: Vec<Product> = products.values().cloned().collect();
        
        // Apply filters
        if let Some(category) = &params.category {
            result.retain(|p| p.category == *category);
        }
        
        if let Some(min_price) = params.min_price {
            result.retain(|p| p.price >= min_price);
        }
        
        if let Some(max_price) = params.max_price {
            result.retain(|p| p.price <= max_price);
        }
        
        // Apply pagination
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);
        let skip = (page - 1) * limit;
        
        let paged_result = result
            .into_iter()
            .skip(skip)
            .take(limit)
            .collect();
            
        Ok(paged_result)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        let products = self.products.lock().unwrap();
        
        products.get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))
    }
    
    async fn create(&self, req: CreateProductRequest) -> Result<Product, ApiError> {
        let mut products = self.products.lock().unwrap();
        
        // Check if SKU already exists
        for product in products.values() {
            if product.sku == req.sku {
                return Err(ApiError::Conflict(format!("Product with SKU {} already exists", req.sku)));
            }
        }
        
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let product = Product {
            id,
            name: req.name,
            description: req.description,
            price: req.price,
            sku: req.sku,
            stock: req.stock,
            category: req.category,
            created_at: now,
            updated_at: now,
        };
        
        products.insert(id, product.clone());
        
        Ok(product)
    }
    
    async fn update(&self, id: Uuid, req: UpdateProductRequest) -> Result<Product, ApiError> {
        let mut products = self.products.lock().unwrap();
        
        let product = products.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
            
        if let Some(name) = req.name {
            product.name = name;
        }
        
        if let Some(description) = req.description {
            product.description = description;
        }
        
        if let Some(price) = req.price {
            product.price = price;
        }
        
        if let Some(stock) = req.stock {
            product.stock = stock;
        }
        
        if let Some(category) = req.category {
            product.category = category;
        }
        
        product.updated_at = Utc::now();
        
        Ok(product.clone())
    }
    
    async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let mut products = self.products.lock().unwrap();
        
        if products.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Product with ID {} not found", id)));
        }
        
        Ok(())
    }
}
```

### src/services/product_service.rs

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::product::{CreateProductRequest, Product, QueryParams, UpdateProductRequest};
use crate::repositories::product_repository::ProductRepository;
use crate::utils::error::ApiError;

#[async_trait]
pub trait ProductService: Send + Sync {
    async fn get_products(&self, params: &QueryParams) -> Result<Vec<Product>, ApiError>;
    async fn get_product(&self, id: Uuid) -> Result<Product, ApiError>;
    async fn create_product(&self, product: CreateProductRequest) -> Result<Product, ApiError>;
    async fn update_product(&self, id: Uuid, product: UpdateProductRequest) -> Result<Product, ApiError>;
    async fn delete_product(&self, id: Uuid) -> Result<(), ApiError>;
}

pub struct ProductServiceImpl<R: ProductRepository> {
    repository: R,
}

impl<R: ProductRepository> ProductServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: ProductRepository + 'static> ProductService for ProductServiceImpl<R> {
    async fn get_products(&self, params: &QueryParams) -> Result<Vec<Product>, ApiError> {
        self.repository.find_all(params).await
    }
    
    async fn get_product(&self, id: Uuid) -> Result<Product, ApiError> {
        self.repository.find_by_id(id).await
    }
    
    async fn create_product(&self, product: CreateProductRequest) -> Result<Product, ApiError> {
        self.repository.create(product).await
    }
    
    async fn update_product(&self, id: Uuid, product: UpdateProductRequest) -> Result<Product, ApiError> {
        self.repository.update(id, product).await
    }
    
    async fn delete_product(&self, id: Uuid) -> Result<(), ApiError> {
        self.repository.delete(id).await
    }
}
```

### src/api/products.rs

```rust
use navius::request::{Json, Path, Query};
use navius::response::{Response, StatusCode};
use navius::routing::{delete, get, post, put, Route, Router};
use uuid::Uuid;
use validator::Validate;

use crate::models::product::{CreateProductRequest, QueryParams, UpdateProductRequest};
use crate::services::product_service::ProductService;
use crate::utils::error::ApiError;

pub fn products_routes<S: ProductService + 'static>() -> Router {
    Router::new()
        .route("/products", get(get_products::<S>))
        .route("/products", post(create_product::<S>))
        .route("/products/:id", get(get_product::<S>))
        .route("/products/:id", put(update_product::<S>))
        .route("/products/:id", delete(delete_product::<S>))
}

async fn get_products<S: ProductService + 'static>(
    service: navius::Extension<S>,
    Query(params): Query<QueryParams>,
) -> Result<Response, ApiError> {
    let products = service.get_products(&params).await?;
    Ok(Response::json(&products))
}

async fn get_product<S: ProductService + 'static>(
    service: navius::Extension<S>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let product = service.get_product(id).await?;
    Ok(Response::json(&product))
}

async fn create_product<S: ProductService + 'static>(
    service: navius::Extension<S>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    payload.validate()?;
    
    let product = service.create_product(payload).await?;
    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .json(&product))
}

async fn update_product<S: ProductService + 'static>(
    service: navius::Extension<S>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    payload.validate()?;
    
    let product = service.update_product(id, payload).await?;
    Ok(Response::json(&product))
}

async fn delete_product<S: ProductService + 'static>(
    service: navius::Extension<S>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    service.delete_product(id).await?;
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(()))
}
```

### src/utils/error.rs

```rust
use navius::response::{Response, StatusCode};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl From<ApiError> for Response {
    fn from(error: ApiError) -> Self {
        let status = match &error {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let message = error.to_string();
        
        let body = ErrorResponse {
            status: status.to_string(),
            message,
        };
        
        Response::builder()
            .status(status)
            .json(&body)
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to serialize error response")
                    .unwrap()
            })
    }
}
```

### src/main.rs

```rust
use navius::{Application, Config};
use std::sync::Arc;

mod api;
mod models;
mod repositories;
mod services;
mod utils;

use crate::api::products::products_routes;
use crate::repositories::product_repository::InMemoryProductRepository;
use crate::services::product_service::ProductServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_file("config/default.yaml")?;
    
    // Initialize repository
    let repository = InMemoryProductRepository::new();
    
    // Initialize service
    let service = ProductServiceImpl::new(repository);
    
    // Create the application
    let app = Application::new()
        .with_config(config)
        .register_extension(service)
        .register_routes(products_routes())
        .build()?;
    
    // Start the server
    app.run().await?;
    
    Ok(())
}
```

## API Endpoints

The API provides the following endpoints:

| Method | Path | Description |
|--------|------|-------------|
| GET | /products | List all products with optional filtering and pagination |
| GET | /products/:id | Get a specific product by ID |
| POST | /products | Create a new product |
| PUT | /products/:id | Update an existing product |
| DELETE | /products/:id | Delete a product |

### Query Parameters for GET /products

| Parameter | Type | Description |
|-----------|------|-------------|
| page | integer | Page number for pagination (default: 1) |
| limit | integer | Number of items per page (default: 10) |
| category | string | Filter products by category |
| min_price | float | Filter products with price >= min_price |
| max_price | float | Filter products with price <= max_price |

## Testing the API

You can test the API using curl, Postman, or any HTTP client. Here are some examples:

### List Products

```bash
curl -X GET http://localhost:8080/products
```

With filters:

```bash
curl -X GET "http://localhost:8080/products?category=electronics&min_price=100&page=1&limit=5"
```

### Get Product by ID

```bash
curl -X GET http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000
```

### Create Product

```bash
curl -X POST http://localhost:8080/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Wireless Headphones",
    "description": "High-quality wireless headphones with noise cancellation",
    "price": 199.99,
    "sku": "WH-2023-001",
    "stock": 50,
    "category": "electronics"
  }'
```

### Update Product

```bash
curl -X PUT http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{
    "price": 179.99,
    "stock": 45
  }'
```

### Delete Product

```bash
curl -X DELETE http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000
```

## Best Practices

When building REST APIs with Navius, consider the following best practices:

1. **Use Proper HTTP Methods**:
   - GET for retrieving resources
   - POST for creating new resources
   - PUT for updating entire resources
   - PATCH for partial updates
   - DELETE for removing resources

2. **Return Appropriate Status Codes**:
   - 200 OK for successful GET, PUT, PATCH, DELETE
   - 201 Created for successful POST
   - 204 No Content for successful DELETE without response body
   - 400 Bad Request for invalid input
   - 404 Not Found for missing resources
   - 409 Conflict for resource conflicts
   - 500 Internal Server Error for server errors

3. **Validate Input**:
   - Always validate and sanitize input data
   - Return helpful validation error messages
   - Use the validator crate with #[derive(Validate)]

4. **Resource Naming**:
   - Use plural nouns for resources (e.g., /products not /product)
   - Use kebab-case for multi-word resources (e.g., /order-items)
   - Keep URLs simple and intuitive

5. **Implement Pagination**:
   - Provide pagination for list endpoints
   - Include metadata about pagination (total, page, limit)
   - Use query parameters for pagination control

6. **Proper Error Handling**:
   - Create consistent error response structures
   - Include meaningful error messages
   - Use typed errors with thiserror

## Common Patterns

### Resource-Based Routing

Organize routes based on resources for clarity:

```rust
Router::new()
    .nest("/products", products_routes())
    .nest("/orders", orders_routes())
    .nest("/customers", customers_routes())
```

### Response Envelope

For more detailed responses, consider using a response envelope:

```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    metadata: Option<ResponseMetadata>,
}

#[derive(Serialize)]
struct ResponseMetadata {
    total: usize,
    page: usize,
    limit: usize,
}
```

### Service Registration

Make services available to the application:

```rust
let app = Application::new()
    .register_extension(product_service)
    .register_extension(order_service)
    .register_routes(api_routes())
    .build()?;
```

## Performance Considerations

1. **Connection Pooling**: Use connection pools for database access to reduce overhead.

2. **Caching**: Implement caching for frequently accessed resources.

3. **Asynchronous Processing**: Use Rust's async/await for non-blocking I/O operations.

4. **Pagination**: Always paginate large collections to prevent memory issues.

5. **Batch Operations**: Provide endpoints for batch operations to reduce round trips.

## Security Considerations

1. **Input Validation**: Always validate and sanitize user input.

2. **CORS Configuration**: Configure CORS appropriately for your application.

3. **Rate Limiting**: Implement rate limiting to prevent abuse.

4. **Authentication and Authorization**: Add proper auth middleware.

5. **HTTPS**: Always use HTTPS in production.

6. **Content Security**: Set appropriate security headers.

## Troubleshooting

### Common Issues

1. **404 Not Found for Valid Routes**:
   - Check route registration order
   - Verify URL path parameters
   - Check for trailing slashes

2. **Serialization Errors**:
   - Ensure all fields in your models implement Serialize/Deserialize
   - Check for circular references
   - Verify date/time formats

3. **Connection Errors**:
   - Verify network configuration
   - Check firewall settings
   - Ensure correct host/port in config

### Debugging Tips

1. Use tracing to debug request flow:

```rust
tracing::debug!("Processing product request: {:?}", product);
```

2. Enable more detailed logs by setting log level to debug or trace in your configuration.

3. Use middleware to log request/response information:

```rust
app.middleware(LoggingMiddleware::new())
```

## Next Steps

After mastering the basics of REST APIs with Navius, consider:

1. Adding authentication and authorization
2. Implementing more advanced filtering and sorting
3. Adding metrics and monitoring
4. Exploring GraphQL as an alternative to REST
5. Implementing WebSockets for real-time updates

For more examples, see the [GraphQL Example](../02_examples/graphql-example.md) or [Dependency Injection Example](../02_examples/dependency-injection-example.md). 
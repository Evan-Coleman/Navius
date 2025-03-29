---
title: "Database Integration in Navius"
description: "Comprehensive guide to integrating databases in Navius applications, including repository patterns, connection management, migrations, and working with popular database systems"
category: examples
tags:
  - database
  - repository-pattern
  - sql
  - nosql
  - migrations
  - transactions
  - orm
related:
  - 02_examples/custom-service-example.md
  - 02_examples/rest-api-example.md
  - 04_guides/data-modeling.md
last_updated: March 27, 2025
version: 1.0
status: stable
---

# Database Integration Example

This example demonstrates how to integrate various database systems with Navius applications, featuring robust patterns for data access, migration strategies, and best practices for production use.

## Overview

Database integration is a critical aspect of most web applications. This example provides a comprehensive approach to working with databases in Navius, including:

- Setting up database connections and providers
- Implementing the Repository Pattern for clean data access
- Working with different database types (SQL and NoSQL)
- Managing database migrations and schema changes
- Implementing transactions and error handling
- Testing database-related code
- Optimizing database operations for performance

By following these patterns, you'll be able to create maintainable, scalable, and testable data access layers for your Navius applications.

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Database Service](#database-service)
  - [Repository Pattern](#repository-pattern)
  - [Entity Models](#entity-models)
  - [SQL Database Integration](#sql-database-integration)
  - [NoSQL Database Integration](#nosql-database-integration)
  - [Migrations](#migrations)
  - [Configuration](#configuration)
- [Testing With Databases](#testing-with-databases)
- [Key Concepts](#key-concepts)
- [Database Best Practices](#database-best-practices)
- [Performance Optimization](#performance-optimization)
- [Advanced Topics](#advanced-topics)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- Basic database concepts (CRUD operations, transactions, etc.)
- SQL fundamentals (for relational database parts)
- JSON document concepts (for NoSQL database parts)

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- sqlx for SQL databases
- tokio for asynchronous operations
- serde for serialization/deserialization
- uuid for unique identifiers

## Project Structure

```
database-integration-example/
├── Cargo.toml                # Project dependencies
├── config/
│   └── default.yaml          # Database configuration
├── migrations/
│   ├── 20250301000000_create_users_table.sql
│   ├── 20250301000001_create_products_table.sql
│   └── 20250301000002_add_user_roles.sql
└── src/
    ├── main.rs               # Application entry point
    ├── models/
    │   ├── mod.rs            # Module exports
    │   ├── user.rs           # User entity
    │   └── product.rs        # Product entity
    ├── repositories/
    │   ├── mod.rs            # Module exports
    │   ├── repository.rs     # Generic repository trait
    │   ├── user_repository.rs # User-specific repository
    │   └── product_repository.rs # Product-specific repository
    ├── services/
    │   ├── mod.rs            # Module exports
    │   ├── database_service.rs # Database connection service
    │   └── migration_service.rs # Database migration service
    ├── providers/
    │   ├── mod.rs            # Module exports
    │   ├── postgres_provider.rs # PostgreSQL implementation
    │   ├── sqlite_provider.rs # SQLite implementation
    │   └── mongodb_provider.rs # MongoDB implementation
    ├── errors.rs             # Error handling
    └── config.rs             # Configuration loading
``` 

## Implementation

### Database Service

The database service provides a centralized way to manage database connections and operations. It abstracts away the specific database implementation details and provides a consistent interface.

#### `src/services/database_service.rs`

```
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use crate::providers::{DatabaseProvider, PostgresProvider, SqliteProvider, MongodbProvider};
use crate::errors::AppError;
use crate::config::DatabaseConfig;

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnectionConfig {
    pub provider: String,
    pub connection_string: String,
    pub max_connections: u32,
    pub idle_timeout_seconds: u64,
    pub connect_timeout_seconds: u64,
}

/// Core database operations that all database implementations must support
#[async_trait]
pub trait DatabaseOperations: Send + Sync {
    /// Execute a raw query and return affected rows
    async fn execute_raw(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, AppError>;
    
    /// Execute a query and return rows as JSON
    async fn query_json(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<serde_json::Value>, AppError>;
    
    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, AppError>;
    
    /// Check if database is available
    async fn ping(&self) -> Result<bool, AppError>;
}

/// Transaction interface for all database types
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<(), AppError>;
    
    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<(), AppError>;
    
    /// Execute a query within the transaction
    async fn execute(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, AppError>;
    
    /// Query and return JSON results within the transaction
    async fn query_json(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<serde_json::Value>, AppError>;
}

/// Database service that manages connections and providers
pub struct DatabaseService {
    providers: Vec<Box<dyn DatabaseProvider>>,
    default_config: DbConnectionConfig,
    connections: RwLock<std::collections::HashMap<String, Arc<dyn DatabaseOperations>>>,
}

impl DatabaseService {
    /// Create a new database service with default providers
    pub fn new() -> Self {
        let mut service = Self {
            providers: Vec::new(),
            default_config: DbConnectionConfig {
                provider: "sqlite".to_string(),
                connection_string: "sqlite::memory:".to_string(),
                max_connections: 5,
                idle_timeout_seconds: 300,
                connect_timeout_seconds: 30,
            },
            connections: RwLock::new(std::collections::HashMap::new()),
        };
        
        // Register default providers
        service.register_provider(Box::new(PostgresProvider::new()));
        service.register_provider(Box::new(SqliteProvider::new()));
        service.register_provider(Box::new(MongodbProvider::new()));
        
        service
    }
    
    /// Register a database provider
    pub fn register_provider(&mut self, provider: Box<dyn DatabaseProvider>) -> &mut Self {
        self.providers.push(provider);
        self
    }
    
    /// Set the default configuration
    pub fn with_default_config(&mut self, config: DbConnectionConfig) -> &mut Self {
        self.default_config = config;
        self
    }
    
    /// Get or create a database connection
    pub async fn get_connection(&self, name: &str) -> Result<Arc<dyn DatabaseOperations>, AppError> {
        // Check if connection exists
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(name) {
                return Ok(conn.clone());
            }
        }
        
        // Create new connection
        let config = self.default_config.clone();
        let provider = self.find_provider(&config.provider)
            .ok_or_else(|| AppError::configuration(format!(
                "No database provider found for type: {}", config.provider
            )))?;
        
        let connection = provider.create_connection(&config).await?;
        
        // Store and return connection
        let mut connections = self.connections.write().await;
        let conn_arc = Arc::new(connection);
        connections.insert(name.to_string(), conn_arc.clone());
        
        Ok(conn_arc)
    }
    
    /// Find a provider by type
    fn find_provider(&self, provider_type: &str) -> Option<&Box<dyn DatabaseProvider>> {
        self.providers.iter().find(|p| p.supports(provider_type))
    }
    
    /// Close all connections
    pub async fn close_all(&self) -> Result<(), AppError> {
        let mut connections = self.connections.write().await;
        connections.clear();
        Ok(())
    }
}

impl Default for DatabaseService {
    fn default() -> Self {
        Self::new()
    }
}

### Repository Pattern

The repository pattern separates data access logic from business logic. Repositories provide type-safe CRUD operations for domain entities.

#### `src/repositories/repository.rs`

```
use async_trait::async_trait;
use std::marker::PhantomData;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Entity;
use crate::services::database_service::DatabaseOperations;

/// Generic repository trait for any entity type
#[async_trait]
pub trait Repository<E: Entity>: Send + Sync {
    /// Find an entity by its ID
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<E>, AppError>;
    
    /// Find all entities of this type
    async fn find_all(&self) -> Result<Vec<E>, AppError>;
    
    /// Save an entity (creates or updates)
    async fn save(&self, entity: &E) -> Result<E, AppError>;
    
    /// Delete an entity by ID
    async fn delete(&self, id: &Uuid) -> Result<bool, AppError>;
    
    /// Count total entities
    async fn count(&self) -> Result<usize, AppError>;
    
    /// Check if entity with ID exists
    async fn exists(&self, id: &Uuid) -> Result<bool, AppError>;
}

/// SQL implementation of the repository pattern
pub struct SqlRepository<E: Entity> {
    db: Arc<dyn DatabaseOperations>,
    table_name: String,
    _marker: PhantomData<E>,
}

impl<E: Entity + serde::de::DeserializeOwned + serde::Serialize> SqlRepository<E> {
    pub fn new(db: Arc<dyn DatabaseOperations>, table_name: &str) -> Self {
        Self {
            db,
            table_name: table_name.to_string(),
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<E: Entity + serde::de::DeserializeOwned + serde::Serialize + Send + Sync> Repository<E> for SqlRepository<E> {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<E>, AppError> {
        let query = format!("SELECT * FROM {} WHERE id = $1", self.table_name);
        let results = self.db.query_json(&query, &[&id.to_string()]).await?;
        
        if let Some(json) = results.into_iter().next() {
            let entity: E = serde_json::from_value(json)?;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }
    
    async fn find_all(&self) -> Result<Vec<E>, AppError> {
        let query = format!("SELECT * FROM {}", self.table_name);
        let results = self.db.query_json(&query, &[]).await?;
        
        let entities: Result<Vec<E>, serde_json::Error> = results
            .into_iter()
            .map(serde_json::from_value)
            .collect();
            
        Ok(entities?)
    }
    
    async fn save(&self, entity: &E) -> Result<E, AppError> {
        // Validate entity before saving
        entity.validate()?;
        
        let entity_json = serde_json::to_value(entity)?;
        
        // Check if entity exists
        let exists = self.exists(entity.id()).await?;
        
        if exists {
            // Update existing entity
            let fields: Vec<String> = entity_json.as_object()
                .unwrap()
                .keys()
                .filter(|k| *k != "id")
                .map(|k| format!("{} = jsonb_extract_path($2, '{}')", k, k))
                .collect();
                
            let update_fields = fields.join(", ");
            let query = format!(
                "UPDATE {} SET {} WHERE id = $1 RETURNING *",
                self.table_name, update_fields
            );
            
            let results = self.db.query_json(&query, &[&entity.id().to_string(), &entity_json.to_string()]).await?;
            let updated: E = serde_json::from_value(results.into_iter().next().unwrap())?;
            Ok(updated)
        } else {
            // Insert new entity
            let mut columns = vec!["id".to_string()];
            let mut placeholders = vec!["$1".to_string()];
            let mut values: Vec<String> = vec![entity.id().to_string()];
            
            let obj = entity_json.as_object().unwrap();
            let mut i = 2;
            
            for (key, value) in obj {
                if key != "id" {
                    columns.push(key.clone());
                    placeholders.push(format!("${}", i));
                    values.push(value.to_string());
                    i += 1;
                }
            }
            
            let query = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                self.table_name,
                columns.join(", "),
                placeholders.join(", ")
            );
            
            let params: Vec<&(dyn ToSql + Sync)> = values.iter()
                .map(|v| v as &(dyn ToSql + Sync))
                .collect();
                
            let results = self.db.query_json(&query, &params).await?;
            let inserted: E = serde_json::from_value(results.into_iter().next().unwrap())?;
            Ok(inserted)
        }
    }
    
    async fn delete(&self, id: &Uuid) -> Result<bool, AppError> {
        let query = format!("DELETE FROM {} WHERE id = $1", self.table_name);
        let affected = self.db.execute_raw(&query, &[&id.to_string()]).await?;
        Ok(affected > 0)
    }
    
    async fn count(&self) -> Result<usize, AppError> {
        let query = format!("SELECT COUNT(*) FROM {}", self.table_name);
        let results = self.db.query_json(&query, &[]).await?;
        
        if let Some(json) = results.into_iter().next() {
            let count: i64 = json.as_object()
                .and_then(|obj| obj.get("count"))
                .and_then(|val| val.as_i64())
                .unwrap_or(0);
                
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }
    
    async fn exists(&self, id: &Uuid) -> Result<bool, AppError> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE id = $1)", self.table_name);
        let results = self.db.query_json(&query, &[&id.to_string()]).await?;
        
        if let Some(json) = results.into_iter().next() {
            let exists: bool = json.as_object()
                .and_then(|obj| obj.get("exists"))
                .and_then(|val| val.as_bool())
                .unwrap_or(false);
                
            Ok(exists)
        } else {
            Ok(false)
        }
    }
}

### Entity Models

Entities are the domain objects that represent the data stored in the database.

#### `src/models/mod.rs`

```
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::errors::AppError;

/// Core trait for all domain entities
pub trait Entity: Clone + std::fmt::Debug {
    type Id;
    
    /// Get the entity's unique identifier
    fn id(&self) -> &Self::Id;
    
    /// Get the collection/table name for this entity type
    fn collection_name() -> String where Self: Sized;
    
    /// Validate entity before saving
    fn validate(&self) -> Result<(), AppError>;
}

pub mod user;
pub mod product;
```

#### `src/models/user.rs`

```
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Admin,
    Moderator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for User {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn collection_name() -> String {
        "users".to_string()
    }
    
    fn validate(&self) -> Result<(), AppError> {
        if self.username.is_empty() {
            return Err(AppError::validation("Username cannot be empty"));
        }
        
        if self.email.is_empty() {
            return Err(AppError::validation("Email cannot be empty"));
        }
        
        if !self.email.contains('@') {
            return Err(AppError::validation("Invalid email format"));
        }
        
        if self.display_name.is_empty() {
            return Err(AppError::validation("Display name cannot be empty"));
        }
        
        Ok(())
    }
}

impl User {
    pub fn new(username: String, email: String, display_name: String) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            display_name,
            password_hash: None,
            role: UserRole::User,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }
    
    pub fn with_password_hash(mut self, hash: String) -> Self {
        self.password_hash = Some(hash);
        self
    }
    
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
    
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}
```

#### `src/models/product.rs`

```
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub sku: String,
    pub in_stock: bool,
    pub stock_quantity: i32,
    pub category: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Product {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn collection_name() -> String {
        "products".to_string()
    }
    
    fn validate(&self) -> Result<(), AppError> {
        if self.name.is_empty() {
            return Err(AppError::validation("Product name cannot be empty"));
        }
        
        if self.price <= 0.0 {
            return Err(AppError::validation("Product price must be positive"));
        }
        
        if self.sku.is_empty() {
            return Err(AppError::validation("SKU cannot be empty"));
        }
        
        if self.stock_quantity < 0 {
            return Err(AppError::validation("Stock quantity cannot be negative"));
        }
        
        Ok(())
    }
}

impl Product {
    pub fn new(name: String, description: String, price: f64, sku: String, category: String) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            price,
            sku,
            in_stock: true,
            stock_quantity: 0,
            category,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn with_stock(mut self, quantity: i32) -> Self {
        self.stock_quantity = quantity;
        self.in_stock = quantity > 0;
        self
    }
    
    pub fn update_stock(&mut self, quantity: i32) {
        self.stock_quantity = quantity;
        self.in_stock = quantity > 0;
        self.updated_at = Utc::now();
    }
    
    pub fn update_price(&mut self, price: f64) -> Result<(), AppError> {
        if price <= 0.0 {
            return Err(AppError::validation("Product price must be positive"));
        }
        
        self.price = price;
        self.updated_at = Utc::now();
        Ok(())
    }
}
```

## Testing With Databases

## Key Concepts

## Database Best Practices

## Performance Optimization

## Advanced Topics

## Troubleshooting

## Service Implementation

The service layer acts as an intermediary between your application logic and repositories. Services implement the business logic using the repositories to access data.

### User Service Implementation

```
// src/services/user_service.rs
use crate::models::user::{User, UserRole};
use crate::repositories::user_repository::UserRepository;
use navius::di::{AutoService, Inject, ServiceRegistry};
use std::sync::Arc;

#[derive(Debug)]
pub struct UserServiceError {
    pub message: String,
    pub code: String,
}

impl UserServiceError {
    pub fn new(message: &str, code: &str) -> Self {
        UserServiceError {
            message: message.to_string(),
            code: code.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
        }
    }
}

pub trait UserService: Send + Sync {
    fn get_user_profile(&self, id: &str) -> Result<UserProfile, UserServiceError>;
    fn create_user(&self, user: User) -> Result<User, UserServiceError>;
    fn get_users_by_role(&self, role: UserRole) -> Result<Vec<UserProfile>, UserServiceError>;
    fn update_user_status(&self, id: &str, is_active: bool) -> Result<(), UserServiceError>;
    fn change_user_role(&self, id: &str, new_role: UserRole) -> Result<(), UserServiceError>;
}

#[derive(AutoService)]
pub struct UserServiceImpl {
    user_repository: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

impl UserService for UserServiceImpl {
    fn get_user_profile(&self, id: &str) -> Result<UserProfile, UserServiceError> {
        match self.user_repository.find_by_id(id) {
            Ok(Some(user)) => Ok(UserProfile::from(user)),
            Ok(None) => Err(UserServiceError::new(
                &format!("User with id {} not found", id),
                "USER_NOT_FOUND",
            )),
            Err(e) => Err(UserServiceError::new(
                &format!("Error fetching user: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn create_user(&self, user: User) -> Result<User, UserServiceError> {
        // Check if username or email already exists
        if let Ok(exists) = self
            .user_repository
            .exists_by_username_or_email(&user.username, &user.email)
        {
            if exists {
                return Err(UserServiceError::new(
                    "Username or email already exists",
                    "USER_ALREADY_EXISTS",
                ));
            }
        }

        // Validate the user
        if let Err(validation_error) = user.validate() {
            return Err(UserServiceError::new(
                &validation_error,
                "VALIDATION_ERROR",
            ));
        }

        match self.user_repository.save(user) {
            Ok(saved_user) => Ok(saved_user),
            Err(e) => Err(UserServiceError::new(
                &format!("Error creating user: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn get_users_by_role(&self, role: UserRole) -> Result<Vec<UserProfile>, UserServiceError> {
        match self.user_repository.find_by_role(role) {
            Ok(users) => Ok(users.into_iter().map(UserProfile::from).collect()),
            Err(e) => Err(UserServiceError::new(
                &format!("Error fetching users by role: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn update_user_status(&self, id: &str, is_active: bool) -> Result<(), UserServiceError> {
        match self.user_repository.find_by_id(id) {
            Ok(Some(mut user)) => {
                user.is_active = is_active;
                match self.user_repository.save(user) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(UserServiceError::new(
                        &format!("Error updating user status: {}", e),
                        "DATABASE_ERROR",
                    )),
                }
            }
            Ok(None) => Err(UserServiceError::new(
                &format!("User with id {} not found", id),
                "USER_NOT_FOUND",
            )),
            Err(e) => Err(UserServiceError::new(
                &format!("Error fetching user: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn change_user_role(&self, id: &str, new_role: UserRole) -> Result<(), UserServiceError> {
        match self.user_repository.find_by_id(id) {
            Ok(Some(mut user)) => {
                user.role = new_role;
                match self.user_repository.save(user) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(UserServiceError::new(
                        &format!("Error updating user role: {}", e),
                        "DATABASE_ERROR",
                    )),
                }
            }
            Ok(None) => Err(UserServiceError::new(
                &format!("User with id {} not found", id),
                "USER_NOT_FOUND",
            )),
            Err(e) => Err(UserServiceError::new(
                &format!("Error fetching user: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }
}

// Registration function for dependency injection
pub fn register_user_service(registry: &mut ServiceRegistry) {
    let user_repository = registry.resolve::<dyn UserRepository>().unwrap();
    let user_service = UserServiceImpl::new(user_repository);
    registry.register::<dyn UserService>(Arc::new(user_service));
}
```

### Product Service Implementation

```
// src/services/product_service.rs
use crate::models::product::{Product, ProductCategory};
use crate::repositories::product_repository::ProductRepository;
use navius::di::{AutoService, Inject, ServiceRegistry};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProductServiceError {
    pub message: String,
    pub code: String,
}

impl ProductServiceError {
    pub fn new(message: &str, code: &str) -> Self {
        ProductServiceError {
            message: message.to_string(),
            code: code.to_string(),
        }
    }
}

pub trait ProductService: Send + Sync {
    fn get_product(&self, id: &str) -> Result<Product, ProductServiceError>;
    fn create_product(&self, product: Product) -> Result<Product, ProductServiceError>;
    fn update_product(&self, product: Product) -> Result<Product, ProductServiceError>;
    fn delete_product(&self, id: &str) -> Result<(), ProductServiceError>;
    fn get_products_by_category(&self, category: ProductCategory) -> Result<Vec<Product>, ProductServiceError>;
    fn get_products_in_stock(&self) -> Result<Vec<Product>, ProductServiceError>;
    fn update_stock_quantity(&self, id: &str, quantity: i32) -> Result<(), ProductServiceError>;
}

#[derive(AutoService)]
pub struct ProductServiceImpl {
    product_repository: Arc<dyn ProductRepository>,
}

impl ProductServiceImpl {
    pub fn new(product_repository: Arc<dyn ProductRepository>) -> Self {
        Self { product_repository }
    }
}

impl ProductService for ProductServiceImpl {
    fn get_product(&self, id: &str) -> Result<Product, ProductServiceError> {
        match self.product_repository.find_by_id(id) {
            Ok(Some(product)) => Ok(product),
            Ok(None) => Err(ProductServiceError::new(
                &format!("Product with id {} not found", id),
                "PRODUCT_NOT_FOUND",
            )),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching product: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn create_product(&self, product: Product) -> Result<Product, ProductServiceError> {
        // Validate the product
        if let Err(validation_error) = product.validate() {
            return Err(ProductServiceError::new(
                &validation_error,
                "VALIDATION_ERROR",
            ));
        }

        // Check if SKU already exists
        if let Ok(exists) = self.product_repository.exists_by_sku(&product.sku) {
            if exists {
                return Err(ProductServiceError::new(
                    "Product with this SKU already exists",
                    "DUPLICATE_SKU",
                ));
            }
        }

        match self.product_repository.save(product) {
            Ok(saved_product) => Ok(saved_product),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error creating product: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn update_product(&self, product: Product) -> Result<Product, ProductServiceError> {
        // Validate the product
        if let Err(validation_error) = product.validate() {
            return Err(ProductServiceError::new(
                &validation_error,
                "VALIDATION_ERROR",
            ));
        }

        // Check if product exists
        match self.product_repository.find_by_id(&product.id) {
            Ok(Some(_)) => match self.product_repository.save(product) {
                Ok(updated_product) => Ok(updated_product),
                Err(e) => Err(ProductServiceError::new(
                    &format!("Error updating product: {}", e),
                    "DATABASE_ERROR",
                )),
            },
            Ok(None) => Err(ProductServiceError::new(
                &format!("Product with id {} not found", product.id),
                "PRODUCT_NOT_FOUND",
            )),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching product: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn delete_product(&self, id: &str) -> Result<(), ProductServiceError> {
        match self.product_repository.find_by_id(id) {
            Ok(Some(_)) => match self.product_repository.delete(id) {
                Ok(_) => Ok(()),
                Err(e) => Err(ProductServiceError::new(
                    &format!("Error deleting product: {}", e),
                    "DATABASE_ERROR",
                )),
            },
            Ok(None) => Err(ProductServiceError::new(
                &format!("Product with id {} not found", id),
                "PRODUCT_NOT_FOUND",
            )),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching product: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn get_products_by_category(&self, category: ProductCategory) -> Result<Vec<Product>, ProductServiceError> {
        match self.product_repository.find_by_category(category) {
            Ok(products) => Ok(products),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching products by category: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn get_products_in_stock(&self) -> Result<Vec<Product>, ProductServiceError> {
        match self.product_repository.find_in_stock() {
            Ok(products) => Ok(products),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching in-stock products: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }

    fn update_stock_quantity(&self, id: &str, quantity: i32) -> Result<(), ProductServiceError> {
        match self.product_repository.find_by_id(id) {
            Ok(Some(mut product)) => {
                product.stock_quantity = quantity;
                product.in_stock = quantity > 0;
                
                match self.product_repository.save(product) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ProductServiceError::new(
                        &format!("Error updating stock quantity: {}", e),
                        "DATABASE_ERROR",
                    )),
                }
            }
            Ok(None) => Err(ProductServiceError::new(
                &format!("Product with id {} not found", id),
                "PRODUCT_NOT_FOUND",
            )),
            Err(e) => Err(ProductServiceError::new(
                &format!("Error fetching product: {}", e),
                "DATABASE_ERROR",
            )),
        }
    }
}

// Registration function for dependency injection
pub fn register_product_service(registry: &mut ServiceRegistry) {
    let product_repository = registry.resolve::<dyn ProductRepository>().unwrap();
    let product_service = ProductServiceImpl::new(product_repository);
    registry.register::<dyn ProductService>(Arc::new(product_service));
}
```

## Database Migrations

Migrations help you evolve your database schema over time. Let's implement a simple migration system for our database.

### Migration Manager

```
// src/utils/migrations.rs
use crate::services::database::{DatabaseError, DatabaseService};
use navius::config::Config;
use navius::logger;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Migration {
    pub id: String,
    pub name: String,
    pub description: String,
    pub up_sql: String,
    pub down_sql: String,
    pub executed_at: Option<chrono::DateTime<Utc>>,
}

impl Migration {
    pub fn new(name: &str, description: &str, up_sql: &str, down_sql: &str) -> Self {
        Migration {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            up_sql: up_sql.to_string(),
            down_sql: down_sql.to_string(),
            executed_at: None,
        }
    }
}

pub struct MigrationManager {
    db_service: Arc<DatabaseService>,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self {
            db_service,
            migrations: Vec::new(),
        }
    }

    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    pub fn initialize(&self) -> Result<(), DatabaseError> {
        let conn = self.db_service.get_connection("default")?;
        
        // Create migrations table if it doesn't exist
        let create_migrations_table = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                executed_at TIMESTAMP WITH TIME ZONE NOT NULL
            )
        "#;
        
        conn.execute_raw(create_migrations_table, &[])?;
        
        Ok(())
    }

    pub fn get_executed_migrations(&self) -> Result<Vec<String>, DatabaseError> {
        let conn = self.db_service.get_connection("default")?;
        
        let query = "SELECT id FROM migrations";
        let results = conn.query_json(query, &[])?;
        
        let executed_migrations: Vec<String> = results
            .into_iter()
            .map(|v| v["id"].as_str().unwrap_or("").to_string())
            .collect();
        
        Ok(executed_migrations)
    }

    pub fn run_migrations(&self) -> Result<u32, DatabaseError> {
        self.initialize()?;
        
        let executed_migrations = self.get_executed_migrations()?;
        let mut count = 0;
        
        for migration in &self.migrations {
            if !executed_migrations.contains(&migration.id) {
                logger::info!("Running migration: {}", migration.name);
                
                let conn = self.db_service.get_connection("default")?;
                let transaction = conn.begin_transaction()?;
                
                // Execute the migration
                transaction.execute_raw(&migration.up_sql, &[])?;
                
                // Record the migration
                let record_query = r#"
                    INSERT INTO migrations (id, name, description, executed_at)
                    VALUES ($1, $2, $3, NOW())
                "#;
                
                transaction.execute_raw(
                    record_query,
                    &[&migration.id, &migration.name, &migration.description],
                )?;
                
                transaction.commit()?;
                count += 1;
                
                logger::info!("Migration completed: {}", migration.name);
            }
        }
        
        logger::info!("Applied {} migrations", count);
        Ok(count)
    }

    pub fn rollback_last_migration(&self) -> Result<Option<String>, DatabaseError> {
        self.initialize()?;
        
        let conn = self.db_service.get_connection("default")?;
        
        let query = "SELECT id, name FROM migrations ORDER BY executed_at DESC LIMIT 1";
        let results = conn.query_json(query, &[])?;
        
        if results.is_empty() {
            logger::info!("No migrations to rollback");
            return Ok(None);
        }
        
        let migration_id = results[0]["id"].as_str().unwrap_or("").to_string();
        let migration_name = results[0]["name"].as_str().unwrap_or("").to_string();
        
        // Find the migration
        if let Some(migration) = self.migrations.iter().find(|m| m.id == migration_id) {
            logger::info!("Rolling back migration: {}", migration_name);
            
            let transaction = conn.begin_transaction()?;
            
            // Execute the down migration
            transaction.execute_raw(&migration.down_sql, &[])?;
            
            // Remove the migration record
            let delete_query = "DELETE FROM migrations WHERE id = $1";
            transaction.execute_raw(delete_query, &[&migration_id])?;
            
            transaction.commit()?;
            
            logger::info!("Rollback completed: {}", migration_name);
            Ok(Some(migration_name))
        } else {
            logger::error!("Migration not found in registry: {}", migration_id);
            Ok(None)
        }
    }
}

// Example migration setup function
pub fn setup_migrations(db_service: Arc<DatabaseService>) -> MigrationManager {
    let mut manager = MigrationManager::new(db_service);
    
    // Create users table
    let users_migration = Migration::new(
        "create_users_table",
        "Create the users table",
        r#"
            CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(36) PRIMARY KEY,
                username VARCHAR(50) UNIQUE NOT NULL,
                email VARCHAR(100) UNIQUE NOT NULL,
                display_name VARCHAR(100) NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                role VARCHAR(20) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            
            CREATE INDEX idx_users_username ON users(username);
            CREATE INDEX idx_users_email ON users(email);
            CREATE INDEX idx_users_role ON users(role);
        "#,
        r#"
            DROP TABLE IF EXISTS users;
        "#
    );
    
    // Create products table
    let products_migration = Migration::new(
        "create_products_table",
        "Create the products table",
        r#"
            CREATE TABLE IF NOT EXISTS products (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                description TEXT,
                price DECIMAL(10, 2) NOT NULL,
                sku VARCHAR(50) UNIQUE NOT NULL,
                in_stock BOOLEAN NOT NULL DEFAULT FALSE,
                stock_quantity INTEGER NOT NULL DEFAULT 0,
                category VARCHAR(50) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            
            CREATE INDEX idx_products_sku ON products(sku);
            CREATE INDEX idx_products_category ON products(category);
            CREATE INDEX idx_products_in_stock ON products(in_stock);
        "#,
        r#"
            DROP TABLE IF EXISTS products;
        "#
    );
    
    manager.add_migration(users_migration);
    manager.add_migration(products_migration);
    
    manager
}
```

## API Endpoints

Now let's implement the API endpoints to expose our database functionality.

### User API

```
// src/api/user_api.rs
use crate::models::user::{User, UserRole};
use crate::services::user_service::{UserProfile, UserService, UserServiceError};
use navius::di::Inject;
use navius::http::{Body, Request, Response, StatusCode};
use navius::router::{delete, get, post, put, RouteHandler};
use navius::web::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub message: String,
    pub code: String,
}

impl From<UserServiceError> for ApiErrorResponse {
    fn from(error: UserServiceError) -> Self {
        ApiErrorResponse {
            message: error.message,
            code: error.code,
        }
    }
}

pub struct UserApi {
    user_service: Arc<dyn UserService>,
}

impl UserApi {
    pub fn new(user_service: Arc<dyn UserService>) -> Self {
        Self { user_service }
    }
    
    pub fn register_routes(&self) -> Vec<RouteHandler> {
        vec![
            get("/api/users", self.get_users()),
            get("/api/users/:id", self.get_user_by_id()),
            post("/api/users", self.create_user()),
            put("/api/users/:id/status", self.update_user_status()),
            put("/api/users/:id/role", self.update_user_role()),
        ]
    }
    
    fn get_users(&self) -> RouteHandler {
        let user_service = Arc::clone(&self.user_service);
        
        RouteHandler::new(move |req: Request<Body>| {
            let role_param = req.query().get("role");
            
            let result = match role_param {
                Some(role_str) => {
                    let role = match role_str.as_str() {
                        "admin" => UserRole::Admin,
                        "manager" => UserRole::Manager,
                        "user" => UserRole::User,
                        _ => UserRole::User,
                    };
                    
                    user_service.get_users_by_role(role)
                },
                None => {
                    // This would need to be implemented in the service
                    Err(UserServiceError::new(
                        "Role parameter is required", 
                        "MISSING_PARAMETER"
                    ))
                }
            };
            
            match result {
                Ok(users) => Response::json(&users).unwrap(),
                Err(e) => {
                    let error: ApiErrorResponse = e.into();
                    Response::json_with_status(&error, StatusCode::BAD_REQUEST).unwrap()
                }
            }
        })
    }
    
    fn get_user_by_id(&self) -> RouteHandler {
        let user_service = Arc::clone(&self.user_service);
        
        RouteHandler::new(move |req: Request<Body>| {
            let id = req.params().get("id").unwrap_or_default();
            
            match user_service.get_user_profile(id) {
                Ok(user) => Response::json(&user).unwrap(),
                Err(e) => {
                    let error: ApiErrorResponse = e.into();
                    let status = if e.code == "USER_NOT_FOUND" {
                        StatusCode::NOT_FOUND
                    } else {
                        StatusCode::INTERNAL_SERVER_ERROR
                    };
                    
                    Response::json_with_status(&error, status).unwrap()
                }
            }
        })
    }
    
    fn create_user(&self) -> RouteHandler {
        let user_service = Arc::clone(&self.user_service);
        
        RouteHandler::new(move |mut req: Request<Body>| {
            let create_req: Result<CreateUserRequest, _> = req.json();
            
            match create_req {
                Ok(data) => {
                    // Create user from request
                    let user = User {
                        id: uuid::Uuid::new_v4().to_string(),
                        username: data.username,
                        email: data.email,
                        display_name: data.display_name,
                        password_hash: hash_password(&data.password), // Implement this function
                        role: data.role.map_or(UserRole::User, |r| {
                            match r.as_str() {
                                "admin" => UserRole::Admin,
                                "manager" => UserRole::Manager,
                                _ => UserRole::User,
                            }
                        }),
                        is_active: true,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    
                    match user_service.create_user(user) {
                        Ok(created_user) => {
                            let profile = UserProfile::from(created_user);
                            Response::json_with_status(&profile, StatusCode::CREATED).unwrap()
                        },
                        Err(e) => {
                            let error: ApiErrorResponse = e.into();
                            Response::json_with_status(&error, StatusCode::BAD_REQUEST).unwrap()
                        }
                    }
                },
                Err(_) => {
                    let error = ApiErrorResponse {
                        message: "Invalid request body".to_string(),
                        code: "INVALID_REQUEST".to_string(),
                    };
                    
                    Response::json_with_status(&error, StatusCode::BAD_REQUEST).unwrap()
                }
            }
        })
    }
    
    fn update_user_status(&self) -> RouteHandler {
        let user_service = Arc::clone(&self.user_service);
        
        RouteHandler::new(move |mut req: Request<Body>| {
            let id = req.params().get("id").unwrap_or_default();
            let status_req: Result<UpdateUserStatusRequest, _> = req.json();
            
            match status_req {
                Ok(data) => {
                    match user_service.update_user_status(id, data.is_active) {
                        Ok(_) => Response::empty(StatusCode::OK),
                        Err(e) => {
                            let error: ApiErrorResponse = e.into();
                            let status = if e.code == "USER_NOT_FOUND" {
                                StatusCode::NOT_FOUND
                            } else {
                                StatusCode::INTERNAL_SERVER_ERROR
                            };
                            
                            Response::json_with_status(&error, status).unwrap()
                        }
                    }
                },
                Err(_) => {
                    let error = ApiErrorResponse {
                        message: "Invalid request body".to_string(),
                        code: "INVALID_REQUEST".to_string(),
                    };
                    
                    Response::json_with_status(&error, StatusCode::BAD_REQUEST).unwrap()
                }
            }
        })
    }
    
    fn update_user_role(&self) -> RouteHandler {
        let user_service = Arc::clone(&self.user_service);
        
        RouteHandler::new(move |mut req: Request<Body>| {
            let id = req.params().get("id").unwrap_or_default();
            let role_req: Result<UpdateUserRoleRequest, _> = req.json();
            
            match role_req {
                Ok(data) => {
                    let role = match data.role.as_str() {
                        "admin" => UserRole::Admin,
                        "manager" => UserRole::Manager,
                        "user" => UserRole::User,
                        _ => UserRole::User,
                    };
                    
                    match user_service.change_user_role(id, role) {
                        Ok(_) => Response::empty(StatusCode::OK),
                        Err(e) => {
                            let error: ApiErrorResponse = e.into();
                            let status = if e.code == "USER_NOT_FOUND" {
                                StatusCode::NOT_FOUND
                            } else {
                                StatusCode::INTERNAL_SERVER_ERROR
                            };
                            
                            Response::json_with_status(&error, status).unwrap()
                        }
                    }
                },
                Err(_) => {
                    let error = ApiErrorResponse {
                        message: "Invalid request body".to_string(),
                        code: "INVALID_REQUEST".to_string(),
                    };
                    
                    Response::json_with_status(&error, StatusCode::BAD_REQUEST).unwrap()
                }
            }
        })
    }
}

// Helper function for password hashing (simplified for example)
fn hash_password(password: &str) -> String {
    // In a real application, use a proper password hashing algorithm like bcrypt
    // This is a simplified example
    format!("hashed_{}", password)
}
```
``` 
//! Database and Cache Integration Example
//!
//! This example demonstrates the integration of database and caching layers
//! in the Navius framework, showcasing various caching strategies and database
//! transaction management.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use navius_cache::{
    config::CacheConfig,
    invalidation::CacheInvalidator,
    operations::Cache,
};
use navius_cache_redis::{connection::RedisConnectionManager, operations::RedisCache};
use navius_core::{
    config::Config,
    di::{Application, ApplicationBuilder, AsyncLifecycle, ComponentScope, Environment, Lifecycle},
    error::{Error, Result},
};
use navius_db::{
    connection::{ConnectionManager, DbConfig},
    repository::Repository,
    transaction::{Transaction, TransactionManager},
};
use navius_db_postgres::{connection::PostgresConnectionManager, query::PostgresQuery};
use navius_http::{
    routing::{Route, Router},
    server::{HttpServer, ServerConfig},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tracing::info;
use uuid::Uuid;

// Product entity
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: Uuid,
    name: String,
    description: String,
    price: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ProductRepository trait defines operations for product storage
#[async_trait]
trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>>;
    async fn find_all(&self) -> Result<Vec<Product>>;
    async fn save(&self, product: &Product) -> Result<Product>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

// Define a simple pattern invalidation strategy
struct PatternInvalidationStrategy;

impl PatternInvalidationStrategy {
    fn new() -> Self {
        Self
    }
    
    async fn invalidate_pattern(&self, cache: &RedisCache, pattern: &str) -> Result<()> {
        // In a real implementation, we would use Redis SCAN + DEL commands
        // For this example, we'll just log the action
        info!("Invalidating cache pattern: {}", pattern);
        Ok(())
    }
}

impl Clone for PatternInvalidationStrategy {
    fn clone(&self) -> Self {
        Self
    }
}

// ProductCacheManager handles cache operations for products
#[derive(Clone)]
struct ProductCacheManager {
    cache: RedisCache,
    invalidator: PatternInvalidationStrategy,
}

impl ProductCacheManager {
    fn new(cache: RedisCache) -> Self {
        let invalidator = PatternInvalidationStrategy::new();
        Self { cache, invalidator }
    }

    async fn get_product(&self, id: Uuid) -> Result<Option<Product>> {
        let key = format!("product:{}", id);
        self.cache.get(&key).await
    }

    async fn set_product(&self, product: &Product) -> Result<()> {
        let key = format!("product:{}", product.id);
        self.cache.set(&key, product, None).await
    }

    async fn invalidate_product(&self, id: Uuid) -> Result<()> {
        let key = format!("product:{}", id);
        self.cache.delete(&key).await
    }

    async fn invalidate_all_products(&self) -> Result<()> {
        self.invalidator
            .invalidate_pattern(&self.cache, "product:*")
            .await
    }

    async fn get_cache_stats(&self) -> Result<HashMap<String, i64>> {
        // In a real implementation, we would get actual stats from Redis
        // For this example, we just return some fake stats
        let mut stats = HashMap::new();
        stats.insert("hits".to_string(), 42);
        stats.insert("misses".to_string(), 12);
        stats.insert("keys".to_string(), 25);
        Ok(stats)
    }
}

// PostgresProductRepository implements ProductRepository using PostgreSQL
struct PostgresProductRepository {
    connection_manager: PostgresConnectionManager,
}

impl PostgresProductRepository {
    fn new(connection_manager: PostgresConnectionManager) -> Self {
        Self { connection_manager }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        let conn = self.connection_manager.get_connection().await?;
        let query = PostgresQuery::new("SELECT * FROM products WHERE id = $1");
        let result = query.execute_query_one(&conn, &[&id]).await?;

        Ok(result.map(|row| Product {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            price: row.get("price"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_all(&self) -> Result<Vec<Product>> {
        let conn = self.connection_manager.get_connection().await?;
        let query = PostgresQuery::new("SELECT * FROM products");
        let results = query.execute_query(&conn, &[]).await?;

        let products = results
            .into_iter()
            .map(|row| Product {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                price: row.get("price"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(products)
    }

    async fn save(&self, product: &Product) -> Result<Product> {
        let conn = self.connection_manager.get_connection().await?;
        
        // For new products, use INSERT
        if product.id == Uuid::nil() {
            let new_id = Uuid::new_v4();
            let now = Utc::now();
            
            let query = PostgresQuery::new(
                "INSERT INTO products (id, name, description, price, created_at, updated_at) 
                VALUES ($1, $2, $3, $4, $5, $6) 
                RETURNING *"
            );
            
            let result = query
                .execute_query_one(
                    &conn, 
                    &[
                        &new_id, 
                        &product.name, 
                        &product.description, 
                        &product.price, 
                        &now, 
                        &now
                    ]
                )
                .await?
                .ok_or_else(|| Error::new("Failed to insert product"))?;
            
            return Ok(Product {
                id: result.get("id"),
                name: result.get("name"),
                description: result.get("description"),
                price: result.get("price"),
                created_at: result.get("created_at"),
                updated_at: result.get("updated_at"),
            });
        }
        
        // For existing products, use UPDATE
        let now = Utc::now();
        let query = PostgresQuery::new(
            "UPDATE products 
            SET name = $2, description = $3, price = $4, updated_at = $5 
            WHERE id = $1 
            RETURNING *"
        );
        
        let result = query
            .execute_query_one(
                &conn, 
                &[
                    &product.id, 
                    &product.name, 
                    &product.description, 
                    &product.price, 
                    &now
                ]
            )
            .await?
            .ok_or_else(|| Error::new("Product not found"))?;
        
        Ok(Product {
            id: result.get("id"),
            name: result.get("name"),
            description: result.get("description"),
            price: result.get("price"),
            created_at: result.get("created_at"),
            updated_at: result.get("updated_at"),
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let conn = self.connection_manager.get_connection().await?;
        let query = PostgresQuery::new("DELETE FROM products WHERE id = $1 RETURNING id");
        let result = query.execute_query_one(&conn, &[&id]).await?;
        
        Ok(result.is_some())
    }
}

// CachedProductRepository combines database and cache operations
struct CachedProductRepository {
    db_repository: PostgresProductRepository,
    cache_manager: ProductCacheManager,
}

impl CachedProductRepository {
    fn new(db_repository: PostgresProductRepository, cache_manager: ProductCacheManager) -> Self {
        Self {
            db_repository,
            cache_manager,
        }
    }
}

#[async_trait]
impl ProductRepository for CachedProductRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        // Cache-aside (lazy loading) pattern:
        // 1. Check cache first
        if let Some(product) = self.cache_manager.get_product(id).await? {
            return Ok(Some(product));
        }
        
        // 2. If not in cache, get from database
        let product = self.db_repository.find_by_id(id).await?;
        
        // 3. Store in cache for future requests
        if let Some(ref product) = product {
            self.cache_manager.set_product(product).await?;
        }
        
        Ok(product)
    }

    async fn find_all(&self) -> Result<Vec<Product>> {
        // For simplicity, we're bypassing the cache for find_all
        // In a real implementation, we might cache the list or individual products
        self.db_repository.find_all().await
    }

    async fn save(&self, product: &Product) -> Result<Product> {
        // Write-through pattern:
        // 1. Save to database first
        let saved_product = self.db_repository.save(product).await?;
        
        // 2. Update cache with the saved product
        self.cache_manager.set_product(&saved_product).await?;
        
        Ok(saved_product)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        // 1. Delete from database
        let result = self.db_repository.delete(id).await?;
        
        // 2. Invalidate cache
        if result {
            self.cache_manager.invalidate_product(id).await?;
        }
        
        Ok(result)
    }
}

// AppServer integrates everything together
struct AppServer {
    server: HttpServer,
    product_repository: Arc<dyn ProductRepository>,
    cache_manager: ProductCacheManager,
}

impl AppServer {
    fn new(
        product_repository: Arc<dyn ProductRepository>, 
        cache_manager: ProductCacheManager
    ) -> Self {
        let config = ServerConfig::default().with_address("127.0.0.1:8080".to_string());
        let server = HttpServer::new(config);
        
        Self {
            server,
            product_repository,
            cache_manager,
        }
    }

    fn configure_routes(&mut self) -> Result<()> {
        let mut router = Router::new();
        
        // Health check endpoint
        router.add_route(Route::get("/health", |_req| {
            Ok(format!("{{\"status\": \"OK\", \"timestamp\": \"{}\"}}", Utc::now()))
        }));
        
        // CRUD operations for products
        let repo = self.product_repository.clone();
        router.add_route(Route::get("/api/products", move |_req| {
            let repo = repo.clone();
            async move {
                let products = repo.find_all().await?;
                Ok(serde_json::to_string(&products)?)
            }
        }));
        
        let repo = self.product_repository.clone();
        router.add_route(Route::get("/api/products/:id", move |req| {
            let repo = repo.clone();
            async move {
                let id = req.param("id")
                    .ok_or_else(|| Error::new("Missing product ID"))?;
                
                let id = Uuid::parse_str(id)
                    .map_err(|_| Error::new("Invalid UUID format"))?;
                
                let product = repo.find_by_id(id).await?;
                match product {
                    Some(p) => Ok(serde_json::to_string(&p)?),
                    None => {
                        let mut response = navius_http::response::Response::new(404);
                        response.set_body("{\"error\": \"Product not found\"}");
                        Err(Error::from_response(response))
                    }
                }
            }
        }));
        
        let repo = self.product_repository.clone();
        router.add_route(Route::post("/api/products", move |req| {
            let repo = repo.clone();
            async move {
                let body = req.body_string().await?;
                let mut product: Product = serde_json::from_str(&body)?;
                product.id = Uuid::nil(); // Ensure this is a new product
                
                let saved_product = repo.save(&product).await?;
                Ok(serde_json::to_string(&saved_product)?)
            }
        }));
        
        let repo = self.product_repository.clone();
        router.add_route(Route::put("/api/products/:id", move |req| {
            let repo = repo.clone();
            async move {
                let id = req.param("id")
                    .ok_or_else(|| Error::new("Missing product ID"))?;
                
                let id = Uuid::parse_str(id)
                    .map_err(|_| Error::new("Invalid UUID format"))?;
                
                let body = req.body_string().await?;
                let mut product: Product = serde_json::from_str(&body)?;
                product.id = id; // Ensure we use the ID from the URL
                
                let saved_product = repo.save(&product).await?;
                Ok(serde_json::to_string(&saved_product)?)
            }
        }));
        
        let repo = self.product_repository.clone();
        router.add_route(Route::delete("/api/products/:id", move |req| {
            let repo = repo.clone();
            async move {
                let id = req.param("id")
                    .ok_or_else(|| Error::new("Missing product ID"))?;
                
                let id = Uuid::parse_str(id)
                    .map_err(|_| Error::new("Invalid UUID format"))?;
                
                let success = repo.delete(id).await?;
                if success {
                    Ok("{\"success\": true}".to_string())
                } else {
                    let mut response = navius_http::response::Response::new(404);
                    response.set_body("{\"error\": \"Product not found\"}");
                    Err(Error::from_response(response))
                }
            }
        }));
        
        // Cache stats endpoint
        let cache_manager = self.cache_manager.clone();
        router.add_route(Route::get("/api/cache/stats", move |_req| {
            let cache_manager = cache_manager.clone();
            async move {
                let stats = cache_manager.get_cache_stats().await?;
                Ok(serde_json::to_string(&stats)?)
            }
        }));
        
        self.server.set_router(router);
        Ok(())
    }
}

#[async_trait]
impl AsyncLifecycle for AppServer {
    async fn on_initialize_async(&self) -> Result<()> {
        info!("Starting AppServer");
        self.server.start().await?;
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        info!("Stopping AppServer");
        self.server.stop().await?;
        Ok(())
    }
}

// Custom schema setup for PostgreSQL
async fn setup_database_schema(connection_manager: &PostgresConnectionManager) -> Result<()> {
    let conn = connection_manager.get_connection().await?;
    
    // Create products table
    let create_table_query = PostgresQuery::new(
        "CREATE TABLE IF NOT EXISTS products (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            price DOUBLE PRECISION NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL
        )"
    );
    
    create_table_query.execute_update(&conn, &[]).await?;
    info!("Database schema initialized successfully");
    
    Ok(())
}

// Generate some sample products
async fn seed_sample_data(product_repository: &dyn ProductRepository) -> Result<()> {
    let products = product_repository.find_all().await?;
    
    // Only seed if the table is empty
    if !products.is_empty() {
        info!("Sample data already exists, skipping seeding");
        return Ok(());
    }
    
    info!("Seeding sample product data");
    
    let mut rng = rand::thread_rng();
    
    let product_names = [
        "Smart Watch",
        "Bluetooth Headphones",
        "Wireless Mouse",
        "Mechanical Keyboard",
        "External SSD",
        "USB-C Hub",
        "Laptop Stand",
        "Webcam Pro",
        "Portable Speaker",
        "Power Bank",
    ];
    
    for name in product_names.iter() {
        let product = Product {
            id: Uuid::nil(), // Will be assigned during save
            name: name.to_string(),
            description: format!("Description for {}", name),
            price: (rng.gen::<f64>() * 200.0 + 10.0).round() * 0.01, // Random price between $10 and $210
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        product_repository.save(&product).await?;
    }
    
    info!("Sample data seeded successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Database + Cache Integration Example");
    
    // Create the application
    let mut app = ApplicationBuilder::new()
        .with_environment(Environment::Development)
        .build();
    
    // Configure database
    let db_config = DbConfig::new("postgres://navius:navius_password@localhost:5432/navius_demo");
    let pg_connection_manager = PostgresConnectionManager::new(db_config);
    
    // Configure cache
    let cache_config = CacheConfig::new(
        "redis",
        "redis://localhost:6379",
        "navius:cache:",
        Duration::from_secs(3600),
    );
    let redis_connection_manager = RedisConnectionManager::new(cache_config).await?;
    let redis_cache = RedisCache::new(redis_connection_manager);
    
    // Set up components
    let product_cache_manager = ProductCacheManager::new(redis_cache);
    let postgres_product_repository = PostgresProductRepository::new(pg_connection_manager.clone());
    
    let cached_repository = CachedProductRepository::new(
        postgres_product_repository,
        product_cache_manager.clone(),
    );
    
    // Register the repository as a component
    let repository: Arc<dyn ProductRepository> = Arc::new(cached_repository);
    
    // Create and register the AppServer
    let mut app_server = AppServer::new(repository.clone(), product_cache_manager);
    app_server.configure_routes()?;
    
    app.register_component("appServer", app_server, ComponentScope::Singleton);
    
    // Initialize the database schema
    setup_database_schema(&pg_connection_manager).await?;
    
    // Seed sample data
    seed_sample_data(repository.as_ref()).await?;
    
    // Start the application
    app.start().await?;
    
    // Wait for user to terminate
    info!("Application started successfully. Press Ctrl+C to stop.");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    
    // Shutdown the application
    info!("Shutting down the application...");
    app.stop().await?;
    
    info!("Application shutdown complete");
    Ok(())
} 
// Core service module exports
pub mod cache_provider;
pub mod cache_service;
pub mod database_interface;
pub mod database_service;
pub mod error;
pub mod health;
pub mod health_dashboard;
pub mod health_discovery;
pub mod health_indicators;
pub mod health_provider;
pub mod memory_cache;
pub mod memory_database;
pub mod memory_repository;
pub mod redis_cache;
pub mod repository_service;
pub mod service_traits;

// Re-export key components
pub use cache_provider::{
    CacheConfig, CacheError, CacheOperations, CacheProvider, CacheProviderRegistry, CacheStats,
    EvictionPolicy,
};
pub use cache_service::{CacheHelpers, CacheService};
pub use database_interface::{
    DatabaseConfig, DatabaseOperations, DatabaseProvider, DatabaseProviderRegistry,
};
pub use database_service::{DatabaseService, InMemoryDatabaseServiceProvider};
pub use health::HealthService;
pub use health_dashboard::{
    HealthDashboardConfig, HealthDashboardService, HealthStatusHistoryEntry,
};
pub use health_discovery::{
    DynamicHealthIndicatorProvider, HealthDiscoveryService, HealthIndicatorDiscovery,
};
pub use health_indicators::{
    CacheHealthIndicator, CoreHealthIndicatorProvider, DatabaseHealthIndicator,
    DatabaseHealthIndicatorProvider, DiskSpaceHealthIndicator, EnvironmentHealthIndicator,
    ServiceRegistryHealthIndicator,
};
pub use health_provider::{
    HealthConfig, HealthIndicator, HealthIndicatorProvider, HealthIndicatorProviderRegistry,
    HealthServiceV2,
};
pub use memory_cache::InMemoryCacheProvider;
pub use memory_database::{InMemoryDatabase, InMemoryDatabaseProvider};
pub use memory_repository::{
    InMemoryRepository, InMemoryRepositoryProvider, register_memory_repository_provider,
};
pub use redis_cache::RedisCacheProvider;
pub use repository_service::{GenericRepository, RepositoryService};
pub use service_traits::{Lifecycle, Service, ServiceProvider, ServiceRegistry};

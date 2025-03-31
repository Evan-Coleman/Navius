# Database and Cache Integration Example

This example demonstrates the integration of database and caching layers in the Navius framework. It illustrates how to effectively combine database operations with caching strategies to improve performance while maintaining data consistency.

## Crates Used

- `navius-core`: For dependency injection, component registry, and configuration
- `navius-http`: For HTTP server, routing, and request/response handling
- `navius-db`: For database abstraction and operations
- `navius-db-postgres`: For PostgreSQL implementation
- `navius-cache`: For caching abstraction and operations
- `navius-cache-redis`: For Redis implementation

## Key Features Demonstrated

1. **Cache Strategies**:
   - Cache-Aside: Read from cache first, falling back to database
   - Write-Through: Update both cache and database
   - Cache Invalidation: Clear cache entries when data changes

2. **Database Transactions**:
   - Transaction management
   - Rollback on error
   - Integration with cache operations

3. **Dependency Injection**:
   - Component registration with different scopes
   - Type-safe dependency resolution
   - Lifecycle hooks for initialization and cleanup

4. **HTTP Server Endpoints**:
   - CRUD operations for product entity
   - Cache status and statistics endpoint
   - Database status endpoint

## Running the Example

```bash
# Start Redis and PostgreSQL instances
docker-compose up -d

# Run the example application
cd workspace_migration/examples/integration/db-cache
cargo run
```

The server will start on `127.0.0.1:8080` with the following endpoints:

- `/health`: Returns the health status of all components
- `/api/products`: CRUD operations for products
- `/api/cache/stats`: Returns cache statistics

## Caching Strategies Demonstrated

### 1. Cache-Aside (Lazy Loading)

When retrieving products, the application first checks the cache. If the data is not in the cache, it retrieves it from the database and then stores it in the cache for future requests.

### 2. Write-Through

When creating, updating, or deleting products, the application updates both the database and the cache simultaneously to ensure consistency.

### 3. Cache Invalidation

The application implements pattern-based invalidation to clear related cache entries when data is modified. This ensures that cache values don't become stale.

## Key Code Concepts

- **Repository Pattern**: Abstracting database operations
- **Cache Manager**: Handling cache strategies and invalidation
- **Component Registry**: Managing component lifecycle and dependencies
- **Transaction Integration**: Coordinating database and cache operations within transactions
- **Monitoring and Statistics**: Collecting and exposing metrics

This example demonstrates real-world patterns for integrating database and cache layers in a production application using the Navius framework. 
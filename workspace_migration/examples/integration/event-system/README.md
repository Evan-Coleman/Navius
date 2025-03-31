# Event System Integration Example

This example demonstrates the implementation of an event-driven architecture using the Navius framework, showcasing event handling, streaming, and real-time data processing capabilities.

## Key Features

- **Event-driven architecture** with publishing and subscribing mechanisms
- **HTTP integration** for triggering events and real-time updates
- **Event persistence** in PostgreSQL database
- **Event handling** through various event types and handlers
- **Real-time monitoring** with metrics and status dashboards

## Crates Used

- `navius_core` - Core utilities and application framework
- `navius_event` - Event system abstractions and interfaces
- `navius_http` - HTTP server and routing capabilities
- `navius_db` - Database abstraction layer
- `navius_db_postgres` - PostgreSQL implementation of database layer

## Running the Example

### Prerequisites

- PostgreSQL database running on `localhost:5433`
- Database named `navius_events` with user `navius` and password `navius_password`

You can start PostgreSQL using Docker:

```bash
docker run -d --name postgres-navius-events -p 5433:5432 \
  -e POSTGRES_USER=navius -e POSTGRES_PASSWORD=navius_password \
  -e POSTGRES_DB=navius_events postgres:13
```

### Starting the Application

```bash
cargo run --example custom_handler
```

or

```bash
cargo run --example real_time_dashboard
```

The application will start a server on `127.0.0.1:8080` with the following endpoints:

- `GET /health` - Health check
- `POST /api/events/trigger/order-created` - Trigger an order created event
- `POST /api/events/trigger/order-shipped` - Trigger an order shipped event
- `POST /api/events/trigger/system-alert` - Trigger a system alert event
- `GET /api/events/history` - View event history
- `GET /api/events/metrics` - View event metrics
- `GET /api/inventory` - View inventory status
- `GET /api/events/stream` - Stream events (SSE)

## Examples

### Custom Handler Example

The `custom_handler.rs` example demonstrates how to create and register a custom event handler that sends SMS notifications when events occur. This example shows how to extend the event system with custom handler implementations.

### Real-Time Dashboard Example

The `real_time_dashboard.rs` example showcases how to implement a real-time monitoring dashboard using the Event System. It features:

- Component health status tracking
- Event metrics aggregation
- Real-time alert monitoring with broadcast channels
- Periodic dashboard displays
- Random event generation for testing

This example demonstrates more advanced event system concepts like:
- Event aggregation and metrics
- Real-time component status monitoring
- Alert broadcasting and subscription
- Time-based metric resets

## Project Structure

- `src/api.rs` - HTTP API implementation
- `src/events.rs` - Event definitions and domain event trait
- `src/handlers.rs` - Event handler implementations
- `src/models.rs` - Data models
- `src/repository.rs` - Database repository for events
- `src/services.rs` - Event bus and service implementations
- `src/lib.rs` - Library interface and plugin implementation
- `src/main.rs` - Standalone application implementation
- `examples/` - Example applications demonstrating specific features 
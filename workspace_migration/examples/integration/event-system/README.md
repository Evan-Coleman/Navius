# Event System Integration Example

This example demonstrates the integration of the event system with other components in the Navius framework. It showcases how to build event-driven applications using the publisher-subscriber pattern.

## Crates Used

- `navius-core`: For dependency injection, component registry, and configuration
- `navius-http`: For HTTP server, routing, and request/response handling
- `navius-event`: For event publishing and subscription
- `navius-db`: For event persistence
- `navius-db-postgres`: For PostgreSQL implementation
- `navius-plugin`: For component discovery and registration

## Key Features Demonstrated

1. **Event-Driven Architecture**:
   - Publishing events from different sources
   - Subscribing to events with multiple handlers
   - Event routing based on topics and content

2. **Publish-Subscribe Pattern**:
   - Event publishers decoupled from subscribers
   - Topic-based subscription
   - Content-based filtering

3. **Event Persistence**:
   - Storing events in database
   - Event replay and recovery
   - Audit log implementation

4. **Integration with HTTP**:
   - Triggering events via HTTP endpoints
   - Webhook event delivery
   - Server-sent events (SSE) for real-time updates

## Running the Example

```bash
# Start PostgreSQL for event persistence
docker-compose up -d

# Run the example application
cd workspace_migration/examples/integration/event-system
cargo run
```

The server will start on `127.0.0.1:8080` with the following endpoints:

- `/health`: Returns the health status of all components
- `/api/events/trigger`: Endpoint to trigger sample events
- `/api/events/history`: View event history
- `/api/events/stream`: Server-sent events endpoint for real-time updates

## Event Types Demonstrated

1. **OrderCreatedEvent**: Triggered when a new order is created
2. **OrderShippedEvent**: Triggered when an order is shipped
3. **InventoryUpdatedEvent**: Triggered when inventory is updated
4. **SystemAlertEvent**: Used for system notifications

## Event Handler Implementation

The example implements several event handlers:

1. **NotificationHandler**: Sends notifications when specific events occur
2. **AnalyticsHandler**: Records event metrics and statistics
3. **AuditLogHandler**: Persists all events to the database for auditing
4. **InventoryHandler**: Updates inventory based on order events

## Key Code Concepts

- **Event Bus**: Central component for publishing and routing events
- **Event Handlers**: Components that subscribe to and process events
- **Event Persistence**: Database storage of events for replay and audit
- **Event Streaming**: Real-time event notifications to clients
- **Component Registration**: Automatic discovery of event handlers

This example demonstrates how to create a loosely coupled, event-driven architecture using the Navius framework, enabling scalable and maintainable applications that can react to system events in real-time. 
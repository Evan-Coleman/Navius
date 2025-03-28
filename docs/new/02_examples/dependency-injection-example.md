---
title: "Dependency Injection Example"
description: "Using dependency injection in Navius applications"
category: examples
tags:
  - examples
  - dependency-injection
  - services
related:
  - examples/custom-service-example.md
  - guides/dependency-injection.md
last_updated: March 26, 2025
version: 1.0
---

# Dependency Injection Example

This example demonstrates how to use Navius' dependency injection system to manage service dependencies and promote loose coupling between components.

## Project Structure

```
dependency-injection-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs
    ├── app/
    │   ├── mod.rs
    │   ├── api/
    │   │   ├── mod.rs
    │   │   └── order_handler.rs
    │   ├── models/
    │   │   ├── mod.rs
    │   │   ├── order.rs
    │   │   └── product.rs
    │   └── services/
    │       ├── mod.rs
    │       ├── order_service.rs
    │       ├── product_service.rs
    │       ├── inventory_service.rs
    │       ├── payment_service.rs
    │       ├── notification_service.rs
    │       └── interfaces/
    │           ├── mod.rs
    │           ├── order_repository.rs
    │           ├── product_repository.rs
    │           ├── payment_processor.rs
    │           └── notifier.rs
    └── core/
        ├── mod.rs
        ├── config.rs
        ├── error.rs
        ├── router.rs
        └── services/
            ├── mod.rs
            └── service_registry.rs
```

## Core Dependency Injection Framework

### `core/services/service_registry.rs`

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::core::error::AppError;

// Main service registry for dependency injection
pub struct ServiceRegistry {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ServiceRegistry {
    // Create a new service registry
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    // Register a service in the registry
    pub fn register<T: 'static + Send + Sync>(&self, service: T) -> Result<(), AppError> {
        let mut services = self.services.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock on service registry")
        })?;
        
        let type_id = TypeId::of::<T>();
        services.insert(type_id, Box::new(service));
        
        Ok(())
    }

    // Register a service that implements a trait
    pub fn register_as<T: 'static + ?Sized + Send + Sync, U: 'static + Send + Sync + AsRef<dyn T>>(
        &self,
        service: U,
    ) -> Result<(), AppError> {
        // Register the concrete type
        self.register(service)?;
        
        Ok(())
    }

    // Get a service by its type
    pub fn get<T: 'static + Clone + Send + Sync>(&self) -> Result<Arc<T>, AppError> {
        let services = self.services.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock on service registry")
        })?;
        
        let type_id = TypeId::of::<T>();
        
        match services.get(&type_id) {
            Some(service) => {
                if let Some(service_ref) = service.downcast_ref::<T>() {
                    Ok(Arc::new(service_ref.clone()))
                } else {
                    Err(AppError::internal_server_error(
                        format!("Service of type {:?} exists but could not be downcast", type_id)
                    ))
                }
            },
            None => Err(AppError::service_not_found(
                format!("No service of type {:?} found in registry", type_id)
            )),
        }
    }

    // Get a service that implements a trait by trait object
    pub fn get_trait<T: 'static + ?Sized + Send + Sync>(&self) -> Result<Arc<Box<dyn T>>, AppError> {
        // This is a more complex implementation that would require type erasure and trait objects
        // For simplicity in this example, we'll use a placeholder implementation
        Err(AppError::not_implemented("Trait-based service resolution is not implemented in this example"))
    }
}
```

## Service Interfaces

### `app/services/interfaces/order_repository.rs`

```rust
use async_trait::async_trait;
use crate::app::models::order::{Order, OrderId};
use crate::core::error::AppError;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn save(&self, order: &Order) -> Result<Order, AppError>;
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, AppError>;
    async fn find_all(&self) -> Result<Vec<Order>, AppError>;
    async fn update(&self, order: &Order) -> Result<Order, AppError>;
    async fn delete(&self, id: OrderId) -> Result<bool, AppError>;
}
```

### `app/services/interfaces/payment_processor.rs`

```rust
use async_trait::async_trait;
use crate::app::models::order::{Order, PaymentDetails};
use crate::core::error::AppError;

#[async_trait]
pub trait PaymentProcessor: Send + Sync {
    async fn process_payment(&self, order: &Order, payment: &PaymentDetails) -> Result<String, AppError>;
    async fn refund_payment(&self, payment_id: &str) -> Result<bool, AppError>;
    async fn verify_payment(&self, payment_id: &str) -> Result<bool, AppError>;
}
```

### `app/services/interfaces/notifier.rs`

```rust
use async_trait::async_trait;
use crate::app::models::order::Order;
use crate::core::error::AppError;

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify_order_created(&self, order: &Order) -> Result<(), AppError>;
    async fn notify_order_updated(&self, order: &Order) -> Result<(), AppError>;
    async fn notify_order_cancelled(&self, order: &Order) -> Result<(), AppError>;
    async fn notify_payment_processed(&self, order: &Order, payment_id: &str) -> Result<(), AppError>;
}
```

## Model Definitions

### `app/models/order.rs`

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

pub type OrderId = String;
pub type ProductId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Created,
    PaymentPending,
    Paid,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: ProductId,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDetails {
    pub payment_method: String,
    pub card_last_four: Option<String>,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
    pub total_amount: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payment_id: Option<String>,
    pub shipping_address: String,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order {{ id: {}, customer: {}, total: ${:.2}, status: {:?} }}",
               self.id, self.customer_id, self.total_amount, self.status)
    }
}
```

## Service Implementations

### `app/services/order_service.rs`

```rust
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::app::models::order::{Order, OrderId, OrderStatus, PaymentDetails};
use crate::app::services::interfaces::order_repository::OrderRepository;
use crate::app::services::interfaces::payment_processor::PaymentProcessor;
use crate::app::services::interfaces::notifier::Notifier;
use crate::core::error::AppError;

// OrderService with injected dependencies
pub struct OrderService {
    order_repository: Arc<dyn OrderRepository>,
    payment_processor: Arc<dyn PaymentProcessor>,
    notifier: Arc<dyn Notifier>,
}

impl OrderService {
    // Constructor with dependency injection
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        payment_processor: Arc<dyn PaymentProcessor>,
        notifier: Arc<dyn Notifier>,
    ) -> Self {
        Self {
            order_repository,
            payment_processor,
            notifier,
        }
    }
    
    // Create a new order
    pub async fn create_order(&self, mut order: Order) -> Result<Order, AppError> {
        // Generate unique ID if not provided
        if order.id.is_empty() {
            order.id = Uuid::new_v4().to_string();
        }
        
        // Set timestamps
        let now = Utc::now();
        order.created_at = now;
        order.updated_at = now;
        
        // Set initial status
        order.status = OrderStatus::Created;
        
        // Save order via repository
        let saved_order = self.order_repository.save(&order).await?;
        
        // Notify about order creation
        self.notifier.notify_order_created(&saved_order).await?;
        
        Ok(saved_order)
    }
    
    // Process payment for order
    pub async fn process_payment(&self, order_id: &OrderId, payment: PaymentDetails) -> Result<Order, AppError> {
        // Retrieve order
        let mut order = self.order_repository.find_by_id(order_id.clone()).await?
            .ok_or_else(|| AppError::not_found(format!("Order not found: {}", order_id)))?;
        
        // Validate order status
        if !matches!(order.status, OrderStatus::Created) {
            return Err(AppError::invalid_state(
                format!("Order {} is not in a valid state for payment", order_id)
            ));
        }
        
        // Process payment
        order.status = OrderStatus::PaymentPending;
        let payment_id = self.payment_processor.process_payment(&order, &payment).await?;
        
        // Update order with payment info
        order.payment_id = Some(payment_id.clone());
        order.status = OrderStatus::Paid;
        order.updated_at = Utc::now();
        
        // Save updated order
        let updated_order = self.order_repository.update(&order).await?;
        
        // Notify about payment processing
        self.notifier.notify_payment_processed(&updated_order, &payment_id).await?;
        
        Ok(updated_order)
    }
    
    // Cancel an order
    pub async fn cancel_order(&self, order_id: &OrderId) -> Result<Order, AppError> {
        // Retrieve order
        let mut order = self.order_repository.find_by_id(order_id.clone()).await?
            .ok_or_else(|| AppError::not_found(format!("Order not found: {}", order_id)))?;
        
        // Validate order status
        if matches!(order.status, OrderStatus::Shipped | OrderStatus::Delivered) {
            return Err(AppError::invalid_state(
                format!("Order {} cannot be cancelled in current state", order_id)
            ));
        }
        
        // If paid, process refund
        if matches!(order.status, OrderStatus::Paid) {
            if let Some(payment_id) = &order.payment_id {
                self.payment_processor.refund_payment(payment_id).await?;
            }
        }
        
        // Update order status
        order.status = OrderStatus::Cancelled;
        order.updated_at = Utc::now();
        
        // Save updated order
        let updated_order = self.order_repository.update(&order).await?;
        
        // Notify about cancellation
        self.notifier.notify_order_cancelled(&updated_order).await?;
        
        Ok(updated_order)
    }
    
    // Get order by ID
    pub async fn get_order(&self, order_id: &OrderId) -> Result<Option<Order>, AppError> {
        self.order_repository.find_by_id(order_id.clone()).await
    }
    
    // Get all orders
    pub async fn get_all_orders(&self) -> Result<Vec<Order>, AppError> {
        self.order_repository.find_all().await
    }
}
```

### Repository Implementations for Testing

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::app::models::order::{Order, OrderId};
use crate::app::services::interfaces::order_repository::OrderRepository;
use crate::core::error::AppError;

// In-memory repository implementation
pub struct InMemoryOrderRepository {
    orders: Arc<Mutex<HashMap<OrderId, Order>>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn save(&self, order: &Order) -> Result<Order, AppError> {
        let mut orders = self.orders.lock().map_err(|_| {
            AppError::internal_server_error("Failed to acquire lock on orders")
        })?;
        
        orders.insert(order.id.clone(), order.clone());
        Ok(order.clone())
    }
    
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, AppError> {
        let orders = self.orders.lock().map_err(|_| {
            AppError::internal_server_error("Failed to acquire lock on orders")
        })?;
        
        Ok(orders.get(&id).cloned())
    }
    
    async fn find_all(&self) -> Result<Vec<Order>, AppError> {
        let orders = self.orders.lock().map_err(|_| {
            AppError::internal_server_error("Failed to acquire lock on orders")
        })?;
        
        Ok(orders.values().cloned().collect())
    }
    
    async fn update(&self, order: &Order) -> Result<Order, AppError> {
        let mut orders = self.orders.lock().map_err(|_| {
            AppError::internal_server_error("Failed to acquire lock on orders")
        })?;
        
        if !orders.contains_key(&order.id) {
            return Err(AppError::not_found(format!("Order not found: {}", order.id)));
        }
        
        orders.insert(order.id.clone(), order.clone());
        Ok(order.clone())
    }
    
    async fn delete(&self, id: OrderId) -> Result<bool, AppError> {
        let mut orders = self.orders.lock().map_err(|_| {
            AppError::internal_server_error("Failed to acquire lock on orders")
        })?;
        
        Ok(orders.remove(&id).is_some())
    }
}
```

### Mock Payment Processor

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::app::models::order::{Order, PaymentDetails};
use crate::app::services::interfaces::payment_processor::PaymentProcessor;
use crate::core::error::AppError;

pub struct MockPaymentProcessor {
    pub always_succeed: bool,
}

impl MockPaymentProcessor {
    pub fn new(always_succeed: bool) -> Self {
        Self { always_succeed }
    }
}

#[async_trait]
impl PaymentProcessor for MockPaymentProcessor {
    async fn process_payment(&self, order: &Order, payment: &PaymentDetails) -> Result<String, AppError> {
        if !self.always_succeed && payment.amount <= 0.0 {
            return Err(AppError::validation_error("Invalid payment amount"));
        }
        
        let payment_id = format!("payment_{}", Uuid::new_v4());
        println!("Processing payment: {} for order: {}", payment_id, order.id);
        Ok(payment_id)
    }
    
    async fn refund_payment(&self, payment_id: &str) -> Result<bool, AppError> {
        println!("Refunding payment: {}", payment_id);
        Ok(true)
    }
    
    async fn verify_payment(&self, payment_id: &str) -> Result<bool, AppError> {
        println!("Verifying payment: {}", payment_id);
        Ok(true)
    }
}
```

### Console Notifier

```rust
use async_trait::async_trait;

use crate::app::models::order::Order;
use crate::app::services::interfaces::notifier::Notifier;
use crate::core::error::AppError;

pub struct ConsoleNotifier;

impl ConsoleNotifier {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Notifier for ConsoleNotifier {
    async fn notify_order_created(&self, order: &Order) -> Result<(), AppError> {
        println!("ORDER CREATED: {}", order);
        Ok(())
    }
    
    async fn notify_order_updated(&self, order: &Order) -> Result<(), AppError> {
        println!("ORDER UPDATED: {}", order);
        Ok(())
    }
    
    async fn notify_order_cancelled(&self, order: &Order) -> Result<(), AppError> {
        println!("ORDER CANCELLED: {}", order);
        Ok(())
    }
    
    async fn notify_payment_processed(&self, order: &Order, payment_id: &str) -> Result<(), AppError> {
        println!("PAYMENT PROCESSED: {} for order {}", payment_id, order.id);
        Ok(())
    }
}
```

## API Handlers

### `app/api/order_handler.rs`

```rust
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::app::models::order::{Order, OrderId, PaymentDetails};
use crate::app::services::order_service::OrderService;
use crate::core::error::AppError;
use crate::core::services::service_registry::ServiceRegistry;

// Get all orders
pub async fn get_orders(
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<Vec<Order>>, AppError> {
    // Get service from registry
    let order_service = registry.get::<OrderService>()?;
    
    // Use service to fetch orders
    let orders = order_service.get_all_orders().await?;
    
    Ok(Json(orders))
}

// Get order by ID
pub async fn get_order(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, AppError> {
    // Get service from registry
    let order_service = registry.get::<OrderService>()?;
    
    // Use service to fetch order
    let order = order_service.get_order(&id).await?
        .ok_or_else(|| AppError::not_found(format!("Order not found: {}", id)))?;
    
    Ok(Json(order))
}

// Create a new order
pub async fn create_order(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(order): Json<Order>,
) -> Result<impl IntoResponse, AppError> {
    // Get service from registry
    let order_service = registry.get::<OrderService>()?;
    
    // Use service to create order
    let created_order = order_service.create_order(order).await?;
    
    Ok((StatusCode::CREATED, Json(created_order)))
}

// Process payment for an order
pub async fn process_payment(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id): Path<OrderId>,
    Json(payment): Json<PaymentDetails>,
) -> Result<Json<Order>, AppError> {
    // Get service from registry
    let order_service = registry.get::<OrderService>()?;
    
    // Use service to process payment
    let updated_order = order_service.process_payment(&id, payment).await?;
    
    Ok(Json(updated_order))
}

// Cancel an order
pub async fn cancel_order(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, AppError> {
    // Get service from registry
    let order_service = registry.get::<OrderService>()?;
    
    // Use service to cancel order
    let cancelled_order = order_service.cancel_order(&id).await?;
    
    Ok(Json(cancelled_order))
}
```

## Main Application

### `main.rs`

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::app::api::order_handler::{
    get_orders, get_order, create_order, process_payment, cancel_order,
};
use crate::app::services::order_service::OrderService;
use crate::app::services::InMemoryOrderRepository;
use crate::app::services::MockPaymentProcessor;
use crate::app::services::ConsoleNotifier;
use crate::core::config::load_config;
use crate::core::error::AppError;
use crate::core::services::service_registry::ServiceRegistry;

mod app;
mod core;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = load_config()
        .map_err(|e| AppError::configuration_error(format!("Failed to load configuration: {}", e)))?;
    
    // Create service registry
    let registry = Arc::new(ServiceRegistry::new());
    
    // Create repositories
    let order_repository = Arc::new(InMemoryOrderRepository::new());
    
    // Create service implementations
    let payment_processor = Arc::new(MockPaymentProcessor::new(true));
    let notifier = Arc::new(ConsoleNotifier::new());
    
    // Create and register OrderService with its dependencies
    let order_service = OrderService::new(
        order_repository.clone(),
        payment_processor.clone(),
        notifier.clone(),
    );
    
    registry.register(order_service)?;
    
    // Create router with dependency injection
    let app = Router::new()
        .route("/orders", get(get_orders))
        .route("/orders", post(create_order))
        .route("/orders/:id", get(get_order))
        .route("/orders/:id/payment", post(process_payment))
        .route("/orders/:id/cancel", put(cancel_order))
        .with_state(registry.clone());
    
    // Extract server address
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()
        .map_err(|_| AppError::configuration_error("Invalid server address"))?;
    
    // Start server
    println!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| AppError::internal_server_error(format!("Server error: {}", e)))?;
    
    Ok(())
}
```

## Testing with Dependency Injection

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::models::order::{Order, OrderStatus, OrderItem};
    use chrono::Utc;
    use std::sync::Arc;
    
    // Mock notifier for testing
    struct TestNotifier {
        called: Arc<Mutex<bool>>,
    }
    
    impl TestNotifier {
        fn new() -> Self {
            Self {
                called: Arc::new(Mutex::new(false)),
            }
        }
        
        fn was_called(&self) -> bool {
            *self.called.lock().unwrap()
        }
    }
    
    #[async_trait]
    impl Notifier for TestNotifier {
        async fn notify_order_created(&self, _order: &Order) -> Result<(), AppError> {
            let mut called = self.called.lock().unwrap();
            *called = true;
            Ok(())
        }
        
        // Implement other methods...
        async fn notify_order_updated(&self, _: &Order) -> Result<(), AppError> { Ok(()) }
        async fn notify_order_cancelled(&self, _: &Order) -> Result<(), AppError> { Ok(()) }
        async fn notify_payment_processed(&self, _: &Order, _: &str) -> Result<(), AppError> { Ok(()) }
    }
    
    #[tokio::test]
    async fn test_create_order() {
        // Create test dependencies
        let order_repository = Arc::new(InMemoryOrderRepository::new());
        let payment_processor = Arc::new(MockPaymentProcessor::new(true));
        let test_notifier = Arc::new(TestNotifier::new());
        
        // Create service with injected mock dependencies
        let order_service = OrderService::new(
            order_repository.clone(),
            payment_processor.clone(),
            test_notifier.clone(),
        );
        
        // Create test order
        let order = Order {
            id: "".to_string(), // Empty ID will be generated
            customer_id: "test-customer".to_string(),
            items: vec![
                OrderItem {
                    product_id: "product-1".to_string(),
                    quantity: 2,
                    unit_price: 10.0,
                }
            ],
            status: OrderStatus::Created,
            total_amount: 20.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            payment_id: None,
            shipping_address: "123 Test St".to_string(),
        };
        
        // Test creating an order
        let result = order_service.create_order(order).await;
        
        // Verify result
        assert!(result.is_ok(), "Order creation failed: {:?}", result.err());
        let created_order = result.unwrap();
        assert!(!created_order.id.is_empty(), "Order ID should be generated");
        assert_eq!(created_order.customer_id, "test-customer");
        assert_eq!(created_order.status, OrderStatus::Created);
        
        // Verify notification was sent
        assert!(test_notifier.was_called(), "Notifier was not called");
        
        // Verify order was stored in repository
        let stored_order = order_repository.find_by_id(created_order.id.clone()).await.unwrap();
        assert!(stored_order.is_some(), "Order should be stored in repository");
    }
}
```

## Running the Example

1. Clone the Navius repository
2. Navigate to the `examples/dependency-injection-example` directory
3. Run the example:

```bash
cargo run
```

4. Test the API endpoints:

```bash
# Create an order
curl -X POST http://localhost:3000/orders \
  -H "Content-Type: application/json" \
  -d '{
    "id": "",
    "customer_id": "user123",
    "items": [
      {
        "product_id": "product456",
        "quantity": 2,
        "unit_price": 19.99
      }
    ],
    "status": "Created",
    "total_amount": 39.98,
    "created_at": "2023-09-25T00:00:00Z",
    "updated_at": "2023-09-25T00:00:00Z",
    "payment_id": null,
    "shipping_address": "123 Main St, Anytown, AN 12345"
  }' | jq

# Get the order ID from the response, then process payment
curl -X POST http://localhost:3000/orders/[ORDER_ID]/payment \
  -H "Content-Type: application/json" \
  -d '{
    "payment_method": "credit_card",
    "card_last_four": "4242",
    "amount": 39.98,
    "currency": "USD"
  }' | jq

# Get all orders
curl http://localhost:3000/orders | jq

# Cancel an order
curl -X PUT http://localhost:3000/orders/[ORDER_ID]/cancel | jq
```

## Key Concepts Demonstrated

1. **Dependency Inversion Principle**: Services depend on abstractions (interfaces) rather than concrete implementations
2. **Constructor Injection**: Dependencies are provided through constructors
3. **Registry Pattern**: ServiceRegistry manages service instances and their lifecycle
4. **Interface Segregation**: Clear interfaces defining cohesive sets of operations
5. **Testability**: Easy to replace real implementations with test doubles

## Best Practices

1. **Prefer Interfaces**: Define clear interfaces before implementations
2. **Single Responsibility**: Each service should have one primary responsibility
3. **Explicit Dependencies**: Make dependencies clear in constructors
4. **Testable Design**: Design services to be easily testable with mock dependencies
5. **Circular Dependency Avoidance**: Structure your dependencies to avoid cycles

## Next Steps

- [Custom Service Example](custom-service-example.md): More examples of creating custom services
- [Error Handling Example](error-handling-example.md): Comprehensive error handling strategies
- [Repository Pattern Example](repository-pattern-example.md): Using the repository pattern with dependency injection
</rewritten_file> 
---
title: "Dependency Injection in Navius"
description: "Comprehensive guide to using dependency injection in Navius applications for managing service dependencies and promoting loose coupling"
category: examples
tags:
  - dependency-injection
  - services
  - service-registry
  - di
  - inversion-of-control
  - loose-coupling
  - interfaces
related:
  - 02_examples/custom-service-example.md
  - 02_examples/rest-api-example.md
  - 01_getting_started/development-setup.md
last_updated: March 27, 2025
version: 1.1
status: stable
---

# Dependency Injection Example

This example demonstrates how to use Navius' dependency injection system to manage service dependencies and promote loose coupling between components.

## Overview

Dependency Injection (DI) is a software design pattern that promotes loose coupling between components by separating behavior from dependency resolution. In Navius, the DI system allows you to:

- Register services in a central registry
- Resolve service dependencies at runtime
- Mock dependencies for testing
- Create modular, maintainable code with clear separation of concerns

This example builds a complete order management system that showcases dependency injection principles in action through a series of services, repositories, and handlers.

## Quick Navigation

- [Project Structure](#project-structure)
- [Core Dependency Injection Framework](#core-dependency-injection-framework)
- [Service Interfaces](#service-interfaces)
- [Model Definitions](#model-definitions)
- [Service Implementations](#service-implementations)
- [API Handlers](#api-handlers)
- [Service Registration](#service-registration)
- [Working with DI in Tests](#working-with-di-in-tests)
- [Best Practices](#best-practices)
- [Common Pitfalls](#common-pitfalls)
- [Advanced Techniques](#advanced-techniques)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics, including traits and trait objects
- Asynchronous programming with Tokio
- Basic software design patterns
- Navius framework fundamentals

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- async-trait 0.1.0 or newer
- tokio for asynchronous operations

## Project Structure

```rust
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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

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
```rust

## Working with DI in Tests

Testing services that use dependency injection is straightforward thanks to Navius' flexible service registry. The DI system allows you to easily replace real implementations with mocks for isolated unit testing.

### Creating Mock Services

First, implement mock versions of your service interfaces:

```rust
use mockall::predicate::*;
use mockall::mock;

// Generate a mock implementation of the OrderRepository trait
mock! {
    OrderRepository {}
    
    #[async_trait]
    impl OrderRepository for OrderRepository {
        async fn save(&self, order: &Order) -> Result<Order, AppError>;
        async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, AppError>;
        async fn find_all(&self) -> Result<Vec<Order>, AppError>;
        async fn update(&self, order: &Order) -> Result<Order, AppError>;
        async fn delete(&self, id: OrderId) -> Result<bool, AppError>;
    }
}

// Generate a mock implementation of the PaymentProcessor trait
mock! {
    PaymentProcessor {}
    
    #[async_trait]
    impl PaymentProcessor for PaymentProcessor {
        async fn process_payment(&self, order: &Order, payment: &PaymentDetails) -> Result<String, AppError>;
        async fn refund_payment(&self, payment_id: &str) -> Result<bool, AppError>;
        async fn verify_payment(&self, payment_id: &str) -> Result<bool, AppError>;
    }
}
```rust

### Unit Testing with Mocks

Now you can use these mocks in your unit tests:

```rust
    #[tokio::test]
async fn test_process_order() {
    // Create mock instances
    let mut mock_repository = MockOrderRepository::new();
    let mut mock_processor = MockPaymentProcessor::new();
    let mut mock_notifier = MockNotifier::new();
    
    // Set up mock expectations
    
    // Expect save to be called once and return a cloned order
    mock_repository
        .expect_save()
        .times(1)
        .returning(|order| Ok(order.clone()));
    
    // Expect process_payment to be called once and return a payment ID
    mock_processor
        .expect_process_payment()
        .times(1)
        .returning(|_, _| Ok("payment-123".to_string()));
        
    // Expect notify_order_created to be called once
    mock_notifier
        .expect_notify_order_created()
        .times(1)
        .returning(|_| Ok(()));
        
    // Create the service with mocked dependencies
    let order_service = OrderServiceImpl::new(
        Arc::new(mock_repository),
        Arc::new(mock_processor),
        Arc::new(mock_notifier)
    );
    
    // Create test data
    let order_request = CreateOrderRequest {
        customer_id: "cust-123".to_string(),
            items: vec![
            OrderItemRequest {
                product_id: "prod-1".to_string(),
                    quantity: 2,
                }
            ],
            shipping_address: "123 Test St".to_string(),
        };
        
    let payment_details = PaymentDetails {
        payment_method: "credit_card".to_string(),
        card_last_four: Some("4242".to_string()),
        amount: 100.0,
        currency: "USD".to_string(),
    };
    
    // Execute the method under test
    let result = order_service.process_order(order_request, payment_details).await;
    
    // Assert results
    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.status, OrderStatus::Paid);
    assert_eq!(order.payment_id, Some("payment-123".to_string()));
}
```rust

### Integration Testing with a Test ServiceRegistry

For integration tests, you can create a test ServiceRegistry with mock or real implementations:

```rust
#[tokio::test]
async fn test_order_handler_integration() {
    // Create a test ServiceRegistry
    let registry = ServiceRegistry::new();
    
    // Register mock services
    let mock_repository = Arc::new(MockOrderRepository::new());
    registry.register::<Box<dyn OrderRepository>>(Box::new(mock_repository.clone()));
    
    let mock_processor = Arc::new(MockPaymentProcessor::new());
    registry.register::<Box<dyn PaymentProcessor>>(Box::new(mock_processor.clone()));
    
    let mock_notifier = Arc::new(MockNotifier::new());
    registry.register::<Box<dyn Notifier>>(Box::new(mock_notifier.clone()));
    
    // Set up expectations on mocks
    // ...
    
    // Create an OrderService with the registry
    let order_service = OrderServiceImpl::new_with_registry(&registry).await.unwrap();
    registry.register::<OrderServiceImpl>(order_service.clone());
    
    // Create a handler with the registry
    let handler = OrderHandler::new(&registry);
    
    // Call the handler's method
    let result = handler.create_order(/* test request */).await;
    
    // Assert the results
    assert!(result.is_ok());
    // Further assertions...
}
```rust

## Best Practices

### 1. Define Clear Interfaces

Create well-defined interfaces (traits) that describe the behavior of your services:

```rust
#[async_trait]
pub trait ProductService: Send + Sync {
    // Clear contract with detailed documentation
    
    /// Retrieves a product by its unique identifier
    async fn get_product(&self, id: ProductId) -> Result<Option<Product>, AppError>;
    
    /// Lists all available products
    async fn list_products(&self) -> Result<Vec<Product>, AppError>;
    
    /// Creates a new product
    async fn create_product(&self, product: CreateProductRequest) -> Result<Product, AppError>;
}
```rust

### 2. Program to Interfaces, Not Implementations

Whenever possible, accept trait objects rather than concrete implementations:

```rust
// Good approach
pub struct OrderServiceImpl {
    order_repository: Arc<dyn OrderRepository>,
    payment_processor: Arc<dyn PaymentProcessor>,
    notifier: Arc<dyn Notifier>,
}

// Avoid this approach
pub struct OrderServiceImpl {
    // Tightly coupled to concrete implementations
    order_repository: Arc<OrderRepositoryImpl>,
    payment_processor: Arc<StripePaymentProcessor>,
    notifier: Arc<EmailNotifier>,
}
```rust

### 3. Use Constructor Injection

Pass dependencies through the constructor rather than creating them inside the service:

```rust
// Good approach
impl OrderServiceImpl {
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
}

// Avoid this approach
impl OrderServiceImpl {
    pub fn new() -> Self {
        Self {
            // Tightly coupled to concrete implementations
            order_repository: Arc::new(OrderRepositoryImpl::new()),
            payment_processor: Arc::new(StripePaymentProcessor::new()),
            notifier: Arc::new(EmailNotifier::new()),
        }
    }
}
```rust

### 4. Consider Factory Methods

When constructing services with many dependencies, use factory methods with the ServiceRegistry:

```rust
impl OrderServiceImpl {
    pub async fn new_with_registry(registry: &ServiceRegistry) -> Result<Self, AppError> {
        let order_repository = registry.get::<Box<dyn OrderRepository>>().await?;
        let payment_processor = registry.get::<Box<dyn PaymentProcessor>>().await?;
        let notifier = registry.get::<Box<dyn Notifier>>().await?;
        
        Ok(Self {
            order_repository,
            payment_processor,
            notifier,
        })
    }
}
```rust

### 5. Register Services as Trait Objects

Register your services as trait objects to enable polymorphism:

```rust
// Setup function
pub fn setup_services(registry: &ServiceRegistry) -> Result<(), AppError> {
    // Create the concrete implementations
    let order_repo = Arc::new(PostgresOrderRepository::new());
    let payment_processor = Arc::new(StripePaymentProcessor::new());
    let notifier = Arc::new(EmailNotifier::new());
    
    // Register them as trait objects
    registry.register::<Box<dyn OrderRepository>>(Box::new(order_repo));
    registry.register::<Box<dyn PaymentProcessor>>(Box::new(payment_processor));
    registry.register::<Box<dyn Notifier>>(Box::new(notifier));
    
    // Register the service that depends on them
    let order_service = Arc::new(OrderServiceImpl::new(
        registry.get::<Box<dyn OrderRepository>>()?,
        registry.get::<Box<dyn PaymentProcessor>>()?,
        registry.get::<Box<dyn Notifier>>()?,
    ));
    
    registry.register::<OrderServiceImpl>(order_service);
    
    Ok(())
}
```rust

## Common Pitfalls

### 1. Circular Dependencies

Avoid circular dependencies between services, as they can cause initialization problems and infinite loops.

**Problem:**
```rust
// Service A depends on Service B
pub struct ServiceA {
    service_b: Arc<ServiceB>,
}

// Service B depends on Service A
pub struct ServiceB {
    service_a: Arc<ServiceA>,
}
```rust

**Solution:**
- Restructure your services to remove the circular dependency
- Extract a common interface that both services can depend on
- Use events or callbacks to communicate between services

### 2. Service Locator Anti-Pattern

Avoid using the ServiceRegistry directly in your business logic:

**Anti-pattern:**
```rust
// Service locator anti-pattern
impl OrderService {
    async fn process_order(&self, registry: &ServiceRegistry, order: Order) -> Result<Order, AppError> {
        // Getting dependencies at runtime
        let repository = registry.get::<Box<dyn OrderRepository>>().await?;
        let payment_processor = registry.get::<Box<dyn PaymentProcessor>>().await?;
        
        // Business logic...
    }
}
```rust

**Better approach:**
```rust
// Constructor injection
impl OrderService {
    pub fn new(
        repository: Arc<dyn OrderRepository>,
        payment_processor: Arc<dyn PaymentProcessor>,
    ) -> Self {
        Self {
            repository,
            payment_processor,
        }
    }
    
    async fn process_order(&self, order: Order) -> Result<Order, AppError> {
        // Using injected dependencies
        // Business logic...
    }
}
```rust

### 3. Over-Abstraction

Don't create interfaces for everything. Consider the following when deciding whether to create an interface:

- Will there be multiple implementations?
- Will you need to mock this service for testing?
- Is there a need to replace the implementation at runtime?

If the answer to all these questions is "no," a direct implementation may be simpler.

### 4. Thread Safety Issues

Since services are shared across async tasks, ensure your services are thread-safe:

```rust
// Ensure all services implement Send + Sync
#[async_trait]
pub trait OrderRepository: Send + Sync {
    // Methods...
}

// Use thread-safe data structures
pub struct InMemoryOrderRepository {
    // Use RwLock or Mutex for shared mutable state
    orders: RwLock<HashMap<OrderId, Order>>,
}
```rust

## Advanced Techniques

### 1. Named Dependencies

When you need multiple implementations of the same interface, use named dependencies:

```rust
// Register named implementations
registry.register_named::<Box<dyn PaymentProcessor>>("stripe", Box::new(StripeProcessor::new()));
registry.register_named::<Box<dyn PaymentProcessor>>("paypal", Box::new(PayPalProcessor::new()));

// Retrieve a specific implementation
let stripe_processor = registry.get_named::<Box<dyn PaymentProcessor>>("stripe")?;
```rust

### 2. Conditional Registration

Register different implementations based on configuration:

```rust
pub fn setup_services(registry: &ServiceRegistry, config: &AppConfig) -> Result<(), AppError> {
    // Choose implementation based on configuration
    if config.use_real_payment_processor {
        let processor = Arc::new(StripePaymentProcessor::new(config.stripe_api_key.clone()));
        registry.register::<Box<dyn PaymentProcessor>>(Box::new(processor));
    } else {
        let processor = Arc::new(MockPaymentProcessor::new());
        registry.register::<Box<dyn PaymentProcessor>>(Box::new(processor));
    }
    
    Ok(())
}
```rust

### 3. Service Lifetimes

Control the lifetime of your services by using different registration strategies:

```rust
// Singleton (default) - one instance shared across the application
registry.register_singleton::<UserService>(Arc::new(UserServiceImpl::new()));

// Transient - new instance created each time it's resolved
registry.register_transient::<UserService, _>(|| {
    Arc::new(UserServiceImpl::new())
});

// Scoped - one instance per scope (e.g., per request)
registry.register_scoped::<UserService, _>(|| {
    Arc::new(UserServiceImpl::new())
});
```rust

### 4. Decorators and Middleware

Use the decorator pattern to add cross-cutting concerns:

```rust
// Base implementation
pub struct BasicOrderProcessor {
    repository: Arc<dyn OrderRepository>,
}

// Decorator that adds logging
pub struct LoggingOrderProcessor {
    inner: Arc<dyn OrderProcessor>,
    logger: Arc<dyn Logger>,
}

impl OrderProcessor for LoggingOrderProcessor {
    async fn process(&self, order: Order) -> Result<Order, AppError> {
        self.logger.log(&format!("Processing order: {}", order.id));
        let result = self.inner.process(order).await;
        
        match &result {
            Ok(order) => self.logger.log(&format!("Order processed successfully: {}", order.id)),
            Err(e) => self.logger.log(&format!("Order processing failed: {}", e)),
        }
        
        result
    }
}

// Register with decorator
let base_processor = Arc::new(BasicOrderProcessor::new(repository));
let logging_processor = Arc::new(LoggingOrderProcessor::new(base_processor, logger));
registry.register::<Box<dyn OrderProcessor>>(Box::new(logging_processor));
```rust

## Conclusion

This example has demonstrated how to use dependency injection in Navius to build modular and testable applications. By leveraging interfaces, constructor injection, and a centralized service registry, you can create loosely coupled components that are easy to test and maintain.

Key takeaways:
- Define clear interfaces using traits
- Inject dependencies through constructors
- Register services in a central registry
- Test using mock implementations
- Follow best practices to avoid common pitfalls

By following these patterns, your Navius applications will be more modular, testable, and maintainable.

## See Also

- [Custom Service Example](./custom-service-example.md) - For more advanced service implementation techniques
- [REST API Example](./rest-api-example.md) - For using DI in API handlers
- [Testing with Navius](../04_guides/testing.md) - For more testing techniques
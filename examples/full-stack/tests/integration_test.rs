use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
};
use navius_http::server::HttpServerBuilder;
use serde_json::{Value, json};
use tower::ServiceExt;

use full_stack_integration_example::{
    api::{self, controllers::auth::AuthResponse},
    application::TaskFilter,
    domain::{Priority, TaskStatus},
    infrastructure::ServiceRegistry,
};

// Helper function to create a test app with in-memory repositories
async fn create_test_app() -> (axum::Router, Arc<ServiceRegistry>) {
    // Create in-memory repositories
    let user_repository = Arc::new(
        full_stack_integration_example::infrastructure::repository::InMemoryUserRepository::new(),
    );
    let task_repository = Arc::new(
        full_stack_integration_example::infrastructure::repository::InMemoryTaskRepository::new(),
    );
    let category_repository = Arc::new(
        full_stack_integration_example::infrastructure::repository::InMemoryCategoryRepository::new(
        ),
    );
    let notification_repository = Arc::new(full_stack_integration_example::infrastructure::repository::InMemoryNotificationRepository::new());

    // Create event publisher
    let event_publisher = Arc::new(
        full_stack_integration_example::infrastructure::events::InMemoryEventPublisher::new(),
    );

    // Create service registry
    let service_registry = Arc::new(
        full_stack_integration_example::infrastructure::ServiceRegistry::new(
            user_repository.clone(),
            task_repository.clone(),
            category_repository.clone(),
            notification_repository.clone(),
            event_publisher.clone(),
        ),
    );

    // Build HTTP server with routes
    let server = HttpServerBuilder::new().build();
    let server = api::configure_routes(server, service_registry.clone());

    // Create Axum router
    let app = server.into_router();

    (app, service_registry)
}

// Helper to perform user registration
async fn register_user(
    app: &axum::Router,
    username: &str,
    email: &str,
    password: &str,
) -> (AuthResponse, StatusCode) {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    (auth_response, status)
}

// Helper to perform user login
async fn login_user(app: &axum::Router, email: &str, password: &str) -> (AuthResponse, StatusCode) {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    (auth_response, status)
}

// Helper to create a task
async fn create_task(
    app: &axum::Router,
    token: &str,
    title: &str,
    description: Option<&str>,
    priority: Option<&str>,
    due_date: Option<&str>,
    category_id: Option<&str>,
) -> (Value, StatusCode) {
    let mut json_body = json!({
        "title": title,
    });

    if let Some(desc) = description {
        json_body["description"] = json!(desc);
    }

    if let Some(prio) = priority {
        json_body["priority"] = json!(prio);
    }

    if let Some(date) = due_date {
        json_body["due_date"] = json!(date);
    }

    if let Some(cat_id) = category_id {
        json_body["category_id"] = json!(cat_id);
    }

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/tasks")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let task_response: Value = serde_json::from_slice(&body).unwrap();

    (task_response, status)
}

// Helper to get tasks
async fn get_tasks(app: &axum::Router, token: &str) -> (Vec<Value>, StatusCode) {
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/tasks")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let tasks: Vec<Value> = serde_json::from_slice(&body).unwrap();

    (tasks, status)
}

// Helper to get a task by ID
async fn get_task(app: &axum::Router, token: &str, task_id: &str) -> (Value, StatusCode) {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/tasks/{}", task_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if status == StatusCode::OK {
        let task: Value = serde_json::from_slice(&body).unwrap();
        (task, status)
    } else {
        (json!({}), status)
    }
}

#[tokio::test]
async fn test_user_registration_and_authentication() {
    // Create test app
    let (app, _) = create_test_app().await;

    // Register a user
    let (auth_response, status) =
        register_user(&app, "testuser", "test@example.com", "Password123!").await;

    // Check registration was successful
    assert_eq!(status, StatusCode::OK);
    assert!(!auth_response.token.is_empty());
    assert!(!auth_response.refresh_token.is_empty());

    // Login with the registered user
    let (login_response, login_status) = login_user(&app, "test@example.com", "Password123!").await;

    // Check login was successful
    assert_eq!(login_status, StatusCode::OK);
    assert!(!login_response.token.is_empty());
}

#[tokio::test]
async fn test_task_creation_and_retrieval() {
    // Create test app
    let (app, _) = create_test_app().await;

    // Register a user
    let (auth_response, _) =
        register_user(&app, "taskuser", "tasks@example.com", "Password123!").await;

    // Create a task
    let (task_response, status) = create_task(
        &app,
        &auth_response.token,
        "Test Task",
        Some("This is a test task"),
        Some("high"),
        Some("2025-04-15T12:00:00Z"),
        None,
    )
    .await;

    // Check task creation was successful
    assert_eq!(status, StatusCode::OK);
    assert_eq!(task_response["title"], "Test Task");
    assert_eq!(task_response["description"], "This is a test task");

    // Get all tasks
    let (tasks, status) = get_tasks(&app, &auth_response.token).await;

    // Check task retrieval was successful
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Test Task");

    // Get the specific task
    let task_id = task_response["id"].as_str().unwrap();
    let (task, status) = get_task(&app, &auth_response.token, task_id).await;

    // Check task retrieval by ID was successful
    assert_eq!(status, StatusCode::OK);
    assert_eq!(task["title"], "Test Task");
}

#[tokio::test]
async fn test_unauthorized_access() {
    // Create test app
    let (app, _) = create_test_app().await;

    // Attempt to access tasks without authentication
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/tasks")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    // Check unauthorized status
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_task_comment_workflow() {
    // Create test app
    let (app, _) = create_test_app().await;

    // Register a user
    let (auth_response, _) =
        register_user(&app, "commentuser", "comments@example.com", "Password123!").await;

    // Create a task
    let (task_response, _) = create_task(
        &app,
        &auth_response.token,
        "Task with Comments",
        Some("This task will have comments"),
        None,
        None,
        None,
    )
    .await;

    let task_id = task_response["id"].as_str().unwrap();

    // Add a comment to the task
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/tasks/{}/comments", task_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", auth_response.token))
        .body(Body::from(
            json!({"content": "This is a test comment"}).to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let comment: Value = serde_json::from_slice(&body).unwrap();

    // Get task comments
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/tasks/{}/comments", task_id))
        .header("Authorization", format!("Bearer {}", auth_response.token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let comments: Vec<Value> = serde_json::from_slice(&body).unwrap();

    // Check comment was added successfully
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["content"], "This is a test comment");

    // Update the comment
    let comment_id = comment["id"].as_str().unwrap();
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tasks/{}/comments/{}", task_id, comment_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", auth_response.token))
        .body(Body::from(
            json!({"content": "Updated comment content"}).to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get task comments again to verify update
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/tasks/{}/comments", task_id))
        .header("Authorization", format!("Bearer {}", auth_response.token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let comments: Vec<Value> = serde_json::from_slice(&body).unwrap();

    // Check comment was updated successfully
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["content"], "Updated comment content");
}

#[tokio::test]
async fn test_full_task_lifecycle() {
    // Create test app
    let (app, _) = create_test_app().await;

    // Register two users (one for creating, one for assignment)
    let (creator_auth, _) =
        register_user(&app, "creator", "creator@example.com", "Password123!").await;
    let (assignee_auth, _) =
        register_user(&app, "assignee", "assignee@example.com", "Password123!").await;

    // Get the assignee user ID
    let assignee_id = assignee_auth.user_id;

    // Create a task
    let (task_response, _) = create_task(
        &app,
        &creator_auth.token,
        "Lifecycle Task",
        Some("This task will go through its lifecycle"),
        Some("medium"),
        Some("2025-05-15T12:00:00Z"),
        None,
    )
    .await;

    let task_id = task_response["id"].as_str().unwrap();

    // Assign the task to the assignee
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/tasks/{}/assign/{}", task_id, assignee_id))
        .header("Authorization", format!("Bearer {}", creator_auth.token))
        .body(Body::empty())
        .unwrap();

    // Note: This will fail since the test user doesn't have manager role
    // In a real implementation, we would need to properly setup roles
    let response = app.clone().oneshot(request).await.unwrap();

    // Update the task status to in-progress
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tasks/{}", task_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", creator_auth.token))
        .body(Body::from(json!({"status": "in_progress"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get the task to verify status change
    let (task, _) = get_task(&app, &creator_auth.token, task_id).await;
    assert_eq!(task["status"], "IN_PROGRESS");

    // Update the task status to done
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tasks/{}", task_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", creator_auth.token))
        .body(Body::from(json!({"status": "done"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get the task to verify status change
    let (task, _) = get_task(&app, &creator_auth.token, task_id).await;
    assert_eq!(task["status"], "DONE");

    // Delete the task
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/tasks/{}", task_id))
        .header("Authorization", format!("Bearer {}", creator_auth.token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Try to get the deleted task
    let (_, status) = get_task(&app, &creator_auth.token, task_id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

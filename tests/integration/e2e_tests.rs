use axum::{
    Router,
    body::{self, Body},
    http::{Request, StatusCode},
};
use metrics_exporter_prometheus::PrometheusBuilder;
use navius::{
    app::router,
    core::{
        config::app_config::{AppConfig, AuthConfig, CacheConfig},
        router::AppState,
        utils::api_resource::ApiResourceRegistry,
    },
    repository::User,
};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json;
use std::sync::Arc;
use std::time::SystemTime;
use tower::ServiceExt;

// Constants
const MAX_BODY_SIZE: usize = 1024 * 1024; // 1MB limit for response bodies

mod common;

/// Helper function to create a test server with all dependencies
async fn setup_test_server() -> Router {
    // Create test configuration
    let config = AppConfig {
        auth: AuthConfig {
            enabled: false,
            ..Default::default()
        },
        cache: CacheConfig {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    // Create test state
    let metrics_recorder = PrometheusBuilder::new().build_recorder();
    let metrics_handle = metrics_recorder.handle();

    let state = Arc::new(AppState {
        client: Client::new(),
        config: config.clone(),
        start_time: SystemTime::now(),
        cache_registry: None,
        metrics_handle,
        token_client: None,
        resource_registry: ApiResourceRegistry::new(),
    });

    // Create router with test state
    router::create_router(state)
}

async fn get_response(app: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = body::to_bytes(response.into_body(), MAX_BODY_SIZE)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    (status, body_str)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_test_server().await;

    // Test health check
    let get_request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (status, body) = get_response(&app, get_request).await;
    assert_eq!(status, StatusCode::OK);
    // Health endpoint returns a detailed JSON response
    assert!(body.contains("healthy"));
    assert!(body.contains("status"));
    assert!(body.contains("version"));
}

#[tokio::test]
async fn test_actuator_endpoint() {
    let app = setup_test_server().await;

    // Test actuator endpoint (which would normally be protected)
    // Note: We've disabled auth in our test config, so this should return OK
    let get_request = Request::builder()
        .uri("/actuator/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (status, body) = get_response(&app, get_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("status"));
}

#[tokio::test]
async fn test_user_endpoints() {
    let app = setup_test_server().await;

    // Create a new user
    let user_data = r#"{
        "username": "test_user",
        "email": "test@example.com",
        "full_name": "Test User",
        "role": "user"
    }"#;

    let create_request = Request::builder()
        .uri("/users")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(user_data))
        .unwrap();

    let (create_status, body) = get_response(&app, create_request).await;
    println!("Create response: {}", body);

    // Check if the user creation endpoint is working
    assert!(
        create_status.is_success(),
        "Failed to create user: {}",
        body
    );
    assert!(
        body.contains("test@example.com"),
        "Response should contain user email"
    );
    assert!(
        body.contains("test_user"),
        "Response should contain username"
    );

    // The response should contain a UUID
    let response_data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        response_data["id"].is_string(),
        "Response should contain a UUID id"
    );

    // Test the users endpoint (GET /users)
    let get_all_request = Request::builder()
        .uri("/users")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (get_all_status, get_all_body) = get_response(&app, get_all_request).await;
    println!("Get all users response: {}", get_all_body);
    assert!(get_all_status.is_success(), "Failed to get all users");
}

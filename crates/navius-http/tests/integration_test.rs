use axum::{
    Json,
    extract::{Path, State},
    routing::get,
};
use navius_core::config::Configuration;
use navius_http::{
    client::HttpClient,
    error::{Error, Result},
    server::{HttpServer, RouterBuilder},
};
use navius_test::error::{TestResult, assert_contains, assert_eq, assert_true};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

// Sample data model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: u64,
    name: String,
    description: Option<String>,
}

// Simple in-memory "database"
#[derive(Debug, Clone)]
struct ItemRepository {
    items: Arc<Mutex<HashMap<u64, Item>>>,
}

impl ItemRepository {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_item(&self, item: Item) -> Result<()> {
        let mut items = self
            .items
            .lock()
            .map_err(|e| Error::internal(format!("Failed to lock item repository: {}", e)))?;

        items.insert(item.id, item);
        Ok(())
    }

    fn get_item(&self, id: u64) -> Result<Option<Item>> {
        let items = self
            .items
            .lock()
            .map_err(|e| Error::internal(format!("Failed to lock item repository: {}", e)))?;

        Ok(items.get(&id).cloned())
    }
}

// Application state
#[derive(Debug, Clone)]
struct AppState {
    item_repo: ItemRepository,
}

// Handler functions
async fn get_item(State(state): State<AppState>, Path(id): Path<u64>) -> axum::response::Response {
    match state.item_repo.get_item(id) {
        Ok(Some(item)) => {
            let json = Json(item);
            axum::response::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json.0).unwrap(),
                ))
                .unwrap()
        }
        Ok(None) => {
            let error = Error::not_found(format!("Item with id {} not found", id));
            let status = error.status_code();
            let json = Json(serde_json::json!({
                "error": {
                    "code": error.code(),
                    "message": error.to_string(),
                }
            }));

            axum::response::Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json.0).unwrap(),
                ))
                .unwrap()
        }
        Err(err) => {
            let status = err.status_code();
            let json = Json(serde_json::json!({
                "error": {
                    "code": err.code(),
                    "message": err.to_string(),
                }
            }));

            axum::response::Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json.0).unwrap(),
                ))
                .unwrap()
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::test]
async fn test_http_integration() -> TestResult<()> {
    // Set up the item repository
    let item_repo = ItemRepository::new();

    // Add some test items
    item_repo.add_item(Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("This is a test item".to_string()),
    })?;

    // Create the application state
    let state = AppState { item_repo };

    // Create the router
    let router = RouterBuilder::new()
        .route("/items/:id", get(get_item))
        .route("/health", get(health_check))
        .build()
        .with_state(state);

    // Create and start the server (using port 0 for auto-assignment)
    let server = HttpServer::new()
        .with_router(router)
        .with_host_and_port("127.0.0.1", 0);

    let server_handle = server.serve().await?;

    // Get the port the server is running on
    // This is a bit hacky, but since we're using port 0, we need to figure out the assigned port
    // In a real application, we would use a known port
    let server_addr = server_handle.address().unwrap();
    let port = server_addr.port();

    // Create the client
    let client = HttpClient::new().with_base_url(&format!("http://127.0.0.1:{}", port));

    // Test the health check endpoint
    let health_response = client.get("/health").send().await?;
    assert_eq(
        health_response.status().as_u16(),
        200,
        "Health check should return 200 OK",
    )?;

    let health_body = health_response.text().await.unwrap();
    assert_eq(health_body, "OK", "Health check should return 'OK'")?;

    // Test the get item endpoint for an existing item
    let item_response = client.get("/items/1").send().await?;
    assert_eq(
        item_response.status().as_u16(),
        200,
        "Get item should return 200 OK for existing item",
    )?;

    let item: Item = item_response.json().await.unwrap();
    assert_eq(item.id, 1, "Item ID should match")?;
    assert_eq(item.name, "Test Item", "Item name should match")?;

    // Test the get item endpoint for a non-existing item
    let not_found_response = client.get("/items/999").send().await?;
    assert_eq(
        not_found_response.status().as_u16(),
        404,
        "Get item should return 404 Not Found for non-existing item",
    )?;

    let error_body: serde_json::Value = not_found_response.json().await.unwrap();
    assert_true(
        error_body["error"]["code"]
            .as_str()
            .unwrap()
            .contains("NOT_FOUND"),
        "Error code should indicate not found",
    )?;
    assert_contains(
        error_body["error"]["message"].as_str().unwrap(),
        "not found",
        "Error message should indicate not found",
    )?;

    // Shutdown the server
    server_handle.shutdown();

    // Wait for server to shut down
    let server_result = timeout(Duration::from_secs(5), server_handle.wait()).await;
    assert_true(server_result.is_ok(), "Server should shut down gracefully")?;

    Ok(())
}

// Add an extension trait to HttpServerHandle for testing
#[cfg(test)]
trait HttpServerHandleExt {
    fn address(&self) -> std::io::Result<std::net::SocketAddr>;
}

#[cfg(test)]
impl HttpServerHandleExt for navius_http::server::HttpServerHandle {
    fn address(&self) -> std::io::Result<std::net::SocketAddr> {
        // This is a hack for testing purposes only.
        // In a real application, we would track the address in the handle.
        Ok(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            3000, // This is just a placeholder - in a real test, we'd need to capture the actual port
        ))
    }
}

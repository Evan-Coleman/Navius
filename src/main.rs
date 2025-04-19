mod handlers;

// Main application with zero boilerplate
#[navius_app(
    name = "zero-boilerplate-app",
    routes = [self::api, self::admin]
)]
async fn main() {
    // The macro handles all the boilerplate for us!
    tracing::info!("Zero-boilerplate application started!");
}

// API route module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        handlers::api_handlers::hello_world().await
    }

    #[route(path = "/echo/{text}", method = "GET")]
    async fn echo(axum::extract::Path(text): axum::extract::Path<String>) -> String {
        handlers::api_handlers::echo(axum::extract::Path(text)).await
    }

    #[route(path = "/users", method = ["GET", "POST"])]
    async fn users(
        // Optional payload for POST requests
        payload: Option<Json<api_handler::UserRequest>>,
    ) -> impl axum::response::IntoResponse {
        handlers::api_handlers::users(payload).await
    }
}

// Admin routes
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        handlers::admin_handlers::status().await
    }
}

use axum::{Json, Router, extract::State, routing::get};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

#[derive(OpenApi)]
#[openapi(paths(get_pet), components(schemas(Pet)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/data", get(get_data))
        .route("/pet", get(get_pet_handler))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(client);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ApiResponse {
    fact: String,
}

#[derive(Deserialize, serde::Serialize)]
struct Data {
    data: String,
}

async fn get_data(State(client): State<Arc<Client>>) -> Json<Data> {
    let url = "https://catfact.ninja/fact";
    let response = client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<ApiResponse>() // Deserialize into ApiResponse first
        .await
        .unwrap();

    // Map to your desired structure
    Json(Data {
        data: response.fact,
    })
}

async fn get_pet_handler() -> impl axum::response::IntoResponse {
    match get_pet_by_id(1).await {
        Ok(pet) => axum::Json(pet),
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub status: Option<String>,
}

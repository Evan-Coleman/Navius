mod petstore_api;

use crate::petstore_api::models::{category, pet, tag};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};

use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// MODEL ENHANCEMENT LAYER
// This layer allows us to enhance the generated models with custom behavior
// and documentation while still using them directly with Utoipa

// A simpler OpenAPI definition to make it easier to debug
#[derive(OpenApi)]
#[openapi(paths(get_pet_by_id))]
struct ApiDoc;

// Helper to convert Status enum to string
fn status_to_string(status: &pet::Status) -> String {
    match status {
        pet::Status::Available => "available".to_string(),
        pet::Status::Pending => "pending".to_string(),
        pet::Status::Sold => "sold".to_string(),
    }
}

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/data", get(get_data))
        .route("/pet/{id}", get(get_pet_by_id))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(client);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ApiResponse {
    fact: String,
}

#[derive(Serialize, Deserialize)]
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
        .json::<ApiResponse>()
        .await
        .unwrap();

    Json(Data {
        data: response.fact,
    })
}

async fn get_pet_by_id_request(client: &Client, pet_id: i64) -> Result<pet::Pet, reqwest::Error> {
    let url = format!("https://petstore3.swagger.io/api/v3/pet/{}", pet_id);
    let response = client.get(&url).send().await?.json::<pet::Pet>().await?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/pet/{id}",
    params(
        ("id" = i64, Path, description = "Pet id to get")
    ),
    responses(
        (status = 200, description = "Pet found successfully"),
        (status = 404, description = "Pet not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_pet_by_id(
    State(client): State<Arc<Client>>,
    Path(pet_id): Path<i64>,
) -> impl IntoResponse {
    match get_pet_by_id_request(&client, pet_id).await {
        Ok(pet) => Json::<pet::Pet>(pet).into_response(),
        Err(e) => {
            eprintln!("Error fetching pet: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

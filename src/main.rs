mod petstore_api;

use crate::petstore_api::models::*;
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
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize)] // ✅ Now implements DeserializeOwned
struct PetSchema {
    inner: Pet, // Wraps Pet inside
}

// Manually implement `ToSchema` for `PetSchema`
impl ToSchema for PetSchema {
    fn schemas() -> utoipa::openapi::schema::Schema {
        <Pet as ToSchema>::schemas()
    }
}

#[derive(OpenApi)]
#[openapi(paths(get_pet_by_id), components(schemas(PetSchema)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/data", get(get_data))
        .route("/pet/{id}", get(get_pet_by_id)) // ✅ Now properly referenced
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

async fn get_pet_by_id_request(client: &Client, pet_id: i64) -> Result<PetSchema, reqwest::Error> {
    let url = format!("https://petstore3.swagger.io/api/v3/pet/{}", pet_id);
    let response = client.get(&url).send().await?.json::<PetSchema>().await?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/pet/{id}",
    responses(
        (status = 200, description = "Returns pet", body = PetSchema),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_pet_by_id(
    State(client): State<Arc<Client>>,
    Path(pet_id): Path<i64>,
) -> impl IntoResponse {
    match get_pet_by_id_request(&client, pet_id).await {
        Ok(pet) => {
            print!("Hello");
            Json(pet).into_response()
        }
        Err(_) => {
            print!("Wrong!");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

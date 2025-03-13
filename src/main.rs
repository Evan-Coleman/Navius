use axum::{Json, Router, extract::State, routing::get};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/data", get(get_data))
        .with_state(client);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
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
        .json::<Data>()
        .await
        .unwrap();
    Json(response)
}

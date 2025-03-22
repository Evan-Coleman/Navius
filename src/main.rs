// Copyright (c) 2025 Rust Backend Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rust_backend::app;
use rust_backend::config;
use rust_backend::error::error_types::AppError;

use std::{fs, path::Path, process};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set tracing subscriber: {}", err);
        process::exit(1);
    }

    // Run the application
    if let Err(err) = run_app().await {
        error!("Application error: {}", err);
        process::exit(1);
    }

    Ok(())
}

async fn run_app() -> Result<(), AppError> {
    // Initialize the application
    let (app, addr) = app::init().await;

    // Load configuration
    let config = config::app_config::load_config()?;
    let protocol = &config.server.protocol;

    // Ensure the OpenAPI directory exists
    let spec_directory = "config/swagger";
    if !Path::new(spec_directory).exists() {
        info!("Creating OpenAPI spec directory: {}", spec_directory);
        fs::create_dir_all(spec_directory).map_err(|e| {
            AppError::InternalError(format!("Failed to create OpenAPI directory: {}", e))
        })?;
    }

    // Start the server
    info!("Starting server on {}://{}", protocol, addr);

    // Bind the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run the server with our app
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::InternalError(format!("Server error: {}", e)))?;

    Ok(())
}

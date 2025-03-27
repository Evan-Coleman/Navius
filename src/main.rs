// Copyright (c) 2025 Navius Contributors
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

use navius::app;
use navius::config;
use navius::error::error_types::AppError;

use std::{fs, path::Path, process};
use tracing::{Level, error, info, warn};
use tracing_subscriber::FmtSubscriber;

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use navius::core::config::app_config::AppConfig;
use navius::core::config::load_config;
use navius::core::router;
use navius::core::router::core_app_router::{RouterBuilder, create_application};

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
    // Load configuration
    let config = config::app_config::load_config()?;

    // Get server address
    let addr = match SocketAddr::from_str(&format!("{}:{}", config.server.host, config.server.port))
    {
        Ok(addr) => addr,
        Err(err) => {
            return Err(AppError::internal_server_error(format!(
                "Invalid server address: {}",
                err
            )));
        }
    };

    // Ensure the OpenAPI directory exists
    let spec_directory = "config/swagger";
    if !Path::new(spec_directory).exists() {
        info!("Creating OpenAPI spec directory: {}", spec_directory);
        fs::create_dir_all(spec_directory).map_err(|e| {
            AppError::internal_server_error(format!("Failed to create OpenAPI directory: {}", e))
        })?;
    }

    // Initialize metrics
    let metrics_handle = navius::core::metrics::init_metrics();

    // Create a Spring Boot-like application
    let app = create_application()
        .with_config(config.clone())
        .with_metrics(Some(metrics_handle))
        .with_cors(true)
        .with_metrics_enabled(true);

    // Register services
    let app = navius::app::api::register_services(app);

    // Build the router
    let app = app.build();

    // Start the server
    info!(
        "Starting server on {}://{}:{}",
        config.server.protocol, config.server.host, config.server.port
    );

    // Bind the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run the server with our app
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::internal_server_error(format!("Server error: {}", e)))?;

    Ok(())
}

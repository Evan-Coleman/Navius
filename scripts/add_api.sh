#!/bin/bash

# Script to automate adding a new API endpoint with downstream integration
# Usage: ./add_api.sh <api_name> <api_url> <schema_url> [endpoint_path] [param_name]

set -e

# Check for required arguments
if [ "$#" -lt 3 ]; then
  echo "Usage: $0 <api_name> <api_url> <schema_url> [endpoint_path] [param_name]"
  echo "Example: $0 jsonplaceholder https://jsonplaceholder.typicode.com https://jsonplaceholder.typicode.com/swagger.json posts id"
  exit 1
fi

# Check for OpenAPI Generator
if ! command -v openapi-generator &> /dev/null; then
  echo "Warning: OpenAPI Generator is not installed."
  echo "This script needs it to generate API clients, but will continue without it for now."
  echo "To install, see https://openapi-generator.tech/docs/installation/"
  SKIP_API_GENERATION=true
else
  SKIP_API_GENERATION=false
fi

# Parse arguments
API_NAME=$1
API_URL=$2
SCHEMA_URL=$3
ENDPOINT_PATH=${4:-${API_NAME}}
PARAM_NAME=${5:-id}

# Convert API_NAME to camelCase and PascalCase
API_NAME_CAMEL=$(echo "$API_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g' | sed 's/^./\l&/')
API_NAME_PASCAL=$(echo "$API_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g')

echo "==== Adding $API_NAME API Integration ===="
echo "API URL: $API_URL"
echo "Schema URL: $SCHEMA_URL"
echo "Endpoint Path: $ENDPOINT_PATH"
echo "Parameter Name: $PARAM_NAME"

# Create directories
echo "Creating directories..."
mkdir -p "src/openapi/$API_NAME"
mkdir -p "src/models"

# 1. Download OpenAPI schema if it's a URL
if [[ "$SCHEMA_URL" == http* ]]; then
  echo "Downloading OpenAPI schema..."
  curl -s -o "src/openapi/$API_NAME/swagger.json" "$SCHEMA_URL"
elif [[ -f "$SCHEMA_URL" ]]; then
  echo "Using local OpenAPI schema file..."
  # If schema is local file, just use it directly for the next steps
  # No need to download
  SCHEMA_FILE="$SCHEMA_URL"
else
  echo "Error: Schema URL is not a valid URL or file path"
  exit 1
fi

# 2. Create OpenAPI Generator config
echo "Creating OpenAPI Generator config..."
cat > "src/openapi/$API_NAME/config.yaml" << EOF
generatorName: rust
outputDir: ./src/${API_NAME}_api
additionalProperties:
  packageName: ${API_NAME}
  serverFramework: axum
EOF

# 3. Create generation script
echo "Creating API generation script..."
cat > "src/openapi/$API_NAME/generate-api.sh" << EOF
#!/bin/bash

set -e

# Clean up previous generated files if they exist
if [ -d "./src/${API_NAME}_api" ]; then
    echo "Cleaning up previous generated files..."
    rm -rf ./src/${API_NAME}_api
fi

# Run OpenAPI Generator
echo "Running OpenAPI Generator..."
openapi-generator generate -i ./src/openapi/${API_NAME}/swagger.json -c ./src/openapi/${API_NAME}/config.yaml

# Create a module file
echo "Creating module declaration file..."
cat > ./src/${API_NAME}_api/mod.rs << EOF2
pub mod models;
EOF2

echo "API generation complete."
EOF

chmod +x "src/openapi/$API_NAME/generate-api.sh"

# 4. Run the script to generate the API client
if [ "$SKIP_API_GENERATION" = false ]; then
  echo "Generating API client..."
  ./src/openapi/$API_NAME/generate-api.sh
else
  echo "Skipping API client generation due to missing OpenAPI Generator..."
  # Create a simple models directory for temporary use
  mkdir -p ./src/${API_NAME}_api/models
  echo "pub mod models;" > ./src/${API_NAME}_api/mod.rs
fi

# 5. Create models file
echo "Creating models file..."
cat > "src/models/${API_NAME}.rs" << EOF
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Enhanced ${API_NAME_PASCAL} model with additional metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Enhanced${API_NAME_PASCAL} {
    /// Item ID
    #[schema(example = 1)]
    pub id: i32,
    
    /// Item title
    #[schema(example = "Enhanced: Sample title")]
    pub title: String,
    
    /// Enhanced body with additional information
    #[schema(example = "Enhanced content with metadata")]
    pub enhanced_body: String,
    
    /// Word count in the body
    #[schema(example = 120)]
    pub word_count: usize,
    
    /// Estimated reading time in minutes
    #[schema(example = 2)]
    pub reading_time_minutes: usize,
    
    /// Additional metadata field
    #[schema(example = "extra information")]
    pub metadata: String,
}
EOF

# 6. Create handler file
echo "Creating handler file..."
cat > "src/handlers/${ENDPOINT_PATH}.rs" << EOF
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::app::AppState;
use crate::models::${API_NAME}::Enhanced${API_NAME_PASCAL};

/// Handler for the enhanced ${ENDPOINT_PATH} endpoint
#[utoipa::path(
    get,
    path = "/${ENDPOINT_PATH}/{${PARAM_NAME}}/enhanced",
    params(
        ("${PARAM_NAME}" = i32, Path, description = "Item ID to fetch and enhance")
    ),
    responses(
        (status = 200, description = "Enhanced item retrieved successfully", body = Enhanced${API_NAME_PASCAL}),
        (status = 404, description = "Item not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "${ENDPOINT_PATH}"
)]
pub async fn get_enhanced_${API_NAME_CAMEL}(
    Path(${PARAM_NAME}): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Enhanced${API_NAME_PASCAL}>, (axum::http::StatusCode, String)> {
    info!("Fetching and enhancing ${API_NAME} with ID: {}", ${PARAM_NAME});

    // Construct the URL to fetch the item by ID
    let url = format!("{}/posts/{}", state.config.api.${API_NAME}_url, ${PARAM_NAME});
    
    // Make request to the API
    let response = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            warn!("Failed to fetch item: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch item: {}", e),
            )
        })?;

    // Check if response is successful
    if response.status().is_success() {
        // Parse the API response
        let item = response.json::<serde_json::Value>().await.map_err(|e| {
            warn!("Failed to parse response: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse response: {}", e),
            )
        })?;
        
        // Extract content from the response - adjust these based on actual API response structure
        let title = item["title"].as_str().unwrap_or("No title").to_string();
        let body = item["body"].as_str().unwrap_or("No content").to_string();
        
        // Transform the data
        let words = body.split_whitespace().count();
        let reading_time = (words as f64 / 200.0).ceil() as usize;  // Assuming 200 words per minute
        
        // Create enhanced item
        let enhanced_item = Enhanced${API_NAME_PASCAL} {
            id: item["id"].as_i64().unwrap_or_default() as i32,
            title: format!("Enhanced: {}", title),
            enhanced_body: format!("{}. Word count: {}. Reading time: {} minute(s).", 
                              body, words, reading_time),
            word_count: words,
            reading_time_minutes: reading_time,
            metadata: format!("Fetched from {}", "${API_URL}"),
        };
        
        info!("Successfully enhanced ${API_NAME} ID: {}", ${PARAM_NAME});
        Ok(Json(enhanced_item))
    } else if response.status() == axum::http::StatusCode::NOT_FOUND {
        warn!("Item with ID {} not found", ${PARAM_NAME});
        Err((axum::http::StatusCode::NOT_FOUND, format!("Item with ID {} not found", ${PARAM_NAME})))
    } else {
        warn!("API returned error status: {}", response.status());
        Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("API returned error status: {}", response.status()),
        ))
    }
}
EOF

# 7. Create backup of existing files
echo "Creating backups of files to be modified..."
cp src/lib.rs src/lib.rs.bak
cp src/models/mod.rs src/models/mod.rs.bak
cp src/handlers/mod.rs src/handlers/mod.rs.bak
cp src/config/app_config.rs src/config/app_config.rs.bak
cp src/app/router.rs src/app/router.rs.bak
cp config/default.yaml config/default.yaml.bak

# 8. Update lib.rs to include the new API module
echo "Updating lib.rs..."
if ! grep -q "${API_NAME}_api" src/lib.rs; then
    sed -i.tmp "/pub mod petstore_api;/a\\
/// ${API_NAME_PASCAL} API client\\
pub mod ${API_NAME}_api;" src/lib.rs
    rm -f src/lib.rs.tmp
fi

# 9. Update models/mod.rs to include the new models
echo "Updating models/mod.rs..."
if ! grep -q "pub mod ${API_NAME};" src/models/mod.rs; then
    sed -i.tmp "/pub mod schemas;/a\\
pub mod ${API_NAME};" src/models/mod.rs
    rm -f src/models/mod.rs.tmp
fi

if ! grep -q "use ${API_NAME}::Enhanced${API_NAME_PASCAL};" src/models/mod.rs; then
    line_number=$(grep -n "pub use schemas::" src/models/mod.rs | cut -d':' -f1)
    last_line=$(grep -n "};" src/models/mod.rs | cut -d':' -f1)
    sed -i.tmp "${last_line}s/};/    Enhanced${API_NAME_PASCAL},\\
};/" src/models/mod.rs
    sed -i.tmp "/pub use schemas::/a\\
pub use ${API_NAME}::Enhanced${API_NAME_PASCAL};" src/models/mod.rs
    rm -f src/models/mod.rs.tmp
fi

# 10. Update handlers/mod.rs to include the new handler
echo "Updating handlers/mod.rs..."
if ! grep -q "pub mod ${ENDPOINT_PATH};" src/handlers/mod.rs; then
    sed -i.tmp "/pub mod pet;/a\\
pub mod ${ENDPOINT_PATH};" src/handlers/mod.rs
    rm -f src/handlers/mod.rs.tmp
fi

if ! grep -q "pub use ${ENDPOINT_PATH}::get_enhanced_${API_NAME_CAMEL};" src/handlers/mod.rs; then
    sed -i.tmp "/pub use pet::get_pet_by_id;/a\\
pub use ${ENDPOINT_PATH}::get_enhanced_${API_NAME_CAMEL};" src/handlers/mod.rs
    rm -f src/handlers/mod.rs.tmp
fi

# 11. Update config/app_config.rs to add the new API URL
echo "Updating config/app_config.rs..."
if ! grep -q "${API_NAME}_url" src/config/app_config.rs; then
    sed -i.tmp "/pub petstore_url: String,/a\\
    pub ${API_NAME}_url: String," src/config/app_config.rs
    rm -f src/config/app_config.rs.tmp
fi

# 12. Update config/default.yaml to include the new API URL
echo "Updating config/default.yaml..."
if ! grep -q "${API_NAME}_url" config/default.yaml; then
    sed -i.tmp "/petstore_url:/a\\
  ${API_NAME}_url: \"${API_URL}\"" config/default.yaml
    rm -f config/default.yaml.tmp
fi

# 13. Update router.rs to add the new route and documentation
echo "Updating app/router.rs..."
if ! grep -q "get_enhanced_${API_NAME_CAMEL}" src/app/router.rs; then
    # Add to API paths
    sed -i.tmp "/crate::handlers::pet::get_pet_by_id,/a\\
        crate::handlers::${ENDPOINT_PATH}::get_enhanced_${API_NAME_CAMEL}," src/app/router.rs
    
    # Add to schemas
    sed -i.tmp "/crate::cache::CacheStats,/a\\
            crate::models::${API_NAME}::Enhanced${API_NAME_PASCAL}," src/app/router.rs
    
    # Add to tags
    sed -i.tmp "/(name = \"pets\", description = \"Pet endpoints\"),/a\\
        (name = \"${ENDPOINT_PATH}\", description = \"${API_NAME_PASCAL} endpoints\")," src/app/router.rs
    
    # Add route
    sed -i.tmp "/route(\"\/pet\/{id}\", get(handlers::pet::get_pet_by_id))/a\\
        .route(\"/${ENDPOINT_PATH}/{${PARAM_NAME}}/enhanced\", get(handlers::${ENDPOINT_PATH}::get_enhanced_${API_NAME_CAMEL}))" src/app/router.rs
    
    rm -f src/app/router.rs.tmp
fi

echo "==== Integration Complete ===="
echo "New API endpoint added: /${ENDPOINT_PATH}/{${PARAM_NAME}}/enhanced"
echo ""
echo "Next Steps:"
echo "1. Run 'cargo build' to verify everything compiles"
echo "2. Start the server with './run_server.sh'"
echo "3. Test the new endpoint with 'curl http://localhost:3000/${ENDPOINT_PATH}/1/enhanced'"
echo ""
echo "If there are any issues, backups of the modified files were created (.bak extension)" 
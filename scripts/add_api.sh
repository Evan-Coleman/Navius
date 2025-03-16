#!/bin/bash

# add_api.sh - Tool for adding new API clients to the Rust backend
#
# This script automates the process of adding a new API client to the system:
# 1. Downloads the OpenAPI/Swagger schema if needed
# 2. Generates API client code in Rust
# 3. Creates enhanced models and handlers
# 4. Updates routing and configuration
# 5. Registers the API in the registry for future regeneration

set -e  # Exit immediately if a command exits with a non-zero status

# Argument handling
if [ "$#" -lt 4 ]; then
    echo "Usage: $0 <api_name> <api_url> <schema_path> <entity_name> [<id_field>]"
    echo ""
    echo "Parameters:"
    echo "  api_name    - Name of the API (e.g., 'petstore', 'pokeapi')"
    echo "  api_url     - Base URL of the API (e.g., 'https://petstore3.swagger.io/api/v3')"
    echo "  schema_path - Path to OpenAPI/Swagger schema (local file or URL)"
    echo "  entity_name - Name of the main entity (e.g., 'pet', 'pokemon')"
    echo "  id_field    - Optional: Name of ID field (default: 'id')"
    echo ""
    echo "Example:"
    echo "  $0 petstore https://petstore3.swagger.io/api/v3 src/openapi/petstore/openapi.yaml pet id"
    exit 1
fi

API_NAME="$1"
API_URL="$2"
SCHEMA_PATH="$3"
ENTITY_NAME="$4"
ID_FIELD="${5:-id}"
API_REGISTRY="api_registry.json"

# Convert API_NAME to CamelCase for handler function
API_NAME_CAMEL=$(echo "$API_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g')

# Convert API_NAME to snake_case for Rust modules
API_NAME_SNAKE=$(echo "$API_NAME" | tr '[:upper:]' '[:lower:]' | tr '-' '_')

# Convert ENTITY_NAME to CamelCase for handler function
ENTITY_NAME_CAMEL=$(echo "$ENTITY_NAME" | sed -r 's/(^|_)([a-z])/\U\2/g')

# Create necessary directories
mkdir -p generated/openapi/${API_NAME}
mkdir -p generated/${API_NAME}_api/src/api
mkdir -p generated/${API_NAME}_api/src/models

# Download schema if it's a URL
if [[ "$SCHEMA_PATH" == http* ]]; then
    echo "Downloading schema from $SCHEMA_PATH..."
    SCHEMA_FILE="generated/openapi/${API_NAME}/openapi.yaml"
    curl -s -o "$SCHEMA_FILE" "$SCHEMA_PATH"
    SCHEMA_PATH="$SCHEMA_FILE"
elif [[ ! -f "$SCHEMA_PATH" ]]; then
    echo "Error: Schema file not found at $SCHEMA_PATH"
    exit 1
else
    # Copy the schema to our generated folder for consistency
    echo "Copying schema from $SCHEMA_PATH..."
    mkdir -p "$(dirname "generated/openapi/${API_NAME}/openapi.yaml")"
    cp "$SCHEMA_PATH" "generated/openapi/${API_NAME}/openapi.yaml"
fi

# Create OpenAPI Generator config file
CONFIG_FILE="generated/openapi/${API_NAME}/config.yaml"

echo "Creating OpenAPI Generator configuration..."
cat > "$CONFIG_FILE" << EOF
# OpenAPI Generator configuration for $API_NAME
generatorName: rust
outputDir: ../../../generated/${API_NAME}_api
packageName: ${API_NAME}_api
library: reqwest
apiPackage: api
modelPackage: models
# Additional properties
additionalProperties:
  supportMultipleResponses: true
  enumNameSuffix: ""
  structPrefix: true
  dateLibrary: chrono
  useSingleRequestParameter: true
EOF

# Check for OpenAPI Generator
if ! command -v openapi-generator &> /dev/null; then
    echo "Error: OpenAPI Generator is not installed."
    echo "Please install it from: https://openapi-generator.tech/docs/installation/"
    exit 1
fi

# Run OpenAPI Generator
echo "Running OpenAPI Generator..."
openapi-generator generate -i "generated/openapi/${API_NAME}/openapi.yaml" -c "$CONFIG_FILE"

if [ $? -ne 0 ]; then
    echo "Error: OpenAPI Generator failed."
    exit 1
fi

echo "API client generated successfully in generated/${API_NAME}_api/"

# Fix structure issues - create the API and models modules
if [ ! -f "generated/${API_NAME}_api/src/api/mod.rs" ]; then
    echo "Creating API module..."
    mkdir -p "generated/${API_NAME}_api/src/api"
    echo "// API module - stub for compatibility" > "generated/${API_NAME}_api/src/api/mod.rs"
fi

if [ ! -f "generated/${API_NAME}_api/src/models/mod.rs" ]; then
    echo "Creating models module..."
    mkdir -p "generated/${API_NAME}_api/src/models"
    
    # Create a simple Pet model for demonstration
    cat > "generated/${API_NAME}_api/src/models/mod.rs" << EOF
// Generated models for the ${API_NAME} API
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// ${ENTITY_NAME_CAMEL} model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ${ENTITY_NAME_CAMEL} {
    /// ID of the ${ENTITY_NAME}
    pub id: i64,
    
    /// Name of the ${ENTITY_NAME}
    pub name: String,
    
    /// Tags for this ${ENTITY_NAME}
    #[serde(default)]
    pub tags: Vec<Tag>,
    
    /// Status of this ${ENTITY_NAME}
    pub status: Option<String>,
    
    /// Category this ${ENTITY_NAME} belongs to
    pub category: Option<Category>,
}

/// Tag model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tag {
    /// ID of the tag
    pub id: i64,
    
    /// Name of the tag
    pub name: String,
}

/// Category model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Category {
    /// ID of the category
    pub id: i64,
    
    /// Name of the category
    pub name: String,
}
EOF
fi

# Create enhanced model for the entity
echo "Creating enhanced model for $ENTITY_NAME..."
mkdir -p "generated/${API_NAME}_api/src/enhanced_models"

cat > "generated/${API_NAME}_api/src/enhanced_models/mod.rs" << EOF
// Enhanced models for ${API_NAME_SNAKE} API

use crate::models;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Enhanced ${ENTITY_NAME_CAMEL} with additional fields and derived properties
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Enhanced${ENTITY_NAME_CAMEL} {
    /// The original ${ENTITY_NAME} data
    #[serde(flatten)]
    pub ${ENTITY_NAME}: models::${ENTITY_NAME_CAMEL},

    /// Generated score/rating
    pub rating: f64,

    /// Additional metadata about the ${ENTITY_NAME}
    pub metadata: String,
}
EOF

# Create handler for the enhanced entity
echo "Creating handler for enhanced $ENTITY_NAME..."
mkdir -p "generated/${API_NAME}_api/src/handlers"

cat > "generated/${API_NAME}_api/src/handlers/mod.rs" << EOF
// Handlers for ${API_NAME_SNAKE} API

use axum::{
    extract::{Path, State},
    Json,
};
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    enhanced_models::Enhanced${ENTITY_NAME_CAMEL},
    models::${ENTITY_NAME_CAMEL},
};

/// Get enhanced ${ENTITY_NAME} by ID
///
/// Fetches a ${ENTITY_NAME} from the ${API_NAME} API and enhances it with additional data
#[utoipa::path(
    get,
    path = "/${ENTITY_NAME}/{id}/enhanced",
    params(
        ("id" = String, Path, description = "${ENTITY_NAME_CAMEL} ID to fetch")
    ),
    responses(
        (status = 200, description = "${ENTITY_NAME_CAMEL} found and enhanced", body = Enhanced${ENTITY_NAME_CAMEL}),
        (status = 404, description = "${ENTITY_NAME_CAMEL} not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "${API_NAME}"
)]
pub async fn get_enhanced_${ENTITY_NAME}(
    Path(id): Path<String>,
    State(state): State<Arc<rust_backend::app::AppState>>,
) -> Result<Json<Enhanced${ENTITY_NAME_CAMEL}>, (StatusCode, String)> {
    info!("Fetching enhanced ${ENTITY_NAME} with ID: {}", id);

    // Construct API URL
    let url = format!("{}/api/v2/${ENTITY_NAME}/{}", state.config.api.${API_NAME_SNAKE}_url, id);
    
    // Fetch data from API
    let response = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            warn!("API request failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch ${ENTITY_NAME}: {}", e))
        })?;

    // Handle non-success responses
    if response.status() == StatusCode::NOT_FOUND {
        warn!("${ENTITY_NAME_CAMEL} with ID {} not found", id);
        return Err((
            StatusCode::NOT_FOUND,
            format!("${ENTITY_NAME_CAMEL} with ID {} not found", id),
        ));
    }

    if !response.status().is_success() {
        warn!("API returned error status: {}", response.status());
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("API returned error status: {}", response.status()),
        ));
    }

    // Parse the response
    let ${ENTITY_NAME} = response.json::<${ENTITY_NAME_CAMEL}>().await.map_err(|e| {
        warn!("Failed to parse response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse ${ENTITY_NAME} data: {}", e),
        )
    })?;

    // Calculate a rating based on some properties
    // This is just an example - customize based on your entity's fields
    let rating = calculate_rating(&${ENTITY_NAME});

    // Create the enhanced entity
    let enhanced = Enhanced${ENTITY_NAME_CAMEL} {
        ${ENTITY_NAME},
        rating,
        metadata: format!("Enhanced ${ENTITY_NAME} data. Generated at: {}", chrono::Utc::now()),
    };

    info!("Successfully enhanced ${ENTITY_NAME} with ID: {}", id);
    Ok(Json(enhanced))
}

// Example rating calculation function
fn calculate_rating(${ENTITY_NAME}: &${ENTITY_NAME_CAMEL}) -> f64 {
    // This is a placeholder implementation - customize based on your entity's fields
    // Example: For Pokemon, might use stats; for products, might use ratings
    
    // Just a dummy value for now
    let name_length = ${ENTITY_NAME}.name.len() as f64;
    (name_length * 7.5) % 10.0 + 1.0 // Rating between 1.0 and 10.0
}
EOF

# Create module file
echo "Creating module file..."
cat > "generated/${API_NAME}_api/src/lib.rs" << EOF
//! ${API_NAME_CAMEL} API Client
//!
//! This is an auto-generated API client for the ${API_NAME} API.

pub mod api;
pub mod models;
pub mod enhanced_models;
pub mod handlers;

// Re-export commonly used types
pub use enhanced_models::Enhanced${ENTITY_NAME_CAMEL};
pub use models::${ENTITY_NAME_CAMEL};
EOF

# Update the bridge file to include the new API
echo "Updating generated_apis.rs..."
cat > "src/generated_apis.rs" << EOF
//! Generated API modules
//!
//! This file serves as a bridge to the generated API code in the /generated directory.
//! It uses the #[path] attribute to reference files outside of the src directory.

#[path = "../generated/${API_NAME}_api/src/lib.rs"]
pub mod ${API_NAME}_api;

// The following re-exports make the API types available directly from generated_apis
pub use ${API_NAME}_api::Enhanced${ENTITY_NAME_CAMEL};
pub use ${API_NAME}_api::${ENTITY_NAME_CAMEL};
EOF

# Update app router to add the new endpoint
echo "Updating router.rs..."
ROUTER_FILE="src/app/router.rs"

# Use awk to find the Router::new line and add our route after it
awk -v api_name="$API_NAME_SNAKE" -v entity_name="$ENTITY_NAME" '
/Router::new\(\)/ {
    print $0;
    print "        .route(\"/" entity_name "/:id/enhanced\", get(crate::generated_apis::" api_name "_api::handlers::get_enhanced_" entity_name "))";
    next;
}
1
' "$ROUTER_FILE" > "${ROUTER_FILE}.new"
mv "${ROUTER_FILE}.new" "$ROUTER_FILE"

# Add the import for the handler
# Fix the sed command with a different approach
if ! grep -q "use crate::generated_apis" "$ROUTER_FILE"; then
    sed -i'.bak' '/use crate::handlers/a\\
use crate::generated_apis;' "$ROUTER_FILE" && rm -f "${ROUTER_FILE}.bak"
fi

# Check if we need to add http import
if ! grep -q "http::{HeaderMap, HeaderValue, Method}" "$ROUTER_FILE"; then
    sed -i'.bak' 's/use axum::{/use axum::{http::{HeaderMap, HeaderValue, Method},/' "$ROUTER_FILE" && rm -f "${ROUTER_FILE}.bak"
fi

# Update the config file to include the API URL
echo "Updating configuration..."
CONFIG_FILE="config/default.yaml"

# Use awk to add the new API URL to the api section
awk -v api_name="$API_NAME_SNAKE" -v api_url="$API_URL" '
/api:/ {
    print $0;
    in_api = 1;
    next;
}
/^[a-z]/ {
    if (in_api) {
        in_api = 0;
        print "  " api_name "_url: \"" api_url "\"";
    }
}
1
' "$CONFIG_FILE" > "${CONFIG_FILE}.new"
mv "${CONFIG_FILE}.new" "$CONFIG_FILE"

# Update the API registry
echo "Updating API registry..."
if [ -f "$API_REGISTRY" ]; then
    # Check if this API is already in the registry
    if jq -e ".apis[] | select(.name == \"$API_NAME\")" "$API_REGISTRY" > /dev/null; then
        echo "API $API_NAME already exists in registry. Updating..."
        # Update the existing entry
        jq --arg name "$API_NAME" \
           --arg url "$API_URL" \
           --arg schema "generated/openapi/${API_NAME}/openapi.yaml" \
           --arg entity "$ENTITY_NAME" \
           --arg id "$ID_FIELD" \
           '.apis = [.apis[] | if .name == $name then {name: $name, url: $url, schema_path: $schema, entity_name: $entity, id_field: $id} else . end]' \
           "$API_REGISTRY" > "${API_REGISTRY}.new"
        mv "${API_REGISTRY}.new" "$API_REGISTRY"
    else
        # Add a new entry
        jq --arg name "$API_NAME" \
           --arg url "$API_URL" \
           --arg schema "generated/openapi/${API_NAME}/openapi.yaml" \
           --arg entity "$ENTITY_NAME" \
           --arg id "$ID_FIELD" \
           '.apis += [{name: $name, url: $url, schema_path: $schema, entity_name: $entity, id_field: $id}]' \
           "$API_REGISTRY" > "${API_REGISTRY}.new"
        mv "${API_REGISTRY}.new" "$API_REGISTRY"
    fi
else
    # Create a new registry file
    cat > "$API_REGISTRY" << EOF
{
  "apis": [
    {
      "name": "$API_NAME",
      "url": "$API_URL",
      "schema_path": "generated/openapi/${API_NAME}/openapi.yaml",
      "entity_name": "$ENTITY_NAME",
      "id_field": "$ID_FIELD"
    }
  ]
}
EOF
fi

echo "✅ Successfully added ${API_NAME} API!"
echo "🚀 New endpoint available at: /${ENTITY_NAME}/{id}/enhanced"
echo "📁 Generated code in: generated/${API_NAME}_api/" 
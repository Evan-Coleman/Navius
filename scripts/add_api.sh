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
  echo "This script normally needs it to generate API clients, but will continue without it for now."
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
mkdir -p "generated/openapi/$API_NAME"
mkdir -p "src/models"

# Make sure the generated directory exists in gitignore
if ! grep -q "^/generated/" .gitignore; then
  echo "Adding /generated/ to .gitignore..."
  echo -e "\n# Generated API clients\n/generated/" >> .gitignore
fi

# 1. Download OpenAPI schema if it's a URL
if [[ "$SCHEMA_URL" == http* ]]; then
  echo "Downloading OpenAPI schema..."
  curl -s -o "generated/openapi/$API_NAME/schema.json" "$SCHEMA_URL"
  SCHEMA_FILE="generated/openapi/$API_NAME/schema.json"
elif [[ -f "$SCHEMA_URL" ]]; then
  echo "Using local OpenAPI schema file..."
  # If schema is local file, copy it to our directory with a consistent name
  cp "$SCHEMA_URL" "generated/openapi/$API_NAME/schema.json"
  SCHEMA_FILE="generated/openapi/$API_NAME/schema.json"
else
  echo "Error: Schema URL is not a valid URL or file path"
  exit 1
fi

# 2. Create OpenAPI Generator config
echo "Creating OpenAPI Generator config..."
cat > "generated/openapi/$API_NAME/config.yaml" << EOF
generatorName: rust
outputDir: ./generated/${API_NAME}_api
additionalProperties:
  packageName: ${API_NAME}
  serverFramework: axum
EOF

# 3. Create generation script
echo "Creating API generation script..."
cat > "generated/openapi/$API_NAME/generate-api.sh" << EOF
#!/bin/bash

set -e

# Clean up previous generated files if they exist
if [ -d "./generated/${API_NAME}_api" ]; then
    echo "Cleaning up previous generated files..."
    rm -rf ./generated/${API_NAME}_api
fi

# Run OpenAPI Generator
echo "Running OpenAPI Generator..."
openapi-generator generate -i ./generated/openapi/${API_NAME}/schema.json -c ./generated/openapi/${API_NAME}/config.yaml

# Create a module file
echo "Creating module declaration file..."
cat > ./generated/${API_NAME}_api/mod.rs << EOF2
pub mod models;
EOF2

echo "API generation complete."
EOF

chmod +x "generated/openapi/$API_NAME/generate-api.sh"

# 4. Run the script to generate the API client
if [ "$SKIP_API_GENERATION" = false ]; then
  echo "Generating API client..."
  ./generated/openapi/$API_NAME/generate-api.sh
else
  echo "Skipping API client generation due to missing OpenAPI Generator..."
  # Create a simple models directory for temporary use
  mkdir -p ./generated/${API_NAME}_api/models
  echo "pub mod models;" > ./generated/${API_NAME}_api/mod.rs
  mkdir -p ./generated/${API_NAME}_api/models
  touch ./generated/${API_NAME}_api/models/mod.rs
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
        ("${PARAM_NAME}" = String, Path, description = "Item ID or name to fetch and enhance")
    ),
    responses(
        (status = 200, description = "Enhanced item retrieved successfully", body = Enhanced${API_NAME_PASCAL}),
        (status = 404, description = "Item not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "${ENDPOINT_PATH}"
)]
pub async fn get_enhanced_${API_NAME}(
    Path(${PARAM_NAME}): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Enhanced${API_NAME_PASCAL}>, (axum::http::StatusCode, String)> {
    info!("Fetching and enhancing ${API_NAME} with ID: {}", ${PARAM_NAME});

    // Construct the URL to fetch the item by ID
    let url = format!("{}/pokemon/{}", state.config.api.${API_NAME}_url, ${PARAM_NAME});
    
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
        let pokemon_id = item["id"].as_i64().unwrap_or_default() as i32;
        let name = item["name"].as_str().unwrap_or("unknown").to_string();
        
        // For PokeAPI, get the first character uppercase
        let title = format!("Pokemon: {}", name.chars().next().unwrap_or('u').to_uppercase().collect::<String>() + &name[1..]);
        
        // Extract description from species endpoint
        let species_url = item["species"]["url"].as_str().unwrap_or_default();
        let body_text = if !species_url.is_empty() {
            match state.client.get(species_url).send().await {
                Ok(res) if res.status().is_success() => {
                    match res.json::<serde_json::Value>().await {
                        Ok(data) => {
                            // Find English flavor text
                            let flavor_texts = data["flavor_text_entries"].as_array();
                            let mut description = String::new();
                            
                            if let Some(texts) = flavor_texts {
                                for entry in texts {
                                    if let Some(lang) = entry["language"]["name"].as_str() {
                                        if lang == "en" {
                                            description = entry["flavor_text"].as_str()
                                                .unwrap_or_default()
                                                .replace(r"\n", " ")
                                                .replace(r"\f", " ");
                                            break;
                                        }
                                    }
                                }
                            }
                            description
                        },
                        Err(_) => "No description available.".to_string()
                    }
                },
                _ => "No description available.".to_string()
            }
        } else {
            "No description available.".to_string()
        };
        
        let word_count = body_text.split_whitespace().count();
        let reading_time = (word_count as f64 / 200.0).ceil() as usize;  // Assuming 200 words per minute
        
        // Extract types for metadata
        let mut types = Vec::new();
        if let Some(types_array) = item["types"].as_array() {
            for type_entry in types_array {
                if let Some(type_name) = type_entry["type"]["name"].as_str() {
                    types.push(type_name.to_string());
                }
            }
        }
        
        // Extract abilities for metadata
        let mut abilities = Vec::new();
        if let Some(abilities_array) = item["abilities"].as_array() {
            for ability_entry in abilities_array {
                if let Some(ability_name) = ability_entry["ability"]["name"].as_str() {
                    abilities.push(ability_name.to_string());
                }
            }
        }
        
        // Create enhanced item
        let enhanced_item = Enhanced${API_NAME_PASCAL} {
            id: pokemon_id,
            title,
            enhanced_body: body_text,
            word_count,
            reading_time_minutes: reading_time,
            metadata: format!("Types: {}. Abilities: {}.", 
                           types.join(", "), 
                           abilities.join(", "))
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
    # Create a module to import the generated API
    cat > "src/generated_apis.rs" << EOM
//! This module re-exports APIs from the generated directory
#[path = "../generated/${API_NAME}_api/mod.rs"]
pub mod ${API_NAME}_api;
EOM

    # Add the generated_apis module to lib.rs if it doesn't exist
    if ! grep -q "pub mod generated_apis;" src/lib.rs; then
        sed -i.tmp "/pub mod petstore_api;/a\\
/// Re-exports generated API clients\\
pub mod generated_apis;" src/lib.rs
        rm -f src/lib.rs.tmp
    fi
    
    # Add the API to the generated_apis module
    sed -i.tmp "/pub mod generated_apis;/a\\
pub use generated_apis::${API_NAME}_api;" src/lib.rs
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

if ! grep -q "pub use ${ENDPOINT_PATH}::get_enhanced_${API_NAME};" src/handlers/mod.rs; then
    sed -i.tmp "/pub use pet::get_pet_by_id;/a\\
pub use ${ENDPOINT_PATH}::get_enhanced_${API_NAME};" src/handlers/mod.rs
    rm -f src/handlers/mod.rs.tmp
fi

# 11. Update config/app_config.rs to add the new API URL
echo "Updating config/app_config.rs..."
if ! grep -q "${API_NAME}_url" src/config/app_config.rs; then
    sed -i.tmp "/pub petstore_url: String,/a\\
    pub ${API_NAME}_url: String," src/config/app_config.rs
    rm -f src/config/app_config.rs.tmp
fi

# Add default value to AppConfig::new()
if ! grep -q "${API_NAME}_url" src/config/app_config.rs | grep -q "set_default"; then
    sed -i.tmp "/set_default(\"api.petstore_url\".*)/a\\
            .set_default(\"api.${API_NAME}_url\", \"${API_URL}\")?" src/config/app_config.rs
    rm -f src/config/app_config.rs.tmp
fi

# 12. Update config/default.yaml to include the new API URL
echo "Updating config/default.yaml..."
if ! grep -q "${API_NAME}_url" config/default.yaml; then
    # Ensure proper YAML formatting when adding new line - add to new line after petstore_url
    # Use awk to ensure proper formatting and avoid syntax errors
    awk -v api_name="${API_NAME}" -v api_url="${API_URL}" '
    /petstore_url:/ {
        print $0;
        print "  " api_name "_url: \"" api_url "\"";
        next;
    }
    { print }
    ' config/default.yaml > config/default.yaml.new
    
    # Replace the original file with the modified one
    mv config/default.yaml.new config/default.yaml
fi

# 13. Update router.rs to add the new route and documentation
echo "Updating app/router.rs..."
if ! grep -q "get_enhanced_${API_NAME}" src/app/router.rs; then
    # Add to API paths
    sed -i.tmp "/crate::handlers::pet::get_pet_by_id,/a\\
        crate::handlers::${ENDPOINT_PATH}::get_enhanced_${API_NAME}," src/app/router.rs
    
    # Add to schemas
    sed -i.tmp "/crate::cache::CacheStats,/a\\
            crate::models::${API_NAME}::Enhanced${API_NAME_PASCAL}," src/app/router.rs
    
    # Add to tags
    sed -i.tmp "/(name = \"pets\", description = \"Pet endpoints\"),/a\\
        (name = \"${ENDPOINT_PATH}\", description = \"${API_NAME_PASCAL} endpoints\")," src/app/router.rs
    
    # Add route (ensuring proper formatting and no trailing comma issues)
    sed -i.tmp "s|.route(\"/pet/{id}\", get(handlers::pet::get_pet_by_id))|.route(\"/pet/{id}\", get(handlers::pet::get_pet_by_id))\n        .route(\"/${ENDPOINT_PATH}/{${PARAM_NAME}}/enhanced\", get(handlers::${ENDPOINT_PATH}::get_enhanced_${API_NAME}))|" src/app/router.rs
    
    rm -f src/app/router.rs.tmp
fi

# 14. Verify directory structures for API module
echo "Verifying API module structure..."
mkdir -p "./generated/${API_NAME}_api/models"
touch "./generated/${API_NAME}_api/models/mod.rs"

# 15. Create a custom build script to copy generated files if needed
echo "Creating custom build script..."
if [ ! -f "build.rs" ]; then
    cat > "build.rs" << EOF
fn main() {
    // Tell Cargo to rerun this build script if any file in the generated directory changes
    println!("cargo:rerun-if-changed=generated");
}
EOF
fi

echo "==== Integration Complete ===="
echo "New API endpoint added: /${ENDPOINT_PATH}/{${PARAM_NAME}}/enhanced"
echo ""
echo "Generated files placed in: /generated/${API_NAME}_api"
echo "The /generated/ directory has been added to .gitignore"
echo ""
echo "Next Steps:"
echo "1. Run 'cargo build' to verify everything compiles"
echo "2. Start the server with './run_server.sh'"
echo "3. Test the new endpoint with 'curl http://localhost:3000/${ENDPOINT_PATH}/1/enhanced'"
echo ""
echo "If there are any issues, backups of the modified files were created (.bak extension)" 
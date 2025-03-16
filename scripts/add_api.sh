#!/bin/bash

# add_api.sh - Tool for adding new API clients to the Rust backend
#
# This script automates the process of adding a new API client to the system:
# 1. Downloads the OpenAPI/Swagger schema if needed
# 2. Generates API client code in Rust based on registry configuration
# 3. Creates models and handlers as specified in the registry
# 4. Updates configuration
# 5. Registers the API in the registry for future regeneration

set -e  # Exit immediately if a command exits with a non-zero status

# Argument handling
if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <api_name> [<api_url> <schema_path> <entity_name> <id_field>]"
    echo ""
    echo "Parameters:"
    echo "  api_name    - Name of the API to add or update (e.g., 'petstore', 'pokeapi')"
    echo "  api_url     - Base URL of the API (e.g., 'https://petstore3.swagger.io/api/v3')"
    echo "  schema_path - Path to OpenAPI/Swagger schema (local file or URL)"
    echo "  entity_name - Name of the main entity (e.g., 'pet', 'pokemon')"
    echo "  id_field    - Optional: Name of ID field (default: 'id')"
    echo ""
    echo "If only api_name is provided, the script will look for an existing entry in the registry."
    echo "If the entry exists, it will use those settings. Otherwise, you must provide all parameters."
    echo ""
    echo "To create a new API with custom options, first add a new entry to config/api_registry.json"
    echo "based on the template, then run this script with just the API name."
    echo ""
    echo "Example:"
    echo "  $0 petstore                                                  # Use existing registry entry"
    echo "  $0 petstore https://petstore3.swagger.io/api/v3 config/swagger/petstore.yaml pet id  # Create new entry"
    exit 1
fi

API_NAME="$1"
API_REGISTRY="config/api_registry.json"
SWAGGER_DIR="config/swagger"

# Check if API registry exists
if [ ! -f "$API_REGISTRY" ]; then
    echo "Error: API registry file not found at $API_REGISTRY"
    exit 1
fi

# Check if API exists in registry
if jq -e ".apis[] | select(.name == \"$API_NAME\")" "$API_REGISTRY" > /dev/null; then
    echo "Found existing API '$API_NAME' in registry. Using registry configuration."
    
    # Extract API details from registry
    API_URL=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .url" "$API_REGISTRY")
    SCHEMA_PATH=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .schema_path" "$API_REGISTRY")
    ENTITY_NAME=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .entity_name" "$API_REGISTRY")
    ID_FIELD=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .id_field" "$API_REGISTRY")
    
    # Extract options
    GENERATE_MODELS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.generate_models // true" "$API_REGISTRY")
    GENERATE_API=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.generate_api // true" "$API_REGISTRY")
    GENERATE_HANDLERS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.generate_handlers // true" "$API_REGISTRY")
    UPDATE_ROUTER=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.update_router // true" "$API_REGISTRY")
    INCLUDE_MODELS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.include_models | join(\",\")" "$API_REGISTRY")
    EXCLUDE_MODELS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.exclude_models | join(\",\")" "$API_REGISTRY")
else
    # New API, check if all required parameters are provided
    if [ "$#" -lt 4 ]; then
        echo "Error: API '$API_NAME' not found in registry. You must provide all parameters for a new API."
        echo "Usage: $0 <api_name> <api_url> <schema_path> <entity_name> [<id_field>]"
        exit 1
    fi
    
    API_URL="$2"
    SCHEMA_PATH="$3"
    ENTITY_NAME="$4"
    ID_FIELD="${5:-id}"
    
    # Default options for new APIs
    GENERATE_MODELS=true
    GENERATE_API=true
    GENERATE_HANDLERS=true
    UPDATE_ROUTER=true
    INCLUDE_MODELS=""
    EXCLUDE_MODELS=""
    
    echo "Creating new API '$API_NAME' with default options."
    echo "To customize options, edit the config/api_registry.json file after creation."
fi

# Early exit if nothing is being generated
if [ "$GENERATE_MODELS" != "true" ] && [ "$GENERATE_API" != "true" ] && [ "$GENERATE_HANDLERS" != "true" ]; then
    echo "⚠️ All generation options are set to false. No code will be generated."
    echo "Updating API registry only..."
    
    # Prepare options for the registry
    OPTIONS_JSON=$(jq -n \
        --arg models "$GENERATE_MODELS" \
        --arg api "$GENERATE_API" \
        --arg handlers "$GENERATE_HANDLERS" \
        --arg router "$UPDATE_ROUTER" \
        --arg include "$INCLUDE_MODELS" \
        --arg exclude "$EXCLUDE_MODELS" \
        '{
            generate_models: ($models == "true"),
            generate_api: ($api == "true"),
            generate_handlers: ($handlers == "true"),
            update_router: ($router == "true"),
            include_models: (if $include == "" then [] else $include | split(",") end),
            exclude_models: (if $exclude == "" then [] else $exclude | split(",") end)
        }')
    
    # Update the API registry
    if jq -e ".apis[] | select(.name == \"$API_NAME\")" "$API_REGISTRY" > /dev/null; then
        jq --arg name "$API_NAME" \
           --arg url "$API_URL" \
           --arg schema "${SWAGGER_DIR}/${API_NAME}.yaml" \
           --arg entity "$ENTITY_NAME" \
           --arg id "$ID_FIELD" \
           --argjson options "$OPTIONS_JSON" \
           '.apis = [.apis[] | if .name == $name then {
               name: $name,
               url: $url,
               schema_path: $schema,
               entity_name: $entity,
               id_field: $id,
               options: $options
           } else . end]' \
           "$API_REGISTRY" > "${API_REGISTRY}.new"
        mv "${API_REGISTRY}.new" "$API_REGISTRY"
    else
        jq --arg name "$API_NAME" \
           --arg url "$API_URL" \
           --arg schema "${SWAGGER_DIR}/${API_NAME}.yaml" \
           --arg entity "$ENTITY_NAME" \
           --arg id "$ID_FIELD" \
           --argjson options "$OPTIONS_JSON" \
           '.apis += [{
               name: $name,
               url: $url,
               schema_path: $schema,
               entity_name: $entity,
               id_field: $id,
               options: $options
           }]' \
           "$API_REGISTRY" > "${API_REGISTRY}.new"
        mv "${API_REGISTRY}.new" "$API_REGISTRY"
    fi
    
    # Update generated_apis.rs to include only active APIs
    update_generated_apis
    
    echo "✅ Successfully updated API registry for ${API_NAME}"
    exit 0
fi

# Function to update generated_apis.rs with only active APIs
update_generated_apis() {
    # Get all active APIs from the registry (those with at least one generation option enabled)
    ACTIVE_APIS=$(jq -r '.apis[] | select(.options.generate_models == true or .options.generate_api == true or .options.generate_handlers == true) | .name' "$API_REGISTRY")
    
    # Update the bridge file to include all active APIs
    echo "Updating generated_apis.rs..."
    cat > "src/generated_apis.rs" << EOF
//! Generated API modules
//!
//! This file serves as a bridge to the generated API code in the /generated directory.
//! It uses the #[path] attribute to reference files outside of the src directory.

EOF

    # Add imports and re-exports for all active APIs
    for ACTIVE_API in $ACTIVE_APIS; do
        # Get entity name for this API
        API_ENTITY=$(jq -r ".apis[] | select(.name == \"$ACTIVE_API\") | .entity_name" "$API_REGISTRY")
        API_ENTITY_CAMEL=$(echo "$API_ENTITY" | sed -r 's/(^|_)([a-z])/\U\2/g')
        
        # Add import for this API
        echo "#[path = \"../generated/${ACTIVE_API}_api/src/lib.rs\"]" >> "src/generated_apis.rs"
        echo "pub mod ${ACTIVE_API}_api;" >> "src/generated_apis.rs"
        echo "" >> "src/generated_apis.rs"
        
        # Add re-exports if models are generated
        if jq -e ".apis[] | select(.name == \"$ACTIVE_API\") | .options.generate_models" "$API_REGISTRY" > /dev/null; then
            echo "// Re-export from ${ACTIVE_API}_api" >> "src/generated_apis.rs"
            echo "pub use ${ACTIVE_API}_api::${API_ENTITY_CAMEL};" >> "src/generated_apis.rs"
            echo "" >> "src/generated_apis.rs"
        fi
    done
}

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
mkdir -p ${SWAGGER_DIR}

# Download schema if it's a URL
if [[ "$SCHEMA_PATH" == http* ]]; then
    echo "Downloading schema from $SCHEMA_PATH..."
    SCHEMA_FILE="${SWAGGER_DIR}/${API_NAME}.yaml"
    curl -s -o "$SCHEMA_FILE" "$SCHEMA_PATH"
    SCHEMA_PATH="$SCHEMA_FILE"
elif [[ ! -f "$SCHEMA_PATH" ]]; then
    echo "Error: Schema file not found at $SCHEMA_PATH"
    exit 1
else
    # Copy the schema to our swagger folder for consistency
    echo "Copying schema from $SCHEMA_PATH..."
    mkdir -p "${SWAGGER_DIR}"
    cp "$SCHEMA_PATH" "${SWAGGER_DIR}/${API_NAME}.yaml"
    SCHEMA_PATH="${SWAGGER_DIR}/${API_NAME}.yaml"
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

# Add model filtering if specified
if [ -n "$INCLUDE_MODELS" ]; then
    echo "  # Only generate the specified models (comma-separated list)" >> "$CONFIG_FILE"
    echo "  modelsDtsModels: $INCLUDE_MODELS" >> "$CONFIG_FILE"
    echo "⚠️ Note: Only generating the following models: $INCLUDE_MODELS"
fi

# Check for OpenAPI Generator
if ! command -v openapi-generator &> /dev/null; then
    echo "Error: OpenAPI Generator is not installed."
    echo "Please install it from: https://openapi-generator.tech/docs/installation/"
    exit 1
fi

# Run OpenAPI Generator
echo "Running OpenAPI Generator..."
openapi-generator generate -i "$SCHEMA_PATH" -c "$CONFIG_FILE"

if [ $? -ne 0 ]; then
    echo "Error: OpenAPI Generator failed."
    exit 1
fi

echo "API client generated successfully in generated/${API_NAME}_api/"

# Fix structure issues - create the API and models modules
if [ "$GENERATE_API" = true ] && [ ! -f "generated/${API_NAME}_api/src/api/mod.rs" ]; then
    echo "Creating API module..."
    mkdir -p "generated/${API_NAME}_api/src/api"
    echo "// API module - stub for compatibility" > "generated/${API_NAME}_api/src/api/mod.rs"
fi

if [ "$GENERATE_MODELS" = true ] && [ ! -f "generated/${API_NAME}_api/src/models/mod.rs" ]; then
    echo "Creating models module..."
    mkdir -p "generated/${API_NAME}_api/src/models"
    
    # Create a simple model for demonstration
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

# Create basic handlers file if needed
if [ "$GENERATE_HANDLERS" = true ]; then
    echo "Creating basic handlers file..."
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

// Basic handlers for the ${API_NAME} API can be added here
EOF
fi

# Create module file
echo "Creating module file..."
cat > "generated/${API_NAME}_api/src/lib.rs" << EOF
//! ${API_NAME_CAMEL} API Client
//!
//! This is an auto-generated API client for the ${API_NAME} API.

EOF

# Add module declarations based on what was generated
if [ "$GENERATE_API" = true ]; then
    echo "pub mod api;" >> "generated/${API_NAME}_api/src/lib.rs"
fi

if [ "$GENERATE_MODELS" = true ]; then
    echo "pub mod models;" >> "generated/${API_NAME}_api/src/lib.rs"
fi

if [ "$GENERATE_HANDLERS" = true ]; then
    echo "pub mod handlers;" >> "generated/${API_NAME}_api/src/lib.rs"
fi

# Add re-exports
if [ "$GENERATE_MODELS" = true ]; then
    echo -e "\n// Re-export commonly used types" >> "generated/${API_NAME}_api/src/lib.rs"
    echo "pub use models::${ENTITY_NAME_CAMEL};" >> "generated/${API_NAME}_api/src/lib.rs"
fi

# Update generated_apis.rs with only active APIs
update_generated_apis

# Update the config file to include the API URL if it doesn't already exist
echo "Updating configuration..."
CONFIG_FILE="config/default.yaml"

# Check if the API URL already exists in the config file
if ! grep -q "${API_NAME_SNAKE}_url:" "$CONFIG_FILE"; then
    echo "Adding ${API_NAME_SNAKE}_url to config file..."
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
else
    echo "${API_NAME_SNAKE}_url already exists in config file, skipping update."
fi

# Prepare options for the registry
OPTIONS_JSON=$(jq -n \
    --arg models "$GENERATE_MODELS" \
    --arg api "$GENERATE_API" \
    --arg handlers "$GENERATE_HANDLERS" \
    --arg router "$UPDATE_ROUTER" \
    --arg include "$INCLUDE_MODELS" \
    --arg exclude "$EXCLUDE_MODELS" \
    '{
        generate_models: ($models == "true"),
        generate_api: ($api == "true"),
        generate_handlers: ($handlers == "true"),
        update_router: ($router == "true"),
        include_models: (if $include == "" then [] else $include | split(",") end),
        exclude_models: (if $exclude == "" then [] else $exclude | split(",") end)
    }')

# Update the API registry
echo "Updating API registry..."
if jq -e ".apis[] | select(.name == \"$API_NAME\")" "$API_REGISTRY" > /dev/null; then
    echo "API $API_NAME already exists in registry. Updating..."
    # Update the existing entry
    jq --arg name "$API_NAME" \
       --arg url "$API_URL" \
       --arg schema "${SWAGGER_DIR}/${API_NAME}.yaml" \
       --arg entity "$ENTITY_NAME" \
       --arg id "$ID_FIELD" \
       --argjson options "$OPTIONS_JSON" \
       '.apis = [.apis[] | if .name == $name then {
           name: $name,
           url: $url,
           schema_path: $schema,
           entity_name: $entity,
           id_field: $id,
           options: $options
       } else . end]' \
       "$API_REGISTRY" > "${API_REGISTRY}.new"
    mv "${API_REGISTRY}.new" "$API_REGISTRY"
else
    # Add a new entry
    jq --arg name "$API_NAME" \
       --arg url "$API_URL" \
       --arg schema "${SWAGGER_DIR}/${API_NAME}.yaml" \
       --arg entity "$ENTITY_NAME" \
       --arg id "$ID_FIELD" \
       --argjson options "$OPTIONS_JSON" \
       '.apis += [{
           name: $name,
           url: $url,
           schema_path: $schema,
           entity_name: $entity,
           id_field: $id,
           options: $options
       }]' \
       "$API_REGISTRY" > "${API_REGISTRY}.new"
    mv "${API_REGISTRY}.new" "$API_REGISTRY"
fi

echo "✅ Successfully added/updated ${API_NAME} API!"
echo "📁 Generated code in: generated/${API_NAME}_api/"
echo "📄 OpenAPI schema stored in: ${SWAGGER_DIR}/${API_NAME}.yaml"

# Print generation options summary
echo -e "\n📋 Generation options:"
echo "   Generate models: $GENERATE_MODELS"
echo "   Generate API client: $GENERATE_API"
echo "   Generate handlers: $GENERATE_HANDLERS"
echo "   Update router: $UPDATE_ROUTER"
if [ -n "$INCLUDE_MODELS" ]; then
    echo "   Include models (only these will be generated): $INCLUDE_MODELS"
fi
if [ -n "$EXCLUDE_MODELS" ]; then
    echo "   Exclude models: $EXCLUDE_MODELS"
fi

echo -e "\nTo modify these options, edit the config/api_registry.json file and run this script again." 
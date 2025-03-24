#!/bin/bash

# validate_api_models.sh
#
# This script compares generated API models with the stored versions in the repository.
# It detects changes in the API schemas and provides options to update the stored models.

set -e  # Exit immediately if a command exits with a non-zero status

# Constants
API_REGISTRY="config/api_registry.json"
GENERATED_DIR="target/generated"
TEMP_GENERATED_DIR="target/temp_generated"
MODELS_DIR="src/models/apis"
VERBOSE=false
AUTO_UPDATE=false
STRICT_MODE=false

# Colors for output
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -v|--verbose) VERBOSE=true ;;
        -a|--auto-update) AUTO_UPDATE=true ;;
        -s|--strict) STRICT_MODE=true ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Function to log verbose messages
log() {
    if [ "$VERBOSE" = true ]; then
        echo "$@"
    fi
}

# Function to generate models in a temp directory
generate_temp_models() {
    local api_name=$1
    log "Generating temporary models for $api_name..."
    
    # Create temp directory
    mkdir -p "$TEMP_GENERATED_DIR"
    
    # Extract API details from registry
    local schema_path=$(jq -r ".apis[] | select(.name == \"$api_name\") | .schema_path" "$API_REGISTRY")
    local api_url=$(jq -r ".apis[] | select(.name == \"$api_name\") | .url" "$API_REGISTRY")
    local entity_name=$(jq -r ".apis[] | select(.name == \"$api_name\") | .entity_name" "$API_REGISTRY")
    local include_models=$(jq -r ".apis[] | select(.name == \"$api_name\") | .options.include_models | join(\",\")" "$API_REGISTRY")
    
    if [ ! -f "$schema_path" ]; then
        echo -e "${RED}Error: Schema file not found at $schema_path${NC}"
        return 1
    fi
    
    # Create OpenAPI Generator config file
    local config_file="${TEMP_GENERATED_DIR}/${api_name}_config.yaml"
    
    cat > "$config_file" << EOF
# OpenAPI Generator configuration for $api_name (validation)
generatorName: rust
outputDir: ${TEMP_GENERATED_DIR}/${api_name}_api
packageName: ${api_name}_api
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
    if [ -n "$include_models" ]; then
        echo "  # Only generate the specified models (comma-separated list)" >> "$config_file"
        echo "  modelsDtsModels: $include_models" >> "$config_file"
    fi
    
    # Run OpenAPI Generator
    openapi-generator generate -i "$schema_path" -c "$config_file" > /dev/null 2>&1
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to generate temporary models for $api_name${NC}"
        return 1
    fi
    
    log "Temporary models generated successfully for $api_name"
    return 0
}

# Function to compare a generated model with the stored version
compare_model() {
    local api_name=$1
    local model_name=$2
    
    # Get the capitalized model name (OpenAPI generator uses U prefix)
    local u_model_name="U${model_name^}"
    
    # Paths to the model files
    local generated_model="${TEMP_GENERATED_DIR}/${api_name}_api/src/models/mod.rs"
    local stored_model="${MODELS_DIR}/${api_name}/${model_name,,}.rs"
    
    if [ ! -f "$generated_model" ]; then
        echo -e "${RED}Error: Generated model file not found at $generated_model${NC}"
        return 1
    fi
    
    if [ ! -f "$stored_model" ]; then
        echo -e "${YELLOW}Warning: Stored model file not found at $stored_model, assuming new model${NC}"
        mkdir -p "${MODELS_DIR}/${api_name}"
        return 2  # Return code 2 means "new model"
    fi
    
    # Extract model structs for comparison (removing comments and formatting differences)
    local generated_struct=$(grep -A 50 "struct $u_model_name" "$generated_model" | sed -n '/struct/,/}/p' | sed 's/\/\/.*//g' | tr -d ' \t\n')
    local stored_struct=$(grep -A 50 "struct ${model_name^}" "$stored_model" | sed -n '/struct/,/}/p' | sed 's/\/\/.*//g' | tr -d ' \t\n')
    
    # Normalize struct names for comparison
    local normalized_generated=${generated_struct//$u_model_name/${model_name^}}
    
    # Compare normalized structs
    if [ "$normalized_generated" != "$stored_struct" ]; then
        log "Differences found in model $model_name"
        return 3  # Return code 3 means "differences found"
    fi
    
    log "No differences found in model $model_name"
    return 0  # Return code 0 means "no differences"
}

# Function to display differences between models
display_diff() {
    local api_name=$1
    local model_name=$2
    
    # Get the capitalized model name (OpenAPI generator uses U prefix)
    local u_model_name="U${model_name^}"
    
    # Paths to the model files
    local generated_model="${TEMP_GENERATED_DIR}/${api_name}_api/src/models/mod.rs"
    local stored_model="${MODELS_DIR}/${api_name}/${model_name,,}.rs"
    
    echo -e "${BLUE}Differences in model ${model_name^}:${NC}"
    
    # Extract the struct definitions
    local generated_struct=$(grep -A 50 "struct $u_model_name" "$generated_model" | sed -n '/struct/,/}/p' | sed "s/$u_model_name/${model_name^}/g")
    local stored_struct=$(grep -A 50 "struct ${model_name^}" "$stored_model" | sed -n '/struct/,/}/p')
    
    # Create temporary files for diff
    local temp_generated=$(mktemp)
    local temp_stored=$(mktemp)
    
    echo "$generated_struct" > "$temp_generated"
    echo "$stored_struct" > "$temp_stored"
    
    # Show diff
    echo -e "${YELLOW}--- Stored model${NC}"
    echo -e "${GREEN}+++ Generated model${NC}"
    diff --color=always -u "$temp_stored" "$temp_generated" || true
    
    # Clean up temp files
    rm "$temp_generated" "$temp_stored"
}

# Function to update a stored model from the generated version
update_model() {
    local api_name=$1
    local model_name=$2
    
    # Get the capitalized model name (OpenAPI generator uses U prefix)
    local u_model_name="U${model_name^}"
    
    # Paths to the model files
    local generated_model="${TEMP_GENERATED_DIR}/${api_name}_api/src/models/mod.rs"
    local stored_model="${MODELS_DIR}/${api_name}/${model_name,,}.rs"
    
    log "Updating $model_name model..."
    
    # Determine if this is a new model
    local is_new=false
    if [ ! -f "$stored_model" ]; then
        is_new=true
        mkdir -p "${MODELS_DIR}/${api_name}"
        
        # Create the initial file with headers
        cat > "$stored_model" << EOF
use serde::{Deserialize, Serialize};

/// ${model_name^} model from the ${api_name^} API
///
/// This model is automatically validated against the OpenAPI specification.
EOF
    else
        # Preserve the header from the original file
        local header=$(sed -n '1,/struct/p' "$stored_model" | sed '$d')
        echo "$header" > "${stored_model}.new"
    fi
    
    # Extract the struct from the generated model and fix the name
    local generated_struct=$(grep -A 50 "struct $u_model_name" "$generated_model" | 
        sed -n '/struct/,/}/p' | 
        sed "s/$u_model_name/${model_name^}/g")
    
    if [ "$is_new" = true ]; then
        # For new models, add the full struct with an implementation
        echo "$generated_struct" >> "$stored_model"
        
        # Add a simple constructor
        cat >> "$stored_model" << EOF

impl ${model_name^} {
    /// Create a new ${model_name} instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ${model_name^} {
    fn default() -> Self {
        Self {
            // Initialize with default values
EOF
        
        # Extract fields for the default implementation
        local fields=$(echo "$generated_struct" | grep -o "pub [^:]*:" | sed 's/pub //g' | sed 's/://g')
        for field in $fields; do
            echo "            $field: Default::default()," >> "$stored_model"
        done
        
        echo "        }" >> "$stored_model"
        echo "    }" >> "$stored_model"
        echo "}" >> "$stored_model"
    else
        # For existing models, update the struct definition but preserve implementation
        echo "$generated_struct" >> "${stored_model}.new"
        
        # Extract implementations from original file
        sed -n '/impl/,$p' "$stored_model" >> "${stored_model}.new"
        
        # Replace the original file
        mv "${stored_model}.new" "$stored_model"
    fi
    
    # If it's a new model, also update the mod.rs file
    if [ "$is_new" = true ]; then
        # First check if the model is already in mod.rs
        if ! grep -q "mod ${model_name,,};" "${MODELS_DIR}/${api_name}/mod.rs"; then
            # Add module declaration
            echo "mod ${model_name,,};" >> "${MODELS_DIR}/${api_name}/mod.rs"
            # Add re-export
            echo "pub use ${model_name,,}::${model_name^};" >> "${MODELS_DIR}/${api_name}/mod.rs"
        fi
    fi
    
    echo -e "${GREEN}Updated model ${model_name^}${NC}"
}

# Main execution
echo "Validating API models against OpenAPI specifications..."

# Check if API registry exists
if [ ! -f "$API_REGISTRY" ]; then
    echo -e "${RED}Error: API registry file not found at $API_REGISTRY${NC}"
    exit 1
fi

# Get the list of APIs from the registry
api_count=$(jq '.apis | length' "$API_REGISTRY")

if [ "$api_count" -eq 0 ]; then
    echo "No APIs registered in $API_REGISTRY. Nothing to validate."
    exit 0
fi

# Validate each API
for i in $(seq 0 $(($api_count - 1))); do
    api_name=$(jq -r ".apis[$i].name" "$API_REGISTRY")
    
    echo "Validating models for $api_name API..."
    
    # Generate temporary models
    generate_temp_models "$api_name"
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to generate temporary models for $api_name. Skipping validation.${NC}"
        continue
    fi
    
    # Get the list of models to validate
    include_models=$(jq -r ".apis[$i].options.include_models[]" "$API_REGISTRY" 2>/dev/null || echo "")
    
    if [ -z "$include_models" ]; then
        # If no specific models are included, assume all models
        echo "No specific models defined, using entity name..."
        entity_name=$(jq -r ".apis[$i].entity_name" "$API_REGISTRY")
        include_models="${entity_name^}"
    fi
    
    # Track if any differences were found
    diff_found=false
    
    # Compare each model
    for model in $include_models; do
        echo "Checking model $model..."
        compare_model "$api_name" "$model"
        result=$?
        
        case $result in
            0)  # No differences
                echo -e "${GREEN}✓ Model $model is up to date${NC}"
                ;;
            1)  # Error
                echo -e "${RED}✗ Error checking model $model${NC}"
                if [ "$STRICT_MODE" = true ]; then
                    exit 1
                fi
                ;;
            2)  # New model
                echo -e "${YELLOW}⚠ Model $model is new and needs to be created${NC}"
                diff_found=true
                
                if [ "$AUTO_UPDATE" = true ]; then
                    update_model "$api_name" "$model"
                else
                    echo -e "  Run with -a to automatically create this model"
                fi
                ;;
            3)  # Differences found
                echo -e "${YELLOW}⚠ Model $model has changed${NC}"
                diff_found=true
                
                # Display differences
                display_diff "$api_name" "$model"
                
                if [ "$AUTO_UPDATE" = true ]; then
                    update_model "$api_name" "$model"
                else
                    echo -e "  Run with -a to automatically update this model"
                fi
                ;;
        esac
    done
    
    # Clean up temporary files
    rm -rf "${TEMP_GENERATED_DIR}/${api_name}_api"
    
    # Exit with an error if differences were found and in strict mode
    if [ "$diff_found" = true ] && [ "$STRICT_MODE" = true ]; then
        echo -e "${RED}Differences found and strict mode enabled. Exiting with error.${NC}"
        exit 1
    fi
done

# Clean up
rm -rf "$TEMP_GENERATED_DIR"

echo "API model validation complete."

if [ "$diff_found" = true ]; then
    echo -e "${YELLOW}⚠ Differences were found in some models.${NC}"
    echo "Run with -a to automatically update all models."
    exit 1
else
    echo -e "${GREEN}✓ All API models are up to date.${NC}"
    exit 0
fi 
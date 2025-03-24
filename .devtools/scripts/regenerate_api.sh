#!/bin/bash

# regenerate_api.sh - Tool for regenerating API clients from the registry
#
# This script automates the process of regenerating API clients:
# 1. Reads the API registry to get configuration
# 2. Regenerates specified API clients or all of them
# 3. Uses the configuration options from the registry

set -e  # Exit immediately if a command exits with a non-zero status

API_REGISTRY="config/api_registry.json"
GENERATED_DIR="target/generated"
SCHEMA_HASH_DIR="$GENERATED_DIR/.schema_hashes"

# Create schema hash directory if it doesn't exist
mkdir -p "$SCHEMA_HASH_DIR"

# Function to display usage information
show_usage() {
    echo "Usage: $0 [options] [api_name1 api_name2 ...]"
    echo ""
    echo "Options:"
    echo "  --help           - Show this help message"
    echo "  --list           - List all registered APIs"
    echo "  --force          - Force regeneration even if schema hasn't changed"
    echo ""
    echo "If no API names are provided, all APIs will be regenerated."
    echo ""
    echo "Examples:"
    echo "  $0 --list                  # List all registered APIs"
    echo "  $0 petstore                # Regenerate only the petstore API"
    echo "  $0                         # Regenerate all APIs"
    echo ""
    echo "Note: All configuration options are read from config/api_registry.json."
    echo "Generated code is stored in ${GENERATED_DIR}/"
}

# Function to list all registered APIs
list_apis() {
    if [ ! -f "$API_REGISTRY" ]; then
        echo "Error: API registry file not found at $API_REGISTRY"
        exit 1
    fi

    echo "Registered APIs:"
    echo "----------------"
    jq -r '.apis[] | "- \(.name): \(.url) [\(.schema_path)]"' "$API_REGISTRY"
    echo ""
    echo "Generation options:"
    echo "------------------"
    
    # Display each API with its options in a more readable format
    jq -r '.apis[] | "- \(.name):\n  Entity: \(.entity_name)\n  Options:\n    generate_models: \(.options.generate_models)\n    generate_api: \(.options.generate_api)\n    generate_handlers: \(.options.generate_handlers)\n    update_router: \(.options.update_router)\n    include_models: \(.options.include_models | if length > 0 then . else "[]" end) (if empty, all models are generated)\n    exclude_models: \(.options.exclude_models | if length > 0 then . else "[]" end)"' "$API_REGISTRY"
    
    echo -e "\nTemplate API (for reference):"
    echo "----------------------------"
    jq -r '.template | "Name: \(.name)\nOptions Description:\n  \(.options.options_description | to_entries[] | "  \(.key): \(.value)")"' "$API_REGISTRY"
}

# Parse command line arguments
APIS_TO_REGENERATE=()
FORCE_REGENERATE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help)
            show_usage
            exit 0
            ;;
        --list)
            list_apis
            exit 0
            ;;
        --force)
            FORCE_REGENERATE=true
            ;;
        -*)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            APIS_TO_REGENERATE+=("$1")
            ;;
    esac
    shift
done

# Check if API registry exists
if [ ! -f "$API_REGISTRY" ]; then
    echo "Error: API registry file not found at $API_REGISTRY"
    exit 1
fi

# If no APIs specified, regenerate all
if [ ${#APIS_TO_REGENERATE[@]} -eq 0 ]; then
    APIS_TO_REGENERATE=($(jq -r '.apis[].name' "$API_REGISTRY"))
    echo "Regenerating all APIs: ${APIS_TO_REGENERATE[*]}"
fi

# Process each API
for API_NAME in "${APIS_TO_REGENERATE[@]}"; do
    echo "Processing API: $API_NAME"
    
    # Check if API exists in registry
    if ! jq -e ".apis[] | select(.name == \"$API_NAME\")" "$API_REGISTRY" > /dev/null; then
        echo "Error: API '$API_NAME' not found in registry"
        continue
    fi
    
    # Get the schema path
    SCHEMA_PATH=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .schema_path" "$API_REGISTRY")
    
    # Check if schema exists
    if [ ! -f "$SCHEMA_PATH" ]; then
        echo "Warning: Schema file not found at $SCHEMA_PATH, skipping..."
        continue
    fi
    
    # Calculate hash of schema file
    SCHEMA_HASH=$(sha256sum "$SCHEMA_PATH" | cut -d ' ' -f 1)
    HASH_FILE="$SCHEMA_HASH_DIR/${API_NAME}.hash"
    
    # Check if regeneration is needed
    if [ "$FORCE_REGENERATE" = "false" ] && [ -f "$HASH_FILE" ] && [ "$(cat "$HASH_FILE")" = "$SCHEMA_HASH" ] && [ -d "$GENERATED_DIR/${API_NAME}_api" ]; then
        echo "Schema for $API_NAME hasn't changed, skipping regeneration."
        echo "✓ Using cached version from previous build."
        echo "----------------------------------------"
        continue
    fi
    
    # Display the options being used
    echo "Using the following options from registry:"
    jq -r ".apis[] | select(.name == \"$API_NAME\") | \"  generate_models: \(.options.generate_models)\n  generate_api: \(.options.generate_api)\n  generate_handlers: \(.options.generate_handlers)\n  update_router: \(.options.update_router)\"" "$API_REGISTRY"
    
    # Check if include_models is specified
    INCLUDE_MODELS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.include_models | join(\", \")" "$API_REGISTRY")
    if [ -n "$INCLUDE_MODELS" ] && [ "$INCLUDE_MODELS" != "" ]; then
        echo "  include_models: $INCLUDE_MODELS (ONLY these models will be generated)"
    else
        echo "  include_models: [] (all models will be generated)"
    fi
    
    # Check if exclude_models is specified
    EXCLUDE_MODELS=$(jq -r ".apis[] | select(.name == \"$API_NAME\") | .options.exclude_models | join(\", \")" "$API_REGISTRY")
    if [ -n "$EXCLUDE_MODELS" ] && [ "$EXCLUDE_MODELS" != "" ]; then
        echo "  exclude_models: $EXCLUDE_MODELS (these models will be excluded)"
    else
        echo "  exclude_models: [] (no models will be excluded)"
    fi
    
    # Call add_api.sh with the API name to use registry configuration
    echo "Regenerating $API_NAME using registry configuration"
    .devtools/scripts/add_api.sh "$API_NAME"
    
    # Save the schema hash for future comparison
    echo "$SCHEMA_HASH" > "$HASH_FILE"
    
    echo "✅ Successfully regenerated $API_NAME"
    echo "----------------------------------------"
done

echo "All specified APIs have been regenerated."

#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Parse command line arguments
SKIP_GEN=false
RELEASE_MODE=false
CONFIG_DIR="config"
ENV_FILE=".env"
RUN_ENV="development"
API_REGISTRY="api_registry.json"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --skip-gen           Skip API model generation"
    echo "  --release            Build and run in release mode"
    echo "  --config-dir=DIR     Use specified config directory (default: config)"
    echo "  --env=FILE           Use specified .env file (default: .env)"
    echo "  --environment=ENV    Use specified environment (default: development)"
    echo "  --help               Show this help message"
}

for arg in "$@"; do
    case $arg in
        --skip-gen)
            SKIP_GEN=true
            shift
            ;;
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --config-dir=*)
            CONFIG_DIR="${arg#*=}"
            shift
            ;;
        --env=*)
            ENV_FILE="${arg#*=}"
            shift
            ;;
        --environment=*)
            RUN_ENV="${arg#*=}"
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            print_usage
            exit 1
            ;;
    esac
done

# Header
echo "==================================================="
echo "  Petstore API Server with Utoipa Integration"
echo "==================================================="

# Check for required tools
echo "Checking dependencies..."
if ! command -v openapi-generator &> /dev/null; then
    echo "Warning: OpenAPI Generator is not installed."
    echo "This is needed for API generation. You can install it from: https://openapi-generator.tech/docs/installation/"
    echo "Continuing without API generation capabilities..."
    SKIP_GEN=true
fi

# Check if config files exist
if [ -d "$CONFIG_DIR" ]; then
    echo "Using config directory: $CONFIG_DIR"
    
    # Check for environment-specific config file
    if [ -f "$CONFIG_DIR/$RUN_ENV.yaml" ]; then
        echo "Found environment config: $CONFIG_DIR/$RUN_ENV.yaml"
    elif [ -f "$CONFIG_DIR/default.yaml" ]; then
        echo "Found default config: $CONFIG_DIR/default.yaml"
    else
        echo "Warning: No configuration files found in $CONFIG_DIR. Using defaults."
    fi
    
    # Export CONFIG_DIR for the application
    export CONFIG_DIR="$CONFIG_DIR"
    # Export RUN_ENV for the application
    export RUN_ENV="$RUN_ENV"
else
    echo "Warning: Config directory $CONFIG_DIR not found. Using defaults."
fi

if [ -f "$ENV_FILE" ]; then
    echo "Using environment file: $ENV_FILE"
    # Load environment variables
    export $(grep -v '^#' "$ENV_FILE" | xargs)
else
    echo "Warning: Environment file $ENV_FILE not found. Using defaults."
fi

# Generate API models if needed
if [ "$SKIP_GEN" = false ]; then
    echo "Checking for APIs that need generation..."
    
    # Check if API registry exists
    if [ -f "$API_REGISTRY" ]; then
        # Create generated directory if it doesn't exist
        mkdir -p generated/openapi
        
        # Read the API registry and generate missing APIs
        api_count=$(jq '.apis | length' "$API_REGISTRY")
        
        if [ "$api_count" -gt 0 ]; then
            echo "Found $api_count registered APIs."
            
            for i in $(seq 0 $(($api_count - 1))); do
                api_name=$(jq -r ".apis[$i].name" "$API_REGISTRY")
                api_url=$(jq -r ".apis[$i].url" "$API_REGISTRY")
                api_schema=$(jq -r ".apis[$i].schema_path" "$API_REGISTRY")
                entity_name=$(jq -r ".apis[$i].entity_name" "$API_REGISTRY")
                id_field=$(jq -r ".apis[$i].id_field" "$API_REGISTRY")
                
                # Check if this API is already generated
                if [ ! -d "generated/${api_name}_api" ]; then
                    echo "Generating API client for $api_name..."
                    
                    # Run the add_api.sh script
                    ./scripts/add_api.sh "$api_name" "$api_url" "$api_schema" "$entity_name" "$id_field"
                    
                    if [ $? -ne 0 ]; then
                        echo "Warning: Failed to generate API client for $api_name. Continuing..."
                    else
                        echo "Successfully generated API client for $api_name."
                    fi
                else
                    echo "API client for $api_name already exists, skipping generation."
                fi
            done
        else
            echo "No APIs registered in $API_REGISTRY."
        fi
    else
        echo "API registry file $API_REGISTRY not found. Skipping API generation."
    fi
else
    echo "Skipping API model generation (--skip-gen flag used)"
fi

# Build the project
echo "Building the project (this will run the build script to add Utoipa annotations)..."
if [ "$RELEASE_MODE" = true ]; then
    echo "Building in release mode..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Error: Release build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/release/rust-backend"
else
    cargo build
    if [ $? -ne 0 ]; then
        echo "Error: Debug build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/debug/rust-backend"
fi

echo "Build successful. Starting server..."

# Set RUST_LOG if not already set
if [ -z "$RUST_LOG" ]; then
    export RUST_LOG=info
    echo "Setting log level to info (RUST_LOG=info)"
fi

# Run the executable
echo "Starting server..."
echo "Press Ctrl+C to stop the server."
echo "---------------------------------------------------"
"$EXEC_PATH"

# This part will execute after server shutdown (Ctrl+C)
echo "Server stopped."

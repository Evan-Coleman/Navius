#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Parse command line arguments
SKIP_GEN=false
RELEASE_MODE=false
CONFIG_FILE="config.yaml"
ENV_FILE=".env"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --skip-gen          Skip API model generation"
    echo "  --release           Build and run in release mode"
    echo "  --config=FILE       Use specified config file (default: config.yaml)"
    echo "  --env=FILE          Use specified .env file (default: .env)"
    echo "  --help              Show this help message"
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
        --config=*)
            CONFIG_FILE="${arg#*=}"
            shift
            ;;
        --env=*)
            ENV_FILE="${arg#*=}"
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
    echo "Error: OpenAPI Generator CLI is not installed."
    echo "Please install it with: npm install @openapitools/openapi-generator-cli -g"
    exit 1
fi

# Check if config files exist
if [ -f "$CONFIG_FILE" ]; then
    echo "Using config file: $CONFIG_FILE"
else
    echo "Warning: Config file $CONFIG_FILE not found. Using defaults."
fi

if [ -f "$ENV_FILE" ]; then
    echo "Using environment file: $ENV_FILE"
    # Load environment variables
    export $(grep -v '^#' "$ENV_FILE" | xargs)
else
    echo "Warning: Environment file $ENV_FILE not found. Using defaults."
fi

# Generate downstream API files
if [ "$SKIP_GEN" = true ]; then
    echo "Skipping API model generation (--skip-gen flag used)"
    export SKIP_API_GEN=1
else
    echo "Generating API models from OpenAPI specification..."
    ./src/openapi/generate-api.sh
    if [ $? -ne 0 ]; then
        echo "Error: Failed to generate API files. See errors above."
        exit 1
    fi
    echo "API models generated successfully."
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

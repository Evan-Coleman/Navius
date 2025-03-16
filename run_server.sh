#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

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

# Generate downstream API files
echo "Generating API models from OpenAPI specification..."
./src/openapi/generate-api.sh
if [ $? -ne 0 ]; then
    echo "Error: Failed to generate API files. See errors above."
    exit 1
fi

echo "API models generated successfully."

# Build the project
echo "Building the project (this will run the build script to add Utoipa annotations)..."
cargo build
if [ $? -ne 0 ]; then
    echo "Error: Build failed. See errors above."
    exit 1
fi

echo "Build successful. Starting server..."

# Run the project
echo "Starting server on http://localhost:3000"
echo "API documentation available at http://localhost:3000/docs"
echo "Press Ctrl+C to stop the server."
echo "---------------------------------------------------"
cargo run

# This part will execute after server shutdown (Ctrl+C)
echo "Server stopped."

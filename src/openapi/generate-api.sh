#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Check if API generation should be skipped
if [ -n "$SKIP_API_GEN" ]; then
    echo "Skipping API generation (SKIP_API_GEN is set)"
    exit 0
fi

# Set the path to your OpenAPI specification file
OPENAPI_SPEC_PATH="./src/openapi/petstore-swagger.yaml"  # Adjust to your OpenAPI spec path

# Set the path to your YAML configuration file
CONFIG_PATH="./src/openapi/petstore-config.yaml"  # Adjust to your YAML config file path

OUTPUT_DIR="./src/petstore_api"

# Clear previous generated files
echo "Cleaning up previous generated files..."
rm -rf $OUTPUT_DIR

# Run OpenAPI Generator using the YAML configuration file
echo "Running OpenAPI Generator..."
openapi-generator generate \
    -i $OPENAPI_SPEC_PATH \
    -c $CONFIG_PATH \
    --openapi-generator-ignore-list "README.md,/docs/*,src/apis/*,.travis.yml,git_push.sh,.gitignore"

# Clean up unwanted files and reorganize directory structure
echo "Cleaning up and organizing generated files..."
rm -rf $OUTPUT_DIR/.openapi-generator
rm -rf $OUTPUT_DIR/.openapi-generator-ignore

# Fix directory structure - move models directory
if [ -d "$OUTPUT_DIR/src/models" ]; then
    echo "Restructuring API directory..."
    mv $OUTPUT_DIR/src/models $OUTPUT_DIR/models
    rm -rf $OUTPUT_DIR/src
fi

# Create a mod.rs file for the API
echo "Creating module declaration file..."
cat > $OUTPUT_DIR/mod.rs << 'EOF'
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
pub mod models;
EOF

# Fix imports in all Rust files - OS agnostic approach
echo "Updating import paths in generated files..."
find "$OUTPUT_DIR" -type f -name "*.rs" | while read -r file; do
    # Check if the file contains 'use crate::models;'
    if grep -q "use crate::models;" "$file"; then
        # Create a temporary file for the replacement to work on any OS
        echo "Updating imports in $file"
        temp_file=$(mktemp)
        sed 's|use crate::models;|use crate::petstore_api::models;|g' "$file" > "$temp_file"
        mv "$temp_file" "$file"
    fi
done

echo "API generation complete."
echo "The build script will now add Utoipa annotations to the models."
#!/bin/bash


# Set the path to your OpenAPI specification file
OPENAPI_SPEC_PATH="./src/openapi/petstore-swagger.yaml"  # Adjust to your OpenAPI spec path

# Set the path to your YAML configuration file
CONFIG_PATH="./src/openapi/petstore-config.yaml"  # Adjust to your YAML config file path


OUTPUT_DIR="./src/petstore_api"
IGNORE_FILE_PATH="./.openapi-generator-ignore"

# Move the .openapi-generator-ignore file to the output directory before running OpenAPI Generator
# cp $IGNORE_FILE_PATH $OUTPUT_DIR/.openapi-generator-ignore

# Run OpenAPI Generator using the YAML configuration file
# openapi-generator generate --ignore-file-override $IGNORE_FILE_PATH -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH

rm -rf $OUTPUT_DIR
openapi-generator generate -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH --openapi-generator-ignore-list "README.md,/docs/*,src/apis/*,.travis.yml,git_push.sh,.gitignore"


rm -rf $OUTPUT_DIR/.openapi-generator
rm -rf $OUTPUT_DIR/.openapi-generator-ignore
rm -rf $OUTPUT_DIRCargo/Cargo.toml
mv $OUTPUT_DIR/src/models $OUTPUT_DIR/models
rm -rf $OUTPUT_DIR/src

touch $OUTPUT_DIR/mod.rs
echo "#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
pub mod models;" > $OUTPUT_DIR/mod.rs



# Find all Rust files (.rs) in the directory and subdirectories
find "$OUTPUT_DIR" -type f -name "*.rs" | while read -r file; do
    # Check if the file contains 'use crate::models;'
    if grep -q "use crate::models;" "$file"; then
        # Replace 'use crate::models;' with 'use crate::petstore_api::models;'
        sed -i '' 's|use crate::models;|use crate::petstore_api::models;|g' "$file"
        echo "Updated $file"
    fi
done
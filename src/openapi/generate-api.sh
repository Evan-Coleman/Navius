#!/bin/bash


# Set the path to your OpenAPI specification file
OPENAPI_SPEC_PATH="./src/openapi/petstore-swagger.yaml"  # Adjust to your OpenAPI spec path

# Set the path to your YAML configuration file
CONFIG_PATH="./src/openapi/petstore-config.yaml"  # Adjust to your YAML config file path


OUTPUT_DIR="./src/petstore-api"
IGNORE_FILE_PATH="./.openapi-generator-ignore"

# Move the .openapi-generator-ignore file to the output directory before running OpenAPI Generator
# cp $IGNORE_FILE_PATH $OUTPUT_DIR/.openapi-generator-ignore

# Run OpenAPI Generator using the YAML configuration file
# openapi-generator generate --ignore-file-override $IGNORE_FILE_PATH -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH

rm -rf /src/petstore-api/
openapi-generator generate -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH --openapi-generator-ignore-list "README.md,/docs/*,src/apis/*,.travis.yml,git_push.sh,.gitignore"


rm -rf src/petstore-api/.openapi-generator
rm -rf src/petstore-api/.openapi-generator-ignore
rm -rf src/petstore-api/Cargo.toml
mv src/petstore-api/src/models src/petstore-api/models
rm -rf src/petstore-api/src

touch ./src/petstore-api/mod.rs
echo "#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
pub mod models;" > ./src/petstore-api/mod.rs
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
openapi-generator generate -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH

#!/bin/bash


# Set the path to your OpenAPI specification file
OPENAPI_SPEC_PATH="./src/openapi/petstore-swagger.yaml"  # Adjust to your OpenAPI spec path

# Set the path to your YAML configuration file
CONFIG_PATH="./src/openapi/petstore-config.yaml"  # Adjust to your YAML config file path

# Run OpenAPI Generator using the YAML configuration file
openapi-generator generate -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH

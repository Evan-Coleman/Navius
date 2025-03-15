#!/bin/bash

# Set the path to the OpenAPI Generator CLI tool
OPENAPI_GEN_PATH="/path/to/openapi-generator-cli.jar"  # Adjust to your OpenAPI Generator JAR location

# Set the path to your OpenAPI specification file
OPENAPI_SPEC_PATH="/path/to/openapi-spec.yaml"  # Adjust to your OpenAPI spec path

# Set the path to your YAML configuration file
CONFIG_PATH="/path/to/openapi-generator-config.yaml"  # Adjust to your YAML config file path

# Run OpenAPI Generator using the YAML configuration file
openapi-generator generate -i $OPENAPI_SPEC_PATH -c $CONFIG_PATH

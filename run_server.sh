#!/bin/bash

# Generate downstream API files
./src/openapi/generate-api.sh

# Build the project
cargo build

# Run the project
cargo run

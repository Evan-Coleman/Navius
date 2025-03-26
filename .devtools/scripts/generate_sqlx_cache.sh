#!/bin/bash

# Script to generate SQLx query cache for offline development

echo "Generating SQLx query cache for offline development..."

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "sqlx-cli not found, installing..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Ensure DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    if [ -f .env ]; then
        echo "Loading DATABASE_URL from .env file..."
        export $(grep DATABASE_URL .env | xargs)
    else
        echo "Error: DATABASE_URL not set and .env file not found."
        echo "Please set DATABASE_URL or create a .env file."
        exit 1
    fi
fi

# Clean up any existing .sqlx directory to ensure fresh cache
if [ -d ".sqlx" ]; then
    echo "Cleaning existing SQLx query cache..."
    rm -rf .sqlx
fi

# Generate query cache
echo "Generating SQLx query cache..."
cargo sqlx prepare -- --all-targets --all-features

# Verify the cache was created
if [ -d ".sqlx" ]; then
    echo "SQLx query cache generation complete!"
    echo "You can now build and test the project with SQLX_OFFLINE=true"
    echo "Query cache stored at: $(pwd)/.sqlx"
else
    echo "Error: Failed to generate SQLx query cache"
    exit 1
fi 
#!/bin/bash

# Wrapper script to run the appropriate environment script

# Default to development mode
ENV="development"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --dev                 Run in development mode (default)"
    echo "  --prod                Run in production mode"
    echo "  --help                Show this help message"
    echo ""
    echo "All other options are passed to the appropriate script."
}

# Process environment option first
for arg in "$@"; do
    case $arg in
        --dev)
            ENV="development"
            shift
            break
            ;;
        --prod)
            ENV="production"
            shift
            break
            ;;
        --help)
            print_usage
            exit 0
            ;;
    esac
done

# Run the appropriate script based on environment
if [ "$ENV" = "development" ]; then
    echo "Starting in development mode..."
    exec .devtools/scripts/run_dev.sh "$@"
elif [ "$ENV" = "production" ]; then
    echo "Starting in production mode..."
    exec .devtools/scripts/deploy_production.sh "$@"
else
    echo "Unknown environment: $ENV"
    exit 1
fi 
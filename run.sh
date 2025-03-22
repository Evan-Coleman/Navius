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

# Based on environment, run the appropriate script
if [ "$ENV" = "development" ]; then
    echo "Running in development mode..."
    if [ -f "./run_dev.sh" ]; then
        ./run_dev.sh "$@"
    else
        echo "Error: Development script run_dev.sh not found."
        exit 1
    fi
elif [ "$ENV" = "production" ]; then
    echo "Running in production mode..."
    if [ -f "./deploy_production.sh" ]; then
        ./deploy_production.sh "$@"
    else
        echo "Error: Production script deploy_production.sh not found."
        exit 1
    fi
else
    echo "Error: Unknown environment: $ENV"
    exit 1
fi 
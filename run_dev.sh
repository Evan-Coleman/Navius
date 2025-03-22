#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Script start time for performance monitoring
SCRIPT_START_TIME=$(date +%s)

# Default values
SERVER_PORT=3000
WATCH_MODE=false
RUN_MIGRATIONS=false
HEALTH_CHECK_TIMEOUT=30

# Add trap for cleanup on exit
cleanup() {
    local exit_code=$?
    # Kill any background processes we started
    if [ ! -z "$SERVER_PID" ]; then
        echo "Cleaning up server process..."
        kill $SERVER_PID 2>/dev/null || true
    fi
    
    # Report total execution time
    if [ ! -z "$SCRIPT_START_TIME" ]; then
        local end_time=$(date +%s)
        local runtime=$((end_time - SCRIPT_START_TIME))
        echo "Total script execution time: ${runtime}s"
    fi
    
    exit $exit_code
}
trap cleanup EXIT INT TERM

# Parse command line arguments first to check for --no-hooks
SKIP_HOOKS=false
for arg in "$@"; do
    case $arg in
        --no-hooks)
            SKIP_HOOKS=true
            break
            ;;
    esac
done

# Setup git hooks if they don't exist and not skipped
if [ "$SKIP_HOOKS" = false ] && [ ! -x .git/hooks/pre-commit ]; then
    echo "Git pre-commit hook not found or not executable. Setting up..."
    if [ -f scripts/setup-hooks.sh ]; then
        ./scripts/setup-hooks.sh
    else
        echo "Warning: setup-hooks.sh not found. Hooks not installed."
    fi
fi

# Parse command line arguments
SKIP_GEN=false
RELEASE_MODE=false
CONFIG_DIR="config"
ENV_FILE=".env"
RUN_ENV="development"
API_REGISTRY="config/api_registry.json"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --skip-gen           Skip API model generation"
    echo "  --release            Build and run in release mode"
    echo "  --config-dir=DIR     Use specified config directory (default: config)"
    echo "  --env=FILE           Use specified .env file (default: .env)"
    echo "  --environment=ENV    Use specified environment (default: development)"
    echo "  --no-hooks           Skip git hooks setup"
    echo "  --port=PORT          Specify server port (default: 3000)"
    echo "  --watch              Restart server on file changes"
    echo "  --run-migrations     Run database migrations before starting"
    echo "  --no-health-check    Skip health check validation after startup"
    echo "  --help               Show this help message"
}

for arg in "$@"; do
    case $arg in
        --skip-gen)
            SKIP_GEN=true
            shift
            ;;
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --config-dir=*)
            CONFIG_DIR="${arg#*=}"
            API_REGISTRY="${CONFIG_DIR}/api_registry.json"
            shift
            ;;
        --env=*)
            ENV_FILE="${arg#*=}"
            shift
            ;;
        --environment=*)
            RUN_ENV="${arg#*=}"
            shift
            ;;
        --no-hooks)
            # Already processed above
            shift
            ;;
        --port=*)
            SERVER_PORT="${arg#*=}"
            shift
            ;;
        --watch)
            WATCH_MODE=true
            shift
            ;;
        --run-migrations)
            RUN_MIGRATIONS=true
            shift
            ;;
        --no-health-check)
            HEALTH_CHECK_TIMEOUT=0
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            print_usage
            exit 1
            ;;
    esac
done

# Header
echo "==================================================="
echo "  Rust Backend Development Server"
echo "==================================================="

# Check for required tools
echo "Checking dependencies..."
MISSING_DEPS=false

if ! command -v openapi-generator &> /dev/null && [ "$SKIP_GEN" = false ]; then
    echo "Warning: OpenAPI Generator is not installed."
    echo "This is needed for API generation. You can install it from: https://openapi-generator.tech/docs/installation/"
    echo "Continuing without API generation capabilities..."
    SKIP_GEN=true
fi

if [ "$WATCH_MODE" = true ] && ! command -v cargo-watch &> /dev/null; then
    echo "Error: cargo-watch is not installed but --watch flag was used."
    echo "Install with: cargo install cargo-watch"
    MISSING_DEPS=true
fi

if [ "$MISSING_DEPS" = true ]; then
    echo "Please install missing dependencies and try again."
    exit 1
fi

# Check if port is already in use
if command -v lsof &> /dev/null; then
    if lsof -Pi :$SERVER_PORT -sTCP:LISTEN -t >/dev/null ; then
        echo "Error: Port $SERVER_PORT is already in use"
        echo "Use --port to specify a different port or stop the process using this port"
        exit 1
    fi
else
    echo "Warning: 'lsof' not found, skipping port availability check"
fi

# Check if config files exist
if [ -d "$CONFIG_DIR" ]; then
    echo "Using config directory: $CONFIG_DIR"
    
    # Check for environment-specific config file
    if [ -f "$CONFIG_DIR/$RUN_ENV.yaml" ]; then
        echo "Found environment config: $CONFIG_DIR/$RUN_ENV.yaml"
    elif [ -f "$CONFIG_DIR/default.yaml" ]; then
        echo "Found default config: $CONFIG_DIR/default.yaml"
    else
        echo "Warning: No configuration files found in $CONFIG_DIR. Using defaults."
    fi
    
    # Export CONFIG_DIR for the application
    export CONFIG_DIR="$CONFIG_DIR"
    # Export RUN_ENV for the application
    export RUN_ENV="$RUN_ENV"
else
    echo "Warning: Config directory $CONFIG_DIR not found. Using defaults."
fi

if [ -f "$ENV_FILE" ]; then
    echo "Using environment file: $ENV_FILE"
    # Load environment variables, properly handling comments
    export $(grep -v '^#' "$ENV_FILE" | sed 's/\s*#.*$//' | xargs)
else
    echo "Warning: Environment file $ENV_FILE not found. Using defaults."
fi

# Run database migrations if enabled
if [ "$RUN_MIGRATIONS" = true ]; then
    echo "Running database migrations..."
    if [ -d "migrations" ]; then
        # Check for sqlx CLI
        if command -v sqlx &> /dev/null; then
            echo "Using sqlx to run migrations..."
            # Ensure DATABASE_URL is set
            if [ -z "$DATABASE_URL" ]; then
                echo "Error: DATABASE_URL environment variable not set. Required for migrations."
                exit 1
            fi
            
            MIGRATION_START_TIME=$(date +%s)
            sqlx migrate run
            
            if [ $? -ne 0 ]; then
                echo "Error: Database migrations failed."
                exit 1
            fi
            
            MIGRATION_END_TIME=$(date +%s)
            MIGRATION_RUNTIME=$((MIGRATION_END_TIME - MIGRATION_START_TIME))
            echo "Migrations completed successfully in ${MIGRATION_RUNTIME}s."
        else
            echo "Error: sqlx CLI not found but --run-migrations flag was used."
            echo "Install with: cargo install sqlx-cli"
            exit 1
        fi
    else
        echo "Error: migrations directory not found."
        exit 1
    fi
fi

# Generate API models if needed
GEN_START_TIME=$(date +%s)
if [ "$SKIP_GEN" = false ]; then
    echo "Checking for APIs that need generation..."
    
    # Check if API registry exists
    if [ -f "$API_REGISTRY" ]; then
        # Create generated directory if it doesn't exist
        mkdir -p generated/openapi
        
        # Always preserve registry settings
        cp "$API_REGISTRY" "${API_REGISTRY}.bak"
        
        # Read the API registry and generate missing APIs
        api_count=$(jq '.apis | length' "$API_REGISTRY")
        
        if [ "$api_count" -gt 0 ]; then
            echo "Found $api_count registered APIs."
            
            for i in $(seq 0 $(($api_count - 1))); do
                api_name=$(jq -r ".apis[$i].name" "$API_REGISTRY")
                
                # Check if this API has generation enabled
                generate_models=$(jq -r ".apis[$i].options.generate_models // true" "$API_REGISTRY")
                
                # Always get the original settings from backup
                generate_api=$(jq -r ".apis[$i].options.generate_api // true" "${API_REGISTRY}.bak")
                generate_handlers=$(jq -r ".apis[$i].options.generate_handlers // true" "${API_REGISTRY}.bak")
                
                # Skip if all generation options are disabled
                if [ "$generate_models" != "true" ] && [ "$generate_api" != "true" ] && [ "$generate_handlers" != "true" ]; then
                    echo "API $api_name has all generation options disabled, skipping."
                    continue
                fi
                
                # Check if this API is already generated
                if [ ! -d "generated/${api_name}_api" ]; then
                    echo "Generating API client for $api_name..."
                    
                    # Always use the backup for generation
                    # Temporarily update the registry with preserved settings
                    jq --arg i "$i" \
                       --arg generate_api "$generate_api" \
                       --arg generate_handlers "$generate_handlers" \
                       '.apis[$i | tonumber].options.generate_api = ($generate_api == "true") | 
                        .apis[$i | tonumber].options.generate_handlers = ($generate_handlers == "true")' \
                       "$API_REGISTRY" > "${API_REGISTRY}.tmp"
                    mv "${API_REGISTRY}.tmp" "$API_REGISTRY"
                    
                    # Run the add_api.sh script with just the API name to use registry configuration
                    ./scripts/add_api.sh "$api_name"
                    
                    if [ $? -ne 0 ]; then
                        echo "Warning: Failed to generate API client for $api_name. Continuing..."
                    else
                        echo "Successfully generated API client for $api_name."
                    fi
                else
                    echo "API client for $api_name already exists, skipping generation."
                fi
            done
            
            # Always restore the original registry
            mv "${API_REGISTRY}.bak" "$API_REGISTRY"
            echo "Preserved original API registry settings."
        else
            echo "No APIs registered in $API_REGISTRY."
        fi
    else
        echo "API registry file $API_REGISTRY not found. Skipping API generation."
    fi
else
    echo "Skipping API model generation (--skip-gen flag used)"
fi
GEN_END_TIME=$(date +%s)
GEN_RUNTIME=$((GEN_END_TIME - GEN_START_TIME))
echo "API generation process completed in ${GEN_RUNTIME}s."

# Build the project
BUILD_START_TIME=$(date +%s)
echo "Building the project..."

# Pass the server port to the application
export SERVER_PORT="$SERVER_PORT"

if [ "$RELEASE_MODE" = true ]; then
    echo "Building in release mode..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Error: Release build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/release/navius"
else
    cargo build
    if [ $? -ne 0 ]; then
        echo "Error: Debug build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/debug/navius"
fi

BUILD_END_TIME=$(date +%s)
BUILD_RUNTIME=$((BUILD_END_TIME - BUILD_START_TIME))
echo "Build successful in ${BUILD_RUNTIME}s."

# Set RUST_LOG if not already set
if [ -z "$RUST_LOG" ]; then
    export RUST_LOG=info
    echo "Setting log level to info (RUST_LOG=info)"
fi

# Function to start the server
start_server() {
    # Run the executable
    echo "Starting server on port $SERVER_PORT..."
    echo "Press Ctrl+C to stop the server."
    echo "---------------------------------------------------"
    
    if [ "$WATCH_MODE" = true ]; then
        echo "Running in watch mode. Server will restart when files change."
        cargo watch -x "run" &
        SERVER_PID=$!
    else
        "$EXEC_PATH" &
        SERVER_PID=$!
    fi
    
    # Wait for server to start and perform health check
    if [ $HEALTH_CHECK_TIMEOUT -gt 0 ]; then
        echo "Waiting for server to start (max ${HEALTH_CHECK_TIMEOUT}s)..."
        for i in $(seq 1 $HEALTH_CHECK_TIMEOUT); do
            if curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
                echo "Server is up and running (verified in ${i}s)"
                break
            fi
            
            # Check if the server process is still running
            if ! kill -0 $SERVER_PID 2>/dev/null; then
                echo "Error: Server process exited unexpectedly"
                exit 1
            fi
            
            sleep 1
            
            if [ $i -eq $HEALTH_CHECK_TIMEOUT ]; then
                echo "Error: Server health check timed out after ${HEALTH_CHECK_TIMEOUT}s"
                exit 1
            fi
        done
    fi
}

# Start the server
start_server

# Wait for the server to complete (or Ctrl+C)
wait $SERVER_PID

# This part will execute after server shutdown (Ctrl+C)
echo "Server stopped."

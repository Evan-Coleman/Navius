#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Script to build the Navius application for different environments

# Default values
BUILD_TYPE="debug"
RUN_TESTS=true
CLEAN_FIRST=false
TARGET_DIR="target"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --release             Build in release mode"
    echo "  --debug               Build in debug mode (default)"
    echo "  --clean               Clean before building"
    echo "  --no-tests            Skip running tests"
    echo "  --target-dir=DIR      Specify target directory"
    echo "  --help                Show this help message"
}

# Process arguments
for arg in "$@"; do
    case $arg in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --clean)
            CLEAN_FIRST=true
            shift
            ;;
        --no-tests)
            RUN_TESTS=false
            shift
            ;;
        --target-dir=*)
            TARGET_DIR="${arg#*=}"
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

# Start of build process
echo "Building Navius in ${BUILD_TYPE} mode..."

# Clean if requested
if [ "$CLEAN_FIRST" = true ]; then
    echo "Cleaning target directory..."
    cargo clean
fi

# Build with appropriate flags
if [ "$BUILD_TYPE" = "release" ]; then
    echo "Building release version..."
    cargo build --release
else
    echo "Building debug version..."
    cargo build
fi

# Run tests if enabled
if [ "$RUN_TESTS" = true ]; then
    echo "Running tests..."
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo test --release
    else
        cargo test
    fi
fi

# Report build success and binary location
if [ "$BUILD_TYPE" = "release" ]; then
    BINARY_PATH="${TARGET_DIR}/release/navius"
else
    BINARY_PATH="${TARGET_DIR}/debug/navius"
fi

if [ -f "$BINARY_PATH" ]; then
    echo "Build successful! Binary located at: ${BINARY_PATH}"
    echo "Run with: ${BINARY_PATH}"
else
    echo "Build failed: Binary not found at expected location: ${BINARY_PATH}"
    exit 1
fi 
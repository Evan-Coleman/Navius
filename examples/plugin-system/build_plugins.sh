#!/bin/bash
# Build script for dynamic plugins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building dynamic plugins..."

# Create plugins directory if it doesn't exist
mkdir -p plugins

# Build the dynamic plugin
echo "Building dynamic_plugin..."
cd plugins/dynamic_plugin
cargo build
cd ../..

# Copy the compiled plugin to the plugins directory
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    cp plugins/dynamic_plugin/target/debug/libdynamic_plugin.dylib plugins/
    echo "Copied libdynamic_plugin.dylib to plugins directory"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    cp plugins/dynamic_plugin/target/debug/libdynamic_plugin.so plugins/
    echo "Copied libdynamic_plugin.so to plugins directory"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    # Windows
    cp plugins/dynamic_plugin/target/debug/dynamic_plugin.dll plugins/
    echo "Copied dynamic_plugin.dll to plugins directory"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

echo "Dynamic plugins built successfully!" 
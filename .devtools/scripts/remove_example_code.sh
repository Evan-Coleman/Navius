#!/bin/bash

# Script to remove all example code from the project
# This will remove files and directories that start with 'example_'

echo "Removing example code from the project..."

# Find and remove all files starting with 'example_'
find ./src -name "example_*" -type f -print -delete

# Find and remove all directories starting with 'example_'
find ./src -name "example_*" -type d -print -delete

# Remove example imports from module files
echo "Cleaning up module imports..."

# Remove 'pub mod example_*' lines from .rs files
find ./src -name "*.rs" -type f -exec sed -i '' '/pub mod example_/d' {} \;

# Remove 'pub use example_*' lines from .rs files
find ./src -name "*.rs" -type f -exec sed -i '' '/pub use example_/d' {} \;

# Remove empty lines that might have been created
find ./src -name "*.rs" -type f -exec sed -i '' '/^$/N;/^\n$/D' {} \;

echo "Example code removal complete!" 
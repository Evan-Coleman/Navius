#!/bin/bash

# Setup Git hooks for the project
# This script installs the pre-commit hook to check for sensitive data

# Ensure hooks directory exists
mkdir -p .git/hooks

# Copy pre-commit hook if it exists
if [ -f scripts/pre-commit.sh ]; then
    echo "Installing pre-commit hook..."
    cp scripts/pre-commit.sh .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "Git hooks installed successfully."
    echo "Pre-commit hook will check for sensitive data before each commit."
else
    echo "Error: pre-commit.sh not found in scripts directory."
    exit 1
fi 
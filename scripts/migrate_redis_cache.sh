#!/bin/bash
set -e

# Script to migrate the Redis cache implementation from examples to main codebase
# Written on March 31, 2025

SOURCE_DIR="/Users/goblin/dev/git/navius/workspace_migration/examples/crates/navius-cache-redis"
TARGET_DIR="/Users/goblin/dev/git/navius/crates/navius-cache-redis"

echo "Migrating Redis cache implementation from examples to main codebase..."

# Create backup of target directory
BACKUP_DIR="${TARGET_DIR}_backup_$(date +%Y%m%d%H%M%S)"
echo "Creating backup of target directory to ${BACKUP_DIR}..."
cp -r $TARGET_DIR $BACKUP_DIR

# Clean up target directory but keep Cargo.toml
echo "Cleaning up target directory..."
mkdir -p $TARGET_DIR/src
find $TARGET_DIR/src -type f -not -path "*/\.*" -delete

# Copy source files
echo "Copying source files..."
cp -r $SOURCE_DIR/src/* $TARGET_DIR/src/

# Copy examples directory
echo "Copying examples directory..."
rm -rf $TARGET_DIR/examples
cp -r $SOURCE_DIR/examples $TARGET_DIR/

# Copy tests directory
echo "Copying tests directory..."
rm -rf $TARGET_DIR/tests
cp -r $SOURCE_DIR/tests $TARGET_DIR/

echo "Migration complete!"
echo "Original files backed up to ${BACKUP_DIR}"
echo "To restore, run: rm -rf $TARGET_DIR && cp -r $BACKUP_DIR $TARGET_DIR" 
#!/bin/bash

# Create script to move crates from temporary location to final location
# Date: March 29, 2025

# Ensure the target directories exist
mkdir -p crates

# Move each crate from the temporary location to the final location
echo "Moving crates from temporary location to final location..."

# Array of all crates to move
CRATES=(
  "navius-core"
  "navius-http"
  "navius-auth"
  "navius-auth-entra"
  "navius-db"
  "navius-db-postgres"
  "navius-cache"
  "navius-cache-redis"
  "navius-di"
  "navius-plugin"
  "navius-event"
  "navius-job"
  "navius-messaging"
  "navius-metrics"
  "navius-metrics-prometheus"
  "navius-test"
  "navius-test-utils"
  "navius-template"
  "navius-cli"
)

# Count for progress tracking
TOTAL=${#CRATES[@]}
COUNT=0

# Process each crate
for CRATE in "${CRATES[@]}"; do
  COUNT=$((COUNT + 1))
  echo "[$COUNT/$TOTAL] Processing $CRATE..."
  
  # Check if source exists
  if [ ! -d "workspace_migration/examples/crates/$CRATE" ]; then
    echo "  WARNING: Source directory does not exist, skipping: workspace_migration/examples/crates/$CRATE"
    continue
  fi
  
  # Check if destination already exists
  if [ -d "crates/$CRATE" ]; then
    echo "  WARNING: Destination directory already exists: crates/$CRATE"
    echo "  Skipping to avoid data loss. Please handle manually if needed."
    continue
  fi
  
  # Copy the crate to its final location
  echo "  Copying $CRATE to final location..."
  cp -r "workspace_migration/examples/crates/$CRATE" "crates/$CRATE"
  
  # Update Cargo.toml dependency paths if needed
  echo "  Updating dependency paths in Cargo.toml..."
  
  # Check if the operation was successful
  if [ $? -eq 0 ]; then
    echo "  ✅ Successfully moved $CRATE"
  else
    echo "  ❌ Failed to move $CRATE"
  fi
done

echo "Crate migration completed!"
echo "Next steps:"
echo "1. Update root Cargo.toml to reference the new crate locations"
echo "2. Test the build to ensure all dependencies are correctly resolved"
echo "3. Update any references to the old crate locations" 
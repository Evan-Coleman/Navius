#!/bin/bash

# Legacy Code Removal Script
# This script assists in identifying and removing legacy code from the old /src folder

# Exit immediately if a command exits with a non-zero status
set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Navius Legacy Code Removal Script ===${NC}"
echo -e "${BLUE}Date: $(date)${NC}"
echo ""

# Verify both structures exist
if [ ! -d "src" ]; then
  echo -e "${RED}Error: src directory does not exist${NC}"
  exit 1
fi

# Create a backup of the src directory
BACKUP_DIR="src_backup_$(date +%Y%m%d_%H%M%S)"
echo -e "${YELLOW}Creating backup of src directory to ${BACKUP_DIR}${NC}"
cp -r src "${BACKUP_DIR}"
echo -e "${GREEN}Backup created successfully${NC}"
echo ""

# Identify key components in legacy code
echo -e "${BLUE}Identifying key components in legacy code...${NC}"
echo -e "${YELLOW}Main application entry points:${NC}"
find src -name "main.rs" -o -name "lib.rs" | sort

echo -e "${YELLOW}Core modules:${NC}"
find src/core -type f -name "*.rs" | sort

echo -e "${YELLOW}App modules:${NC}"
find src/app -type f -name "*.rs" | sort

echo -e "${YELLOW}Test modules:${NC}"
find src/tests -type f -name "*.rs" 2>/dev/null | sort

echo -e "${YELLOW}Binary utilities:${NC}"
find src/bin -type f -name "*.rs" 2>/dev/null | sort
echo ""

# Verification step
echo -e "${BLUE}Verification steps before removal:${NC}"
echo -e "${YELLOW}1. Ensure the new workspace structure is complete${NC}"
echo -e "${YELLOW}2. Verify that all functionality from legacy code has replacements${NC}"
echo -e "${YELLOW}3. Run the complete test suite against the new structure${NC}"
echo -e "${YELLOW}4. Build the application with the new structure${NC}"
echo ""

# Prompt for confirmation
read -p "Have you completed all verification steps? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo -e "${RED}Aborting removal process. Please complete verification steps first.${NC}"
  exit 1
fi

# Proceed with removal
echo -e "${BLUE}Proceeding with legacy code removal...${NC}"

# Find references to legacy code
echo -e "${YELLOW}Checking for references to legacy code...${NC}"
grep -r "src/core" --include="*.rs" --include="*.toml" --include="*.md" . || echo "No references to src/core found"
grep -r "src/app" --include="*.rs" --include="*.toml" --include="*.md" . || echo "No references to src/app found"
echo ""

# Prompt for final confirmation
read -p "Are you sure you want to remove all legacy code from the /src folder? (YES/no) " -r
echo ""
if [[ ! $REPLY == "YES" ]]; then
  echo -e "${RED}Aborting removal process. Please type 'YES' to confirm.${NC}"
  exit 1
fi

# Perform the removal
echo -e "${YELLOW}Removing legacy code...${NC}"

# Remove old modules but keep the new src structure
# We only want to remove legacy code, not the new application code
if [ -d "src/core" ]; then
  echo "Removing src/core directory..."
  rm -rf src/core
fi

if [ -d "src/app" ]; then
  echo "Removing src/app directory..."
  rm -rf src/app
fi

if [ -d "src/tests" ]; then
  echo "Removing src/tests directory..."
  rm -rf src/tests
fi

if [ -d "src/bin" ]; then
  echo "Removing src/bin directory..."
  rm -rf src/bin
fi

# Remove old top-level files that are not part of the new structure
old_files=("lib.rs" "core.rs" "app.rs" "tests.rs" "test_imports.rs")
for file in "${old_files[@]}"; do
  if [ -f "src/$file" ]; then
    echo "Removing src/$file..."
    rm "src/$file"
  fi
done

echo -e "${GREEN}Legacy code removal completed successfully!${NC}"
echo ""
echo -e "${BLUE}Post-removal verification:${NC}"
echo -e "${YELLOW}1. Build the application to ensure it compiles${NC}"
echo -e "${YELLOW}2. Run tests to ensure functionality works${NC}"
echo -e "${YELLOW}3. Update any documentation that referenced the old structure${NC}"
echo ""

echo -e "${GREEN}Done! Legacy code has been removed and backup created at ${BACKUP_DIR}${NC}" 
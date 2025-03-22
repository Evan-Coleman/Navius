#!/bin/bash
# verify-structure.sh - Verifies the project structure against expected organization
# This script checks that the project follows the expected directory structure and patterns

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Verifying Navius project structure...${NC}"

# Expected directories at the root level
expected_root_dirs=(".devtools" "config" "docs" "migrations" "src" "target" "tests")
expected_core_dirs=("api" "auth" "cache" "config" "database" "error" "metrics" "reliability" 
                   "repository" "router" "services" "utils")
expected_app_dirs=("api" "services")

# Check root directories
echo -e "\n${BLUE}Checking root directories...${NC}"
for dir in "${expected_root_dirs[@]}"; do
  if [ -d "$dir" ]; then
    echo -e "${GREEN}✓ Found $dir${NC}"
  else
    echo -e "${RED}✗ Missing $dir${NC}"
  fi
done

# Check src/core directories
echo -e "\n${BLUE}Checking src/core directories...${NC}"
for dir in "${expected_core_dirs[@]}"; do
  if [ -d "src/core/$dir" ]; then
    echo -e "${GREEN}✓ Found src/core/$dir${NC}"
  else
    echo -e "${RED}✗ Missing src/core/$dir${NC}"
  fi
done

# Check src/app directories
echo -e "\n${BLUE}Checking src/app directories and files...${NC}"
if [ -f "src/app/router.rs" ]; then
  echo -e "${GREEN}✓ Found src/app/router.rs${NC}"
else
  echo -e "${RED}✗ Missing src/app/router.rs${NC}"
fi

for dir in "${expected_app_dirs[@]}"; do
  if [ -d "src/app/$dir" ]; then
    echo -e "${GREEN}✓ Found src/app/$dir${NC}"
  else
    echo -e "${YELLOW}! Missing src/app/$dir (not required but recommended)${NC}"
  fi
done

# Check for lib.rs and main.rs
echo -e "\n${BLUE}Checking entry point files...${NC}"
for file in "src/lib.rs" "src/main.rs" "src/generated_apis.rs"; do
  if [ -f "$file" ]; then
    echo -e "${GREEN}✓ Found $file${NC}"
  else
    echo -e "${RED}✗ Missing $file${NC}"
  fi
done

# Check for potentially misplaced modules
echo -e "\n${BLUE}Checking for potentially misplaced modules...${NC}"
for dir in $(find src -maxdepth 1 -type d -not -path "src" -not -path "src/app" -not -path "src/core"); do
  if [ "$dir" != "src/cache" ] && [ "$dir" != "src/config" ]; then
    echo -e "${YELLOW}! Potentially misplaced directory: $dir (should be in src/core or src/app)${NC}"
  fi
done

# Check for core module imports in app modules
echo -e "\n${BLUE}Checking for proper imports...${NC}"
grep_result=$(grep -r --include="*.rs" "use crate::core" src/app || echo "")
if [ -n "$grep_result" ]; then
  echo -e "${GREEN}✓ Found proper core imports in app modules${NC}"
else
  echo -e "${YELLOW}! No core imports found in app modules. App modules should use core functionality.${NC}"
fi

# Check for public API exports in lib.rs
echo -e "\n${BLUE}Checking for public exports in lib.rs...${NC}"
if grep -q "pub mod app" src/lib.rs && grep -q "pub mod core" src/lib.rs; then
  echo -e "${GREEN}✓ Found proper public exports in lib.rs${NC}"
else
  echo -e "${RED}✗ Missing public exports in lib.rs (should export app and core modules)${NC}"
fi

# Check for generated directory in target
echo -e "\n${BLUE}Checking for generated code...${NC}"
if [ -d "target/generated" ]; then
  echo -e "${GREEN}✓ Found target/generated directory${NC}"
else
  echo -e "${YELLOW}! Missing target/generated directory. Run 'cargo build' to generate API clients.${NC}"
fi

# Count tests to ensure there are some
echo -e "\n${BLUE}Counting tests...${NC}"
test_count=$(grep -r --include="*.rs" "#\[test\]" . | wc -l)
echo -e "${GREEN}Found $test_count tests${NC}"
if [ "$test_count" -lt 50 ]; then
  echo -e "${YELLOW}! Warning: Test count is lower than expected (< 50)${NC}"
fi

echo -e "\n${BLUE}Structure verification complete.${NC}" 
#!/bin/bash

# Verification and Testing Script for Migration
# This script assists in verifying the success of the code migration

# Exit immediately if a command exits with a non-zero status
set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Navius Migration Verification Script ===${NC}"
echo -e "${BLUE}Date: $(date)${NC}"
echo ""

# Check for workspace structure
echo -e "${BLUE}Verifying workspace structure...${NC}"
if [ ! -d "crates" ]; then
  echo -e "${RED}Error: crates directory does not exist${NC}"
  exit 1
fi

if [ ! -d "src" ]; then
  echo -e "${RED}Error: src directory does not exist${NC}"
  exit 1
fi

if [ ! -f "Cargo.toml" ]; then
  echo -e "${RED}Error: Cargo.toml does not exist${NC}"
  exit 1
fi

echo -e "${GREEN}Workspace structure verified!${NC}"
echo ""

# Check for workspace members
echo -e "${BLUE}Verifying workspace members...${NC}"
members=$(grep -c "\[workspace\].members" Cargo.toml)
if [ $members -eq 0 ]; then
  echo -e "${RED}Error: No workspace members defined in Cargo.toml${NC}"
  exit 1
fi

echo -e "${GREEN}Workspace members verified!${NC}"
echo ""

# Verify core crates
echo -e "${BLUE}Verifying core crates...${NC}"
core_crates=("navius-core" "navius-http" "navius-auth" "navius-db" "navius-cache" "navius-di")
for crate in "${core_crates[@]}"; do
  if [ ! -d "crates/$crate" ]; then
    echo -e "${RED}Error: Core crate $crate not found in crates directory${NC}"
    exit 1
  else
    echo -e "✅ $crate"
  fi
done

echo -e "${GREEN}Core crates verified!${NC}"
echo ""

# Verify main application structure
echo -e "${BLUE}Verifying main application structure...${NC}"
app_components=("main.rs" "config.rs" "api.rs" "application.rs" "infrastructure.rs")
for component in "${app_components[@]}"; do
  if [ ! -f "src/$component" ]; then
    echo -e "${RED}Error: Application component src/$component not found${NC}"
    exit 1
  else
    echo -e "✅ src/$component"
  fi
done

echo -e "${GREEN}Main application structure verified!${NC}"
echo ""

# Verify config directory
echo -e "${BLUE}Verifying configuration files...${NC}"
if [ ! -d "config" ]; then
  echo -e "${RED}Error: config directory does not exist${NC}"
  exit 1
fi

if [ ! -f "config/default.yaml" ]; then
  echo -e "${RED}Error: Default configuration file not found${NC}"
  exit 1
fi

echo -e "${GREEN}Configuration files verified!${NC}"
echo ""

# Run cargo check to verify the project builds
echo -e "${BLUE}Running cargo check to verify the project builds...${NC}"
cargo check

if [ $? -ne 0 ]; then
  echo -e "${RED}Error: cargo check failed${NC}"
  exit 1
fi

echo -e "${GREEN}Project builds successfully!${NC}"
echo ""

# Run unit tests
echo -e "${BLUE}Running unit tests...${NC}"
cargo test --workspace --lib

if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Unit tests failed${NC}"
  exit 1
fi

echo -e "${GREEN}Unit tests passed!${NC}"
echo ""

# Run integration tests
echo -e "${BLUE}Running integration tests...${NC}"
cargo test --workspace --test '*'

if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Integration tests failed${NC}"
  exit 1
fi

echo -e "${GREEN}Integration tests passed!${NC}"
echo ""

# Build the project
echo -e "${BLUE}Building the project...${NC}"
cargo build --workspace

if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Build failed${NC}"
  exit 1
fi

echo -e "${GREEN}Project built successfully!${NC}"
echo ""

# Measure build performance
echo -e "${BLUE}Measuring build performance...${NC}"
time cargo build --workspace --release

echo -e "${GREEN}Build performance measured!${NC}"
echo ""

# Verify documentation
echo -e "${BLUE}Verifying documentation...${NC}"
cargo doc --no-deps --workspace

if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Documentation generation failed${NC}"
  exit 1
fi

echo -e "${GREEN}Documentation verified!${NC}"
echo ""

echo -e "${BLUE}Verification Summary:${NC}"
echo -e "${GREEN}✅ Workspace structure${NC}"
echo -e "${GREEN}✅ Workspace members${NC}"
echo -e "${GREEN}✅ Core crates${NC}"
echo -e "${GREEN}✅ Main application structure${NC}"
echo -e "${GREEN}✅ Configuration files${NC}"
echo -e "${GREEN}✅ Project builds${NC}"
echo -e "${GREEN}✅ Unit tests${NC}"
echo -e "${GREEN}✅ Integration tests${NC}"
echo -e "${GREEN}✅ Documentation${NC}"
echo ""

echo -e "${GREEN}Migration verification completed successfully! The project is in good shape.${NC}" 
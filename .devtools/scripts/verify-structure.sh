#!/bin/bash
# Script to verify the project structure and check for proper module exports

echo "=== Project Structure Verification ==="
echo "Checking for proper module exports in lib.rs..."

# Define colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to check if app directories exist
check_app_directories() {
  local issues=0

  echo -e "\nChecking app directories..."
  if [ -d "src/app/api" ]; then
    echo -e "${GREEN}[OK]${NC} src/app/api exists"
  else
    echo -e "${RED}[ISSUE]${NC} src/app/api does not exist"
    ((issues++))
  fi

  if [ -d "src/app/services" ]; then
    echo -e "${GREEN}[OK]${NC} src/app/services exists"
  else
    echo -e "${RED}[ISSUE]${NC} src/app/services does not exist"
    ((issues++))
  fi

  return $issues
}

# Function to check lib.rs for proper module exports
check_lib_rs_exports() {
  echo -e "\nChecking lib.rs for proper module exports..."
  local lib_rs="src/lib.rs"
  local issues=0

  # Define module names
  local modules=(
    "metrics" "repository" "api" "error" "auth" "reliability" "utils" "models" "services"
  )

  for module in "${modules[@]}"; do
    if grep -q "pub mod $module" "$lib_rs"; then
      if grep -q "pub use crate::core::$module" "$lib_rs"; then
        echo -e "${GREEN}[OK]${NC} lib.rs properly exports $module through core"
      else
        echo -e "${YELLOW}[WARNING]${NC} lib.rs may not properly export $module through core"
        ((issues++))
      fi
    else
      echo -e "${RED}[ISSUE]${NC} lib.rs does not export $module"
      ((issues++))
    fi
  done

  return $issues
}

# Function to check app/mod.rs for proper submodule exports
check_app_exports() {
  echo -e "\nChecking app/mod.rs for proper submodule exports..."
  local app_mod="src/app/mod.rs"
  local issues=0

  if grep -q "pub mod api" "$app_mod"; then
    echo -e "${GREEN}[OK]${NC} app/mod.rs properly exports api submodule"
  else
    echo -e "${RED}[ISSUE]${NC} app/mod.rs does not export api submodule"
    ((issues++))
  fi

  if grep -q "pub mod services" "$app_mod"; then
    echo -e "${GREEN}[OK]${NC} app/mod.rs properly exports services submodule"
  else
    echo -e "${RED}[ISSUE]${NC} app/mod.rs does not export services submodule"
    ((issues++))
  fi

  return $issues
}

# Run all checks
app_dir_issues=0
lib_rs_issues=0
app_export_issues=0

check_app_directories
app_dir_issues=$?

check_lib_rs_exports
lib_rs_issues=$?

check_app_exports
app_export_issues=$?

total_issues=$((app_dir_issues + lib_rs_issues + app_export_issues))

echo -e "\n=== Verification Summary ==="
if [ $total_issues -eq 0 ]; then
  echo -e "${GREEN}All checks passed! The project structure is in compliance with the standards.${NC}"
  echo -e "Note: Physical files in src/ may still exist but they are properly redirected through lib.rs"
  exit 0
else
  echo -e "${YELLOW}Found $total_issues structural issues that need attention.${NC}"
  echo -e "Please review the roadmap in docs/roadmaps/11_project_structure_future_improvements.md"
  exit 1
fi 
#!/bin/bash
# Script to verify the project structure and check for proper module exports

echo "=== Project Structure Verification ==="
echo "Checking for compliance with Navius structure standards..."

# Define colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to check if app directories exist
check_app_directories() {
  local issues=0

  echo -e "\n${CYAN}Checking app directories...${NC}"
  if [ -d "src/app/api" ]; then
    echo -e "${GREEN}[OK]${NC} src/app/api exists"
  else
    echo -e "${RED}[ISSUE]${NC} src/app/api does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create this directory with 'mkdir -p src/app/api'"
    ((issues++))
  fi

  if [ -d "src/app/services" ]; then
    echo -e "${GREEN}[OK]${NC} src/app/services exists"
  else
    echo -e "${RED}[ISSUE]${NC} src/app/services does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create this directory with 'mkdir -p src/app/services'"
    ((issues++))
  fi

  if [ -d "src/app/api/examples" ]; then
    echo -e "${GREEN}[OK]${NC} src/app/api/examples exists"
  else
    echo -e "${YELLOW}[WARNING]${NC} src/app/api/examples does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create this directory with example handlers for reference"
    ((issues++))
  fi

  if [ -d "src/app/services/examples" ]; then
    echo -e "${YELLOW}[WARNING]${NC} src/app/services/examples does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create this directory with example services for reference"
    ((issues++))
  fi

  return $issues
}

# Function to check lib.rs for proper module exports
check_lib_rs_exports() {
  echo -e "\n${CYAN}Checking lib.rs for proper module exports...${NC}"
  local lib_rs="src/lib.rs"
  local issues=0

  if [ ! -f "$lib_rs" ]; then
    echo -e "${RED}[ERROR]${NC} $lib_rs does not exist"
    return 1
  fi

  # Define expected core module names
  local core_modules=(
    "metrics" "repository" "api" "error" "auth" "reliability" "utils" "models" "services" "router" "cache" "config"
  )

  # Check for core module exports
  for module in "${core_modules[@]}"; do
    if grep -q "pub mod core::$module" "$lib_rs" || grep -q "pub use crate::core::$module" "$lib_rs"; then
      echo -e "${GREEN}[OK]${NC} lib.rs properly exports core::$module"
    else
      echo -e "${RED}[ISSUE]${NC} lib.rs does not properly export core::$module"
      echo -e "  ${MAGENTA}Recommendation:${NC} Add 'pub use crate::core::$module;' to lib.rs"
      ((issues++))
    fi
  done

  # Check for app module exports
  if grep -q "pub mod app" "$lib_rs" || grep -q "pub use crate::app" "$lib_rs"; then
    echo -e "${GREEN}[OK]${NC} lib.rs properly exports app module"
  else
    echo -e "${RED}[ISSUE]${NC} lib.rs does not properly export app module"
    echo -e "  ${MAGENTA}Recommendation:${NC} Add 'pub mod app;' to lib.rs"
    ((issues++))
  fi

  return $issues
}

# Function to check app/mod.rs for proper submodule exports
check_app_exports() {
  echo -e "\n${CYAN}Checking app/mod.rs for proper submodule exports...${NC}"
  local app_mod="src/app/mod.rs"
  local issues=0

  if [ ! -f "$app_mod" ]; then
    echo -e "${RED}[ERROR]${NC} $app_mod does not exist"
    return 1
  fi

  if grep -q "pub mod api" "$app_mod"; then
    echo -e "${GREEN}[OK]${NC} app/mod.rs properly exports api submodule"
  else
    echo -e "${RED}[ISSUE]${NC} app/mod.rs does not export api submodule"
    echo -e "  ${MAGENTA}Recommendation:${NC} Add 'pub mod api;' to app/mod.rs"
    ((issues++))
  fi

  if grep -q "pub mod services" "$app_mod"; then
    echo -e "${GREEN}[OK]${NC} app/mod.rs properly exports services submodule"
  else
    echo -e "${RED}[ISSUE]${NC} app/mod.rs does not export services submodule"
    echo -e "  ${MAGENTA}Recommendation:${NC} Add 'pub mod services;' to app/mod.rs"
    ((issues++))
  fi

  return $issues
}

# Function to check for old source code structure
check_old_src_structure() {
  echo -e "\n${CYAN}Checking for old structure patterns...${NC}"
  local issues=0
  
  # List of directories that should not exist at the root src/ level
  local old_directories=(
    "src/metrics" "src/repository" "src/api" "src/error" "src/auth" 
    "src/reliability" "src/utils" "src/models" "src/services"
    "src/handlers" "src/apis" "src/controllers"
  )
  
  for dir in "${old_directories[@]}"; do
    if [ -d "$dir" ]; then
      echo -e "${RED}[ISSUE]${NC} $dir exists but should be in src/core/ or src/app/"
      echo -e "  ${MAGENTA}Recommendation:${NC} Move this directory to the appropriate location using core transition process"
      ((issues++))
    else
      echo -e "${GREEN}[OK]${NC} $dir properly relocated or doesn't exist"
    fi
  done
  
  return $issues
}

# Function to check import patterns in Rust files
check_import_patterns() {
  echo -e "\n${CYAN}Checking import patterns across the codebase...${NC}"
  local issues=0
  local files_to_check=$(find src -name "*.rs" | grep -v "target" | grep -v "generated")
  
  echo -e "${BLUE}Analyzing imports in Rust files...${NC}"
  
  # Find files using old import paths
  for file in $files_to_check; do
    local old_imports=$(grep -E "use crate::(metrics|repository|api|error|auth|reliability|utils|models|services)" "$file" | grep -v "use crate::core::" | wc -l)
    
    if [ "$old_imports" -gt 0 ]; then
      echo -e "${YELLOW}[WARNING]${NC} $file contains $old_imports imports that should use the core module"
      echo -e "  ${MAGENTA}Recommendation:${NC} Update imports to use 'crate::core::' prefix"
      grep -n -E "use crate::(metrics|repository|api|error|auth|reliability|utils|models|services)" "$file" | grep -v "use crate::core::" | head -3 | while read -r line; do
        echo -e "    ${YELLOW}Line:${NC} $line"
      done
      ((issues++))
    fi
  done
  
  return $issues
}

# Function to check correct naming conventions
check_naming_conventions() {
  echo -e "\n${CYAN}Checking naming conventions...${NC}"
  local issues=0
  
  # Find files that don't follow naming conventions
  local files_with_bad_naming=$(find src -name "*.rs" | grep -v "mod.rs" | grep -v "lib.rs" | grep -v "main.rs" | grep -E '[-.]' | wc -l)
  
  if [ "$files_with_bad_naming" -gt 0 ]; then
    echo -e "${YELLOW}[WARNING]${NC} Found $files_with_bad_naming files that may not follow Rust naming conventions"
    echo -e "  ${MAGENTA}Recommendation:${NC} Rename files to use snake_case instead of kebab-case or dot.notation"
    find src -name "*.rs" | grep -v "mod.rs" | grep -v "lib.rs" | grep -v "main.rs" | grep -E '[-.]' | head -5 | while read -r file; do
      echo -e "    ${YELLOW}File:${NC} $file"
    done
    ((issues++))
  else
    echo -e "${GREEN}[OK]${NC} All Rust files follow proper naming conventions"
  fi
  
  return $issues
}

# Function to check IDE configuration
check_ide_config() {
  echo -e "\n${CYAN}Checking IDE configuration...${NC}"
  local issues=0
  
  if [ -d ".devtools/ide/vscode" ]; then
    echo -e "${GREEN}[OK]${NC} VS Code configuration exists"
    
    if [ -f ".devtools/ide/vscode/settings.json" ]; then
      echo -e "${GREEN}[OK]${NC} VS Code settings file exists"
    else
      echo -e "${YELLOW}[WARNING]${NC} VS Code settings file does not exist"
      echo -e "  ${MAGENTA}Recommendation:${NC} Create a settings.json file with recommended settings"
      ((issues++))
    fi
    
    if [ -f ".devtools/ide/vscode/extensions.json" ]; then
      echo -e "${GREEN}[OK]${NC} VS Code extensions file exists"
    else
      echo -e "${YELLOW}[WARNING]${NC} VS Code extensions file does not exist"
      echo -e "  ${MAGENTA}Recommendation:${NC} Create an extensions.json file with recommended extensions"
      ((issues++))
    fi
    
  else
    echo -e "${YELLOW}[WARNING]${NC} VS Code configuration does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create .devtools/ide/vscode directory with configuration files"
    ((issues++))
  fi
  
  return $issues
}

# Function to check build configuration
check_build_config() {
  echo -e "\n${CYAN}Checking build configuration...${NC}"
  local issues=0
  
  if [ -d ".cargo" ]; then
    echo -e "${GREEN}[OK]${NC} Cargo configuration directory exists"
    
    if [ -f ".cargo/config.toml" ]; then
      echo -e "${GREEN}[OK]${NC} Cargo config file exists"
    else
      echo -e "${YELLOW}[WARNING]${NC} Cargo config file does not exist"
      echo -e "  ${MAGENTA}Recommendation:${NC} Create a config.toml file with build optimizations"
      ((issues++))
    fi
  else
    echo -e "${YELLOW}[WARNING]${NC} Cargo configuration directory does not exist"
    echo -e "  ${MAGENTA}Recommendation:${NC} Create .cargo directory with config.toml for build optimizations"
    ((issues++))
  fi
  
  return $issues
}

# Run all checks
app_dir_issues=0
lib_rs_issues=0
app_export_issues=0
old_structure_issues=0
import_issues=0
naming_issues=0
ide_issues=0
build_issues=0

check_app_directories
app_dir_issues=$?

check_lib_rs_exports
lib_rs_issues=$?

check_app_exports
app_export_issues=$?

check_old_src_structure
old_structure_issues=$?

check_import_patterns
import_issues=$?

check_naming_conventions
naming_issues=$?

check_ide_config
ide_issues=$?

check_build_config
build_issues=$?

total_issues=$((app_dir_issues + lib_rs_issues + app_export_issues + old_structure_issues + import_issues + naming_issues + ide_issues + build_issues))

echo -e "\n${CYAN}=== Verification Summary ===${NC}"
if [ $total_issues -eq 0 ]; then
  echo -e "${GREEN}All checks passed! The project structure is in compliance with the standards.${NC}"
  echo -e "${BLUE}Note:${NC} The structure verification does not check the content quality, only the organizational patterns."
  exit 0
else
  echo -e "${YELLOW}Found $total_issues structural issues that need attention.${NC}"
  echo -e "${BLUE}Please review the detailed recommendations above and consult:${NC}"
  echo -e "  - docs/roadmaps/11_project_structure_future_improvements.md"
  echo -e "  - docs/guides/project-structure-cheatsheet.md"
  echo -e "  - docs/architecture/diagrams/app-module-diagram.md"
  echo -e "  - docs/architecture/diagrams/core-module-diagram.md"
  
  # Show summary by category
  echo -e "\n${CYAN}Issues by Category:${NC}"
  echo -e "  App Directories: $app_dir_issues"
  echo -e "  Lib.rs Exports: $lib_rs_issues"
  echo -e "  App Exports: $app_export_issues"
  echo -e "  Old Structure: $old_structure_issues"
  echo -e "  Import Patterns: $import_issues"
  echo -e "  Naming Conventions: $naming_issues"
  echo -e "  IDE Configuration: $ide_issues"
  echo -e "  Build Configuration: $build_issues"
  
  exit 1
fi 
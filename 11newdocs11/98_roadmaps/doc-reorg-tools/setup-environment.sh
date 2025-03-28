#!/bin/bash

# setup-environment.sh - Prepares the environment for documentation tools
#
# This script creates the necessary directory structure and ensures
# all dependencies are ready for the documentation tools to run.

# Script directory - used for relative path resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="/Users/goblin/dev/git/navius"
VERBOSE=false

# Process command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose)
      VERBOSE=true
      shift
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --verbose    Show detailed progress information"
      echo "  --help       Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Function to log messages
log() {
  if [[ "${VERBOSE}" == "true" ]]; then
    echo "$1"
  fi
}

# Function to create directory if it doesn't exist
ensure_directory() {
  local dir="$1"
  local description="$2"
  
  if [[ ! -d "$dir" ]]; then
    log "Creating $description directory: $dir"
    mkdir -p "$dir"
  else
    log "$description directory already exists: $dir"
  fi
}

# Function to make scripts executable
make_executable() {
  local script="$1"
  
  if [[ -f "$script" ]]; then
    log "Making executable: $script"
    chmod +x "$script"
  else
    log "Script not found: $script"
  fi
}

# Create necessary directories
create_directories() {
  # Main directories
  ensure_directory "${SCRIPT_DIR}/logs" "Logs"
  ensure_directory "${SCRIPT_DIR}/reports" "Reports"
  ensure_directory "${SCRIPT_DIR}/templates" "Templates"
  ensure_directory "${SCRIPT_DIR}/data" "Data"
  ensure_directory "${SCRIPT_DIR}/backups" "Backups"
  
  # Subdirectories
  ensure_directory "${SCRIPT_DIR}/logs/daily" "Daily logs"
  ensure_directory "${SCRIPT_DIR}/reports/daily" "Daily reports"
  ensure_directory "${SCRIPT_DIR}/backups/$(date +%Y%m%d)" "Today's backups"
}

# Check and make scripts executable
ensure_executables() {
  log "Checking scripts and making them executable..."
  
  # Get all bash scripts in the directory
  local scripts=("${SCRIPT_DIR}"/*.sh)
  
  for script in "${scripts[@]}"; do
    make_executable "$script"
  done
}

# Create template files if they don't exist
create_templates() {
  local frontmatter_template="${SCRIPT_DIR}/templates/frontmatter.md"
  
  if [[ ! -f "$frontmatter_template" ]]; then
    log "Creating frontmatter template"
    cat > "$frontmatter_template" << 'EOF'
---
title: "Document Title"
description: "Brief description of this document"
created_at: "YYYY-MM-DD"
updated_at: "YYYY-MM-DD"
tags: ["tag1", "tag2"]
---
EOF
  fi
}

# Create .gitignore file to exclude logs and reports
create_gitignore() {
  local gitignore="${SCRIPT_DIR}/.gitignore"
  
  if [[ ! -f "$gitignore" ]]; then
    log "Creating .gitignore file"
    cat > "$gitignore" << 'EOF'
# Ignore logs
logs/*
!logs/.gitkeep

# Ignore reports
reports/*
!reports/.gitkeep

# Ignore backups
backups/*
!backups/.gitkeep

# Ignore data files
data/*.json
data/*.csv
!data/.gitkeep

# Keep directory structure
!*/
EOF
  fi
  
  # Create .gitkeep files to preserve directory structure
  touch "${SCRIPT_DIR}/logs/.gitkeep"
  touch "${SCRIPT_DIR}/reports/.gitkeep"
  touch "${SCRIPT_DIR}/templates/.gitkeep"
  touch "${SCRIPT_DIR}/data/.gitkeep"
  touch "${SCRIPT_DIR}/backups/.gitkeep"
}

# Verify that all tools are available
verify_tools() {
  log "Verifying required tools are available..."
  
  local missing_tools=()
  
  # Check for core tools
  for tool in grep sed awk find xargs; do
    if ! command -v $tool &> /dev/null; then
      missing_tools+=("$tool")
    fi
  done
  
  # Report missing tools
  if [[ ${#missing_tools[@]} -gt 0 ]]; then
    echo "WARNING: The following required tools are missing:"
    for tool in "${missing_tools[@]}"; do
      echo "  - $tool"
    done
    echo "Please install these tools before running the scripts."
  else
    log "All required tools are available."
  fi
}

# Create known-issues.md file if it doesn't exist
create_known_issues() {
  local known_issues="${SCRIPT_DIR}/data/known-issues.md"
  
  if [[ ! -f "$known_issues" ]]; then
    log "Creating known issues file"
    cat > "$known_issues" << 'EOF'
# Known Issues

This file tracks known issues with documentation and link fixing tools that are being addressed.

## Link Fixing

- **External links** - The tools do not validate external links (http/https)
- **Case sensitivity** - Some filesystems are case insensitive, causing issues with link validation
- **Special characters** - Links with special characters may not be properly handled

## Frontmatter

- **Date formats** - Inconsistent date formats in frontmatter are not fully standardized
- **Required fields** - Not all required fields are enforced in all documents

## Sections

- **Inconsistent headings** - Heading levels may vary between documents
- **Missing sections** - Some required sections may be missing in legacy documents

## Action Items

- [ ] Add support for validating external links
- [ ] Improve case-sensitivity handling
- [ ] Standardize date formats in frontmatter
- [ ] Enforce required fields in frontmatter
- [ ] Standardize heading levels
EOF
  fi
}

# Main execution
echo "Setting up documentation tool environment..."
create_directories
ensure_executables
create_templates
create_gitignore
verify_tools
create_known_issues
echo "Environment setup complete. Documentation tools are ready to use."

exit 0 
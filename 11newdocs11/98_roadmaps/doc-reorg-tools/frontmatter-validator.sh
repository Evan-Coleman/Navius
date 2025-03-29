#!/bin/bash
# frontmatter-validator.sh
# Script to validate frontmatter in markdown files
# Created: April 3, 2025

set -e

# Configuration
SCRIPT_DIR="$(dirname "$0")"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${LOGS_DIR}/frontmatter_validation_${TIMESTAMP}.log"
REQUIRED_FIELDS=("title" "description" "category" "tags" "last_updated")
VERBOSE=false
FIX_MODE=false
PROCESSED_FILES=0
VALID_FILES=0
MISSING_FRONTMATTER=0
INVALID_FRONTMATTER=0
FIXED_FILES=0
DUPLICATE_FRONTMATTER=0

# Create logs directory if it doesn't exist
mkdir -p "${LOGS_DIR}"

# Display help message
show_help() {
  echo "Usage: $0 [options] [--dir <directory> | --file <file>]"
  echo
  echo "Options:"
  echo "  --dir <directory>   Validate frontmatter in all markdown files in the specified directory (recursive)"
  echo "  --file <file>       Validate frontmatter in the specified markdown file"
  echo "  --fix               Attempt to fix missing frontmatter (adds basic template)"
  echo "  --verbose           Display detailed output"
  echo "  --help              Display this help message"
  echo
  echo "Example:"
  echo "  $0 --dir ../04_guides/deployment --fix --verbose"
  exit 0
}

# Log messages
log() {
  local level="$1"
  local message="$2"
  
  # Format timestamp
  local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
  
  # Print to console if verbose mode is enabled or level is not DEBUG
  if [[ "$VERBOSE" == "true" ]] || [[ "$level" != "DEBUG" ]]; then
    echo "[$level] $message"
  fi
  
  # Log to file
  echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# Check if a file has valid frontmatter
check_frontmatter() {
  local file="$1"
  
  # Check if file exists
  if [[ ! -f "$file" ]]; then
    log "ERROR" "File not found: $file"
    return 1
  fi
  
  # Check if file has frontmatter (delimited by ---)
  # We need to check if it has the opening AND closing ---
  local frontmatter_count=$(grep -c "^---$" "$file" | head -2)
  
  if [[ "$frontmatter_count" -lt 2 ]]; then
    log "WARNING" "Missing frontmatter in $file"
    ((MISSING_FRONTMATTER++))
    return 1
  fi
  
  # Check if there are duplicate frontmatter blocks
  if [[ "$frontmatter_count" -gt 2 ]]; then
    log "WARNING" "Duplicate frontmatter detected in $file"
    ((DUPLICATE_FRONTMATTER++))
    return 2
  fi
  
  # Extract frontmatter content
  local frontmatter=$(sed -n '/^---$/,/^---$/p' "$file")
  
  # Check required fields
  local invalid_fields=0
  for field in "${REQUIRED_FIELDS[@]}"; do
    if ! echo "$frontmatter" | grep -q "^$field:"; then
      log "WARNING" "Missing '$field' in frontmatter of $file"
      ((invalid_fields++))
    fi
  done
  
  if [[ "$invalid_fields" -gt 0 ]]; then
    log "WARNING" "Invalid frontmatter in $file (missing $invalid_fields required fields)"
    ((INVALID_FRONTMATTER++))
    return 3
  fi
  
  log "DEBUG" "Valid frontmatter in $file"
  ((VALID_FILES++))
  return 0
}

# Validate frontmatter in a file
validate_frontmatter() {
  local file="$1"
  
  log "DEBUG" "Validating frontmatter in $file"
  
  # Check if the file is a markdown file
  if [[ "$file" != *.md ]]; then
    log "DEBUG" "Skipping non-markdown file: $file"
    return 0
  fi
  
  # Check frontmatter status
  check_frontmatter "$file"
  local status=$?
  
  # If fix mode is enabled and frontmatter is missing or invalid, fix it
  if [[ "$FIX_MODE" == "true" ]]; then
    if [[ "$status" -eq 1 ]]; then
      # Missing frontmatter
      add_frontmatter "$file"
    elif [[ "$status" -eq 2 ]]; then
      # Duplicate frontmatter
      fix_duplicate_frontmatter "$file"
    elif [[ "$status" -eq 3 ]]; then
      # Invalid frontmatter (missing fields)
      fix_missing_fields "$file"
    fi
  fi
  
  ((PROCESSED_FILES++))
}

# Fix duplicate frontmatter in a file
fix_duplicate_frontmatter() {
  local file="$1"
  log "INFO" "Fixing duplicate frontmatter in $file"
  
  # Create a temporary file
  local temp_file=$(mktemp)
  
  # Using awk to remove the first frontmatter block
  awk '
  BEGIN { count = 0; printing = 1; }
  /^---$/ {
    count++;
    if (count == 1) {
      printing = 0;
      next;
    }
    if (count == 2) {
      printing = 1;
    }
    if (count >= 3) {
      printing = 1;
    }
  }
  { if (printing) print; }
  ' "$file" > "$temp_file"
  
  # Replace the original file with the fixed content
  mv "$temp_file" "$file"
  
  log "INFO" "Fixed duplicate frontmatter in $file"
  ((FIXED_FILES++))
}

# Add frontmatter to a file
add_frontmatter() {
  local file="$1"
  
  log "INFO" "Adding frontmatter to $file"
  
  # Get the base name of the file without path and extension
  local file_name=$(basename "$file" .md)
  
  # Convert to title case
  local title=$(echo "$file_name" | tr '-_' ' ' | awk '{for(i=1;i<=NF;i++){ $i=toupper(substr($i,1,1)) substr($i,2) }}1')
  
  # Create a temporary file
  local temp_file=$(mktemp)
  
  # Add frontmatter template to the temporary file
  cat > "$temp_file" << EOL
---
title: "$title"
description: ""
category: "Documentation"
tags: []
last_updated: "$(date +"%B %d, %Y")"
version: "1.0"
---

EOL
  
  # Append the original content
  cat "$file" >> "$temp_file"
  
  # Replace the original file with the fixed content
  mv "$temp_file" "$file"
  
  log "INFO" "Added frontmatter template to $file"
  ((FIXED_FILES++))
}

# Fix missing fields in frontmatter
fix_missing_fields() {
  local file="$1"
  
  log "INFO" "Fixing missing fields in frontmatter of $file"
  
  # Extract frontmatter content
  local frontmatter=$(sed -n '/^---$/,/^---$/p' "$file" | sed '1d;$d')
  
  # Get file content excluding frontmatter
  local content=$(sed '1,/^---$/d' "$file")
  
  # Create a temporary file
  local temp_file=$(mktemp)
  
  # Add fixed frontmatter
  echo "---" > "$temp_file"
  
  # Add existing fields
  for field in "title" "description" "category" "tags" "last_updated" "version"; do
    if echo "$frontmatter" | grep -q "^$field:"; then
      grep "^$field:" <<< "$frontmatter" >> "$temp_file"
    else
      # Add missing field with default value
      case "$field" in
        "title")
          # Get title from file name
          local file_name=$(basename "$file" .md)
          local title=$(echo "$file_name" | tr '-_' ' ' | awk '{for(i=1;i<=NF;i++){ $i=toupper(substr($i,1,1)) substr($i,2) }}1')
          echo "title: \"$title\"" >> "$temp_file"
          ;;
        "description")
          echo "description: \"\"" >> "$temp_file"
          ;;
        "category")
          echo "category: \"Documentation\"" >> "$temp_file"
          ;;
        "tags")
          echo "tags: []" >> "$temp_file"
          ;;
        "last_updated")
          echo "last_updated: \"$(date +"%B %d, %Y")\"" >> "$temp_file"
          ;;
        "version")
          echo "version: \"1.0\"" >> "$temp_file"
          ;;
      esac
    fi
  done
  
  # Close frontmatter
  echo "---" >> "$temp_file"
  
  # Add content
  echo "$content" >> "$temp_file"
  
  # Replace the original file
  mv "$temp_file" "$file"
  
  log "INFO" "Fixed missing fields in frontmatter of $file"
  ((FIXED_FILES++))
}

# Process a directory
process_directory() {
  local dir="$1"
  
  log "INFO" "Processing directory: $dir"
  
  # Find all markdown files in the directory
  local files=$(find "$dir" -type f -name "*.md" 2>/dev/null)
  
  # Process each file
  for file in $files; do
    validate_frontmatter "$file"
  done
  
  log "INFO" "Processed $PROCESSED_FILES markdown files in $dir"
  log "INFO" "Valid frontmatter: $VALID_FILES / $PROCESSED_FILES"
}

# Generate summary report
generate_summary() {
  log "INFO" "========== Frontmatter Validation Summary =========="
  log "INFO" "Total files processed: $PROCESSED_FILES"
  log "INFO" "Files with missing frontmatter: $MISSING_FRONTMATTER"
  log "INFO" "Files with invalid frontmatter: $INVALID_FRONTMATTER"
  log "INFO" "Files with duplicate frontmatter: $DUPLICATE_FRONTMATTER"
  log "INFO" "Files fixed: $FIXED_FILES"
  log "INFO" "Valid files: $VALID_FILES"
  
  # Calculate compliance rate
  local compliance=0
  if [[ "$PROCESSED_FILES" -gt 0 ]]; then
    compliance=$(( (VALID_FILES * 100) / PROCESSED_FILES ))
  fi
  
  log "INFO" "Compliance rate: $compliance%"
  log "INFO" "Log file: $LOG_FILE"
  log "INFO" "================================================="
}

# Parse command line arguments
if [[ $# -eq 0 ]]; then
  show_help
fi

TARGET_DIR=""
TARGET_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help)
      show_help
      ;;
    --dir)
      TARGET_DIR="$2"
      shift 2
      ;;
    --file)
      TARGET_FILE="$2"
      shift 2
      ;;
    --fix)
      FIX_MODE=true
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      ;;
  esac
done

# Start validation
log "INFO" "Starting frontmatter validation"

if [[ -n "$TARGET_DIR" ]]; then
  if [[ ! -d "$TARGET_DIR" ]]; then
    log "ERROR" "Directory not found: $TARGET_DIR"
    exit 1
  fi
  
  process_directory "$TARGET_DIR"
elif [[ -n "$TARGET_FILE" ]]; then
  if [[ ! -f "$TARGET_FILE" ]]; then
    log "ERROR" "File not found: $TARGET_FILE"
    exit 1
  fi
  
  validate_frontmatter "$TARGET_FILE"
else
  log "ERROR" "No target specified. Use --dir or --file option."
  show_help
fi

# Generate summary
generate_summary

exit 0 
#!/bin/bash
# frontmatter-validator.sh
# Script to validate frontmatter in markdown files
# Created: March 31, 2025

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${LOGS_DIR}/frontmatter_validation_${TIMESTAMP}.log"
REQUIRED_FIELDS=("title" "description" "category" "tags" "last_updated")
VERBOSE=false
MISSING_COUNT=0
INVALID_COUNT=0
FIXED_COUNT=0
TOTAL_FILES=0

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
  local message="$1"
  local level="${2:-INFO}"
  local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
  echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
  
  if [[ "$VERBOSE" == true || "$level" != "DEBUG" ]]; then
    echo "[$level] $message"
  fi
}

# Check if a file has frontmatter
has_frontmatter() {
  local file="$1"
  
  # Check if file starts with --- (frontmatter delimiter)
  if grep -q "^---" "$file"; then
    # Check if it has a second --- delimiter
    if grep -q "^---" "$file" | wc -l | grep -q "2"; then
      return 0 # Has frontmatter
    fi
  fi
  
  return 1 # No frontmatter
}

# Validate frontmatter in a markdown file
validate_frontmatter() {
  local file="$1"
  local missing_fields=()
  local fix_mode="${2:-false}"
  local has_issues=false
  
  TOTAL_FILES=$((TOTAL_FILES + 1))
  
  log "Validating frontmatter in $file" "DEBUG"
  
  # Check if file has frontmatter
  if ! has_frontmatter "$file"; then
    log "Missing frontmatter in $file" "WARNING"
    MISSING_COUNT=$((MISSING_COUNT + 1))
    has_issues=true
    
    if [[ "$fix_mode" == true ]]; then
      add_frontmatter "$file"
    fi
    
    return 1
  fi
  
  # Check for required fields
  for field in "${REQUIRED_FIELDS[@]}"; do
    if ! grep -q "^$field:" "$file"; then
      missing_fields+=("$field")
      has_issues=true
    fi
  done
  
  if [[ ${#missing_fields[@]} -gt 0 ]]; then
    log "Missing required fields in $file: ${missing_fields[*]}" "WARNING"
    INVALID_COUNT=$((INVALID_COUNT + 1))
    
    if [[ "$fix_mode" == true ]]; then
      fix_missing_fields "$file" "${missing_fields[@]}"
    fi
    
    return 1
  fi
  
  log "Frontmatter valid in $file" "DEBUG"
  return 0
}

# Add basic frontmatter to a file
add_frontmatter() {
  local file="$1"
  local filename=$(basename "$file" .md)
  local title=$(echo "$filename" | sed 's/-/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
  local current_date=$(date +"%B %d, %Y")
  local file_content=$(cat "$file")
  
  log "Adding frontmatter to $file" "INFO"
  
  # Create frontmatter
  cat > "$file" << EOF
---
title: "$title"
description: ""
category: "Documentation"
tags: []
last_updated: "$current_date"
version: "1.0"
---

$file_content
EOF

  log "Added frontmatter template to $file" "INFO"
  FIXED_COUNT=$((FIXED_COUNT + 1))
}

# Fix missing fields in frontmatter
fix_missing_fields() {
  local file="$1"
  shift
  local missing_fields=("$@")
  local current_date=$(date +"%B %d, %Y")
  local filename=$(basename "$file" .md)
  local title=$(echo "$filename" | sed 's/-/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
  local temp_file="${file}.temp"
  
  log "Fixing missing fields in $file: ${missing_fields[*]}" "INFO"
  
  # Read the file into a variable
  local content=$(cat "$file")
  
  # Find where frontmatter ends
  local frontmatter_end=$(grep -n "^---" "$file" | awk 'NR==2 {print $1}' | cut -d: -f1)
  
  # Create temporary file
  echo "$content" > "$temp_file"
  
  # Add missing fields
  for field in "${missing_fields[@]}"; do
    case "$field" in
      "title")
        sed -i "${frontmatter_end}i title: \"$title\"" "$temp_file"
        ;;
      "description")
        sed -i "${frontmatter_end}i description: \"\"" "$temp_file"
        ;;
      "category")
        sed -i "${frontmatter_end}i category: \"Documentation\"" "$temp_file"
        ;;
      "tags")
        sed -i "${frontmatter_end}i tags: []" "$temp_file"
        ;;
      "last_updated")
        sed -i "${frontmatter_end}i last_updated: \"$current_date\"" "$temp_file"
        ;;
      "version")
        sed -i "${frontmatter_end}i version: \"1.0\"" "$temp_file"
        ;;
    esac
  done
  
  # Replace original file with the modified one
  mv "$temp_file" "$file"
  
  log "Fixed missing fields in $file" "INFO"
  FIXED_COUNT=$((FIXED_COUNT + 1))
}

# Process a directory
process_directory() {
  local dir="$1"
  local fix_mode="$2"
  local file_count=0
  local valid_count=0
  
  log "Processing directory: $dir" "INFO"
  
  # Find all markdown files in the directory
  while IFS= read -r -d '' file; do
    file_count=$((file_count + 1))
    validate_frontmatter "$file" "$fix_mode" && valid_count=$((valid_count + 1))
  done < <(find "$dir" -type f -name "*.md" -print0)
  
  log "Processed $file_count markdown files in $dir" "INFO"
  log "Valid frontmatter: $valid_count / $file_count" "INFO"
}

# Generate summary report
generate_summary() {
  log "========== Frontmatter Validation Summary ==========" "INFO"
  log "Total files processed: $TOTAL_FILES" "INFO"
  log "Files with missing frontmatter: $MISSING_COUNT" "INFO"
  log "Files with invalid frontmatter: $INVALID_COUNT" "INFO"
  log "Files fixed: $FIXED_COUNT" "INFO"
  log "Valid files: $((TOTAL_FILES - MISSING_COUNT - INVALID_COUNT + FIXED_COUNT))" "INFO"
  log "Compliance rate: $(( (TOTAL_FILES - MISSING_COUNT - INVALID_COUNT + FIXED_COUNT) * 100 / TOTAL_FILES ))%" "INFO"
  log "Log file: $LOG_FILE" "INFO"
  log "=================================================" "INFO"
}

# Parse command line arguments
if [[ $# -eq 0 ]]; then
  show_help
fi

FIX_MODE=false
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
log "Starting frontmatter validation" "INFO"

if [[ -n "$TARGET_DIR" ]]; then
  if [[ ! -d "$TARGET_DIR" ]]; then
    log "Directory not found: $TARGET_DIR" "ERROR"
    exit 1
  fi
  
  process_directory "$TARGET_DIR" "$FIX_MODE"
elif [[ -n "$TARGET_FILE" ]]; then
  if [[ ! -f "$TARGET_FILE" ]]; then
    log "File not found: $TARGET_FILE" "ERROR"
    exit 1
  fi
  
  validate_frontmatter "$TARGET_FILE" "$FIX_MODE"
else
  log "No target specified. Use --dir or --file option." "ERROR"
  show_help
fi

# Generate summary
generate_summary

exit 0 
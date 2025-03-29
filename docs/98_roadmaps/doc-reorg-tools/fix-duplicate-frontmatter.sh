#!/bin/bash

# fix-duplicate-frontmatter.sh - Created April 3, 2025
# Script to find and fix duplicate frontmatter in markdown files

set -e

SCRIPT_DIR="$(dirname "$0")"
LOGS_DIR="${SCRIPT_DIR}/logs"
LOG_FILE="${LOGS_DIR}/duplicate_frontmatter_fixes_$(date +%Y%m%d_%H%M%S).log"
VERBOSE=false
DRY_RUN=false
PROCESSED_FILES=0
FIXED_FILES=0

# Create logs directory if it doesn't exist
mkdir -p "${LOGS_DIR}"

# Display help message
show_help() {
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  --dir DIR          Directory to scan for files with duplicate frontmatter"
  echo "  --file FILE        Single file to fix"
  echo "  --dry-run          Show what would be fixed without making changes"
  echo "  --verbose          Display detailed output"
  echo "  --help             Display this help message"
}

# Logging function
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

# Check if a file has duplicate frontmatter
has_duplicate_frontmatter() {
  local file="$1"
  
  # Count the number of frontmatter delimiters (---)
  local frontmatter_count=$(grep -c "^---$" "$file")
  
  # If there are more than 2 delimiters, there's duplicate frontmatter
  if [[ "$frontmatter_count" -gt 2 ]]; then
    return 0 # true
  else
    return 1 # false
  fi
}

# Fix duplicate frontmatter in a file
fix_duplicate_frontmatter() {
  local file="$1"
  log "INFO" "Fixing duplicate frontmatter in $file"
  
  # Create a temporary file
  local temp_file=$(mktemp)
  
  # Get the line number of the third --- delimiter
  local third_delimiter=$(grep -n "^---$" "$file" | awk 'NR==3 {print $1}' | cut -d: -f1)
  
  # Keep only the content after the second frontmatter block
  awk -v third=$third_delimiter 'NR >= third - 2 {print}' "$file" > "$temp_file"
  
  if [[ "$DRY_RUN" == "true" ]]; then
    log "INFO" "Would fix duplicate frontmatter in $file (dry run)"
    rm "$temp_file"
  else
    # Replace the original file with the fixed content
    mv "$temp_file" "$file"
    log "INFO" "Fixed duplicate frontmatter in $file"
    ((FIXED_FILES++))
  fi
}

# Process a file
process_file() {
  local file="$1"
  ((PROCESSED_FILES++))
  
  log "DEBUG" "Checking file: $file"
  
  # Check if file has duplicate frontmatter
  if has_duplicate_frontmatter "$file"; then
    fix_duplicate_frontmatter "$file"
  else
    log "DEBUG" "No duplicate frontmatter in $file"
  fi
}

# Process a directory recursively
process_directory() {
  local dir="$1"
  
  log "INFO" "Processing directory: $dir"
  
  # Find all markdown files in the directory
  while IFS= read -r file; do
    if [[ -f "$file" ]]; then
      process_file "$file"
    fi
  done < <(find "$dir" -type f -name "*.md" 2>/dev/null)
  
  log "INFO" "Processed $PROCESSED_FILES files in $dir"
  log "INFO" "Fixed $FIXED_FILES files with duplicate frontmatter"
}

# Generate a summary report
generate_summary() {
  log "INFO" "========== Duplicate Frontmatter Fix Summary =========="
  log "INFO" "Total files processed: $PROCESSED_FILES"
  log "INFO" "Files fixed: $FIXED_FILES"
  log "INFO" "Log file: $LOG_FILE"
  log "INFO" "================================================="
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  
  case $key in
    --dir)
      DIR="$2"
      shift 2
      ;;
    --file)
      FILE="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --help)
      show_help
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

# Main execution
log "INFO" "Starting duplicate frontmatter fix"

if [[ "$DRY_RUN" == "true" ]]; then
  log "INFO" "Running in dry-run mode. No changes will be made."
fi

# Check if directory or file is specified
if [[ -n "$DIR" ]]; then
  if [[ ! -d "$DIR" ]]; then
    log "ERROR" "Directory not found: $DIR"
    exit 1
  fi
  
  process_directory "$DIR"
elif [[ -n "$FILE" ]]; then
  if [[ ! -f "$FILE" ]]; then
    log "ERROR" "File not found: $FILE"
    exit 1
  fi
  
  process_file "$FILE"
else
  log "ERROR" "No directory or file specified. Use --dir or --file option."
  show_help
  exit 1
fi

# Generate summary report
generate_summary

exit 0 
#!/bin/bash

# check-markdown-codeblocks.sh - Created April 4, 2025
# Script to check for syntax errors in markdown code blocks without fixing them

set -e

# Configuration
SCRIPT_DIR="$(dirname "$0")"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${LOGS_DIR}/codeblock_check_${TIMESTAMP}.log"
VERBOSE=false

# Create logs directory if it doesn't exist
mkdir -p "${LOGS_DIR}"

# Stats
PROCESSED_FILES=0
FILES_WITH_ISSUES=0
TOTAL_ISSUES=0

# Display help message
show_help() {
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  --dir DIR       Directory to scan for markdown files (recursive)"
  echo "  --file FILE     Check a single markdown file"
  echo "  --verbose       Display detailed output"
  echo "  --help          Display this help message"
  echo ""
  echo "This script checks markdown files for code block syntax issues without fixing them."
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

# Check a file for code block issues
check_file() {
  local file="$1"
  local file_issues=0
  
  log "DEBUG" "Checking ${file}"
  
  # Check if the file exists
  if [[ ! -f "$file" ]]; then
    log "ERROR" "File not found: $file"
    return 1
  fi
  
  # Check for code blocks with language identifiers after the closing backticks
  local closing_with_lang=$(grep -n "^```[a-zA-Z0-9_]*$" "$file" || true)
  if [[ -n "$closing_with_lang" ]]; then
    log "WARNING" "Found code block with language identifier after closing backticks in $file"
    log "DEBUG" "$closing_with_lang"
    ((file_issues++))
  fi
  
  # Check for mismatched opening/closing backticks
  local opening_count=$(grep -c "^```[a-zA-Z0-9_]*$" "$file" || true)
  local closing_count=$(grep -c "^```$" "$file" || true)
  
  if [[ "$opening_count" != "$closing_count" ]]; then
    log "WARNING" "Mismatched code block backticks in $file (opening: $opening_count, closing: $closing_count)"
    ((file_issues++))
  fi
  
  # Check for empty code blocks
  local empty_blocks=$(awk '
    /^```[a-zA-Z0-9_]*$/ {
      start = NR
      lang = $0
    }
    /^```$/ {
      if (NR == start + 1) {
        print "Empty code block at line " start
      }
    }
  ' "$file" || true)
  
  if [[ -n "$empty_blocks" ]]; then
    log "WARNING" "Found empty code blocks in $file"
    log "DEBUG" "$empty_blocks"
    ((file_issues++))
  fi
  
  # Check for code blocks without language identifiers
  local no_lang_blocks=$(grep -n "^```$" "$file" | grep -B1 -A1 "^```$" || true)
  if [[ -n "$no_lang_blocks" ]]; then
    log "WARNING" "Found code blocks without language identifiers in $file"
    log "DEBUG" "$no_lang_blocks"
    ((file_issues++))
  fi
  
  # Update statistics
  if [[ "$file_issues" -gt 0 ]]; then
    log "WARNING" "Found $file_issues issues in $file"
    ((FILES_WITH_ISSUES++))
    ((TOTAL_ISSUES += file_issues))
  else
    log "DEBUG" "No issues found in $file"
  fi
  
  ((PROCESSED_FILES++))
}

# Process a directory
process_directory() {
  local dir="$1"
  
  log "INFO" "Processing directory: $dir"
  
  # Find all markdown files in the directory and its subdirectories
  find "$dir" -type f -name "*.md" | while read -r file; do
    check_file "$file"
  done
}

# Generate summary
generate_summary() {
  log "INFO" "========== Code Block Check Summary =========="
  log "INFO" "Processed $PROCESSED_FILES files"
  log "INFO" "Found $FILES_WITH_ISSUES files with issues"
  log "INFO" "Found $TOTAL_ISSUES code block issues"
  
  # Calculate compliance rate
  local compliance=100
  if [[ "$PROCESSED_FILES" -gt 0 ]]; then
    compliance=$(( 100 - (FILES_WITH_ISSUES * 100 / PROCESSED_FILES) ))
  fi
  
  log "INFO" "Compliance rate: $compliance%"
  log "INFO" "Log file: $LOG_FILE"
  log "INFO" "================================================="
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  
  case $key in
    --help)
      show_help
      exit 0
      ;;
    --dir)
      DIR="$2"
      shift 2
      ;;
    --file)
      FILE="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

# Main execution
log "INFO" "Starting markdown code block check"

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
  
  check_file "$FILE"
else
  log "ERROR" "No directory or file specified. Use --dir or --file option."
  show_help
  exit 1
fi

# Generate summary
generate_summary

exit 0 
#!/bin/bash

# Script to fix incorrect markdown code block syntax
# This script identifies and fixes markdown files with code blocks that have language markers
# at both the beginning and end (which is incorrect markdown syntax)

# Find files with potential issues (look for closing backticks with language markers)
find_affected_files() {
  grep -l "^\`\`\`[a-z]" "$1" --include="*.md" -r
}

# Fix a single file
fix_file() {
  local file="$1"
  echo "Fixing file: $file"
  
  # Create a backup
  cp "$file" "${file}.bak"
  
  # Replace closing code blocks with just backticks
  # This uses sed to find lines that only contain backticks followed by a language marker
  sed -i '' 's/^```[a-z][a-z]*$/```/g' "$file"
  
  echo "Fixed file: $file"
}

# Count the number of fixes made in a file
count_fixes() {
  local file="$1"
  local original_file="${file}.bak"
  local diff_count=$(diff "$file" "$original_file" | grep -c "^\<")
  echo "$diff_count fixes in $file"
  return $diff_count
}

# Main function
main() {
  local directory="${1:-11newdocs11}"
  local log_file="markdown_fixes_$(date +%Y%m%d_%H%M%S).log"
  local dry_run=false
  local total_fixes=0
  local total_files=0
  
  # Process arguments
  while [[ "$#" -gt 0 ]]; do
    case $1 in
      --dry-run) dry_run=true; shift ;;
      --dir) directory="$2"; shift 2 ;;
      *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
  done
  
  echo "Searching for affected files in $directory..."
  local affected_files=$(find_affected_files "$directory")
  
  if [ -z "$affected_files" ]; then
    echo "No affected files found."
    exit 0
  fi
  
  # Create log directory
  mkdir -p logs
  
  echo "Found potentially affected files:" | tee -a "logs/$log_file"
  echo "$affected_files" | tee -a "logs/$log_file"
  echo "" | tee -a "logs/$log_file"
  
  if [ "$dry_run" = true ]; then
    echo "DRY RUN: No changes will be made." | tee -a "logs/$log_file"
    exit 0
  fi
  
  echo "Fixing files..." | tee -a "logs/$log_file"
  for file in $affected_files; do
    fix_file "$file"
    
    # Count fixes
    count_fixes "$file"
    fixes=$?
    
    if [ $fixes -gt 0 ]; then
      echo "$fixes incorrect code blocks fixed in $file" | tee -a "logs/$log_file"
      total_fixes=$((total_fixes + fixes))
      total_files=$((total_files + 1))
    else
      echo "No fixes needed in $file (false positive)" | tee -a "logs/$log_file" 
      # Restore the original if no changes were made
      mv "${file}.bak" "$file"
    fi
  done
  
  echo "===== Fix Summary =====" | tee -a "logs/$log_file"
  echo "Total files fixed: $total_files" | tee -a "logs/$log_file"
  echo "Total code blocks fixed: $total_fixes" | tee -a "logs/$log_file"
  echo "Log saved to: logs/$log_file" | tee -a "logs/$log_file"
}

# Run the script with the provided arguments
main "$@" 
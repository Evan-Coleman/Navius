#!/bin/bash

# verify-frontmatter.sh
# A script to check markdown files for proper frontmatter and generate a report of any issues found
# Can operate on individual files or entire directories

set -o errexit
set -o nounset
set -o pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORTS_DIR="${SCRIPT_DIR}/reports"
BACKUPS_DIR="${SCRIPT_DIR}/backups"

# Create reports and backups directories if they don't exist
mkdir -p "${REPORTS_DIR}"
mkdir -p "${BACKUPS_DIR}"

# Default values
VERBOSE=false
DRY_RUN=false
RECURSIVE=false
FIX_ISSUES=false
TARGET_DIR=""
TARGET_FILE=""
OUTPUT_FILE=""

# Required frontmatter fields
REQUIRED_FIELDS=("title" "description" "category" "tags" "last_updated")

# Function to display usage information
function show_usage {
  cat << EOF
Usage: $(basename $0) [OPTIONS]

Verify markdown frontmatter and generate a report.

Options:
  --dir DIR             Directory containing markdown files to verify
  --file FILE           Single markdown file to verify
  --recursive           Process directories recursively
  --fix                 Automatically fix issues (adds missing fields)
  --dry-run             Show what would be fixed without making changes
  --output FILE         Output report to this file
  --verbose             Display detailed progress information
  --help                Display this help message

Examples:
  $(basename $0) --dir ./docs --recursive --output ./reports/frontmatter-report.md
  $(basename $0) --file ./docs/getting-started.md --fix
EOF
}

# Function to log messages
log_message() {
  if [[ "${VERBOSE}" == "true" ]] || [[ "$1" == "ERROR:"* ]]; then
    echo "$1"
  fi
}

# Function to check if a file has frontmatter
has_frontmatter() {
  local file="$1"
  grep -q "^---$" "$file" && grep -q "^---$" "$file" -m 2
  return $?
}

# Function to extract a field from frontmatter
get_frontmatter_field() {
  local file="$1"
  local field="$2"
  
  # Extract content between first two --- markers and grep for the field
  sed -n '/^---$/,/^---$/p' "$file" | grep "^$field:" | sed "s/^$field:[[:space:]]*//;s/\"//g"
}

# Function to check if a field exists in frontmatter
field_exists() {
  local file="$1"
  local field="$2"
  
  sed -n '/^---$/,/^---$/p' "$file" | grep -q "^$field:"
  return $?
}

# Function to validate date format (YYYY-MM-DD)
validate_date_format() {
  local date_string="$1"
  [[ "$date_string" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]
  return $?
}

# Function to validate tags format (should be an array)
validate_tags_format() {
  local tags_string="$1"
  [[ "$tags_string" =~ ^\[.*\]$ ]] || [[ "$tags_string" =~ ^-\ .*$ ]]
  return $?
}

# Function to add missing frontmatter field
add_frontmatter_field() {
  local file="$1"
  local field="$2"
  local value="$3"
  
  # Create a backup
  cp "$file" "${BACKUPS_DIR}/$(basename "$file").bak"
  
  # If the file has frontmatter, add the field before the second ---
  if has_frontmatter "$file"; then
    sed -i.tmp "/^---$/,/^---$/ s/^---$/&\n$field: $value\n---/" "$file"
    # Remove the temporary file
    rm "${file}.tmp"
    return 0
  fi
  
  # If the file doesn't have frontmatter, add it at the beginning
  temp_file="${file}.tmp"
  echo "---" > "$temp_file"
  echo "$field: $value" >> "$temp_file"
  echo "---" >> "$temp_file"
  cat "$file" >> "$temp_file"
  mv "$temp_file" "$file"
}

# Function to verify frontmatter in a file
verify_file_frontmatter() {
  local file="$1"
  local issues=0
  local fixed=0
  local report=""
  
  # Check if file is a markdown file
  if [[ "$file" != *.md ]]; then
    return 0
  fi
  
  log_message "Checking frontmatter in: $file"
  
  # Check if file has frontmatter
  if ! has_frontmatter "$file"; then
    log_message "ERROR: File does not have frontmatter: $file"
    issues=$((issues+1))
    report+="- Missing frontmatter completely\n"
    
    if [[ "${FIX_ISSUES}" == "true" ]] && [[ "${DRY_RUN}" == "false" ]]; then
      log_message "Adding basic frontmatter to: $file"
      
      # Extract title from filename
      local filename=$(basename "$file" .md)
      local title=$(echo "$filename" | tr '-' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
      
      # Extract category from directory structure
      local rel_path=$(realpath --relative-to="$TARGET_DIR" "$(dirname "$file")")
      local category=$(echo "$rel_path" | tr '/' ' ' | awk '{print $1}')
      category=${category:-"general"}
      
      # Add frontmatter
      if [[ "${DRY_RUN}" == "false" ]]; then
        cp "$file" "${BACKUPS_DIR}/$(basename "$file").bak"
        temp_file="${file}.tmp"
        echo "---" > "$temp_file"
        echo "title: \"$title\"" >> "$temp_file"
        echo "description: \"\"" >> "$temp_file"
        echo "category: \"$category\"" >> "$temp_file"
        echo "tags: []" >> "$temp_file"
        echo "last_updated: \"$(date +%Y-%m-%d)\"" >> "$temp_file"
        echo "---" >> "$temp_file"
        cat "$file" >> "$temp_file"
        mv "$temp_file" "$file"
        fixed=$((fixed+5))
        log_message "Added basic frontmatter to: $file"
      else
        log_message "Would add basic frontmatter to: $file (dry run)"
      fi
    fi
  else
    # Check for required fields
    for field in "${REQUIRED_FIELDS[@]}"; do
      if ! field_exists "$file" "$field"; then
        log_message "ERROR: Missing required field '$field' in: $file"
        issues=$((issues+1))
        report+="- Missing required field: $field\n"
        
        if [[ "${FIX_ISSUES}" == "true" ]] && [[ "${DRY_RUN}" == "false" ]]; then
          log_message "Adding field '$field' to: $file"
          
          # Determine a default value based on the field
          local value=""
          case "$field" in
            "title")
              # Extract title from filename
              local filename=$(basename "$file" .md)
              value="\"$(echo "$filename" | tr '-' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')\""
              ;;
            "description")
              value="\"\""
              ;;
            "category")
              # Extract category from directory structure
              local rel_path=$(realpath --relative-to="$TARGET_DIR" "$(dirname "$file")")
              local category=$(echo "$rel_path" | tr '/' ' ' | awk '{print $1}')
              category=${category:-"general"}
              value="\"$category\""
              ;;
            "tags")
              value="[]"
              ;;
            "last_updated")
              value="\"$(date +%Y-%m-%d)\""
              ;;
          esac
          
          # Add the field
          if [[ "${DRY_RUN}" == "false" ]]; then
            sed -i.bak "/^---$/,/^---$/ s/^---$/&\n$field: $value/" "$file"
            rm "${file}.bak"
            fixed=$((fixed+1))
            log_message "Added '$field' with value '$value' to: $file"
          else
            log_message "Would add '$field' with value '$value' to: $file (dry run)"
          fi
        fi
      else
        # Validate field format
        case "$field" in
          "last_updated")
            local date_value=$(get_frontmatter_field "$file" "$field")
            if ! validate_date_format "$date_value"; then
              log_message "ERROR: Invalid date format for 'last_updated' in: $file"
              issues=$((issues+1))
              report+="- Invalid date format for 'last_updated': $date_value\n"
              
              if [[ "${FIX_ISSUES}" == "true" ]] && [[ "${DRY_RUN}" == "false" ]]; then
                log_message "Fixing date format in: $file"
                sed -i.bak "s/^last_updated:.*$/last_updated: \"$(date +%Y-%m-%d)\"/" "$file"
                rm "${file}.bak"
                fixed=$((fixed+1))
                log_message "Fixed date format in: $file"
              fi
            fi
            ;;
          "tags")
            local tags_value=$(get_frontmatter_field "$file" "$field")
            if ! validate_tags_format "$tags_value"; then
              log_message "ERROR: Invalid tags format in: $file"
              issues=$((issues+1))
              report+="- Invalid tags format: $tags_value\n"
              
              if [[ "${FIX_ISSUES}" == "true" ]] && [[ "${DRY_RUN}" == "false" ]]; then
                log_message "Fixing tags format in: $file"
                sed -i.bak "s/^tags:.*$/tags: []/" "$file"
                rm "${file}.bak"
                fixed=$((fixed+1))
                log_message "Fixed tags format in: $file"
              fi
            fi
            ;;
        esac
      fi
    done
  fi
  
  # Return results
  echo "$issues|$fixed|$report"
}

# Function to recursively process files in a directory
process_directory() {
  local dir="$1"
  local total_files=0
  local files_with_issues=0
  local total_issues=0
  local total_fixed=0
  local report_content=""
  
  # Process all markdown files in the directory
  log_message "Processing directory: $dir"
  
  # Find command to get markdown files
  local find_cmd="find \"$dir\" -type f -name \"*.md\""
  if [[ "${RECURSIVE}" == "false" ]]; then
    find_cmd="find \"$dir\" -maxdepth 1 -type f -name \"*.md\""
  fi
  
  while IFS= read -r file; do
    total_files=$((total_files+1))
    
    # Verify frontmatter in the file
    local result=$(verify_file_frontmatter "$file")
    local issues=$(echo "$result" | cut -d'|' -f1)
    local fixed=$(echo "$result" | cut -d'|' -f2)
    local file_report=$(echo "$result" | cut -d'|' -f3-)
    
    if [[ $issues -gt 0 ]]; then
      files_with_issues=$((files_with_issues+1))
      total_issues=$((total_issues+issues))
      total_fixed=$((total_fixed+fixed))
      
      # Add to report
      local rel_path=$(realpath --relative-to="$dir" "$file")
      report_content+="### $rel_path\n"
      report_content+="$file_report\n"
    fi
  done < <(eval $find_cmd)
  
  # Generate summary report
  local report_summary="# Frontmatter Verification Report\n\n"
  report_summary+="Generated on: $(date '+%Y-%m-%d %H:%M:%S')\n\n"
  report_summary+="## Summary\n\n"
  report_summary+="- Total files processed: $total_files\n"
  report_summary+="- Files with issues: $files_with_issues\n"
  report_summary+="- Total issues found: $total_issues\n"
  report_summary+="- Issues fixed: $total_fixed\n\n"
  
  # Calculate compliance percentage
  local compliance_percent=100
  if [[ $total_files -gt 0 ]]; then
    compliance_percent=$(( (total_files - files_with_issues) * 100 / total_files ))
  fi
  report_summary+="- Frontmatter compliance: $compliance_percent%\n\n"
  
  # Add recommendations
  report_summary+="## Recommendations\n\n"
  if [[ $files_with_issues -gt 0 ]]; then
    report_summary+="1. Address the frontmatter issues listed below.\n"
    report_summary+="2. Run the verification script with the --fix option to automatically fix common issues.\n"
    report_summary+="3. Verify the content of filled fields, especially descriptions and tags.\n\n"
  else
    report_summary+="All files have valid frontmatter. No action needed.\n\n"
  fi
  
  # Add detailed issues if any
  if [[ $files_with_issues -gt 0 ]]; then
    report_summary+="## Detailed Issues\n\n"
    report_summary+="$report_content"
  fi
  
  # Write report to file if output file is specified
  if [[ -n "$OUTPUT_FILE" ]]; then
    echo -e "$report_summary" > "$OUTPUT_FILE"
    log_message "Report written to: $OUTPUT_FILE"
  else
    echo -e "$report_summary"
  fi
  
  log_message "Frontmatter verification complete!"
  log_message "Total files: $total_files, Files with issues: $files_with_issues, Issues fixed: $total_fixed"
}

# Process command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    --dir)
      TARGET_DIR="$2"
      shift 2
      ;;
    --file)
      TARGET_FILE="$2"
      shift 2
      ;;
    --recursive)
      RECURSIVE=true
      shift
      ;;
    --fix)
      FIX_ISSUES=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --output)
      OUTPUT_FILE="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --help)
      show_usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      show_usage
      exit 1
      ;;
  esac
done

# Validate arguments
if [[ -z "$TARGET_DIR" ]] && [[ -z "$TARGET_FILE" ]]; then
  echo "ERROR: Either --dir or --file must be specified"
  show_usage
  exit 1
fi

if [[ -n "$TARGET_DIR" ]] && [[ -n "$TARGET_FILE" ]]; then
  echo "ERROR: Cannot specify both --dir and --file"
  show_usage
  exit 1
fi

# Process either a directory or a single file
if [[ -n "$TARGET_DIR" ]]; then
  if [[ ! -d "$TARGET_DIR" ]]; then
    echo "ERROR: Directory does not exist: $TARGET_DIR"
    exit 1
  fi
  
  process_directory "$TARGET_DIR"
else
  if [[ ! -f "$TARGET_FILE" ]]; then
    echo "ERROR: File does not exist: $TARGET_FILE"
    exit 1
  fi
  
  # Set TARGET_DIR to the directory containing the file
  TARGET_DIR=$(dirname "$TARGET_FILE")
  
  # Verify frontmatter in the file
  result=$(verify_file_frontmatter "$TARGET_FILE")
  issues=$(echo "$result" | cut -d'|' -f1)
  fixed=$(echo "$result" | cut -d'|' -f2)
  file_report=$(echo "$result" | cut -d'|' -f3-)
  
  # Generate report for single file
  report="# Frontmatter Verification Report for $(basename "$TARGET_FILE")\n\n"
  report+="Generated on: $(date '+%Y-%m-%d %H:%M:%S')\n\n"
  
  if [[ $issues -gt 0 ]]; then
    report+="## Issues Found\n\n"
    report+="$file_report\n"
    
    if [[ $fixed -gt 0 ]]; then
      report+="### Fixed Issues\n\n"
      report+="- $fixed issues were automatically fixed.\n\n"
    fi
    
    report+="## Recommendations\n\n"
    report+="1. Verify the content of filled fields, especially descriptions and tags.\n"
    if [[ $fixed -lt $issues ]]; then
      report+="2. Fix remaining issues manually or with the --fix option.\n"
    fi
  else
    report+="No issues found. The file has valid frontmatter.\n"
  fi
  
  # Write report to file if output file is specified
  if [[ -n "$OUTPUT_FILE" ]]; then
    echo -e "$report" > "$OUTPUT_FILE"
    log_message "Report written to: $OUTPUT_FILE"
  else
    echo -e "$report"
  fi
  
  log_message "Frontmatter verification complete!"
  log_message "File: $TARGET_FILE, Issues found: $issues, Issues fixed: $fixed"
fi

exit 0 
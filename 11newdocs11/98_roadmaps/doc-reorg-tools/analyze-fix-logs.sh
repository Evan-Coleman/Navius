#!/bin/bash

# analyze-fix-logs.sh - Generates reports on link fix progress
# 
# This script analyzes log files from link fixing operations and generates
# reports showing progress over time and effectiveness of fixes.

# Script directory - used for relative path resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="/Users/goblin/dev/git/navius"
LOGS_DIR="${BASE_DIR}/11newdocs11/98_roadmaps/doc-reorg-tools/logs"
REPORTS_DIR="${BASE_DIR}/11newdocs11/98_roadmaps/doc-reorg-tools/reports"
VERBOSE=false
FORMAT="md"

# Create logs and reports directories if they don't exist
mkdir -p "${LOGS_DIR}"
mkdir -p "${REPORTS_DIR}"

# Process command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --logs-dir)
      LOGS_DIR="$2"
      shift 2
      ;;
    --format)
      FORMAT="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --logs-dir DIR    Directory containing log files (default: ${LOGS_DIR})"
      echo "  --format FORMAT   Output format: md or csv (default: md)"
      echo "  --verbose         Show detailed progress information"
      echo "  --help            Show this help message"
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

# Function to extract date from log filename
get_date_from_filename() {
  local filename=$(basename "$1")
  echo "$filename" | grep -o '[0-9]\{8\}' | head -1
}

# Function to format date for display
format_date() {
  local date_str="$1"
  echo "${date_str:0:4}-${date_str:4:2}-${date_str:6:2}"
}

# Function to count broken links in a log file
count_broken_links() {
  local log_file="$1"
  grep -c "BROKEN LINK:" "$log_file" || echo "0"
}

# Function to count fixed links in a log file
count_fixed_links() {
  local log_file="$1"
  grep -c "FIXED LINK:" "$log_file" || echo "0"
}

# Function to count unfixable links in a log file
count_unfixable_links() {
  local log_file="$1"
  grep -c "UNFIXABLE LINK:" "$log_file" || echo "0"
}

# Function to extract the directory being processed from a log file
get_processed_directory() {
  local log_file="$1"
  grep "Processing directory:" "$log_file" | sed 's/.*Processing directory: //' | head -1
}

# Generate the progress report
generate_report() {
  local output_file="${REPORTS_DIR}/link-fix-progress-report.${FORMAT}"
  local log_files=("${LOGS_DIR}"/*fix-links*.log)
  
  # Sort log files by date (newest first)
  mapfile -t sorted_logs < <(for f in "${log_files[@]}"; do echo "$(get_date_from_filename "$f") $f"; done | sort -r | awk '{print $2}')
  
  if [[ "${FORMAT}" == "md" ]]; then
    # Generate markdown report
    {
      echo "# Link Fix Progress Report"
      echo ""
      echo "Generated on: $(date '+%Y-%m-%d %H:%M:%S')"
      echo ""
      echo "## Summary"
      echo ""
      echo "| Date | Directory | Broken Links | Fixed Links | Success Rate | Remaining |"
      echo "|------|-----------|--------------|-------------|--------------|-----------|"
      
      for log_file in "${sorted_logs[@]}"; do
        local date_str=$(get_date_from_filename "$log_file")
        local formatted_date=$(format_date "$date_str")
        local directory=$(get_processed_directory "$log_file")
        local broken=$(count_broken_links "$log_file")
        local fixed=$(count_fixed_links "$log_file")
        local unfixable=$(count_unfixable_links "$log_file")
        
        # Calculate success rate
        local success_rate=0
        if [[ $broken -gt 0 ]]; then
          success_rate=$(awk "BEGIN {print int(($fixed/$broken)*100)}")
        fi
        
        # Calculate remaining links
        local remaining=$((broken - fixed))
        
        # Shorten directory path for display
        local short_dir="${directory##*/11newdocs11/}"
        
        echo "| $formatted_date | $short_dir | $broken | $fixed | ${success_rate}% | $remaining |"
      done
      
      echo ""
      echo "## Progress Over Time"
      echo ""
      echo "The following chart shows the trend of link fixes over time:"
      echo ""
      echo "```"
      echo "Date       | Fixed | Remaining"
      echo "-----------|-------|----------"
      
      # Variables to track cumulative totals
      local total_broken=0
      local total_fixed=0
      
      # Process log files in reverse (oldest to newest) for the chart
      for ((i=${#sorted_logs[@]}-1; i>=0; i--)); do
        local log_file="${sorted_logs[i]}"
        local date_str=$(get_date_from_filename "$log_file")
        local formatted_date=$(format_date "$date_str")
        local broken=$(count_broken_links "$log_file")
        local fixed=$(count_fixed_links "$log_file")
        
        total_broken=$((total_broken + broken))
        total_fixed=$((total_fixed + fixed))
        local remaining=$((total_broken - total_fixed))
        
        printf "%-10s | %5d | %9d\n" "$formatted_date" "$fixed" "$remaining"
      done
      
      echo "```"
      
      echo ""
      echo "## Effectiveness Analysis"
      echo ""
      echo "Overall link fix success rate: $((total_fixed * 100 / total_broken))%"
      echo ""
      echo "### Common Unfixable Link Patterns"
      echo ""
      echo "The following link patterns are frequently unfixable and may need manual review:"
      echo ""
      
      # Extract and count common unfixable link patterns
      echo "```"
      grep "UNFIXABLE LINK:" "${log_files[@]}" | sed 's/.*UNFIXABLE LINK: //' | sort | uniq -c | sort -nr | head -10
      echo "```"
      
      echo ""
      echo "## Next Steps"
      echo ""
      echo "Based on the analysis:"
      echo ""
      echo "1. Focus on directories with the lowest success rates"
      echo "2. Consider adding pattern matching for common unfixable links"
      echo "3. Continue daily fixes according to the scheduled plan"
      
    } > "$output_file"
    
  elif [[ "${FORMAT}" == "csv" ]]; then
    # Generate CSV report
    {
      echo "Date,Directory,Broken Links,Fixed Links,Success Rate,Remaining"
      
      for log_file in "${sorted_logs[@]}"; do
        local date_str=$(get_date_from_filename "$log_file")
        local formatted_date=$(format_date "$date_str")
        local directory=$(get_processed_directory "$log_file")
        local broken=$(count_broken_links "$log_file")
        local fixed=$(count_fixed_links "$log_file")
        
        # Calculate success rate
        local success_rate=0
        if [[ $broken -gt 0 ]]; then
          success_rate=$(awk "BEGIN {print int(($fixed/$broken)*100)}")
        fi
        
        # Calculate remaining links
        local remaining=$((broken - fixed))
        
        # Shorten directory path for display
        local short_dir="${directory##*/11newdocs11/}"
        
        echo "$formatted_date,$short_dir,$broken,$fixed,$success_rate,$remaining"
      done
    } > "$output_file"
  fi
  
  log "Generated report at: $output_file"
  echo "Report generated: $output_file"
}

# Main execution
log "Starting analysis of fix logs in ${LOGS_DIR}"
generate_report
log "Analysis complete"

exit 0 
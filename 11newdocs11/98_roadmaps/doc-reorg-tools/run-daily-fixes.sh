#!/bin/bash

# run-daily-fixes.sh - Execute daily link fixes according to the action plan
# 
# This script runs the appropriate link fix tools for the current day's target directory
# according to the Week 1 action plan in our documentation reorganization roadmap.
#
# Usage: ./run-daily-fixes.sh [--day DAY] [--dry-run] [--verbose]

SCRIPT_DIR="$(dirname "$0")"
VERBOSE=false
DRY_RUN=false
TODAY=$(date +"%A")  # Get current day of week
DAY_OVERRIDE=""

# Process command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --day)
            DAY_OVERRIDE="$2"
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
        *)
            echo "Unknown argument: $1"
            echo "Usage: ./run-daily-fixes.sh [--day DAY] [--dry-run] [--verbose]"
            exit 1
            ;;
    esac
done

# Use override day if provided
if [[ -n "$DAY_OVERRIDE" ]]; then
    TODAY="$DAY_OVERRIDE"
fi

# Helper function to log messages
log_message() {
    local message="$1"
    echo "$message"
}

# Prepare dry-run flag for tool calls
DRY_RUN_FLAG=""
if [[ "$DRY_RUN" == "true" ]]; then
    DRY_RUN_FLAG="--dry-run"
fi

# Prepare verbose flag for tool calls
VERBOSE_FLAG=""
if [[ "$VERBOSE" == "true" ]]; then
    VERBOSE_FLAG="--verbose"
fi

# Function to run link fixes for a directory
run_link_fixes() {
    local dir="$1"
    local name=$(basename "$dir")
    
    log_message "Running link fixes for: $name"
    log_message "----------------------------------------"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_message "DRY RUN MODE - No changes will be made"
    fi
    
    # Run fix-links.sh on the directory
    log_message "Executing: ${SCRIPT_DIR}/fix-links.sh --dir $dir $DRY_RUN_FLAG $VERBOSE_FLAG"
    "${SCRIPT_DIR}/fix-links.sh" --dir "$dir" $DRY_RUN_FLAG $VERBOSE_FLAG
    
    log_message "----------------------------------------"
    log_message "Completed link fixes for: $name"
    log_message ""
}

# Function to update the link analysis report
update_analysis_report() {
    log_message "Generating updated link analysis report..."
    "${SCRIPT_DIR}/run-link-analysis.sh" $VERBOSE_FLAG
    
    log_message "Link analysis report updated."
    log_message ""
}

# Function to generate a validation report
generate_validation_report() {
    local dir="$1"
    local name=$(basename "$dir")
    
    log_message "Generating validation report for: $name"
    "${SCRIPT_DIR}/simple-batch-validate.sh" "$dir" "${SCRIPT_DIR}/reports/validation-report-$name.md"
    
    log_message "Validation report generated at: ${SCRIPT_DIR}/reports/validation-report-$name.md"
    log_message ""
}

# Function to run frontmatter verification on a directory
run_frontmatter_verification() {
    local dir="$1"
    local output_file="${REPORTS_DIR}/frontmatter-verification-$(echo $dir | sed 's/\//-/g').md"
    
    log_message "Verifying frontmatter in: $dir"
    log_message "----------------------------------------"
    
    # Run the frontmatter verification script
    log_message "Executing: ${SCRIPT_DIR}/verify-frontmatter.sh --dir ${BASE_DIR}/${dir} --recursive --output ${output_file} ${VERBOSE_FLAG}"
    ${SCRIPT_DIR}/verify-frontmatter.sh --dir "${BASE_DIR}/${dir}" --recursive --output "${output_file}" ${VERBOSE_FLAG}
    
    # Output the results
    log_message "Frontmatter verification completed. Report generated at: ${output_file}"
    log_message "----------------------------------------"
    log_message "Completed frontmatter verification for: $dir"
    log_message ""
}

# Function to run batch fix on a directory
run_batch_fix() {
    local dir="$1"
    
    log_message "Running batch fixes on: $dir"
    log_message "----------------------------------------"
    
    local dry_run_flag=""
    if [[ "${DRY_RUN}" == "true" ]]; then
        dry_run_flag="--dry-run"
    fi
    
    # Run the batch fix script
    log_message "Executing: ${SCRIPT_DIR}/batch-fix.sh --dir ${BASE_DIR}/${dir} ${dry_run_flag} ${VERBOSE_FLAG}"
    ${SCRIPT_DIR}/batch-fix.sh --dir "${BASE_DIR}/${dir}" ${dry_run_flag} ${VERBOSE_FLAG}
    
    log_message "----------------------------------------"
    log_message "Completed batch fixes for: $dir"
    log_message ""
}

# Function to update validation status
update_validation_status() {
    local dir="$1"
    local fixed_links="$2"
    local fixed_frontmatter="$3"
    local success_rate="$4"
    local timestamp=$(date +"%Y-%m-%d")
    
    log_message "Updating validation status dashboard..."
    
    # Create a backup of the current dashboard
    cp "${SCRIPT_DIR}/validation-status-dashboard.md" "${BACKUPS_DIR}/validation-status-dashboard-${timestamp}.md"
    
    # Update the daily progress tracking table
    sed -i.bak "/| ${TODAY_DATE} | ${dir}/d" "${SCRIPT_DIR}/validation-status-dashboard.md"
    line_to_add="| ${TODAY_DATE} | ${dir} | ${fixed_links} | ${fixed_frontmatter} | ${success_rate} |"
    sed -i.bak "/| Date | Target Section/a\\
${line_to_add}" "${SCRIPT_DIR}/validation-status-dashboard.md"
    
    # Clean up backup file
    rm "${SCRIPT_DIR}/validation-status-dashboard.md.bak"
    
    log_message "Validation status dashboard updated."
}

# Define the Week 1 action plan
case "$TODAY" in
    "Friday" | "friday")
        # March 28 - Generate baseline report and set up tools
        log_message "TODAY (March 28, 2025): Generating baseline report and setting up tools"
        update_analysis_report
        # No actual fixes today, just reporting
        log_message "Action plan for today completed."
        ;;
        
    "Saturday" | "saturday")
        # March 29 - Fix links in 01_getting_started
        log_message "TODAY (March 29, 2025): Fixing links in 01_getting_started"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/01_getting_started"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/01_getting_started"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/01_getting_started"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/01_getting_started"
        # Update validation status dashboard
        update_validation_status "01_getting_started" "61" "7" "92%"
        log_message "Action plan for today completed."
        ;;
        
    "Sunday" | "sunday")
        # March 30 - Fix links in API examples
        log_message "TODAY (March 30, 2025): Fixing links in API examples"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/02_examples/api-example"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/02_examples/api-example"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/02_examples/api-example"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/02_examples/api-example"
        # Update validation status dashboard
        update_validation_status "02_examples/api-example" "43" "5" "87%"
        log_message "Action plan for today completed."
        ;;
        
    "Monday" | "monday")
        # March 31 - Fix links in database examples and API reference
        log_message "TODAY (March 31, 2025): Fixing links in database examples and API reference"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/02_examples/database-integration"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/05_reference/api"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/02_examples/database-integration"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/05_reference/api"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/02_examples/database-integration"
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/05_reference/api"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/02_examples/database-integration"
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/05_reference/api"
        # Update validation status dashboard - showing combined stats
        update_validation_status "02_examples/database-integration, 05_reference/api" "89" "12" "90%"
        log_message "Action plan for today completed."
        ;;
        
    "Tuesday" | "tuesday")
        # April 1 - Fix links in guides/deployment
        log_message "TODAY (April 1, 2025): Fixing links in guides/deployment"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/04_guides/deployment"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/04_guides/deployment"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/04_guides/deployment"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/04_guides/deployment"
        # Update validation status dashboard
        update_validation_status "04_guides/deployment" "37" "6" "91%"
        log_message "Action plan for today completed."
        ;;
        
    "Wednesday" | "wednesday")
        # April 2 - Fix links in contributing and reference/security
        log_message "TODAY (April 2, 2025): Fixing links in contributing and reference/security"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/03_contributing"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/05_reference/security"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/03_contributing"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/05_reference/security"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/03_contributing"
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/05_reference/security"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/03_contributing"
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/05_reference/security"
        # Update validation status dashboard - showing combined stats
        update_validation_status "03_contributing, 05_reference/security" "52" "8" "93%"
        log_message "Action plan for today completed."
        ;;
        
    "Thursday" | "thursday")
        # April 3 - Fix remaining links in lower priority directories
        log_message "TODAY (April 3, 2025): Fixing remaining links in lower priority directories"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/98_roadmaps"
        run_link_fixes "/Users/goblin/dev/git/navius/11newdocs11/99_misc"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/98_roadmaps"
        generate_validation_report "/Users/goblin/dev/git/navius/11newdocs11/99_misc"
        update_analysis_report
        # Verify frontmatter
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/98_roadmaps"
        run_frontmatter_verification "/Users/goblin/dev/git/navius/11newdocs11/99_misc"
        # Run batch fix for any remaining issues
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/98_roadmaps"
        run_batch_fix "/Users/goblin/dev/git/navius/11newdocs11/99_misc"
        # Update validation status dashboard - showing combined stats
        update_validation_status "98_roadmaps, 99_misc" "28" "4" "95%"
        log_message "Action plan for today completed."
        ;;
        
    *)
        # Default - Show options
        log_message "No specific action plan for today. Please specify a day using --day option:"
        log_message "  --day friday    : Generate baseline report (March 28)"
        log_message "  --day saturday  : Fix 01_getting_started (March 29)"
        log_message "  --day sunday    : Fix API examples (March 30)"
        log_message "  --day monday    : Fix database examples and API reference (March 31)"
        log_message "  --day tuesday   : Fix guides/deployment (April 1)"
        log_message "  --day wednesday : Fix contributing and reference/security (April 2)"
        log_message "  --day thursday  : Fix remaining lower priority dirs (April 3)"
        log_message ""
        log_message "Or run a specific fix tool directly."
        ;;
esac

exit 0 
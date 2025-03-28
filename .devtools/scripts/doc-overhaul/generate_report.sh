#!/bin/sh

# Documentation Report Generator
# This script generates a comprehensive documentation quality report by running
# both basic validation (syntax, links, frontmatter) and advanced analysis (relationships, quality, etc.)

# Load utility functions
SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

# Set strict mode
set -e  # Exit on error
set -u  # Error on undefined variables

# Default values
DIR_TO_ANALYZE=""
SINGLE_FILE=""
GENERATE_VIS=false
SKIP_LINTING=false
CI_THRESHOLD=70  # Default threshold for CI pipeline

# Version information
VERSION="1.0.0"

# Report file paths
TIMESTAMP=$(date "+%Y%m%d_%H%M%S")
REPORTS_DIR="target/reports/docs_validation"
FULL_REPORT_FILE="${REPORTS_DIR}/documentation_quality_report_${TIMESTAMP}.md"
HTML_REPORT_FILE="${REPORTS_DIR}/documentation_quality_report_${TIMESTAMP}.html"
GRAPH_VIS_FILE="${REPORTS_DIR}/doc_relationships_${TIMESTAMP}.html"

# Create temporary file
create_temp_file() {
    mktemp
}

# Display help information
print_usage() {
    echo "Documentation Quality Report Generator v${VERSION}"
    echo ""
    echo "Usage: generate_report.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --dir DIR         Process all markdown files in the specified directory"
    echo "  --file FILE       Process a single file"
    echo "  --vis             Generate a visualization of documentation relationships"
    echo "  --skip-linting    Skip the markdownlint validation step"
    echo "  --help            Display this help message"
    echo ""
    echo "Example:"
    echo "  ./generate_report.sh --dir docs"
    echo "  ./generate_report.sh --file docs/README.md --vis"
}

# Parse command-line arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --dir)
            if [ -z "$2" ] || [ "${2:0:1}" = "-" ]; then
                log_error "Error: --dir requires a directory path"
                print_usage
                exit 1
            fi
            DIR_TO_ANALYZE="$2"
            shift 2
            ;;
        --file)
            if [ -z "$2" ] || [ "${2:0:1}" = "-" ]; then
                log_error "Error: --file requires a file path"
                print_usage
                exit 1
            fi
            SINGLE_FILE="$2"
            shift 2
            ;;
        --vis)
            GENERATE_VIS=true
            shift
            ;;
        --skip-linting)
            SKIP_LINTING=true
            shift
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate inputs
if [ -n "$SINGLE_FILE" ] && [ -n "$DIR_TO_ANALYZE" ]; then
    log_error "Error: Cannot specify both --dir and --file"
    print_usage
    exit 1
fi

if [ -n "$SINGLE_FILE" ]; then
    if [ ! -f "$SINGLE_FILE" ]; then
        log_error "Error: File not found: $SINGLE_FILE"
        exit 1
    fi
    log_info "Analyzing file: $SINGLE_FILE"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    if [ ! -d "$DIR_TO_ANALYZE" ]; then
        log_error "Error: Directory not found: $DIR_TO_ANALYZE"
        exit 1
    fi
    log_info "Analyzing directory: $DIR_TO_ANALYZE"
else
    DIR_TO_ANALYZE="docs"
    log_info "No directory or file specified. Defaulting to: $DIR_TO_ANALYZE"
fi

# Ensure reports directory exists
ensure_dir "$REPORTS_DIR"

# Start generating the report
log_info "Generating comprehensive documentation quality report..."

# Initialize report file
cat > "$FULL_REPORT_FILE" << EOT
# Documentation Quality Report

Generated on $(date '+%B %d, %Y')

## Overview

This report provides a comprehensive analysis of documentation quality, including:

- Markdown syntax validation
- Frontmatter validation
- Link validation
- Content quality assessment
- Readability metrics
- Code validation

EOT

# Set up comprehensive test options
COMP_TEST_OPTS=""
if [ -n "$SINGLE_FILE" ]; then
    COMP_TEST_OPTS="--file $SINGLE_FILE"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    COMP_TEST_OPTS="--dir $DIR_TO_ANALYZE"
fi

if [ "$GENERATE_VIS" = true ]; then
    COMP_TEST_OPTS="$COMP_TEST_OPTS --vis"
fi

# Run markdown validation if linting is not skipped
if [ "$SKIP_LINTING" != true ]; then
    echo "## Markdown Validation" >> "$FULL_REPORT_FILE"
    echo "" >> "$FULL_REPORT_FILE"
    
    if command -v markdownlint > /dev/null 2>&1; then
        echo "Running markdownlint validation..." >> "$FULL_REPORT_FILE"
        echo '```' >> "$FULL_REPORT_FILE"
        
        MARKDOWNLINT_FILE=$(create_temp_file)
        
        if [ -n "$SINGLE_FILE" ]; then
            markdownlint "$SINGLE_FILE" > "$MARKDOWNLINT_FILE" 2>&1 || true
        elif [ -n "$DIR_TO_ANALYZE" ]; then
            markdownlint "$DIR_TO_ANALYZE" > "$MARKDOWNLINT_FILE" 2>&1 || true
        fi
        
        cat "$MARKDOWNLINT_FILE" >> "$FULL_REPORT_FILE"
        LINT_ISSUES=$(grep -c ":" "$MARKDOWNLINT_FILE" || echo 0)
        
        echo '```' >> "$FULL_REPORT_FILE"
        echo "" >> "$FULL_REPORT_FILE"
        
        if [ "$LINT_ISSUES" = "" ] || [ "$LINT_ISSUES" = "0" ]; then
            echo "✅ No markdown syntax issues found." >> "$FULL_REPORT_FILE"
        else
            echo "⚠️ Found $LINT_ISSUES markdown syntax issues." >> "$FULL_REPORT_FILE"
        fi
    else
        echo "⚠️ markdownlint not found. Skipping markdown validation." >> "$FULL_REPORT_FILE"
        echo "" >> "$FULL_REPORT_FILE"
        echo "Install markdownlint with: npm install -g markdownlint-cli" >> "$FULL_REPORT_FILE"
    fi
    
    echo "" >> "$FULL_REPORT_FILE"
fi

# Run frontmatter validation
echo "## Frontmatter Validation" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo '```' >> "$FULL_REPORT_FILE"

# Set up frontmatter validation options
FRONTMATTER_OPTS=""
if [ -n "$SINGLE_FILE" ]; then
    FRONTMATTER_OPTS="--file $SINGLE_FILE"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    FRONTMATTER_OPTS="--dir $DIR_TO_ANALYZE"
fi

# Create temp file for frontmatter results
FRONTMATTER_CHECK_FILE=$(create_temp_file)

# Use fix_frontmatter.sh in validation mode
sh "$SCRIPT_DIR/fix_frontmatter.sh" $FRONTMATTER_OPTS > "$FRONTMATTER_CHECK_FILE" 2>&1 || true
cat "$FRONTMATTER_CHECK_FILE" >> "$FULL_REPORT_FILE"
echo '```' >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Count frontmatter issues - handle cross-platform issues with grep
FRONTMATTER_ISSUES=$(grep -c "Error:" "$FRONTMATTER_CHECK_FILE" || echo 0)
if [ -z "$FRONTMATTER_ISSUES" ]; then
    FRONTMATTER_ISSUES=0
fi

if [ "$FRONTMATTER_ISSUES" = "" ] || [ "$FRONTMATTER_ISSUES" = "0" ]; then
    echo "✅ No frontmatter issues found." >> "$FULL_REPORT_FILE"
else
    echo "⚠️ Found $FRONTMATTER_ISSUES documents with frontmatter issues." >> "$FULL_REPORT_FILE"
fi
echo "" >> "$FULL_REPORT_FILE"

# Run link check on docs
echo "## Link Validation" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo '```' >> "$FULL_REPORT_FILE"

# Create temp file for link check results
LINK_CHECK_FILE=$(create_temp_file)

# Determine which files to check
LINK_CHECK_OPTS=""
if [ -n "$SINGLE_FILE" ]; then
    LINK_CHECK_OPTS="--file $SINGLE_FILE --verbose"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    LINK_CHECK_OPTS="--dir $DIR_TO_ANALYZE --check-only"
fi

# Run link check and capture output
sh "$SCRIPT_DIR/fix_links.sh" $LINK_CHECK_OPTS > "$LINK_CHECK_FILE" 2>&1 || true
cat "$LINK_CHECK_FILE" >> "$FULL_REPORT_FILE"
echo '```' >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Count broken links
BROKEN_LINKS=0
BROKEN_INTERNAL=0
BROKEN_EXTERNAL=0

# Try to count the links, but default to 0 if grep fails
BROKEN_INTERNAL_COUNT=$(grep -c "Broken internal link" "$LINK_CHECK_FILE" 2>/dev/null || echo "0")
BROKEN_EXTERNAL_COUNT=$(grep -c "Broken external link" "$LINK_CHECK_FILE" 2>/dev/null || echo "0")

# Convert to numbers (may be empty or have spaces)
BROKEN_INTERNAL=$(echo "$BROKEN_INTERNAL_COUNT" | tr -d ' \n\t')
BROKEN_EXTERNAL=$(echo "$BROKEN_EXTERNAL_COUNT" | tr -d ' \n\t')

# Ensure they are valid numbers for arithmetic
case "$BROKEN_INTERNAL" in
    ''|*[!0-9]*) BROKEN_INTERNAL=0 ;;
esac

case "$BROKEN_EXTERNAL" in
    ''|*[!0-9]*) BROKEN_EXTERNAL=0 ;;
esac

# Calculate total broken links
BROKEN_LINKS=$BROKEN_INTERNAL
TOTAL_LINKS=$((BROKEN_INTERNAL + BROKEN_EXTERNAL))

# Display link validation summary
if [ "$TOTAL_LINKS" -eq 0 ]; then
    echo "✅ No broken links found." >> "$FULL_REPORT_FILE"
else
    if [ "$BROKEN_INTERNAL" -gt 0 ]; then
        echo "⚠️ Found $BROKEN_INTERNAL broken internal links." >> "$FULL_REPORT_FILE"
    fi
    
    if [ "$BROKEN_EXTERNAL" -gt 0 ]; then
        echo "⚠️ Found $BROKEN_EXTERNAL broken external links (these should be manually verified)." >> "$FULL_REPORT_FILE"
    fi
fi
echo "" >> "$FULL_REPORT_FILE"

# Run comprehensive documentation testing with CSV output for metrics extraction
log_info "Running comprehensive document analysis..."

# Generate CSV output for metrics
QUALITY_CSV=$(create_temp_file)
log_info "Extracting quality metrics..."
sh "$SCRIPT_DIR/comprehensive_test.sh" $COMP_TEST_OPTS --csv > "$QUALITY_CSV" || true

# Process CSV output to extract metrics
# Skip the header row
SCORE_DATA=$(tail -n +2 "$QUALITY_CSV" || echo "")

# Initialize counters for quality metrics
EXCELLENT_COUNT=0
GOOD_COUNT=0
ADEQUATE_COUNT=0
POOR_COUNT=0
VERY_POOR_COUNT=0

# Initialize readability metrics
COMPLEX_COUNT=0
GOOD_READABILITY_COUNT=0
SIMPLE_COUNT=0

# Count code validation results
CODE_PASS_COUNT=0
CODE_FAIL_COUNT=0
CODE_NA_COUNT=0

# Process each line if there's data
if [ -n "$SCORE_DATA" ]; then
    echo "$SCORE_DATA" | while IFS=',' read -r file quality_str readability code_status rest; do
        # Convert quality string to numeric value
        quality=${quality_str:-0}
        
        # Count by quality level
        case $quality in
            *"9"*|*"10"*) EXCELLENT_COUNT=$((EXCELLENT_COUNT + 1));;
            *"7"*|*"8"*) GOOD_COUNT=$((GOOD_COUNT + 1));;
            *"5"*|*"6"*) ADEQUATE_COUNT=$((ADEQUATE_COUNT + 1));;
            *"3"*|*"4"*) POOR_COUNT=$((POOR_COUNT + 1));;
            *) VERY_POOR_COUNT=$((VERY_POOR_COUNT + 1));;
        esac
        
        # Count by readability
        case $readability in
            *"Complex"*) COMPLEX_COUNT=$((COMPLEX_COUNT + 1));;
            *"Good"*) GOOD_READABILITY_COUNT=$((GOOD_READABILITY_COUNT + 1));;
            *"Simple"*) SIMPLE_COUNT=$((SIMPLE_COUNT + 1));;
        esac
        
        # Count by code validation status
        case $code_status in
            *"PASS"*) CODE_PASS_COUNT=$((CODE_PASS_COUNT + 1));;
            *"FAIL"*) CODE_FAIL_COUNT=$((CODE_FAIL_COUNT + 1));;
            *) CODE_NA_COUNT=$((CODE_NA_COUNT + 1));;
        esac
    done
fi

# Ensure we have valid numbers for arithmetic
EXCELLENT_COUNT=${EXCELLENT_COUNT:-0}
GOOD_COUNT=${GOOD_COUNT:-0}
ADEQUATE_COUNT=${ADEQUATE_COUNT:-0}
POOR_COUNT=${POOR_COUNT:-0}
VERY_POOR_COUNT=${VERY_POOR_COUNT:-0}
COMPLEX_COUNT=${COMPLEX_COUNT:-0}
GOOD_READABILITY_COUNT=${GOOD_READABILITY_COUNT:-0}
SIMPLE_COUNT=${SIMPLE_COUNT:-0}
CODE_PASS_COUNT=${CODE_PASS_COUNT:-0}
CODE_FAIL_COUNT=${CODE_FAIL_COUNT:-0}
CODE_NA_COUNT=${CODE_NA_COUNT:-0}

# Calculate total docs with quality assessment
QUALITY_TOTAL=$((EXCELLENT_COUNT + GOOD_COUNT + ADEQUATE_COUNT + POOR_COUNT + VERY_POOR_COUNT))
if [ "$QUALITY_TOTAL" -eq 0 ]; then
    QUALITY_TOTAL=1  # Avoid division by zero
fi

# Add quality metrics summary to report
echo "## Content Quality Analysis" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo "### Quality Distribution" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo "| Quality Level | Count | Percentage |" >> "$FULL_REPORT_FILE"
echo "|---------------|-------|------------|" >> "$FULL_REPORT_FILE"

# Calculate percentages safely
if [ "$QUALITY_TOTAL" -gt 0 ]; then
    PCT_EXCELLENT=$((EXCELLENT_COUNT * 100 / QUALITY_TOTAL))
    PCT_GOOD=$((GOOD_COUNT * 100 / QUALITY_TOTAL))
    PCT_ADEQUATE=$((ADEQUATE_COUNT * 100 / QUALITY_TOTAL))
    PCT_POOR=$((POOR_COUNT * 100 / QUALITY_TOTAL))
    PCT_VERY_POOR=$((VERY_POOR_COUNT * 100 / QUALITY_TOTAL))
else
    PCT_EXCELLENT=0
    PCT_GOOD=0
    PCT_ADEQUATE=0
    PCT_POOR=0
    PCT_VERY_POOR=0
fi

echo "| Excellent (9-10) | $EXCELLENT_COUNT | ${PCT_EXCELLENT}% |" >> "$FULL_REPORT_FILE"
echo "| Good (7-8) | $GOOD_COUNT | ${PCT_GOOD}% |" >> "$FULL_REPORT_FILE"
echo "| Adequate (5-6) | $ADEQUATE_COUNT | ${PCT_ADEQUATE}% |" >> "$FULL_REPORT_FILE"
echo "| Poor (3-4) | $POOR_COUNT | ${PCT_POOR}% |" >> "$FULL_REPORT_FILE"
echo "| Very Poor (0-2) | $VERY_POOR_COUNT | ${PCT_VERY_POOR}% |" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Calculate health score
if [ "$QUALITY_TOTAL" -gt 0 ]; then
    WEIGHTED_SCORE=$((EXCELLENT_COUNT * 100 + GOOD_COUNT * 75 + ADEQUATE_COUNT * 50 + POOR_COUNT * 25))
    HEALTH_SCORE=$((WEIGHTED_SCORE / QUALITY_TOTAL))
else
    HEALTH_SCORE=0
fi

# Add health score to report
echo "### Documentation Health Score" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo "**Overall Documentation Health Score: $HEALTH_SCORE / 100**" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

if [ "$HEALTH_SCORE" -ge 90 ]; then
    echo "🟢 **Excellent** - Documentation is in exceptional condition with minimal issues." >> "$FULL_REPORT_FILE"
elif [ "$HEALTH_SCORE" -ge 75 ]; then
    echo "🟢 **Good** - Documentation is generally high quality with room for some improvements." >> "$FULL_REPORT_FILE"
elif [ "$HEALTH_SCORE" -ge 60 ]; then
    echo "🟡 **Adequate** - Documentation has significant areas that need improvement." >> "$FULL_REPORT_FILE"
elif [ "$HEALTH_SCORE" -ge 40 ]; then
    echo "🟠 **Below Average** - Documentation has major quality issues that need to be addressed." >> "$FULL_REPORT_FILE"
else
    echo "🔴 **Poor** - Documentation requires immediate attention and significant rework." >> "$FULL_REPORT_FILE"
fi
echo "" >> "$FULL_REPORT_FILE"

# Add readability metrics to report
echo "### Readability Assessment" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"
echo "| Readability Level | Count | Percentage |" >> "$FULL_REPORT_FILE"
echo "|-------------------|-------|------------|" >> "$FULL_REPORT_FILE"

# Calculate readability percentages safely
READABILITY_TOTAL=$((COMPLEX_COUNT + GOOD_READABILITY_COUNT + SIMPLE_COUNT))
if [ $READABILITY_TOTAL -eq 0 ]; then
    READABILITY_TOTAL=1  # Avoid division by zero
fi

PCT_COMPLEX=$((COMPLEX_COUNT * 100 / READABILITY_TOTAL))
PCT_GOOD_READ=$((GOOD_READABILITY_COUNT * 100 / READABILITY_TOTAL))
PCT_SIMPLE=$((SIMPLE_COUNT * 100 / READABILITY_TOTAL))

echo "| Complex | $COMPLEX_COUNT | ${PCT_COMPLEX}% |" >> "$FULL_REPORT_FILE"
echo "| Good | $GOOD_READABILITY_COUNT | ${PCT_GOOD_READ}% |" >> "$FULL_REPORT_FILE"
echo "| Simple | $SIMPLE_COUNT | ${PCT_SIMPLE}% |" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Add recommendations based on results
echo "## Improvement Recommendations" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Add quality recommendations
echo "### Priority Actions" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Initialize recommendation counter
RECOMMENDATION_COUNT=0

# Link issues recommendations
if [ "$BROKEN_LINKS" -gt 0 ]; then
    RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
    echo "$RECOMMENDATION_COUNT. **Fix Broken Links**: Fix the $BROKEN_LINKS broken internal links identified in the link validation section." >> "$FULL_REPORT_FILE"
fi

# Frontmatter recommendations
if [ "$FRONTMATTER_ISSUES" != "" ] && [ "$FRONTMATTER_ISSUES" != "0" ]; then
    RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
    echo "$RECOMMENDATION_COUNT. **Fix Frontmatter Issues**: Address the $FRONTMATTER_ISSUES documents with frontmatter problems." >> "$FULL_REPORT_FILE"
    echo "   - Run: \`.devtools/scripts/doc-overhaul/fix_frontmatter.sh\` to automatically fix common issues." >> "$FULL_REPORT_FILE"
fi

# Quality recommendations
NEED_QUALITY_IMPROVEMENT=0
if [ "$POOR_COUNT" -gt 0 ]; then
    NEED_QUALITY_IMPROVEMENT=1
fi
if [ "$VERY_POOR_COUNT" -gt 0 ]; then
    NEED_QUALITY_IMPROVEMENT=1
fi

if [ "$NEED_QUALITY_IMPROVEMENT" -eq 1 ]; then
    RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
    LOW_QUALITY_COUNT=$((POOR_COUNT + VERY_POOR_COUNT))
    echo "$RECOMMENDATION_COUNT. **Improve Low Quality Content**: Prioritize improving the $LOW_QUALITY_COUNT documents rated as 'Poor' or 'Very Poor'." >> "$FULL_REPORT_FILE"
    echo "   - Use: \`.devtools/scripts/doc-overhaul/comprehensive_test.sh\` to identify lowest quality documents." >> "$FULL_REPORT_FILE"
fi

# Readability recommendations
if [ "$COMPLEX_COUNT" -gt 0 ]; then
    RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
    echo "$RECOMMENDATION_COUNT. **Improve Document Readability**: Simplify content in documents with 'Complex' readability ratings." >> "$FULL_REPORT_FILE"
    echo "   - Break long sentences into shorter ones" >> "$FULL_REPORT_FILE"
    echo "   - Use simpler language where possible" >> "$FULL_REPORT_FILE"
    echo "   - Add more section headers to break up content" >> "$FULL_REPORT_FILE"
fi

# Code validation recommendations
if [ "$CODE_FAIL_COUNT" -gt 0 ]; then
    RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
    echo "$RECOMMENDATION_COUNT. **Fix Code Examples**: Ensure code examples in documentation are syntactically valid." >> "$FULL_REPORT_FILE"
    echo "   - Found $CODE_FAIL_COUNT documents with invalid code blocks" >> "$FULL_REPORT_FILE"
fi

# General recommendations
RECOMMENDATION_COUNT=$((RECOMMENDATION_COUNT + 1))
echo "$RECOMMENDATION_COUNT. **Regular Maintenance**: Schedule regular documentation reviews and updates." >> "$FULL_REPORT_FILE"
echo "   - Run this report weekly or monthly" >> "$FULL_REPORT_FILE"
echo "   - Add documentation checks to CI pipeline" >> "$FULL_REPORT_FILE"
echo "   - Assign documentation maintenance responsibilities" >> "$FULL_REPORT_FILE"
echo "" >> "$FULL_REPORT_FILE"

# Generate HTML version if pandoc is available
if command -v pandoc > /dev/null 2>&1; then
    log_info "Generating HTML report with pandoc..."
    pandoc "$FULL_REPORT_FILE" -f markdown -t html -s -o "$HTML_REPORT_FILE" || true
    
    if [ -f "$HTML_REPORT_FILE" ]; then
        log_success "HTML report generated: $HTML_REPORT_FILE"
    else
        log_warning "Failed to generate HTML report. Please install pandoc or check for errors."
    fi
fi

# Final summary
log_success "Documentation quality report generated: $FULL_REPORT_FILE"
echo "Documentation Health Score: $HEALTH_SCORE / 100"

if [ "$HEALTH_SCORE" -ge "$CI_THRESHOLD" ]; then
    log_success "Documentation quality meets CI threshold ($CI_THRESHOLD)."
    exit 0
else
    log_error "Documentation quality below CI threshold ($CI_THRESHOLD)."
    log_error "Please address issues outlined in the report."
    exit 1
fi 
#!/bin/bash

# Documentation Report Generator
# This script generates a comprehensive documentation quality report by running
# both basic validation (syntax, links, frontmatter) and advanced analysis (relationships, quality, etc.)

set -e

SCRIPT_DIR="$(dirname "$0")"
DOCS_DIR="docs"
REPORTS_DIR="target/reports/docs_validation"
DATE_STAMP=$(date "+%Y-%m-%d")
FULL_REPORT_FILE="${REPORTS_DIR}/documentation_quality_report_${DATE_STAMP}.md"
HTML_REPORT_FILE="${REPORTS_DIR}/documentation_quality_report_${DATE_STAMP}.html"

# Command-line options
DIR_TO_ANALYZE=""
SINGLE_FILE=""
FULL_VISUALIZATION=false
SKIP_LINTING=false
CONFIG_FILE=".devtools/config/docs_report_config.json"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dir)
            DIR_TO_ANALYZE="$2"
            shift 2
            ;;
        --file)
            SINGLE_FILE="$2"
            shift 2
            ;;
        --vis)
            FULL_VISUALIZATION=true
            shift
            ;;
        --skip-linting)
            SKIP_LINTING=true
            shift
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --help)
            echo "Usage: generate_report.sh [OPTIONS]"
            echo "Options:"
            echo "  --dir DIRECTORY   Focus analysis on a specific directory"
            echo "  --file FILE       Analyze a single file only"
            echo "  --vis             Generate full visualization"
            echo "  --skip-linting    Skip markdown linting (faster)"
            echo "  --config FILE     Specify custom configuration file"
            echo "  --help            Display this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Create reports directory if it doesn't exist
mkdir -p "$REPORTS_DIR"

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
INCLUDE_CODE_VALIDATION=true
INCLUDE_READABILITY=true
INCLUDE_VISUALIZATION=true
MINIMUM_QUALITY_THRESHOLD="Good"
CI_THRESHOLD=70

# Load configuration if file exists and jq is available
if [ -f "$CONFIG_FILE" ] && command -v jq &> /dev/null; then
    echo -e "${BLUE}Loading configuration from $CONFIG_FILE...${NC}"
    INCLUDE_CODE_VALIDATION=$(jq -r '.include_code_validation // true' "$CONFIG_FILE")
    INCLUDE_READABILITY=$(jq -r '.include_readability // true' "$CONFIG_FILE")
    INCLUDE_VISUALIZATION=$(jq -r '.include_visualization // true' "$CONFIG_FILE")
    MINIMUM_QUALITY_THRESHOLD=$(jq -r '.minimum_quality_threshold // "Good"' "$CONFIG_FILE")
    CI_THRESHOLD=$(jq -r '.ci_threshold // 70' "$CONFIG_FILE")
fi

# Set up options for comprehensive_test.sh
COMP_TEST_OPTS=""
if [ -n "$DIR_TO_ANALYZE" ]; then
    COMP_TEST_OPTS="--dir $DIR_TO_ANALYZE"
    DOCS_DIR="$DIR_TO_ANALYZE"
    echo -e "${BLUE}Analyzing directory: $DIR_TO_ANALYZE${NC}"
elif [ -n "$SINGLE_FILE" ]; then
    COMP_TEST_OPTS="--file $SINGLE_FILE"
    echo -e "${BLUE}Analyzing single file: $SINGLE_FILE${NC}"
fi

echo -e "${BLUE}Generating comprehensive documentation quality report...${NC}"

# Start building the report
echo "# Navius Documentation Quality Report" > $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "**Generated:** $(date '+%B %d, %Y at %H:%M:%S')" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

if [ -n "$DIR_TO_ANALYZE" ]; then
    echo "**Scope:** Directory - $DIR_TO_ANALYZE" >> $FULL_REPORT_FILE
elif [ -n "$SINGLE_FILE" ]; then
    echo "**Scope:** Single file - $SINGLE_FILE" >> $FULL_REPORT_FILE
else
    echo "**Scope:** All documentation" >> $FULL_REPORT_FILE
fi
echo "" >> $FULL_REPORT_FILE

echo "## Overview" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "This report provides a comprehensive assessment of the Navius documentation quality." >> $FULL_REPORT_FILE
echo "It combines both basic validation (syntax, links, formatting) and advanced analysis (document relationships, content quality, readability, code validation)." >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Count documents
if [ -n "$SINGLE_FILE" ]; then
    TOTAL_DOCS=1
else
    TOTAL_DOCS=$(find $DOCS_DIR -name "*.md" | wc -l)
fi

# Run markdown linting if not skipped
LINTING_ISSUES=0
if [ "$SKIP_LINTING" = false ]; then
    echo -e "${BLUE}Running markdown linting...${NC}"
    echo "## Markdown Linting" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    echo "```" >> $FULL_REPORT_FILE

    # Ensure markdownlint-cli is installed
    if ! command -v markdownlint &> /dev/null; then
        echo "markdownlint not found. Installing..."
        npm install -g markdownlint-cli
    fi

    # Run markdownlint and capture output
    if [ -n "$SINGLE_FILE" ]; then
        LINTING_OUTPUT=$(markdownlint "$SINGLE_FILE" --config .devtools/config/markdownlint.json 2>&1 || true)
    elif [ -n "$DIR_TO_ANALYZE" ]; then
        LINTING_OUTPUT=$(markdownlint "$DIR_TO_ANALYZE/**/*.md" --config .devtools/config/markdownlint.json 2>&1 || true)
    else
        LINTING_OUTPUT=$(markdownlint docs/**/*.md --config .devtools/config/markdownlint.json 2>&1 || true)
    fi
    echo "$LINTING_OUTPUT" >> $FULL_REPORT_FILE
    echo "```" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE

    # Count linting issues
    LINTING_ISSUES=$(echo "$LINTING_OUTPUT" | wc -l)
    if [ $LINTING_ISSUES -eq 0 ]; then
        echo "✅ No markdown linting issues found." >> $FULL_REPORT_FILE
    else
        echo "⚠️ Found $LINTING_ISSUES markdown linting issues." >> $FULL_REPORT_FILE
    fi
    echo "" >> $FULL_REPORT_FILE
fi

# Run link checker
echo -e "${BLUE}Checking for broken links...${NC}"
echo "## Link Validation" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Create a temporary file for link check results
LINK_CHECK_FILE=$(mktemp)
BROKEN_LINKS=0

echo "### Internal Links" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Run internal link checker using our own script
LINK_CHECK_OPTS="--check-only"
if [ -n "$SINGLE_FILE" ]; then
    LINK_CHECK_OPTS="$LINK_CHECK_OPTS --file $SINGLE_FILE"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    LINK_CHECK_OPTS="$LINK_CHECK_OPTS --dir $DIR_TO_ANALYZE"
fi

bash "$SCRIPT_DIR/fix_links.sh" $LINK_CHECK_OPTS > $LINK_CHECK_FILE 2>&1 || true
cat $LINK_CHECK_FILE >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Count broken internal links
BROKEN_INTERNAL=$(grep -c "Broken link" $LINK_CHECK_FILE || echo 0)
if [ $BROKEN_INTERNAL -eq 0 ]; then
    echo "✅ No broken internal links found." >> $FULL_REPORT_FILE
else
    BROKEN_LINKS=$((BROKEN_LINKS + BROKEN_INTERNAL))
    echo "⚠️ Found $BROKEN_INTERNAL broken internal links." >> $FULL_REPORT_FILE
fi
echo "" >> $FULL_REPORT_FILE

# Run frontmatter validation
echo -e "${BLUE}Validating frontmatter...${NC}"
echo "## Frontmatter Validation" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "```" >> $FULL_REPORT_FILE

# Run frontmatter validation script with proper options
FRONTMATTER_OPTS=""
if [ -n "$SINGLE_FILE" ]; then
    FRONTMATTER_OPTS="--file $SINGLE_FILE"
elif [ -n "$DIR_TO_ANALYZE" ]; then
    FRONTMATTER_OPTS="--dir $DIR_TO_ANALYZE"
fi

# Use fix_frontmatter.sh in validation mode
bash "$SCRIPT_DIR/fix_frontmatter.sh" --validate-all $FRONTMATTER_OPTS 2>&1 | tee $LINK_CHECK_FILE
cat $LINK_CHECK_FILE >> $FULL_REPORT_FILE
echo "```" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Count frontmatter issues
FRONTMATTER_ISSUES=$(grep -c "Error:" $LINK_CHECK_FILE || echo 0)
if [ $FRONTMATTER_ISSUES -eq 0 ]; then
    echo "✅ No frontmatter issues found." >> $FULL_REPORT_FILE
else
    echo "⚠️ Found $FRONTMATTER_ISSUES documents with frontmatter issues." >> $FULL_REPORT_FILE
fi
echo "" >> $FULL_REPORT_FILE

# Run comprehensive documentation testing with CSV output for metrics extraction
echo -e "${BLUE}Running comprehensive document analysis...${NC}"

# Generate CSV output for metrics
QUALITY_CSV=$(mktemp)
echo -e "${BLUE}Extracting quality metrics...${NC}"
bash "$SCRIPT_DIR/comprehensive_test.sh" $COMP_TEST_OPTS --csv > $QUALITY_CSV

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
AVG_WPS=0

# Initialize code validation metrics
CODE_PASS_COUNT=0
CODE_FAIL_COUNT=0
CODE_TOTAL=0
CODE_PASS_PERCENT=0

# Process CSV output to extract metrics if file is not empty
if [ -s "$QUALITY_CSV" ]; then
    # Skip header line
    tail -n +2 "$QUALITY_CSV" > "${QUALITY_CSV}.tmp"
    mv "${QUALITY_CSV}.tmp" "$QUALITY_CSV"
    
    # Extract quality counts
    EXCELLENT_COUNT=$(grep -c "Excellent" $QUALITY_CSV || echo 0)
    GOOD_COUNT=$(grep -c "Good" $QUALITY_CSV || echo 0)
    ADEQUATE_COUNT=$(grep -c "Adequate" $QUALITY_CSV || echo 0)
    POOR_COUNT=$(grep -c "Poor" $QUALITY_CSV || echo 0)
    VERY_POOR_COUNT=$(grep -c "Very Poor" $QUALITY_CSV || echo 0)
    
    # Extract readability metrics
    COMPLEX_COUNT=$(grep -c "Complex" $QUALITY_CSV || echo 0)
    GOOD_READABILITY_COUNT=$(grep -c "Good" $QUALITY_CSV || echo 0)
    SIMPLE_COUNT=$(grep -c "Simple" $QUALITY_CSV || echo 0)
    
    # Calculate average words per sentence if awk is available
    if command -v awk &> /dev/null; then
        WPS_SUM=$(awk -F',' '{sum+=$8} END {print sum}' $QUALITY_CSV 2>/dev/null || echo 0)
        WPS_COUNT=$(wc -l < $QUALITY_CSV)
        if [ $WPS_COUNT -gt 0 ]; then
            AVG_WPS=$(echo "scale=2; $WPS_SUM / $WPS_COUNT" | bc)
        fi
    fi
    
    # Extract code validation metrics
    CODE_PASS_COUNT=$(grep -c "PASS" $QUALITY_CSV || echo 0)
    CODE_FAIL_COUNT=$(grep -c "FAIL" $QUALITY_CSV || echo 0)
    CODE_TOTAL=$((CODE_PASS_COUNT + CODE_FAIL_COUNT))
    
    if [ $CODE_TOTAL -gt 0 ]; then
        CODE_PASS_PERCENT=$((CODE_PASS_COUNT * 100 / CODE_TOTAL))
    else
        CODE_PASS_PERCENT=100  # No code examples means perfect score
    fi
fi

# Run comprehensive test and capture the report path for detailed analysis
echo -e "${BLUE}Generating detailed document analysis...${NC}"
echo "## Advanced Document Analysis" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

COMPREHENSIVE_REPORT=$(bash "$SCRIPT_DIR/comprehensive_test.sh" $COMP_TEST_OPTS | grep -o "target/reports/docs_validation/comprehensive_test_.*\.md" || echo "")

if [ -n "$COMPREHENSIVE_REPORT" ] && [ -f "$COMPREHENSIVE_REPORT" ]; then
    # Extract only the content sections we want (skip the first 3 lines which are the title and empty line)
    sed -n '4,$p' "$COMPREHENSIVE_REPORT" >> $FULL_REPORT_FILE
else
    echo "⚠️ Could not generate comprehensive analysis report." >> $FULL_REPORT_FILE
fi

# Add content quality distribution section
echo -e "${BLUE}Adding quality metrics...${NC}"
echo "## Content Quality Distribution" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

if [ $TOTAL_DOCS -gt 0 ]; then
    EXCELLENT_PERCENT=$((EXCELLENT_COUNT * 100 / TOTAL_DOCS))
    GOOD_PERCENT=$((GOOD_COUNT * 100 / TOTAL_DOCS))
    ADEQUATE_PERCENT=$((ADEQUATE_COUNT * 100 / TOTAL_DOCS))
    POOR_PERCENT=$((POOR_COUNT * 100 / TOTAL_DOCS))
    VERY_POOR_PERCENT=$((VERY_POOR_COUNT * 100 / TOTAL_DOCS))
else
    EXCELLENT_PERCENT=0
    GOOD_PERCENT=0
    ADEQUATE_PERCENT=0
    POOR_PERCENT=0
    VERY_POOR_PERCENT=0
fi

echo "| Quality Level | Count | Percentage |" >> $FULL_REPORT_FILE
echo "|---------------|-------|------------|" >> $FULL_REPORT_FILE
echo "| Excellent | $EXCELLENT_COUNT | $EXCELLENT_PERCENT% |" >> $FULL_REPORT_FILE
echo "| Good | $GOOD_COUNT | $GOOD_PERCENT% |" >> $FULL_REPORT_FILE
echo "| Adequate | $ADEQUATE_COUNT | $ADEQUATE_PERCENT% |" >> $FULL_REPORT_FILE
echo "| Poor | $POOR_COUNT | $POOR_PERCENT% |" >> $FULL_REPORT_FILE
echo "| Very Poor | $VERY_POOR_COUNT | $VERY_POOR_PERCENT% |" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Add readability metrics if included
if [ "$INCLUDE_READABILITY" = true ]; then
    echo -e "${BLUE}Adding readability metrics...${NC}"
    echo "## Readability Analysis" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    
    if [ $TOTAL_DOCS -gt 0 ]; then
        COMPLEX_PERCENT=$((COMPLEX_COUNT * 100 / TOTAL_DOCS))
        GOOD_READABILITY_PERCENT=$((GOOD_READABILITY_COUNT * 100 / TOTAL_DOCS))
        SIMPLE_PERCENT=$((SIMPLE_COUNT * 100 / TOTAL_DOCS))
    else
        COMPLEX_PERCENT=0
        GOOD_READABILITY_PERCENT=0
        SIMPLE_PERCENT=0
    fi
    
    echo "| Readability Level | Count | Percentage |" >> $FULL_REPORT_FILE
    echo "|-------------------|-------|------------|" >> $FULL_REPORT_FILE
    echo "| Complex | $COMPLEX_COUNT | $COMPLEX_PERCENT% |" >> $FULL_REPORT_FILE
    echo "| Good | $GOOD_READABILITY_COUNT | $GOOD_READABILITY_PERCENT% |" >> $FULL_REPORT_FILE
    echo "| Simple | $SIMPLE_COUNT | $SIMPLE_PERCENT% |" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    echo "- **Average Words Per Sentence:** $AVG_WPS" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
fi

# Add code validation results if included
if [ "$INCLUDE_CODE_VALIDATION" = true ]; then
    echo -e "${BLUE}Adding code validation results...${NC}"
    echo "## Code Example Validation" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    
    echo "- **Code Examples:** $CODE_TOTAL total" >> $FULL_REPORT_FILE
    echo "- **Passing:** $CODE_PASS_COUNT ($CODE_PASS_PERCENT%)" >> $FULL_REPORT_FILE
    echo "- **Failing:** $CODE_FAIL_COUNT ($((100 - CODE_PASS_PERCENT))%)" >> $FULL_REPORT_FILE
    
    # List documents with failing code examples
    if [ $CODE_FAIL_COUNT -gt 0 ]; then
        echo "" >> $FULL_REPORT_FILE
        echo "### Documents with Failing Code Examples" >> $FULL_REPORT_FILE
        echo "" >> $FULL_REPORT_FILE
        grep "FAIL" $QUALITY_CSV | cut -d',' -f1,2 | sed 's/,/ - /' >> $FULL_REPORT_FILE
    fi
    echo "" >> $FULL_REPORT_FILE
fi

# Include document relationship visualization if enabled
if [ "$INCLUDE_VISUALIZATION" = true ] || [ "$FULL_VISUALIZATION" = true ]; then
    echo -e "${BLUE}Adding document relationship visualization...${NC}"
    
    # Find the latest graph visualization
    GRAPH_HTML=$(find "$REPORTS_DIR" -name "document_graph_*.html" -type f -exec ls -t {} \; | head -1)
    
    if [ -n "$GRAPH_HTML" ] && [ -f "$GRAPH_HTML" ]; then
        echo "## Document Relationship Visualization" >> $FULL_REPORT_FILE
        echo "" >> $FULL_REPORT_FILE
        echo "Document relationship visualization is available [here]($(basename $GRAPH_HTML))" >> $FULL_REPORT_FILE
        echo "" >> $FULL_REPORT_FILE
        
        # If generating HTML report, embed the SVG directly
        if command -v pandoc &> /dev/null; then
            GRAPH_SVG=$(find "$REPORTS_DIR" -name "document_graph_*.svg" -type f -exec ls -t {} \; | head -1)
            if [ -n "$GRAPH_SVG" ] && [ -f "$GRAPH_SVG" ]; then
                cp "$GRAPH_SVG" "$(dirname $HTML_REPORT_FILE)/$(basename $GRAPH_SVG)"
            fi
        fi
    fi
fi

# Include AI-assisted improvement recommendations
echo -e "${BLUE}Adding improvement recommendations...${NC}"
echo "## Top Improvement Recommendations" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

if [ -n "$COMPREHENSIVE_REPORT" ] && [ -f "$COMPREHENSIVE_REPORT" ]; then
    # Extract the improvement recommendations section
    sed -n '/^## Improvement Recommendations/,/^## /p' "$COMPREHENSIVE_REPORT" | sed '$ d' >> $FULL_REPORT_FILE
else
    echo "No improvement recommendations available." >> $FULL_REPORT_FILE
fi
echo "" >> $FULL_REPORT_FILE

# Add historical trend tracking
HISTORY_FILE="${REPORTS_DIR}/documentation_metrics_history.csv"

# Create the history file if it doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "Date,TotalDocs,HealthScore,LintingIssues,BrokenLinks,FrontmatterIssues,Excellent,Good,Adequate,Poor,VeryPoor,CodePass,CodeFail" > "$HISTORY_FILE"
fi

# Generate executive summary
echo -e "${BLUE}Generating executive summary...${NC}"

# Calculate health score based on issues found
# This is a more comprehensive scoring system including new metrics
HEALTH_SCORE=100

# Basic issues impact
if [ $LINTING_ISSUES -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - LINTING_ISSUES / 2))
fi
if [ $BROKEN_LINKS -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - BROKEN_LINKS * 5))
fi
if [ $FRONTMATTER_ISSUES -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - FRONTMATTER_ISSUES * 3))
fi

# Quality metrics impact
if [ $TOTAL_DOCS -gt 0 ]; then
    # Quality distribution impact (weighted)
    QUALITY_SCORE=$((EXCELLENT_COUNT * 5 + GOOD_COUNT * 3 + ADEQUATE_COUNT * 1))
    MAX_QUALITY_SCORE=$((TOTAL_DOCS * 5))
    if [ $MAX_QUALITY_SCORE -gt 0 ]; then
        QUALITY_PERCENT=$((QUALITY_SCORE * 100 / MAX_QUALITY_SCORE))
        QUALITY_IMPACT=$((100 - QUALITY_PERCENT))
        HEALTH_SCORE=$((HEALTH_SCORE - QUALITY_IMPACT / 5))
    fi
    
    # Code validation impact
    if [ $CODE_TOTAL -gt 0 ]; then
        CODE_IMPACT=$((100 - CODE_PASS_PERCENT))
        HEALTH_SCORE=$((HEALTH_SCORE - CODE_IMPACT / 10))
    fi
    
    # Readability impact
    if [ $TOTAL_DOCS -gt 0 ]; then
        READABILITY_PERCENT=$((GOOD_READABILITY_COUNT * 100 / TOTAL_DOCS))
        READABILITY_IMPACT=$((100 - READABILITY_PERCENT))
        HEALTH_SCORE=$((HEALTH_SCORE - READABILITY_IMPACT / 10))
    fi
fi

# Ensure score doesn't go below 0
if [ $HEALTH_SCORE -lt 0 ]; then
    HEALTH_SCORE=0
fi

# Ensure score doesn't go above 100
if [ $HEALTH_SCORE -gt 100 ]; then
    HEALTH_SCORE=100
fi

# Determine health rating
if [ $HEALTH_SCORE -ge 90 ]; then
    HEALTH_RATING="Excellent 🟢"
elif [ $HEALTH_SCORE -ge 70 ]; then
    HEALTH_RATING="Good 🟡"
elif [ $HEALTH_SCORE -ge 50 ]; then
    HEALTH_RATING="Fair 🟠"
else
    HEALTH_RATING="Poor 🔴"
fi

# Add today's metrics to the history file
if [ -z "$SINGLE_FILE" ]; then
    echo "$DATE_STAMP,$TOTAL_DOCS,$HEALTH_SCORE,$LINTING_ISSUES,$BROKEN_LINKS,$FRONTMATTER_ISSUES,$EXCELLENT_COUNT,$GOOD_COUNT,$ADEQUATE_COUNT,$POOR_COUNT,$VERY_POOR_COUNT,$CODE_PASS_COUNT,$CODE_FAIL_COUNT" >> "$HISTORY_FILE"
fi

# Generate trend visualization if gnuplot is available and we have enough data
if command -v gnuplot &> /dev/null && [ $(wc -l < "$HISTORY_FILE") -gt 2 ] && [ -z "$SINGLE_FILE" ]; then
    echo -e "${BLUE}Generating trend visualization...${NC}"
    TREND_PLOT="${REPORTS_DIR}/quality_trend_${DATE_STAMP}.png"
    
    gnuplot <<EOF
    set terminal png size 800,400
    set output "$TREND_PLOT"
    set title "Documentation Quality Trends"
    set xlabel "Date"
    set ylabel "Score"
    set xdata time
    set timefmt "%Y-%m-%d"
    set format x "%y-%m-%d"
    set key outside
    set grid
    
    plot "$HISTORY_FILE" using 1:3 with lines title "Health Score", \
         "$HISTORY_FILE" using 1:7 with lines title "Excellent Docs", \
         "$HISTORY_FILE" using 1:12 with lines title "Code Pass Rate"
EOF

    echo "## Quality Trends" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    echo "![Quality Trends]($(basename $TREND_PLOT))" >> $FULL_REPORT_FILE
    echo "" >> $FULL_REPORT_FILE
    
    if command -v pandoc &> /dev/null; then
        cp "$TREND_PLOT" "$(dirname $HTML_REPORT_FILE)/$(basename $TREND_PLOT)"
    fi
fi

# Add executive summary at the top
EXEC_SUMMARY=$(mktemp)
echo "## Executive Summary" > $EXEC_SUMMARY
echo "" >> $EXEC_SUMMARY

# Basic metrics
echo "- **Total Documents:** $TOTAL_DOCS" >> $EXEC_SUMMARY
echo "- **Documentation Health Score:** $HEALTH_SCORE/100 ($HEALTH_RATING)" >> $EXEC_SUMMARY
echo "- **Linting Issues:** $LINTING_ISSUES" >> $EXEC_SUMMARY
echo "- **Broken Links:** $BROKEN_LINKS" >> $EXEC_SUMMARY
echo "- **Frontmatter Issues:** $FRONTMATTER_ISSUES" >> $EXEC_SUMMARY

# Quality metrics
echo "- **Quality Distribution:** $EXCELLENT_COUNT Excellent, $GOOD_COUNT Good, $ADEQUATE_COUNT Adequate, $POOR_COUNT Poor, $VERY_POOR_COUNT Very Poor" >> $EXEC_SUMMARY

# Code validation metrics if enabled
if [ "$INCLUDE_CODE_VALIDATION" = true ]; then
    echo "- **Code Validation:** $CODE_PASS_COUNT/$CODE_TOTAL passing ($CODE_PASS_PERCENT%)" >> $EXEC_SUMMARY
fi

# Readability metrics if enabled
if [ "$INCLUDE_READABILITY" = true ]; then
    echo "- **Readability:** $GOOD_READABILITY_COUNT/$TOTAL_DOCS documents with good readability ($GOOD_READABILITY_PERCENT%)" >> $EXEC_SUMMARY
fi

echo "" >> $EXEC_SUMMARY

# Insert actions needed if there are issues
if [ $LINTING_ISSUES -gt 0 ] || [ $BROKEN_LINKS -gt 0 ] || [ $FRONTMATTER_ISSUES -gt 0 ] || [ $POOR_COUNT -gt 0 ] || [ $VERY_POOR_COUNT -gt 0 ] || [ $CODE_FAIL_COUNT -gt 0 ]; then
    echo "### Recommended Actions" >> $EXEC_SUMMARY
    echo "" >> $EXEC_SUMMARY
    
    # Critical issues first
    if [ $BROKEN_LINKS -gt 0 ]; then
        echo "- **High Priority:** Fix $BROKEN_LINKS broken links using fix_links.sh" >> $EXEC_SUMMARY
    fi
    
    if [ $CODE_FAIL_COUNT -gt 0 ]; then
        echo "- **High Priority:** Fix $CODE_FAIL_COUNT failing code examples" >> $EXEC_SUMMARY
    fi
    
    if [ $POOR_COUNT -gt 0 ] || [ $VERY_POOR_COUNT -gt 0 ]; then
        TOTAL_POOR=$((POOR_COUNT + VERY_POOR_COUNT))
        echo "- **Medium Priority:** Improve $TOTAL_POOR documents with poor/very poor quality scores" >> $EXEC_SUMMARY
    fi
    
    if [ $FRONTMATTER_ISSUES -gt 0 ]; then
        echo "- **Medium Priority:** Fix $FRONTMATTER_ISSUES frontmatter issues using fix_frontmatter.sh" >> $EXEC_SUMMARY
    fi
    
    if [ $COMPLEX_COUNT -gt 0 ]; then
        echo "- **Low Priority:** Simplify $COMPLEX_COUNT documents with complex readability" >> $EXEC_SUMMARY
    fi
    
    if [ $LINTING_ISSUES -gt 0 ]; then
        echo "- **Low Priority:** Address $LINTING_ISSUES markdown linting issues" >> $EXEC_SUMMARY
    fi
    
    echo "" >> $EXEC_SUMMARY
fi

# Add trend information
echo "### Trends" >> $EXEC_SUMMARY
echo "" >> $EXEC_SUMMARY
if [ $(wc -l < "$HISTORY_FILE") -gt 2 ] && [ -z "$SINGLE_FILE" ]; then
    echo "See the Quality Trends section for historical quality metrics." >> $EXEC_SUMMARY
else
    echo "This is the current documentation quality snapshot. Run this report regularly to track improvements." >> $EXEC_SUMMARY
fi
echo "" >> $EXEC_SUMMARY

# Insert the executive summary at the beginning of the report
sed -i '' -e "/^# Navius Documentation Quality Report/r $EXEC_SUMMARY" $FULL_REPORT_FILE

# Generate HTML version if pandoc is available
if command -v pandoc &> /dev/null; then
    echo -e "${BLUE}Generating HTML report...${NC}"
    pandoc -s --toc -c https://cdn.jsdelivr.net/npm/water.css@2/out/water.css -o $HTML_REPORT_FILE $FULL_REPORT_FILE
    HTML_URL="file://$(pwd)/$HTML_REPORT_FILE"
    echo -e "${GREEN}HTML report generated: ${BLUE}$HTML_URL${NC}"
fi

# Clean up temporary files
rm -f $LINK_CHECK_FILE $EXEC_SUMMARY $QUALITY_CSV

# Generate report URL
REPORT_URL="file://$(pwd)/$FULL_REPORT_FILE"
echo -e "${GREEN}Documentation quality report completed.${NC}"
echo -e "${GREEN}Report saved to: ${BLUE}$FULL_REPORT_FILE${NC}"
echo -e "${GREEN}View the report at: ${BLUE}$REPORT_URL${NC}"

# CI integration - set exit code based on health score if in CI environment
if [ "${CI:-false}" = "true" ]; then
    # Simplified output for CI
    echo "::group::Documentation Quality Report"
    echo "Health Score: $HEALTH_SCORE/100 ($HEALTH_RATING)"
    echo "Quality Distribution: $EXCELLENT_COUNT Excellent, $GOOD_COUNT Good, $ADEQUATE_COUNT Adequate, $POOR_COUNT Poor, $VERY_POOR_COUNT Very Poor"
    echo "Code Validation: $CODE_PASS_PERCENT% passing"
    echo "::endgroup::"
    
    # Set exit code based on health score threshold
    if [ $HEALTH_SCORE -lt $CI_THRESHOLD ]; then
        echo "::error::Documentation quality score ($HEALTH_SCORE) is below threshold ($CI_THRESHOLD)"
        exit 1
    fi
fi

# Return success status
exit 0 
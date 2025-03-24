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

# Create reports directory if it doesn't exist
mkdir -p "$REPORTS_DIR"

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Generating comprehensive documentation quality report...${NC}"

# Start building the report
echo "# Navius Documentation Quality Report" > $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "**Generated:** $(date '+%B %d, %Y at %H:%M:%S')" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "## Overview" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE
echo "This report provides a comprehensive assessment of the Navius documentation quality." >> $FULL_REPORT_FILE
echo "It combines both basic validation (syntax, links, formatting) and advanced analysis (document relationships, content quality, etc.)." >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Run markdown linting
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
LINTING_OUTPUT=$(markdownlint docs/**/*.md --config .devtools/config/markdownlint.json 2>&1 || true)
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
bash "$SCRIPT_DIR/fix_links.sh" --check-only > $LINK_CHECK_FILE 2>&1 || true
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

# Run frontmatter validation script
python3 .devtools/scripts/validate_frontmatter.py 2>&1 | tee $LINK_CHECK_FILE
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

# Run comprehensive documentation testing
echo -e "${BLUE}Running comprehensive document analysis...${NC}"
echo "## Advanced Document Analysis" >> $FULL_REPORT_FILE
echo "" >> $FULL_REPORT_FILE

# Run the comprehensive test script and capture the report path
COMPREHENSIVE_REPORT=$(bash "$SCRIPT_DIR/comprehensive_test.sh" | grep -o "target/reports/docs_validation/comprehensive_test_.*\.md" || echo "")

if [ -n "$COMPREHENSIVE_REPORT" ] && [ -f "$COMPREHENSIVE_REPORT" ]; then
    # Extract only the content sections we want (skip the first 3 lines which are the title and empty line)
    sed -n '4,$p' "$COMPREHENSIVE_REPORT" >> $FULL_REPORT_FILE
else
    echo "⚠️ Could not generate comprehensive analysis report." >> $FULL_REPORT_FILE
fi

# Generate summary
echo -e "${BLUE}Generating executive summary...${NC}"

# Add executive summary at the top
EXEC_SUMMARY=$(mktemp)
echo "## Executive Summary" > $EXEC_SUMMARY
echo "" >> $EXEC_SUMMARY

# Count documents
TOTAL_DOCS=$(find $DOCS_DIR -name "*.md" | wc -l)
echo "- **Total Documents:** $TOTAL_DOCS" >> $EXEC_SUMMARY

# Calculate health score based on issues found
# This is a simple scoring system - adjust weights as needed
HEALTH_SCORE=100
if [ $LINTING_ISSUES -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - LINTING_ISSUES / 2))
fi
if [ $BROKEN_LINKS -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - BROKEN_LINKS * 5))
fi
if [ $FRONTMATTER_ISSUES -gt 0 ]; then
    HEALTH_SCORE=$((HEALTH_SCORE - FRONTMATTER_ISSUES * 3))
fi

# Ensure score doesn't go below 0
if [ $HEALTH_SCORE -lt 0 ]; then
    HEALTH_SCORE=0
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

echo "- **Documentation Health Score:** $HEALTH_SCORE/100 ($HEALTH_RATING)" >> $EXEC_SUMMARY
echo "- **Linting Issues:** $LINTING_ISSUES" >> $EXEC_SUMMARY
echo "- **Broken Links:** $BROKEN_LINKS" >> $EXEC_SUMMARY
echo "- **Frontmatter Issues:** $FRONTMATTER_ISSUES" >> $EXEC_SUMMARY
echo "" >> $EXEC_SUMMARY

# Insert actions needed if there are issues
if [ $LINTING_ISSUES -gt 0 ] || [ $BROKEN_LINKS -gt 0 ] || [ $FRONTMATTER_ISSUES -gt 0 ]; then
    echo "### Recommended Actions" >> $EXEC_SUMMARY
    echo "" >> $EXEC_SUMMARY
    
    if [ $BROKEN_LINKS -gt 0 ]; then
        echo "- **High Priority:** Fix $BROKEN_LINKS broken links using fix_links.sh" >> $EXEC_SUMMARY
    fi
    
    if [ $FRONTMATTER_ISSUES -gt 0 ]; then
        echo "- **Medium Priority:** Fix $FRONTMATTER_ISSUES frontmatter issues using fix_frontmatter.sh" >> $EXEC_SUMMARY
    fi
    
    if [ $LINTING_ISSUES -gt 0 ]; then
        echo "- **Low Priority:** Address $LINTING_ISSUES markdown linting issues" >> $EXEC_SUMMARY
    fi
    
    echo "" >> $EXEC_SUMMARY
fi

# Add trend information (this is a placeholder - in production this would compare with previous reports)
echo "### Trends" >> $EXEC_SUMMARY
echo "" >> $EXEC_SUMMARY
echo "This is the current documentation quality snapshot. Run this report regularly to track improvements." >> $EXEC_SUMMARY
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
rm -f $LINK_CHECK_FILE $EXEC_SUMMARY

# Generate report URL
REPORT_URL="file://$(pwd)/$FULL_REPORT_FILE"
echo -e "${GREEN}Documentation quality report completed.${NC}"
echo -e "${GREEN}Report saved to: ${BLUE}$FULL_REPORT_FILE${NC}"
echo -e "${GREEN}View the report at: ${BLUE}$REPORT_URL${NC}"

# Return success status
exit 0 
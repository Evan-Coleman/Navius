#!/bin/bash

# analyze-content-coverage.sh - Created April 5, 2025
# Analyzes content coverage in documentation sections to identify areas needing improvement

set -e

SCRIPT_DIR="$(dirname "$0")"
DOCS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="${SCRIPT_DIR}/reports/content"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create necessary directories
mkdir -p "${REPORT_DIR}"

# Log function
log() {
  echo "[INFO] $1"
}

# Analyze content coverage for a section
analyze_section() {
  local section="$1"
  local section_path="${DOCS_ROOT}/${section}"
  local report_file="${REPORT_DIR}/${section}-content.md"
  
  log "Analyzing content coverage for ${section}"
  
  # Check if directory exists
  if [ ! -d "${section_path}" ]; then
    log "Directory not found: ${section_path}"
    return 1
  fi
  
  # Get total number of files
  local total_files=$(find "${section_path}" -type f -name "*.md" | wc -l)
  
  # Get total content size
  local total_size=$(find "${section_path}" -type f -name "*.md" -exec cat {} \; | wc -c)
  
  # Set minimum expected values based on section
  local min_size=10000
  local min_files=5
  
  case "$section" in
    "01_getting_started")
      min_size=10000
      min_files=5
      expected_subsections="introduction installation quickstart configuration"
      ;;
    "02_examples")
      min_size=30000
      min_files=15
      expected_subsections="basic-application database api authentication middleware dependency-injection error-handling"
      ;;
    "03_contributing")
      min_size=15000
      min_files=10
      expected_subsections="style-guide code-review-process testing-guidelines documentation-standards"
      ;;
    "04_guides")
      min_size=20000
      min_files=8
      expected_subsections="deployment security performance optimization migrations"
      ;;
    "05_reference")
      min_size=50000
      min_files=20
      expected_subsections="api auth standards config"
      ;;
  esac
  
  # Calculate coverage percentages
  local size_coverage=$(( total_size * 100 / min_size ))
  if [ "${size_coverage}" -gt 100 ]; then
    size_coverage=100
  fi
  
  local file_coverage=$(( total_files * 100 / min_files ))
  if [ "${file_coverage}" -gt 100 ]; then
    file_coverage=100
  fi
  
  # Check subsections
  local missing_subsections=""
  local found_subsections=""
  local found_subsection_count=0
  
  for subsection in ${expected_subsections}; do
    if find "${section_path}" -type f -name "*${subsection}*.md" | grep -q .; then
      found_subsections="${found_subsections} ${subsection}"
      found_subsection_count=$((found_subsection_count + 1))
    else
      missing_subsections="${missing_subsections} ${subsection}"
    fi
  done
  
  # Calculate subsection coverage
  local total_subsections=$(echo ${expected_subsections} | wc -w | tr -d ' ')
  local subsection_coverage=0
  if [ "${total_subsections}" -gt 0 ]; then
    subsection_coverage=$(( found_subsection_count * 100 / total_subsections ))
  fi
  
  # Calculate overall coverage (weighted)
  local overall_coverage=$(( (size_coverage + file_coverage + subsection_coverage) / 3 ))
  
  # Find shortest files (potentially needing expansion)
  local shortest_files=$(find "${section_path}" -type f -name "*.md" -exec wc -c {} \; | sort -n | head -5)
  
  # Generate report
  cat > "${report_file}" << EOF
---
title: "${section} Content Coverage Analysis"
description: "Detailed analysis of content coverage in ${section} documentation section"
category: "Documentation"
tags: ["content", "analysis", "${section}", "documentation", "coverage"]
last_updated: "$(date +"%B %d, %Y")"
---

# ${section} Content Coverage Analysis

## Overview

This report analyzes content coverage for the \`${section}\` documentation section, identifying areas that need improvement.

## Summary Metrics

| Metric | Value | Target | Coverage | Status |
|--------|-------|--------|----------|--------|
| Total Files | ${total_files} | ${min_files} | ${file_coverage}% | $(if [ ${file_coverage} -ge 95 ]; then echo "🟩 Excellent"; elif [ ${file_coverage} -ge 80 ]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| Total Content Size | ${total_size} chars | ${min_size} chars | ${size_coverage}% | $(if [ ${size_coverage} -ge 95 ]; then echo "🟩 Excellent"; elif [ ${size_coverage} -ge 80 ]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| Subsection Coverage | ${found_subsection_count}/${total_subsections} | ${total_subsections} | ${subsection_coverage}% | $(if [ ${subsection_coverage} -ge 95 ]; then echo "🟩 Excellent"; elif [ ${subsection_coverage} -ge 80 ]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| **Overall Coverage** | - | - | **${overall_coverage}%** | $(if [ ${overall_coverage} -ge 95 ]; then echo "🟩 Excellent"; elif [ ${overall_coverage} -ge 80 ]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |

## Missing Subsections

$(if [ -z "${missing_subsections}" ]; then
  echo "✅ All expected subsections are present."
else
  echo "The following expected subsections are missing:"
  for subsection in ${missing_subsections}; do
    echo "- \`${subsection}\`"
  done
fi)

## Shortest Files (Potential Expansion Candidates)

The following files have the least content and may need expansion:

\`\`\`
${shortest_files}
\`\`\`

## Recommendations

$(if [ ${overall_coverage} -lt 90 ]; then
  echo "This section requires significant content improvements:"
  if [ ${file_coverage} -lt 90 ]; then
    echo "- Add $(( min_files - total_files )) more files to reach the target of ${min_files} files"
  fi
  if [ ${size_coverage} -lt 90 ]; then
    echo "- Add approximately $(( min_size - total_size )) more characters of content"
  fi
  if [ ${subsection_coverage} -lt 100 ]; then
    echo "- Create documents for the missing subsections"
  fi
elif [ ${overall_coverage} -lt 95 ]; then
  echo "This section needs moderate content improvements:"
  if [ ${subsection_coverage} -lt 100 ]; then
    echo "- Create documents for the missing subsections"
  fi
  echo "- Expand the shortest files with more detailed content"
else
  echo "This section has good content coverage. Consider:"
  echo "- Reviewing existing content for quality and clarity"
  echo "- Adding more examples or diagrams to enhance understanding"
fi)

## Next Steps

1. $(if [ ${subsection_coverage} -lt 100 ]; then echo "Create content for missing subsections"; else echo "Review existing subsections for completeness"; fi)
2. Expand the shortest files with more comprehensive content
3. Review overall organization and flow of the section
4. Ensure all content follows the documentation standards

EOF
  
  log "Content analysis report generated for ${section} at ${report_file}"
  
  # Return the overall coverage percentage
  echo "${overall_coverage}"
}

# Generate summary report of all sections
generate_summary_report() {
  local report_file="${REPORT_DIR}/content-coverage-summary.md"
  
  log "Generating content coverage summary report"
  
  # Create the report header
  cat > "${report_file}" << EOF
---
title: "Documentation Content Coverage Summary"
description: "Overview of content coverage across all documentation sections"
category: "Documentation"
tags: ["content", "coverage", "documentation", "summary", "week2"]
last_updated: "$(date +"%B %d, %Y")"
---

# Documentation Content Coverage Summary

## Overview

This report summarizes content coverage across all documentation sections, identifying priorities for Week 2 content improvements.

## Section Coverage Summary

| Section | Files | Content Size | Subsections | Overall | Priority |
|---------|-------|--------------|-------------|---------|----------|
EOF
  
  # Variable to hold section data
  local high_priority=""
  local medium_priority=""
  local low_priority=""
  
  # Add section data to the summary
  for section in "01_getting_started" "02_examples" "03_contributing" "04_guides" "05_reference"; do
    local section_report="${REPORT_DIR}/${section}-content.md"
    if [ -f "${section_report}" ]; then
      local file_coverage=$(grep "Total Files" "${section_report}" | awk -F'|' '{print $4}' | sed 's/%//' | xargs)
      local size_coverage=$(grep "Total Content Size" "${section_report}" | awk -F'|' '{print $4}' | sed 's/%//' | xargs)
      local subsection_coverage=$(grep "Subsection Coverage" "${section_report}" | awk -F'|' '{print $4}' | sed 's/%//' | xargs)
      local overall=$(grep "Overall Coverage" "${section_report}" | awk -F'|' '{print $4}' | sed 's/\*\*//g' | sed 's/%//' | xargs)
      
      local priority="Low"
      if [ "${overall}" -lt 80 ]; then
        priority="High"
        high_priority="${high_priority}- ${section} (${overall}% coverage)\n"
      elif [ "${overall}" -lt 90 ]; then
        priority="Medium"
        medium_priority="${medium_priority}- ${section} (${overall}% coverage)\n"
      else
        low_priority="${low_priority}- ${section} (${overall}% coverage)\n"
      fi
      
      echo "| ${section} | ${file_coverage}% | ${size_coverage}% | ${subsection_coverage}% | ${overall}% | ${priority} |" >> "${report_file}"
    fi
  done
  
  # Add the improvement plan
  cat >> "${report_file}" << EOF

## Week 2 Content Improvement Plan

### High Priority Sections

$(echo -e "${high_priority}")

### Medium Priority Sections

$(echo -e "${medium_priority}")

### Low Priority Sections

$(echo -e "${low_priority}")

## Content Improvement Tasks

1. **Create Missing Subsections**:
   - Create content for all missing subsections in high and medium priority sections
   - Focus especially on core functionality documentation

2. **Expand Shortest Files**:
   - Identify files with minimal content across all sections
   - Expand with more comprehensive explanations, examples, and diagrams

3. **Add Missing Files**:
   - For sections with low file coverage, create additional documentation files
   - Ensure new files follow consistent naming conventions and structure

4. **Enhance Existing Content**:
   - Review and improve clarity, completeness, and accuracy of existing content
   - Add diagrams, examples, and cross-references to enhance understanding

## Implementation Plan

| Day | Focus Area | Target Sections | Tasks |
|-----|------------|-----------------|-------|
| April 5 | Missing Subsections | High Priority Sections | Create content templates and begin filling in missing subsections |
| April 6 | API Documentation | 05_reference/api | Complete missing API reference documentation |
| April 7 | Content Expansion | Medium Priority Sections | Expand shortest files and add missing content |
| April 8 | Diagrams & Visuals | All Sections | Create and integrate technical diagrams |
| April 9-11 | Final Reviews | All Sections | Review and finalize content improvements |

## Success Metrics

Content improvement will be considered successful when:

1. All sections achieve at least 90% overall content coverage
2. All required subsections are present across all documentation
3. No file is less than 2KB in size (excluding index files)
4. Each example includes code samples, explanations, and usage guidelines
5. Documentation quality and clarity are consistently high across all sections

EOF
  
  log "Content coverage summary report generated at ${report_file}"
}

# Main execution
log "Starting content coverage analysis"

# Create array to store overall coverage for each section
declare SECTION_COVERAGE_01=""
declare SECTION_COVERAGE_02=""
declare SECTION_COVERAGE_03=""
declare SECTION_COVERAGE_04=""
declare SECTION_COVERAGE_05=""

# Analyze each section
for section in "01_getting_started" "02_examples" "03_contributing" "04_guides" "05_reference"; do
  coverage=$(analyze_section "${section}")
  case "$section" in
    "01_getting_started") SECTION_COVERAGE_01="${coverage}" ;;
    "02_examples") SECTION_COVERAGE_02="${coverage}" ;;
    "03_contributing") SECTION_COVERAGE_03="${coverage}" ;;
    "04_guides") SECTION_COVERAGE_04="${coverage}" ;;
    "05_reference") SECTION_COVERAGE_05="${coverage}" ;;
  esac
  log "${section} overall coverage: ${coverage}%"
  log "--------------------------------------------"
done

# Generate summary report
generate_summary_report

log "Content coverage analysis completed. Reports available in ${REPORT_DIR}"

exit 0 
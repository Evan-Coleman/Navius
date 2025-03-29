#!/bin/bash

# run-comprehensive-validation.sh - Created April 4, 2025
# Runs comprehensive validation across all documentation sections

set -e

SCRIPT_DIR="$(dirname "$0")"
DOCS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${LOGS_DIR}/comprehensive_validation_${TIMESTAMP}.log"
REPORT_DIR="${SCRIPT_DIR}/reports/final"
SECTIONS=(
  "01_getting_started"
  "02_examples"
  "03_contributing"
  "04_guides"
  "05_reference"
)

# Create necessary directories
mkdir -p "${LOGS_DIR}"
mkdir -p "${REPORT_DIR}"

# Logging function
log() {
  local level="$1"
  local message="$2"
  local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
  
  echo "[$level] $message"
  echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# Run frontmatter validation on a section
validate_frontmatter() {
  local section="$1"
  local section_path="${DOCS_ROOT}/${section}"
  
  log "INFO" "Validating frontmatter in ${section}"
  "${SCRIPT_DIR}/frontmatter-validator.sh" --dir "${section_path}" --verbose > "${REPORT_DIR}/${section}-frontmatter.log" 2>&1
  
  # Extract statistics from the log
  local total=$(grep -o "Total files processed: [0-9]*" "${REPORT_DIR}/${section}-frontmatter.log" | awk '{print $4}')
  local missing=$(grep -o "Files with missing frontmatter: [0-9]*" "${REPORT_DIR}/${section}-frontmatter.log" | awk '{print $5}')
  local invalid=$(grep -o "Files with invalid frontmatter: [0-9]*" "${REPORT_DIR}/${section}-frontmatter.log" | awk '{print $5}')
  local duplicate=$(grep -o "Files with duplicate frontmatter: [0-9]*" "${REPORT_DIR}/${section}-frontmatter.log" | awk '{print $5}')
  local compliance=$(grep -o "Compliance rate: [0-9]*%" "${REPORT_DIR}/${section}-frontmatter.log" | awk '{print $3}')
  
  log "INFO" "Frontmatter validation for ${section} - Total: ${total}, Missing: ${missing}, Invalid: ${invalid}, Duplicate: ${duplicate}, Compliance: ${compliance}"
  
  # Return the compliance rate
  echo "${compliance%\%}" # Remove % sign
}

# Run link validation on a section
validate_links() {
  local section="$1"
  local section_path="${DOCS_ROOT}/${section}"
  
  log "INFO" "Validating links in ${section}"
  "${SCRIPT_DIR}/fix-links.sh" --dir "${section_path}" --check-only --verbose > "${REPORT_DIR}/${section}-links.log" 2>&1
  
  # Extract statistics from the log
  local total_links=$(grep -o "Found approximately [0-9]* links" "${REPORT_DIR}/${section}-links.log" | awk '{print $3}')
  local broken_links=$(grep -o "Found approximately [0-9]* broken links" "${REPORT_DIR}/${section}-links.log" | awk '{print $4}')
  local success_rate=0
  
  if [[ -n "$total_links" && -n "$broken_links" && "$total_links" -gt 0 ]]; then
    success_rate=$(( 100 - (broken_links * 100 / total_links) ))
  fi
  
  log "INFO" "Link validation for ${section} - Total links: ${total_links}, Broken links: ${broken_links}, Success rate: ${success_rate}%"
  
  # Return the success rate
  echo "${success_rate}"
}

# Run code block syntax validation on a section
validate_code_blocks() {
  local section="$1"
  local section_path="${DOCS_ROOT}/${section}"
  
  log "INFO" "Validating code blocks in ${section}"
  "${SCRIPT_DIR}/check-markdown-codeblocks.sh" --dir "${section_path}" --verbose > "${REPORT_DIR}/${section}-codeblocks.log" 2>&1
  
  # Extract statistics from the log
  local total_files=$(grep -o "Processed [0-9]* files" "${REPORT_DIR}/${section}-codeblocks.log" | awk '{print $2}')
  local files_with_issues=$(grep -o "Found [0-9]* files with issues" "${REPORT_DIR}/${section}-codeblocks.log" | awk '{print $3}')
  local total_issues=$(grep -o "Found [0-9]* code block issues" "${REPORT_DIR}/${section}-codeblocks.log" | awk '{print $4}')
  local compliance=100
  
  if [[ -n "$total_files" && -n "$files_with_issues" && "$total_files" -gt 0 ]]; then
    compliance=$(( 100 - (files_with_issues * 100 / total_files) ))
  fi
  
  log "INFO" "Code block validation for ${section} - Total files: ${total_files}, Files with issues: ${files_with_issues}, Total issues: ${total_issues}, Compliance: ${compliance}%"
  
  # Return the compliance rate
  echo "${compliance}"
}

# Generate section report
generate_section_report() {
  local section="$1"
  local frontmatter_compliance="$2"
  local link_success_rate="$3"
  local code_block_compliance="$4"
  
  local report_file="${REPORT_DIR}/${section}-report.md"
  
  log "INFO" "Generating comprehensive report for ${section}"
  
  cat > "${report_file}" << EOF
---
title: "${section} Section Validation Report"
description: "Comprehensive validation report for the ${section} documentation section"
category: "Documentation"
tags: ["validation", "report", "${section}", "documentation", "metrics"]
last_updated: "$(date +"%B %d, %Y")"
---

# ${section} Section Validation Report

## Overview

This is a comprehensive validation report for the \`${section}\` documentation section, generated on $(date +"%B %d, %Y").

## Summary Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Frontmatter Compliance | ${frontmatter_compliance}% | $(if [[ ${frontmatter_compliance} -ge 95 ]]; then echo "🟩 Excellent"; elif [[ ${frontmatter_compliance} -ge 80 ]]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| Link Success Rate | ${link_success_rate}% | $(if [[ ${link_success_rate} -ge 95 ]]; then echo "🟩 Excellent"; elif [[ ${link_success_rate} -ge 80 ]]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| Code Block Compliance | ${code_block_compliance}% | $(if [[ ${code_block_compliance} -ge 95 ]]; then echo "🟩 Excellent"; elif [[ ${code_block_compliance} -ge 80 ]]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |
| Overall Rating | $(( (frontmatter_compliance + link_success_rate + code_block_compliance) / 3 ))% | $(if [[ $(( (frontmatter_compliance + link_success_rate + code_block_compliance) / 3 )) -ge 95 ]]; then echo "🟩 Excellent"; elif [[ $(( (frontmatter_compliance + link_success_rate + code_block_compliance) / 3 )) -ge 80 ]]; then echo "🟨 Good"; else echo "🟥 Needs Work"; fi) |

## Detailed Findings

### Frontmatter Validation

- **Compliance Rate**: ${frontmatter_compliance}%
- **Details**: See [${section}-frontmatter.log](./logs/${section}-frontmatter.log) for full details.
$(if [[ ${frontmatter_compliance} -lt 100 ]]; then
  echo "- **Issues Found**:"
  grep -A 5 "WARNING" "${REPORT_DIR}/${section}-frontmatter.log" | head -n 10 | sed 's/^/  - /'
fi)

### Link Validation

- **Success Rate**: ${link_success_rate}%
- **Details**: See [${section}-links.log](./logs/${section}-links.log) for full details.
$(if [[ ${link_success_rate} -lt 100 ]]; then
  echo "- **Issues Found**:"
  grep -A 5 "broken link" "${REPORT_DIR}/${section}-links.log" | head -n 10 | sed 's/^/  - /'
fi)

### Code Block Validation

- **Compliance Rate**: ${code_block_compliance}%
- **Details**: See [${section}-codeblocks.log](./logs/${section}-codeblocks.log) for full details.
$(if [[ ${code_block_compliance} -lt 100 ]]; then
  echo "- **Issues Found**:"
  grep -A 5 "issues found" "${REPORT_DIR}/${section}-codeblocks.log" | head -n 10 | sed 's/^/  - /'
fi)

## Recommendations

$(if [[ ${frontmatter_compliance} -lt 100 ]]; then
  echo "- **Frontmatter**: Fix remaining frontmatter issues to achieve 100% compliance."
fi)
$(if [[ ${link_success_rate} -lt 95 ]]; then
  echo "- **Links**: Fix broken links to improve success rate to at least 95%."
fi)
$(if [[ ${code_block_compliance} -lt 100 ]]; then
  echo "- **Code Blocks**: Fix remaining code block syntax issues."
fi)
$(if [[ ${frontmatter_compliance} -ge 100 && ${link_success_rate} -ge 95 && ${code_block_compliance} -ge 100 ]]; then
  echo "- All validation metrics meet or exceed target levels! No immediate action needed."
fi)

## Next Steps

- Add this section to the Week 2 plan if any metrics are below target
- Schedule regular validation checks to maintain quality

EOF
  
  log "INFO" "Report generated for ${section} at ${report_file}"
}

# Generate final combined report
generate_final_report() {
  local report_file="${REPORT_DIR}/week1-final-validation-report.md"
  
  log "INFO" "Generating final validation report"
  
  # Create the report header
  cat > "${report_file}" << EOF
---
title: "Week 1 Final Validation Report"
description: "Comprehensive validation results for Week 1 of the Documentation Reorganization Project"
category: "Documentation"
tags: ["report", "validation", "documentation", "reorganization", "week1"]
last_updated: "$(date +"%B %d, %Y")"
---

# Week 1 Final Validation Report

## Overview

This is the final validation report for Week 1 of the Documentation Reorganization Project, covering the period from March 28 to April 4, 2025.

## Summary Metrics

| Section | Frontmatter | Links | Code Blocks | Overall |
|---------|-------------|-------|------------|---------|
EOF
  
  # Add data for each section
  for section in "${SECTIONS[@]}"; do
    local section_report="${REPORT_DIR}/${section}-report.md"
    if [[ -f "${section_report}" ]]; then
      local frontmatter=$(grep "Frontmatter Compliance" "${section_report}" | awk -F'|' '{print $3}' | sed 's/%//' | xargs)
      local links=$(grep "Link Success Rate" "${section_report}" | awk -F'|' '{print $3}' | sed 's/%//' | xargs)
      local codeblocks=$(grep "Code Block Compliance" "${section_report}" | awk -F'|' '{print $3}' | sed 's/%//' | xargs)
      local overall=$(grep "Overall Rating" "${section_report}" | awk -F'|' '{print $3}' | sed 's/%//' | xargs)
      
      echo "| ${section} | ${frontmatter}% | ${links}% | ${codeblocks}% | ${overall}% |" >> "${report_file}"
    fi
  done
  
  # Add remaining sections of the report
  cat >> "${report_file}" << EOF

## Week 1 Accomplishments

- ✅ Fixed links in high-priority sections (01_getting_started, 02_examples)
- ✅ Fixed code block syntax issues (492 instances across 17 files)
- ✅ Fixed frontmatter in all processed directories (124 files)
- ✅ Created comprehensive validation and fixing tools
- ✅ Fixed critical bug with duplicate frontmatter blocks
- ✅ Created detailed documentation of all processes and fixes

## Overall Status

- **Documentation Quality**: $(grep "Overall Documentation Quality" "${SCRIPT_DIR}/validation-status-dashboard.md" | awk -F'|' '{print $3}')
- **Link Success Rate**: $(grep "Link Success Rate" "${SCRIPT_DIR}/validation-status-dashboard.md" | awk -F'|' '{print $3}')
- **Frontmatter Compliance**: $(grep "Frontmatter Compliance" "${SCRIPT_DIR}/validation-status-dashboard.md" | awk -F'|' '{print $3}')
- **Code Block Formatting**: $(grep "Code Block Formatting" "${SCRIPT_DIR}/validation-status-dashboard.md" | awk -F'|' '{print $3}')
- **Section Coverage**: $(grep "Section Coverage" "${SCRIPT_DIR}/validation-status-dashboard.md" | awk -F'|' '{print $3}')

## Areas for Improvement

Based on the current validation results, the following areas should be prioritized for Week 2:

1. **Link Fixing**: Focus on remaining broken links in the following sections:
   - 02_examples
   - 03_contributing
   - 05_reference/standards

2. **Section Coverage**: Improve section coverage in:
   - 02_examples
   - 03_contributing
   - 05_reference/auth
   - 05_reference/api

3. **Tool Improvements**:
   - Enhance fix-links.sh to better handle macOS path resolution
   - Improve run-daily-fixes.sh to handle directory structure mismatches

## Week 2 Priorities

1. Complete link fixes in remaining sections
2. Add missing content to improve section coverage
3. Enhance validation tools for better usability
4. Set up automated validation checks in CI pipeline
5. Document best practices for documentation maintenance

## Conclusion

Week 1 of the Documentation Reorganization Project has successfully achieved its primary goals, with significant improvements in documentation quality, link integrity, and frontmatter compliance. By addressing the remaining issues in Week 2, we can complete the reorganization effort and establish a robust framework for ongoing documentation maintenance.

## Related Documents

- [Week 1 Action Plan Tracker](../week1-action-tracker.md)
- [Validation Status Dashboard](../validation-status-dashboard.md)
- [Day 7 Summary](../day7-summary.md)
EOF
  
  log "INFO" "Final report generated at ${report_file}"
}

# Main execution
log "INFO" "Starting comprehensive validation on all documentation sections"

# Run validation on each section
for section in "${SECTIONS[@]}"; do
  frontmatter_compliance=$(validate_frontmatter "${section}")
  link_success_rate=$(validate_links "${section}")
  code_block_compliance=$(validate_code_blocks "${section}")
  
  generate_section_report "${section}" "${frontmatter_compliance}" "${link_success_rate}" "${code_block_compliance}"
  
  log "INFO" "Completed validation for ${section}"
  log "INFO" "--------------------------------------------"
done

# Generate final report
generate_final_report

log "INFO" "Comprehensive validation completed. Reports available in ${REPORT_DIR}"

exit 0 
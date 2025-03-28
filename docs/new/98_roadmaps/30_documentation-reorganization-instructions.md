---
title: Documentation Reorganization Instructions
description: Step-by-step guide for implementing the Navius documentation restructuring
category: roadmap
tags:
  - documentation
  - organization
  - migration
  - standards
related:
  - 30_documentation-reorganization-roadmap.md
  - ../contributing/documentation-guidelines.md
  - ../reference/standards/documentation-standards.md
last_updated: March 27, 2025
version: 1.0
status: not started
---

# Documentation Reorganization Instructions

This document provides detailed instructions for implementing the Navius documentation reorganization outlined in the [Documentation Reorganization Roadmap](30_documentation-reorganization-roadmap.md).

## Initial Setup

### Create Base Structure

1. Create the new directory structure in `/docs/new/`:

```bash
mkdir -p docs/new/{01_getting_started,02_examples,03_contributing,04_guides,05_reference,98_roadmaps,99_misc}
```

2. Generate README files for each section:

```bash
for dir in docs/new/*/; do
  echo "# $(basename "$dir" | sed 's/^[0-9]*_//' | tr '_' ' ' | sed 's/\b\(.\)/\u\1/g')" > "$dir/README.md"
done
```

3. Set up template files:

```bash
cp docs/contributing/documentation-guidelines.md docs/new/03_contributing/
touch docs/new/99_misc/document-template.md
```

### Define Document Template

Create a document template at `docs/new/99_misc/document-template.md` with the following content:

```markdown
---
title: Document Title
description: Brief description of the document's purpose and content
category: [getting-started|examples|contributing|guides|reference|roadmaps|misc]
tags:
  - tag1
  - tag2
  - tag3
related:
  - path/to/related-document1.md
  - path/to/related-document2.md
last_updated: YYYY-MM-DD
version: 1.0
---

# Document Title

## Overview

Brief introduction to the document content.

## Main Section 1

Content for the first main section.

### Subsection 1.1

Content for subsection.

```code example if applicable```

### Subsection 1.2

More content with examples.

## Main Section 2

Content for the second main section.

## Related Documents

- [Document 1](path/to/document1.md)
- [Document 2](path/to/document2.md)
```

For more detailed guidelines on document formatting, writing style, and accessibility requirements, refer to the [Documentation Standards](/docs/reference/standards/documentation-standards.md).

## Content Inventory and Assessment

### Generate Content Inventory

1. Run the comprehensive documentation analysis script to generate an inventory and analysis:

```bash
# Generate a comprehensive report with quality metrics
.devtools/scripts/doc-overhaul/generate_report.sh

# Generate a CSV file with detailed quality metrics for all documents
.devtools/scripts/doc-overhaul/comprehensive_test.sh --csv > docs/new/99_misc/content-inventory.csv

# Generate document relationship visualization
.devtools/scripts/doc-overhaul/comprehensive_test.sh
```

This will create several valuable resources:
- A detailed report at `target/reports/docs_validation/documentation_quality_report_YYYY-MM-DD.md`
- A CSV inventory with quality scores, readability metrics, and code validation results
- A document relationship visualization in both DOT and HTML formats
- Document-specific improvement recommendations

2. Create an initial document inventory:

```bash
mkdir -p docs/new/99_misc
find docs -name "*.md" -not -path "*/new/*" -not -path "*/\.*" | sort > docs/new/99_misc/content-inventory.txt
```

### Document Assessment

Use the generated reports to complete the inventory assessment:

1. Quality assessment:
   - Use the "Content Quality Assessment" scores from the report
   - Documents are automatically rated as:
     - Excellent (9-10 points)
     - Good (7-8 points)
     - Adequate (5-6 points)
     - Poor (3-4 points)
     - Very Poor (0-2 points)

2. Readability assessment:
   - Use the readability metrics from the report
   - Pay attention to documents classified as "Complex" or "Simple"
   - Note the words per sentence metric for improvement targets
   - Ensure documents follow the writing style guidelines in our [Documentation Standards](/docs/reference/standards/documentation-standards.md)

3. Code validation:
   - Check which documents have code blocks that failed validation
   - Prioritize fixing code examples marked as "FAIL"

4. Up-to-date assessment:
   - Refer to the frontmatter last_updated field
   - Verify manually for critical documentation

5. Target location determination:
   - Use the document category and content to determine the best location in the new structure
   - Refer to the mapping table below

6. Review AI-assisted recommendations:
   - Check the "Improvement Recommendations" section of the report
   - Use these targeted suggestions to prioritize document improvements

### Document Relationship Analysis

Review the document relationship visualization:

1. Open the generated HTML visualization from `target/reports/docs_validation/document_graph_*.html`
2. Identify:
   - Central documents (those with many connections)
   - Isolated documents (those with few or no connections)
   - Document clusters (groups of closely related documents)
3. Use this information to plan document organization and cross-referencing

### Automated Testing of Current Documentation

Run the full documentation test suite to identify current issues:

```bash
# Test all documentation for issues with a comprehensive quality report
.devtools/scripts/doc-overhaul/generate_report.sh

# Focus analysis on a specific directory
.devtools/scripts/doc-overhaul/generate_report.sh --dir docs/getting-started

# Generate quality report for a single file
.devtools/scripts/doc-overhaul/generate_report.sh --file docs/getting-started/installation.md

# Skip linting for faster report generation
.devtools/scripts/doc-overhaul/generate_report.sh --skip-linting

# Generate full visualization
.devtools/scripts/doc-overhaul/generate_report.sh --vis

# Validate frontmatter across all documents
.devtools/scripts/doc-overhaul/fix_frontmatter.sh --validate-all --report

# Check for broken links across all documentation
.devtools/scripts/doc-overhaul/fix_links.sh --check-only --report
```

Review the test results to identify priority areas for improvement during the migration.

## Content Migration Plan

### Content Mapping

Based on the inventory assessment, create a detailed content mapping for migration:

| Current Location | New Location | Migration Strategy | Quality Score | Readability | Code Status |
|------------------|--------------|-------------------|---------------|-------------|------------|
| docs/getting-started/installation.md | docs/new/01_getting_started/installation.md | Direct with updates | Good | Good | PASS |
| docs/getting-started/first-steps.md | docs/new/01_getting_started/first-steps.md | Direct with updates | Adequate | Good | PASS |
| docs/guides/application-structure.md | docs/new/04_guides/application-structure.md | Direct with updates | Poor | Complex | FAIL |
| docs/examples/* | docs/new/02_examples/* | Consolidate and restructure | Varies | Varies | Varies |
| docs/architecture/* | docs/new/05_reference/architecture/* | Restructure with updates | Varies | Varies | Varies |
| docs/reference/standards/* | docs/new/05_reference/standards/* | Update format and verify content | Good | Good | PASS |
| docs/contributing/* | docs/new/03_contributing/* | Update with new standards | Adequate | Good | PASS |
| docs/roadmaps/* | docs/new/98_roadmaps/* | Preserve as reference | Adequate | Good | N/A |

### Prioritization Strategy

Use the quality metrics to prioritize migration and improvement efforts:

1. **High Priority**:
   - Documents with "Poor" or "Very Poor" quality scores
   - Documents with "FAIL" code validation status
   - Documents with "Complex" readability

2. **Medium Priority**:
   - Documents with "Adequate" quality scores
   - Documents with "Simple" readability
   - Documents with missing metadata or sections

3. **Low Priority**:
   - Documents with "Good" or "Excellent" scores
   - Documents with "Good" readability
   - Documents with "PASS" code validation

### Duplication Resolution

Use the comprehensive test report to identify documents with overlapping content:

1. Review the "Document Relationships" section to identify strong relationships
2. Use the graph visualization to identify clusters of related documents
3. Choose primary location for consolidated content
4. Create redirects or cross-references as needed
5. Document decisions in migration notes

### New Content Requirements

Based on gap analysis in the comprehensive report:

1. Review the "Coverage Gaps" section to identify missing documentation
2. Identify sections with low documentation coverage
3. Prioritize creation based on feature importance
4. Assign responsibilities for creating new content

## Migration Process

### Document Migration Steps

For each document to be migrated:

1. **Review and Plan**:
   - Review source document quality using the inventory assessment
   - Check document quality score, readability metrics, and code validation results
   - Review specific recommendations from the comprehensive report
   - Plan structure updates and content improvements
   - Verify the document will meet the formatting requirements in our [Documentation Standards](/docs/reference/standards/documentation-standards.md)

2. **Migrate Content**:
   ```bash
   cp [source_path] [target_path]
   ```

3. **Fix Frontmatter**:
   - Use the frontmatter fixing script:
   ```bash
   # Fix frontmatter for a single file
   .devtools/scripts/doc-overhaul/fix_frontmatter.sh [target_path]
   
   # Process an entire directory
   .devtools/scripts/doc-overhaul/fix_frontmatter.sh --dir [directory] --recursive
   
   # Auto-apply changes without confirmation
   .devtools/scripts/doc-overhaul/fix_frontmatter.sh [target_path] auto
   ```
   - Ensure all required metadata is present
   - Verify category, tags, and related documents
   - Reading time will be automatically calculated based on content length

4. **Add Standard Sections**:
   - Use the section adding script:
   ```bash
   # Add missing sections to a single file
   .devtools/scripts/doc-overhaul/add_sections.sh [target_path]
   
   # Process an entire directory
   .devtools/scripts/doc-overhaul/add_sections.sh --dir [directory] --recursive
   
   # Auto-apply changes without confirmation
   .devtools/scripts/doc-overhaul/add_sections.sh [target_path] auto
   
   # Add specific sections to a document
   .devtools/scripts/doc-overhaul/add_sections.sh --sections "Overview,Prerequisites,Troubleshooting" [target_path]
   
   # Add all recommended sections based on document type
   .devtools/scripts/doc-overhaul/add_sections.sh --add-all [target_path]
   
   # Only check for missing sections without making changes
   .devtools/scripts/doc-overhaul/add_sections.sh --check-only --dir [directory] --report
   ```
   - Ensure consistent document structure across all files

5. **Improve Readability**:
   - For documents marked as "Complex":
     - Break down long sentences
     - Simplify wording
     - Use more subheadings and lists
   - For documents marked as "Simple":
     - Add more detailed explanations
     - Provide context and background information
   - Follow the writing style guidelines in the [Documentation Standards](/docs/reference/standards/documentation-standards.md)

6. **Fix Code Examples**:
   - Focus on examples marked as "FAIL" in the validation
   - Verify all code examples work with current version
   - Add context and explanations
   - Use consistent syntax highlighting

7. **Fix Links**:
   - Use the link fixing script:
   ```bash
   # Fix links in a single file
   .devtools/scripts/doc-overhaul/fix_links.sh [target_path]
   
   # Process an entire directory
   .devtools/scripts/doc-overhaul/fix_links.sh --dir [directory] --recursive
   
   # Auto-apply changes without confirmation
   .devtools/scripts/doc-overhaul/fix_links.sh [target_path] auto
   
   # Only check for broken links without making changes
   .devtools/scripts/doc-overhaul/fix_links.sh --check-only --dir [directory] --report
   ```
   - Ensure all internal links use absolute paths
   - Add cross-references to related documents

8. **Final Review**:
   - Run document-specific validation:
   ```bash
   .devtools/scripts/doc-overhaul/generate_report.sh --file [target_path]
   ```
   - Review quality score improvements
   - Verify code validation passes
   - Check readability improvements
   - Confirm all recommendations have been addressed

### Automated Document Processing

To process multiple files efficiently:

```bash
# Process a set of files with consistent standards
.devtools/scripts/doc-overhaul/improve_docs.sh

# Process all documents in a specific directory
.devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new/01_getting_started
```

These tools will guide you through an interactive process to select and improve multiple documents efficiently.

### Cleanup Process

After migration of all content:

1. Run comprehensive testing on the new structure:
   ```bash
   .devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new
   ```

2. Check for orphaned or unlinked documents using the report
3. Verify cross-references and internal links
4. Examine the document relationship visualization to ensure proper connectivity
5. Remove duplicated content based on the content inventory
6. Archive obsolete content

## Testing and Validation

### Build Testing

1. Configure mdbook to build from the new structure:

```bash
cp docs/book.toml docs/new/
```

2. Update `SUMMARY.md` to reflect the new structure:

```bash
echo "# Summary" > docs/new/SUMMARY.md
echo >> docs/new/SUMMARY.md
find docs/new -type f -name "*.md" | sort | sed 's/docs\/new\//- \[/; s/\.md/\](&)/; s/_/ /g; s/\/[0-9]*_/\//g' >> docs/new/SUMMARY.md
```

3. Build documentation to verify structure:

```bash
cd docs/new
mdbook build
```

### Validation Checklist

Run automated validation on the new structure:

```bash
# Generate comprehensive quality report with trend tracking
.devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new

# Generate updated document relationship visualization
.devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new --vis

# Skip linting for faster validation focusing on content quality
.devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new --skip-linting

# Validate frontmatter completeness across all documents
.devtools/scripts/doc-overhaul/fix_frontmatter.sh --validate-all --dir docs/new --report

# Check for broken links in all documents
.devtools/scripts/doc-overhaul/fix_links.sh --check-only --dir docs/new --report
```

Verify that each section meets these requirements:
- [ ] All documents have proper frontmatter
- [ ] All documents follow the template structure
- [ ] All documents score "Good" or "Excellent" in quality assessment
- [ ] All documents have "Good" readability scores
- [ ] All code examples pass validation
- [ ] All internal links work correctly
- [ ] All documents have appropriate cross-references
- [ ] No references to deprecated features or approaches
- [ ] All documents conform to the formatting and style requirements in our [Documentation Standards](/docs/reference/standards/documentation-standards.md)

## Finalization and Publication

### Final Review

1. Generate a final documentation quality report:
   ```bash
   .devtools/scripts/doc-overhaul/generate_report.sh --dir docs/new
   ```

2. Review the report for any remaining issues:
   - Check the quality distribution (aim for >80% Good or Excellent)
   - Verify code validation pass rate (aim for >95%)
   - Check readability metrics (aim for >90% Good)
   - Review the historical trends to confirm overall improvement
   
3. Address all critical and high-priority issues
4. Verify all success criteria from roadmap are met

### Publication Steps

1. Backup current documentation:
   ```bash
   cp -r docs docs.bak
   ```

2. Move new structure to replace current:
   ```bash
   rm -rf docs/{getting-started,examples,contributing,guides,reference,architecture,roadmaps}
   mv docs/new/* docs/
   rmdir docs/new
   ```

3. Update build configuration:
   ```bash
   # Update CI/CD settings if needed
   ```

4. Publish and announce:
   - Deploy updated documentation
   - Announce changes to community
   - Provide transition guide for users

## Maintenance Plan

### Ongoing Documentation Quality

1. **Regular Automated Testing**:
   - Configure scheduled runs of documentation testing in CI:
     ```bash
     # Add to CI pipeline
     .devtools/scripts/doc-overhaul/generate_report.sh
     ```
   - Set up weekly documentation quality reports
   - Monitor documentation health score
   - Track quality trends over time using the historical data

2. **Process for Updates**:
   - Document update workflow using the provided tools
   - Add documentation validation to PR process
   - Configure pre-commit hooks for documentation standards

3. **Metrics Tracking**:
   - Track quality distribution over time (% of Excellent/Good/etc.)
   - Monitor code validation pass rate
   - Track readability metrics improvement
   - Monitor fix rate for identified issues
   - Review user feedback on documentation usability

## Example Migration

### Example: Migrating Installation Guide

Original: `/docs/getting-started/installation.md`
Target: `/docs/new/01_getting_started/installation.md`

```bash
# Copy the file
cp docs/getting-started/installation.md docs/new/01_getting_started/installation.md

# Check quality metrics with detailed report
.devtools/scripts/doc-overhaul/generate_report.sh --file docs/new/01_getting_started/installation.md

# Fix frontmatter and automatically apply changes
.devtools/scripts/doc-overhaul/fix_frontmatter.sh docs/new/01_getting_started/installation.md auto

# Add any missing standard sections
.devtools/scripts/doc-overhaul/add_sections.sh docs/new/01_getting_started/installation.md

# Improve readability if needed
# (Edit based on readability metrics from the quality report)

# Fix any failing code examples
# (Edit based on code validation results from the quality report)

# Fix any broken links
.devtools/scripts/doc-overhaul/fix_links.sh docs/new/01_getting_started/installation.md

# Verify the document quality after improvements
.devtools/scripts/doc-overhaul/generate_report.sh --file docs/new/01_getting_started/installation.md
```

## Appendix: Templates and Resources

### Document Templates

- Basic document template (shown above)
- Example-specific template
- API reference template
- Guide template

### Migration Tracking

Use a tracking sheet to monitor migration progress:

| Document | Assigned To | Status | Quality Score | Readability | Code Status | Review Status |
|----------|-------------|--------|---------------|-------------|-------------|---------------|
| Installation | Alice | Complete | Good | Good | PASS | Approved |
| First Steps | Bob | In Progress | Adequate | Complex | FAIL | - |
| ... | ... | ... | ... | ... | ... | ... |

### Documentation Test Suite

The `.devtools/scripts/doc-overhaul/` directory contains a comprehensive set of tools for improving and validating documentation:

- **generate_report.sh**: Generates overall documentation quality report with:
  - Executive summary with health score and priority recommendations
  - Quality metrics and distribution visualization
  - Readability analysis and content complexity metrics
  - Code validation results and failing examples identification
  - Document relationship visualization
  - Historical trend tracking with quality metrics over time
  - CI/CD integration capabilities
- **comprehensive_test.sh**: Performs in-depth documentation analysis including:
  - Content quality assessment (10-point scoring system)
  - Readability analysis (words per sentence metrics)
  - Code validation (syntax checking for Rust, Bash)
  - Document relationship visualization
  - AI-assisted improvement recommendations
- **fix_links.sh**: Identifies and fixes broken links with these features:
  - Converts relative links to absolute paths for consistency
  - Batch processing capabilities with `--dir` option
  - Validation-only mode with `--check-only` flag
  - Report generation with `--report` option
  - Recursive directory processing with `--recursive` option
  - Intelligent link suggestion based on filename matching
  - Support for both current and new documentation structure
- **fix_frontmatter.sh**: Adds or corrects document frontmatter with these features:
  - Batch processing of entire directories with `--dir` option
  - Recursive directory traversal with `--recursive` option
  - Validation-only mode with `--validate-all` flag
  - Report generation with `--report` option
  - Automatic reading time calculation
  - Enhanced tag extraction and document categorization
  - Support for both current and new documentation structure
- **add_sections.sh**: Ensures consistent document structure with these features:
  - Automatic detection of document type based on path or frontmatter category
  - Support for both old and new directory structures
  - Intelligent section generation based on document type
  - Automatic last_updated field maintenance
  - Batch processing with directory and recursive options
  - Check-only mode for validation without changes
  - Report generation for section compliance
  - Integration with quality reporting system
- **improve_docs.sh**: Interactive workflow for guided documentation improvement with these features:
  - Step-by-step guided process for improving individual documents
  - Batch processing options for common documentation issues (frontmatter, sections, links)
  - Integration with all documentation validation tools
  - Automatic detection and updating of frontmatter metadata
  - Readability metrics calculation with words per sentence analysis
  - Document quality assessment with detailed reporting
  - Support for both old and new directory structures
  - Frontmatter last_updated field maintenance with current date (March 27, 2025)
  - Document relationship visualization generation
  - Quality report generation with visualization options
  - Interactive document improvement workflow

Use these tools throughout the migration process to ensure high-quality results.

### Command Line Options for fix_frontmatter.sh

The enhanced fix_frontmatter.sh script supports the following options:

```
Usage:
  ./fix_frontmatter.sh <markdown_file> [auto]       # Process a single file
  ./fix_frontmatter.sh --dir <directory>            # Process all markdown files in a directory 
  ./fix_frontmatter.sh --validate-all               # Validate all markdown files (no changes)
  
Options:
  auto                  Apply changes automatically without confirmation
  --dir <directory>     Specify the directory to process (default: docs)
  --recursive, -r       Process directories recursively
  --validate-all        Only validate frontmatter without making changes
  --report              Generate a detailed report of validation results
  --verbose, -v         Show more detailed information during processing
  --help, -h            Display help message
```

### Command Line Options for fix_links.sh

The enhanced fix_links.sh script supports the following options:

```
Usage:
  ./fix_links.sh <markdown_file> [auto]       # Process a single file
  ./fix_links.sh --dir <directory>            # Process all markdown files in a directory 
  ./fix_links.sh --check-only                 # Validate links without making changes
  
Options:
  auto                  Apply changes automatically without confirmation
  --dir <directory>     Specify the directory to process (default: docs)
  --recursive, -r       Process directories recursively
  --check-only          Only validate links without making changes
  --report              Generate a detailed report of validation results
  --verbose, -v         Show more detailed information during processing
  --help, -h            Display help message
```

These options allow for more targeted and efficient documentation testing during the migration process.

### Command Line Options for add_sections.sh

The enhanced add_sections.sh script supports the following options:

```
Usage:
  ./add_sections.sh <markdown_file> [auto]       # Process a single file
  ./add_sections.sh --dir <directory>            # Process all markdown files in a directory 
  ./add_sections.sh --check-only                 # Check for missing sections without making changes
  
Options:
  auto                      Apply changes automatically without confirmation
  --dir <directory>         Specify the directory to process (default: docs)
  --recursive, -r           Process directories recursively
  --check-only              Only check for missing sections without making changes
  --report                  Generate a detailed report of validation results
  --verbose, -v             Show more detailed information during processing
  --add-all                 Add all possible sections appropriate for document type
  --sections "sec1,sec2"    Specify custom sections to add (comma-separated)
  --help, -h                Display help message
```

The script detects document type based on path and adds appropriate sections:
- Getting Started documents: Overview, Prerequisites, Installation, Usage, Troubleshooting, Related Documents
- Guides: Overview, Prerequisites, Usage, Configuration, Examples, Troubleshooting, Related Documents
- Reference: Overview, Configuration, Examples, Implementation Details, Related Documents
- Examples: Overview, Prerequisites, Usage, Related Documents
- Contributing: Overview, Prerequisites, Related Documents
- Architecture: Overview, Implementation Details, Related Documents
- Roadmaps: Overview, Current State, Target State, Implementation Phases, Success Criteria, Related Documents
- Misc: Overview, Related Documents

These options allow for more targeted and efficient document section management during the migration process.

## Related Documents

- [Documentation Reorganization Roadmap](30_documentation-reorganization-roadmap.md) - Strategic plan and goals
- [Documentation Standards](/docs/reference/standards/documentation-standards.md) - Detailed formatting and writing style guidelines
- [Documentation Guidelines](../contributing/documentation-guidelines.md) - General contribution guidelines 
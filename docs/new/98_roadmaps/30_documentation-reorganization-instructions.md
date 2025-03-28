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
  - ../03_contributing/documentation-guidelines.md
  - ../05_reference/standards/documentation-standards.md
last_updated: March 27, 2025
version: 1.0
status: started
---

# Documentation Reorganization Instructions

This document provides detailed instructions for implementing the Navius documentation reorganization outlined in the [Documentation Reorganization Roadmap](30_documentation-reorganization-roadmap.md).

## Initial Setup

### Create Base Structure

1. Create the new directory structure in `/docs/new/`:

```bash
mkdir -p docs/new/{01_getting_started,02_examples,03_contributing,04_guides,05_reference,98_roadmaps,99_misc}
```
✅ Completed on March 27, 2025

2. Generate README files for each section:

```bash
for dir in docs/new/*/; do
  echo "# $(basename "$dir" | sed 's/^[0-9]*_//' | tr '_' ' ' | sed 's/\b\(.\)/\u\1/g')" > "$dir/README.md"
done
```
✅ Completed on March 27, 2025

3. Set up template files:

```bash
cp docs/contributing/documentation-guidelines.md docs/new/03_contributing/
touch docs/new/99_misc/document-template.md
```
✅ Completed on March 27, 2025

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
✅ Completed on March 27, 2025

For more detailed guidelines on document formatting, writing style, and accessibility requirements, refer to the [Documentation Standards](../05_reference/standards/documentation-standards.md).

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
❌ Could not complete due to script issues. The scripts have syntax errors and compatibility issues.

> **Note:** We are encountering issues with the documentation scripts. They appear to have shell compatibility problems and syntax errors. Fixing these scripts has been identified as a high priority task, documented in the roadmap. In the meantime, we're proceeding with manual processes.

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
✅ Completed on March 27, 2025

### Alternative Manual Approach

Since the automated tools are not functioning correctly, we've implemented a manual approach:

1. Created a basic inventory using find
2. Developed a manual migration plan mapping document
3. Copied content directly to the new structure
4. Created a document that explains the reorganization process

### Document Assessment

Use the generated reports to complete the inventory assessment:

1. Quality assessment:
   - ❌ Automated quality assessment (blocked by script issues)
   - ✅ Manual inspection of key documents

2. Readability assessment:
   - ❌ Automated readability metrics (blocked by script issues)
   - ✅ Manual review of selected documents

3. Code validation:
   - ❌ Automated code validation (blocked by script issues)

4. Up-to-date assessment:
   - ✅ Manual review of last_updated fields in frontmatter

5. Target location determination:
   - ✅ Created mapping plan based on document content and category
   - ✅ Implemented new structure based on the plan

6. Review AI-assisted recommendations:
   - ❌ AI-assisted recommendations (blocked by script issues)

### Document Relationship Analysis

Review the document relationship visualization:

1. Open the generated HTML visualization from `target/reports/docs_validation/document_graph_*.html`
   - ❌ Visualization generation (blocked by script issues)

2. Identify:
   - ❌ Central documents analysis (blocked by script issues)
   - ❌ Isolated documents analysis (blocked by script issues)
   - ❌ Document clusters analysis (blocked by script issues)

3. Manual alternative:
   - ✅ Created manual mapping of related documents

### Automated Testing of Current Documentation

Run the full documentation test suite to identify current issues:

```bash
# Test all documentation for issues with a comprehensive quality report
.devtools/scripts/doc-overhaul/generate_report.sh
```
❌ Automated testing (blocked by script issues)

## Content Migration Plan

### Content Mapping

Based on the inventory assessment, we created a detailed content mapping for migration:

✅ Created migration plan document at `docs/new/99_misc/migration-plan.md`

This document contains the detailed mapping of all files from their original locations to the new structure.

### Migration Process Implementation

Following the migration plan, we have implemented the following steps:

1. ✅ Created the directory structure with numbered sections
2. ✅ Copied content from the old structure to the new structure
3. ✅ Renamed files where needed (e.g., spring-boot-comparison.md, postgresql-integration.md)
4. ✅ Updated the main README.md file
5. ✅ Created the reorganization explanation document
6. ✅ Generated a new SUMMARY.md for navigation

Remaining steps:
1. ❌ Update frontmatter in all documents (pending)
2. ❌ Fix internal links to point to new paths (pending)
3. ❌ Add missing sections to documents (pending)
4. ❌ Validate the new structure (pending)

## Script Issues and Mitigation

The documentation scripts located in `.devtools/scripts/doc-overhaul/` are currently not functioning correctly. These issues have been documented in the [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md) and [Documentation Scripts Fix Instructions](31_documentation-script-fixes-instructions.md).

Until these scripts are fixed, follow these alternative manual processes:

1. **For document quality assessment**:
   - Use manual review based on the documentation standards
   - Use simple grep commands to find common issues: `grep -r "TODO" docs/new/`
   - Use basic markdown linting through IDE extensions or online tools

2. **For frontmatter validation**:
   - Manually verify frontmatter against the templates
   - Use a simple text editor or find/replace for batch updates
   - Create sample valid frontmatter and use it as a reference

3. **For link validation**:
   - Use IDE search functionality to find broken links
   - Manually check links when migrating content
   - Update links systematically by directory

See the [Documentation Scripts Fix Instructions](31_documentation-script-fixes-instructions.md) for detailed plans to fix each script.

## Next Steps

1. Fix the documentation scripts
2. Continue with manual updates to frontmatter and links
3. Validate the new structure
4. Prepare for final deployment

## Related Documents

- [Documentation Reorganization Roadmap](30_documentation-reorganization-roadmap.md) - Strategic plan and goals
- [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md)
- [Documentation Scripts Fix Instructions](31_documentation-script-fixes-instructions.md)
- [Documentation Standards](../05_reference/standards/documentation-standards.md) - Detailed formatting and writing style guidelines
- [Documentation Guidelines](../03_contributing/documentation-guidelines.md) - General contribution guidelines 
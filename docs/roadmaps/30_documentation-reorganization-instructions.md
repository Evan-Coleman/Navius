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

## Content Inventory and Assessment

### Generate Content Inventory

1. Create a content inventory spreadsheet with the following columns:
   - File Path
   - Document Title
   - Content Type
   - Quality Assessment (1-5)
   - Up-to-Date Assessment (1-5)
   - Target Location in New Structure
   - Migration Priority (1-3)
   - Notes

2. Run the inventory script to generate the initial list of files:

```bash
find docs -name "*.md" -not -path "*/new/*" -not -path "*/\.*" | sort > docs/new/99_misc/content-inventory.txt
```

3. Process each file to complete the inventory spreadsheet:

```bash
for file in $(cat docs/new/99_misc/content-inventory.txt); do
  title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
  echo "$file,$title,,,,,," >> docs/new/99_misc/content-inventory.csv
done
```

### Review and Assess Content

For each document in the inventory:

1. Review content quality:
   - 5: Excellent, comprehensive, clear
   - 4: Good, minor improvements needed
   - 3: Adequate, needs updating
   - 2: Poor, significant rewrite needed
   - 1: Very poor or outdated, should be replaced

2. Assess if content is up-to-date:
   - 5: Completely current
   - 4: Minor updates needed
   - 3: Moderate updates needed
   - 2: Significant updates needed
   - 1: Completely outdated

3. Determine target location in new structure
4. Assign migration priority:
   - 1: High priority (essential docs)
   - 2: Medium priority (important docs)
   - 3: Low priority (optional docs)

## Content Migration Plan

### Content Mapping

Based on the inventory assessment, create a detailed content mapping for migration:

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/getting-started/installation.md | docs/new/01_getting_started/installation.md | Direct with updates |
| docs/getting-started/first-steps.md | docs/new/01_getting_started/first-steps.md | Direct with updates |
| docs/guides/application-structure.md | docs/new/04_guides/application-structure.md | Direct with updates |
| docs/examples/* | docs/new/02_examples/* | Consolidate and restructure |
| docs/architecture/* | docs/new/05_reference/architecture/* | Restructure with updates |
| docs/reference/standards/* | docs/new/05_reference/standards/* | Update format and verify content |
| docs/contributing/* | docs/new/03_contributing/* | Update with new standards |
| docs/roadmaps/* | docs/new/98_roadmaps/* | Preserve as reference |

### Duplication Resolution

Identify sets of documents with overlapping content and decide on consolidation:

1. Review similar content across sections
2. Choose primary location for consolidated content
3. Create redirects or cross-references as needed
4. Document decisions in migration notes

### New Content Requirements

Based on gap analysis, identify new documents that need to be created:

1. List missing documentation in each section
2. Assign priorities for creation
3. Allocate resources for writing new content
4. Set deadlines for completion

## Migration Process

### Document Migration Steps

For each document to be migrated:

1. **Review and Plan**:
   - Review source document for quality and relevance
   - Identify updated structure and outline
   - Note required changes and additions

2. **Migrate Content**:
   ```bash
   cp [source_path] [target_path]
   ```

3. **Update Frontmatter**:
   - Add or update YAML frontmatter
   - Ensure all required metadata is present
   - Verify category, tags, and related documents

4. **Restructure Content**:
   - Apply consistent heading structure
   - Reorganize content for logical flow
   - Add or update introduction and conclusion sections

5. **Update Code Examples**:
   - Verify all code examples work with current version
   - Add context and explanations
   - Use consistent syntax highlighting

6. **Update Cross-References**:
   - Fix internal links to reference new structure
   - Add cross-references to related documents
   - Check for broken or outdated external links

7. **Final Review**:
   - Verify content is up-to-date
   - Check adherence to style guidelines
   - Run mdbook build to verify rendering

### Cleanup Process

After migration of all content:

1. Review generated inventory to ensure all documents are accounted for
2. Check for orphaned or unlinked documents
3. Verify cross-references and internal links
4. Remove duplicated content
5. Archive obsolete content

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

For each section:
- [ ] All documents have proper frontmatter
- [ ] All documents follow the template structure
- [ ] All code examples are functional and current
- [ ] All internal links work correctly
- [ ] All documents have appropriate cross-references
- [ ] No references to deprecated features or approaches

## Finalization and Publication

### Final Review

1. Conduct full documentation review by multiple reviewers
2. Address any remaining issues or gaps
3. Verify all success criteria from roadmap are met

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

### Ongoing Documentation Processes

1. **Regular Reviews**:
   - Monthly audit of documentation
   - Check for outdated content
   - Verify all examples still work

2. **Process for Updates**:
   - Document update workflow
   - Pull request templates for documentation
   - Guidelines for community contributions

3. **Metrics Tracking**:
   - Page view analytics
   - User satisfaction metrics
   - Documentation-related support requests

## Example Migration

### Example: Migrating Installation Guide

Original: `/docs/getting-started/installation.md`
Target: `/docs/new/01_getting_started/installation.md`

```bash
# Copy the file
cp docs/getting-started/installation.md docs/new/01_getting_started/installation.md

# Edit to add frontmatter and update structure
cat > docs/new/01_getting_started/installation.md << 'EOL'
---
title: Installation Guide
description: How to install the Navius framework in your development environment
category: getting-started
tags:
  - installation
  - setup
  - requirements
related:
  - first-steps.md
  - development-setup.md
last_updated: 2025-03-27
version: 1.0
---

# Installation Guide

## Overview

This guide walks you through the process of installing the Navius framework.

## System Requirements

- Rust 1.70 or later
- Cargo
- PostgreSQL 14.0 or later (optional)
- Redis 6.0 or later (optional)

## Installation Steps

### Step 1: Install Rust

If you don't already have Rust installed...

[rest of content...]
EOL
```

## Appendix: Templates and Resources

### Document Templates

- Basic document template (shown above)
- Example-specific template
- API reference template
- Guide template

### Migration Tracking

Use a tracking sheet to monitor migration progress:

| Document | Assigned To | Status | Quality Check | Link Check | Review Status |
|----------|-------------|--------|---------------|------------|---------------|
| Installation | Alice | Complete | ✅ | ✅ | Approved |
| First Steps | Bob | In Progress | - | - | - |
| ... | ... | ... | ... | ... | ... |

### Helpful Scripts

#### Content Inventory Script

```bash
#!/bin/bash
# inventory.sh - Generate documentation inventory

echo "File Path,Title,Word Count,Last Modified,Frontmatter" > inventory.csv

find docs -name "*.md" -not -path "*/\.*" | while read file; do
  title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
  word_count=$(wc -w < "$file")
  last_modified=$(date -r "$file" "+%Y-%m-%d")
  has_frontmatter=$(grep -c "^---" "$file")
  
  echo "\"$file\",\"$title\",$word_count,$last_modified,$has_frontmatter" >> inventory.csv
done

echo "Inventory saved to inventory.csv"
``` 
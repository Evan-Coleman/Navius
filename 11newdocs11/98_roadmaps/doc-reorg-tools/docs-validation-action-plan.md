---
title: Documentation Validation Action Plan
description: Strategic plan for addressing documentation validation issues
category: documentation
tags:
  - planning
  - validation
  - quality
related:
  - ../30_documentation-reorganization-roadmap.md
  - ../30_documentation-reorganization-instructions.md
  - ./validation-summary.md
  - ./README.md
last_updated: March 28, 2025
---

# Documentation Validation Action Plan

## Overview

This document outlines a structured approach to addressing documentation validation issues identified during the documentation reorganization project. The plan focuses on systematic issue resolution using our newly created validation and fix tools, with a phased approach based on document priority.

## Validation Results Summary

Based on our most recent validation results:

- **26%** of documents have frontmatter issues
- **58%** of documents have structure issues
- **0%** of documents have broken links
- **0%** of code blocks are properly marked as Rust code

## Prioritization Framework

We'll address documents in three tiers based on importance and reference count:

### Tier 1 (Immediate Fixes)
- Most referenced documents (5+ references)
- Getting started guides
- Core API references
- Installation instructions

### Tier 2 (Secondary Fixes)
- Examples with code blocks
- Documents with 2-4 references
- Contributing guidelines

### Tier 3 (Final Fixes)
- Supplementary materials
- Documents with 0-1 references
- Historical roadmaps

## Action Items by Issue Type

### 1. Fix Frontmatter Issues

**Tools:**
- `fix-frontmatter.sh` - Automatically detects and adds missing frontmatter

**Process:**
1. Run fix script on Tier 1 documents:
   ```bash
   for file in $(grep -l "Missing frontmatter" 11newdocs11/98_roadmaps/doc-reorg-tools/validation-summary.md | grep "/01_getting_started/\|/05_reference/"); do
     ./11newdocs11/98_roadmaps/doc-reorg-tools/fix-frontmatter.sh --file "$file"
   done
   ```
2. Run simple validator to verify fixes:
   ```bash
   ./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11/01_getting_started 11newdocs11/98_roadmaps/doc-reorg-tools/getting-started-validation-updated.md
   ./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11/05_reference 11newdocs11/98_roadmaps/doc-reorg-tools/reference-validation-updated.md
   ```
3. Repeat for Tier 2 and Tier 3 documents

**Deadline:** April 4, 2025

### 2. Fix Document Structure Issues

**Tools:**
- `add-sections.sh` - Adds missing required sections based on document type

**Process:**
1. Run fix script on Tier 1 documents:
   ```bash
   for file in $(grep -l "Missing required sections" 11newdocs11/98_roadmaps/doc-reorg-tools/validation-summary.md | grep "/01_getting_started/\|/05_reference/"); do
     ./11newdocs11/98_roadmaps/doc-reorg-tools/add-sections.sh --file "$file"
   done
   ```
2. Run simple validator to verify structure fixes:
   ```bash
   ./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11/01_getting_started 11newdocs11/98_roadmaps/doc-reorg-tools/getting-started-structure-updated.md
   ./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11/05_reference 11newdocs11/98_roadmaps/doc-reorg-tools/reference-structure-updated.md
   ```
3. Repeat for Tier 2 and Tier 3 documents

**Deadline:** April 11, 2025

### 3. Fix Code Example Markup

**Tools:**
- `code-example-tagger.sh` - Detects and tags code blocks with appropriate language identifiers

**Process:**
1. Run fix script on Tier 1 documents with code examples:
   ```bash
   for file in $(grep -A1 "Documents with the Most Code Examples" 11newdocs11/98_roadmaps/doc-reorg-tools/validation-summary.md | grep -o "/.*\.md"); do
     ./11newdocs11/98_roadmaps/doc-reorg-tools/code-example-tagger.sh --file "$file"
   done
   ```
2. Verify fixes with simple validator:
   ```bash
   ./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11/02_examples 11newdocs11/98_roadmaps/doc-reorg-tools/examples-code-updated.md
   ```
3. Repeat for Tier 2 and Tier 3 documents

**Deadline:** April 18, 2025

## High-Priority Documents

The following documents should be fixed first, based on reference count:

1. **documentation-standards.md** (19 references)
2. **30_documentation-reorganization-instructions.md** (13 references)
3. **hello-world.md** (10 references)
4. **30_documentation-reorganization-roadmap.md** (10 references)
5. **development-setup.md** (8 references)

## Implementation Schedule

**Week 1 (March 28 - April 4):** Frontmatter Fixes
- Fix frontmatter for all Tier 1 documents
- Generate updated validation report
- Review progress and update plan if needed

**Week 2 (April 5 - April 11):** Structure Fixes
- Fix structure issues for all Tier 1 documents
- Begin frontmatter fixes for Tier 2 documents
- Generate updated validation report

**Week 3 (April 12 - April 18):** Code Example Fixes
- Fix code example markup for Tier 1 documents
- Complete structure fixes for Tier 2 documents
- Generate updated validation report

**Week 4 (April 19 - April 25):** Tier 2 Focus
- Complete all fixes for Tier 2 documents
- Begin fixes for Tier 3 documents
- Generate updated validation report

**Week 5 (April 26 - May 2):** Final Validation
- Complete all fixes for Tier 3 documents
- Final full validation pass
- Generate comprehensive validation summary

## Progress Tracking

To track progress, we'll run weekly validation reports using:

```bash
# After each week's fixes
./11newdocs11/98_roadmaps/doc-reorg-tools/simple-batch-validate.sh 11newdocs11
./11newdocs11/98_roadmaps/doc-reorg-tools/generate-summary.sh
```

We'll compare metrics week-over-week to measure improvement:
- Reduction in frontmatter issues
- Reduction in structure issues
- Increase in tagged code blocks

## Batch Processing Commands

### Using the batch-fix.sh script:

```bash
# Run on a directory in dry-run mode first
./11newdocs11/98_roadmaps/doc-reorg-tools/batch-fix.sh 11newdocs11/01_getting_started --dry-run

# Apply all fixes to a directory
./11newdocs11/98_roadmaps/doc-reorg-tools/batch-fix.sh 11newdocs11/01_getting_started
```

### Alternative: Fix all frontmatter issues in a directory:

```bash
for file in $(find 11newdocs11/01_getting_started -name "*.md"); do
  ./11newdocs11/98_roadmaps/doc-reorg-tools/fix-frontmatter.sh --file "$file"
done
```

### Add missing sections in a directory:

```bash
for file in $(find 11newdocs11/02_examples -name "*.md"); do
  ./11newdocs11/98_roadmaps/doc-reorg-tools/add-sections.sh --file "$file"
done
```

### Tag all code blocks in a directory:

```bash
for file in $(find 11newdocs11/05_reference -name "*.md"); do
  ./11newdocs11/98_roadmaps/doc-reorg-tools/code-example-tagger.sh --file "$file"
done
```

## Automation Opportunities

To improve efficiency, we could develop additional tools:

1. **Validation Dashboard** - A simple HTML dashboard that visualizes validation progress
2. **Batch Fix Script** - A single script that identifies and applies all required fixes
3. **CI/CD Integration** - Add validation checks to the pull request process

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Documentation Reorganization Instructions](../30_documentation-reorganization-instructions.md)
- [Validation Summary](./validation-summary.md)
- [Documentation Validation Tools README](./README.md) 
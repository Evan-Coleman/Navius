---
title: Documentation Validation Tools
description: Suite of tools for validating documentation quality during the reorganization project
category: Documentation
last_updated: March 28, 2025
---

# Documentation Validation Tools

## Overview

This directory contains a suite of tools developed to support the documentation reorganization project. These tools help validate document structure, check for broken links, extract and verify code examples, and generate comprehensive reports.

## Available Tools

| Tool | Description |
|------|-------------|
| [`simple-validate.sh`](#simple-validatesh) | Validates a single document's frontmatter, structure, code examples, and links |
| [`simple-batch-validate.sh`](#simple-batch-validatesh) | Runs validation on multiple documents and generates a consolidated report |
| [`generate-summary.sh`](#generate-summarysh) | Creates an executive summary of validation results with actionable recommendations |

## Tool Usage

### simple-validate.sh

A standalone validator that checks a single document for common issues.

```bash
./simple-validate.sh <file_path>
```

This tool validates:
- Frontmatter (presence and required fields)
- Document structure (main heading, overview, related documents sections)
- Code examples (counts total and Rust-specific blocks)
- Internal links (validates they point to existing files)

### simple-batch-validate.sh

Runs validation on multiple documents in a directory and generates a consolidated report.

```bash
./simple-batch-validate.sh <directory> [output_report]
```

- `<directory>`: Directory containing markdown files to validate
- `[output_report]`: Optional path for the output report (default: `validation-report-YYYYMMDD.md`)

This tool processes all markdown files in the specified directory, runs validation on each, and generates a report summarizing the findings.

### generate-summary.sh

Creates an executive summary of validation results with actionable recommendations.

```bash
./generate-summary.sh [report_directory]
```

- `[report_directory]`: Optional path to directory containing validation results (default: `./reports`)

This tool analyzes validation results to:
- Identify documents with the most code examples
- Find the most referenced documents
- List documents requiring attention (frontmatter, structure, or link issues)
- Generate specific recommendations for improving documentation quality

## Typical Workflow

1. Validate a single document:
   ```bash
   ./simple-validate.sh 11newdocs11/98_roadmaps/30_documentation-reorganization-roadmap.md
   ```

2. Validate all documents in a directory:
   ```bash
   ./simple-batch-validate.sh 11newdocs11/98_roadmaps
   ```

3. Generate an executive summary:
   ```bash
   ./generate-summary.sh
   ```

## Output Files

- Individual validation results are stored in the `reports` directory with the naming pattern `filename_validation.txt`
- The batch validation report is stored as `validation-report.md` (or custom name if specified)
- The executive summary is stored as `validation-summary.md`

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Documentation Reorganization Instructions](../30_documentation-reorganization-instructions.md)
- [Documentation Standards](/11newdocs11/05_reference/standards/documentation-standards.md) 
---
title: "Consolidated Validation Script Usage Guide"
description: "Instructions for using the run-consolidated-validation.sh script to validate documentation quality"
category: reference
tags:
  - documentation
  - validation
  - tools
  - quality
related:
  - ../30_documentation-reorganization-roadmap.md
  - ./phase2-completion-plan.md
  - ./validation-tracking-template.md
  - ../30_documentation-reorganization-instructions.md
last_updated: March 27, 2025
version: 1.0
---

# Consolidated Validation Script Usage Guide

## Overview

The `run-consolidated-validation.sh` script provides a unified interface for validating documentation quality across multiple dimensions. It combines the functionality of the individual validation tools:

- Code example extraction and verification
- Internal link analysis
- Document structure validation
- Frontmatter validation

This tool is designed to support the phased validation approach outlined in the [Phase 2 Completion Plan](./phase2-completion-plan.md), enabling efficient validation of documents based on their priority tier.

## Installation

1. Ensure the script is executable:
   ```bash
   chmod +x run-consolidated-validation.sh
   ```

2. Verify that the required validation tools are available in the same directory:
   - `code-example-extractor.sh`
   - `code-example-verifier.sh`
   - `link-analyzer.sh`
   - `document-validator.sh`

## Usage Options

The script supports several modes of operation:

### Validating a Single Document

```bash
./run-consolidated-validation.sh --file <path_to_document.md>
```

Example:
```bash
./run-consolidated-validation.sh --file ../../01_getting_started/installation.md
```

### Validating a Directory of Documents

```bash
./run-consolidated-validation.sh --dir <directory_path>
```

Example:
```bash
./run-consolidated-validation.sh --dir ../../01_getting_started/
```

### Tiered Validation

Specify the validation tier to control the sampling rate:

```bash
./run-consolidated-validation.sh --dir <directory_path> --tier <1|2|3>
```

- **Tier 1**: Validates 100% of documents (default)
- **Tier 2**: Validates approximately 50% of documents (random sampling)
- **Tier 3**: Validates approximately 20% of documents (spot checking)

Example:
```bash
./run-consolidated-validation.sh --dir ../../02_examples/ --tier 2
```

### Generating Reports

Reports are always generated, but you can explicitly request a report-only run:

```bash
./run-consolidated-validation.sh --dir <directory_path> --report
```

## Output and Reports

When you run the script, it will:

1. Show real-time validation progress in the terminal
2. Generate a Markdown report in the `reports/` subdirectory
3. Display a summary of validation results upon completion

### Report Structure

The generated report includes:

- **Overview**: Brief information about the validated documents
- **Summary**: Quantitative metrics of issues found
- **Detailed Results**: Document-by-document breakdown of validation results
- **Recommendations**: Suggested actions based on validation results
- **Next Steps**: Guidance on proceeding with documentation improvements

### Sample Report Output

```markdown
# Consolidated Validation Report

## Overview

This report contains the consolidated results of validation checks performed on the documents in: `../../01_getting_started/`.

Validation Tier: 1
Generated on: March 27, 2025 at 14:30:45

## Summary

| Category | Count |
|----------|-------|
| Documents Validated | 12 |
| Code Examples Found | 45 |
| Links Found | 63 |
| Frontmatter Issues | 3 |
| Structure Issues | 5 |
| Code Issues | 8 |
| Link Issues | 7 |
| Overall Validation Status | ❌ FAIL (4/12 documents failed) |
```

## Integration with Tracking

After running the validation script, you should:

1. Open the [Validation Tracking Template](./validation-tracking-template.md)
2. Update the relevant sections with the validation results
3. Prioritize fixing issues in Tier 1 documents first

## Automation and CI Integration

The script can be integrated into automated workflows:

```bash
# Validate Tier 1 documents and save report to a specific location
./run-consolidated-validation.sh --dir ../../01_getting_started/ --tier 1 > validation_results.txt
```

## Troubleshooting

### Common Issues

1. **Script not executing**:
   - Ensure the script has execute permissions: `chmod +x run-consolidated-validation.sh`

2. **Missing dependent tools**:
   - The script will warn if any of the required tools are missing
   - It will continue execution with reduced functionality

3. **Permission denied for temporary files**:
   - Ensure you have write permissions in the directory

4. **Empty or incomplete reports**:
   - Check if the target documents exist and are properly formatted Markdown

## Next Steps

After running the validation:

1. Review the generated report
2. Update the [Validation Tracking Template](./validation-tracking-template.md) with results
3. Fix issues starting with highest priority documents
4. Re-run validation to verify improvements

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Phase 2 Completion Plan](./phase2-completion-plan.md)
- [Validation Tracking Template](./validation-tracking-template.md)
- [Code Example Issues](./code-example-issues.md) 
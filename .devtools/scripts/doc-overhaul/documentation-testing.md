# Documentation Testing Tools

This directory contains tools for testing documentation quality and maintaining consistent standards across the Navius documentation.

## Overview

These tools provide multiple levels of documentation validation:

1. **Basic Validation**: Syntax checking, link validation, frontmatter validation
2. **Comprehensive Analysis**: Document relationships, content quality, section standardization  
3. **Consolidated Reporting**: Combined quality reports with actionable recommendations

## Available Tools

### Main Scripts

- **generate_report.sh**: Generates a comprehensive documentation quality report combining all validation levels
- **comprehensive_test.sh**: Performs advanced documentation analysis (relationships, structure, quality)
- **fix_links.sh**: Identifies and fixes broken links in documentation
- **fix_frontmatter.sh**: Adds or fixes frontmatter in documentation files
- **add_sections.sh**: Adds standard sections to documentation files

### Integration with CI

The documentation testing is integrated with CI in two ways:

1. **validate_docs.sh**: Basic validation runs on every PR that changes documentation
2. **Manual Quality Report**: A comprehensive report can be triggered manually in GitLab CI

## Usage

### Local Development

Run the comprehensive report generator locally:

```bash
.devtools/scripts/doc-overhaul/generate_report.sh
```

This will:
- Check for syntax issues with markdownlint
- Validate internal and external links
- Verify frontmatter completeness
- Analyze document relationships
- Create a quality report with recommendations

### Fixing Individual Files

To fix issues in specific files:

```bash
# Fix links in a file
.devtools/scripts/doc-overhaul/fix_links.sh path/to/file.md

# Fix frontmatter in a file
.devtools/scripts/doc-overhaul/fix_frontmatter.sh path/to/file.md

# Add standard sections to a file
.devtools/scripts/doc-overhaul/add_sections.sh path/to/file.md
```

### Full Documentation Improvement Workflow

For a guided approach to improving documentation:

```bash
.devtools/scripts/doc-overhaul/improve_docs.sh
```

This interactive script will:
1. Scan all documentation for issues
2. Help you select files to improve
3. Guide you through fixing them one by one

## Reports

Documentation testing generates several reports:

- **Basic Validation Report**: Simple text report in the project root
- **Comprehensive Test Report**: Markdown report with relationship analysis
- **Quality Report**: Combined report with executive summary and recommendations
- **HTML Report**: An HTML version of the quality report (if pandoc is installed)

All reports are stored in `target/reports/docs_validation/`.

## CI Integration

The GitLab CI is configured to:

1. Run basic validation on PRs that change documentation
2. Fail the build if critical issues are found
3. Provide a manual job to generate a full quality report

## Documentation Standards

These tools enforce the following standards:

- All documents must have complete frontmatter (title, description, category, etc.)
- All documents should have a standard heading structure
- All documents should have a Related Documents section
- Internal links should use absolute paths from the project root
- All links should resolve correctly

## Extending the Testing

To extend the documentation testing:

1. Add new checks to comprehensive_test.sh
2. Update the generate_report.sh script to include your new checks
3. Update this documentation to reflect your changes

## Related Documents

- [Documentation Overhaul Roadmap](/docs/roadmaps/12_document_overhaul.md) - Overall roadmap
- [Documentation Standards](/docs/reference/documentation-standards.md) - Detailed standards 
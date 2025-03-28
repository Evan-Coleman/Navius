---
title: Documentation Reorganization Roadmap
description: Strategic plan for restructuring Navius documentation to improve usability and maintenance
category: roadmap
tags:
  - documentation
  - organization
  - planning
  - standards
related:
  - 30_documentation-reorganization-instructions.md
  - ../03_contributing/documentation-guidelines.md
  - ../05_reference/standards/documentation-standards.md
last_updated: March 27, 2025
version: 1.0
status: started
---

# Documentation Reorganization Roadmap

## Overview

This roadmap outlines our plan to restructure the Navius documentation to improve discoverability, maintainability, and overall user experience. The reorganization will simplify navigation, eliminate duplication, update outdated content, and ensure all documents adhere to consistent standards.

## Current State

The Navius documentation currently faces several challenges:

- **Inconsistent Organization**: Documentation is scattered across multiple directories without clear categorization, making it difficult for users to find relevant information.
- **Duplication**: Similar content appears in multiple places, leading to maintenance issues and inconsistencies.
- **Outdated References**: Some documents reference deprecated features, code patterns, or architectural components.
- **Inconsistent Formatting**: Documents vary in structure, headings, and metadata, making navigation less intuitive.
- **Incomplete Coverage**: Gaps exist in documentation for newer features and components.
- **Non-functioning Documentation Scripts**: The documentation validation and improvement scripts are not working correctly, making automated quality assessment difficult.

## Target State

After reorganization, the Navius documentation will:

- **Have an Intuitive Structure**: Clear categorization with progressive depth, from getting started to advanced topics.
- **Eliminate Redundancy**: No duplicate content, with cross-references instead.
- **Be Consistently Formatted**: All documents following the same template and style guidelines as defined in our [Documentation Standards](../05_reference/standards/documentation-standards.md).
- **Be Up-to-Date**: All content verified as current and accurate.
- **Have Complete Coverage**: Documentation for all major features and components.
- **Be Maintainable**: Clear processes for ongoing updates and improvements.
- **Have Working Documentation Tools**: Functional scripts for validation, quality assessment, and improvement.

## New Structure

The documentation will be reorganized into the following primary sections:

| Directory | Purpose | Content Types |
|-----------|---------|---------------|
| `01_getting_started` | Essential onboarding | Installation, quick start, hello world |
| `02_examples` | Practical implementation examples | Code examples, use cases, patterns |
| `03_contributing` | Contributor guidelines | Development setup, PR process, coding standards |
| `04_guides` | In-depth feature documentation | Feature guides, tutorials, best practices |
| `05_reference` | Technical reference material | API documentation, architecture, standards |
| `98_roadmaps` | Development planning | Historical and future development plans |
| `99_misc` | Supplementary material | Templates, miscellaneous information |

## Implementation Phases

### Phase 0: Fix Documentation Scripts (High Priority)

The documentation scripts located in `.devtools/scripts/doc-overhaul/*.sh` are not working correctly. These scripts are intended to validate, improve, and manage the documentation, but they currently have significant issues:

- Shell compatibility problems with macOS (zsh)
- Syntax errors in expressions and variable handling
- Issues with markdownlint detection
- File count and reporting errors

This phase must be prioritized before continuing with the automated portions of the reorganization. See the [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md) for a detailed plan to address these issues.

### Tasks
- ❌ Analyze and document all script errors in detail
- ❌ Create a shell-agnostic utility library
- ❌ Fix individual scripts (generate_report.sh, comprehensive_test.sh, etc.)
- ❌ Test fixes in multiple environments
- ❌ Update script documentation
- ❌ Support the new directory structure in all scripts

While these issues are being addressed, manual processes will be used for the documentation reorganization.

### Phase 1: Structure Setup and Content Analysis (Week 1)

1. **Create New Directory Structure**
   - ✅ Set up the new directory hierarchy in a temporary location
   - ✅ Prepare document templates with consistent frontmatter
   - ✅ Establish cross-referencing conventions

2. **Content Analysis**
   - ✅ Created basic content inventory using find
   - ❌ Automated content analysis with generate_report.sh (blocked by script issues)
   - ❌ Detailed quality metrics extraction (blocked by script issues)
   - ❌ Document relationship visualization (blocked by script issues)

3. **Migration Tracking**
   - ✅ Created migration plan document
   - ✅ Set up tracking mechanisms for migration progress
   - ✅ Created migration validation checklist

### Phase 2: Content Migration (Weeks 2-3)

1. **Inventory Existing Content**
   - ✅ Basic inventory of existing documentation files
   - ❌ Quality assessment (blocked by script issues)

2. **Migrate High-Priority Content**
   - ✅ Migrated essential getting started documentation
   - ✅ Migrated contributing documentation
   - ✅ Migrated examples
   - ❌ Apply frontmatter fixes (blocked by script issues)
   - ❌ Fix internal links (blocked by script issues)

3. **Migrate Secondary Content**
   - ✅ Migrated guides and reference documentation
   - ✅ Migrated roadmaps
   - ✅ Migrated miscellaneous content
   - ❌ Add standard sections to documents (blocked by script issues)
   - ❌ Update cross-references to match new structure

4. **Initial Organization Improvements**
   - ✅ Created proper directory structure with numbered sections
   - ✅ Updated main README.md to reflect new structure
   - ✅ Created new SUMMARY.md for navigation
   - ✅ Created README-reorganization.md to explain the changes

### Phase 3: Gap Analysis and Content Creation (Weeks 4-5)

1. **Identify Missing Documentation**
   - ❌ Run updated reports to identify gaps (blocked by script issues)
   - ❌ Identify sections with low coverage

2. **Create New Content**
   - ❌ Develop missing documentation
   - ❌ Use provided templates and document standards

3. **Validation and Improvement**
   - ❌ Run quality assessment on each section (blocked by script issues)
   - ❌ Improve readability
   - ❌ Update cross-references

### Phase 4: Review and Refinement (Week 6)

1. **Technical Review**
   - ❌ Verify technical accuracy
   - ❌ Test code examples

2. **Automated Testing**
   - ❌ Run full documentation validation suite (blocked by script issues)
   - ❌ Generate quality reports (blocked by script issues)

3. **Final Adjustments**
   - ❌ Address feedback and validation issues
   - ❌ Fix any remaining structural problems

### Phase 5: Publication and Monitoring (Week 7)

1. **Deploy New Structure**
   - ❌ Replace existing documentation with new structure
   - ❌ Update build configurations

2. **Announce and Communicate**
   - ❌ Inform users about changes
   - ❌ Document reorganization benefits

3. **Ongoing Maintenance**
   - ❌ Set up regular documentation quality checks
   - ❌ Establish process for addressing documentation issues

## Progress Update

As of March 27, 2025, we have made significant progress with the initial structure setup and content migration:

- ✅ Created the new directory structure with numbered sections
- ✅ Created a document template for standardization
- ✅ Migrated content from the old structure to the new structure
- ✅ Created supporting documentation (migration plan, README, etc.)
- ✅ Updated README.md with new navigation paths
- ❌ Documentation scripts are not working correctly - this is now a high priority

Next steps:
1. Fix the documentation scripts or develop alternatives
2. Update frontmatter in all documents
3. Fix internal links to point to the new paths
4. Validate the structure and make final adjustments

## Script Issues

The following issues have been identified with the documentation scripts:

1. **generate_report.sh**
   - Shell compatibility issues with certain commands
   - Error with markdownlint detection
   - Syntax errors in expressions

2. **comprehensive_test.sh**
   - Invalid option for declare command (likely bash vs zsh differences)
   - Unable to run with --csv option

3. **fix_frontmatter.sh**
   - Reports incorrect file counts
   - May have path resolution issues

4. **fix_links.sh**
   - Not tested yet, but likely has similar issues

Until these issues are resolved, we are proceeding with manual processes for the documentation reorganization.

## Success Criteria

The documentation reorganization will be considered successful when:

1. All existing valid content has been migrated to the new structure
2. All documents follow the established standards and templates as specified in our [Documentation Standards](../05_reference/standards/documentation-standards.md)
3. No duplicate content exists
4. All content has been verified as current and accurate
5. Documentation build completes without errors
6. All automated validation tests pass with no critical issues
7. Users can find information more easily
8. Maintenance processes are clearly established
9. Documentation tools are working correctly

## Metrics

We will track the following metrics to measure success:

- **Migration Completion**: Percentage of documents successfully migrated
- **Standards Compliance**: Percentage of documents adhering to standards
- **Content Coverage**: Percentage of features with complete documentation
- **Documentation Health Score**: Overall score from `generate_report.sh` (target: 90+)
- **Quality Distribution**: Percentage of documents rated "Good" or "Excellent" (target: >80%)
- **Code Example Validity**: Percentage of code examples passing validation (target: >95%)
- **Readability Metrics**: Average readability score across all documents
- **Validation Success**: Pass rate of automated validation tests
- **User Satisfaction**: Feedback scores from users
- **Search Success**: Success rate of users finding information
- **Quality Trend**: Measurable improvement in health score over time

## Validation Tools

We will leverage the documentation validation tools in `.devtools/scripts/doc-overhaul/` to ensure quality:

- **generate_report.sh**: Generates comprehensive quality reports with:
  - Executive summary with health score and actionable recommendations
  - Quality metrics and distribution analysis
  - Readability assessment
  - Code validation results
  - Document relationship visualization
  - Historical trend tracking
  - CI/CD integration capabilities

- **comprehensive_test.sh**: Performs detailed document analysis including:
  - Content quality assessment (10-point scoring system)
  - Readability analysis (words per sentence metrics)
  - Code validation (syntax checking for Rust, Bash)
  - Document relationship visualization
  - AI-assisted improvement recommendations
  
- **fix_frontmatter.sh**: For correcting and standardizing document metadata:
  - Batch processing capabilities for entire directories
  - Validation-only mode for CI/CD integration
  - Report generation for quality assessment
  - Automatic reading time calculation
  - Smart tag extraction and document categorization
  - Support for both existing and new documentation structure

- **fix_links.sh**: For identifying and fixing broken internal links:
  - Automatic conversion of relative links to absolute paths
  - Batch processing with directory and recursive options
  - Validation-only mode with reporting capabilities
  - Intelligent link suggestion based on filename matching
  - Support for automated CI/CD validation
  - Detailed report generation for issue tracking
  - Integration with other documentation tools

- **add_sections.sh**: For ensuring consistent document structure:
  - Automatic detection of document type and missing sections
  - Smart section generation based on document context
  - Support for both old and new numbered directory structures
  - Frontmatter maintenance with automatic last_updated field updates
  - Detection of document type from frontmatter category when path-based detection fails
  - Batch processing for multiple files with recursive options
  - Customizable section definitions with the `--sections` option
  - Comprehensive section templates with detailed code examples and Mermaid diagrams
  - Document-specific section recommendations for all document types
  - Integration with quality reporting system
  - Check-only mode for validation without changes

- **improve_docs.sh**: Interactive workflow for guided documentation improvement:
  - Step-by-step guided process for improving individual documents
  - Batch processing options for common documentation issues
  - Integration with all documentation validation tools
  - Automatic detection and updating of frontmatter metadata
  - Readability metrics calculation with recommendations
  - Quality assessment reporting
  - Support for both old and new directory structures
  - Frontmatter last_updated field maintenance
  - Document relationship visualization generation
  - Advanced options for documentation refactoring

These tools will be integrated into our workflow to provide continuous validation throughout the reorganization process.

## Responsible Parties

- **Documentation Lead**: Overall coordination and quality assurance
- **Technical Writers**: Content migration and creation
- **Development Team**: Technical review and example validation
- **Community Contributors**: Feedback and testing
- **DevOps**: Build and deployment configuration

## Risk Management

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Content loss during migration | High | Low | Regular backups, version control, and automated validation |
| Broken links | Medium | Medium | Use `fix_links.sh --check-only` to identify issues and `fix_links.sh --dir` to fix multiple files at once |
| Inadequate resources | Medium | Medium | Clear prioritization, phased approach |
| User confusion during transition | Medium | High | Clear communication, temporary redirects |
| Scope creep | Medium | High | Strict adherence to roadmap, change management |
| Inconsistent document quality | Medium | Medium | Use automated validation tools and follow the [Documentation Standards](/docs/reference/standards/documentation-standards.md) |
| Incomplete frontmatter | Medium | High | Leverage `fix_frontmatter.sh --validate-all` for comprehensive validation |
| Invalid code examples | High | Medium | Leverage code validation in generate_report.sh |
| Declining quality over time | Medium | High | Implement trend tracking and scheduled quality reviews |

## Related Documents

- [Documentation Reorganization Instructions](30_documentation-reorganization-instructions.md)
- [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md)
- [Documentation Guidelines](../05_reference/standards/documentation-standards.md) 
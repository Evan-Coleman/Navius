---
title: "Documentation Validation Tracking Template"
description: "Template for tracking progress on code example verification, link fixing, and document validation"
category: reference
tags:
  - documentation
  - validation
  - tracking
  - quality
related:
  - ../30_documentation-reorganization-roadmap.md
  - ./phase2-completion-plan.md
  - ./code-example-issues.md
last_updated: March 27, 2025
version: 1.0
---

# Documentation Validation Tracking Template

## Overview

This document provides tracking templates for the three main validation tasks in Phase 2 of the documentation reorganization:

1. Code example verification
2. Internal link fixing
3. Document validation

These templates can be adapted to spreadsheets, issue trackers, or other tracking tools to monitor progress and ensure comprehensive validation coverage.

## 1. Code Example Verification Tracking

### Summary Statistics

| Category | Total Docs | Docs with Examples | Examples Count | Verified | Issues Found | Issues Fixed | Completion % |
|----------|------------|-------------------|----------------|----------|--------------|--------------|--------------|
| Getting Started | | | | | | | |
| Examples | | | | | | | |
| Guides | | | | | | | |
| Reference | | | | | | | |
| Other | | | | | | | |
| **TOTAL** | | | | | | | |

### Detail Tracking Table

| Document Path | Category | Examples Count | Priority | Status | Issues Found | Issue Types | Fix Applied | Verification Date | Verifier | Notes |
|---------------|----------|----------------|----------|--------|--------------|-------------|-------------|-------------------|----------|-------|
| | | | | | | | | | | |

### Issue Types Distribution

| Issue Type | Count | Percentage | Most Affected Category |
|------------|-------|------------|------------------------|
| Missing Imports | | | |
| Outdated API | | | |
| Error Handling | | | |
| Incomplete Example | | | |
| Missing Main Function | | | |
| Type Mismatch | | | |
| Other | | | |

### Weekly Progress

| Week | Examples Verified | Issues Fixed | Cumulative Completion % | Blockers/Challenges | Next Focus |
|------|-------------------|--------------|-------------------------|---------------------|------------|
| Week 1 | | | | | |
| Week 2 | | | | | |
| Week 3 | | | | | |
| Week 4 | | | | | |
| Week 5 | | | | | |

## 2. Internal Link Fixing Tracking

### Summary Statistics

| Category | Total Docs | Docs with Links | Link Count | Fixed | Broken | Completion % |
|----------|------------|-----------------|------------|-------|--------|--------------|
| Getting Started | | | | | | |
| Examples | | | | | | |
| Guides | | | | | | |
| Reference | | | | | | |
| Other | | | | | | |
| **TOTAL** | | | | | | |

### Detail Tracking Table

| Document Path | Category | Links Count | Priority | Status | Broken Links | Link Types | Fix Applied | Verification Date | Validator | Notes |
|---------------|----------|-------------|----------|--------|--------------|------------|-------------|-------------------|-----------|-------|
| | | | | | | | | | | |

### Link Types Distribution

| Link Type | Count | Percentage | Most Affected Category |
|-----------|-------|------------|------------------------|
| Relative Same Directory | | | |
| Relative Parent Directory | | | |
| Absolute Path | | | |
| Cross-Section Links | | | |
| External Links | | | |
| Broken/Missing | | | |

### Critical Path Testing

| Path | Starting Document | Target Document | Status | Notes |
|------|-------------------|-----------------|--------|-------|
| Getting Started → Examples | | | | |
| Examples → API Reference | | | | |
| Getting Started → Guides | | | | |
| Guides → Reference | | | | |
| Roadmap → Implementation | | | | |

### Weekly Progress

| Week | Links Verified | Links Fixed | Cumulative Completion % | Blockers/Challenges | Next Focus |
|------|----------------|-------------|-------------------------|---------------------|------------|
| Week 1 | | | | | |
| Week 2 | | | | | |
| Week 3 | | | | | |
| Week 4 | | | | | |
| Week 5 | | | | | |

## 3. Document Validation Tracking

### Tier Summary Statistics

| Validation Tier | Total Docs | Validated | Issues Found | Issues Fixed | Completion % |
|-----------------|------------|-----------|--------------|--------------|--------------|
| Tier 1 (100%) | | | | | |
| Tier 2 (50%) | | | | | |
| Tier 3 (Spot) | | | | | |
| **TOTAL** | | | | | |

### Detail Tracking Table

| Document Path | Category | Tier | Priority | Status | Issues Found | Issue Types | Fix Applied | Validation Date | Validator | Notes |
|---------------|----------|------|----------|--------|--------------|-------------|-------------|-----------------|-----------|-------|
| | | | | | | | | | | |

### Issue Types Distribution

| Issue Type | Count | Percentage | Most Affected Category |
|------------|-------|------------|------------------------|
| Frontmatter Issues | | | |
| Missing Sections | | | |
| Formatting Inconsistencies | | | |
| Content Quality | | | |
| Code Examples | | | |
| Internal Links | | | |
| Cross-References | | | |
| Other | | | |

### Weekly Progress

| Week | Documents Validated | Issues Fixed | Cumulative Completion % | Blockers/Challenges | Next Focus |
|------|---------------------|--------------|-------------------------|---------------------|------------|
| Week 1 | | | | | |
| Week 2 | | | | | |
| Week 3 | | | | | |
| Week 4 | | | | | |
| Week 5 | | | | | |

## Consolidated Progress Dashboard

### Overall Phase 2 Completion

| Task | Total Items | Completed | Completion % | Target Completion Date | Status |
|------|-------------|-----------|--------------|------------------------|--------|
| Code Example Verification | | | | | |
| Internal Link Fixing | | | | | |
| Document Validation | | | | | |
| **OVERALL PHASE 2** | | | | | |

### Critical Milestone Tracking

| Milestone | Target Date | Actual/Projected Date | Status | Blockers/Dependencies |
|-----------|-------------|----------------------|--------|------------------------|
| Sample Code Verification Complete | | | | |
| Tier 1 Document Validation Complete | | | | |
| Critical Path Links Verified | | | | |
| High-Priority Examples Fixed | | | | |
| Phase 2 Complete | | | | |

### Resource Allocation

| Resource | Role | Tasks | Hours/Week | Progress | Notes |
|----------|------|-------|------------|----------|-------|
| Documentation Lead | | | | | |
| Technical Writers | | | | | |
| Developers | | | | | |
| DevOps | | | | | |

## Using This Template

1. Copy this template to a spreadsheet or project management tool
2. Populate initial counts from code-example-extractor.sh output
3. Update weekly during team meetings
4. Use as the basis for status reports
5. Identify bottlenecks and adjust resource allocation as needed

### Automating Data Collection with the Consolidated Validation Script

The `run-consolidated-validation.sh` script can be used to automatically gather data for this tracking document:

1. Run the script on your target directory or file:
   ```bash
   ./run-consolidated-validation.sh --dir ../../01_getting_started/ --tier 1
   ```

2. Review the generated report in the `reports/` directory

3. Transfer the summary data to the appropriate tables in this tracking document

4. For document-specific issues, transfer the detailed data to the Detail Tracking Tables

This integrated approach ensures consistency between validation results and tracking data, making it easier to monitor progress and identify areas needing attention.

## Related Documents

- [Phase 2 Completion Plan](./phase2-completion-plan.md)
- [Code Example Issue Templates](./code-example-issues.md)
- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Consolidated Validation Script Usage Guide](./consolidated-validation-usage.md) 
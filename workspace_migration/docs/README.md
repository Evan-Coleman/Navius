# Navius Framework Documentation

Welcome to the Navius Framework documentation repository. This repository contains comprehensive documentation for the Navius Framework, including guides, architectural decisions, API documentation, and development resources.

## Directory Structure

```
docs/
├── api-review/            # API Review documentation
│   ├── templates/         # Templates for API Review
│   └── samples/           # Sample API documents
├── architectural-decisions/ # Architectural decision records
├── guides/                # User and developer guides
└── testing/               # Testing documentation
```

## Recent Updates

### March 29, 2025
- Added Design Evaluation for navius-di crate
- Added Design Evaluation for navius-plugin crate
- Added Design Evaluation for navius-event crate
- Added Design Evaluation for navius-auth crate
- Added Provider Pattern Implementation Guide based on database and cache evaluations
- Updated API inventory reports with latest changes
- Updated workspace migration roadmap with latest progress

### March 28, 2025
- Added Design Evaluation for navius-cache crate
- Added Design Evaluation for navius-db crate
- Updated documentation standards draft

### March 27, 2025
- Added Design Evaluation for navius-http crate
- Added Design Evaluation for navius-core crate
- Updated API Review Timeline with current progress
- Added Connection Pooling Tuning Guide
- Added Redis Performance Optimization Guide

### March 26, 2025
- Added Design Evaluation for navius-metrics and navius-test-utils crates
- Added API Inventory Summary
- Updated project roadmap

## Key Documentation

### Getting Started
- [Workspace Migration Tutorial](./workspace-migration-tutorial.md) - Introduction to the workspace migration
- [API Review Guidelines](./api-review-guidelines.md) - Guidelines for API review process
- [API Review Timeline](./api-review-timeline.md) - Timeline and milestones for API review

### Architecture
- [Workspace vs Feature Flags](./workspace-vs-feature-flags.md) - Comparison of workspace vs feature flag approaches
- [Provider Pattern Implementation Guide](./guides/provider-pattern-implementation-guide.md) - Guidelines for implementing the provider pattern

### Reports
- **Design Evaluations (10/15 completed, 67%):**
  - [navius-di](../reports/design-evaluation-navius-di.md)
  - [navius-plugin](../reports/design-evaluation-navius-plugin.md)
  - [navius-event](../reports/design-evaluation-navius-event.md)
  - [navius-auth](../reports/design-evaluation-navius-auth.md)
  - [navius-cache](../reports/design-evaluation-navius-cache.md)
  - [navius-db](../reports/design-evaluation-navius-db.md)
  - [navius-http](../reports/design-evaluation-navius-http.md)
  - [navius-core](../reports/design-evaluation-navius-core.md)
  - [navius-metrics](../reports/design-evaluation/navius-metrics-evaluation.md)
  - [navius-test-utils](../reports/design-evaluation/navius-test-utils-evaluation.md)
- [API Inventory Summary](./api-review/samples/api-inventory-summary.md)
- [Progress Reports](./progress.md)

## Contributing to Documentation

### Style Guide
All documentation should follow the [Microsoft Writing Style Guide](https://docs.microsoft.com/style-guide/) with additional Rust-specific conventions:

- Use active voice
- Be concise and clear
- Include code examples for API documentation
- Reference related documentation where appropriate
- Include version information when relevant

### Adding New Documentation
1. Create a new markdown file in the appropriate directory
2. Add a reference to the main README.md if it's a major document
3. Follow the established format for similar documents
4. Include metadata at the top (date, version, status)
5. Submit a pull request for review

### Updating Existing Documentation
1. Ensure you're working with the latest version
2. Make changes in line with the style guide
3. Update the "Last Updated" date if present
4. Add a note in the Recent Updates section if it's a significant change
5. Submit a pull request for review

## Contact

For questions or suggestions about the documentation, please contact the Documentation Team at docs@navius.example.com 
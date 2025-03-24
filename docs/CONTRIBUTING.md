---
title: Documentation Contributing Guide
description: "Guide for contributing to Navius documentation, including setup instructions, standards, and build process"
category: contributing
tags:
  - documentation
  - contributing
  - mdbook
  - standards
related:
  - contributing/contribution-guide.md
  - reference/standards/documentation-standards.md
last_updated: March 23, 2025
version: 1.0
---

# Contributing to Navius Documentation

This guide explains how to work with and contribute to the Navius documentation.

## Quick Start

1. Install mdBook:
   ```bash
   cargo install mdbook
   ```

2. Serve documentation locally:
   ```bash
   cd docs
   mdbook serve --open
   ```

## Directory Structure

```
/docs
├── book/              # Generated documentation site
├── book.toml         # mdBook configuration
├── theme/            # Custom theme files
│   ├── custom.css    # Custom styling
│   └── custom.js     # Enhanced functionality
├── SUMMARY.md        # Documentation navigation
├── README.md         # Main documentation page
├── getting-started/  # Getting started guides
├── guides/          # Various guides
├── reference/       # Reference documentation
└── contributing/    # Contributing guidelines
```

## Documentation Standards

1. **File Organization**
   - Use kebab-case for file names
   - Group related documents in appropriate directories
   - Include README.md in each directory

2. **Metadata Headers**
   ```yaml
   ---
   title: Document Title
   description: "Clear, concise description"
   category: category-name
   tags:
     - relevant
     - tags
   related:
     - path/to/related/doc.md
   last_updated: YYYY-MM-DD
   version: 1.0
   ---
   ```

3. **Content Guidelines**
   - Use clear, concise language
   - Include code examples where relevant
   - Add cross-references to related documents
   - Keep content up-to-date

## Building Documentation

The documentation is built using mdBook. You can build it in several ways:

1. **Using Make** (recommended):
   ```bash
   make docs-build    # Build documentation
   make docs-serve    # Serve documentation locally
   make docs-check    # Check for broken links and other issues
   ```

2. **Using mdBook directly**:
   ```bash
   cd docs
   mdbook build      # Build documentation
   mdbook serve      # Serve documentation locally
   mdbook test       # Run documentation tests
   ```

3. **Using CI/CD**:
   Documentation is automatically built and deployed on pushes to main.

## Adding New Content

1. **Create new file** in appropriate directory
2. **Add to SUMMARY.md** for navigation
3. **Include metadata header**
4. **Follow style guide**
5. **Add related documents**
6. **Test locally**

## Making Changes

1. Create a new branch
2. Make your changes
3. Test locally using `make docs-serve`
4. Submit a pull request

## Common Tasks

### Adding a New Guide

1. Create file in appropriate directory:
   ```bash
   touch docs/guides/features/my-guide.md
   ```

2. Add metadata header
3. Add to SUMMARY.md
4. Write content
5. Test locally

### Updating Navigation

1. Edit `docs/SUMMARY.md`
2. Follow existing structure
3. Test navigation locally

### Customizing Appearance

1. Edit `docs/theme/custom.css`
2. Edit `docs/theme/custom.js`
3. Test changes locally

## Need Help?

- Check [Documentation Standards](../reference/standards/documentation-standards.md)
- Join our [Discord Community](https://discord.gg/navius)
- Open an issue on GitHub 
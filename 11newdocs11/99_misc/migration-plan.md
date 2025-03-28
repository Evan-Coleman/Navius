---
title: Documentation Migration Plan
description: Plan for migrating existing documentation to the new structure
category: misc
tags:
  - documentation
  - migration
  - planning
related:
  - ../../roadmaps/30_documentation-reorganization-roadmap.md
  - ../../roadmaps/30_documentation-reorganization-instructions.md
last_updated: May 30, 2024
version: 1.0
---

# Documentation Migration Plan

## Overview

This document outlines the mapping of existing documentation to the new directory structure, based on the [Documentation Reorganization Roadmap](../../roadmaps/30_documentation-reorganization-roadmap.md) and [Documentation Reorganization Instructions](../../roadmaps/30_documentation-reorganization-instructions.md).

## Content Mapping

### 01_getting_started

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/getting-started/* | docs/new/01_getting_started/* | Direct migration with updated formatting |
| docs/getting-started/README.md | docs/new/01_getting_started/README.md | Direct migration with updated formatting |
| docs/getting-started/installation.md | docs/new/01_getting_started/installation.md | Direct migration with updated formatting |
| docs/getting-started/hello-world.md | docs/new/01_getting_started/hello-world.md | Direct migration with updated formatting |
| docs/getting-started/first-steps.md | docs/new/01_getting_started/first-steps.md | Direct migration with updated formatting |
| docs/getting-started/development-setup.md | docs/new/01_getting_started/development-setup.md | Direct migration with updated formatting |
| docs/getting-started/cli-reference.md | docs/new/01_getting_started/cli-reference.md | Direct migration with updated formatting |

### 02_examples

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/examples/* | docs/new/02_examples/* | Direct migration with updated formatting |
| docs/examples/README.md | docs/new/02_examples/README.md | Direct migration with updated formatting |
| docs/examples/basic-application-example.md | docs/new/02_examples/basic-application-example.md | Direct migration with updated formatting |
| docs/examples/rest-api-example.md | docs/new/02_examples/rest-api-example.md | Direct migration with updated formatting |
| docs/examples/dependency-injection-example.md | docs/new/02_examples/dependency-injection-example.md | Direct migration with updated formatting |
| docs/examples/error-handling-example.md | docs/new/02_examples/error-handling-example.md | Direct migration with updated formatting |
| docs/examples/cache-provider-example.md | docs/new/02_examples/cache-provider-example.md | Direct migration with updated formatting |
| docs/examples/database-service-example.md | docs/new/02_examples/database-service-example.md | Direct migration with updated formatting |
| docs/examples/health-service-example.md | docs/new/02_examples/health-service-example.md | Direct migration with updated formatting |
| docs/examples/logging-service-example.md | docs/new/02_examples/logging-service-example.md | Direct migration with updated formatting |
| docs/examples/repository-pattern-example.md | docs/new/02_examples/repository-pattern-example.md | Direct migration with updated formatting |
| docs/examples/two-tier-cache-example.md | docs/new/02_examples/two-tier-cache-example.md | Direct migration with updated formatting |
| docs/examples/configuration-example.md | docs/new/02_examples/configuration-example.md | Direct migration with updated formatting |
| docs/examples/custom-service-example.md | docs/new/02_examples/custom-service-example.md | Direct migration with updated formatting |
| docs/examples/graphql-example.md | docs/new/02_examples/graphql-example.md | Direct migration with updated formatting |
| docs/examples/server-customization-example.md | docs/new/02_examples/server-customization-example.md | Direct migration with updated formatting |
| docs/examples/20_spring-boot-comparison.md | docs/new/02_examples/spring-boot-comparison.md | Rename and migrate with updated formatting |

### 03_contributing

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/contributing/* | docs/new/03_contributing/* | Direct migration with updated formatting |
| docs/contributing/README.md | docs/new/03_contributing/README.md | Direct migration with updated formatting |
| docs/contributing/CONTRIBUTING.md | docs/new/03_contributing/CONTRIBUTING.md | Direct migration with updated formatting |
| docs/contributing/code-of-conduct.md | docs/new/03_contributing/code-of-conduct.md | Direct migration with updated formatting |
| docs/contributing/contribution-guide.md | docs/new/03_contributing/contribution-guide.md | Direct migration with updated formatting |
| docs/contributing/development-process.md | docs/new/03_contributing/development-process.md | Direct migration with updated formatting |
| docs/contributing/development-setup.md | docs/new/03_contributing/development-setup.md | Direct migration with updated formatting |
| docs/contributing/documentation-guidelines.md | docs/new/03_contributing/documentation-guidelines.md | Already migrated |
| docs/contributing/ide-setup.md | docs/new/03_contributing/ide-setup.md | Direct migration with updated formatting |
| docs/contributing/onboarding.md | docs/new/03_contributing/onboarding.md | Direct migration with updated formatting |
| docs/contributing/test-implementation-template.md | docs/new/03_contributing/test-implementation-template.md | Direct migration with updated formatting |
| docs/contributing/testing-guidelines.md | docs/new/03_contributing/testing-guidelines.md | Direct migration with updated formatting |
| docs/contributing/testing-prompt.md | docs/new/03_contributing/testing-prompt.md | Direct migration with updated formatting |

### 04_guides

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/guides/* | docs/new/04_guides/* | Direct migration with updated formatting |
| docs/guides/README.md | docs/new/04_guides/README.md | Direct migration with updated formatting |
| docs/guides/application-structure.md | docs/new/04_guides/application-structure.md | Direct migration with updated formatting |
| docs/guides/caching-strategies.md | docs/new/04_guides/caching-strategies.md | Direct migration with updated formatting |
| docs/guides/configuration.md | docs/new/04_guides/configuration.md | Direct migration with updated formatting |
| docs/guides/dependency-injection.md | docs/new/04_guides/dependency-injection.md | Direct migration with updated formatting |
| docs/guides/error-handling.md | docs/new/04_guides/error-handling.md | Direct migration with updated formatting |
| docs/guides/feature-selection.md | docs/new/04_guides/feature-selection.md | Direct migration with updated formatting |
| docs/guides/postgresql_integration.md | docs/new/04_guides/postgresql-integration.md | Rename and migrate with updated formatting |
| docs/guides/service-registration.md | docs/new/04_guides/service-registration.md | Direct migration with updated formatting |
| docs/guides/testing.md | docs/new/04_guides/testing.md | Direct migration with updated formatting |
| docs/guides/deployment/* | docs/new/04_guides/deployment/* | Direct migration with subdirectory structure |
| docs/guides/development/* | docs/new/04_guides/development/* | Direct migration with subdirectory structure |
| docs/guides/features/* | docs/new/04_guides/features/* | Direct migration with subdirectory structure |

### 05_reference

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/reference/* | docs/new/05_reference/* | Direct migration with updated formatting |
| docs/reference/README.md | docs/new/05_reference/README.md | Direct migration with updated formatting |
| docs/reference/api/* | docs/new/05_reference/api/* | Direct migration with subdirectory structure |
| docs/reference/architecture/* | docs/new/05_reference/architecture/* | Direct migration with subdirectory structure |
| docs/reference/configuration/* | docs/new/05_reference/configuration/* | Direct migration with subdirectory structure |
| docs/reference/patterns/* | docs/new/05_reference/patterns/* | Direct migration with subdirectory structure |
| docs/reference/standards/* | docs/new/05_reference/standards/* | Direct migration with subdirectory structure |
| docs/architecture/* | docs/new/05_reference/architecture/* | Consolidate with existing reference/architecture |
| docs/api/* | docs/new/05_reference/api/* | Consolidate with existing reference/api |
| docs/auth/* | docs/new/05_reference/auth/* | Create new subdirectory and migrate |
| docs/generated/* | docs/new/05_reference/generated/* | Direct migration with subdirectory structure |

### 98_roadmaps

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/roadmaps/* | docs/new/98_roadmaps/* | Direct migration with updated formatting |
| docs/roadmaps/README.md | docs/new/98_roadmaps/README.md | Direct migration with updated formatting |
| docs/roadmaps/01-dependency-injection.md | docs/new/98_roadmaps/01-dependency-injection.md | Direct migration with updated formatting |
| docs/roadmaps/02-database-integration.md | docs/new/98_roadmaps/02-database-integration.md | Direct migration with updated formatting |
| docs/roadmaps/03-testing-framework.md | docs/new/98_roadmaps/03-testing-framework.md | Direct migration with updated formatting |
| docs/roadmaps/completed/* | docs/new/98_roadmaps/completed/* | Direct migration with subdirectory structure |
| docs/roadmaps/roadmap-instructions/* | docs/new/98_roadmaps/roadmap-instructions/* | Direct migration with subdirectory structure |
| docs/roadmaps/tests/* | docs/new/98_roadmaps/tests/* | Direct migration with subdirectory structure |

### 99_misc

| Current Location | New Location | Migration Strategy |
|------------------|--------------|-------------------|
| docs/templates/* | docs/new/99_misc/templates/* | Direct migration with subdirectory structure |
| docs/release-notes/* | docs/new/99_misc/release-notes/* | Direct migration with subdirectory structure |
| docs/issues/* | docs/new/99_misc/issues/* | Direct migration with subdirectory structure |
| docs/notes/* | docs/new/99_misc/notes/* | Direct migration with subdirectory structure |
| docs/feature-system.md | docs/new/99_misc/feature-system.md | Direct migration with updated formatting |
| docs/testing-guidance.md | docs/new/99_misc/testing-guidance.md | Direct migration with updated formatting |
| docs/LICENSE.md | docs/new/99_misc/LICENSE.md | Direct migration with updated formatting |
| docs/CONTRIBUTING.md | docs/new/03_contributing/CONTRIBUTING.md | Move to appropriate section |
| docs/README.md | docs/new/README.md | Update to reflect new structure |
| docs/SUMMARY.md | docs/new/SUMMARY.md | Regenerate based on new structure |

## Implementation Strategy

1. Create all necessary subdirectories in the new structure
2. Migrate files according to the mapping above
3. Update frontmatter in each file
4. Fix internal links to reflect the new structure
5. Update cross-references
6. Validate the new structure

## Next Steps

1. Implement the directory structure creation with subdirectories
2. Begin migrating high-priority documentation:
   - Getting Started section
   - Contributing section
   - Key guides and references
3. Update frontmatter and links
4. Validate and test the new structure 
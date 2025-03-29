# Workspace Migration Project

This directory contains all resources, documentation, code samples, and scripts related to the migration from feature flags to a workspace-based approach in the Navius project.

## Directory Structure

- **`/roadmap`**: Contains the high-level roadmap and implementation plan
  - `workspace-migration-plan.md`: Detailed migration plan with phases
  - `40-workspace-migration.md`: Official roadmap entry with tasks and tracking

- **`/docs`**: Documentation about the migration process
  - `workspace-migration-tutorial.md`: Step-by-step tutorial for migration
  - `workspace-vs-feature-flags.md`: Comparison between approaches

- **`/scripts`**: Helper scripts to assist with the migration
  - `migration-script.sh`: Automated script to help with crate creation and analysis

- **`/examples`**: Example code and configuration files
  - `README.md`: Example updated README for the workspace structure
  - `Cargo.toml`: Example workspace root Cargo.toml
  - `/crates`: Example crate structures
    - `/navius-core`: Core crate example
    - `/navius-metrics`: Metrics crate example
    - `/navius-metrics-prometheus`: Backend implementation example

- **`/reports`**: Progress reports and metrics (to be added as migration progresses)

## Getting Started

1. **Review the Roadmap**: Start by reading the roadmap in `/roadmap/40-workspace-migration.md` to understand the overall plan and current status.

2. **Understand the Migration Strategy**: Read `/roadmap/workspace-migration-plan.md` for a detailed understanding of the migration approach.

3. **Follow the Tutorial**: The `/docs/workspace-migration-tutorial.md` file provides step-by-step instructions for migrating modules.

4. **Examine the Examples**: The `/examples` directory contains examples of how files should be structured after migration.

5. **Use the Migration Script**: The `/scripts/migration-script.sh` script can help automate parts of the migration process.

## Important Rules to Follow

Always refer to the following rules when working on the workspace migration:

- **[.cursor/rules/020-roadmaps.mdc](mdc:.cursor/rules/020-roadmaps.mdc)**: Guidelines for working with roadmap files and updates
- **[.cursor/rules/023-mod-rules.mdc](mdc:.cursor/rules/023-mod-rules.mdc)**: Rules for module organization (especially important during extraction)

## Migration Process

The migration follows these main phases:

1. **Setup Workspace Structure**: Configure the workspace and create the core crate
2. **Module Extraction**: Move modules one by one into separate crates
3. **Application Integration**: Connect all crates in the main application
4. **Cleanup and Optimization**: Remove old code and optimize

### Current Phase: Setup Workspace Structure

We are currently in Phase 1 of the migration, focusing on:
- Setting up the workspace configuration
- Creating the navius-core crate
- Setting up testing infrastructure

## Tracking Progress

As you work on the migration:

1. Update the roadmap (`/roadmap/40-workspace-migration.md`) to mark tasks as completed
2. Add progress reports to the `/reports` directory
3. Update documentation as needed to reflect any changes to the approach

## Assistance

If you need help with the migration process, refer to:
- The migration tutorial in `/docs/workspace-migration-tutorial.md`
- The helper script in `/scripts/migration-script.sh` (use `./scripts/migration-script.sh --help` for options)

## Migration Order

We're following this priority order for crate extraction:
1. navius-core
2. navius-auth
3. navius-metrics
4. navius-database
5. navius-cache
6. navius-api
7. navius-cli
8. navius-test-utils 
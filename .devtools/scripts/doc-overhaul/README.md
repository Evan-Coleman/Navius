# Documentation Overhaul Tools

This directory contains scripts for methodically improving documentation quality in a controlled, file-by-file approach. Instead of making bulk changes that could introduce errors, these tools focus on making targeted improvements to one file at a time.

## Available Scripts

### Main Script

- **improve_docs.sh**: The main user interface that guides you through the documentation improvement process. It will validate documentation, help you select files to work on, and apply necessary fixes one file at a time.

```bash
./scripts/doc-overhaul/improve_docs.sh
```

### Individual Tools

- **detailed_validation.sh**: Analyzes all documentation files without making changes, producing a detailed report of issues.

```bash
./scripts/doc-overhaul/detailed_validation.sh
```

- **fix_frontmatter.sh**: Adds or fixes frontmatter on a specific file.

```bash
./scripts/doc-overhaul/fix_frontmatter.sh path/to/file.md
```

- **add_sections.sh**: Adds only the Related Documents section to a document based on its type.

```bash
./scripts/doc-overhaul/add_sections.sh path/to/file.md
```

- **fix_links.sh**: Identifies and helps fix broken links in a single document.

```bash
./scripts/doc-overhaul/fix_links.sh path/to/file.md
```

## Workflow

The recommended workflow is:

1. Run `./scripts/doc-overhaul/improve_docs.sh` to start the guided process
2. Follow the interactive prompts to improve one document at a time
3. Review and approve changes before committing
4. Repeat until all documentation issues are fixed

## Link Path Standards

These scripts enforce the use of absolute paths from the project root for all internal documentation links, rather than relative paths. This has several key benefits:

1. **Resilience to Moves**: When files are relocated, absolute paths don't break
2. **Easier Maintenance**: No need to calculate relative paths with `../` which can be error-prone
3. **Better Readability**: Absolute paths make it immediately clear where the target document is located
4. **Consistent Pattern**: All links follow the same pattern

### Examples

```markdown
<!-- ✅ GOOD: Absolute paths from project root -->
[Installation Guide](/docs/guides/installation.md)
[Project Structure](/docs/architecture/project-structure.md)

<!-- ❌ BAD: Relative paths with ../ -->
[Installation Guide](../guides/installation.md)
[Project Structure](../architecture/project-structure.md)

<!-- ❌ BAD: Absolute paths without /docs/ prefix -->
[Installation Guide](/guides/installation.md)
[Project Structure](/architecture/project-structure.md)
```

### Important Note

All absolute paths should begin with `/docs/` to ensure they correctly resolve both in the repository and when the documentation is deployed. Paths that start with just `/` without the `docs/` prefix will be automatically corrected by the scripts.

## Philosophy

This approach follows these key principles:

1. **Control**: Make changes to one file at a time to maintain control of the process
2. **Verification**: Verify each change before committing
3. **Incremental Progress**: Break down a large overhaul into manageable steps
4. **Interactive**: Get human input on decisions that require judgment
5. **Reversible**: Every change can be reviewed and reversed if needed

## Documentation Standards

These tools enforce the following standards:

- All documents have proper frontmatter (title, description, category, tags, etc.)
- All documents include a Related Documents section for easy navigation
- All internal links use absolute paths from the project root (starting with /docs/)
- All links resolve correctly to existing files

## Benefits vs. Batch Processing

While batch processing scripts might seem faster, this file-by-file approach:

1. Reduces the risk of introducing errors
2. Allows for human judgment on complex decisions
3. Ensures higher quality results
4. Makes it easier to track progress
5. Creates smaller, more focused git commits

## Next Steps After Completion

Once the documentation overhaul is complete:

1. Update the roadmap status to reflect completion
2. Implement a documentation validation step in CI to maintain quality
3. Create documentation contribution guidelines based on the established standards
4. Set up automated documentation testing 
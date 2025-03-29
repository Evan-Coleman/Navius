---
title: "Git Workflow for Navius Development"
description: "Best practices and guidelines for using Git effectively in Navius projects"
category: "Guides"
tags: ["development", "git", "version control", "collaboration", "branching", "commits"]
last_updated: "April 7, 2025"
version: "1.0"
---

# Git Workflow for Navius Development

This guide outlines the recommended Git workflow and best practices for Navius projects. Following these guidelines ensures consistent version control, simplifies collaboration, and maintains a clean project history.

## Table of Contents

- [Git Configuration](#git-configuration)
- [Branching Strategy](#branching-strategy)
- [Commit Guidelines](#commit-guidelines)
- [Pull Requests and Code Review](#pull-requests-and-code-review)
- [Integration and Deployment](#integration-and-deployment)
- [Advanced Git Techniques](#advanced-git-techniques)
- [Troubleshooting](#troubleshooting)

## Git Configuration

### Initial Setup

Configure your Git environment for Navius development:

```bash
# Set your identity
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Configure line endings (important for cross-platform development)
# For macOS/Linux
git config --global core.autocrlf input
# For Windows
git config --global core.autocrlf true

# Enable helpful coloring
git config --global color.ui auto

# Set default branch to main
git config --global init.defaultBranch main
```

### Navius-Specific Configuration

Add the recommended Git hooks for Navius development:

```bash
# Copy hooks from the repository
cp -r .devtools/git-hooks/* .git/hooks/
chmod +x .git/hooks/*
```

These hooks provide:
- Pre-commit formatting with rustfmt
- Pre-push checks with Clippy
- Commit message validation

## Branching Strategy

Navius uses a simplified GitFlow branching strategy:

### Main Branches

- **`main`** - The production-ready code
- **`develop`** - Integration branch for features (when applicable for larger projects)

### Supporting Branches

- **`feature/*`** - New features or enhancements
- **`bugfix/*`** - Bug fixes
- **`hotfix/*`** - Urgent fixes for production
- **`release/*`** - Release preparation branches
- **`docs/*`** - Documentation changes
- **`refactor/*`** - Code refactoring without changing functionality
- **`test/*`** - Adding or modifying tests

### Branch Naming Convention

Follow this naming convention for branches:

```
<type>/<issue-number>-<short-description>
```

Examples:
- `feature/123-add-user-authentication`
- `bugfix/456-fix-database-connection`
- `docs/789-update-api-documentation`

### Branch Lifecycle

1. **Create a branch** from the appropriate base:
   ```bash
   # For features and most work, branch from main
   git checkout main
   git pull
   git checkout -b feature/123-add-user-authentication
   ```

2. **Work on your branch**:
   ```bash
   # Make changes, commit frequently
   git add .
   git commit -m "Add login form component"
   ```

3. **Keep your branch updated**:
   ```bash
   # Regularly pull and rebase from main
   git fetch origin
   git rebase origin/main
   ```

4. **Complete your work** and prepare for merge:
   ```bash
   # Ensure tests pass
   cargo test
   
   # Push your branch
   git push -u origin feature/123-add-user-authentication
   ```

5. **Create a pull request** (on GitHub/GitLab)

6. **After approval and merge**, delete the branch:
   ```bash
   git checkout main
   git pull
   git branch -d feature/123-add-user-authentication
   ```

## Commit Guidelines

### Commit Message Format

Navius follows the Conventional Commits specification. Each commit message should have this structure:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Where:
- **`<type>`** is one of:
  - `feat`: A new feature
  - `fix`: A bug fix
  - `docs`: Documentation changes
  - `style`: Code style changes (formatting, indentation)
  - `refactor`: Code refactoring
  - `test`: Adding or modifying tests
  - `chore`: Changes to the build process, tools, etc.
  - `perf`: Performance improvements

- **`<scope>`** is optional and specifies the module affected (e.g., `auth`, `api`, `db`)
- **`<description>`** is a concise summary of the change

Example:
```
feat(auth): implement JWT token validation

Add JWT validation middleware to protect API routes.
Includes token expiration checking and role-based verification.

Resolves: #123
```

### Commit Best Practices

1. **Make frequent, small commits** - Easier to review and understand
2. **One logical change per commit** - Don't mix unrelated changes
3. **Write clear commit messages** - Explain what and why, not how
4. **Reference issue numbers** - Link commits to issues
5. **Ensure code compiles before committing** - Don't break the build

## Pull Requests and Code Review

### Creating a Pull Request

1. Push your branch to the remote repository:
   ```bash
   git push -u origin feature/123-add-user-authentication
   ```

2. Create a pull request using your project's Git hosting service (GitHub/GitLab)

3. Complete the pull request template with:
   - A clear description of the changes
   - Link to related issues
   - Testing procedures
   - Screenshots (if UI changes)
   - Any deployment considerations

### Pull Request Guidelines

- **Keep PRs focused and small** - Ideally under 500 lines of changes
- **Complete the PR description thoroughly** - Help reviewers understand your changes
- **Self-review before requesting reviews** - Check your own code first
- **Respond promptly to review comments** - Maintain momentum
- **Rebase before merging** - Keep history clean

### Code Review Process

1. **Automated checks** - CI must pass before review
2. **Reviewer assignment** - At least one required reviewer
3. **Review feedback cycle** - Address all comments
4. **Approval and merge** - Squash or rebase merge preferred

## Integration and Deployment

### Continuous Integration

Navius uses GitHub Actions/GitLab CI for continuous integration. Every commit triggers:

1. **Compilation** - Ensuring code builds
2. **Testing** - Running unit and integration tests
3. **Linting** - Checking code quality with Clippy
4. **Formatting** - Verifying rustfmt compliance

### Release Process

1. **Version Bumping**:
   ```bash
   # Update version in Cargo.toml and other files
   cargo bump patch  # or minor, major
   
   # Commit version bump
   git add .
   git commit -m "chore: bump version to 1.2.3"
   ```

2. **Create a release tag**:
   ```bash
   git tag -a v1.2.3 -m "Release v1.2.3"
   git push origin v1.2.3
   ```

3. **Create a release** on GitHub/GitLab with release notes

## Advanced Git Techniques

### Useful Git Commands

```bash
# View branch history with graph
git log --graph --oneline --decorate

# Temporarily stash changes
git stash
git stash pop

# Find which commit introduced a bug
git bisect start
git bisect bad  # current commit has the bug
git bisect good <commit-hash>  # known good commit

# Show changes between commits
git diff <commit1>..<commit2>

# Amend last commit
git commit --amend

# Interactive rebase to clean history
git rebase -i HEAD~3  # rebase last 3 commits
```

### Git Workflows for Specific Scenarios

#### Handling Merge Conflicts

1. **Rebasing approach**:
   ```bash
   git fetch origin
   git rebase origin/main
   # Resolve conflicts
   git add .
   git rebase --continue
   ```

2. **Merging approach**:
   ```bash
   git fetch origin
   git merge origin/main
   # Resolve conflicts
   git add .
   git commit
   ```

#### Cherry-picking Specific Commits

```bash
# Find the commit hash
git log

# Cherry-pick the commit
git cherry-pick <commit-hash>
```

#### Creating a Hotfix

```bash
# Branch from production tag
git checkout v1.2.3
git checkout -b hotfix/critical-security-fix

# Make changes, commit, and push
git add .
git commit -m "fix: address security vulnerability in auth"
git push -u origin hotfix/critical-security-fix

# After review and approval, merge to main and develop
```

## Troubleshooting

### Common Issues and Solutions

#### "Permission denied" when pushing

- **Issue**: SSH key not configured
- **Solution**: 
  ```bash
  # Generate SSH key
  ssh-keygen -t ed25519 -C "your.email@example.com"
  
  # Add to SSH agent
  eval "$(ssh-agent -s)"
  ssh-add ~/.ssh/id_ed25519
  
  # Add public key to GitHub/GitLab
  cat ~/.ssh/id_ed25519.pub
  ```

#### Accidentally committed sensitive data

- **Issue**: Credentials or sensitive data committed
- **Solution**:
  ```bash
  # Remove file from Git history
  git filter-branch --force --index-filter "git rm --cached --ignore-unmatch path/to/sensitive-file" --prune-empty --tag-name-filter cat -- --all
  
  # Force push
  git push origin --force --all
  
  # Update credentials/tokens immediately
  ```

#### Merge conflicts during rebase

- **Issue**: Complex conflicts during rebase
- **Solution**:
  ```bash
  # Abort rebase if too complex
  git rebase --abort
  
  # Try merge instead
  git merge origin/main
  
  # Or use a visual merge tool
  git mergetool
  ```

### Getting Help

If you're stuck with Git issues:

1. Check the [Navius Developer Forum](https://forum.navius.dev/category/git)
2. Review [Git documentation](https://git-scm.com/docs)
3. Ask in the #dev-help channel on the [Navius Discord Server](https://discord.gg/navius)

## Related Resources

- [Development Workflow](./development-workflow.md)
- [Code Review Process](../contributing/code-review-process.md)
- [Contributing Guidelines](../contributing/contributing-guidelines.md)
- [IDE Setup Guide](./ide-setup.md)
- [Official Git Documentation](https://git-scm.com/doc)

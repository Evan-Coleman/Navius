---
title: "CONTRIBUTING"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Contributing to Navius"
description: "# Clone the repository"
category: contributing
tags:
  - development
  - documentation
  - performance
  - security
  - testing
last_updated: March 27, 2025
version: 1.0
---
# Contributing to Navius

Thank you for your interest in contributing to Navius! This document provides guidelines and instructions for contributing to the project.

## Repository Structure

Navius uses a dual repository approach:

- **Primary Repository (GitLab)**: [gitlab.com/ecoleman2/navius](https://gitlab.com/ecoleman2/navius)
  - All core development work
  - Issue tracking and planning
  - CI/CD pipelines and deployments
  - Official releases
  - Team member contributions

- **Secondary Repository (GitHub)**: [github.com/Evan-Coleman/Navius](https://github.com/Evan-Coleman/Navius)
  - Public visibility and community engagement
  - Community contributions and pull requests
  - Documentation and examples
  - Public issue discussions
  - Status badges for marketing

## How to Contribute

### For Core Team Members

1. Work directly on the GitLab repository
2. Create feature branches from `main`
3. Submit Merge Requests to GitLab
4. Follow the GitLab workflow for code review
5. After merging, changes will be automatically mirrored to GitHub

### For Community Contributors

1. Fork the GitHub repository
2. Create a feature branch from `main`
3. Implement your changes
4. Submit a Pull Request to the GitHub repository
5. Your PR will be reviewed by maintainers
6. If accepted, your contribution will be imported to GitLab and processed through our standard pipeline
7. After merging on GitLab, the changes will be mirrored back to GitHub

## Development Setup

```bash
# Clone the repository
git clone https://gitlab.com/ecoleman2/navius.git
# OR for community contributors
git clone https://github.com/Evan-Coleman/Navius.git

# Navigate to the project directory
cd navius

# Set up development environment
./run_dev.sh
```

## Code Style and Standards

- Follow Rust's official [style guidelines](https://doc.rust-lang.org/1.0.0/style/README.html)
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new features
- Maintain or improve code coverage
- Document public APIs using rustdoc

## Commit Message Format

```
type(scope): subject

body
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, missing semi-colons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Code change that improves performance
- `test`: Adding or updating tests
- `chore`: Changes to the build process or auxiliary tools

## Testing

Run the test suite before submitting your contribution:

```bash
cargo test
```

For coverage information:

```bash
cargo tarpaulin
```

## Issue Reporting

### Core Business Issues

For core business issues, security vulnerabilities, or features on the roadmap:

- Report directly on [GitLab Issues](https://gitlab.com/ecoleman2/navius/-/issues)
- Use the appropriate issue template
- Provide detailed reproduction steps
- Link to any related issues or documentation

### Community Feedback and Bug Reports

For general feedback, suggestions, or bug reports from community users:

- Report on [GitHub Issues](https://github.com/Evan-Coleman/Navius/issues)
- Use the appropriate issue template
- Community issues may be transferred to GitLab if they align with our roadmap

## Documentation

- Update documentation for new features
- Provide code examples when appropriate
- Ensure documentation builds correctly
- Check for spelling and grammar

## Questions?

If you have any questions about contributing:

- For core team: Use GitLab discussions
- For community: Open a discussion on GitHub

Thank you for contributing to Navius!

## Running Tests

Navius has a comprehensive test suite. To run all tests:

```bash
cargo test
```

See the [Testing Guide](docs/testing_guide.md) for more information on testing.

## Code Style

Navius follows the standard Rust code style. We use `rustfmt` and `clippy` to enforce style guidelines:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy
```

## Documentation

When adding or modifying features, please update the documentation accordingly:

- Add or update doc comments in the code
- Update relevant markdown files in the `docs/` directory
- Consider adding examples to demonstrate usage

## Releasing

Releases are managed by the core team. We follow semantic versioning (MAJOR.MINOR.PATCH).

## Getting Help

If you need help with contributing, please:

- Check the [Developer Guide](docs/DEVELOPMENT.md)
- Join our [Discord community](https://discord.gg/navius)
- Open a GitHub issue with your question

## Acknowledgments

Thank you to all the contributors who have helped make Navius better!

## License

By contributing to Navius, you agree that your contributions will be licensed under the project's [Apache License 2.0](LICENSE). 

## Related Documents
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to the project
- [Development Setup](../01_getting_started/development-setup.md) - Setting up your development environment

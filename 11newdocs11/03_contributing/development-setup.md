# Development Setup for Contributors

This guide provides instructions for setting up a development environment specifically for contributing to the Navius project. If you're just using Navius in your own projects, refer to the [general development setup](../getting-started/development-setup.md) guide instead.

## Prerequisites

Before setting up your development environment for contributions, ensure you have:

- Completed the [Installation Guide](../getting-started/installation.md)
- Read the [Contribution Guide](contribution-guide.md)
- Familiarized yourself with our [Code of Conduct](code-of-conduct.md)
- Git installed and configured
- A GitHub account

## Fork the Repository

1. Visit the [Navius repository](https://github.com/navius-framework/navius)
2. Click the "Fork" button in the top-right corner
3. Clone your forked repository locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/navius.git
   cd navius
   ```
4. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/navius-framework/navius.git
   ```

## Development Environment Setup

### 1. IDE Configuration

We recommend using Visual Studio Code or JetBrains CLion for Navius development.

#### Visual Studio Code

1. **Install Visual Studio Code**
   - Download and install from [code.visualstudio.com](https://code.visualstudio.com/)

2. **Install Essential Extensions**
   ```bash
   code --install-extension rust-lang.rust-analyzer
   code --install-extension tamasfe.even-better-toml
   code --install-extension serayuzgur.crates
   code --install-extension vadimcn.vscode-lldb
   ```

3. **Configure VS Code Settings**
   
   Create or update `.vscode/settings.json` in the project directory:

   ```json
   {
     "rust-analyzer.checkOnSave.command": "clippy",
     "rust-analyzer.checkOnSave.allTargets": true,
     "editor.formatOnSave": true,
     "rust-analyzer.cargo.allFeatures": true,
     "rust-analyzer.procMacro.enable": true
   }
   ```

#### JetBrains CLion

1. **Install CLion**
   - Download and install from [jetbrains.com/clion](https://www.jetbrains.com/clion/)

2. **Install Rust Plugin**
   - Go to Settings/Preferences → Plugins
   - Search for "Rust" and install the official plugin
   - Restart CLion

3. **Configure Toolchain**
   - Go to Settings/Preferences → Languages & Frameworks → Rust
   - Set the toolchain location to your Rust installation
   - Enable "Run external linter to analyze code on the fly"

### 2. Set Up Commit Hooks

Contributors are required to use our pre-commit hooks to ensure code quality:

1. **Install pre-commit**
   ```bash
   pip install pre-commit
   ```

2. **Set up hooks**
   ```bash
   pre-commit install
   ```

This will automatically run our standard checks before each commit, including:
- Clippy for linting
- Rustfmt for formatting
- Detection of large files
- Check for AWS keys and other secrets

### 3. Install Development Tools

1. **Install Required Cargo Extensions**
   ```bash
   cargo install cargo-edit     # For dependency management
   cargo install cargo-watch    # For auto-reloading during development
   cargo install cargo-expand   # For macro debugging
   cargo install cargo-tarpaulin # For code coverage
   cargo install cargo-audit    # For security audits
   ```

2. **Install mdBook for Documentation Development**
   ```bash
   cargo install mdbook
   cargo install mdbook-linkcheck
   ```

### 4. Set Up the Database

1. **Install PostgreSQL**
   - macOS: `brew install postgresql`
   - Ubuntu: `sudo apt install postgresql`
   - Windows: Download from [postgresql.org](https://www.postgresql.org/download/windows/)

2. **Create a development database**
   ```bash
   createdb navius_dev
   ```

3. **Run migrations**
   ```bash
   cd navius
   ./scripts/db_setup.sh
   ```

### 5. Configure Environment

1. **Create a development environment file**
   ```bash
   cp .env.example .env.development
   ```

2. **Edit the file with your local settings**
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/navius_dev
   RUST_LOG=debug
   JWT_SECRET=dev_secret_key_do_not_use_in_production
   REDIS_URL=redis://localhost:6379/0
   ```

## Development Workflow

### 1. Create a Feature Branch

Always work on a feature branch, never directly on `main` or `develop`:

```bash
git checkout -b feature/your-feature-name
```

### 2. Run the Development Server

```bash
cargo run
```

Or with auto-reloading:

```bash
cargo watch -x run
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with coverage report
cargo tarpaulin
```

### 4. Build Documentation

```bash
cd docs
mdbook build
```

View the documentation by opening `docs/book/index.html` in your browser.

### 5. Check Code Quality

Before submitting a pull request, verify your code passes all quality checks:

```bash
# Run clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt -- --check

# Run audit for security vulnerabilities
cargo audit
```

## Submitting Contributions

1. **Commit your changes**
   ```bash
   git add .
   git commit -m "Your detailed commit message"
   ```

2. **Pull latest changes from upstream**
   ```bash
   git fetch upstream
   git rebase upstream/develop
   ```

3. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request**
   - Go to the [Navius repository](https://github.com/navius-framework/navius)
   - Click "Pull requests" → "New pull request"
   - Click "compare across forks"
   - Select your fork and branch
   - Click "Create pull request"
   - Follow the pull request template

## Troubleshooting

### Common Issues

- **Rust Analyzer Not Working**: Ensure the rust-analyzer extension is properly installed and VS Code has been restarted
- **Pre-commit Hooks Failed**: Fix the issues reported by the hooks, or use `git commit --no-verify` to bypass (not recommended for final commits)
- **Database Connection Issues**: Ensure PostgreSQL is running and the credentials in your .env file are correct

## Additional Resources

- [Project Architecture](../architecture/README.md)
- [Coding Standards](../reference/standards/code-style.md)
- [Documentation Guidelines](documentation-guidelines.md)
- [Testing Guidelines](testing-guidelines.md)

## Getting Help

If you encounter problems with your development setup, please:

1. Check the troubleshooting section above
2. Search for similar issues in our GitHub Issues
3. Ask for help in our [Discord server](https://discord.gg/navius)
4. Create a new issue with the label "dev-setup" if it's a new problem 
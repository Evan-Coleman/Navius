---
title: Navius Development Environment Setup
description: Comprehensive guide for setting up a complete Navius development environment
category: getting-started
tags:
  - development
  - setup
  - tools
  - ide
  - configuration
related:
  - installation.md
  - first-steps.md
  - hello-world.md
  - ../03_contributing/coding-standards.md
last_updated: March 27, 2025
version: 1.1
status: active
---

# Navius Development Environment Setup

## Overview

This guide walks you through setting up a comprehensive development environment for working with the Navius framework. It covers IDE configuration, tools, extensions, and best practices that will enhance your development experience and productivity.

## Prerequisites

Before setting up your development environment, ensure you have:

- Completed the [Installation Guide](installation.md) for basic Navius setup
- Basic familiarity with command-line tools and Git
- Admin/sudo rights on your development machine
- Rust toolchain installed (1.70.0 or later)

## Quick Start

For experienced developers who want to get started quickly:

```bash
# Clone the repository if you haven't already
git clone https://github.com/your-organization/navius.git
cd navius

# Install recommended development tools
cargo install cargo-edit cargo-watch cargo-expand cargo-tarpaulin

# Set up VSCode with extensions (if using VSCode)
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb
code --install-extension matklad.rust-analyzer
code --install-extension bungcip.better-toml
```

## Step-by-step Setup

### 1. IDE Installation and Configuration

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
   code --install-extension matklad.rust-analyzer
   code --install-extension bungcip.better-toml
   ```

3. **Configure VS Code Settings**
   
   Create or update `.vscode/settings.json` in the project directory:

   ```json
   {
     "rust-analyzer.checkOnSave.command": "clippy",
     "rust-analyzer.checkOnSave.allTargets": true,
     "editor.formatOnSave": true,
     "rust-analyzer.cargo.allFeatures": true,
     "rust-analyzer.procMacro.enable": true,
     "[rust]": {
       "editor.defaultFormatter": "rust-lang.rust-analyzer"
     }
   }
   ```

4. **Configure VS Code Launch Configuration**

   Create or update `.vscode/launch.json` for debugging:

   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug Navius",
         "cargo": {
           "args": ["build", "--bin", "navius"],
           "filter": {
             "name": "navius",
             "kind": "bin"
           }
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "RUST_LOG": "debug"
         }
       }
     ]
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

4. **Configure Run Configurations**
   - Go to Run → Edit Configurations
   - Add a new Cargo configuration
   - Set the command to "run" and add any necessary environment variables

### 2. Git Configuration

1. **Configure Git Identity**
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

2. **Set Up Git Hooks**
   ```bash
   cd navius
   cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

3. **Configure Git Hooks**
   
   Edit `.git/hooks/pre-commit` to include:

   ```bash
   #!/bin/sh
   
   # Run clippy before commit
   cargo clippy -- -D warnings
   if [ $? -ne 0 ]; then
     echo "Clippy failed, commit aborted"
     exit 1
   fi
   
   # Run tests before commit
   cargo test
   if [ $? -ne 0 ]; then
     echo "Tests failed, commit aborted"
     exit 1
   fi
   ```

4. **Configure Git Aliases (Optional)**

   ```bash
   git config --global alias.st status
   git config --global alias.co checkout
   git config --global alias.br branch
   git config --global alias.cm "commit -m"
   ```

### 3. Command-line Tools

1. **Install Cargo Extensions**
   ```bash
   cargo install cargo-edit     # For dependency management
   cargo install cargo-watch    # For auto-reloading during development
   cargo install cargo-expand   # For macro debugging
   cargo install cargo-tarpaulin # For code coverage
   cargo install cargo-outdated # For checking outdated dependencies
   cargo install cargo-bloat    # For analyzing binary size
   ```

2. **Install Database Tools**
   ```bash
   # For PostgreSQL
   pip install pgcli           # Better PostgreSQL CLI
   
   # For Redis (if using Redis)
   brew install redis-cli      # macOS with Homebrew
   # or
   sudo apt install redis-tools # Ubuntu
   ```

3. **Install API Testing Tools**
   ```bash
   # Install httpie for API testing
   pip install httpie
   
   # Or install Postman
   # Download from https://www.postman.com/downloads/
   ```

4. **Install Documentation Tools**
   ```bash
   # Install mdbook for documentation previewing
   cargo install mdbook
   
   # Install additional mdbook components
   cargo install mdbook-mermaid  # For diagrams
   cargo install mdbook-linkcheck # For validating links
   ```

### 4. Environment Configuration

1. **Create Development Environment Files**
   ```bash
   cp .env.example .env.development
   ```
   
   Edit `.env.development` with your local settings:
   ```
   # Environment selection
   RUN_ENV=development
   
   # Logging
   RUST_LOG=debug
   
   # Database configuration
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/navius
   
   # Secrets (development only)
   JWT_SECRET=dev_secret_key_do_not_use_in_production
   
   # Other settings
   ENABLE_SWAGGER=true
   ```

2. **Configure Shell Aliases**
   
   Add to your `~/.bashrc` or `~/.zshrc`:
   ```bash
   # Navius development aliases
   alias ns="cd /path/to/navius && ./run_dev.sh"
   alias nt="cd /path/to/navius && cargo test"
   alias nc="cd /path/to/navius && cargo clippy"
   alias nw="cd /path/to/navius && cargo watch -x run"
   alias ndoc="cd /path/to/navius && cd docs && mdbook serve"
   ```

### 5. Docker Setup (Optional)

1. **Install Docker and Docker Compose**
   - Download and install from [docker.com](https://www.docker.com/products/docker-desktop)

2. **Verify Installation**
   ```bash
   docker --version
   docker-compose --version
   ```

3. **Set Up Development Containers**
   ```bash
   cd navius/test/resources/docker
   docker-compose -f docker-compose.dev.yml up -d
   ```

4. **Configure Docker Integration with IDE**
   - In VS Code, install the Docker extension
   - In CLion, configure Docker integration in settings

## Verification

To verify your development environment:

1. **Run the Linter**
   ```bash
   cargo clippy
   ```

2. **Run Tests**
   ```bash
   cargo test
   ```

3. **Start the Development Server**
   ```bash
   ./run_dev.sh --watch
   ```

4. **Access the Application**
   
   Open a browser and navigate to [http://localhost:3000/actuator/health](http://localhost:3000/actuator/health)

5. **Check API Documentation**

   Navigate to [http://localhost:3000/docs](http://localhost:3000/docs) to view the Swagger UI.

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| **Rust Analyzer Not Working** | Ensure the rust-analyzer extension is properly installed and VS Code has been restarted |
| **Build Errors** | Run `cargo clean` followed by `cargo build` to rebuild from scratch |
| **Git Hooks Not Running** | Check permissions with `ls -la .git/hooks/` and ensure hooks are executable |
| **Database Connection Errors** | Ensure PostgreSQL is running and the connection string is correct |
| **Missing Dependencies** | Run `rustup update` and `cargo update` to update Rust and dependencies |
| **Hot Reload Not Working** | Check cargo-watch installation and ensure watchexec is working |

### IDE-Specific Issues

- **VS Code**: If intellisense is not working, try "Restart Rust Analyzer" from the command palette
- **CLion**: If cargo features aren't recognized, invalidate caches via File → Invalidate Caches and Restart

### Environment-Specific Solutions

#### macOS
- If you encounter OpenSSL issues, install it via Homebrew: `brew install openssl`
- For PostgreSQL installation: `brew install postgresql`

#### Linux 
- On Ubuntu, install build essentials: `sudo apt install build-essential`
- For debugging tools: `sudo apt install lldb`

#### Windows
- Use Windows Subsystem for Linux (WSL2) for the best experience
- Install the C++ build tools with `rustup component add rust-src`

## Development Workflow Tips

1. **Use Feature Branches**
   - Create a new branch for each feature: `git checkout -b feature/my-feature`
   - Keep branches focused on a single task

2. **Run Tests Frequently**
   - Use `cargo test` or the alias `nt` before committing
   - Consider setting up continuous integration

3. **Format Code Automatically**
   - Use `cargo fmt` or enable formatting on save in your IDE
   - Run `cargo clippy` to catch common mistakes

4. **Review Documentation**
   - Update docs when changing functionality
   - Preview documentation changes with mdbook

## Next Steps

- Continue to [First Steps](first-steps.md) to learn about basic Navius concepts
- Try building a [Hello World Application](hello-world.md)
- Review the [Coding Standards](../03_contributing/coding-standards.md) before contributing code

## Related Documents

- [Installation Guide](installation.md) - Prerequisites for this guide
- [First Steps](first-steps.md) - Next steps after setting up your environment
- [Coding Standards](../03_contributing/coding-standards.md) - Guidelines for code contributions
- [Development Workflow](../04_guides/development/development-workflow.md) - Understanding the development process 
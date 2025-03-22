# Contributing to Rust Backend

Thank you for considering contributing to Rust Backend! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project.

## How to Contribute

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Make your changes
4. Run tests and ensure they pass
5. Submit a pull request

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/Evan-Coleman/rust-backend.git
   cd rust-backend
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Set up configuration:
   - Copy `.env.example` to `.env` and adjust as needed
   - Configuration files are in the `config/` directory

4. Run the server:
   ```bash
   ./run_dev.sh
   ```

## Pull Request Process

1. Update the README.md with details of changes if appropriate
2. Update the documentation if you're changing functionality
3. The PR should work on the main branch
4. Include tests for new functionality

## Coding Standards

- Follow Rust's official style guide
- Use meaningful variable and function names
- Write comments for complex logic
- Include documentation for public APIs

## Testing

- Write tests for new functionality
- Ensure all tests pass before submitting a PR
- Run tests with `cargo test`

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT License.

## Running the Application

You can run the application using the wrapper script:

```bash
# For development (default mode)
./run.sh

# For production
./run.sh --prod
```

Or specifically for development:

```bash
./run_dev.sh
``` 
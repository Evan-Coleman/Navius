.PHONY: docs-deps docs-build docs-serve docs-check docs-clean

# Documentation targets
docs-deps:
	@echo "Installing documentation dependencies..."
	cargo install mdbook
	@echo "Dependencies installed successfully."

docs-build:
	@echo "Building documentation..."
	cd docs && mdbook build
	@echo "Documentation built successfully in target/book/"

docs-serve:
	@echo "Starting documentation server..."
	cd docs && mdbook serve --open

docs-check:
	@echo "Checking documentation..."
	cd docs && mdbook test
	@echo "Running link checker..."
	find docs -name "*.md" -exec markdown-link-check {} \;
	@echo "Validating frontmatter..."
	./scripts/validate-docs.sh

docs-clean:
	@echo "Cleaning documentation build..."
	rm -rf target/book
	@echo "Documentation cleaned successfully."

# Combined targets
docs: docs-deps docs-build

# Help target
help:
	@echo "Navius Makefile targets:"
	@echo
	@echo "Documentation:"
	@echo "  docs-deps   - Install documentation dependencies"
	@echo "  docs-build  - Build the documentation"
	@echo "  docs-serve  - Serve documentation locally"
	@echo "  docs-check  - Check documentation for issues"
	@echo "  docs-clean  - Clean documentation build"
	@echo "  docs        - Install deps and build docs" 
#!/bin/bash
set -e

echo "🔧 Setting up Rust Backend development environment..."

# Make sure we have Rust installed
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust is already installed"
    rustc --version
fi

# Install development tools
echo "📦 Installing development tools..."
cargo install cargo-watch # For watch mode during development
cargo install cargo-tarpaulin # For test coverage

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p .github/workflows

echo "⚙️ Checking for config file..."
if [ ! -f .env ]; then
    echo "📝 Creating sample .env file..."
    cat > .env << EOF
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SERVER_PROTOCOL=http

# Logging Configuration
LOG_LEVEL=debug
LOG_FORMAT=json

# Authentication Configuration
AUTH_ENABLED=true
AUTH_DEBUG=false

# Cache Configuration
CACHE_ENABLED=true
CACHE_TTL_SECONDS=3600
EOF
    echo "✅ Sample .env file created"
else
    echo "✅ .env file already exists"
fi

# Build the project to verify everything works
echo "🏗️ Building project..."
cargo build

echo "🧪 Running tests..."
cargo test --lib

echo "✨ Setup complete! You can now run the server with:"
echo "   cargo run"
echo ""
echo "🔍 For development with auto-reload:"
echo "   cargo watch -x run"
echo ""
echo "📊 For test coverage:"
echo "   cargo tarpaulin --out Html" 
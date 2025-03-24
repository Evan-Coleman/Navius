#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Production deployment script - for use in production environments only

# Default configuration
CONFIG_DIR="config"
RUN_ENV="production"
SERVER_PORT=3000
MAX_STARTUP_TIME=60
MIGRATION_TIMEOUT=300
ROLLBACK_ON_FAILURE=true
ENABLE_MONITORING=true

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --config-dir=DIR       Use specified config directory (default: config)"
    echo "  --environment=ENV      Use specified environment (default: production)"
    echo "  --port=PORT            Specify server port (default: 3000)"
    echo "  --skip-migrations      Skip database migrations"
    echo "  --no-rollback          Don't rollback on failure"
    echo "  --no-monitoring        Don't enable monitoring integrations"
    echo "  --help                 Show this help message"
}

for arg in "$@"; do
    case $arg in
        --config-dir=*)
            CONFIG_DIR="${arg#*=}"
            shift
            ;;
        --environment=*)
            RUN_ENV="${arg#*=}"
            shift
            ;;
        --port=*)
            SERVER_PORT="${arg#*=}"
            shift
            ;;
        --skip-migrations)
            SKIP_MIGRATIONS=true
            shift
            ;;
        --no-rollback)
            ROLLBACK_ON_FAILURE=false
            shift
            ;;
        --no-monitoring)
            ENABLE_MONITORING=false
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            print_usage
            exit 1
            ;;
    esac
done

echo "==================================================="
echo "  Production Deployment Script"
echo "==================================================="
echo "Environment: $RUN_ENV"
echo "Port: $SERVER_PORT"
echo "==================================================="

# Check for systemd (for service management)
if ! command -v systemctl &> /dev/null; then
    echo "Error: systemctl not found. This script requires systemd for service management."
    exit 1
fi

# Check that we have the built release binary
if [ ! -f "./target/release/navius" ]; then
    echo "Error: Release binary not found at ./target/release/navius"
    exit 1
fi

# Export environment variables
export CONFIG_DIR="$CONFIG_DIR"
export RUN_ENV="$RUN_ENV"
export SERVER_PORT="$SERVER_PORT"
export RUST_LOG="${RUST_LOG:-info}"

# Ensure log directory exists
mkdir -p /var/log/navius

# Create a backup of the current version (for rollback)
if [ -f "/usr/local/bin/navius" ]; then
    echo "Backing up existing binary..."
    cp /usr/local/bin/navius /usr/local/bin/navius.bak
fi

# Run database migrations (if not skipped)
if [ "$SKIP_MIGRATIONS" != "true" ]; then
    echo "Running database migrations..."
    
    # Check DATABASE_URL
    if [ -z "$DATABASE_URL" ]; then
        echo "Error: DATABASE_URL environment variable not set. Required for migrations."
        exit 1
    fi
    
    # Set a timeout for migrations to prevent hanging
    echo "Running migrations with ${MIGRATION_TIMEOUT}s timeout..."
    timeout $MIGRATION_TIMEOUT sqlx migrate run --source src/app/database/migrations
    
    if [ $? -ne 0 ]; then
        echo "Error: Database migrations failed."
        if [ "$ROLLBACK_ON_FAILURE" = "true" ]; then
            echo "Rolling back to previous version..."
            if [ -f "/usr/local/bin/navius.bak" ]; then
                mv /usr/local/bin/navius.bak /usr/local/bin/navius
                echo "Rollback completed. Previous version restored."
            else
                echo "Warning: No backup found for rollback."
            fi
        fi
        exit 1
    fi
    
    echo "Migrations completed successfully."
fi

# Deploy binary
echo "Deploying binary..."
cp ./target/release/navius /usr/local/bin/navius
chmod +x /usr/local/bin/navius

# Create systemd service
echo "Creating systemd service..."
cat > /etc/systemd/system/navius.service << EOF
[Unit]
Description=Navius API Server
After=network.target

[Service]
User=navius
Group=navius
ExecStart=/usr/local/bin/navius
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=CONFIG_PATH=/etc/navius/config

# Security settings
NoNewPrivileges=true
ProtectSystem=full
PrivateTmp=true
ProtectHome=true

# Logging
StandardOutput=append:/var/log/navius/stdout.log
StandardError=append:/var/log/navius/stderr.log

[Install]
WantedBy=multi-user.target
EOF

# Enable service
echo "Enabling service..."
systemctl enable navius.service

# Start service
echo "Starting service..."
systemctl restart navius.service

# Service status check
echo "Checking service status..."
if ! systemctl is-active --quiet navius.service; then
    echo "Service failed to start!"
    systemctl status navius.service
    exit 1
fi

# Configure monitoring integration if enabled
if [ "$ENABLE_MONITORING" = "true" ]; then
    echo "Setting up monitoring integrations..."
    
    # Check for Prometheus Node Exporter
    if command -v node_exporter &> /dev/null; then
        echo "Ensuring Prometheus Node Exporter is running..."
        systemctl enable --now node_exporter
    else
        echo "Warning: Prometheus Node Exporter not found. Metrics collection may be limited."
    fi
    
    # Add log rotation
    cat > /etc/logrotate.d/navius << EOF
/var/log/navius/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 navius navius
    postrotate
    systemctl kill -s HUP navius.service
    endscript
}
EOF
fi

echo "==================================================="
echo "Deployment completed successfully!"
echo "Service is running on port $SERVER_PORT"
echo "==================================================="

# Clean up any old backups after successful deployment
if [ -f "/usr/local/bin/navius.bak" ] && [ "$DEPLOY_SUCCESS" = "true" ]; then
    rm /usr/local/bin/navius.bak
fi

exit 0 
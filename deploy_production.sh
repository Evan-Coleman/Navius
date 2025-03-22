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
if [ ! -f "./target/release/rust-backend" ]; then
    echo "Error: Release binary not found at ./target/release/rust-backend"
    echo "Please build the application first with: cargo build --release"
    exit 1
fi

# Export environment variables
export CONFIG_DIR="$CONFIG_DIR"
export RUN_ENV="$RUN_ENV"
export SERVER_PORT="$SERVER_PORT"
export RUST_LOG="${RUST_LOG:-info}"

# Ensure log directory exists
mkdir -p /var/log/rust-backend

# Create a backup of the current version (for rollback)
if [ -f "/usr/local/bin/rust-backend" ]; then
    echo "Creating backup of current version..."
    cp /usr/local/bin/rust-backend /usr/local/bin/rust-backend.bak
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
    timeout $MIGRATION_TIMEOUT sqlx migrate run
    
    if [ $? -ne 0 ]; then
        echo "Error: Database migrations failed."
        if [ "$ROLLBACK_ON_FAILURE" = "true" ]; then
            echo "Rolling back to previous version..."
            if [ -f "/usr/local/bin/rust-backend.bak" ]; then
                mv /usr/local/bin/rust-backend.bak /usr/local/bin/rust-backend
                echo "Rollback completed. Previous version restored."
            else
                echo "Warning: No backup found for rollback."
            fi
        fi
        exit 1
    fi
    
    echo "Migrations completed successfully."
fi

# Copy the new binary to the production location
echo "Installing new version..."
cp ./target/release/rust-backend /usr/local/bin/rust-backend
chmod +x /usr/local/bin/rust-backend

# Create or update systemd service file
echo "Configuring systemd service..."
cat > /etc/systemd/system/rust-backend.service << EOF
[Unit]
Description=Rust Backend API Service
After=network.target postgresql.service

[Service]
Type=simple
User=rust-backend
Group=rust-backend
ExecStart=/usr/local/bin/rust-backend
Restart=on-failure
RestartSec=5
Environment=CONFIG_DIR=$CONFIG_DIR
Environment=RUN_ENV=$RUN_ENV
Environment=SERVER_PORT=$SERVER_PORT
Environment=RUST_LOG=$RUST_LOG

# Security measures
PrivateTmp=true
ProtectSystem=full
NoNewPrivileges=true

# Logging
StandardOutput=append:/var/log/rust-backend/stdout.log
StandardError=append:/var/log/rust-backend/stderr.log

# Resource limits
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd to pick up the new service file
systemctl daemon-reload

# Enable service to start on boot
systemctl enable rust-backend.service

# Restart the service
echo "Restarting service..."
systemctl restart rust-backend.service

# Wait for service to start and verify it's running
echo "Waiting for service to start (max ${MAX_STARTUP_TIME}s)..."
for i in $(seq 1 $MAX_STARTUP_TIME); do
    if curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
        echo "Service is up and running (verified in ${i}s)"
        DEPLOY_SUCCESS=true
        break
    fi
    
    # Check if service is still running
    if ! systemctl is-active --quiet rust-backend.service; then
        echo "Error: Service failed to start"
        systemctl status rust-backend.service
        DEPLOY_SUCCESS=false
        break
    fi
    
    sleep 1
    
    if [ $i -eq $MAX_STARTUP_TIME ]; then
        echo "Error: Service health check timed out after ${MAX_STARTUP_TIME}s"
        DEPLOY_SUCCESS=false
    fi
done

# Handle deployment failure if needed
if [ "$DEPLOY_SUCCESS" != "true" ]; then
    echo "Deployment failed!"
    
    if [ "$ROLLBACK_ON_FAILURE" = "true" ]; then
        echo "Rolling back to previous version..."
        if [ -f "/usr/local/bin/rust-backend.bak" ]; then
            mv /usr/local/bin/rust-backend.bak /usr/local/bin/rust-backend
            systemctl restart rust-backend.service
            echo "Rollback completed. Previous version restored."
        else
            echo "Warning: No backup found for rollback."
        fi
    fi
    
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
    cat > /etc/logrotate.d/rust-backend << EOF
/var/log/rust-backend/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 rust-backend rust-backend
    postrotate
        systemctl kill -s HUP rust-backend.service
    endscript
}
EOF
fi

echo "==================================================="
echo "Deployment completed successfully!"
echo "Service is running on port $SERVER_PORT"
echo "==================================================="

# Clean up any old backups after successful deployment
if [ -f "/usr/local/bin/rust-backend.bak" ] && [ "$DEPLOY_SUCCESS" = "true" ]; then
    rm /usr/local/bin/rust-backend.bak
fi

exit 0 
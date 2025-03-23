---
title: Navius Installation Guide
description: Step-by-step instructions for installing the Navius framework
category: getting-started
tags:
  - installation
  - setup
  - prerequisites
related:
  - development-setup.md
  - ../guides/development/development-workflow.md
last_updated: March 23, 2025
version: 1.0
---

# Navius Installation Guide

## Overview
This guide provides step-by-step instructions for installing the Navius framework on your local development environment. It covers prerequisites, installation steps, and verification procedures.

## Prerequisites
Before installing Navius, ensure you have the following:

- **Rust** (1.70.0 or later)
  - Check version with `rustc --version`
  - Install from [rust-lang.org](https://www.rust-lang.org/tools/install)
- **Cargo** (included with Rust)
  - Check version with `cargo --version`
- **Git** (2.30.0 or later)
  - Check version with `git --version`
  - Install from [git-scm.com](https://git-scm.com/downloads)
- **Database**
  - PostgreSQL 14 or later (recommended)
  - Docker (for containerized database)

## Step-by-step Installation

### 1. Clone the Repository

```bash
git clone https://github.com/your-organization/navius.git
cd navius
```

### 2. Configure Environment Variables

Create a `.env` file in the project root directory:

```bash
cp .env.example .env
```

Edit the `.env` file with your configuration:

```
DATABASE_URL=postgres://username:password@localhost:5432/navius
JWT_SECRET=your_secret_key
RUST_LOG=info
```

### 3. Install Dependencies

Install all required dependencies using Cargo:

```bash
cargo build
```

This will download and compile all dependencies specified in the `Cargo.toml` file.

### 4. Set Up the Database

If using PostgreSQL directly:

```bash
psql -c "CREATE DATABASE navius;"
psql -c "CREATE USER navius_user WITH ENCRYPTED PASSWORD 'your_password';"
psql -c "GRANT ALL PRIVILEGES ON DATABASE navius TO navius_user;"
```

If using Docker:

```bash
docker run --name navius-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=navius -p 5432:5432 -d postgres:14
```

### 5. Run Migrations

Initialize the database schema:

```bash
cargo run --bin migration
```

## Verification

To verify that Navius has been installed correctly:

1. Start the application in development mode:

```bash
./run_dev.sh
```

2. Open your browser and navigate to:

```
http://localhost:3000/actuator/health
```

You should see a health check response indicating the application is running.

## Troubleshooting

### Common Issues

- **Compiler errors**: Ensure you have the correct Rust version (`rustc --version`)
- **Database connection errors**: Check your `.env` file and database credentials
- **Port conflicts**: Ensure port 3000 is not in use by another application

### Database Connection Issues

If you encounter database connection problems:

1. Verify PostgreSQL is running:
   ```bash
   pg_isready -h localhost -p 5432
   ```

2. Check connection credentials in `.env` file

3. Ensure the database exists:
   ```bash
   psql -l | grep navius
   ```

## Related Documents

- [Development Setup](development-setup.md) - Next steps after installation
- [Development Workflow](../guides/development/development-workflow.md) - Understanding the development process
- [Database Access](../guides/features/database-access.md) - Working with the database 
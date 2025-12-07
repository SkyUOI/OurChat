# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OurChat is a cross-platform chat application built with Rust (server) and Flutter (client). The server uses a modern async architecture with gRPC and HTTP APIs, supporting real-time messaging, group chats, end-to-end encryption, and self-hosting capabilities.

## Repository Structure

### Core Components
- `server/` – Rust backend (multi-crate workspace with gRPC/HTTP APIs, database, messaging queues)
- `client/` – Flutter frontend (cross-platform desktop/mobile/web)
- `service/` – Protobuf definitions shared between server and client

### Deployment & Configuration
- `docker/` – Docker Compose setup for server deployment
- `config/` – Configuration files for non-Docker deployment (load balancer, etc.)
- `files_storage/` – User file storage
- `log/` – Application logs

### Development Utilities
- `script/` – Python utility scripts (protobuf generation, database migrations, etc.)
- `stress_test/` – Stress testing configuration and tools
- `stress_config/` – Additional stress test configurations
- `local/` – Local development files

### Resources
- `resource/` – Project resources (logos, icons)
- `screenshots/` – Application screenshots

### Workspace Structure (Server)
The server is a Rust workspace with multiple crates:
- `server/` – Main server application
- `server/entities/` – SeaORM database entities
- `server/migration/` – Database migrations
- `server/pb/` – Protobuf code generation
- `server/derive/` – Custom derive macros
- `server/base/` – Base library
- `server/load_balancer/` – Load balancing components
- `server/client/` – Client library
- `server/utils/` – Utility functions
- `server/stress_test/` – Server stress testing
- `server/web-panel/` – Web administration panel

## Cross-Cutting Scripts

### Essential Development Scripts
- **`python script/generate.pb.dart.py`** – Generates Dart gRPC code from `.proto` files (run when protobuf definitions change)
- **`python script/db_migration.py`** – Runs SeaORM database migrations
- **`python script/regenerate_entity.py`** – Updates Rust entity code after database schema changes
- **`python script/generate_grpc_web.py`** – Generates gRPC-Web client code
- **`dart run build_runner build --delete-conflicting-outputs`** – Generates Drift database code for Flutter client

### Build & Deployment Scripts
- **`python script/build_production_container.py`** – Builds production Docker containers
- **`python script/rebuild_dev_environment.py`** – Rebuilds development environment
- **`python script/stress_test.py`** – Runs stress tests

### Code Quality & Maintenance
- **`python script/generate_about_code.py`** – Generates about page code
- **`python script/pre-commit.py`** – Pre-commit hook script
- **`python script/init_valgrind_rust.py`** – Initializes Valgrind for Rust memory checking

### Database Workflow
1. **Create migration**: `sea migrate generate xxx` in `server/` directory
2. **Run migration**: `python scripts/db_migration.py`
3. **Update entities**: `scripts/regenerate_entities.py`
4. **Full example**: `sea migrate generate xxx && python scripts/db_migration.py down -n 100 && python scripts/regenerate_entities.py`

Refer to the subdirectory-specific CLAUDE.md files for detailed guidance:

- server/CLAUDE.md – Rust toolchain, build commands, architecture, configuration, testing, database operations, deployment
- client/CLAUDE.md – Flutter development, build commands, architecture, state management, local database, gRPC integration, internationalization

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

### Resources

- `resource/` – Project resources (logos, icons)

## Cross-Cutting Scripts

### Essential Development Scripts

- **`python script/generate.pb.dart.py`** – Generates Dart gRPC code from `.proto` files for client (run when protobuf definitions change)
- **`python script/db_migration.py`** – Runs SeaORM database migrations
- **`python script/regenerate_entity.py`** – Updates Rust entity code after database schema changes
- **`python script/generate_grpc_web.py`** – Generates gRPC-Web client code from `.proto` files
- **`dart run build_runner build --delete-conflicting-outputs`** – Generates Drift database code and other codes for Flutter client

Refer to the subdirectory-specific CLAUDE.md files for detailed guidance:

- server/CLAUDE.md – Rust toolchain, build commands, architecture, configuration, testing, database operations, deployment
- client/CLAUDE.md – Flutter development, build commands, architecture, state management, local database, gRPC integration, internationalization

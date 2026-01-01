# CLAUDE.md - Server

This file provides guidance to Claude Code (claude.ai/code) when working with the **server** portion of this repository.

## Project Overview

OurChat server is a cross-platform chat application backend built with Rust. It uses a modern async architecture with gRPC and HTTP APIs, supporting real-time messaging, group chats, end-to-end encryption, and self-hosting capabilities.

## Quick Reference

### Most Common Commands

```bash
# Build and run server
cargo build
cargo run --bin server

# Run tests
cargo test

# Run migrations
python scripts/db_migration.py
python scripts/regenerate_entities.py

# Docker deployment
cd docker && docker compose up -d
```

### Critical Notes

- Uses **nightly Rust** toolchain (see `rust-toolchain.toml`)
- **SeaORM** for database operations, **Axum** for HTTP, **Tonic** for gRPC
- Configuration files in `docker/config/`
- See sections below for detailed guidance

## Development Environment

### Rust Toolchain

- Uses **nightly** Rust toolchain (`rust-toolchain.toml`)
- Rust edition 2024
- Minimum Rust version: 1.91

### Build Commands

**Server Development:**

```bash
# Build server
cargo build

# Run server
cargo run --bin server

# Run tests
cargo test

# Build with optimizations
cargo build --release
```

**Database Migrations:**

```bash
# Run migrations
python scripts/db_migration.py
python scripts/regenerate_entities.py
```

### Workspace Structure

The server is a Rust workspace with multiple crates:

- **`server/`** - Main server application
- **`server/entities/`** - SeaORM database entities
- **`server/migration/`** - Database migrations
- **`server/pb/`** - Protobuf code generation
- **`server/derive/`** - Custom derive macros
- **`server/base/`** - Base library
- **`server/load_balancer/`** - Load balancing components
- **`server/client/`** - Client library
- **`server/utils/`** - Utility functions
- **`server/stress_test/`** - Server stress testing
- **`server/web-panel/`** - Web administration panel

**Key source directories in `server/src/`:**

- `process/` - Business logic (authentication, messaging, sessions)
- `db/` - Database operations
- `matrix/` - Matrix protocol integration
- `network/` - Network utilities

## Architecture Overview

### Server Structure

- **Main entry**: `server/src/main.rs` - Application bootstrap
- **Core**: `server/src/lib.rs` - Application struct and shared state
- **HTTP API**: `server/src/httpserver.rs` - Axum-based REST endpoints
- **gRPC API**: `server/src/server.rs` - Tonic-based gRPC services
- **Business Logic**: `server/src/process/` - Authentication, messaging, etc.

### Database Layer

- **PostgreSQL**: Primary data storage with SeaORM
- **Redis**: Caching and session management
- **RabbitMQ**: Real-time message queue
- **Entities**: `server/entities/src/entities/` - Database models
- **Migrations**: `server/migration/src/` - Database schema management

### API Architecture

- **gRPC Services**: Core chat functionality via Tonic
- **HTTP Endpoints**: REST API under `/v1/` with Axum
- **Authentication**: JWT tokens with Argon2 password hashing
- **OAuth**: GitHub OAuth integration

## Key Configuration

### Configuration Files

- **Main config**: `docker/config/ourchat.toml`
- **Database**: `docker/config/database.toml`
- **Redis**: `docker/config/redis.toml`
- **RabbitMQ**: `docker/config/rabbitmq.toml`

### Configuration System

- Hierarchical config with environment variable fallback
- Multiple file support with merging capability
- Located in `server/src/config.rs`

## Development Workflow

### Testing

- Unit tests throughout the codebase
- Integration tests in `server/tests/` directory
- Use `cargo test` to run all tests
- See `.clinerules/testing.md` for comprehensive testing documentation

#### Test Categories

- **Server Tests**: Core functionality, authentication, messaging, sessions
- **HTTP Tests**: REST API endpoints, TLS, rate limiting
- **Matrix Tests**: Matrix protocol integration
- **Database Tests**: Migration and data integrity
- **RabbitMQ Tests**: Message queue reliability

#### Test Framework

- Uses **Tokio** for async testing
- **Claims** library for enhanced assertions
- **TestApp** wrapper for application lifecycle
- Custom test helpers for user and session management

### Code Quality

- Uses `rustfmt.toml` for code formatting
- Typo checking with `_typos.toml`
- Structured logging with `tracing` crate

### Database Operations

- Entity definitions in `server/entities/src/entities/`
- Migrations managed via SeaORM
- Use `cargo run --bin migration` to apply migrations

## Common Development Tasks

### Adding New API Endpoints

1. Add gRPC service definition in `server/pb/`
2. Implement service in `server/src/process/`
3. Add HTTP route in `server/src/httpserver.rs` if needed

### Database Schema Changes

1. Create migration with `sea migrate generate xxx` in `server` dir
2. please run migration with `scripts/db_migration.py`.
3. Update entity models with `scripts/regenerate_entities.py` after migration has been run.
4. For example, run `sea migrate generate xxx` and then run `python scripts/db_migration.py down -n 100` and then run `python scripts/regenerate_entities.py`. this is a correct flow of new migration creation

### Setting Logs

1. Use OURCHAT_LOG instead of RUST_LOG, for example, OURCHAT_LOG=trace

### Authentication Flow

- JWT tokens with 5-day expiration
- Password hashing with Argon2
- OAuth support via GitHub
- Session management with Redis

## Important Notes

- The project uses **nightly Rust** features
- **SeaORM** is used for database operations
- **Axum** for HTTP routing and **Tonic** for gRPC
- **RabbitMQ** handles real-time message delivery
- **Redis** manages sessions and caching
- Configuration supports both single instance and distributed deployment
- Rate limiting is implemented with `tower-governor`
- CORS is configured for web client support
- Don't use mod.rs

## Troubleshooting

### Common Issues

**Database migration failures:**

- Check `docker/config/database.toml` credentials
- Run `python scripts/db_migration.py down -n 100` to rollback migrations, then re-apply

**Server fails to start:**

- Check `log/` directory for error logs
- Verify Redis and RabbitMQ are running
- Ensure configuration files exist in `docker/config/` or `config/`

**gRPC connection issues:**

- Verify server is running: `cargo run --bin server`
- Check TLS configuration if using secure connections

**Test failures:**

- Tests require running PostgreSQL, Redis, and RabbitMQ instances
- Check `server/tests/` directory for test-specific configuration

### Performance Tuning

**Database optimization:**

- Configure connection pool size in `database.toml`
- Enable query logging for slow queries
- Consider adding indexes for frequently queried columns

**Memory management:**

- Monitor Redis memory usage
- Configure RabbitMQ message TTL
- Set appropriate file storage limits in `ourchat.toml`

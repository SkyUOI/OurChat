# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OurChat is a cross-platform chat application built with Rust (server) and Flutter (client). The server uses a modern async architecture with gRPC and HTTP APIs, supporting real-time messaging, group chats, end-to-end encryption, and self-hosting capabilities.

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

**Docker Deployment:**
```bash
cd docker
docker compose up -d
```

**Database Migrations:**
```bash
# Run migrations
cargo run --bin migration
```

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
1. Create migration in `server/migration/src/migrations/`
2. Update entity models in `server/entities/src/entities/`
3. Run migration with `cargo run --bin migration`

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

## Deployment

- Docker Compose setup in `docker/` directory
- Supports both development and production configurations
- Multi-instance deployment with leader election
- TLS/SSL support available
- Log rotation and cleanup configured
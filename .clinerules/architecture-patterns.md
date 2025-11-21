# Architecture Patterns

## Server Architecture

### Application Structure
- **Main entry**: `server/src/main.rs` - Uses `tokio::main` with `mimalloc::MiMalloc` global allocator
- **Application struct**: Manages entire server lifecycle with `run_forever()` method
- **SharedData**: Global state with configuration, verification records, and maintenance mode

### Module Organization
- **Core modules**: `config`, `db`, `httpserver`, `process`, `rabbitmq`, `webrtc`
- **Database layer**: Separate modules for user, friend, messages, session, file storage
- **Process layer**: Business logic organized by functionality (auth, friends, messages, etc.)

### Configuration System
- Hierarchical configuration with multiple file support
- Environment variable fallback via `CONFIG_FILE_ENV_VAR`
- Path conversion utilities for relative-to-absolute path resolution

## Code Organization Patterns

### File Structure
- Each major functionality gets its own module file
- Shared utilities in `helper` modules
- Error handling centralized in `error_msg` modules

### Database Layer
- SeaORM entities in `server/entities/src/entities/`
- Database operations in `server/src/db/` modules
- Connection pooling with `DbPool` struct

### API Layer Separation
- gRPC services in `server/src/server.rs`
- HTTP endpoints in `server/src/httpserver.rs`
- Business logic in `server/src/process/` modules
# CLAUDE.md - Server

This file provides guidance to Claude Code (claude.ai/code) when working with the **server** portion of this repository.

## Project Overview

OurChat server is a cross-platform chat application backend built with Rust. It uses a modern async architecture with gRPC and HTTP APIs, supporting real-time messaging, group chats, end-to-end encryption, and self-hosting capabilities.

## Quick Reference

### Database Workflow

1. **Create migration**: `sea migrate generate xxx` in `server/` directory
2. **Run migration**: `python scripts/db_migration.py`
3. **Update entities**: `scripts/regenerate_entities.py`
4. **Full example**: `sea migrate generate xxx && python scripts/db_migration.py down -n 100 && python scripts/regenerate_entities.py`

### Critical Notes

- **SeaORM** for database operations, **Axum** for HTTP, **Tonic** for gRPC
- Configuration files in `docker/config/` (Docker) or `config/` (non-Docker), see "Configuration Files" section

## Development Environment

### Rust Toolchain

- Uses **nightly** Rust toolchain (`rust-toolchain.toml`)
- Rust edition 2024
- Minimum Rust version: 1.91

### Build Commands

### Workspace Structure

The server is a Rust workspace with multiple crates:

- **`server/`** - Main server application
- **`server/entities/`** - SeaORM database entities
- **`server/migration/`** - Database migrations
- **`server/pb/`** - Protobuf code generation
- **`server/derive/`** - Custom derive macros
- **`server/base/`** - Base library
- **`server/client/`** - Client library
- **`server/utils/`** - Utility functions
- **`server/stress_test/`** - Server stress testing
- **`server/web-panel/`** - Web administration panel

**Key source directories in `server/src/`:**

- `process/` - Business logic (authentication, messaging, sessions)
- `db/` - Database operations
- `matrix/` - Matrix protocol integration

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

Configuration files are located in two main directories:

- **Docker deployment**: `docker/config/` (primary configs for containerized deployment)
- **Non-Docker deployment**: `config/` (configs for non-containerized deployment)

**Main configuration files:**

- `ourchat.toml` - Main server configuration
- `database.toml` - PostgreSQL database configuration
- `redis.toml` - Redis configuration
- `rabbitmq.toml` - RabbitMQ configuration
- `user_setting.toml` - User settings and policies
- `http.toml` - HTTP server configuration
- `email.toml` - Email configuration (optional)

### Configuration System

- Hierarchical config with environment variable fallback (`OURCHAT_CONFIG_FILE`)
- Multiple file support with merging capability
- Configuration inheritance via `inherit` field
- Located in `server/src/config.rs`
- Uses the `config` crate for TOML parsing
- Default values defined in `server/base/src/constants.rs`
- Environment variable: `OURCHAT_LOG` for log level (instead of `RUST_LOG`)

**Key structs:**

- `MainCfg` - Primary configuration struct with all server settings (`server/src/config.rs`)
- `RawMainCfg` - Raw deserialization struct with serde attributes
- `Cfg` - Aggregated configuration containing all sub-configs
- `HttpCfg` - HTTP server configuration (`server/src/config/http.rs`)

**Loading order:**

1. Default values from constants (`server/base/src/constants.rs`)
2. Config file values (with inheritance via `inherit` field)
3. Environment variables (limited)
4. Command line arguments (for specific overrides)

### Adding New Configuration Entries

Follow this pattern to add new configuration entries:

**Step 1: Add default constant** (`server/base/src/constants.rs`)

```rust
pub const fn default_enable_metrics() -> bool {
    true
}

pub const fn default_metrics_snapshot_interval() -> Duration {
    Duration::from_mins(1)
}
```

**Step 2: Add field to `MainCfg` struct** (`server/src/config.rs`)

```rust
pub struct MainCfg {
    // ... existing fields
    pub enable_metrics: bool,
    pub metrics_snapshot_interval: Duration,
    // ... more fields
}
```

**Step 3: Add field to `RawMainCfg` with serde attributes** (`server/src/config.rs`)

```rust
pub struct RawMainCfg {
    // ... existing fields
    #[serde(default = "constants::default_enable_metrics")]
    pub enable_metrics: bool,
    #[serde(
        default = "constants::default_metrics_snapshot_interval",
        with = "humantime_serde"
    )]
    pub metrics_snapshot_interval: Duration,
    // ... more fields
}
```

**Step 4: Update `Deserialize` implementation** (`server/src/config.rs`)
Update the `impl<'de> Deserialize<'de> for MainCfg` to map from `RawMainCfg` to `MainCfg`:

```rust
impl<'de> Deserialize<'de> for MainCfg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawMainCfg::deserialize(deserializer)?;
        // ... validation
        Ok(MainCfg {
            // ... existing mappings
            enable_metrics: raw.enable_metrics,
            metrics_snapshot_interval: raw.metrics_snapshot_interval,
            // ... more mappings
        })
    }
}
```

**Step 5: Add validation if needed**
Add validation logic in the `Deserialize` implementation for the new field:

```rust
if raw.metrics_snapshot_interval.is_zero() {
    return Err(D::Error::custom("metrics_snapshot_interval cannot be zero"));
}
```

**Step 6: Add to config file** (`docker/config/ourchat.toml`)

```toml
# If record the metrics of the server, which will be used for monitoring and debugging
enable_metrics = true
metrics_snapshot_interval = "1m"
```

**Notes:**

- Use `#[serde(with = "humantime_serde")]` for `Duration` fields
- Use `#[serde(default = "constants::default_*")]` to reference default functions
- Use `size::Size` for file size fields with human-readable string parsing
- Add comprehensive validation in the `Deserialize` implementation

## Development Workflow

### Testing

The server uses a comprehensive test architecture with both unit and integration tests.

#### Test Directory Structure

**Main test directories:**

- `server/tests/` - Primary test directory
  - `server/` - Integration tests for server functionality
    - `basic_services/` - Database and RabbitMQ tests
    - `session/` - Session management tests
    - `friend/` - Friend management tests
    - `server_manage/` - Server management tests (including metrics)
    - `voip/` - VoIP functionality tests
    - `webrtc/` - WebRTC tests
  - `http_test/` - HTTP API tests
  - `matrix_test/` - Matrix protocol integration tests

**Test data directory:**

- `server/test_data/` - Test certificates and files
  - `certs/` - TLS certificates for testing
  - `private/` - Private keys
  - Test files like `test_avatar.png`

#### Test Organization

**Integration Tests:**

- Located in `server/tests/`
- Use `TestApp` helper to launch full server instances
- Test real server functionality with databases and message queues
- Organized by feature areas (auth, session, friend, etc.)

#### Test Helpers and Common Patterns

**Key Test Helpers:**

- `TestApp` (`server/client/src/oc_helper/client.rs`) - Manages server lifecycle, creates isolated databases and RabbitMQ vhosts
- `TestUser` (`server/client/src/oc_helper/user.rs`) - Represents authenticated test users with methods for operations

**Common Test Pattern:**

```rust
#[tokio::test]
async fn test_name() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    // Test logic
    app.async_drop().await;
}
```

**Custom Configuration Pattern:**

```rust
let (mut config, args) = TestApp::get_test_config().unwrap();
config.main_cfg.some_setting = some_value;
let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
    .await
    .unwrap();
```

#### Adding New Tests

**Adding Integration Tests:**

1. **Create new test file** in appropriate subdirectory under `server/tests/server/`
2. **Import dependencies**: `use client::TestApp;`, `use claims::assert_*;`
3. **Write test function** with `#[tokio::test]` attribute
4. **Use `TestApp`** to launch server instance
5. **Create test users** with `app.new_user().await.unwrap()`
6. **Perform operations** using user methods
7. **Assert results** using `claims` macros or standard assertions
8. **Clean up** with `app.async_drop().await`

**Example authentication test:**

```rust
#[tokio::test]
async fn auth_token() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.ocid_auth().await);
    app.async_drop().await;
}
```

#### Test Configuration and Setup

**Test Dependencies (from Cargo.toml):**

- `client` - Test client crate with `TestApp` and `TestUser`
- `tempfile` - Temporary file management

**Test Execution:**

- Each test launches its own server instance on random port
- Database migrations run automatically for each test
- Cleanup happens via `async_drop()` method

#### Running Tests

#### Test Utilities and Macros

**Assertion Macros (from `claims` crate):**

- `assert_ok!` - Assert result is Ok
- `assert_err!` - Assert result is Err
- `assert_ge!`, `assert_gt!`, `assert_le!`, `assert_lt!` - Comparison assertions
- `assert_some!`, `assert_none!` - Option assertions

**Test Helper Functions:**

- `TestApp::new_session_db_level()` - Create test session with users
- `TestUser::random_readable()` - Create user with readable names
- `TestUser::random_unreadable()` - Create user with random strings
- Various assertion helpers in test files

### Code Quality

- Structured logging with `tracing` crate

### Database Operations

- Entity definitions in `server/entities/src/entities/`
- Migrations managed via SeaORM

## Common Development Tasks

### Adding New API Endpoints

1. Add gRPC service definition in `service/*.proto`
2. Implement service in `server/src/process/`
3. Add HTTP route in `server/src/httpserver.rs` if needed

### Setting Logs

1. Use OURCHAT_LOG instead of RUST_LOG, for example, OURCHAT_LOG=trace

### Authentication Flow

- JWT tokens with 5-day expiration
- Password hashing with Argon2
- OAuth support via GitHub
- Session management with Redis

## Important Notes

- Rate limiting is implemented with `tower-governor`
- Don't use mod.rs

## Troubleshooting

### Common Issues

**Database migration failures:**

- Check `docker/config/database.toml` credentials
- Run `python scripts/db_migration.py down -n 100` to rollback migrations, then re-apply

**Server fails to start:**

- Check `log/` directory for error logs

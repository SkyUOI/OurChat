# OurChat Server Testing Documentation

## Overview

The OurChat server uses a comprehensive testing strategy with multiple test types organized in a modular structure. Tests are written in Rust using async/await patterns and cover various aspects of the chat application including authentication, messaging, sessions, and HTTP APIs.

## Test Organization

### Test Directory Structure
```
server/tests/
├── server/           # Core server functionality tests
│   ├── basic.rs      # Basic server operations
│   ├── auth_register.rs  # Authentication and registration
│   ├── msg_send.rs   # Message sending and delivery
│   ├── session.rs    # Session management
│   ├── basic_services/ # Database and RabbitMQ tests
│   └── session/      # Session-specific operations
├── http_test/        # HTTP API tests
│   ├── http.rs       # HTTP endpoints and TLS
│   ├── avatar.rs     # Avatar handling
│   └── verify.rs     # Verification endpoints
└── matrix_test/      # Matrix protocol integration tests
```

## Testing Frameworks and Tools

### Core Testing Dependencies
- **Tokio**: Async runtime for async tests
- **Claims**: Enhanced assertion library for better error messages
- **TestApp**: Custom test application wrapper
- **SeaORM**: Database testing with migrations
- **Deadpool**: Connection pooling for Redis and RabbitMQ

### Test Categories

#### 1. Server Functionality Tests (`server/`)
- **Basic Operations**: Server info, timestamps, support info
- **Authentication**: User registration, login, token validation
- **Messaging**: Text message sending, delivery, recall
- **Sessions**: Session creation, management, permissions
- **Database**: Migration testing, session relations
- **RabbitMQ**: Message exchange testing

#### 2. HTTP API Tests (`http_test/`)
- **HTTP Endpoints**: Status checks, rate limiting
- **TLS**: HTTPS configuration and certificate handling
- **File Operations**: Avatar and logo serving
- **Rate Limiting**: Request throttling validation

#### 3. Matrix Protocol Tests (`matrix_test/`)
- **Matrix Integration**: Protocol compatibility
- **Helper Functions**: Test configuration setup

## Test Patterns and Conventions

### Test Structure
```rust
#[tokio::test]
async fn test_name() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    // Test setup
    // Test execution
    // Assertions
    app.async_drop().await;
}
```

### Common Test Patterns

#### 1. TestApp Lifecycle
- **Setup**: `TestApp::new_with_launching_instance()`
- **Execution**: Use app methods and services
- **Cleanup**: `app.async_drop().await`

#### 2. User Management
- **Create Users**: `app.new_user().await`
- **Authentication**: `user.lock().await.email_auth().await`
- **Registration**: `user.lock().await.register().await`

#### 3. Session Testing
- **Create Sessions**: `app.new_session_db_level()`
- **Message Sending**: `user.lock().await.send_msg()`
- **Message Fetching**: `user.lock().await.fetch_msgs()`

#### 4. Error Testing
- **Expected Errors**: Use `assert_err!` and error code checking
- **Error Messages**: Validate specific error messages
- **Status Codes**: Check gRPC and HTTP status codes

### Assertion Patterns

#### Claims Library Usage
```rust
use claims::{assert_ok, assert_err, assert_lt, assert_eq};

assert_ok!(result);        // Assert result is Ok
assert_err!(result);       // Assert result is Err
assert_lt!(value1, value2); // Assert value1 < value2
assert_eq!(actual, expected); // Assert equality
```

#### Error Validation
```rust
let err = result.unwrap_err().unwrap_rpc_status();
assert_eq!(err.code(), tonic::Code::NotFound);
assert_eq!(err.message(), error_msg::not_found::USER);
```

### Configuration Testing

#### Custom Configuration
```rust
let (mut config, args) = TestApp::get_test_config().unwrap();
config.main_cfg.unregister_policy = server::config::UnregisterPolicy::Delete;
let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {}).await.unwrap();
```

## Running Tests

### Basic Test Commands
```bash
# Run all tests
cargo test

# Run specific test module
cargo test server::auth_register

# Run with verbose output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration
```

### Test Configuration
- Tests use isolated databases with unique names
- RabbitMQ exchanges are created and cleaned up
- Redis connections are pooled and managed
- HTTP servers run on ephemeral ports

## Test Data Management

### Database
- Each test creates its own database instance
- Migrations are applied and rolled back
- Test data is cleaned up automatically

### Message Queues
- RabbitMQ exchanges are tested for reliability
- Message delivery is validated end-to-end
- Connection recovery is tested

### File System
- Temporary files are used for TLS certificates
- Avatar and logo files are served via HTTP
- File cleanup is handled automatically

## Best Practices

### Writing New Tests
1. Use `#[tokio::test]` for async tests
2. Always clean up with `app.async_drop().await`
3. Use descriptive test names
4. Test both success and error cases
5. Validate error messages and codes

### Test Organization
1. Group related tests in modules
2. Use helper functions for common assertions
3. Share setup code between similar tests
4. Document test purpose and expected behavior

### Performance Considerations
1. Use timeouts for async operations
2. Avoid unnecessary sleeps
3. Clean up resources promptly
4. Use connection pooling efficiently

## Common Test Scenarios

### Authentication Flow
1. Register user
2. Authenticate with credentials
3. Validate token
4. Test unregister/delete policies

### Messaging Flow
1. Create session with multiple users
2. Send messages
3. Validate delivery to all participants
4. Test message recall

### Session Management
1. Create session
2. Invite users
3. Test permissions and roles
4. Validate session info updates

### HTTP API Testing
1. Test endpoints with various HTTP methods
2. Validate response codes and content
3. Test rate limiting
4. Verify TLS configuration

This testing framework provides comprehensive coverage of OurChat server functionality with a focus on reliability, performance, and maintainability.
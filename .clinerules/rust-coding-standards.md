# Rust Coding Standards

## Toolchain & Dependencies
- **Nightly Rust**: Required for features like `decl_macro`, `duration_constructors`
- **Workspace structure**: Multiple crates with shared dependencies
- **Edition 2024**: Latest Rust edition for modern features

## Async Patterns
- **Tokio runtime**: Use `#[tokio::main]` for async main functions
- **Async traits**: Use `async-trait` crate for trait async methods
- **Blocking operations**: Use `helper::spawn_blocking_with_tracing` for CPU-intensive tasks

## Error Handling
- **thiserror**: Define custom error types with `#[derive(thiserror::Error)]`
- **anyhow**: For application-level errors with context
- **Tonic Status**: Convert errors to gRPC status codes
- **Pattern**: Internal error enum + public status conversion

## Code Structure Patterns

### Function Documentation
```rust
/// Function description
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// Result with description
///
/// # Errors
/// When and why errors occur
```

### Module Organization
- **Public API**: Mark with `pub mod`
- **Internal modules**: Keep private with `mod`
- **Prelude**: Use `prelude` modules for common imports

### Static Data
- **LazyLock**: Use for global static initialization
- **Mutex**: For shared mutable state
- **Once**: For one-time initialization

## Performance Patterns
- **mimalloc**: Global allocator for better performance
- **DashMap**: For concurrent hash maps
- **Arc**: For shared ownership across threads
- **Clone-on-write**: Use `Arc` for large shared data
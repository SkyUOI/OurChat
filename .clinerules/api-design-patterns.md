# API Design Patterns

## gRPC Service Patterns

### Service Structure
- **Service providers**: Implement traits like `AuthServiceProvider`
- **Request processing**: Separate internal and external error handling
- **Response conversion**: Convert internal results to gRPC responses

### Error Handling Pattern
```rust
pub async fn api_endpoint(
    server: &ServiceProvider,
    request: tonic::Request<RequestType>,
) -> Result<Response<ResponseType>, Status> {
    match internal_impl(server, request.into_inner()).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            InternalError::SpecificError => Err(Status::specific_error(ERROR_MSG)),
            _ => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}
```

### Authentication Pattern
- **JWT tokens**: 5-day expiration with user ID claims
- **Bearer tokens**: Extract from `Authorization` header
- **Token generation**: Use `generate_access_token(user_id)`

## HTTP API Patterns

### Axum Router Structure
- **Route organization**: Group by functionality
- **Middleware**: CORS, rate limiting, authentication
- **State sharing**: Use `Arc<SharedData>` for global state

### Rate Limiting
- **tower-governor**: For request rate limiting
- **Configurable**: Burst size and replenish duration
- **Background cleanup**: Regular storage cleanup

### CORS Configuration
```rust
let cors = tower_http::cors::CorsLayer::new()
    .allow_origin(tower_http::cors::Any)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    .allow_headers([
        http::header::CONTENT_TYPE,
        http::header::AUTHORIZATION,
        http::HeaderName::from_static("x-requested-with"),
    ]);
```

## Message Processing Patterns

### Real-time Messaging
- **RabbitMQ**: For message queuing and distribution
- **Session-based**: Route messages to user sessions
- **Broadcast**: Support for group messaging

### File Upload/Download
- **File storage**: Database-backed file system
- **Size limits**: Configurable user file storage limits
- **Streaming**: Efficient file transfer

## WebRTC Patterns
- **Room management**: Create and clean up VoIP rooms
- **Session keys**: Time-limited room access keys
- **Member management**: Track room participants
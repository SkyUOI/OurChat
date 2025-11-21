# Security Patterns

## Authentication & Authorization

### Password Security
- **Argon2 hashing**: Use for password storage with configurable parameters
- **Salt generation**: Automatic salt generation in Argon2
- **Password verification**: Use `verify_password_hash` function

```rust
fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let expected = PasswordHash::new(password_hash).context("Not PHC string")?;
    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &expected)
        .context("wrong password")?;
    Ok(())
}
```

### JWT Token Security
- **Token generation**: Use `generate_access_token(user_id)`
- **Expiration**: 5-day token validity
- **Claims**: Include user ID and minimal necessary data

### OAuth Integration
- **GitHub OAuth**: Support for third-party authentication
- **Secure redirects**: Validate redirect URIs
- **Token exchange**: Secure token handling

## Input Validation

### Email Validation
- **email_address crate**: Use for proper email validation
- **Format checking**: Ensure valid email format before storage
- **Uniqueness**: Enforce unique email addresses

### User Input Sanitization
- **Length limits**: Enforce reasonable input length limits
- **Content validation**: Validate file types and sizes
- **SQL injection prevention**: Use parameterized queries

## Rate Limiting & Protection

### Request Rate Limiting
- **tower-governor**: Implement request rate limiting
- **Configurable limits**: Set burst size and replenish rates
- **IP-based**: Limit requests per IP address

### DDoS Protection
- **Connection limits**: Limit concurrent connections
- **Request size limits**: Enforce maximum request sizes
- **Timeout handling**: Proper request timeouts

## Data Protection

### End-to-End Encryption
- **WebRTC encryption**: Secure real-time communication
- **Session keys**: Time-limited encryption keys
- **Key management**: Secure key storage and rotation

### Privacy Features
- **User data minimization**: Store only necessary user data
- **Data retention policies**: Configurable data retention periods
- **User control**: Allow users to manage their data

## Secure Configuration

### TLS/SSL Configuration
- **Rustls**: Use for TLS implementation
- **Certificate management**: Proper certificate handling
- **Secure protocols**: Enforce modern TLS versions

### Secret Management
- **Environment variables**: Use for sensitive configuration
- **File-based secrets**: Secure file storage for secrets
- **No hardcoding**: Avoid hardcoded secrets in code
# Database Patterns

## Entity Design

### SeaORM Entities
- **Generated entities**: Use sea-orm-codegen for database schema
- **Relationships**: Define proper relations between entities
- **Soft delete**: Support for soft deletion where appropriate

### Entity Structure
```rust
// Example entity definition pattern
pub struct User {
    pub id: i32,
    pub email: String,
    pub ocid: String,
    pub passwd: String,
    // ... other fields
}
```

## Database Operations

### Query Patterns
- **EntityTrait**: Use `Entity::find()` for queries
- **Filter chains**: Use `.filter(column.eq(value))` for conditions
- **Eager loading**: Load related entities when needed

### Connection Management
- **Connection pools**: Use `DbPool` for database connections
- **Redis integration**: Separate Redis pool for caching/sessions
- **Transaction support**: Use database transactions for atomic operations

## Migration Patterns

### Migration Structure
- **Incremental migrations**: Each migration builds on previous
- **Version tracking**: Track migration versions for compatibility
- **Rollback support**: Design migrations with rollback capability

### Migration Naming
- **Timestamp-based**: `m20220101_000001_create_table.rs`
- **Descriptive**: Include table/feature name in migration
- **Sequential**: Maintain chronological order

## Caching Patterns

### Redis Usage
- **Session storage**: User sessions and authentication state
- **Cache layer**: Frequently accessed data
- **Message queues**: Temporary message storage

### Cache Key Patterns
- **Namespaced keys**: Use prefixes for different data types
- **TTL support**: Set appropriate expiration times
- **Consistency**: Ensure cache invalidation on data changes

## File Storage

### Database File System
- **Metadata in DB**: Store file metadata in database
- **File content**: Store actual files in filesystem with DB references
- **Size limits**: Enforce user storage limits

### File Operations
- **Upload**: Validate and store files with metadata
- **Download**: Serve files with proper authentication
- **Cleanup**: Remove orphaned files periodically
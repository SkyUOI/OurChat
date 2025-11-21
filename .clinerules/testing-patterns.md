# Testing Patterns

## Test Organization

### Test Structure
- **Unit tests**: Place in same file as code being tested
- **Integration tests**: In `tests/` directory at crate root
- **Test modules**: Use `#[cfg(test)]` for test-only code

### Test Naming
- **Descriptive names**: `test_function_name_scenario`
- **Module organization**: Group related tests together
- **Setup/teardown**: Use test fixtures where needed

## Test Patterns

### Database Testing
- **Test database**: Use separate database for tests
- **Transaction rollback**: Use transactions to isolate tests
- **Test data**: Create realistic test data

### Async Testing
- **Tokio test**: Use `#[tokio::test]` for async tests
- **Timeout handling**: Set appropriate test timeouts
- **Concurrent testing**: Test concurrent scenarios

### Mocking Patterns
- **mockall crate**: Use for dependency mocking
- **Trait-based**: Mock traits rather than concrete types
- **Expectation setup**: Set up mock expectations clearly

## Integration Testing

### API Testing
- **gRPC client testing**: Test service endpoints
- **HTTP endpoint testing**: Test REST API endpoints
- **Authentication testing**: Test auth flows

### End-to-End Testing
- **Full workflow**: Test complete user workflows
- **Multi-user scenarios**: Test concurrent user interactions
- **Error scenarios**: Test error conditions and recovery

## Test Data Patterns

### Test Fixtures
- **Reusable data**: Create reusable test data structures
- **Factory functions**: Use factory functions for complex objects
- **Random data**: Use fake data generation where appropriate

### Cleanup
- **Database cleanup**: Ensure tests clean up after themselves
- **File cleanup**: Remove test files after tests
- **Resource cleanup**: Properly release test resources
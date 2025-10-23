## Architecture

The library is built on:

- **Protocol Buffers** - Efficient data serialization matching AnyList's internal format
- **Tokio** - Async runtime for concurrent operations
- **Reqwest** - HTTP client for API communication
- **Prost** - Protocol buffer implementation

### Module Structure

```
src/
├── client.rs          - Core client with authentication and HTTP methods
├── error.rs           - Error types and Result alias
├── lists.rs           - List CRUD operations
├── items.rs           - Item CRUD operations
├── recipes.rs         - Recipe management
├── collections.rs     - Recipe collection management
├── categories.rs      - Category management
├── stores.rs          - Store management
├── meal_planning.rs   - Meal planning calendar
└── utils.rs          - Utility functions (ID generation, timestamps)
```

### Error Handling

The library uses a custom `AnyListError` enum covering common failure modes:

```rust
pub enum AnyListError {
    AuthenticationFailed(String),
    NetworkError(String),
    InvalidResponse(String),
    NotFound(String),
    PermissionDenied(String),
    ProtobufError(String),
    Other(String),
}
```

All operations return `Result<T, AnyListError>` for consistent error handling.

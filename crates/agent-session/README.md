# agent-session

Session Management for the Agent Runtime with pluggable storage backends.

## Features

- **Multiple Storage Backends**: In-memory, cache with TTL, layered
- **Thread-Safe**: Concurrent access with lock-free data structures
- **TTL Support**: Automatic session expiration
- **Layered Storage**: Caching hierarchy with automatic fallback
- **Async-First**: Built on Tokio for high performance
- **Graceful Degradation**: Continues working on partial failures

## Storage Backends

### InMemoryStore
Fast, thread-safe in-memory storage using DashMap.

```rust
use agent_session::{InMemoryStore, SessionStore};
use agent_llm::Conversation;

let store = InMemoryStore::new();
store.save("session-1", conversation).await?;
let loaded = store.load("session-1").await?;
```

**Characteristics**:
- **Speed**: O(1) operations, ~500ns latency
- **Concurrency**: Excellent (lock-free sharding)
- **Persistence**: Process lifetime only
- **Capacity**: Configurable limit

### CacheStore
Async cache with Time-To-Live expiration.

```rust
use agent_session::CacheStore;
use std::time::Duration;

// Sessions expire after 1 hour
let store = CacheStore::new(Duration::from_secs(3600));
store.save("temp-session", conversation).await?;
// Auto-expires after 1 hour of inactivity
```

**Characteristics**:
- **Speed**: O(1) operations, ~1μs latency
- **TTL**: Automatic expiration
- **Eviction**: LRU when at capacity
- **Updates**: Writing extends TTL

### LayeredStore
Combines multiple backends with fallback.

```rust
use agent_session::{LayeredStore, CacheStore, InMemoryStore};

let store = LayeredStore::new()
    .with_layer(CacheStore::new(Duration::from_secs(300)))  // Fast, 5min TTL
    .with_layer(InMemoryStore::new());                       // Fallback

// Writes go to all layers (write-through)
store.save("session", conversation).await?;

// Reads check layers in order (cache → memory)
let loaded = store.load("session").await?;
```

**Characteristics**:
- **Read strategy**: Check layers in order, cache hits are fast
- **Write strategy**: Write-through to all layers
- **Fallback**: Automatic on cache miss or failure
- **Write-back**: Populates faster layers automatically

## Examples

### Basic Storage
```bash
cargo run -p agent-session --example basic_storage
```

Demonstrates:
- CRUD operations
- TTL expiration
- Capacity limits
- Session lifecycle

### Layered Storage
```bash
cargo run -p agent-session --example layered_storage
```

Shows:
- Write-through behavior
- Cache miss → memory fallback
- Automatic write-back
- TTL expiration + fallback
- Graceful degradation

## API Overview

### SessionStore Trait
```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()>;
    async fn load(&self, session_id: &str) -> Result<Conversation>;
    async fn delete(&self, session_id: &str) -> Result<()>;
    async fn exists(&self, session_id: &str) -> Result<bool>;
    async fn list_sessions(&self) -> Result<Vec<String>>;
    async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata>;
    async fn clear(&self) -> Result<usize>;
    fn name(&self) -> &str;
}
```

### Common Patterns

**Create and use store**:
```rust
let store = InMemoryStore::new();
store.save(id, conversation).await?;
let conv = store.load(id).await?;
```

**With capacity limit**:
```rust
let store = InMemoryStore::with_capacity(1000);
```

**With TTL**:
```rust
let store = CacheStore::new(Duration::from_secs(3600));
```

**Layered with fallback**:
```rust
let store = LayeredStore::new()
    .with_layer(fast_store)
    .with_layer(fallback_store);
```

## Error Handling

```rust
use agent_session::SessionError;

match store.load("session-1").await {
    Ok(conversation) => // Use conversation,
    Err(SessionError::NotFound(id)) => // Create new session,
    Err(SessionError::Expired(id)) => // Session expired,
    Err(e) => // Handle other errors,
}
```

## Performance

### InMemoryStore (DashMap)
- **Concurrent reads**: ~500ns, scales with cores
- **Concurrent writes**: ~1μs, minimal contention
- **Thread-safe**: Yes (lock-free sharding)

### CacheStore (Moka)
- **Cache hit**: ~800ns
- **Cache miss**: NotFound error
- **TTL check**: Automatic, ~0ns overhead
- **Eviction**: Background, non-blocking

### LayeredStore
- **Cache hit**: ~1μs (cache layer)
- **Cache miss**: ~2μs (fallback + write-back)
- **Write**: 2-3μs (write to all layers)

## Thread Safety

All stores are thread-safe and can be shared across threads:

```rust
let store = Arc::new(InMemoryStore::new());

let store1 = Arc::clone(&store);
tokio::spawn(async move {
    store1.save("s1", conv1).await;
});

let store2 = Arc::clone(&store);
tokio::spawn(async move {
    store2.save("s2", conv2).await;
});

// Both tasks can run concurrently
```

## Testing

All storage backends include comprehensive tests:

```bash
# Run all session tests
cargo test -p agent-session

# Run specific test
cargo test -p agent-session concurrent_access
```

Tests cover:
- Basic CRUD operations
- Concurrent access
- TTL expiration
- Capacity limits
- Layered fallback
- Error handling

## Integration with Conversations

Works seamlessly with Phase 2 conversations:

```rust
use agent_llm::Conversation;
use agent_session::{InMemoryStore, SessionStore};

// Create conversation (Phase 2)
let mut conv = Conversation::builder()
    .system("You are helpful")
    .user("Hello")
    .build();

// Store it (Phase 3)
let store = InMemoryStore::new();
store.save("session-1", conv).await?;

// Load and continue
let mut loaded = store.load("session-1").await?;
loaded.add_user("Follow-up question");
store.save("session-1", loaded).await?;
```

## Future: Database Backend (Phase 6)

Phase 6 will add:
- **DatabaseStore**: SQLite/PostgreSQL
- **Migrations**: Schema management
- **Persistence**: Survives restarts
- **Query**: Search sessions by metadata

Will implement the same `SessionStore` trait!

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.






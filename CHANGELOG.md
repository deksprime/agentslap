# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 6 - Coming Soon
- Database Storage (SQLite)
- Persistent sessions across restarts
- Database migrations

## [0.6.0] - 2025-12-01

### Added - Phase 5: Agent Runtime

#### New Crate: agent-runtime
- **Critical**: Separate crate to avoid circular dependencies
- Composition layer sitting above all other crates
- Clean dependency graph (acyclic, no cycles)

#### Agent Struct
- Main agent combining LLM provider, tools, session store
- Builder pattern for flexible construction
- Configuration support (max iterations, session ID, system message)
- Session integration (auto load/save)
- Error handling across all layers

#### AgentBuilder
- Fluent API for agent construction
- Type-safe component injection
- Optional configuration
- Validation on build()

#### Tool Call Parser
- Parse OpenAI function calls from responses
- Parse Anthropic tool uses from responses
- Extract tool name and parameters
- Handle multiple tool calls
- Handle malformed responses
- 5 parser tests

#### Agent Loop Foundation
- Basic run() method implementation
- Load or create conversation from session
- Add user message to conversation
- Call LLM with conversation history
- Save conversation to session after each turn
- Max iterations safeguard

#### Integration
- All 5 phases work together seamlessly
- Agent uses LLM providers from Phase 1
- Agent uses conversations from Phase 2
- Agent uses session storage from Phase 3
- Agent uses tools from Phase 4
- Clean composition, no circular references

#### Testing
- 13 new unit tests (agent builder, parser, integration)
- Total: 119 tests across all phases (100% pass)
- Examples demonstrating full agent

#### Dependencies
- agent-core, agent-llm, agent-session, agent-tools
- **Direction**: All point upward in dependency graph
- **Cycles**: Zero (verified by successful compilation)

#### CLI Updates
- Updated to show all phases complete
- Now depends on agent-runtime
- Ready for Phase 6+

## [0.5.0] - 2025-12-01

### Added - Phase 4: Tool Calling System

#### Tool System (`agent-tools`)
- `Tool` trait for extensible tool definition
- Async tool execution support
- ToolResult type for structured results
- JSON schema generation for parameters
- Thread-safe tool registry using DashMap

#### Tool Registry
- `ToolRegistry` for managing tools
- Thread-safe registration and lookup
- Tool execution with error handling
- OpenAI function calling format generation
- Anthropic tool use format generation
- List and count operations
- Duplicate detection

#### JSON Schema Support
- `ToolSchema` for parameter definitions
- Property helpers (string, number, boolean, enum)
- Required fields support
- Multi-provider format conversion

#### Built-in Tools
- **CalculatorTool**: Arithmetic operations (add, subtract, multiply, divide)
  - Parameter validation
  - Division by zero handling
  - Error results for invalid operations
- **EchoTool**: Text echo for testing
- **CurrentTimeTool**: Returns current date/time in multiple formats

#### Features
- Async tool execution
- Type-safe parameter handling
- Error boundaries (tool failures don't crash agent)
- Extensible tool system (easy to add custom tools)
- Multi-format schema generation

#### Testing
- 30 new unit tests (100% pass)
- Tool execution tests
- Schema generation tests
- Error handling tests
- Total: 106 tests across all phases

#### Examples
- Tool demo (registration, execution, schemas)

#### Dependencies
- `schemars` 0.8 - JSON Schema generation
- `dashmap` 6.1 - Tool registry (reused)
- `chrono` 0.4 - DateTime (reused)

## [0.4.0] - 2025-12-01

### Added - Phase 3: Session Management

#### SessionStore Trait (`agent-session`)
- Async trait for storage abstraction
- CRUD operations (save, load, delete, exists)
- Session listing and metadata retrieval
- Bulk clear operation
- Support for multiple backend implementations

#### InMemoryStore
- Concurrent HashMap using DashMap (lock-free)
- Thread-safe for multi-threaded access
- O(1) lookups and inserts
- Optional capacity limits
- Process-lifetime persistence
- 8 unit tests covering all operations

#### CacheStore with TTL
- Async cache using Moka
- Time-To-Live expiration support
- LRU eviction when at capacity
- Automatic expiration handling
- Write extends TTL (activity-based)
- Background maintenance tasks
- 8 unit tests including TTL verification

#### LayeredStore
- Combines multiple storage backends
- Write-through to all layers
- Read-through with automatic write-back
- Graceful degradation on partial failures
- Session deduplication across layers
- Production-ready caching hierarchy
- 7 unit tests for fallback scenarios

#### Features
- Thread-safe concurrent access
- Automatic session expiration
- Capacity management
- Session metadata without full load
- Bulk operations (clear, list)
- Error handling with detailed types

#### Testing
- 27 new unit tests (100% pass)
- Concurrent access tests
- TTL expiration tests
- Layered fallback tests
- Total: 76 tests across all phases

#### Examples
- Basic storage demo (CRUD operations)
- Layered storage demo (caching hierarchy)

#### Dependencies
- `dashmap` 6.1 - Lock-free concurrent HashMap
- `moka` 0.12 - Async cache with TTL
- `chrono` 0.4 - DateTime for metadata

## [0.3.0] - 2025-12-01

### Added - Phase 2: Message & Conversation Management

#### Conversation Management (`agent-llm`)
- `Conversation` struct for multi-turn interaction tracking
- Unique conversation IDs with UUID v4
- Timestamps for creation and updates
- Flexible metadata support with JSON values

#### Builder Pattern
- `ConversationBuilder` for fluent conversation construction
- Chainable methods for all message types
- Clean, readable API design
- Metadata configuration support

#### Token Management
- Approximate token counting (chars / 4 heuristic)
- Per-message token estimation
- Conversation-wide token counting
- Foundation for accurate tokenization

#### Context Window Management
- Smart truncation to fit token limits
- System message preservation
- Sliding window strategy (keeps recent messages)
- Configurable truncation limits

#### Serialization
- Full JSON serialization/deserialization
- Conversation import/export capabilities
- Preserves all state (messages, metadata, timestamps)
- Compatible with serde ecosystem

#### Utilities
- Conversation summaries with statistics
- Message accessors (first, last, system)
- Collection operations (len, is_empty, clear)
- Message iteration support

#### Testing
- 11 new unit tests (100% pass)
- Cumulative: 49 tests across all phases
- Conversation demo example
- LLM integration example

#### Dependencies
- `chrono` 0.4 - DateTime handling
- `uuid` 1.11 - Unique identifiers

#### Examples
- Conversation demo (offline)
- Conversation with LLM integration
- Multi-turn conversation examples

## [0.2.0] - 2025-12-01

### Added - Phase 1: LLM Provider Abstraction

#### Core Infrastructure (`agent-llm`)
- `LLMProvider` trait for model-agnostic LLM interactions
- Async trait support with `async-trait`
- Provider factory pattern for runtime selection
- Comprehensive error types with retry-ability detection

#### OpenAI Provider
- Full OpenAI API integration
- Support for all chat models (GPT-4, GPT-3.5-turbo, etc.)
- Non-streaming completion
- Streaming completion with Server-Sent Events
- Automatic retry with exponential backoff
- Rate limit handling with retry-after header support
- Configurable timeouts
- Token usage tracking

#### Anthropic Provider  
- Full Anthropic (Claude) API integration
- Support for Claude 3 models (Opus, Sonnet, Haiku)
- Non-streaming completion
- Streaming completion with Server-Sent Events
- System message handling (separate parameter)
- Automatic retry with exponential backoff
- Rate limit handling
- Token usage tracking

#### Features
- Message types (System, User, Assistant)
- Response types with token usage
- Stream chunk types
- Error handling with retry logic
- HTTP timeout support
- Environment-based API key management
- Provider abstraction layer

#### Testing
- 18 unit tests (100% pass)
- 3 doc tests (100% pass)
- Mock provider for testing
- Provider creation tests
- Message formatting tests
- Error handling tests

#### Examples
- Basic completion example (both providers)
- Streaming completion example (both providers)
- Provider switching demonstration

#### Documentation
- Comprehensive rustdoc for all public APIs
- README for agent-llm crate
- Usage examples in documentation
- API integration guides

## [0.1.0] - 2025-12-01

### Added - Phase 0: Project Foundation

#### Infrastructure
- Cargo workspace with 10 crates structure
- Comprehensive `.gitignore` for Rust projects
- Makefile with development shortcuts
- Git repository initialization
- README and documentation

#### Error Handling (`agent-core`)
- `AgentError` enum with comprehensive error types
- Custom error types using `thiserror`
- Error conversion from standard library types
- `Result<T>` type alias for ergonomic error handling
- Full test coverage for error handling

#### Logging System (`agent-core`)
- Structured logging with `tracing` crate
- Async-aware logging infrastructure
- Configurable log levels (trace, debug, info, warn, error)
- JSON and pretty-print output formats
- Environment variable support for configuration
- Log level filtering

#### Configuration Management (`agent-core`)
- Multi-format support (TOML, JSON, YAML)
- Environment variable overrides with `AGENT__` prefix
- Default configuration fallback
- Type-safe configuration with `serde`
- Extensible configuration structure
- `AgentConfig` with logging and agent settings

#### Testing
- 11 unit tests for individual components
- 7 integration tests for component interaction
- 2 documentation tests
- 100% test pass rate
- Example program demonstrating all features

#### CLI Application (`agent-cli`)
- Basic command-line interface
- Configuration loading
- Logging initialization
- Foundation for future CLI commands

#### Documentation
- Comprehensive README
- Quick start guide
- Phase 0 completion checklist
- Phase 0 detailed completion report
- Implementation plan reference
- Inline documentation and examples

### Technical Details
- **Rust Edition**: 2021
- **Minimum Rust Version**: 1.75+
- **Dependencies**:
  - tokio: 1.42 (async runtime)
  - serde: 1.0 (serialization)
  - tracing: 0.1 (logging)
  - config: 0.14 (configuration)
  - thiserror: 1.0 (error handling)
  - anyhow: 1.0 (error context)

### Code Quality
- Zero clippy warnings with `-D warnings`
- All code formatted with `cargo fmt`
- Full test coverage on critical paths
- Clean compilation with no warnings

### Project Structure
```
agentslap/
├── crates/
│   ├── agent-core/          ✅ Implemented
│   ├── agent-cli/           ✅ Implemented
│   ├── agent-llm/           📅 Phase 1
│   ├── agent-session/       📅 Phase 3
│   ├── agent-tools/         📅 Phase 4
│   ├── agent-guardrails/    📅 Phase 7
│   ├── agent-hitl/          📅 Phase 8
│   ├── agent-context/       📅 Phase 9
│   ├── agent-mcp/           📅 Phase 10
│   └── agent-comms/         📅 Phase 11-12
```

## Development Timeline

- **Phase 0**: December 1, 2025 - Foundation ✅
- **Phase 1**: TBD - LLM Providers
- **Phase 2**: TBD - Message Management
- **Phase 3**: TBD - Session Management
- **Phase 4**: TBD - Tool Calling
- **Phase 5**: TBD - Agent Core Loop

[Unreleased]: https://github.com/yourusername/agent-runtime/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/agent-runtime/releases/tag/v0.1.0


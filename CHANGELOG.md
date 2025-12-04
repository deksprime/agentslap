# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 2 - Coming Soon
- Message & Conversation Management
- Multi-turn conversation handling
- Token counting utilities

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


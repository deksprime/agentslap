# Agent Runtime

A portable, production-ready agent runtime built in Rust with support for multiple LLM providers, tool calling, session management, and inter-agent communication.

## Features

- 🤖 **Multi-Provider LLM Support**: OpenAI, Anthropic, and more
- 🔧 **Tool Calling**: Extensible function calling system
- 💾 **Flexible Session Storage**: In-memory, cache, and database options
- 🛡️ **Guardrails**: Safety mechanisms and control systems
- 👤 **Human-in-the-Loop**: Approval workflows for sensitive operations
- 🧠 **Dynamic Context Management**: Intelligent context window handling
- 🔌 **MCP Support**: Model Context Protocol integration
- 🌐 **Agent Communication**: In-memory and network-based collaboration

## Project Structure

This is a Cargo workspace with multiple crates:

- `agent-core`: Core agent logic and orchestration
- `agent-llm`: LLM provider abstractions
- `agent-session`: Session management
- `agent-tools`: Tool system
- `agent-guardrails`: Safety systems
- `agent-hitl`: Human-in-the-loop
- `agent-context`: Context management
- `agent-mcp`: MCP implementation
- `agent-comms`: Communication layer
- `agent-cli`: Command-line interface

## Getting Started

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- API keys for LLM providers (OpenAI and/or Anthropic)
- cargo-watch (optional): `cargo install cargo-watch`

### Setup Environment Variables

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your API keys
nano .env
```

Your `.env` file should contain:
```bash
OPENAI_API_KEY=sk-your-actual-key
ANTHROPIC_API_KEY=sk-ant-your-actual-key
```

**Note**: The `.env` file is git-ignored and safe for local development. See `ENV_VARS.md` for production deployment options.

### Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific crate
cargo test -p agent-core
```

### Development

```bash
# Watch for changes and run tests automatically
cargo watch -x test

# Check code without building
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

## Development Phases

This project is being developed in phases:

- ✅ **Phase 0**: Project foundation
- ✅ **Phase 1**: LLM provider abstraction (current)
- 🔄 **Phase 2**: Message & conversation management
- 🔄 **Phase 3**: Session management
- 🔄 **Phase 4**: Tool calling system
- 🔄 **Phase 5**: Agent core loop

See [implementation_plan.md](../implementation_plan.md) for full roadmap.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

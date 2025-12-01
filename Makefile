.PHONY: help build test clean run example clippy fmt check all

help:
	@echo "Agent Runtime - Development Commands"
	@echo ""
	@echo "Usage:"
	@echo "  make build      - Build all crates"
	@echo "  make test       - Run all tests"
	@echo "  make example    - Run Phase 0 example"
	@echo "  make run        - Run CLI"
	@echo "  make clippy     - Run linter"
	@echo "  make fmt        - Format code"
	@echo "  make check      - Quick check without building"
	@echo "  make all        - Run tests, clippy, and fmt check"
	@echo "  make clean      - Clean build artifacts"
	@echo ""

build:
	cargo build

build-release:
	cargo build --release

test:
	cargo test --all

example:
	cargo run -p agent-core --example phase0_basic

run:
	cargo run -p agent-cli

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check --all

all: test clippy fmt-check
	@echo "✅ All checks passed!"

clean:
	cargo clean

watch:
	cargo watch -x test

# Phase-specific targets
phase0:
	@echo "🚀 Running Phase 0 verification..."
	@cargo test -p agent-core
	@cargo run -p agent-core --example phase0_basic
	@echo "✅ Phase 0 complete!"


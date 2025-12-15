# echidnabot - Development Tasks
set shell := ["bash", "-uc"]
set dotenv-load := true

project := "echidnabot"

# Show all recipes
default:
    @just --list --unsorted

# ============================================================================
# Build & Run
# ============================================================================

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the server (debug)
run *ARGS:
    cargo run -- serve {{ARGS}}

# Run the server (release)
run-release *ARGS:
    cargo run --release -- serve {{ARGS}}

# Run with verbose logging
run-debug:
    RUST_LOG=debug cargo run -- -v serve

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-one TEST:
    cargo test {{TEST}} -- --nocapture

# Run tests with coverage (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html

# ============================================================================
# Code Quality
# ============================================================================

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Run all checks (fmt + lint + test)
check: fmt-check lint test

# Fix clippy warnings automatically
fix:
    cargo clippy --fix --allow-dirty

# ============================================================================
# Database
# ============================================================================

# Initialize the database
init-db:
    cargo run -- init-db

# Reset database (delete and reinitialize)
reset-db:
    rm -f echidnabot.db
    cargo run -- init-db

# ============================================================================
# CLI Commands
# ============================================================================

# Register a repository
register REPO PLATFORM="github":
    cargo run -- register --repo {{REPO}} --platform {{PLATFORM}}

# Trigger a manual check
trigger REPO COMMIT="":
    cargo run -- check --repo {{REPO}} {{if COMMIT != "" { "--commit " + COMMIT } else { "" } }}

# Show status
status TARGET:
    cargo run -- status --target {{TARGET}}

# ============================================================================
# Development Utilities
# ============================================================================

# Watch for changes and rebuild
watch:
    cargo watch -x build

# Watch and run tests on change
watch-test:
    cargo watch -x test

# Generate documentation
docs:
    cargo doc --no-deps --open

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit
audit:
    cargo audit

# ============================================================================
# Docker
# ============================================================================

# Build Docker image
docker-build:
    podman build -t echidnabot:latest .

# Run in Docker
docker-run:
    podman run -p 8080:8080 echidnabot:latest

# ============================================================================
# Guix
# ============================================================================

# Enter Guix development shell
guix-shell:
    guix shell -D -f guix.scm

# Build with Guix
guix-build:
    guix build -f guix.scm

# ============================================================================
# Release
# ============================================================================

# Create a release build
release: fmt-check lint test build-release
    @echo "Release build complete: target/release/echidnabot"

# Package for distribution
package: release
    strip target/release/echidnabot
    @echo "Packaged binary: target/release/echidnabot"

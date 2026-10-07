# List available recipes.
default:
    @just --list

# Compile a debug binary.
build:
    cargo build --locked

alias compile := build

# Compile an optimized release binary.
release:
    cargo build --locked --release

# Check compilation without building a binary.
check:
    cargo check --locked

# Format Rust sources.
fmt:
    cargo fmt

# Check formatting without changing files.
fmt-check:
    cargo fmt --check

# Lint all targets, treating warnings as errors.
lint:
    cargo clippy --locked --all-targets -- -D warnings

# Run tests.
test:
    cargo test --locked

# Run all quality checks.
verify: fmt-check check lint test

# Run the CLI with arguments (for example: just run screenshot.png).
run *args:
    cargo run --locked -- {{args}}

# Install the CLI using Cargo.
install:
    cargo install --locked --path .

# Remove build artifacts.
clean:
    cargo clean

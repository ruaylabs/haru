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

# Print the package version.
version:
    @grep -m 1 '^version = ' Cargo.toml | sed 's/^version = "\([^"]*\)".*/\1/'

# Set the version, update Cargo.lock, commit, and tag (for example: just bump-version 0.2.0).
bump-version version:
    #!/usr/bin/env bash
    set -euo pipefail
    version={{quote(version)}}
    sed -i'' -e "/^\[package\]$/,/^\[/ s/^version = \".*\"/version = \"$version\"/" Cargo.toml
    cargo check
    git add Cargo.toml Cargo.lock
    if ! git diff --cached --quiet -- Cargo.toml Cargo.lock; then git commit --only -m "chore(release): bump v$version" -- Cargo.toml Cargo.lock; fi
    git tag -a "v$version" -m "v$version"

# Remove build artifacts.
clean:
    cargo clean

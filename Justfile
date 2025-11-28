# dtop development recipes
# Run `just` to see all available recipes

# Default recipe - show help
default:
    @just --list

# ─────────────────────────────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────────────────────────────

# Run dtop with local Docker daemon
run *ARGS:
    cargo run -- {{ARGS}}

# Run dtop with a remote host
run-remote HOST:
    cargo run -- --host {{HOST}}

# Run with multiple hosts
run-multi *HOSTS:
    cargo run -- {{HOSTS}}

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Build release without self-update (smaller binary)
build-minimal:
    cargo build --release --no-default-features

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Format check (CI)
fmt-check:
    cargo fmt -- --check

# Lint with clippy
lint:
    cargo clippy -- -D warnings

# ─────────────────────────────────────────────────────────────────────────────
# Testing
# ─────────────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run a specific test
test-one NAME:
    cargo test {{NAME}} -- --nocapture

# Run snapshot tests with insta
test-snapshots:
    cargo insta test

# Review pending snapshots
snapshots-review:
    cargo insta review

# Accept all pending snapshots
snapshots-accept:
    cargo insta accept

# Reject all pending snapshots
snapshots-reject:
    cargo insta reject

# ─────────────────────────────────────────────────────────────────────────────
# Docker
# ─────────────────────────────────────────────────────────────────────────────

# Build Docker image for current platform
docker-build:
    docker build -t dtop .

# Build Docker image with specific tag
docker-build-tag TAG:
    docker build -t dtop:{{TAG}} .

# Build multi-platform Docker image (amd64 + arm64)
docker-build-multi TAG="latest":
    docker buildx build --platform linux/amd64,linux/arm64 -t dtop:{{TAG}} .

# Run dtop in Docker (local daemon)
docker-run:
    docker run -v /var/run/docker.sock:/var/run/docker.sock -it --rm dtop

# Run dtop in Docker with custom args
docker-run-args *ARGS:
    docker run -v /var/run/docker.sock:/var/run/docker.sock -it --rm dtop {{ARGS}}

# Show Docker image size
docker-size:
    docker images dtop --format "{{{{.Repository}}}}:{{{{.Tag}}}}\t{{{{.Size}}}}"

# ─────────────────────────────────────────────────────────────────────────────
# Changelog & Release
# ─────────────────────────────────────────────────────────────────────────────

# Generate changelog for latest tag
changelog-latest:
    git-cliff --latest

# Generate changelog for unreleased changes
changelog-unreleased:
    git-cliff --unreleased

# Generate changelog for a version range
changelog-range FROM TO:
    git-cliff --tag {{FROM}}..{{TO}}

# Write full changelog to file
changelog-write:
    git-cliff -o CHANGELOG.md

# Preview changelog for next version
changelog-preview VERSION:
    git-cliff --unreleased --tag {{VERSION}}

# ─────────────────────────────────────────────────────────────────────────────
# Self-update
# ─────────────────────────────────────────────────────────────────────────────

# Test self-update command (dry run via cargo)
update-test:
    cargo run -- update

# ─────────────────────────────────────────────────────────────────────────────
# CI/Quality
# ─────────────────────────────────────────────────────────────────────────────

# Run all CI checks locally
ci: fmt-check lint test
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Show binary size (release build)
size: build-release
    @ls -lh target/release/dtop | awk '{print "Binary size: " $5}'

# Show minimal binary size (without self-update)
size-minimal: build-minimal
    @ls -lh target/release/dtop | awk '{print "Minimal binary size: " $5}'

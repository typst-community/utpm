# Install `just` via: cargo install just
# Then run: just --list

# Default recipe to display help
default:
    @just --list

# Format all Rust code
fmt:
    cargo fmt --all

# Check formatting without making changes
fmt-check:
    cargo fmt --all -- --check

# Run Clippy with all checks
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run Clippy and automatically fix issues when possible
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Run all tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Check if the code compiles
check:
    cargo check --all-targets --all-features

# Run all checks (format, clippy, tests)
ci: fmt-check clippy test
    @echo "✓ All checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Install the binary locally
install:
    cargo install --path .

# Install the binary to ~/.cargo/bin
install-local:
    cargo build --release
    cp ./target/release/utpm ~/.cargo/bin/

# Update dependencies
update:
    cargo update

# Generate documentation
doc:
    cargo doc --no-deps --open

# Run the binary with arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Setup git hooks
setup-hooks:
    #!/usr/bin/env bash
    echo "Setting up git hooks..."
    mkdir -p .git/hooks
    cat > .git/hooks/pre-commit << 'EOF'
    #!/bin/sh
    # Pre-commit hook for UTPM
    
    echo "Running pre-commit checks..."
    
    # Check formatting
    echo "→ Checking code formatting..."
    if ! cargo fmt -- --check; then
        echo "❌ Code is not formatted. Run 'cargo fmt' or 'just fmt' to fix."
        exit 1
    fi
    
    # Run Clippy
    echo "→ Running Clippy..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        echo "❌ Clippy found issues. Run 'cargo clippy --fix' or 'just clippy-fix' to fix."
        exit 1
    fi
    
    # Run tests
    echo "→ Running tests..."
    if ! cargo test --all-features; then
        echo "❌ Tests failed."
        exit 1
    fi
    
    echo "✓ All pre-commit checks passed!"
    EOF
    chmod +x .git/hooks/pre-commit
    echo "✓ Git hooks installed successfully!"

# Remove git hooks
remove-hooks:
    rm -f .git/hooks/pre-commit
    @echo "✓ Git hooks removed"

# Fix all auto-fixable issues
fix:
    cargo fmt --all
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
    @echo "✓ Auto-fixes applied. Review changes and commit."

# Run benchmarks (if any)
bench:
    cargo bench

# Check for security vulnerabilities
audit:
    cargo audit

# Watch for changes and run checks
watch:
    cargo watch -x check -x test -x run

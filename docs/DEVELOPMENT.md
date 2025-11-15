# Development Workflow for UTPM

This document describes the development workflow and tooling for UTPM contributors.

## Quick Reference

### Daily Development

```bash
# Format code
just fmt

# Check your code
just check

# Run tests
just test

# Run all CI checks locally
just ci

# Auto-fix issues
just fix
```

### First-Time Setup

```bash
# Install tools (if not already installed)
cargo install just          # Command runner
cargo install cargo-watch   # Watch for changes (optional)
cargo install cargo-audit   # Security audits (optional)

# Setup git hooks
just setup-hooks
```

## Code Quality Standards

### 1. Formatting (`rustfmt`)

All code must be formatted with `rustfmt` using the project's configuration.

**Configuration**: [`rustfmt.toml`](../rustfmt.toml)

**Run**:
```bash
cargo fmt --all
# Or
just fmt
```

**Check**:
```bash
cargo fmt --all -- --check
# Or
just fmt-check
```

**Editor Integration**:
- **VS Code**: Install "rust-analyzer" extension, enable "Format on Save"
- **IntelliJ/CLion**: Settings → Editor → Code Style → Rust → Rustfmt
- **Vim/Neovim**: Use `rust.vim` or `rust-tools.nvim`

### 2. Linting (`clippy`)

All code must pass Clippy with no warnings.

**Configuration**: [`clippy.toml`](../clippy.toml)

**Run**:
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Or
just clippy
```

**Auto-fix**:
```bash
cargo clippy --all-targets --all-features --fix --allow-dirty
# Or
just clippy-fix
```

**Common Clippy Warnings**:
- Unused imports → Remove them
- Unnecessary `clone()` → Review if clone is needed
- Complex types → Consider type aliases
- Missing documentation → Add doc comments for public items

### 3. Testing

All new features and bug fixes must include tests.

**Run tests**:
```bash
cargo test --all-features
# Or
just test
```

**Test with output**:
```bash
cargo test --all-features -- --nocapture
# Or
just test-verbose
```

**Test specific module**:
```bash
cargo test --test test_name
cargo test module_name::test_function
```

## Git Hooks

The project provides optional git hooks to automatically check code quality before commits.

### Install Hooks

```bash
just setup-hooks
```

This creates a pre-commit hook that:
1. Checks formatting
2. Runs Clippy
3. Runs tests

If any check fails, the commit is aborted.

### Remove Hooks

```bash
just remove-hooks
```

### Bypass Hooks (Emergency Only)

```bash
git commit --no-verify
```

**⚠️ Only use this in emergencies!** Your PR will still need to pass CI checks.

## Continuous Integration (CI)

All pull requests must pass CI checks before merging.

### CI Checks

1. **Formatting**: `cargo fmt --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Tests**: `cargo test` (on Ubuntu and Windows)
4. **Documentation**: `cargo doc`
5. **MSRV**: Minimum Supported Rust Version check

### Running CI Checks Locally

```bash
just ci
```

This runs all checks that CI will run, allowing you to catch issues before pushing.

## Common Workflows

### Adding a New Feature

1. Create a branch:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Implement your feature

3. Add tests

4. Format and fix:
   ```bash
   just fix
   ```

5. Run all checks:
   ```bash
   just ci
   ```

6. Commit:
   ```bash
   git commit -m "feat: add my feature"
   ```

7. Push and create PR

### Fixing a Bug

1. Create a branch:
   ```bash
   git checkout -b fix/issue-123
   ```

2. Write a failing test that reproduces the bug

3. Fix the bug

4. Verify the test passes

5. Format and check:
   ```bash
   just fix
   just ci
   ```

6. Commit:
   ```bash
   git commit -m "fix: resolve issue #123"
   ```

### Refactoring

1. Ensure all tests pass before starting:
   ```bash
   just test
   ```

2. Make your changes incrementally

3. Run tests frequently:
   ```bash
   just test
   ```

4. Before committing:
   ```bash
   just ci
   ```

## Troubleshooting

### Formatting Conflicts

If `rustfmt` changes conflict with your code style preferences:

1. Check if the change improves readability
2. If not, discuss in a PR comment
3. Never commit unformatted code

### Clippy False Positives

If Clippy raises a false positive:

1. Try to refactor to satisfy Clippy (usually improves code)
2. If refactoring makes code worse, use `#[allow(clippy::lint_name)]`
3. Add a comment explaining why the lint is allowed

Example:
```rust
// Clippy suggests using `if let`, but match is clearer here for all cases
#[allow(clippy::single_match)]
match value {
    Some(x) => process(x),
    None => (),
}
```

### Test Failures

If tests fail:

1. Check if you broke existing functionality
2. Update tests if behavior changed intentionally
3. Never disable tests without good reason

### CI Failures

If CI fails but local checks pass:

1. Ensure you're using the correct Rust version
2. Pull latest changes: `git pull origin main`
3. Rebase your branch: `git rebase main`
4. Re-run checks: `just ci`

## Additional Tools

### cargo-watch

Watch for file changes and run commands automatically:

```bash
cargo install cargo-watch

# Watch and run checks
just watch
```

### cargo-audit

Check for security vulnerabilities:

```bash
cargo install cargo-audit

# Run audit
just audit
```

## Questions?

- Check [CONTRIBUTING.md](./CONTRIBUTING.md)
- Open a discussion on GitHub
- Ask in a PR or issue

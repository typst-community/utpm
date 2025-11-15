# UTPM Tests

This directory contains the test suite for UTPM (Unofficial Typst Package Manager).

## Test Structure

```
tests/
├── common/
│   └── mod.rs           # Common test utilities and helpers
├── utils_tests.rs       # Unit tests for utils modules
├── command_tests.rs     # Tests for individual commands
├── integration_tests.rs # Integration tests
└── README.md           # This file
```

## Test Categories

### 1. Unit Tests (`utils_tests.rs`)

Tests for individual utility functions and modules:

- **Regex tests**: Package format validation, import pattern matching
- **Path tests**: Directory operations, file checks
- **Specs tests**: `[tool.utpm]` configuration parsing
- **State tests**: Error handling and error message formatting
- **Output tests**: Output format handling
- **Dry-run tests**: Dry-run mode functionality

### 2. Command Tests (`command_tests.rs`)

Tests for individual command functionality:

- **init**: Manifest creation, file structure, interactive mode
- **link**: Package linking, symlinks, file filtering
- **clone**: Package cloning from Typst Universe
- **unlink**: Package removal, cleanup
- **bump**: Version bumping in manifests and files
- **sync**: Dependency synchronization, import updates
- **list**: Package listing, multiple versions
- **metadata**: Metadata extraction from manifests
- **get**: Package information retrieval
- **install**: Git-based package installation

### 3. Integration Tests (`integration_tests.rs`)

End-to-end workflow tests:

- **Full workflows**: Init → Link → List → Unlink
- **Dependencies**: Package with dependencies
- **Version management**: Bumping and syncing
- **File filtering**: Exclude patterns, ignore files
- **Multiple namespaces**: Local vs preview packages
- **Error handling**: Missing files, invalid formats
- **Performance**: Many packages, large files

## Running Tests

### Run All Tests
```bash
just test
# or
cargo test --all-features
```

### Run Tests with Output
```bash
just test-verbose
# or
cargo test --all-features -- --nocapture
```

### Run Only Unit Tests
```bash
just test-unit
# or
cargo test --lib --all-features
```

### Run Only Integration Tests
```bash
just test-integration
# or
cargo test --test '*' --all-features
```

### Run Tests for Specific Module
```bash
just test-module regex_tests
# or
cargo test regex_tests --all-features -- --nocapture
```

### Run Tests in Watch Mode
```bash
just test-watch
# or
cargo watch -x 'test --all-features'
```

### Test Coverage
```bash
just test-coverage
# Requires: cargo install cargo-tarpaulin
```

## Test Helpers

The `common/mod.rs` module provides helpful utilities:

### Setup Functions
- `setup_temp_dir()` - Create temporary test directory
- `setup_test_env()` - Set up UTPM environment variables
- `cleanup_test_env()` - Clean up environment after tests

### File Creation
- `create_test_manifest()` - Create basic typst.toml
- `create_custom_manifest()` - Create typst.toml with custom content
- `create_test_entrypoint()` - Create main.typ file
- `create_test_package()` - Create complete package structure

### Assertions
- `assert_file_exists()` - Verify file exists
- `assert_dir_exists()` - Verify directory exists
- `assert_not_exists()` - Verify path doesn't exist
- `read_file_string()` - Read file content for verification

## Writing New Tests

### Example Unit Test

```rust
#[test]
fn test_my_function() {
    let temp_dir = setup_temp_dir();
    
    // Your test code here
    
    assert!(condition);
}
```

### Example Integration Test

```rust
#[test]
fn test_full_workflow() {
    let temp_dir = setup_temp_dir();
    setup_test_env(temp_dir.path());
    
    // Step 1: Create package
    create_test_package(temp_dir.path(), "my-pkg", "1.0.0");
    
    // Step 2: Verify structure
    assert_file_exists(&temp_dir.path().join("typst.toml"));
    
    // Step 3: Clean up
    cleanup_test_env();
}
```

## Test Guidelines

1. **Isolation**: Each test should be independent and not affect others
2. **Cleanup**: Always clean up temporary files and environment variables
3. **Descriptive Names**: Use clear, descriptive test function names
4. **Documentation**: Add comments for complex test logic
5. **Assertions**: Use clear assertion messages
6. **Edge Cases**: Test both valid and invalid inputs

## Dependencies

Test dependencies are declared in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.15"  # Temporary directory management
```

## Continuous Integration

Tests are automatically run in CI on:
- Every push to `main` or `dev` branches
- Every pull request
- Pre-commit hooks (if installed)

CI runs:
```bash
just ci  # Includes: format check, clippy, and all tests
```

## Debugging Tests

### Print Debug Output
```bash
cargo test test_name -- --nocapture
```

### Run Single Test
```bash
cargo test test_name --all-features
```

### Show Test Output
```bash
cargo test -- --show-output
```

### Run Tests in Sequence (Not Parallel)
```bash
cargo test -- --test-threads=1
```

## Common Issues

### Environment Variables Not Reset
- Solution: Always call `cleanup_test_env()` in tests
- Use `#[serial]` attribute if tests must run sequentially

### Temporary Files Not Cleaned
- Solution: Use `tempfile::TempDir` which auto-cleans on drop
- Ensure test doesn't panic before cleanup

### Flaky Tests
- Avoid time-dependent tests
- Use deterministic inputs
- Don't rely on external services in unit tests

## Coverage

To generate test coverage report:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
just test-coverage

# Open report
open coverage/index.html
```

## Contributing

When adding new features:
1. Write tests first (TDD approach)
2. Ensure all tests pass: `just test`
3. Check coverage: `just test-coverage`
4. Run full CI checks: `just ci`

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Testing Best Practices](https://rust-lang.github.io/api-guidelines/testing.html)

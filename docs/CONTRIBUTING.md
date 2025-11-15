# Contributing to UTPM

Thank you for your interest in contributing to UTPM! This document provides guidelines and standards for contributing to the project.

## Code Standards

### Formatting

UTPM uses `rustfmt` to ensure consistent code formatting across the project.

**Before submitting a PR:**
```bash
# Format your code
cargo fmt --all

# Or use just
just fmt
```

**To check formatting without changes:**
```bash
cargo fmt --all -- --check
# Or
just fmt-check
```

Configuration is in [`rustfmt.toml`](./rustfmt.toml).

### Linting

UTPM uses Clippy to catch common mistakes and enforce best practices.

**Run Clippy:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Or
just clippy
```

**Auto-fix Clippy issues:**
```bash
cargo clippy --all-targets --all-features --fix --allow-dirty
# Or
just clippy-fix
```

Configuration is in [`clippy.toml`](./clippy.toml).

### Testing

All new features and bug fixes should include tests.

**Run tests:**
```bash
cargo test --all-features
# Or
just test
```

**Run tests with output:**
```bash
cargo test --all-features -- --nocapture
# Or
just test-verbose
```

## Development Workflow

### 1. Setup

```bash
# Clone the repository
git clone https://github.com/typst-community/utpm.git
cd utpm

# Install git hooks (optional but recommended)
just setup-hooks
```

The git hooks will automatically run formatting checks, Clippy, and tests before each commit.

### 2. Making Changes

1. Create a new branch for your feature/fix:
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. Make your changes following the code standards

3. Format and lint your code:
   ```bash
   just fix  # Auto-format and fix Clippy issues
   ```

4. Run tests:
   ```bash
   just test
   ```

5. Commit your changes:
   ```bash
   git add .
   git commit -m "feat: add my awesome feature"
   ```

### 3. Before Submitting a PR

Run all checks to ensure your code is ready:

```bash
just ci
```

This will run:
- Format check (`cargo fmt -- --check`)
- Clippy (`cargo clippy`)
- Tests (`cargo test`)

### 4. Submitting a Pull Request

1. Push your branch:
   ```bash
   git push origin feature/my-awesome-feature
   ```

2. Open a Pull Request on GitHub

3. Ensure all CI checks pass

4. Address any review feedback

## Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

**Examples:**
```
feat(metadata): add command to extract typst.toml metadata
fix(link): respect typst_ignore flag when linking packages
docs: update README with metadata command examples
chore: add rustfmt and clippy configuration
```

## Code Organization

### Project Structure

See the source code and comments for detailed project architecture and patterns.

Key principles:
- All commands are in `src/commands/`
- Utilities are in `src/utils/`
- Use `utpm_log!` for logging
- Use `utpm_bail!` for errors
- Check `get_dry_run()` before file operations
- All command functions are `async`

### Adding a New Command

1. Create `src/commands/new_command.rs`
2. Add to `src/commands.rs` (module declaration and args struct)
3. Add to command enum in `src/commands.rs`
4. Add dispatcher in `src/main.rs`
5. Add tests
6. Update documentation (README.md, CONTRIBUTING.md, DEVELOPMENT.md)

See the [Metadata command](./src/commands/metadata.rs) as a reference implementation.

## Tools and Utilities

### Just Commands

We use [`just`](https://github.com/casey/just) as a command runner. Install it:

```bash
cargo install just
```

Available commands:
```bash
just --list          # Show all available commands
just fmt             # Format code
just clippy          # Run Clippy
just test            # Run tests
just ci              # Run all CI checks
just fix             # Auto-fix formatting and Clippy issues
just build-release   # Build in release mode
just install-local   # Install to ~/.cargo/bin
just setup-hooks     # Setup git hooks
```

### Editor Configuration

An `.editorconfig` file is provided for consistent editor settings. Most modern editors support EditorConfig automatically or via plugins.

## Getting Help

- **Issues**: Open an issue on GitHub for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the source code for technical details

## License

By contributing to UTPM, you agree that your contributions will be licensed under the MIT License.

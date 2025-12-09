# UTPM - Copilot Instructions

## Project Overview

**UTPM** (Unofficial Typst Package Manager) is a command-line package manager written in Rust for managing [Typst](https://typst.app/) packages. It provides comprehensive tools for creating, managing, linking, and publishing Typst packages for both local development and remote distribution via Typst Universe.

### Key Information
- **Language**: Rust (Edition 2024)
- **Version**: 0.3.0
- **License**: MIT
- **Repository**: https://github.com/typst-community/utpm
- **Primary Dependencies**: 
  - `clap` (4.5.39) - CLI parsing with derive macros
  - `typst-kit` (0.13.1) - Core Typst functionality and package downloading
  - `typst-syntax` (0.13.1) - Parsing and working with Typst manifests
  - `tokio` (1.45.1) - Async runtime with full features
  - `ignore` (0.4.23) - File filtering with gitignore-style patterns
  - `octocrab` (0.44.1) - GitHub API client
  - `tracing` (0.1.41) - Structured logging and instrumentation
  - `inquire` (0.7.5) - Interactive CLI prompts
  - `semver` (1.0.26) - Semantic versioning
  - `serde` (1.0) - Serialization/deserialization
  - `shadow-rs` (1.1.1) - Build-time information
  - `ecow` (0.2) - Efficient copy-on-write data structures

### Project Philosophy
- **Dry-run first**: All destructive operations support `--dry-run` mode
- **Flexible output**: Support for JSON, YAML, TOML, HJSON, and text output formats
- **Respectful of conventions**: Honors `.gitignore`, `.typstignore`, and custom ignore files
- **Developer-friendly**: Clear error messages, verbose logging, and intuitive CLI
- **Code quality**: Enforced formatting (rustfmt) and linting (Clippy) standards

### Recent Changes & Features

**Version 0.3.0 Updates:**

1. **Metadata Command** (Issue #89)
   - New command: `utpm prj metadata` to extract metadata from `typst.toml`
   - Supports extracting specific fields with `-f` flag (name, version, authors, etc.)
   - Outputs in multiple formats (text, JSON, YAML, TOML, HJSON)
   - Ideal for shell scripts and CI/CD pipelines
   - Properly converts `EcoString` types to `String` for serialization

2. **File Exclusion Documentation** (Issue #88)
   - Documented `[tool.utpm]` exclude patterns in README
   - Added comprehensive examples in `assets/typst.toml.example`
   - Fixed bug where `typst_ignore` flag was not respected in `link` and `publish` commands

3. **Code Quality Standards**
   - Added `rustfmt.toml` for consistent code formatting (stable features only)
   - Added `clippy.toml` for Clippy linting configuration
   - Added `.editorconfig` for cross-editor consistency
   - Created `justfile` with common development commands
   - Git hooks support via `just setup-hooks`
   - CI already includes format and lint checks

4. **Documentation Reorganization**
   - Created `docs/` directory for all developer documentation
   - Added `docs/GUIDE.md` - Comprehensive guide for users, package authors, and contributors
   - Moved `docs/CONTRIBUTING.md` - Contribution guidelines and code standards
   - Moved `docs/DEVELOPMENT.md` - Development workflow and tooling
   - Added `docs/TESTING.md` - Comprehensive testing guide and documentation
   - `.github/copilot-instructions.md` remains in .github/ for AI assistant context
   - All documentation accessible and beginner-friendly

5. **Comprehensive Test Suite** (November 2025)
   - Created full test infrastructure with 60+ tests
   - Test structure: `tests/common/mod.rs` with 10 helper functions
   - Unit tests: `tests/utils_tests.rs` - 18 tests for utility functions
   - Command tests: `tests/command_tests.rs` - 24 tests for all commands
   - Integration tests: `tests/integration_tests.rs` - 14 end-to-end workflow tests
   - Test dependencies: `tempfile = "3.15"` for isolated testing
   - Library exports: `src/lib.rs` exposes modules for testing
   - Enhanced `justfile` with test commands (test-unit, test-integration, test-module, test-coverage, test-watch)
   - Documentation: `docs/TESTING.md` with comprehensive testing guide
   - All tests passing, environment isolation working correctly

6. **Complete API Documentation** (November 2025)
   - All 60+ public functions now have comprehensive documentation
   - Documented all utility functions in `src/utils.rs` (copy_dir_all, try_find_path, try_find, write_manifest)
   - Documented all path functions in `src/utils/paths.rs` (get_data_dir, get_cache_dir, c_packages, d_packages, etc.)
   - Documented all git operations in `src/utils/git.rs` (exist_git, clone_git, push_git, pull_git, add_git, commit_git)
   - Documented all command run functions and helpers
   - Added function descriptions, parameter explanations, return values, and error conditions
   - Documentation includes usage examples and edge cases
   - cargo doc builds successfully without warnings

7. **Feature Flag Cleanup** (November 2025)
   - Removed all command-specific feature flags (install, clone, publish, etc.)
   - Kept only output format features: `output_json`, `output_yaml`, `output_hjson`
   - Default feature: `output_json` (most common use case)
   - Full output feature: `full_output = ["output_json", "output_hjson", "output_yaml"]`
   - Features properly used with `#[cfg(feature = "...")]` in 4 files:
     - `src/utils/state.rs` - Error handling for output formats
     - `src/utils/macros.rs` - utpm_log! macro output formatting
     - `src/utils/output.rs` - OutputFormat enum variants
     - `src/commands/metadata.rs` - Metadata serialization
   - Simplified build configuration and reduced binary size options

8. **Performance Optimization with `ecow`** (November 2025)
   - Integrated `ecow` (0.2) for efficient copy-on-write data structures
   - Replaced `Vec<String>` with `EcoVec<String>` in `Extra` struct (utils/specs.rs)
   - Benefits: Reduced memory allocations when cloning exclude patterns
   - Clone operations on `Extra::exclude` now use reference counting instead of deep copies
   - Optimized string concatenations throughout codebase:
     - Replaced `curr.clone() + MANIFEST_PATH` with `format!("{}{}", curr, MANIFEST_PATH)`
     - Removed unnecessary `.clone()` calls in command handlers
     - Used string slices and references where possible
   - Optimized HashMap construction in `get.rs` with `with_capacity` and single iteration
   - Updated tests to use `eco_vec!` macro for test data
   - Performance improvements particularly noticeable with large exclude pattern lists

9. **Automated Multi-Platform Release System** (November 2025)
   - Created comprehensive GitHub Actions workflow (`.github/workflows/release.yml`)
   - Multi-platform build matrix for 5 platforms (Windows excluded - handled by other maintainer):
     - Linux: x86_64-gnu, aarch64-gnu, x86_64-musl
     - macOS: x86_64, aarch64
   - Automated workflow steps:
     1. Builds optimized binaries for all platforms (with stripping)
     2. Generates shell completions (bash, fish, zsh) automatically
     3. Creates release archives (`.tar.gz` with proper structure)
     4. Uploads artifacts to GitHub Release
     5. Calculates SHA256 checksums for all binaries
     6. Updates package manager files with new versions/checksums
     7. Creates Pull Request with updated checksums
     8. **Publishes to AUR** (utpm-bin and utpm-git packages)
     9. **Publishes to Homebrew Tap** (typst-community/utpm)
     10. **Publishes to Snap Store** (stable channel)
     11. **Creates Flatpak PR** (to Flathub, requires manual merge)
   - Trigger methods:
     - Automatic: Push git tag starting with `v` (e.g., `v0.3.0`)
     - Manual: Workflow dispatch from GitHub Actions UI
   - Package manager automation:
     - **AUR**: SSH authentication, automatic PKGBUILD update and .SRCINFO generation
     - **Homebrew Tap**: Direct push to typst-community/homebrew-utpm repository
     - **Snap Store**: Build snap and upload with snapcraft credentials
     - **Flatpak**: Create PR to Flathub repository (manual review/merge required)
   - Package manager file updates (automatic via sed):
     - AUR PKGBUILD (utpm-bin) - version + checksums (x86_64 + aarch64)
     - Homebrew formula - URL + checksum
     - Debian changelog - version number
     - RPM spec - version number
     - Snap snapcraft.yaml - version + source tag
     - Flatpak manifest - git tag
   - Documentation created:
     - `docs/RELEASING.md` - Release workflow guide for contributors
     - `docs/PACKAGING.md` - Technical package manager details for developers
     - `docs/PUBLISHING.md` - Publishing guide (mostly automated, minimal manual steps)
     - `docs/SECRETS.md` - GitHub secrets configuration guide for maintainers
     - Updated `README.md` with installation instructions for all package managers
   - PR automation: Uses `peter-evans/create-pull-request@v5` to create PR with checksums
   - Cross-compilation: Includes aarch64 Linux support with gcc-aarch64-linux-gnu
   - Required GitHub secrets for full automation:
     - `AUR_SSH_KEY` - SSH private key for publishing to AUR
     - `SNAPCRAFT_TOKEN` - Snapcraft store credentials for Snap publishing
     - `FLATPAK_TOKEN` - GitHub token for creating PRs on Flathub
   - Workflow jobs: `build` → `release` → `update-checksums` → `publish-aur` + `publish-homebrew` + `publish-snap` + `publish-flatpak`
   - All jobs run in parallel after checksums are updated for maximum efficiency

## Architecture

### Project Structure

```
src/
├── main.rs                 # Entry point, CLI dispatcher, logging setup
├── lib.rs                  # Library exports for testing and external use
├── commands.rs             # CLI argument definitions and command routing
├── utils.rs                # Utility functions and module aggregator
├── commands/               # Individual command implementations
│   ├── init.rs            # Create new typst.toml manifest
│   ├── link.rs            # Link packages for local development
│   ├── unlink.rs          # Remove linked packages
│   ├── clone.rs           # Clone packages from Typst Universe
│   ├── publish.rs         # Publish packages to Typst Universe (WIP)
│   ├── bump.rs            # Bump package versions
│   ├── sync.rs            # Sync dependencies to latest versions
│   ├── metadata.rs        # Extract metadata from typst.toml
│   ├── install.rs         # Install packages from git repos
│   ├── get.rs             # Get package information from remote
│   ├── list.rs            # List local packages
│   ├── package_path.rs    # Display package directory path
│   └── generate.rs        # Generate shell completions
└── utils/                  # Utility modules
    ├── dryrun.rs          # Dry-run mode support
    ├── git.rs             # Git operations
    ├── macros.rs          # Custom macros (utpm_log!, utpm_bail!)
    ├── output.rs          # Output format handling
    ├── paths.rs           # Path resolution and directory management
    ├── specs.rs           # [tool.utpm] configuration parsing
    └── state.rs           # Error types and Result definitions

tests/
├── common/
│   └── mod.rs              # Common test utilities (10 helper functions)
├── utils_tests.rs          # Unit tests for utils modules (18 tests)
├── command_tests.rs        # Tests for individual commands (24 tests)
└── integration_tests.rs    # Integration tests (14 tests)
```

### Command Hierarchy

The CLI has two main subcommands:

1. **`project` (alias: `prj`)** - Project management
   - `init` - Create new typst.toml manifest
   - `link` - Link package for local development
   - `clone` - Clone from Typst Universe
   - `bump` - Bump version numbers
   - `sync` - Sync dependencies
   - `metadata` - Extract metadata from typst.toml
   - `publish` - Publish to Typst Universe (WIP)

2. **`packages` (alias: `pkg`)** - Package management
   - `list` - List local packages
   - `path` - Show package directory path
   - `unlink` - Remove linked packages
   - `get` - Get package info from remote
   - `install` - Install from git repository

3. **`generate` (alias: `g`)** - Generate shell completions

### Core Concepts

#### Package Manifest (`typst.toml`)

Every Typst package requires a `typst.toml` manifest file with standard package metadata. UTPM extends this with a `[tool.utpm]` section for UTPM-specific configuration.

**Standard fields:**
- `[package]` - Name, version, entrypoint, authors, license, description, repository, etc.
- `[template]` - Template configuration (optional)
- `[tool.*]` - Tool-specific configuration

**UTPM Extension (`[tool.utpm]`):**
```toml
[tool.utpm]
exclude = [
  ".git",
  ".github", 
  "*.md",
  "tests/",
  "examples/"
]
```

The `exclude` field uses glob patterns to exclude files from:
- `utpm prj link` - Linking packages for development
- `utpm prj publish` - Publishing packages

#### Package Locations

UTPM manages packages in two locations:

1. **Local packages** (user-created): `$DATA_DIR/typst/packages/`
   - Default: `~/.local/share/typst/packages/` on Linux
   - Override with `UTPM_DATA_DIR` environment variable
   - Structure: `{namespace}/{name}/{version}/`

2. **Cache packages** (downloaded from Typst Universe): `$CACHE_DIR/typst/packages/`
   - Default: `~/.cache/typst/packages/` on Linux
   - Override with `UTPM_CACHE_DIR` environment variable
   - Structure: `preview/{name}/{version}/` (preview namespace only)

#### File Filtering System

UTPM uses the `ignore` crate with `WalkBuilder` and `OverrideBuilder` to implement intelligent file filtering for `link` and `publish` commands.

**Supported ignore files (configurable via CLI flags):**
- `.gitignore` (default: enabled, flag: `-g`)
- `.typstignore` (default: enabled, flag: `-t`)
- `.ignore` (default: disabled, flag: `-i`)
- Global `.gitignore` (default: enabled, flag: `-G`)
- `.git/info/exclude` (default: enabled, flag: `-x`)
- `[tool.utpm] exclude` patterns from `typst.toml`

**Pattern syntax:**
- Standard glob patterns (`*`, `**`, `?`)
- Prefix with `!` to negate/include
- Patterns are converted to exclude format by prepending `!` in code

## Code Quality & Standards

### Formatting

UTPM uses `rustfmt` for consistent code formatting. Configuration is in `rustfmt.toml`.

**Important**: Only stable rustfmt features are enabled by default. Nightly features are commented out in the config.

**Format code:**
```bash
cargo fmt --all
# Or with just
just fmt
```

**Check formatting:**
```bash
cargo fmt --all -- --check
# Or with just
just fmt-check
```

### Linting

UTPM uses Clippy for catching common mistakes and enforcing best practices. Configuration is in `clippy.toml`.

**Run Clippy:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Or with just
just clippy
```

**Auto-fix issues:**
```bash
cargo clippy --all-targets --all-features --fix --allow-dirty
# Or with just
just clippy-fix
```

### Development Tools

**justfile**: Common commands are defined in `justfile`. Install `just`: `cargo install just`

```bash
just --list          # Show all commands
just fmt             # Format code
just clippy          # Run Clippy
just test            # Run tests
just ci              # Run all CI checks
just fix             # Auto-fix formatting and linting
just setup-hooks     # Install git hooks
```

**Git Hooks**: Optional pre-commit hooks that run formatting, Clippy, and tests.

```bash
just setup-hooks     # Install hooks
just remove-hooks    # Remove hooks
```

### Contributing Guidelines

See `CONTRIBUTING.md` for detailed guidelines on:
- Code formatting and linting standards
- Testing requirements
- Commit message conventions (Conventional Commits)
- PR submission process

See `.github/DEVELOPMENT.md` for development workflow details.

## Code Patterns and Conventions

### Error Handling

UTPM uses a custom error type `UtpmError` defined in `utils/state.rs`:

```rust
pub type Result<T> = anyhow::Result<T, UtpmError>;

#[derive(Debug, TError)]
pub enum UtpmError {
    SemVer(#[from] semver::Error),
    IO(#[from] std::io::Error),
    Manifest,
    PackageNotExist,
    // ... more variants
}
```

**Key macros:**
- `utpm_bail!(ErrorVariant, args...)` - Return an error early
- `utpm_log!(level, message, key => value, ...)` - Structured logging

**Examples:**
```rust
// Simple error
utpm_bail!(Manifest);

// Error with arguments
utpm_bail!(AlreadyExist, name.to_string(), version, "Info:".to_string());

// Simple logging
utpm_log!(info, "Package linked successfully");

// Logging with structured data
utpm_log!(trace, "Processing file", "path" => path, "size" => size);

// Logging with format strings
utpm_log!(info, "Found {} packages", count);
```

### Logging

UTPM uses `tracing` for structured logging with multiple output formats:

- **Log levels**: Controlled by `UTPM_DEBUG` env var or `--verbose` flag
- **Output formats**: 
  - Text (default) - Human-readable
  - JSON - Machine-readable (enabled when output format != Text)
- **Log macro**: `utpm_log!(level, "message", "key" => value)`

### Async Operations

The main function uses `tokio` for async operations:

```rust
#[tokio::main]
async fn main() {
    // All command handlers are async
    let res = match commands {
        Commands::Project(ProjectArgs::Link(cmd)) => commands::link::run(cmd, None, true).await,
        // ...
    }.await;
}
```

### Command Structure

Each command module follows this pattern:

```rust
// In commands/{command}.rs
use crate::utils::*;

#[instrument(skip(cmd))]  // Tracing instrumentation (from tracing crate)
pub async fn run(cmd: &CommandArgs) -> Result<bool> {
    utpm_log!(trace, "executing {command} command");
    
    // Check dry-run mode before any file operations
    if !get_dry_run() {
        // Perform actual operations (write files, create dirs, etc.)
        fs::write(&path, content)?;
    }
    
    // Always log success, even in dry-run
    utpm_log!(info, "Command completed successfully");
    
    // Return success
    Ok(true)
}
```

**Key points:**
- All command functions are `async` (required by tokio)
- Use `#[instrument(skip(cmd))]` for automatic trace logging
- Check `get_dry_run()` before any file system modifications
- Return `Result<bool>` (typically `Ok(true)` on success)
- Use `utpm_log!` for all output (respects output format)

### Configuration Access

The `Extra` struct in `utils/specs.rs` represents `[tool.utpm]` configuration:

```rust
pub struct Extra {
    pub exclude: Option<Vec<String>>,
}

// Extract from manifest
let config = try_find(&curr)?;
let extra = Extra::from(config.tool);
let excludes = extra.exclude.unwrap_or(vec![]);
```

### Path Resolution

Use functions from `utils/paths.rs`:

```rust
// Get paths
let current = get_current_dir()?;      // Can override with UTPM_CURRENT_DIR
let data = d_packages()?;              // ~/.local/share/typst/packages
let cache = c_packages()?;             // ~/.cache/typst/packages

// Check paths
check_path_dir(&path);   // Returns bool
check_path_file(&path);  // Returns bool
has_content(&path)?;     // Returns Result<bool>
```

### Dry-Run Mode

All file operations should check dry-run mode:

```rust
use crate::utils::dryrun::get_dry_run;

if !get_dry_run() {
    fs::write(&path, content)?;  // Only write if not dry-run
}
```

### Output Formatting

Commands can output in multiple formats:

```rust
use crate::utils::output::{get_output_format, OutputFormat};

match get_output_format() {
    OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
    OutputFormat::Text => println!("Human readable: {}", data),
    // ... other formats
}
```

## Important Implementation Details

### Link Command (`commands/link.rs`)

The `link` command copies or symlinks a package to the local package directory:

1. Loads `typst.toml` manifest from current directory
2. Determines destination: `{data|cache}/typst/packages/{namespace}/{name}/{version}/`
3. Checks for existing package (error unless `--force`)
4. If `--no-copy`: creates symlink
5. If copying: uses `WalkBuilder` with ignore files + `[tool.utpm]` excludes
6. Respects `.gitignore`, `.typstignore`, and custom ignore files

**Critical implementation notes:**
- The `typst_ignore` flag must be checked before adding `.typstignore`
- Exclude patterns from manifest are prefixed with `!` for `OverrideBuilder`
- Files are copied preserving directory structure

### Publish Command (`commands/publish.rs`)

Similar to `link` but packages files for publishing (WIP):

1. Uses same filtering system as `link`
2. Creates package archive
3. Validates package structure
4. (TODO) Publishes to Typst Universe

### Init Command (`commands/init.rs`)

Creates a new `typst.toml` manifest:

1. Interactive prompts (unless `--cli` mode)
2. Creates directory structure (examples/, src/)
3. Generates `typst.toml` with `[tool.utpm]` section
4. Creates default entrypoint file

**Default `[tool.utpm]` section:**
```rust
let mut keys: BTreeMap<_, Table> = BTreeMap::new();
keys.insert("utpm".into(), Table::try_from(Extra::default())?);
```

**CLI mode flags:**
- `-m, --cli` - Disable interactive session
- `-f, --force` - Overwrite existing manifest
- `-n, --name` - Project name
- `-V, --version` - Project version
- `-e, --entrypoint` - Main file (default: "main.typ")
- `-a, --authors` - Authors (comma-separated)
- `-l, --license` - License identifier
- `-d, --description` - Short description
- `-r, --repository` - Repository URL

### Bump Command (`commands/bump.rs`)

Updates version numbers across multiple files:

1. Parses new version with semver
2. Updates `typst.toml`
3. Searches additional files specified with `--include`
4. Uses regex or tag-based search (with `--tag` option)
5. Supports Markdown/HTML version tags

### Sync Command (`commands/sync.rs`)

Synchronizes dependencies to latest versions:

1. Reads dependencies from `typst.toml`
2. Queries Typst Universe API for latest versions
3. Updates manifest with new versions
4. Can check without updating (`--check`)

### Metadata Command (`commands/metadata.rs`)

Extracts metadata from `typst.toml` for use in scripts:

1. Loads the manifest from current or specified directory
2. Extracts all package metadata fields
3. Supports extracting a specific field with `-f` flag
4. Outputs in multiple formats (text, JSON, YAML, TOML, HJSON)

**Key features:**
- Can extract specific fields: `name`, `version`, `entrypoint`, `authors`, `license`, `description`, `repository`, `homepage`, `keywords`, `categories`, `disciplines`, `compiler`, `exclude`
- Returns plain text for single field extraction (ideal for shell scripts)
- Returns structured data for all metadata
- Converts `EcoString` types from typst-syntax to `String` for serialization

**Usage examples:**
```bash
# Get all metadata
utpm prj metadata

# Get specific field (for scripts)
VERSION=$(utpm prj metadata -f version)
NAME=$(utpm prj metadata -f name)

# Get metadata as JSON
utpm -o json prj metadata

# Get metadata from specific path
utpm prj metadata -p /path/to/project
```

## Development Guidelines

### Adding a New Command

1. Create `src/commands/{command}.rs`:
```rust
use crate::utils::*;

#[instrument(skip(cmd))]
pub async fn run(cmd: &CommandArgs) -> Result<bool> {
    utpm_log!(trace, "executing command");
    // Implementation
    Ok(true)
}
```

2. Add to `src/commands.rs`:
```rust
pub mod {command};

#[derive(Parser, Clone, Debug, PartialEq)]
pub struct CommandArgs {
    // Define arguments
}
```

3. Add to command enum and dispatcher in `src/main.rs`

### Adding to `[tool.utpm]` Configuration

1. Update `Extra` struct in `utils/specs.rs`:
```rust
pub struct Extra {
    pub exclude: Option<Vec<String>>,
    pub new_field: Option<Type>,
}
```

2. Update `From<ToolInfo>` implementation to parse new field
3. Update `Default` implementation
4. Document in README.md and example files

### Working with Ignore Patterns

When using `WalkBuilder` for file filtering:

```rust
let mut wb = WalkBuilder::new(&path);
let mut overr = OverrideBuilder::new(&path);

// Add excludes from [tool.utpm]
for exclude in Extra::from(config.tool).exclude.unwrap_or(vec![]) {
    overr.add(("!".to_string() + &exclude).as_str())?;
}
wb.overrides(overr.build()?);

// Configure ignore files (check flags first!)
wb.ignore(cmd.ignore)
  .git_ignore(cmd.git_ignore)
  .git_global(cmd.git_global_ignore)
  .git_exclude(cmd.git_exclude);

// Add .typstignore if enabled
if cmd.typst_ignore {
    let typstignore = path.join(".typstignore");
    if check_path_file(&typstignore) {
        wb.add_custom_ignore_filename(".typstignore");
    }
}

// Walk and process files
for entry in wb.build() {
    // Process each file/directory
}
```

### Testing

- Check dry-run mode: `utpm --dry-run <command>`
- Use verbose logging: `utpm -v trace <command>` or `UTPM_DEBUG=trace`
- Test with different output formats: `utpm -o json <command>`
- Override paths for testing:
  - `UTPM_DATA_DIR`
  - `UTPM_CACHE_DIR`
  - `UTPM_CURRENT_DIR`

## Common Patterns

### Reading Manifest

```rust
use crate::utils::{try_find, try_find_path};

// Get manifest path
let manifest_path = try_find_path(&current_dir)?;

// Parse manifest
let manifest: PackageManifest = try_find(&current_dir)?;
```

### Writing Manifest

```rust
use crate::utils::write_manifest;

let manifest = PackageManifest { /* ... */ };
write_manifest(&manifest)?;  // Respects dry-run mode
```

### Platform-Specific Symlinks

```rust
use crate::utils::symlink_all;

symlink_all(&source, &destination)?;  // Works on Windows and Unix
```

### Package Regex

```rust
use crate::utils::regex_package;

let regex = regex_package();
// Matches: @namespace/name:1.0.0
```

## Known Issues and TODOs

1. **Publish command** - Still work in progress
2. **Progress indicator** - `ProgressPrint` struct not fully implemented
3. **YAML/HJSON/TOML output** - Experimental, requires manual build with features
4. **Error messages** - Some could be more descriptive
5. **Git package management** - Limited functionality for git-based packages

## Quick Reference

### Common CLI Patterns

```bash
# Initialize new package
utpm prj init

# Link package for development (respects .gitignore and .typstignore by default)
utpm prj link

# Link with custom namespace
utpm prj link preview

# Link with force (overwrite existing)
utpm prj link --force

# Dry-run mode (no file writes)
utpm --dry-run prj link

# JSON output
utpm -o json pkg list

# Verbose logging
utpm -v trace prj link

# Clone from Typst Universe
utpm prj clone @preview/example:1.0.0

# Bump version
utpm prj bump 1.1.0

# Sync dependencies
utpm prj sync

# Get metadata from typst.toml
utpm prj metadata

# Get specific field (useful for scripts)
utpm prj metadata -f version
utpm prj metadata -f name

# Get metadata as JSON
utpm -o json prj metadata

# List local packages
utpm pkg list

# Unlink package
utpm pkg unlink @local/mypackage:1.0.0
```

### Macro Quick Reference

```rust
// Error handling
utpm_bail!(Manifest);
utpm_bail!(AlreadyExist, name, version, "Info:".to_string());

// Logging
utpm_log!(info, "message");
utpm_log!(trace, "Processing", "file" => path);
utpm_log!(error, "Failed with code {}", code);

// Check dry-run
if !get_dry_run() {
    fs::write(&path, content)?;
}

// Get output format
match get_output_format() {
    OutputFormat::Json => { /* ... */ },
    OutputFormat::Text => { /* ... */ },
    _ => { /* ... */ }
}
```

## Important Files

- `build.rs` - Build script using shadow-rs for build info
- `Cargo.toml` - Dependencies and feature flags
- `assets/typst.toml.example` - Example configuration
- `.github/workflows/` - CI/CD pipelines

## Environment Variables

- `UTPM_DEBUG` - Set log level (trace, debug, info, warn, error)
- `UTPM_DATA_DIR` - Override local package directory
- `UTPM_CACHE_DIR` - Override cache directory
- `UTPM_CURRENT_DIR` - Override current working directory

## Key Dependencies Explained

- **typst-kit** - Core Typst functionality and package downloading
- **typst-syntax** - Parsing and working with Typst manifests
- **ignore** - File filtering with gitignore-style patterns
- **clap** - CLI argument parsing with derive macros
- **tokio** - Async runtime
- **tracing** - Structured logging and instrumentation
- **octocrab** - GitHub API client
- **inquire** - Interactive CLI prompts
- **semver** - Semantic versioning
- **serde** - Serialization/deserialization

## Best Practices

1. **Always check dry-run mode** before file operations
2. **Use structured logging** with `utpm_log!` macro
3. **Handle errors properly** with `UtpmError` and `Result<T>`
4. **Respect ignore files** when processing directories
5. **Document new configuration** in README and examples
6. **Use async/await** for I/O operations
7. **Check flags** before applying ignore files (typst_ignore, git_ignore, etc.)
8. **Test with different namespaces** - preview vs local have different paths

## Contributing

When contributing to UTPM:

1. Follow existing code patterns and conventions
2. Add tracing instrumentation to new functions
3. Handle dry-run mode in operations that modify files
4. Update documentation (README, examples, copilot-instructions)
5. Test with various configurations and edge cases
6. Use descriptive error messages with `UtpmError`
7. Add CLI documentation strings to command arguments

## Troubleshooting & Debugging

### Debugging Commands

```bash
# Enable trace logging
UTPM_DEBUG=trace utpm prj link

# Use dry-run to see what would happen
utpm --dry-run prj link

# JSON output for programmatic inspection
utpm -o json pkg list

# Override directories for testing
UTPM_DATA_DIR=/tmp/test-data utpm prj link
UTPM_CACHE_DIR=/tmp/test-cache utpm prj link
UTPM_CURRENT_DIR=/path/to/project utpm prj link
```

### Common Error Patterns

```rust
// Missing manifest
Err(UtpmError::Manifest) // Can't find typst.toml

// Package already exists
Err(UtpmError::AlreadyExist(name, version, info))

// Package not found
Err(UtpmError::PackageNotExist)

// Git not available
Err(UtpmError::GitNotFound)

// Invalid package format
Err(UtpmError::PackageNotValid)
```

### File Structure Expectations

```
project/
├── typst.toml          # REQUIRED - Package manifest
├── main.typ            # Default entrypoint (configurable)
├── examples/           # Optional - Example files
│   └── tests.typ
├── .gitignore          # Optional - Honored by link command
├── .typstignore        # Optional - Honored by link command
└── src/                # Optional - Source files
```

## Implementation Notes

### State Management

UTPM uses `OnceLock` and `OnceCell` for global state:

```rust
// In utils/output.rs
pub static OUTPUT_FORMAT: OnceCell<OutputFormat> = OnceCell::new();

// In utils/dryrun.rs
pub static DRYRUN: OnceCell<bool> = OnceCell::new();

// In utils/git.rs
static STRING: OnceLock<Mutex<State>> = OnceLock::new();
```

These are set once in `main.rs` and accessed throughout the application.

### Build-Time Information

The project uses `shadow-rs` for build-time information:

```rust
// In main.rs
use shadow_rs::shadow;
shadow!(build);

// Access build info
build::VERSION
build::BUILD_TIME
build::COMMIT_HASH
```

### Platform-Specific Code

```rust
// Unix symlinks
#[cfg(not(windows))]
pub fn symlink_all(origin: impl AsRef<Path>, new_path: impl AsRef<Path>) -> R<(), std::io::Error> {
    use std::os::unix::fs::symlink;
    symlink(origin, new_path)
}

// Windows symlinks
#[cfg(windows)]
pub fn symlink_all(origin: impl AsRef<Path>, new_path: impl AsRef<Path>) -> R<(), std::io::Error> {
    use std::os::windows::fs::symlink_dir;
    symlink_dir(origin, new_path)
}
```

## Documentation Structure

UTPM provides comprehensive documentation for different audiences:

### User Documentation
- **README.md** - Quick start, features overview, basic usage
- **docs/GUIDE.md** - Complete guide for users, package authors, and contributors
  - Installation instructions for all platforms
  - Detailed usage examples with explanations
  - Best practices for package development
  - Comprehensive FAQ section
  - Troubleshooting guides

### Developer Documentation
- **docs/CONTRIBUTING.md** - Contribution guidelines, code standards, commit conventions
- **docs/DEVELOPMENT.md** - Development workflow, tooling, and daily commands
- **docs/TESTING.md** - Comprehensive testing guide and documentation
- **assets/typst.toml.example** - Example configuration file
- **.github/copilot-instructions.md** - This file, for AI assistants only (not user-facing)

### Maintainer Documentation
- **docs/RELEASING.md** - How to create releases (for contributors)
- **docs/PACKAGING.md** - Technical details about package managers (for developers)
- **docs/PUBLISHING.md** - Publishing guide (mostly automated)
- **docs/SECRETS.md** - GitHub secrets configuration for automation

### GitHub Actions
- **[Thumuss/setup-utpm](https://github.com/Thumuss/setup-utpm)** - Composite action to install UTPM in workflows (separate repository)
- **docs/ACTION.md** - Redirect to setup-utpm repository for action documentation

### Documentation Guidelines

When updating UTPM:
1. Update **README.md** for feature announcements and quick reference
2. Update **docs/GUIDE.md** for detailed user-facing documentation
3. Update **docs/CONTRIBUTING.md** for new code standards or processes
4. Update **docs/DEVELOPMENT.md** for new development tools or workflows
5. Update **docs/TESTING.md** for testing practices and new test categories
6. Update **docs/RELEASING.md** for release process changes
7. Update **docs/PACKAGING.md** for package manager technical changes
8. Update **docs/PUBLISHING.md** for publishing procedures (mostly automated)
9. Update **docs/SECRETS.md** for new secrets or configuration requirements
10. Update **.github/copilot-instructions.md** for technical implementation details (AI context only)
11. Update **assets/typst.toml.example** for new configuration options

**Important**: Never reference `.github/copilot-instructions.md` in user-facing documentation. This file is for AI assistants only.

## Resources

- [Typst Packages Documentation](https://github.com/typst/packages)
- [Typst Universe](https://typst.app/universe)
- [UTPM Repository](https://github.com/typst-community/utpm)
- [Clap Documentation](https://docs.rs/clap/)
- [Ignore Crate Documentation](https://docs.rs/ignore/)
- [Tracing Documentation](https://docs.rs/tracing/)
- [Tokio Documentation](https://docs.rs/tokio/)

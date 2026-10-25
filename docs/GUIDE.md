# UTPM Complete Guide

Welcome to the complete guide for **UTPM** (Unofficial Typst Package Manager)! This guide covers everything from basic usage to advanced development topics.

**Quick Navigation:**
- [For Users](#for-users) - How to use UTPM
- [For Contributors](#for-contributors) - How to contribute code
- [For Package Authors](#for-package-authors) - How to create and publish packages
- [Technical Reference](#technical-reference) - Deep dive into UTPM internals

---

## For Users

### Installation

#### From Binary (Recommended)

Download the latest release from [GitHub Releases](https://github.com/typst-community/utpm/releases).

#### From Source

You'll need Rust installed. Get it from [rustup.rs](https://rustup.rs).

```bash
# Clone the repository
git clone https://github.com/typst-community/utpm.git
cd utpm

# Build and install
cargo install --path .
```

Or, for a development build:

```bash
cargo build --release
cp ./target/release/utpm ~/.cargo/bin/
```

### Basic Usage

UTPM has two main command groups:

1. **`utpm project`** (or `utpm prj`) - Manage your Typst projects
2. **`utpm packages`** (or `utpm pkg`) - Manage installed packages

#### Creating a New Package

```bash
# Create a new package interactively
utpm prj init

# Or use CLI mode (non-interactive)
utpm prj init --cli \
  --name my-package \
  --version 0.1.0 \
  --description "My awesome Typst package"
```

This creates:
- `typst.toml` - Your package manifest
- `src/` directory - For your source files
- `examples/` directory - For example files
- A default entrypoint file (`main.typ`)

#### Creating a Template Package

Template packages additionally contain a set of files that Typst copies into a
new project. The interactive mode asks whether to create a template; in CLI
mode, pass `--template-path` and `--template-entrypoint` (plus an optional
`--template-thumbnail`). Template packages must declare at least one category:

```bash
utpm prj init --cli \
  --name organized-ajp \
  --entrypoint lib.typ \
  --categories thesis \
  --template-path template \
  --template-entrypoint main.typ \
  --template-thumbnail thumbnail.png \
  --populate
```

This creates the `[template]` section in `typst.toml` and scaffolds the
`template/` directory, whose entrypoint imports your package and applies it
with a show rule.

#### Linking Your Package for Development

When you're developing a package, you want to test it in Typst without publishing it first.

```bash
# Link your current package to the local package directory
utpm prj link

# Link with a specific namespace (default is "local")
utpm prj link preview

# Force overwrite if package already exists
utpm prj link --force
```

> **Package locations:** every UTPM operation writes to the local package
> directory (`~/.local/share/typst/packages`, override with
> `TYPST_PACKAGE_PATH`). The package cache (`~/.cache/typst/packages`) belongs
> to the Typst compiler: UTPM never writes to it. Linking to the `@preview`
> namespace prints a warning, since a version that does not exist on the
> registry would shadow the official package.

After linking, you can use your package in Typst:
```typst
#import "@local/my-package:0.1.0": *
```

**What gets copied?** By default, UTPM respects:
- `.gitignore` - Git ignore rules
- `.typstignore` - Typst-specific ignore rules
- `exclude` in `[package]` (`typst.toml`) - Custom exclude patterns

You can control this with flags:
```bash
# Don't respect .gitignore
utpm prj link --no-git-ignore

# Don't respect .typstignore
utpm prj link --no-typst-ignore

# Create a symlink instead of copying files
utpm prj link --no-copy
```

#### Excluding Files from Your Package

Edit your `typst.toml` to exclude files when linking or publishing:

```toml
[package]
name = "my-package"
version = "0.1.0"
exclude = [
  ".git",           # Git directory
  ".github",        # GitHub workflows
  "*.md",           # All Markdown files
  "tests/",         # Test directory
  "examples/",      # Examples directory
]
```

**Pattern syntax:**
- `*` - Matches any characters except `/` (e.g., `*.md` matches `README.md`)
- `**` - Matches any characters including `/` (e.g., `**/*.md` matches `docs/guide.md`)
- `?` - Matches any single character
- `!pattern` - Negates the pattern (includes files that would otherwise be excluded)

#### Cloning Packages from Typst Universe

```bash
# Clone a specific version
utpm prj clone @preview/example:1.0.0

# Clone latest version
utpm prj clone @preview/example
```

Packages from the registry are downloaded into the local package directory;
the package cache stays untouched.

#### Managing Package Versions

```bash
# Bump to a specific version
utpm prj bump 1.2.0

# Bump with semantic versioning
utpm prj bump --major    # 1.0.0 -> 2.0.0
utpm prj bump --minor    # 1.0.0 -> 1.1.0
utpm prj bump --patch    # 1.0.0 -> 1.0.1
```

The bump command updates:
- `typst.toml` version field
- Any other files you specify with `--include` flag

#### Syncing Dependencies

```bash
# Update all dependencies to their latest versions
utpm prj sync

# Check for updates without applying them
utpm prj sync --check
```

#### Listing Installed Packages

```bash
# List all packages
utpm pkg list

# List with tree view
utpm pkg list --tree

# List as JSON
utpm -o json pkg list
```

#### Getting Metadata from Your Package

Extract metadata for use in scripts or CI/CD:

```bash
# Get all metadata
utpm prj metadata

# Get specific field (outputs plain text)
VERSION=$(utpm prj metadata -f version)
NAME=$(utpm prj metadata -f name)
AUTHORS=$(utpm prj metadata -f authors)

# Use in a script
echo "Building $NAME version $VERSION"
git tag "v$VERSION"

# Get as JSON for complex processing
utpm -o json prj metadata | jq '.version'
```

Available fields: `name`, `version`, `entrypoint`, `authors`, `license`, `description`, `repository`, `homepage`, `keywords`, `categories`, `disciplines`, `compiler`, `exclude`

#### Unlinking Packages

```bash
# Remove a package from the local package directory
utpm pkg unlink @local/my-package:0.1.0

# Remove a package from the package cache shared with the Typst compiler
utpm pkg unlink --cache @preview/my-package:0.1.0
```

#### Dry-Run Mode

Before making changes, you can see what would happen:

```bash
# See what would be linked without actually linking
utpm --dry-run prj link

# See what would be bumped
utpm --dry-run prj bump 2.0.0
```

#### Verbose Output

For debugging or understanding what UTPM is doing:

```bash
# Enable trace-level logging
utpm -v trace prj link

# Or set environment variable
UTPM_DEBUG=trace utpm prj link
```

#### Output Formats

UTPM can output in different formats for scripting:

```bash
# JSON output
utpm -o json pkg list

# YAML, TOML, HJSON (experimental, requires manual build)
utpm -o yaml pkg list
```

---

## For Package Authors

### Creating a High-Quality Package

#### 1. Initialize with Good Metadata

```bash
utpm prj init
```

Fill in all the metadata fields:
- **name**: Short, descriptive, lowercase with hyphens
- **version**: Start with `0.1.0`, follow [semantic versioning](https://semver.org)
- **authors**: Your name and email
- **license**: Use a standard license (MIT, Apache-2.0, GPL-3.0, etc.)
- **description**: One-line description of your package
- **repository**: Link to your Git repository
- **keywords**: Help users discover your package
- **categories**: Classify your package

#### 2. Structure Your Package

Recommended structure:
```
my-package/
├── typst.toml              # Package manifest
├── README.md               # Documentation
├── LICENSE                 # License file
├── main.typ                # Main entrypoint
├── src/                    # Additional source files
│   ├── utils.typ
│   └── styles.typ
├── examples/               # Example usage
│   ├── basic.typ
│   └── advanced.typ
└── tests/                  # Tests (not included in package)
    └── test.typ
```

#### 3. Configure File Exclusion

Add to your `typst.toml`:

```toml
[package]
name = "my-package"
version = "0.1.0"
# ... other standard fields
exclude = [
  ".git",
  ".github",
  ".gitignore",
  "*.md",              # Exclude markdown (keep documentation in repository only)
  "tests/",            # Don't include tests in the package
  "examples/",         # Don't include examples in the package
  "*.typ.bak",         # Exclude backup files
  ".vscode/",          # Exclude editor configs
  ".idea/",
]
```

Or keep documentation and examples:
```toml
[package]
name = "my-package"
version = "0.1.0"
exclude = [
  ".git",
  ".github",
  "tests/",
  "*.bak",
]
```

#### 4. Test Locally

```bash
# Link your package
utpm prj link

# Create a test file
cat > test.typ << 'EOF'
#import "@local/my-package:0.1.0": *

// Test your package here
EOF

# Compile with Typst
typst compile test.typ
```

#### 5. Version Your Package

Follow [semantic versioning](https://semver.org):
- **Major** (1.0.0 → 2.0.0): Breaking changes
- **Minor** (1.0.0 → 1.1.0): New features, backward compatible
- **Patch** (1.0.0 → 1.0.1): Bug fixes

```bash
# Bump version
utpm prj bump 0.2.0

# Or use semantic flags
utpm prj bump --patch
```

#### 6. Publish (Coming Soon)

Publishing to Typst Universe is currently in development. For now, share your package via:
- GitHub repository
- Manual installation instructions
- Git-based installation with `utpm pkg install`

---

## For Contributors

### Setting Up Development Environment

#### Prerequisites

1. **Rust** (latest stable): Get from [rustup.rs](https://rustup.rs)
2. **Just** (command runner): `cargo install just`
3. **Git**: For version control

#### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/typst-community/utpm.git
cd utpm

# Install git hooks (optional but recommended)
just setup-hooks
```

### Development Workflow

#### Daily Commands

```bash
# Format your code
just fmt

# Check for compilation errors
just check

# Run tests
just test

# Run linter (Clippy)
just clippy

# Run all checks (format, lint, test) - do this before committing!
just ci

# Auto-fix formatting and linting issues
just fix
```

#### Building and Testing

```bash
# Build in debug mode (faster compilation, slower runtime)
just build

# Build in release mode (slower compilation, optimized binary)
just build-release

# Install locally for testing
just install-local    # Copies to ~/.cargo/bin/utpm

# Run with arguments
just run prj init
```

#### All Available Commands

Run `just --list` to see all commands:
```bash
just --list
```

### Code Standards

#### 1. Formatting

We use `rustfmt` with project-specific configuration in `rustfmt.toml`.

**Rules:**
- Max line width: 100 characters
- Edition: 2024
- Use shorthand for `try!` → `?`
- Reorder imports alphabetically

**Before committing:**
```bash
cargo fmt --all
# Or
just fmt
```

**Check formatting:**
```bash
cargo fmt --all -- --check
# Or
just fmt-check
```

#### 2. Linting

We use Clippy to catch common mistakes and enforce best practices.

**Configuration in `clippy.toml`:**
- Cognitive complexity threshold: 25
- Max line length: 100
- Warn on common mistakes

**Before committing:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Or
just clippy
```

**Auto-fix issues:**
```bash
cargo clippy --fix --allow-dirty
# Or
just clippy-fix
```

#### 3. Testing

All new features must include tests.

**Run tests:**
```bash
cargo test --all-features
# Or
just test
```

**Run with output:**
```bash
cargo test --all-features -- --nocapture
# Or
just test-verbose
```

**For detailed testing documentation**, see [**Testing Guide**](TESTING.md):
- 60+ tests covering utils, commands, and integration workflows
- Test helpers and utilities
- Running specific test suites
- Writing new tests
- Test coverage and debugging

### Git Workflow

#### Commit Messages

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
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```bash
git commit -m "feat(metadata): add field extraction with -f flag"
git commit -m "fix(link): respect typst_ignore flag when adding .typstignore"
git commit -m "docs: update README with metadata command examples"
```


#### Git Hooks

The project includes optional pre-commit hooks that run checks before each commit.

**Install:**
```bash
just setup-hooks
```

**What the hooks do:**
1. Check code formatting
2. Run Clippy linter
3. Run all tests

If any check fails, the commit is blocked until you fix it.

**Remove hooks:**
```bash
just remove-hooks
```

### Project Architecture

UTPM is structured as follows:

```
src/
├── main.rs                 # Entry point, CLI setup, logging
├── commands.rs             # CLI argument definitions
├── utils.rs                # Utility module aggregator
├── commands/               # Command implementations
│   ├── init.rs            # Create new packages
│   ├── link.rs            # Link packages for development
│   ├── unlink.rs          # Remove linked packages
│   ├── clone.rs           # Clone from Typst Universe
│   ├── publish.rs         # Publish to Universe (WIP)
│   ├── bump.rs            # Version bumping
│   ├── sync.rs            # Dependency syncing
│   ├── metadata.rs        # Metadata extraction
│   ├── install.rs         # Install from git repos
│   ├── get.rs             # Get package info
│   ├── list.rs            # List packages
│   ├── package_path.rs    # Show package paths
│   └── generate.rs        # Generate shell completions
└── utils/                  # Utility modules
    ├── dryrun.rs          # Dry-run mode support
    ├── git.rs             # Git operations
    ├── macros.rs          # Custom macros
    ├── output.rs          # Output format handling
    ├── paths.rs           # Path utilities
    ├── specs.rs           # Configuration parsing
    └── state.rs           # Error types
```

#### Key Dependencies

- **clap** (4.5.39): CLI argument parsing with derive macros
- **typst-kit** (0.13.1): Core Typst functionality
- **typst-syntax** (0.13.1): Parsing Typst manifests
- **tokio** (1.45.1): Async runtime
- **ignore** (0.4.23): File filtering with gitignore patterns
- **serde** (1.0): Serialization/deserialization
- **semver** (1.0.26): Semantic versioning
- **tracing** (0.1.41): Structured logging

### Adding a New Command

#### Step 1: Create Command File

Create `src/commands/mycommand.rs`:

```rust
use crate::utils::*;
use tracing::instrument;

/// Execute the mycommand command
#[instrument(skip(cmd))]
pub async fn run(cmd: &MyCommandArgs) -> Result<bool> {
    utpm_log!(trace, "executing mycommand");
    
    // Your implementation here
    
    utpm_log!(info, "Command completed successfully");
    Ok(true)
}
```

#### Step 2: Define Arguments

Add to `src/commands.rs`:

```rust
pub mod mycommand;

#[derive(Parser, Clone, Debug, PartialEq)]
pub struct MyCommandArgs {
    /// Description of the argument
    #[arg(short, long)]
    pub my_arg: Option<String>,
}
```

#### Step 3: Add to Command Enum

In `src/commands.rs`, add to the appropriate enum:

```rust
#[derive(Subcommand, Clone, Debug, PartialEq)]
pub enum ProjectArgs {
    // ... existing commands ...
    
    /// My new command
    MyCommand(MyCommandArgs),
}
```

#### Step 4: Wire Up Dispatcher

In `src/main.rs`, add to the match statement:

```rust
match commands {
    // ... existing commands ...
    Commands::Project(ProjectArgs::MyCommand(cmd)) => {
        commands::mycommand::run(&cmd).await
    },
}
```

#### Step 5: Write Tests

Add tests to your command file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mycommand() {
        // Your tests here
    }
}
```

### Code Patterns

#### Error Handling

Use the `utpm_bail!` macro for errors:

```rust
use crate::utils::*;

// Simple error
utpm_bail!(Manifest);

// Error with arguments
utpm_bail!(AlreadyExist, name, version, "Additional info");
```

#### Logging

Use the `utpm_log!` macro for all output:

```rust
// Simple message
utpm_log!(info, "Package linked successfully");

// With structured data
utpm_log!(trace, "Processing file", "path" => &path, "size" => size);

// With format strings
utpm_log!(info, "Found {} packages", count);
```

#### Dry-Run Mode

Always check dry-run before file operations:

```rust
use crate::utils::dryrun::get_dry_run;

if !get_dry_run() {
    // Only perform actual operations if not in dry-run mode
    fs::write(&path, content)?;
}

// Always log success, even in dry-run
utpm_log!(info, "File written successfully");
```

#### Async Functions

All command functions must be async:

```rust
#[instrument(skip(cmd))]
pub async fn run(cmd: &CommandArgs) -> Result<bool> {
    // Implementation
    Ok(true)
}
```

### Debugging Tips

#### Enable Trace Logging

```bash
# Via flag
utpm -v trace prj link

# Via environment variable
UTPM_DEBUG=trace utpm prj link
```

#### Use Dry-Run Mode

```bash
utpm --dry-run prj link
```

#### Override Directories

```bash
# Test with custom directories
TYPST_PACKAGE_PATH=/tmp/test-data utpm prj link
TYPST_PACKAGE_CACHE_PATH=/tmp/test-cache utpm pkg unlink --cache @preview/example:1.0.0
```

#### JSON Output for Debugging

```bash
utpm -o json pkg list | jq '.'
```

### Common Issues

#### "Format check failed"

```bash
# Fix:
just fmt
```

#### "Clippy warnings"

```bash
# Fix:
just clippy-fix
```

#### "Tests failed"

```bash
# Run with output to see details:
just test-verbose
```

#### "Git hooks blocking commit"

If hooks are too strict for your workflow:
```bash
# Temporarily bypass (not recommended)
git commit --no-verify

# Or remove hooks
just remove-hooks
```

---

## Technical Reference

### Package Locations

UTPM manages packages in two locations. Every UTPM operation writes to the
local package directory; the package cache belongs to the Typst compiler and
is only read by `pkg list --all`, and can only be removed from with
`pkg unlink --cache`.

#### 1. Local Packages

**Path**: `{TYPST_PACKAGE_PATH}/{namespace}/{name}/{version}/`

**Default locations:**
- Linux: `~/.local/share/typst/packages/`
- macOS: `~/Library/Application Support/typst/packages/`
- Windows: `%APPDATA%\typst\packages\`

**Override**: Set `TYPST_PACKAGE_PATH` environment variable

**Used for:**
- Packages created with `utpm prj init`
- Packages linked with `utpm prj link`
- Custom namespaces (usually `local`)
- Packages downloaded by `utpm prj clone` (including `@preview`)

#### 2. Cache Packages (Typst compiler)

**Path**: `{TYPST_PACKAGE_CACHE_PATH}/{namespace}/{name}/{version}/`

**Default locations:**
- Linux: `~/.cache/typst/packages/`
- macOS: `~/Library/Caches/typst/packages/`
- Windows: `%LOCALAPPDATA%\typst\packages\`

**Override**: Set `TYPST_PACKAGE_CACHE_PATH` environment variable

**Used for:**
- Packages downloaded by the Typst compiler itself
- UTPM never writes to this directory; `pkg unlink --cache` is the only
  command that can remove packages from it

### File Filtering System

UTPM uses the `ignore` crate to filter files when linking or publishing packages.

#### Ignore Files Supported

1. **`.gitignore`** - Git ignore patterns (enabled by default)
2. **`.typstignore`** - Typst-specific ignore (enabled by default)
3. **`.ignore`** - Generic ignore file (disabled by default, enable with `-i`)
4. **Global `.gitignore`** - User's global git ignore (enabled by default)
5. **`.git/info/exclude`** - Git's local exclude file (enabled by default)
6. **`exclude` in `[package]`** - Patterns in `typst.toml`

#### Controlling Ignore Files

```bash
# Disable .gitignore
utpm prj link --no-git-ignore

# Disable .typstignore
utpm prj link --no-typst-ignore

# Enable .ignore file
utpm prj link --ignore

# Disable global .gitignore
utpm prj link --no-git-global-ignore

# Disable .git/info/exclude
utpm prj link --no-git-exclude
```

#### Pattern Syntax

The patterns follow standard glob syntax:

- `*` - Match any characters except `/`
  - Example: `*.md` matches `README.md` but not `docs/README.md`

- `**` - Match any characters including `/`
  - Example: `**/*.md` matches `README.md` and `docs/README.md`

- `?` - Match exactly one character
  - Example: `file?.txt` matches `file1.txt`, `fileA.txt`

- `[abc]` - Match any character in brackets
  - Example: `file[0-9].txt` matches `file0.txt` through `file9.txt`

- `!` - Negate pattern (include files that would be excluded)
  - Example: `!important.md` includes `important.md` even if `*.md` is excluded

- `/` at start - Match from root of package
  - Example: `/tests/` matches only `tests/` at root, not `src/tests/`

- `/` at end - Only match directories
  - Example: `build/` matches directory `build/` but not file `build`

#### Priority Order

When a file matches multiple patterns, UTPM applies them in this order:

1. `exclude` patterns in `[package]` of `typst.toml`
2. `.typstignore` (if enabled)
3. `.gitignore` (if enabled)
4. `.ignore` (if enabled)
5. Global `.gitignore` (if enabled)
6. `.git/info/exclude` (if enabled)

Later patterns can override earlier ones using `!` negation.

### Environment Variables

#### Logging

- `UTPM_DEBUG` - Set log level: `trace`, `debug`, `info`, `warn`, `error`
  ```bash
  UTPM_DEBUG=trace utpm prj link
  ```

#### Paths

- `TYPST_PACKAGE_PATH` - Override the local package directory (used by every
  UTPM operation, including `@preview` downloads and links)
  ```bash
  TYPST_PACKAGE_PATH=/custom/path utpm prj link
  ```

- `TYPST_PACKAGE_CACHE_PATH` - Override the package cache (the Typst compiler
  directory; UTPM never writes to it, `pkg unlink --cache` targets it)
  ```bash
  TYPST_PACKAGE_CACHE_PATH=/custom/cache utpm pkg unlink --cache @preview/example:1.0.0
  ```

- `UTPM_CURRENT_DIR` - Override current working directory
  ```bash
  UTPM_CURRENT_DIR=/path/to/project utpm prj link
  ```

### Output Formats

UTPM supports multiple output formats for scripting and automation.

#### Text (Default)

Human-readable output:
```bash
utpm pkg list
```

#### JSON

Machine-readable JSON:
```bash
utpm -o json pkg list
```

Use with `jq` for processing:
```bash
utpm -o json pkg list | jq '.[] | select(.namespace == "local")'
```

#### YAML, TOML, HJSON (Experimental)

These formats are experimental and require manual build:

```bash
# Enable features
cargo build --release --features yaml,toml,hjson

# Use
utpm -o yaml pkg list
utpm -o toml pkg list
utpm -o hjson pkg list
```

### Package Manifest Format

UTPM uses Typst's standard `typst.toml` format. File exclusion uses the standard `exclude` field of the `[package]` section.

#### Minimal Example

```toml
[package]
name = "my-package"
version = "0.1.0"
entrypoint = "main.typ"
authors = ["Your Name <you@example.com>"]
license = "MIT"
description = "A short description"
```

#### Complete Example

```toml
[package]
name = "my-awesome-package"
version = "1.0.0"
entrypoint = "main.typ"
authors = [
  "First Author <first@example.com>",
  "Second Author <second@example.com>"
]
license = "MIT"
description = "A comprehensive package for awesome things"
homepage = "https://example.com/my-awesome-package"
repository = "https://github.com/username/my-awesome-package"
keywords = ["awesome", "package", "typst"]
categories = ["layout", "visualization"]
disciplines = ["mathematics", "engineering"]
compiler = "0.12.0"

[template]
path = "template"
entrypoint = "main.typ"
thumbnail = "thumbnail.png"

exclude = [
  ".git",
  ".github",
  "*.md",
  "tests/",
  "examples/",
]
```

#### Field Reference

**Required fields:**
- `name` - Package name (lowercase, hyphens allowed)
- `version` - Semantic version (e.g., "1.0.0")
- `entrypoint` - Main file (usually "main.typ")

**Recommended fields:**
- `authors` - List of authors with optional email
- `license` - SPDX license identifier
- `description` - One-line description

**Optional fields:**
- `homepage` - Package homepage URL
- `repository` - Source repository URL
- `keywords` - Search keywords (array of strings)
- `categories` - Package categories (array of strings)
- `disciplines` - Academic disciplines (array of strings)
- `compiler` - Minimum Typst version required

**Template fields** (for templates):
- `template.path` - Path to template directory
- `template.entrypoint` - Template entrypoint file
- `template.thumbnail` - Preview image

**UTPM fields:**
- `exclude` - Files to exclude when linking/publishing (in `[package]`)

### Semantic Versioning

UTPM follows [Semantic Versioning 2.0.0](https://semver.org).

Version format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality, backward compatible
- **PATCH**: Bug fixes, backward compatible

Examples:
- `1.0.0` → `2.0.0`: Breaking change
- `1.0.0` → `1.1.0`: New feature
- `1.0.0` → `1.0.1`: Bug fix

Use `utpm prj bump` to update versions:
```bash
# Specific version
utpm prj bump 1.2.3

# Semantic increment
utpm prj bump --major    # 1.0.0 → 2.0.0
utpm prj bump --minor    # 1.0.0 → 1.1.0
utpm prj bump --patch    # 1.0.0 → 1.0.1
```

---

## FAQ

### General Questions

**Q: What is the difference between `utpm prj link` and `utpm prj clone`?**

A: `link` is for local development - it copies (or symlinks) your current project to the local package directory. `clone` downloads a package from Typst Universe to your cache directory.

**Q: Where are packages stored?**

A: Local packages (from `link`) go to `~/.local/share/typst/packages/` on Linux. Downloaded packages (from `clone`) go to `~/.cache/typst/packages/`. See [Package Locations](#package-locations).

**Q: Can I use UTPM without Rust installed?**

A: Yes, if you download a pre-built binary from the releases page. You only need Rust to build from source.

**Q: Does UTPM work on Windows?**

A: Yes! UTPM supports Linux, macOS, and Windows.

### Package Development

**Q: Why are my files not being linked?**

A: Check your `.gitignore`, `.typstignore`, and `exclude` patterns in `[package]`. Use `--dry-run` to see what would be copied:
```bash
utpm --dry-run prj link
```

**Q: How do I test my package before publishing?**

A: Use `utpm prj link` to link it locally, then import it in a test Typst file:
```typst
#import "@local/my-package:0.1.0": *
```

**Q: Should I include examples and tests in my package?**

A: It depends. Examples can help users understand your package, but they increase package size. Tests should generally be excluded. Configure via the `exclude` field in `[package]`.

**Q: How do I version my package?**

A: Follow semantic versioning. Use `utpm prj bump` to update versions consistently across files.

### Contributing

**Q: Do I need to install git hooks?**

A: No, they're optional. But they help catch issues before pushing to GitHub. Install with `just setup-hooks`.

**Q: My pre-commit hook is failing, what do I do?**

A: Run the checks manually to see the errors:
```bash
just ci
```
Then fix the issues and commit again.

**Q: How do I run just one test?**

A:
```bash
cargo test test_name
```

**Q: Can I contribute if I'm new to Rust?**

A: Absolutely! Start with documentation improvements, bug reports, or small features. Check the issues labeled "good first issue" on GitHub.

### Troubleshooting

**Q: `utpm: command not found`**

A: Make sure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Q: Permission denied when installing**

A: Don't use `sudo` with cargo. Instead:
```bash
cargo install --path .
```

**Q: UTPM is linking files I want to exclude**

A: Check your patterns in the `exclude` field of `[package]`. Remember:
- Patterns are relative to package root
- Use `/` at the end for directories: `tests/`
- Use `**` for recursive: `**/*.bak`

**Q: How do I reset everything and start fresh?**

A:
```bash
# Remove local packages
rm -rf ~/.local/share/typst/packages/local/my-package

# Remove cache
rm -rf ~/.cache/typst/packages/

# Unlink a specific package
utpm pkg unlink @local/my-package:0.1.0
```

---

## Getting Help

- **Documentation**: You're reading it!
- **Issues**: [GitHub Issues](https://github.com/typst-community/utpm/issues)
- **Discussions**: [GitHub Discussions](https://github.com/typst-community/utpm/discussions)
- **Source Code**: [GitHub Repository](https://github.com/typst-community/utpm)

---

## License

UTPM is licensed under the MIT License. See [LICENSE](../LICENSE) for details.

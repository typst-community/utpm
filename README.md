<div align="center">

![UTPM logo](./assets/logo.svg)

> _Unofficial Typst Package Manager_

**UTPM** is a powerful command-line package manager for [Typst](https://typst.app/). Create, manage, and share Typst packages with ease — whether for local development or publishing to **Typst Universe**.

[![typst-community - utpm](https://img.shields.io/static/v1?label=typst-community&message=utpm&color=blue&logo=github)](https://github.com/typst-community/utpm "Go to GitHub repo")
[![stars - utpm](https://img.shields.io/github/stars/typst-community/utpm?style=social)](https://github.com/typst-community/utpm)
[![forks - utpm](https://img.shields.io/github/forks/typst-community/utpm?style=social)](https://github.com/typst-community/utpm)
<br/>
[![GitHub tag](https://img.shields.io/github/tag/typst-community/utpm?include_prereleases=&sort=semver&color=blue)](https://github.com/typst-community/utpm/releases/)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)
[![issues - utpm](https://img.shields.io/github/issues/typst-community/utpm)](https://github.com/typst-community/utpm/issues)

</div>

---

## ✨ Quick Start

```bash
# Install UTPM (see [Installation](#-installation) for all options)
cargo install utpm

# Create a new package
utpm prj init

# Link it for local development
utpm prj link

# Use it in Typst!
# #import "@local/my-package:0.1.0": *
```

> [!NOTE]
> **UTPM** is actively developed and growing! Some features are still in progress. \
> Contributions are welcome — check out our [contributing guide](docs/CONTRIBUTING.md)!


## 🔥 Why UTPM?

- **🚀 Rapid Development** - Create and link packages instantly for local testing
- **📦 Smart File Management** - Respects `.gitignore`, `.typstignore`, and custom exclude patterns
- **🔄 Dependency Management** - Sync dependencies and bump versions with ease
- **📊 Metadata Extraction** - Extract package info for scripts and CI/CD pipelines
- **🎨 Flexible Output** - JSON, HJSON, YAML, TOML, or human-readable text
- **🛡️ Try Features without Risk** - Dry-run mode for all destructive operations
- **⚡ Fast & Lightweight** - Written in Rust for speed and reliability

## 🎯 Features

### Package Development
- ✨ **Initialize** packages with interactive prompts (`utpm prj init`)
- 🔗 **Link** packages for local development (`utpm prj link`)
- ⬆️ **Bump** versions with semantic versioning (`utpm prj bump`)
- 📋 **Extract metadata** for automation (`utpm prj metadata`)

### Dependency Management
- 📥 **Clone** packages from Typst Universe (`utpm prj clone`)
- 🔄 **Sync** dependencies to latest versions (`utpm prj sync`)
- 📦 **Install** from git repositories (`utpm pkg install`)

### Package Discovery
- 🗃️ **List** local packages with tree view (`utpm pkg list --tree`)
- ℹ️ **Get** package info from remote (`utpm pkg get`)
- 🔍 **Check** for updates without applying (`utpm prj sync -c`)

### Coming Soon
- 🚀 **Publish** directly to Typst Universe (in development)


---

## 📦 Installation

### Quick Install

#### Cargo (Cross-platform)

```bash
# Using cargo-binstall (fastest)
cargo binstall utpm

# Or build from source
cargo install utpm
```

#### Arch Linux

```bash
# Using an AUR helper (yay, paru, etc.)
yay -S utpm-bin      # Pre-built binaries
# or
yay -S utpm-git      # Build from latest git
```

#### Homebrew (macOS/Linux)

```bash
# Coming soon - waiting for tap approval
brew install typst-community/utpm/utpm
```

#### Nix

<details>
<summary>📦 Nix Installation Options</summary>

##### With Flakes

Temporary shell:
```bash
nix shell github:typst-community/utpm
```

Permanent installation in `flake.nix`:
```nix
{
  inputs.utpm.url = "github:typst-community/utpm";
  
  outputs = { self, nixpkgs, ... }@inputs: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [{
        environment.systemPackages = [ 
          inputs.utpm.packages.${system}.default 
        ];
      }];
    };
  };
}
```

##### Without Flakes

```bash
git clone https://github.com/typst-community/utpm.git
cd utpm
nix-build
./result/bin/utpm
```

</details>

### More Package Managers

UTPM is being packaged for multiple package managers:

- 📦 **Snap** - `sudo snap install utpm` (automated publishing)
- 📦 **Flatpak** - Available on Flathub (semi-automated)
- 📦 **Debian/Ubuntu** - `.deb` packages (manual, see [docs/PUBLISHING.md](docs/PUBLISHING.md))
- 📦 **Fedora/RHEL** - RPM packages (manual, see [docs/PUBLISHING.md](docs/PUBLISHING.md))
- 📦 **Arch/AUR** - AUR packages (manual, see [docs/PUBLISHING.md](docs/PUBLISHING.md))
<!-- - 📦 **Homebrew (mac)** -  (manual, see [docs/PUBLISHING.md](docs/PUBLISHING.md)) 
For now I don't want to publish on it but it will comes 
 -->

**For maintainers:** See [docs/PACKAGING.md](docs/PACKAGING.md) for technical details and [docs/SECRETS.md](docs/SECRETS.md) for automation setup.

**Note:** Windows builds are generated here but scoop is managed by an another user. See [Scoop bucket for typst](https://github.com/typst-community/scoop-bucket) for more infos.

### From Source

```bash
git clone https://github.com/typst-community/utpm.git
cd utpm
cargo install --path .
```

---

## 🚀 Usage

### GitHub Actions

Use UTPM in your CI/CD workflows:

```yaml
- name: Setup UTPM
  uses: Thumuss/setup-utpm@v1
```

See the [setup-utpm repository](https://github.com/Thumuss/setup-utpm) for complete documentation and examples.

### Basic Commands

```bash
# Create a new package
utpm prj init

# Link for local development
utpm prj link

# Clone from Typst Universe
utpm prj clone @preview/example:1.0.0

# Bump version
utpm prj bump 1.2.0

# List installed packages
utpm pkg list --tree

# Get metadata for scripts
VERSION=$(utpm prj metadata -f version)
```

### Command Overview

#### Project Management (`utpm prj`)

| Command | Alias | Description |
|---------|-------|-------------|
| `init` | `n` | Create a new `typst.toml` manifest interactively |
| `link` | `l` | Link package for local development (respects ignore files) |
| `clone` | `c` | Clone a package from Typst Universe |
| `bump` | `b` | Bump package version (supports semantic versioning) |
| `sync` | `s` | Sync dependencies to latest versions |
| `metadata` | `m` | Extract metadata for scripts and automation |
| `publish` | `p` | 🚧 Publish to Typst Universe _(coming soon)_ |

#### Package Management (`utpm pkg`)

| Command | Alias | Description |
|---------|-------|-------------|
| `list` | `l` | List installed packages (supports tree view) |
| `path` | `p` | Show package directory path |
| `unlink` | `u` | Remove a linked package |
| `get` | `g` | Get package info from remote |
| `install` | `i` | Install package from git repository |

#### Other Commands

- `utpm generate` (`g`) - Generate shell completion scripts

### Global Options

```bash
utpm [OPTIONS] <COMMAND>

Options:
  -v, --verbose <LEVEL>     Logging level (trace, debug, info, warn, error)
  -o, --output <FORMAT>     Output format (text, json, yaml, toml, hjson)
  -D, --dry-run             Preview changes without writing to disk
  -h, --help                Show help information
  -V, --version             Show version
```

> 💡 **Tip**: Use `utpm <command> --help` for detailed command-specific help

---

## ⚙️ Configuration

UTPM extends the standard `typst.toml` with a `[tool.utpm]` section for package-specific settings.

### Excluding Files

Control which files are included when linking or publishing:

```toml
[package]
name = "my-package"
version = "0.1.0"
# ... other standard fields

[tool.utpm]
exclude = [
  ".git",
  ".github",
  "*.md",           # Exclude all Markdown files
  "tests/",         # Exclude tests directory
  "examples/",      # Exclude examples
  "**/*.bak",       # Exclude backup files recursively
]
```

**Pattern Syntax:**
- `*` - Match files in current directory (e.g., `*.md`)
- `**` - Match recursively (e.g., `**/*.tmp`)
- `!pattern` - Negate/include pattern
- Patterns ending with `/` match directories only

**Ignore Files Respected:**
- `.gitignore` (default: enabled)
- `.typstignore` (default: enabled)
- `.ignore` (optional, enable with `--ignore`)
- Custom patterns in `[tool.utpm]`

### Metadata Extraction

Extract package metadata for scripts and CI/CD:

```bash
# Get specific field (outputs plain text)
VERSION=$(utpm prj metadata -f version)
NAME=$(utpm prj metadata -f name)

# Use in automation
echo "Building $NAME version $VERSION"
git tag "v$VERSION"

# Get all metadata as JSON
utpm -o json prj metadata | jq '.authors'
```

**Available fields:** `name`, `version`, `entrypoint`, `authors`, `license`, `description`, `repository`, `homepage`, `keywords`, `categories`, `disciplines`, `compiler`, `exclude`

📄 **Example**: See [`assets/typst.toml.example`](assets/typst.toml.example) for a complete configuration reference.

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| **[📘 Complete Guide](docs/GUIDE.md)** | Comprehensive guide for users, package authors, and contributors |
| **[🤝 Contributing](docs/CONTRIBUTING.md)** | Code standards, testing, and contribution process |
| **[🛠️ Development](docs/DEVELOPMENT.md)** | Development setup, workflow, and tools |
| **[🧪 Testing](docs/TESTING.md)** | Testing guide, running tests, and writing new tests |

---

## 🤝 Contributing

We welcome contributions of all kinds! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

### Quick Start for Contributors

```bash
# 1. Install development tools
cargo install just

# 2. Setup git hooks (optional but recommended)
just setup-hooks

# 3. Before committing
just fix    # Auto-format and fix linting issues
just ci     # Run all checks (format, lint, test)
```

### What You Can Do

- 🐛 **Report bugs** - Open an issue with details
- 💡 **Suggest features** - Share your ideas in discussions
- 📝 **Improve docs** - Help make documentation clearer
- 🔧 **Fix issues** - Pick up a "good first issue"
- ✨ **Add features** - Implement new functionality

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for detailed guidelines on code standards, testing, and the PR process.

---

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

Built with ❤️ by the [Typst Community](https://github.com/typst-community)

**Key Dependencies:**
- [Typst](https://typst.app/) - The amazing typesetting system
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Clap](https://github.com/clap-rs/clap) - Command-line argument parsing

---

<div align="center">

**[⬆ Back to Top](#)**

Made with ❤️ for the Typst community

</div>

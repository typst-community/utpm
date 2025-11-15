# Setup UTPM Action

GitHub Action to install and configure UTPM (Unofficial Typst Package Manager) in your workflows.

See the repo on **[github.com/Thumuss/setup-utpm](https://github.com/Thumuss/setup-utpm)**


## Features

- ✅ Cross-platform support (Linux x86_64/aarch64, macOS x86_64/arm64)
- ✅ Version pinning or latest release
- ✅ Built-in caching for faster workflows
- ✅ Automatic platform detection
- ✅ Simple one-line setup

## Usage

### Basic Usage

```yaml
- name: Setup UTPM
  uses: Thumuss/utpm@v0.3.0
```

### Specify Version

```yaml
- name: Setup UTPM
  uses: Thumuss/utpm@v0.3.0
  with:
    version: '0.3.0'
```

### With GitHub Token (Recommended)

```yaml
- name: Setup UTPM
  uses: Thumuss/utpm@v0.3.0
  with:
    version: 'latest'
    token: ${{ secrets.GITHUB_TOKEN }}
```

## Inputs

| Name | Description | Required | Default |
|------|-------------|----------|---------|
| `version` | Version of UTPM to install (e.g., "0.3.0" or "latest") | No | `latest` |
| `token` | GitHub token for API requests (to avoid rate limiting) | No | `${{ github.token }}` |

## Outputs

| Name | Description |
|------|-------------|
| `version` | The installed version of UTPM |
| `cache-hit` | Whether the installation was restored from cache |

## Examples

### Use UTPM in a Typst project

```yaml
name: Build Typst Document

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup UTPM
        uses: Thumuss/utpm@v0.3.0
      
      - name: Link local package
        run: utpm prj link
      
      - name: Install Typst
        uses: Thumuss/setup-typst@v3
      
      - name: Compile document
        run: typst compile main.typ
```

### Test package before publishing

```yaml
name: Test Package

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup UTPM
        uses: Thumuss/utpm@v0.3.0
        with:
          version: 'latest'
      
      - name: Link package
        run: utpm prj link --dry-run
      
      - name: Validate package
        run: utpm prj metadata
```

### Multi-platform testing

```yaml
name: Multi-Platform Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, macos-14, windows-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup UTPM
        uses: Thumuss/utpm@v0.3.0
      
      - name: Test UTPM
        run: |
          utpm --version
          utpm prj metadata -f name
```

## Supported Platforms

All platforms tested in CI:

- ✅ Linux x86_64 (ubuntu-latest)
- ✅ Linux aarch64 (via cross-compilation)
- ✅ macOS x86_64 (macos-latest - Intel)
- ✅ macOS arm64 (macos-14 - Apple Silicon)
- ✅ Windows x86_64 (windows-latest)

**Note:** The action automatically detects your runner's platform and downloads the appropriate binary.

## Caching

The action automatically caches the UTPM binary based on platform and version. This significantly speeds up subsequent workflow runs.

Cache key format: `utpm-{platform}-{version}`

## Troubleshooting

### Rate Limiting

If you encounter GitHub API rate limiting, provide a token:

```yaml
- uses: Thumuss/utpm@v0.3.0
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

### Unsupported Platform

If you see "Unsupported OS" or "Unsupported architecture", check that your runner is one of the supported platforms listed above.

### Permission Denied

Ensure the action has permission to write to `~/.local/bin`. This is the default installation location.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for contribution guidelines.

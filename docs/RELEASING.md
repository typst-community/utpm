# Release Automation

UTPM uses GitHub Actions for fully automated multi-platform releases.

## Quick Start

To create a new release:

```bash
# Update version in Cargo.toml first
git add Cargo.toml
git commit -m "chore: bump version to 0.3.0"
git push

# Create and push the tag
git tag v0.3.0
git push --tags
```

That's it! The GitHub Actions workflow automatically:

1. ✅ Builds binaries for 6 platforms
2. ✅ Generates shell completions
3. ✅ Creates release archives
4. ✅ Calculates SHA256 checksums
5. ✅ Creates GitHub Release
6. ✅ Uploads all artifacts

## What Gets Built

The workflow builds for these platforms:

| Platform | Target | Archive Format |
|----------|--------|----------------|
| Linux x86_64 (glibc) | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux aarch64 (glibc) | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| Linux x86_64 (musl) | `x86_64-unknown-linux-musl` | `.tar.gz` |
| macOS x86_64 | `x86_64-apple-darwin` | `.tar.gz` |
| macOS aarch64 | `aarch64-apple-darwin` | `.tar.gz` |

**Note:** Windows builds are handled separately by another maintainer.

Each archive contains:
- Binary (`utpm` or `utpm.exe`)
- Shell completions (bash, fish, zsh)
- `README.md`
- `LICENSE`

## Manual Workflow Dispatch

You can also trigger releases manually without a tag:

1. Go to GitHub Actions → Release workflow
2. Click "Run workflow"
3. Enter version (e.g., `0.3.0`)
4. Click "Run"

## After Release

After a successful release:

1. Check the [Releases page](https://github.com/typst-community/utpm/releases) for all artifacts
2. Download checksums are automatically calculated (SHA256)
3. Users can download pre-built binaries for their platform
4. Update package manager repositories (see [PACKAGING.md](PACKAGING.md)) if needed

## Package Manager Integration

Once released, binaries can be used for package managers:

- **AUR** - Reference GitHub release URLs in PKGBUILD
- **Homebrew** - Update formula with new version and checksums
- **Snap/Flatpak** - Build from tagged release
- **Debian/Fedora** - Update version numbers in packaging files

See [PACKAGING.md](PACKAGING.md) for technical details and [PUBLISHING.md](PUBLISHING.md) for step-by-step publishing instructions.

## Troubleshooting

**Build fails for a specific platform:**
- Check the GitHub Actions logs for that platform
- Common issues: missing cross-compilation tools, target not installed
- The workflow will continue building other platforms even if one fails

**Release doesn't trigger:**
- Ensure tag starts with `v` (e.g., `v0.3.0` not `0.3.0`)
- Check that tag was pushed to GitHub: `git push --tags`
- Verify GitHub Actions is enabled in repository settings

**Checksums don't match:**
- Always use checksums from the GitHub Release artifacts
- Never calculate checksums from locally built binaries
- The workflow calculates official checksums automatically

## Continuous Integration

The repository also has a CI workflow (`.github/workflows/ci.yml`) that runs on every push/PR:

- ✅ Format check (`cargo fmt`)
- ✅ Lint check (`cargo clippy`)
- ✅ Tests (`cargo test`)
- ✅ Documentation build (`cargo doc`)

Make sure CI passes before creating a release.

## Version Workflow

Recommended workflow for releases:

1. **Update version**: Edit `Cargo.toml`
2. **Update changelog**: Document changes
3. **Commit**: `git commit -m "chore: bump version to X.Y.Z"`
4. **Push**: `git push`
5. **Wait for CI**: Ensure tests pass
6. **Tag**: `git tag vX.Y.Z`
7. **Push tag**: `git push --tags`
8. **Wait for release**: GitHub Actions builds and publishes

## Files Involved

- `.github/workflows/release.yml` - Main release automation
- `.github/workflows/ci.yml` - Continuous integration
- `Cargo.toml` - Version source of truth
- `docs/PACKAGING.md` - Package manager distribution guide

---

For questions or issues with the release process, open an issue on GitHub.

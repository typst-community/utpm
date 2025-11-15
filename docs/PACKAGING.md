# Package Distribution Guide

This guide explains how to distribute UTPM through various package managers and how the automated release process works.

## Table of Contents

- [Automated Release Process](#automated-release-process)
- [Package Managers](#package-managers)
  - [Arch Linux (AUR)](#arch-linux-aur)
  - [Homebrew (macOS/Linux)](#homebrew-macoslinux)
  - [Debian/Ubuntu](#debianubuntu)
  - [Fedora/RHEL](#fedorarhel)
  - [Snap](#snap)
  - [Flatpak](#flatpak)
- [Manual Release Process](#manual-release-process)
- [Testing Packages](#testing-packages)

## Automated Release Process

UTPM uses GitHub Actions to automate the entire release process. When you push a version tag or manually trigger the workflow, it:

1. **Builds binaries** for 5 platforms (Linux x86_64/aarch64/musl, macOS x86_64/aarch64)
2. **Generates shell completions** (bash, fish, zsh) for each platform
3. **Creates release archives** (`.tar.gz` format)
4. **Creates GitHub Release** with all artifacts attached
5. **Calculates SHA256 checksums** for all binaries
6. **Updates package manager files** with new versions/checksums
7. **Creates a Pull Request** with all updated files

**Note:** Windows builds are handled separately by another maintainer.

**For maintainers:** See [PUBLISHING.md](PUBLISHING.md) for step-by-step instructions on how to publish to each package manager after a release.

### Triggering a Release

**Method 1: Git Tag (Recommended)**
```bash
# Create and push a version tag
git tag v0.3.0
git push --tags

# The workflow automatically triggers and creates the release
```

**Method 2: Manual Workflow Dispatch**
1. Go to GitHub Actions → Release workflow
2. Click "Run workflow"
3. Enter the version (e.g., `0.3.0`)
4. Click "Run"

### What Happens After Release

1. GitHub Actions builds all binaries (~5-10 minutes)
2. A GitHub Release is created with all artifacts
3. A Pull Request is opened with updated package manager files
4. Review and merge the PR to update checksums in the repository
5. Package maintainers can use the updated files to publish

## Package Managers

All package manager configuration files are in the `packaging/` directory:

```
packaging/
├── aur/              # Arch Linux (AUR)
├── homebrew/         # Homebrew formula
├── debian/           # Debian/Ubuntu packaging
├── fedora/           # Fedora/RHEL RPM spec
├── snap/             # Snap package
└── flatpak/          # Flatpak manifest
```

**Note:** Scoop (Windows) configuration is maintained separately.

### Arch Linux (AUR)

**Location**: `packaging/aur/`

UTPM provides two AUR packages:

1. **utpm-bin** - Pre-built binaries (faster installation)
   - Downloads release binaries directly
   - Automatically updated by GitHub Actions with new checksums
   
2. **utpm-git** - Build from source (latest development)
   - Clones and builds from git repository
   - Useful for testing unreleased features

**Publishing to AUR**:
```bash
# Clone AUR repository
git clone ssh://aur@aur.archlinux.org/utpm-bin.git

# Copy updated PKGBUILD
cp packaging/aur/utpm-bin/PKGBUILD utpm-bin/

# Update .SRCINFO
cd utpm-bin
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```

**Testing**:
```bash
cd packaging/aur/utpm-bin
makepkg -si  # Build and install locally
```

### Homebrew (macOS/Linux)

**Location**: `packaging/homebrew/utpm.rb`

Homebrew formula for macOS and Linux.

**Publishing to Homebrew**:

Option 1: Official tap (requires approval)
```bash
# Fork homebrew-core
# Update formula at Formula/u/utpm.rb
# Create pull request
```

Option 2: Custom tap (faster)
```bash
# Create a tap repository: homebrew-utpm
# Copy formula to Formula/utpm.rb
# Users install with: brew install typst-community/utpm/utpm
```

**Testing**:
```bash
brew install --build-from-source packaging/homebrew/utpm.rb
brew test packaging/homebrew/utpm.rb
```

**Update checklist**:
- [ ] Version number updated
- [ ] Source URL points to new release
- [ ] SHA256 checksum matches tarball
- [ ] Tested on macOS x86_64 and aarch64

### Debian/Ubuntu

**Location**: `packaging/debian/`

Standard Debian packaging with:
- `control` - Package metadata and dependencies
- `rules` - Build instructions
- `changelog` - Version history
- `copyright` - License information

**Building .deb package**:
```bash
# Install build dependencies
sudo apt-get install build-essential debhelper dh-cargo cargo

# Build package
dpkg-buildpackage -us -uc -b

# Result: ../utpm_X.Y.Z-1_amd64.deb
```

**Publishing**:
- Upload to a PPA (Personal Package Archive)
- Submit to Debian/Ubuntu repositories (long approval process)
- Host in custom repository

**Testing**:
```bash
sudo dpkg -i ../utpm_X.Y.Z-1_amd64.deb
utpm --version
```

### Fedora/RHEL

**Location**: `packaging/fedora/utpm.spec`

RPM spec file for Fedora, RHEL, and derivatives.

**Building .rpm package**:
```bash
# Install build tools
sudo dnf install rpm-build rpmdevtools cargo rust

# Setup build tree
rpmdev-setuptree

# Copy spec file
cp packaging/fedora/utpm.spec ~/rpmbuild/SPECS/

# Download source tarball
spectool -g -R ~/rpmbuild/SPECS/utpm.spec

# Build
rpmbuild -ba ~/rpmbuild/SPECS/utpm.spec

# Result: ~/rpmbuild/RPMS/x86_64/utpm-X.Y.Z-1.x86_64.rpm
```

**Publishing**:
- Submit to COPR (Fedora community repository)
- Create custom RPM repository

**Testing**:
```bash
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/utpm-X.Y.Z-1.x86_64.rpm
utpm --version
```

### Snap

**Location**: `packaging/snap/snapcraft.yaml`

Snap package for universal Linux distribution.

**Building snap**:
```bash
# Install snapcraft
sudo snap install snapcraft --classic

# Build
cd packaging/snap
snapcraft

# Result: utpm_X.Y.Z_amd64.snap
```

**Publishing to Snap Store**:
```bash
# Login (one-time)
snapcraft login

# Upload
snapcraft upload utpm_X.Y.Z_amd64.snap --release=stable

# Or upload to edge for testing
snapcraft upload utpm_X.Y.Z_amd64.snap --release=edge
```

**Testing**:
```bash
sudo snap install utpm_X.Y.Z_amd64.snap --dangerous
utpm --version
```

### Flatpak

**Location**: `packaging/flatpak/`

Flatpak manifest and metainfo for Flathub.

**Building flatpak**:
```bash
# Install flatpak and flatpak-builder
sudo apt-get install flatpak flatpak-builder

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Build
cd packaging/flatpak
flatpak-builder --force-clean build-dir io.github.typst_community.utpm.yaml

# Install locally
flatpak-builder --user --install --force-clean build-dir io.github.typst_community.utpm.yaml
```

**Publishing to Flathub**:
1. Fork https://github.com/flathub/flathub
2. Create new repository: `io.github.typst_community.utpm`
3. Add manifest and metainfo
4. Submit pull request to flathub/flathub

**Testing**:
```bash
flatpak run io.github.typst_community.utpm --version
```

## Manual Release Process

If you need to create a release manually (not recommended):

1. **Update version numbers**:
   - `Cargo.toml` - Project version
   - `packaging/*/` - All package manager files

2. **Build binaries**:
   ```bash
   # Linux
   cargo build --release --target x86_64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   
   # macOS
   cargo build --release --target x86_64-apple-darwin
   cargo build --release --target aarch64-apple-darwin
   
   # Windows
   cargo build --release --target x86_64-pc-windows-msvc
   ```

3. **Generate completions**:
   ```bash
   utpm generate bash > completions/utpm.bash
   utpm generate fish > completions/utpm.fish
   utpm generate zsh > completions/_utpm
   ```

4. **Create archives**:
   ```bash
   # Unix
   tar czf utpm-x86_64-unknown-linux-gnu.tar.gz \
     target/x86_64-unknown-linux-gnu/release/utpm \
     completions/* README.md LICENSE
   
   # Windows
   zip utpm-x86_64-pc-windows-msvc.zip \
     target/x86_64-pc-windows-msvc/release/utpm.exe \
     completions/* README.md LICENSE
   ```

5. **Calculate checksums**:
   ```bash
   sha256sum utpm-*.tar.gz utpm-*.zip > SHA256SUMS
   ```

6. **Create GitHub Release**:
   - Go to Releases → New Release
   - Create tag `vX.Y.Z`
   - Upload all archives and SHA256SUMS
   - Publish release

7. **Update package files** with new checksums

8. **Publish to package managers** following their respective processes

## Testing Packages

### Pre-release Checklist

Before publishing to package managers:

- [ ] All binaries build successfully
- [ ] Version numbers are consistent across all files
- [ ] SHA256 checksums are correct
- [ ] Shell completions are included
- [ ] LICENSE and README are included
- [ ] Packages install without errors
- [ ] `utpm --version` shows correct version
- [ ] Basic commands work (`utpm prj init`, `utpm pkg list`)

### Test Environments

Use containers or VMs to test packages:

```bash
# Arch Linux
docker run -it archlinux bash
# Install and test utpm

# Ubuntu
docker run -it ubuntu:22.04 bash
# Install and test .deb

# Fedora
docker run -it fedora:latest bash
# Install and test .rpm

# Windows
# Use Windows Sandbox or VM for testing
```

### Automated Testing

Add package installation tests to CI:

```yaml
# .github/workflows/test-packages.yml
- name: Test AUR package
  run: |
    docker run -v $PWD:/workspace archlinux bash -c \
      "cd /workspace/packaging/aur/utpm-bin && makepkg -si --noconfirm"
```

## Troubleshooting

### Common Issues

**Checksum mismatch**:
- Ensure you're calculating checksum of the exact release artifact
- Use `sha256sum` on Linux/macOS, `certutil -hashfile` on Windows
- GitHub Actions calculates these automatically

**Build failures**:
- Check Rust toolchain version
- Verify all dependencies are available
- Review build logs for missing libraries

**Permission errors** (Snap/Flatpak):
- Ensure manifest requests correct permissions
- Test with `--devmode` flag first

**AUR package rejected**:
- Verify PKGBUILD follows Arch packaging standards
- Run `namcap` to check for common issues
- Use `.SRCINFO` generated by `makepkg --printsrcinfo`

### Getting Help

- **GitHub Issues**: https://github.com/typst-community/utpm/issues
- **Package-specific**:
  - AUR: https://wiki.archlinux.org/title/AUR_submission_guidelines
  - Homebrew: https://docs.brew.sh/Formula-Cookbook
  - Snap: https://snapcraft.io/docs
  - Flatpak: https://docs.flatpak.org/

## Contributing

To add support for a new package manager:

1. Create directory in `packaging/`
2. Add configuration files
3. Update this documentation
4. Add to `.github/workflows/release.yml` for automation
5. Test thoroughly
6. Submit pull request

---

**Note**: The GitHub Actions workflow handles most of this automatically. Manual steps are only needed for initial setup or debugging.

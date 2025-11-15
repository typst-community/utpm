# Publishing to Package Managers

This guide documents the automated and manual steps for publishing UTPM to various package managers.

## Overview

After creating a release with `git tag vX.Y.Z && git push --tags`, the GitHub Actions workflow automatically:

1. Builds binaries for 5 platforms (Linux x86_64/aarch64/musl, macOS x86_64/aarch64)
2. Creates a GitHub Release with all artifacts
3. Calculates SHA256 checksums
4. Updates package configuration files
5. Creates a Pull Request with updated checksums
6. **Publishes to AUR** (utpm-bin and utpm-git)
7. **Publishes to Homebrew Tap** (typst-community/utpm)
8. **Publishes to Snap Store** (stable channel)
9. **Creates PR to Flathub** (requires manual merge)

**Fully automated:** AUR, Homebrew Tap, Snap Store
**Semi-automated:** Flatpak (PR created, manual review required)
**Manual only:** Homebrew Core, Debian PPA, Fedora COPR

---

## Release Workflow

### 1. Create Release

```bash
# Update version in Cargo.toml
vim Cargo.toml  # version = "X.Y.Z"

# Commit and push
git add Cargo.toml
git commit -m "chore: bump version to X.Y.Z"
git push

# Wait for CI to pass (tests, format, clippy)

# Create and push tag
git tag vX.Y.Z
git push --tags
```

### 2. Verify GitHub Actions

1. Go to https://github.com/typst-community/utpm/actions
2. Verify "Release" workflow executes successfully
3. Wait for all builds to complete

### 3. Merge Checksums PR

1. A PR will be automatically created with updated checksums
2. Review checksums
3. Merge the PR

### 4. Verify Automated Publishing

The workflow automatically publishes to:
- **AUR** (utpm-bin and utpm-git)
- **Homebrew Tap** (typst-community/utpm)
- **Snap Store** (stable channel)
- **Flatpak** (PR created to Flathub, requires manual merge)

Check the Actions tab to verify all jobs completed successfully.

---

## Required Secrets Configuration

For automation to work, the following GitHub secrets must be configured:

- `AUR_SSH_KEY` - SSH private key with access to aur@aur.archlinux.org
- `SNAPCRAFT_TOKEN` - Snapcraft store credentials (from `snapcraft export-login`)
- `FLATPAK_TOKEN` - GitHub token with permission to create PRs on Flathub repos

## Automated Publishing

### AUR (Arch Linux) - ✅ Fully Automated

Both `utpm-bin` and `utpm-git` are automatically published to AUR.

**What the workflow does:**
1. Updates PKGBUILD with new version and checksums
2. Generates .SRCINFO
3. Commits and pushes to AUR repositories

**Manual verification (optional):**
```bash
# Test locally if needed
git clone ssh://aur@aur.archlinux.org/utpm-bin.git
cd utpm-bin
makepkg -si
namcap PKGBUILD
```

### Homebrew Tap - ✅ Fully Automated

The formula is automatically updated in `typst-community/homebrew-utpm`.

**What the workflow does:**
1. Updates formula with new version and checksum
2. Commits and pushes to homebrew-utpm repository

**Users install with:**
```bash
brew install typst-community/utpm/utpm
```

### Snap Store - ✅ Fully Automated

The snap is automatically built and published to the stable channel.

**What the workflow does:**
1. Builds snap from latest code
2. Uploads to Snap Store
3. Releases to stable channel

**Users install with:**
```bash
sudo snap install utpm
```

### Flatpak (Flathub) - 🔄 Semi-Automated

A PR is automatically created on Flathub, but requires manual review.

**What the workflow does:**
1. Updates manifest with new version tag
2. Creates PR to Flathub repository

**Manual step required:**
- Review and merge the PR on Flathub

---

## Manual Publishing (Optional)

### Homebrew Core

For official Homebrew Core inclusion (optional, requires maintainer approval):

```bash
# Fork homebrew-core
git clone https://github.com/Homebrew/homebrew-core.git
cd homebrew-core

# Create branch
git checkout -b utpm-X.Y.Z

# Create Formula/u/utpm.rb
mkdir -p Formula/u
cp ../utpm/packaging/homebrew/utpm.rb Formula/u/

# Test
brew install --build-from-source Formula/u/utpm.rb
brew test Formula/u/utpm.rb
brew audit --strict Formula/u/utpm.rb

# Create PR on homebrew-core
git add Formula/u/utpm.rb
git commit -m "utpm X.Y.Z (new formula)"
git push origin utpm-X.Y.Z
```

### Debian/Ubuntu PPA

For Debian/Ubuntu distribution via PPA:

```bash
# Create Launchpad account: https://launchpad.net
# Create PPA: https://launchpad.net/~/+activate-ppa

# Build package
cd utpm
dpkg-buildpackage -us -uc -b

# Upload to PPA
dput ppa:your-username/utpm ../utpm_X.Y.Z-1_source.changes

# Users install with:
# sudo add-apt-repository ppa:your-username/utpm
# sudo apt update
# sudo apt install utpm
```

### Fedora COPR

For Fedora/RHEL distribution via COPR:

```bash
# Create COPR account: https://copr.fedorainfracloud.org
# Create new project: utpm

# Build in COPR (web interface or CLI)
copr-cli build utpm packaging/fedora/utpm.spec

# Users install with:
# sudo dnf copr enable your-username/utpm
# sudo dnf install utpm
```

---

## Post-Release Checklist

- [ ] GitHub Release created with all artifacts
- [ ] Checksums PR merged
- [ ] Verify Actions completed successfully:
  - [ ] AUR utpm-bin published (automated ✅)
  - [ ] AUR utpm-git updated (automated ✅)
  - [ ] Homebrew tap updated (automated ✅)
  - [ ] Snap published to Snap Store (automated ✅)
  - [ ] Flatpak PR created (automated 🔄, manual merge required)
- [ ] Optional manual publishing:
  - [ ] Homebrew Core (if desired)
  - [ ] Debian/Ubuntu PPA (if setup)
  - [ ] Fedora COPR (if setup)
- [ ] Announcement on Discord/Reddit Typst community

## Troubleshooting

### Incorrect Checksums

```bash
# Download artifact from GitHub Release
wget https://github.com/typst-community/utpm/releases/download/vX.Y.Z/utpm-x86_64-unknown-linux-gnu.tar.gz

# Calculate checksum
sha256sum utpm-x86_64-unknown-linux-gnu.tar.gz

# Compare with value in configuration file
```

### Build Failures

```bash
# Test locally first
cd packaging/aur/utpm-bin  # or other package manager
makepkg -si

# Check logs
cat /tmp/makepkg.log
```

### Package Rejected

- **AUR**: Check with `namcap PKGBUILD`
- **Homebrew**: Check with `brew audit --strict`
- **Snap**: Check logs in Snap Store dashboard
- **Flatpak**: Check with `flatpak-builder --verbose`

## Resources

- **AUR**: https://wiki.archlinux.org/title/AUR_submission_guidelines
- **Homebrew**: https://docs.brew.sh/Formula-Cookbook
- **Snap**: https://snapcraft.io/docs/releasing-your-app
- **Flatpak**: https://docs.flatpak.org/en/latest/
- **Debian**: https://wiki.debian.org/Packaging
- **Fedora COPR**: https://docs.pagure.org/copr.copr/

---

**Note:** Publishing to package managers is a manual process, but checksums are automatically calculated by the GitHub Actions workflow. Always use checksums from the official GitHub Release.

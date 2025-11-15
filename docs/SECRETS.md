# GitHub Secrets Configuration

This document explains how to configure the required GitHub secrets for automated package publishing.

## Required Secrets

The automated release workflow requires the following secrets to be configured in the GitHub repository settings:

### 1. AUR_SSH_KEY

**Purpose:** Authenticate with AUR to publish packages

**How to generate:**

```bash
# Generate SSH key pair
ssh-keygen -t ed25519 -C "your-email@example.com" -f ~/.ssh/aur

# Add public key to AUR
# Go to https://aur.archlinux.org/account/
# Navigate to "My Account" → "SSH Public Key"
# Paste contents of ~/.ssh/aur.pub

# Copy private key for GitHub secret
cat ~/.ssh/aur
```

**Add to GitHub:**
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `AUR_SSH_KEY`
4. Value: Paste the **private key** (contents of `~/.ssh/aur`)

### 2. SNAPCRAFT_TOKEN

**Purpose:** Authenticate with Snap Store to publish snaps

**How to generate:**

```bash
# Login to Snapcraft (first time)
snapcraft login

# Export credentials
snapcraft export-login snapcraft-token.txt

# View the token
cat snapcraft-token.txt
```

**Add to GitHub:**
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `SNAPCRAFT_TOKEN`
4. Value: Paste the contents of `snapcraft-token.txt`

**Security note:** Delete `snapcraft-token.txt` after adding to GitHub secrets.

### 3. FLATPAK_TOKEN

**Purpose:** Create PRs on Flathub repositories

**How to generate:**

1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Give it a descriptive name: "UTPM Flatpak Publishing"
4. Set expiration as desired (or "No expiration" for convenience)
5. Select scopes:
   - `public_repo` (for creating PRs)
6. Click "Generate token"
7. Copy the token immediately (you won't be able to see it again)

**Add to GitHub:**
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `FLATPAK_TOKEN`
4. Value: Paste the GitHub token

## Verification

After adding all secrets, trigger a test release:

```bash
# Create a test tag
git tag v0.0.0-test
git push --tags

# Watch the Actions tab
# https://github.com/typst-community/utpm/actions
```

Check that all publishing jobs complete successfully:
- ✅ publish-aur
- ✅ publish-homebrew
- ✅ publish-snap
- ✅ publish-flatpak

## Troubleshooting

### AUR Publishing Fails

**Error:** `Permission denied (publickey)`

**Solution:**
- Verify SSH key is added to AUR account
- Verify private key is correctly pasted in `AUR_SSH_KEY` secret
- Check that key doesn't have a passphrase (GitHub Actions can't handle passphrases)

### Snap Publishing Fails

**Error:** `Invalid credentials`

**Solution:**
- Re-export credentials: `snapcraft export-login snapcraft-token.txt`
- Update `SNAPCRAFT_TOKEN` secret with new credentials
- Verify you have permissions to publish to the snap name

### Flatpak PR Fails

**Error:** `403 Forbidden`

**Solution:**
- Verify GitHub token has `public_repo` scope
- Check token hasn't expired
- Ensure token belongs to a user with push access to the fork

## Security Best Practices

1. **Rotate secrets regularly** - Update tokens/keys every 6-12 months
2. **Use minimal permissions** - Only grant necessary scopes
3. **Monitor Actions logs** - Check for unauthorized access attempts
4. **Never commit secrets** - Always use GitHub Secrets, never hardcode
5. **Delete local credentials** - Remove exported files after adding to secrets

## Revoking Access

If secrets are compromised:

1. **AUR:** Remove SSH key from https://aur.archlinux.org/account/
2. **Snap Store:** Revoke via `snapcraft logout` and regenerate
3. **GitHub Token:** Revoke at https://github.com/settings/tokens
4. **Update secrets:** Generate new credentials and update GitHub secrets

## Repository Setup Checklist

For new maintainers setting up automation:

- [ ] Generate AUR SSH key and add to AUR account
- [ ] Export Snapcraft credentials
- [ ] Generate GitHub token for Flatpak
- [ ] Add all three secrets to GitHub repository
- [ ] Create `homebrew-utpm` repository under `typst-community`
- [ ] Fork Flathub app repository to `typst-community`
- [ ] Test with a pre-release tag
- [ ] Document any issues in this file

---

**Last Updated:** November 2025

# OpenPaste Release Process

## Overview

This document describes the release process for OpenPaste, including versioning, build procedures, testing requirements, and distribution methods.

## Versioning

### Semantic Versioning

OpenPaste follows Semantic Versioning (SemVer 2.0.0):

**Format:** `MAJOR.MINOR.PATCH`

- **MAJOR:** Incompatible API changes
- **MINOR:** New functionality (backward compatible)
- **PATCH:** Bug fixes (backward compatible)

**Examples:**
- `0.1.0` → First release
- `0.2.0` → New features
- `0.2.1` → Bug fix
- `1.0.0` → Stable release

### Pre-Release Versions

**Format:** `MAJOR.MINOR.PATCH-PRERELEASE+METADATA`

**Pre-release identifiers:**
- `alpha`: Early development
- `beta`: Feature complete, testing
- `rc`: Release candidate

**Examples:**
- `0.1.0-alpha.1`
- `0.1.0-beta.1`
- `0.1.0-rc.1`

### Version Bumping

**When to Bump:**
- **MAJOR:** Breaking changes to API or data format
- **MINOR:** New features, deprecations
- **PATCH:** Bug fixes, documentation

**Deprecation Process:**
1. Mark as deprecated in documentation
2. Add warning in code
3. Remove in next MAJOR version

## Release Checklist

### Pre-Release

**Code:**
- [ ] All tests pass
- [ ] Code coverage meets targets
- [ ] No critical bugs
- [ ] No known security vulnerabilities
- [ ] Dependencies updated
- [ ] Code reviewed

**Documentation:**
- [ ] CHANGELOG updated
- [ ] Release notes written
- [ ] API documentation updated
- [ ] User guide updated
- [ ] Migration guide (if breaking changes)

**Build:**
- [ ] Windows build successful
- [ ] macOS build successful
- [ ] Linux build successful
- [ ] All artifacts generated
- [ ] Code signing complete

**Testing:**
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Manual testing complete
- [ ] Platform-specific testing complete

### Release

**Version Update:**
- [ ] Update version in Cargo.toml
- [ ] Update version in package.json
- [ ] Update version in README
- [ ] Create git tag

**Publishing:**
- [ ] Publish to GitHub Releases
- [ ] Upload installers
- [ ] Upload packages
- [ ] Update website (if applicable)

**Announcement:**
- [ ] Post release notes
- [ ] Announce on social media
- [ ] Notify community
- [ ] Update documentation site

### Post-Release

**Monitoring:**
- [ ] Monitor for issues
- [ ] Monitor crash reports
- [ ] Monitor performance
- [ ] Gather user feedback

**Follow-up:**
- [ ] Address critical bugs
- [ ] Plan next release
- [ ] Update roadmap

## Build Process

### Prerequisites

**Required Tools:**
- Rust 1.75+
- Node.js 18+
- Platform-specific build tools
- Code signing certificates

**Environment Variables:**
```bash
export CARGO_REGISTRY_TOKEN=...
export APPLE_SIGNING_ID=...
export WINDOWS_SIGNING_CERT=...
```

### Windows Build

**Build:**
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

**Package:**
```bash
# Create installer with WiX or similar
cargo install cargo-wix
cargo wix
```

**Sign:**
```bash
signtool sign /f cert.pfx /p password /t timestamp_url dist/openpaste.msi
```

### macOS Build

**Build:**
```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**Package (Tauri):**
```bash
cd apps/desktop
npm run tauri build
```

**Sign and Notarize:**
```bash
codesign --deep --force --verify --verbose --sign "$SIGNING_ID" OpenPaste.app
xcrun notarytool submit OpenPaste.app --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$TEAM_ID"
xcrun stapler staple OpenPaste.app
```

### Linux Build

**Build:**
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

**Package (Deb):**
```bash
cargo install cargo-deb
cargo deb
```

**Package (RPM):**
```bash
cargo install cargo-rpm
cargo rpm build
```

**Package (AppImage):**
```bash
# Use linuxdeploy or similar
linuxdeploy-x86_64.AppImage --appdir AppDir --executable openpaste --output appimage
```

## CI/CD Pipeline

### GitHub Actions Workflow

**.github/workflows/release.yml:**
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Test
        run: cargo test --release --target ${{ matrix.target }}
      
      - name: Package
        run: cargo package --target ${{ matrix.target }}
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v3
        with:
          name: openpaste-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/openpaste*

  release:
    needs: build
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Download Artifacts
        uses: actions/download-artifact@v3
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            openpaste-*
          draft: false
          prerelease: ${{ contains(github.ref, 'alpha') || contains(github.ref, 'beta') }}
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Release Notes

### Release Notes Template

```markdown
# OpenPaste v{VERSION}

## Highlights
- Major feature 1
- Major feature 2

## New Features
- Feature 1
- Feature 2

## Improvements
- Improvement 1
- Improvement 2

## Bug Fixes
- Bug fix 1
- Bug fix 2

## Breaking Changes
- Breaking change 1 (migration guide: link)

## Known Issues
- Known issue 1

## Upgrade Instructions
### From v{PREVIOUS_VERSION}
- Step 1
- Step 2

## Downloads
- [Windows](link)
- [macOS](link)
- [Linux](link)

## Checksums
```
windows: SHA256 hash
macos: SHA256 hash
linux: SHA256 hash
```

## Contributors
- @contributor1
- @contributor2
```

### CHANGELOG Format

**Keep a Changelog format:**
```markdown
## [Unreleased]

### Added
- New feature

### Changed
- Changed behavior

### Deprecated
- Deprecated feature

### Removed
- Removed feature

### Fixed
- Bug fix

### Security
- Security fix
```

## Distribution

### GitHub Releases

**Primary Distribution:**
- Source code
- Pre-built binaries
- Installers
- Packages
- Checksums

**Release Types:**
- Pre-releases (alpha, beta, rc)
- Stable releases
- LTS releases (future)

### Package Managers

**Homebrew (macOS):**
```ruby
# Formula
class Openpaste < Formula
  desc "Cross-platform clipboard manager"
  homepage "https://github.com/openpaste/openpaste"
  url "https://github.com/openpaste/openpaste/archive/v#{version}.tar.gz"
  sha256 "..."
  
  depends_on "rust"
  
  def install
    system "cargo", "install", "--path", "."
  end
end
```

**APT (Debian/Ubuntu):**
```bash
# Add repository
echo "deb https://apt.openpaste.org stable main" | sudo tee /etc/apt/sources.list.d/openpaste.list

# Install
sudo apt update
sudo apt install openpaste
```

**RPM (Fedora/RHEL):**
```bash
# Add repository
sudo dnf config-manager --add-repo https://yum.openpaste.org/openpaste.repo

# Install
sudo dnf install openpaste
```

**Snap (Linux):**
```bash
sudo snap install openpaste
```

**Flatpak (Linux):**
```bash
flatpak install flathub com.openpaste.OpenPaste
```

**Chocolatey (Windows):**
```powershell
choco install openpaste
```

**Winget (Windows):**
```powershell
winget install openpaste
```

## Code Signing

### Windows Code Signing

**Certificate:**
- Code signing certificate from CA
- EV certificate preferred
- Store in secure location

**Process:**
```bash
signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com /td sha256 dist/openpaste.msi
```

### macOS Code Signing

**Certificate:**
- Apple Developer certificate
- Team ID required
- Store in keychain

**Process:**
```bash
codesign --deep --force --verify --verbose --sign "$SIGNING_ID" OpenPaste.app
```

### Linux Signing

**GPG Signing:**
```bash
gpg --default-key "$KEY_ID" --detach-sign --armor openpaste-linux.tar.gz
```

**Repository Signing:**
```bash
# Sign repository metadata
gpg --default-key "$KEY_ID" --detach-sign --armor Release
```

## Verification

### Checksum Verification

**Generate Checksums:**
```bash
sha256sum openpaste-* > SHA256SUMS
```

**Verify Checksums:**
```bash
sha256sum -c SHA256SUMS
```

### Signature Verification

**Verify GPG Signature:**
```bash
gpg --verify openpaste-linux.tar.gz.asc
```

**Import Public Key:**
```bash
gpg --keyserver keys.openpgp.org --recv-keys KEY_ID
```

## Rollback Procedure

### If Rollback Needed

**Steps:**
1. Identify issue
2. Determine rollback version
3. Re-release previous version as hotfix
4. Communicate with users
5. Investigate root cause

**Hotfix Version:**
- Use same MAJOR.MINOR
- Increment PATCH
- Mark as hotfix in notes

## Security Releases

### Security Release Process

**When:**
- Critical security vulnerability
- High severity vulnerability with exploit

**Process:**
1. Fix vulnerability
2. Security audit
3. Coordinated disclosure
4. Release as soon as ready
5. Clear communication

**Timeline:**
- Target: Within 7 days of discovery
- Critical: Within 48 hours

## Testing Before Release

### Release Testing Checklist

**Functional Testing:**
- [ ] Clipboard capture works
- [ ] Search works
- [ ] Storage works
- [ ] Encryption works
- [ ] UI works
- [ ] CLI works

**Platform Testing:**
- [ ] Windows 10/11
- [ ] macOS 12+
- [ ] Ubuntu 22.04
- [ ] Fedora 38
- [ ] Debian 12

**Edge Cases:**
- [ ] Large clipboard items
- [ ] Special characters
- [ ] Unicode content
- [ ] Empty clipboard
- [ ] Database corruption

**Performance:**
- [ ] Startup time
- [ ] Search latency
- [ ] Memory usage
- [ ] CPU usage

## Post-Release Monitoring

### Metrics to Track

**Downloads:**
- Download count per platform
- Download count per version
- Geographic distribution

**Issues:**
- Bug reports
- Feature requests
- Crash reports

**Performance:**
- Error rates
- Latency
- Resource usage

### User Feedback

**Channels:**
- GitHub Issues
- GitHub Discussions
- Discord (when available)
- Email

**Response:**
- Acknowledge within 24 hours
- Provide timeline for fix
- Communicate progress

## Release Schedule

### Regular Releases

**Target:**
- Monthly minor releases
- Weekly patch releases (as needed)
- Quarterly major releases

### Release Cadence

**Alpha:** Weekly during development

**Beta:** Bi-weekly during feature freeze

**RC:** Weekly during stabilization

**Stable:** As needed

## Communication

### Release Announcement

**Channels:**
- GitHub Release
- Blog post
- Twitter/X
- Reddit
- Hacker News

**Content:**
- Release highlights
- Download links
- Upgrade instructions
- Known issues

### Changelog

**Automatic:**
- Generate from git commits
- Use conventional commits
- Filter by type

**Manual:**
- Add context
- Group related changes
- Highlight breaking changes

## Troubleshooting

### Common Build Issues

**Windows Build Fails:**
- Check Visual Studio Build Tools
- Check MSVC version
- Check linker settings

**macOS Build Fails:**
- Check Xcode version
- Check code signing
- Check notarization

**Linux Build Fails:**
- Check dependencies
- Check linker
- Check library versions

### Common Release Issues

**Artifacts Missing:**
- Check CI logs
- Check artifact upload
- Check file permissions

**Checksums Wrong:**
- Regenerate checksums
- Verify files
- Check upload process

**Signing Fails:**
- Check certificate validity
- Check certificate chain
- Check timestamp server

## References

### Documentation

**SemVer:**
- [Semantic Versioning 2.0.0](https://semver.org/)

**Keep a Changelog:**
- [Keep a Changelog](https://keepachangelog.com/)

**Release Best Practices:**
- [GitHub Release Best Practices](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)

### Tools

**Build Tools:**
- [cargo-wix](https://github.com/roblabla/cargo-wix)
- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-rpm](https://github.com/cargo-bins/cargo-rpm)

**Signing Tools:**
- [signtool](https://docs.microsoft.com/en-us/windows/win32/seccrypto/signtool)
- [codesign](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigning/)
- [gpg](https://gnupg.org/)

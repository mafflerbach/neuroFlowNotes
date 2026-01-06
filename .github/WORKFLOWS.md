# GitHub Actions Workflows

This document describes the CI/CD workflows for NeuroFlow Notes.

---

## Available Workflows

### 1. CI (Continuous Integration)
**File:** `.github/workflows/ci.yml`

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch
- Manual dispatch

**Purpose:** Fast quality checks before merging code

**Jobs:**
- **TypeScript validation** - Type checking with `npm run check`
- **Rust tests** - Run all workspace tests with `cargo test`
- **Clippy linting** - Strict linting with `-D warnings`
- **Build verification** - Ensure the app builds successfully

**Usage:**
- Runs automatically on every push/PR
- Manual run: Go to Actions → CI → Run workflow

---

### 2. Build Artifacts
**File:** `.github/workflows/build-artifacts.yml`

**Triggers:**
- Push to `main` branch (when source files change)
- Manual dispatch

**Purpose:** Build platform-specific installers for testing (not for release)

**Platforms Built:**
- **macOS** - Universal binary (Intel + Apple Silicon)
- **Linux** - `.deb`, `.AppImage`
- **Windows** - `.msi`, `.exe` installer

**Artifacts:**
- Stored as GitHub Actions artifacts
- Retention: 30 days
- Naming: `{platform}-{commit-sha}`

**Usage:**
```bash
# Automatic: Push to main
git push origin main

# Manual: 
# 1. Go to Actions → Build Artifacts → Run workflow
# 2. Choose whether to upload artifacts (default: true)
```

**Download artifacts:**
1. Go to Actions → Build Artifacts → Select run
2. Scroll to "Artifacts" section
3. Download desired platform bundle

---

### 3. Release
**File:** `.github/workflows/release.yml`

**Triggers:**
- Push a tag matching `v*` (e.g., `v0.1.0`, `v1.2.3`)
- Manual dispatch with version input

**Purpose:** Create official GitHub releases with installers attached

**Key Features:**
- ✅ **Dynamic version handling** - Extracts version from tag name
- ✅ **Per-release artifacts** - Each tag creates its own release
- ✅ **Draft releases** - Review before publishing
- ✅ **Multi-platform builds** - macOS (both architectures), Linux, Windows

**Platforms Built:**
- **macOS** 
  - Apple Silicon: `aarch64-apple-darwin`
  - Intel: `x86_64-apple-darwin`
- **Linux** - `x86_64-unknown-linux-gnu`
- **Windows** - `x86_64-pc-windows-msvc`

**Release Artifacts:**
Each release includes:
- `NeuroFlow.Notes_X.Y.Z_aarch64.dmg` (macOS Apple Silicon)
- `NeuroFlow.Notes_X.Y.Z_x64.dmg` (macOS Intel)
- `NeuroFlow.Notes_X.Y.Z_amd64.deb` (Linux Debian/Ubuntu)
- `NeuroFlow.Notes_X.Y.Z_amd64.AppImage` (Linux universal)
- `NeuroFlow.Notes_X.Y.Z_x64-setup.exe` (Windows installer)
- `NeuroFlow.Notes_X.Y.Z_x64_en-US.msi` (Windows MSI)

---

## How to Create a Release

### Method 1: Using Git Tags (Recommended)

```bash
# 1. Update version in package.json and Cargo.toml
npm version 0.2.0  # Or manually edit files

# 2. Commit the version bump
git add package.json package-lock.json src-tauri/Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"

# 3. Create and push the tag
git tag v0.2.0
git push origin main
git push origin v0.2.0

# 4. GitHub Actions will automatically:
#    - Build for all platforms
#    - Create a draft release
#    - Attach installers to the release

# 5. Review and publish the release:
#    - Go to GitHub → Releases
#    - Edit the draft release
#    - Add release notes (or use CHANGELOG)
#    - Click "Publish release"
```

### Method 2: Manual Workflow Dispatch

```bash
# 1. Go to GitHub Actions → Release → Run workflow
# 2. Enter version (e.g., 0.2.0) - without the 'v' prefix
# 3. Click "Run workflow"
# 4. Review and publish the draft release when ready
```

---

## Version Management

### Version Extraction Logic

The release workflow automatically determines the version:

**Tag Push (`v0.2.0`):**
```
Tag: v0.2.0
→ Extract version: 0.2.0
→ Release name: "NeuroFlow Notes v0.2.0"
```

**Manual Dispatch (input: `0.2.0`):**
```
Input: 0.2.0
→ Create tag: v0.2.0
→ Release name: "NeuroFlow Notes v0.2.0"
```

### Files to Update

Before releasing, ensure these files have the correct version:

1. **`package.json`** - `"version": "0.2.0"`
2. **`src-tauri/Cargo.toml`** - `version = "0.2.0"`
3. **`Cargo.lock`** - Auto-updated by `cargo build`
4. **`CHANGELOG.md`** - Document changes (optional but recommended)

---

## Workflow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CI Workflow                         │
│  (Fast checks on every push/PR)                             │
│  ✓ TypeScript   ✓ Tests   ✓ Clippy   ✓ Build              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Build Artifacts Workflow                  │
│  (Test builds on main branch)                               │
│  → Upload as GitHub Actions artifacts (30-day retention)    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     Release Workflow                        │
│  (Official releases on version tags)                        │
│  1. Prepare job → Extract version from tag                  │
│  2. Build job → Create installers for all platforms         │
│  3. Finalize job → Create GitHub Release with artifacts     │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Improvements (Recent Changes)

### ✅ Dynamic Version Handling
- **Before:** Hardcoded `v__VERSION__` placeholder (always created v0.1.0)
- **After:** Extracts version from git tag or manual input

### ✅ Per-Release Artifacts
- **Before:** All releases attached to v0.1.0
- **After:** Each tag (v0.1.0, v0.2.0, etc.) gets its own release

### ✅ Prepare Job
- **New:** Centralized version extraction logic
- **Benefit:** Consistent version handling across jobs

### ✅ Rust Caching
- **Added:** `Swatinem/rust-cache@v2` to speed up builds
- **Benefit:** Faster CI/release builds (reuses compiled dependencies)

### ✅ Better Build Artifacts Workflow
- **Added:** Conditional artifact upload
- **Added:** More paths to trigger builds (crates, package files)
- **Added:** Manual control over artifact upload

---

## Troubleshooting

### Release not created
- **Check:** Did you push the tag? (`git push origin v0.2.0`)
- **Check:** Is the tag format correct? (must start with `v`)
- **Check:** View Actions tab for build logs

### Artifacts missing
- **Check:** Build completed successfully (no errors in Actions)
- **Check:** Artifacts are attached to draft release
- **Note:** Draft releases are not public until published

### Version mismatch
- **Check:** Version in `package.json` matches tag
- **Check:** Version in `src-tauri/Cargo.toml` matches tag
- **Fix:** Update files and create a new tag

### Build failures
- **Check:** CI workflow passes first
- **Check:** All tests pass locally (`cargo test --workspace`)
- **Check:** App builds locally (`npm run build`)

---

## Best Practices

1. **Always run CI before releasing**
   ```bash
   npm run check
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   ```

2. **Use semantic versioning**
   - `v0.1.0` → `v0.1.1` (patch - bug fixes)
   - `v0.1.0` → `v0.2.0` (minor - new features)
   - `v0.1.0` → `v1.0.0` (major - breaking changes)

3. **Update CHANGELOG.md**
   - Document user-facing changes
   - Include upgrade instructions if needed

4. **Test draft releases**
   - Download and test installers before publishing
   - Verify version numbers in app

5. **Review release notes**
   - Add meaningful descriptions
   - Link to closed issues/PRs
   - Highlight breaking changes

---

## Examples

### Example 1: Patch Release

```bash
# Fix a bug, create v0.1.1
git checkout main
git pull

# Make changes, commit
git add .
git commit -m "fix: resolve crash on startup"

# Update version
npm version patch  # 0.1.0 → 0.1.1

# Create release
git push origin main
git tag v0.1.1
git push origin v0.1.1

# GitHub creates draft release → Review → Publish
```

### Example 2: Minor Release

```bash
# Add new feature, create v0.2.0
npm version minor  # 0.1.1 → 0.2.0

git add package.json package-lock.json src-tauri/Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"

git push origin main
git tag v0.2.0
git push origin v0.2.0
```

### Example 3: Manual Release (No Git Tag)

```bash
# Update versions manually in files
# Then go to GitHub Actions → Release → Run workflow
# Enter: 0.3.0
```

---

## Additional Resources

- **Tauri Action Docs:** https://github.com/tauri-apps/tauri-action
- **GitHub Releases:** https://docs.github.com/en/repositories/releasing-projects-on-github
- **Semantic Versioning:** https://semver.org/

# Publishing DOL to crates.io

This document outlines the process, requirements, and considerations for publishing the DOL crate to crates.io.

## Overview

[crates.io](https://crates.io) is Rust's official package registry. Publishing there allows anyone to:
- Install with `cargo install dol`
- Add as dependency with `cargo add dol`
- Access documentation on [docs.rs](https://docs.rs)

## Pre-Publication Checklist

### 1. Account Setup

- [ ] Create account at https://crates.io (requires GitHub login)
- [ ] Generate API token at https://crates.io/settings/tokens
- [ ] Login locally: `cargo login <token>`

### 2. Cargo.toml Requirements

| Field | Status | Required |
|-------|--------|----------|
| `name` | `dol` | Yes |
| `version` | `0.1.0` | Yes |
| `edition` | `2021` | Yes |
| `authors` | ✓ Set | Recommended |
| `description` | ✓ Set | Yes |
| `license` | `MIT OR Apache-2.0` | Yes |
| `repository` | ✓ Set | Recommended |
| `homepage` | ✓ Set | Recommended |
| `documentation` | ✓ Set | Recommended |
| `readme` | `README.md` | Recommended |
| `keywords` | ✓ Set (max 5) | Recommended |
| `categories` | ✓ Set | Recommended |

Current Cargo.toml looks good. All required fields are present.

### 3. Name Availability

```bash
# Check if 'dol' is available
cargo search dol
```

**Note**: The name `dol` may already be taken. If so, alternatives:
- `dol-lang`
- `dol-parser`
- `univrs-dol`
- `metadol`

### 4. Code Quality Gates

- [x] All tests pass: `cargo test`
- [x] No clippy warnings: `cargo clippy -- -D warnings`
- [x] Formatting clean: `cargo fmt --check`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] No unused dependencies: `cargo +nightly udeps` (optional)

### 5. Documentation Requirements

For docs.rs to build properly:
- All public items should have `///` doc comments
- Examples in doc comments should compile
- README.md should explain basic usage

```bash
# Test documentation locally
cargo doc --no-deps --open
```

### 6. License Files

Ensure these exist in the repo root:
- [ ] `LICENSE-MIT` or `LICENSE`
- [ ] `LICENSE-APACHE` (if dual-licensed)

## Pre-Publish Verification

### Dry Run

```bash
# Verify package contents without publishing
cargo publish --dry-run
```

This checks:
- Package can be built
- All required metadata is present
- No files exceed size limits
- Dependencies resolve correctly

### Package Contents

```bash
# List what will be published
cargo package --list
```

Review the list to ensure:
- No secrets (`.env`, credentials)
- No large binary files
- No unnecessary test fixtures
- Excludes match what's in `Cargo.toml`

Current excludes in Cargo.toml:
```toml
exclude = [
    ".github/",
    "docs/tutorials/",
    "examples/*.test",
]
```

Consider adding:
```toml
exclude = [
    ".github/",
    "docs/tutorials/",
    "examples/*.test",
    "claude-flow-*.yaml",    # Workflow files
    "RELEASE_NOTES_*.md",    # Release artifacts
]
```

## Publishing Process

### Step 1: Final Verification

```bash
# Run all checks
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo doc --no-deps
cargo publish --dry-run
```

### Step 2: Publish

```bash
cargo publish
```

**Warning**: Publishing is permanent. You cannot:
- Delete a published version
- Overwrite a published version
- Change the crate name

You can only:
- Yank a version (hide from new downloads, existing users unaffected)
- Publish a new version

### Step 3: Verify Publication

After publishing:
1. Check https://crates.io/crates/dol
2. Wait for docs.rs build (~15 minutes)
3. Verify https://docs.rs/dol

## Version Strategy

### Semantic Versioning

| Version | Meaning |
|---------|---------|
| 0.x.y | Pre-1.0, API may change |
| 0.1.0 | Current (first public release) |
| 0.1.1 | Bug fixes only |
| 0.2.0 | New features, possible breaking changes |
| 1.0.0 | Stable API commitment |

### When to Bump

- **Patch (0.1.x)**: Bug fixes, documentation
- **Minor (0.x.0)**: New features, deprecations
- **Major (x.0.0)**: Breaking changes (after 1.0)

## Post-Publication

### Announcing

- Update README with crates.io badge
- Announce on relevant channels (Reddit r/rust, Twitter, etc.)

### Badges for README

```markdown
[![Crates.io](https://img.shields.io/crates/v/dol.svg)](https://crates.io/crates/dol)
[![Documentation](https://docs.rs/dol/badge.svg)](https://docs.rs/dol)
[![License](https://img.shields.io/crates/l/dol.svg)](LICENSE)
```

### Maintenance

- Respond to issues promptly
- Keep dependencies updated
- Publish security fixes quickly

## Yanking (If Needed)

If a version has critical bugs:

```bash
# Hide version from new installations
cargo yank --version 0.1.0

# Undo yank
cargo yank --version 0.1.0 --undo
```

Yanked versions:
- Still work for existing Cargo.lock files
- Won't be picked for new installations
- Show as yanked on crates.io

## Common Issues

### 1. Name Taken

```
error: crate `dol` already exists
```

Solution: Choose alternative name or contact current owner.

### 2. Missing Fields

```
error: missing field `description`
```

Solution: Add required field to Cargo.toml.

### 3. Dependency Issues

```
error: failed to select a version for `some-dep`
```

Solution: Ensure all dependencies are published on crates.io.

### 4. Size Limit

```
error: package exceeds maximum size
```

Solution: Add large files to `exclude` in Cargo.toml.

## Commands Summary

```bash
# Check name availability
cargo search dol

# Login to crates.io
cargo login

# Dry run (verify without publishing)
cargo publish --dry-run

# List package contents
cargo package --list

# Publish
cargo publish

# Yank a version
cargo yank --version 0.1.0
```

## Current Status

| Item | Status |
|------|--------|
| Cargo.toml metadata | ✅ Complete |
| Tests passing | ✅ 626 tests |
| Clippy clean | ✅ No warnings |
| Formatting | ✅ Clean |
| Documentation | ⚠️ Verify with `cargo doc` |
| License files | ⚠️ Verify exist |
| Name availability | ⚠️ Check with `cargo search` |
| Dry run | ⏳ Pending |

## Next Steps

1. Run `cargo search dol` to check name availability
2. Run `cargo doc --no-deps` to verify docs build
3. Verify LICENSE files exist
4. Run `cargo publish --dry-run`
5. Review output and fix any issues
6. Run `cargo publish` when ready

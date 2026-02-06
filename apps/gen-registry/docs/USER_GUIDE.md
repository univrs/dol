# Gen Registry User Guide

## Getting Started

### Installation

```bash
# Install from source
git clone https://github.com/univrs/dol
cd dol/apps/gen-registry
cargo install --path . --bin gen-registry

# Verify installation
gen-registry --version
```

### Setup

```bash
# Set your DID (decentralized identifier)
export GEN_REGISTRY_DID="did:key:your-key-here"

# Optional: Set custom data directory
export GEN_REGISTRY_DATA_DIR="$HOME/.gen-registry"
```

## Publishing Modules

### Step 1: Prepare Your Module

Ensure you have:
- DOL source file (`.dol`)
- Compiled WASM binary (`.wasm`)
- Module metadata (name, description, tags)

### Step 2: Publish

```bash
gen-registry publish \
  --id io.yourorg.modulename \
  --name "Module Display Name" \
  --description "Brief description of your module" \
  --license MIT \
  --version 1.0.0 \
  --wasm path/to/module.wasm \
  --changelog "Initial release" \
  --tags "category1,category2"
```

**Module ID Format**: Reverse domain notation
- ✓ `io.univrs.authentication`
- ✓ `com.company.database`
- ✗ `authentication` (too short)
- ✗ `io..auth` (empty segment)

### Step 3: Verify

```bash
gen-registry info io.yourorg.modulename
```

## Discovering Modules

### Search by Keyword

```bash
gen-registry search "authentication"
```

Output:
```
Found 12 modules:

  io.univrs.user v2.1.0
    User authentication and profile management
    Score: 0.95

  com.example.auth v1.5.0
    JWT-based authentication
    Score: 0.87
```

### Search with Limit

```bash
gen-registry search "database" --limit 5
```

### Browse by Tag

Use the web interface for tag-based browsing.

## Installing Modules

### Install Latest Version

```bash
gen-registry install io.univrs.database
```

### Install Specific Version

```bash
gen-registry install io.univrs.crypto --version 2.1.0
```

### Install with Auto-Update

```bash
gen-registry install io.univrs.http --auto-update
```

Auto-update will:
- Check for updates daily
- Install compatible versions (respects semver)
- Keep update history

### View Installed Modules

```bash
gen-registry list
```

Output:
```
Installed modules (3):

  io.univrs.database v3.0.1
  io.univrs.crypto v2.1.0
    Auto-update: enabled
  io.univrs.http v4.2.0
    Auto-update: enabled
```

## Rating Modules

### Submit a Rating

```bash
gen-registry rate io.univrs.database \
  --stars 5 \
  --review "Excellent database module! Easy to use."
```

**Star Range**: 1-5
- 5 stars: Excellent
- 4 stars: Good
- 3 stars: Average
- 2 stars: Below average
- 1 star: Poor

## P2P Networking

### Start Sync Daemon

```bash
gen-registry daemon --port 8080
```

The daemon:
- Discovers peers on the network
- Syncs module metadata
- Serves the web interface

### Sync Specific Module

```bash
gen-registry sync io.univrs.user
```

### Check Status

```bash
gen-registry status
```

Output:
```
Registry Status:
  Peers: 5
  Modules: 142
  Synced: 2 minutes ago
```

## Web Interface

### Access

1. Start daemon: `gen-registry daemon --port 8080`
2. Open browser: `http://localhost:8080`

### Features

**Search**:
- Type in search box
- Filter by tags
- Sort by popularity, rating, or recency

**Browse**:
- Grid view of modules
- Card shows: name, version, description, stats
- Click "Install" to install
- Click "Info" for details

**Module Details**:
- Version history
- Dependencies
- Capabilities (functions, types)
- Reviews and ratings
- Installation command

## Advanced Usage

### Custom Data Directory

```bash
gen-registry --data-dir /custom/path search authentication
```

### Verbose Logging

```bash
gen-registry --verbose search database
```

### Environment Variables

```bash
# DID for authentication
GEN_REGISTRY_DID="did:key:..."

# Data directory
GEN_REGISTRY_DATA_DIR="$HOME/.gen-registry"

# Enable debug logging
RUST_LOG=debug
```

## Troubleshooting

### Module Not Found

**Problem**: `Error: Module not found: io.univrs.xyz`

**Solutions**:
1. Check spelling
2. Run `gen-registry sync io.univrs.xyz`
3. Ensure P2P is enabled
4. Check network connectivity

### Installation Fails

**Problem**: Installation hangs or fails

**Solutions**:
1. Check dependencies are installed
2. Verify disk space
3. Try again with `--verbose` flag
4. Check network connection

### P2P Sync Issues

**Problem**: No peers discovered

**Solutions**:
1. Check firewall settings
2. Ensure port is not blocked
3. Try relay server
4. Check network configuration

### WASM Validation Errors

**Problem**: `Error: WASM validation failed`

**Solutions**:
1. Verify WASM is valid
2. Check file size (must be < 10 MB)
3. Ensure correct target: `wasm32-unknown-unknown`
4. Rebuild WASM module

## Best Practices

### Publishing

- Use semantic versioning strictly
- Write detailed changelogs
- Tag modules appropriately
- Test WASM before publishing
- Sign modules cryptographically

### Discovering

- Use specific search terms
- Filter by tags for category
- Check ratings before installing
- Review dependencies

### Installing

- Pin versions in production
- Use auto-update in development
- Test after installation
- Check compatibility

### Rating

- Be constructive in reviews
- Rate based on quality
- Update ratings when module improves
- Report security issues privately

## Resources

- [API Documentation](api.md)
- [Architecture](ARCHITECTURE.md)
- [Contributing](../CONTRIBUTING.md)
- [Examples](../examples/)

---

**Need Help?** Open an issue on GitHub: https://github.com/univrs/dol/issues

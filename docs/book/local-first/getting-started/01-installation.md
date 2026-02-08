# Installation Guide

This guide walks you through installing the complete VUDO toolchain for building local-first applications.

## Prerequisites

### Required

- **Rust** 1.75+ and `cargo`
- **Node.js** 20+ and `npm` or `pnpm`
- **Git** for version control

### Optional (but recommended)

- **wasm-pack** for WASM builds
- **wasmtime** for testing WASM components
- **Docker** for relay server deployment

## Quick Install (Recommended)

The fastest way to get started is using the official installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://dol.univrs.io/install.sh | sh
```

This installs:
- `dol` - CLI for project management
- `dol-parse` - Parser and validator
- `dol-codegen-rust` - Rust code generator
- `dol-codegen-wit` - WIT interface generator
- `dol-test` - CRDT property testing
- `vudo-runtime` - Runtime library

### Verify Installation

```bash
dol --version
# Output: dol 0.8.0

dol-parse --version
# Output: dol-parse 0.8.0

cargo --version
# Output: cargo 1.76.0
```

## Manual Installation (Advanced)

If you prefer to build from source:

### 1. Clone the Repository

```bash
git clone https://github.com/univrs/dol.git
cd dol
```

### 2. Build the Toolchain

```bash
# Build all CLI tools
cargo build --release --bins

# Install globally
cargo install --path crates/dol-cli
cargo install --path crates/dol-parse
cargo install --path crates/dol-codegen-rust
cargo install --path crates/dol-codegen-wit
```

### 3. Add to PATH

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

### 4. Verify

```bash
which dol
# Output: /home/user/.cargo/bin/dol
```

## Platform-Specific Setup

### macOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install rust node

# Install DOL
curl --proto '=https' --tlsv1.2 -sSf https://dol.univrs.io/install.sh | sh
```

### Linux (Ubuntu/Debian)

```bash
# Install dependencies
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install DOL
curl --proto '=https' --tlsv1.2 -sSf https://dol.univrs.io/install.sh | sh
```

### Windows (WSL2 Recommended)

```powershell
# Enable WSL2
wsl --install

# Inside WSL2, follow Linux instructions above
```

For native Windows:

```powershell
# Install via Chocolatey
choco install rust nodejs

# Install DOL (PowerShell)
iwr -useb https://dol.univrs.io/install.ps1 | iex
```

## WASM Tooling

For compiling DOL to WASM, install additional tools:

### wasm-pack

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### wasm-bindgen-cli

```bash
cargo install wasm-bindgen-cli
```

### wasm-opt (Binaryen)

```bash
# macOS
brew install binaryen

# Ubuntu/Debian
sudo apt install binaryen

# Windows (via Chocolatey)
choco install binaryen
```

### wasmtime (for testing)

```bash
curl https://wasmtime.dev/install.sh -sSf | bash
```

## Editor Setup

### VS Code

Install the DOL Language Server extension:

```bash
code --install-extension univrs.dol-language-server
```

**Features**:
- Syntax highlighting for `.dol` files
- Autocomplete for CRDT annotations
- Inline error checking
- Jump to definition
- Hover documentation

**Configuration** (`settings.json`):
```json
{
  "dol.checkOnSave": true,
  "dol.codegen.autoRun": false,
  "dol.trace.server": "off"
}
```

### Neovim

Install via [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig):

```lua
require('lspconfig').dol_ls.setup{
  cmd = {"dol-language-server"},
  filetypes = {"dol"},
  root_dir = require('lspconfig.util').root_pattern("Cargo.toml", ".git"),
}
```

### Emacs

Install [dol-mode](https://github.com/univrs/dol-mode):

```elisp
(use-package dol-mode
  :ensure t
  :mode "\\.dol\\'"
  :hook (dol-mode . lsp-deferred))
```

## Create Your First Project

```bash
# Create a new local-first project
dol new my-app --template local-first

cd my-app

# Project structure
tree .
# my-app/
# ├── schemas/          # DOL schema definitions
# │   └── example.dol
# ├── src/              # Rust source
# │   ├── generated/    # Generated code (gitignored)
# │   └── lib.rs
# ├── web/              # Frontend
# │   ├── src/
# │   └── package.json
# ├── Cargo.toml
# └── dol.toml          # Project configuration
```

### Project Configuration

Edit `dol.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2.0"  # DOL 2.0 with CRDT annotations

[codegen]
target = "rust"
output_dir = "src/generated"
wasm_bindgen = true

[crdt]
default_strategy = "lww"  # Default if no annotation
validate_types = true

[sync]
protocol = "iroh"
relay_servers = ["relay.univrs.io:4433"]

[storage]
backend = "indexeddb"  # For web
# backend = "sqlite"   # For desktop
```

## Test the Installation

Run the example app:

```bash
# Compile DOL schemas
dol build

# Check for errors
dol check schemas/

# Generate Rust code
dol codegen

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Run dev server
cd web && npm run dev
```

Visit `http://localhost:5173` - you should see a working collaborative editor!

## Troubleshooting

### "dol: command not found"

**Solution**: Add Cargo bin directory to PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc  # or ~/.zshrc
```

### "error: linking with `cc` failed"

**Solution**: Install build essentials:

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
```

### WASM build fails

**Solution**: Add WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

### "dol-codegen-rust not found"

**Solution**: Build from source or reinstall:

```bash
cargo install dol-cli
```

### Permission denied errors

**Solution**: Don't use `sudo` with Rust/Cargo:

```bash
# Wrong
sudo cargo install dol-cli

# Correct
cargo install dol-cli
```

## Updating

To update to the latest version:

```bash
# Update DOL toolchain
dol update

# Or manually
cargo install --force dol-cli
cargo install --force dol-codegen-rust
```

## Uninstall

To remove the DOL toolchain:

```bash
# Remove binaries
cargo uninstall dol-cli
cargo uninstall dol-codegen-rust
cargo uninstall dol-codegen-wit
cargo uninstall dol-parse

# Remove configuration
rm -rf ~/.dol
```

## Next Steps

Now that you have the toolchain installed, let's build your first app:

→ **Next**: [Your First App](./02-first-app.md)

## Additional Resources

### Documentation
- [DOL Language Reference](../../specification.md)
- [CRDT Guide](../crdt-guide/00-overview.md)
- [API Reference](../api-reference/dol-syntax.md)

### Community
- [GitHub Discussions](https://github.com/univrs/dol/discussions)
- [Discord Server](https://discord.gg/univrs)
- [Stack Overflow Tag: `dol`](https://stackoverflow.com/questions/tagged/dol)

### Examples
- [Example Repository](https://github.com/univrs/dol-examples)
- [Workspace App](/apps/workspace)
- [Community Showcase](https://univrs.io/showcase)

---

**Next**: [Your First App →](./02-first-app.md)

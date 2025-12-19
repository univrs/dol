#!/bin/bash
# Metal DOL Remediation - Claude-Flow Bootstrap Script
# This script initializes the project and prepares for multi-agent orchestration

set -e

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        Metal DOL Remediation - Claude-Flow Bootstrap             ║"
echo "╚══════════════════════════════════════════════════════════════════╝"

# Configuration
REPO_URL="https://github.com/univrs/metadol.git"
WORK_DIR="."
BRANCH="remediation/phase-1"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Step 1: Clone or update repository
echo ""
log_info "Step 1: Setting up repository..."

if [ -d "$WORK_DIR" ]; then
    log_warn "Work directory exists, updating..."
    cd "$WORK_DIR"
    git fetch origin
    git checkout main
    git pull origin main
else
    log_info "Cloning repository..."
    git clone "$REPO_URL" "$WORK_DIR"
    cd "$WORK_DIR"
fi

# Step 2: Create remediation branch
log_info "Step 2: Creating remediation branch..."
git checkout -b "$BRANCH" 2>/dev/null || git checkout "$BRANCH"

# Step 3: Create directory structure
log_info "Step 3: Creating directory structure..."
mkdir -p src/bin
mkdir -p tests
mkdir -p examples/{genes,traits,constraints,systems}
mkdir -p docs/tutorials
mkdir -p .github/workflows
mkdir -p .claude-flow

# Step 4: Copy scaffolding files
log_info "Step 4: Copying scaffolding files..."

# Copy project files from the remediation package
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cp "$PROJECT_DIR/CLAUDE.md" . 2>/dev/null || log_warn "CLAUDE.md not found"
cp "$PROJECT_DIR/docs/roadmap.md" docs/ 2>/dev/null || log_warn "roadmap.md not found"
cp "$PROJECT_DIR/.claude-flow/tasks.yaml" .claude-flow/ 2>/dev/null || log_warn "tasks.yaml not found"
cp "$PROJECT_DIR/.claude-flow/orchestration.yaml" .claude-flow/ 2>/dev/null || log_warn "orchestration.yaml not found"
cp "$PROJECT_DIR/docs/grammar.ebnf" docs/ 2>/dev/null || log_warn "grammar.ebnf not found"

# Step 5: Initialize Cargo.toml if needed
log_info "Step 5: Checking Cargo.toml..."
if [ ! -f "Cargo.toml" ]; then
    log_info "Creating Cargo.toml..."
    cat > Cargo.toml << 'EOF'
[package]
name = "metadol"
version = "0.0.1"
edition = "2021"
authors = ["Univrs <team@univrs.io>"]
description = "Metal DOL - Design Ontology Language for ontology-first development"
license = "MIT OR Apache-2.0"
repository = "https://github.com/univrs/metadol"
keywords = ["dsl", "ontology", "design", "specification", "testing"]
categories = ["development-tools", "parsing"]

[lib]
name = "metadol"
path = "src/lib.rs"

[[bin]]
name = "dol-parse"
path = "src/bin/dol-parse.rs"
required-features = ["cli"]

[[bin]]
name = "dol-test"
path = "src/bin/dol-test.rs"
required-features = ["cli"]

[[bin]]
name = "dol-check"
path = "src/bin/dol-check.rs"
required-features = ["cli"]

[features]
default = []
cli = ["clap", "anyhow", "colored"]
serde = ["dep:serde", "dep:serde_json"]
codegen = []

[dependencies]
thiserror = "1.0"
clap = { version = "4.4", features = ["derive"], optional = true }
anyhow = { version = "1.0", optional = true }
colored = { version = "2.0", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }

[dev-dependencies]
pretty_assertions = "1.4"
insta = "1.34"
criterion = "0.5"

[profile.release]
lto = true
strip = true

[package.metadata.docs.rs]
all-features = true
EOF
fi

# Step 6: Verify structure
log_info "Step 6: Verifying structure..."
echo ""
echo "Directory structure:"
find . -type d -name ".git" -prune -o -type f -print | head -30
echo ""

# Step 7: Run initial checks
log_info "Step 7: Running initial checks..."
if command -v cargo &> /dev/null; then
    cargo check 2>/dev/null && log_success "cargo check passed" || log_warn "cargo check had issues (expected for new project)"
else
    log_warn "Cargo not found, skipping build check"
fi

# Summary
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║                    Bootstrap Complete!                           ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
log_success "Project initialized at: $(pwd)"
log_success "Branch: $BRANCH"
echo ""
echo "Next steps for claude-flow:"
echo "  1. Load orchestration config: .claude-flow/orchestration.yaml"
echo "  2. Review roadmap: docs/roadmap.md"
echo "  3. Start Phase 1 tasks with lexer-agent and parser-agent"
echo ""
echo "Quick start command:"
echo "  claude-flow run --config .claude-flow/orchestration.yaml --phase phase_1_foundation"
echo ""

# Tutorial 09: Multi-Language Code Generation Workflow

> **Single schema → Rust, TypeScript, Python, WIT, JSON Schema with build automation**
>
> **Level**: Intermediate | **Time**: 70 minutes | **Lines**: 190+

## Overview

Build a complete polyglot SDK from a single DOL schema with:
- Automated code generation for 5 languages
- Package manager integration
- CI/CD workflows
- Version synchronization

## Project Structure

```
polyglot-sdk/
├── schema/
│   ├── user.dol
│   ├── product.dol
│   └── order.dol
├── generated/
│   ├── rust/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── typescript/
│   │   ├── package.json
│   │   └── src/index.ts
│   ├── python/
│   │   ├── setup.py
│   │   └── src/__init__.py
│   ├── wit/
│   │   └── world.wit
│   └── json-schema/
│       └── schema.json
├── scripts/
│   ├── generate.sh
│   ├── test-all.sh
│   └── publish.sh
├── Makefile
└── .github/
    └── workflows/
        └── ci.yml
```

## Complete Schema Example

**File**: `schema/ecommerce.dol`

```dol
module ecommerce @ 1.0.0

// User domain
gen User {
    @crdt(immutable)
    has id: string

    @crdt(lww)
    has email: string

    @crdt(lww)
    has name: string

    @crdt(lww)
    has created_at: Int
}

// Product domain
gen Product {
    @crdt(immutable)
    has id: string

    @crdt(lww)
    has name: string

    @crdt(lww)
    has price: Float64 where price >= 0.0

    @crdt(pn_counter)
    has stock: Int where stock >= 0

    @crdt(or_set)
    has tags: Set<string>
}

// Order domain
gen Order {
    @crdt(immutable)
    has id: string

    @crdt(immutable)
    has user_id: string

    @crdt(rga)
    has items: Vec<OrderItem>

    @crdt(lww)
    has status: OrderStatus

    @crdt(lww)
    has total: Float64
}

gen OrderItem {
    has product_id: string
    has quantity: Int where quantity > 0
    has unit_price: Float64
    has subtotal: Float64
}

gen OrderStatus {
    enum {
        Pending,
        Processing,
        Shipped,
        Delivered,
        Cancelled
    }
}

docs {
    E-commerce domain model for polyglot SDK generation.
    Version: 1.0.0
}
```

## Build Script: Generate All Targets

**File**: `scripts/generate.sh` (80+ lines)

```bash
#!/bin/bash
# Generate code for all 5 targets

set -e

SCHEMA_DIR="schema"
OUT_DIR="generated"
VERSION="1.0.0"

echo "🔨 Multi-Language Code Generation"
echo "=================================="

# Clean previous output
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 1. Generate Rust
echo -e "${BLUE}Generating Rust...${NC}"
mkdir -p "$OUT_DIR/rust/src"

dol-codegen --target rust \
    --package-name ecommerce \
    --version "$VERSION" \
    "$SCHEMA_DIR"/*.dol > "$OUT_DIR/rust/src/lib.rs"

# Create Cargo.toml
cat > "$OUT_DIR/rust/Cargo.toml" << EOF
[package]
name = "ecommerce"
version = "$VERSION"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
criterion = "0.5"
EOF

echo -e "${GREEN}✓ Rust generated${NC}"

# 2. Generate TypeScript
echo -e "${BLUE}Generating TypeScript...${NC}"
mkdir -p "$OUT_DIR/typescript/src"

dol-codegen --target typescript \
    --package-name "@company/ecommerce" \
    --version "$VERSION" \
    "$SCHEMA_DIR"/*.dol > "$OUT_DIR/typescript/src/index.ts"

# Create package.json
cat > "$OUT_DIR/typescript/package.json" << EOF
{
  "name": "@company/ecommerce",
  "version": "$VERSION",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "jest"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "jest": "^29.0.0"
  }
}
EOF

# Create tsconfig.json
cat > "$OUT_DIR/typescript/tsconfig.json" << EOF
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "declaration": true,
    "outDir": "./dist",
    "strict": true
  },
  "include": ["src/**/*"]
}
EOF

echo -e "${GREEN}✓ TypeScript generated${NC}"

# 3. Generate Python
echo -e "${BLUE}Generating Python...${NC}"
mkdir -p "$OUT_DIR/python/src/ecommerce"

dol-codegen --target python \
    --package-name ecommerce \
    --version "$VERSION" \
    "$SCHEMA_DIR"/*.dol > "$OUT_DIR/python/src/ecommerce/__init__.py"

# Create setup.py
cat > "$OUT_DIR/python/setup.py" << 'EOF'
from setuptools import setup, find_packages

setup(
    name="ecommerce",
    version="$VERSION",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    install_requires=[
        "pydantic>=2.0.0",
    ],
    python_requires=">=3.10",
)
EOF

# Create pyproject.toml
cat > "$OUT_DIR/python/pyproject.toml" << EOF
[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"

[project]
name = "ecommerce"
version = "$VERSION"
requires-python = ">=3.10"
dependencies = [
    "pydantic>=2.0.0",
]
EOF

echo -e "${GREEN}✓ Python generated${NC}"

# 4. Generate WIT
echo -e "${BLUE}Generating WIT...${NC}"
mkdir -p "$OUT_DIR/wit"

dol-codegen --target wit \
    --world ecommerce-world \
    "$SCHEMA_DIR"/*.dol > "$OUT_DIR/wit/world.wit"

echo -e "${GREEN}✓ WIT generated${NC}"

# 5. Generate JSON Schema
echo -e "${BLUE}Generating JSON Schema...${NC}"
mkdir -p "$OUT_DIR/json-schema"

dol-codegen --target json-schema \
    "$SCHEMA_DIR"/*.dol > "$OUT_DIR/json-schema/schema.json"

# Validate JSON Schema
if command -v jsonschema &> /dev/null; then
    jsonschema --check "$OUT_DIR/json-schema/schema.json" && \
        echo -e "${GREEN}✓ JSON Schema valid${NC}"
fi

# Generate summary
echo ""
echo "📊 Generation Summary"
echo "===================="
echo "Rust:        $(wc -l < "$OUT_DIR/rust/src/lib.rs") lines"
echo "TypeScript:  $(wc -l < "$OUT_DIR/typescript/src/index.ts") lines"
echo "Python:      $(wc -l < "$OUT_DIR/python/src/ecommerce/__init__.py") lines"
echo "WIT:         $(wc -l < "$OUT_DIR/wit/world.wit") lines"
echo "JSON Schema: $(wc -l < "$OUT_DIR/json-schema/schema.json") lines"

echo ""
echo -e "${GREEN}✅ All targets generated successfully!${NC}"
```

## Makefile for Automation

**File**: `Makefile` (50 lines)

```makefile
.PHONY: all generate build test clean publish

SCHEMA_FILES := $(wildcard schema/*.dol)

all: generate build test

# Generate code for all targets
generate: $(SCHEMA_FILES)
	@echo "Generating code..."
	@bash scripts/generate.sh

# Build all targets
build: generate
	@echo "Building Rust..."
	cd generated/rust && cargo build --release

	@echo "Building TypeScript..."
	cd generated/typescript && npm install && npm run build

	@echo "Building Python..."
	cd generated/python && pip install -e .

	@echo "All targets built!"

# Run tests for all targets
test: build
	@echo "Testing Rust..."
	cd generated/rust && cargo test

	@echo "Testing TypeScript..."
	cd generated/typescript && npm test

	@echo "Testing Python..."
	cd generated/python && pytest

# Type checking
typecheck:
	@echo "Type checking TypeScript..."
	cd generated/typescript && npx tsc --noEmit

	@echo "Type checking Python..."
	cd generated/python && mypy src/

# Lint all generated code
lint:
	cd generated/rust && cargo clippy -- -D warnings
	cd generated/typescript && npx eslint src/
	cd generated/python && ruff check src/

# Clean generated files
clean:
	rm -rf generated/
	rm -rf */target */node_modules */dist */__pycache__

# Publish packages
publish: test
	@bash scripts/publish.sh

# Watch schema files and regenerate on change
watch:
	@echo "Watching schema files..."
	@while true; do \
		inotifywait -e modify schema/*.dol; \
		make generate; \
	done

# Generate documentation
docs: generate
	cd generated/rust && cargo doc --no-deps
	cd generated/typescript && npx typedoc src/index.ts
	cd generated/python && pdoc --html src/ecommerce

# Benchmark generated code
bench: build
	cd generated/rust && cargo bench
	cd generated/typescript && npm run bench
	cd generated/python && pytest --benchmark-only

# Check for breaking changes
check-compat:
	@echo "Checking API compatibility..."
	@cargo-semver-checks semver-check
```

## CI/CD Pipeline

**File**: `.github/workflows/ci.yml` (60 lines)

```yaml
name: Multi-Language CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install DOL
        run: |
          cargo install metadol
          dol-codegen --version

      - name: Generate code
        run: make generate

      - name: Upload generated code
        uses: actions/upload-artifact@v3
        with:
          name: generated-code
          path: generated/

  test-rust:
    needs: generate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/download-artifact@v3
        with:
          name: generated-code
          path: generated/

      - name: Build Rust
        run: cd generated/rust && cargo build --release

      - name: Test Rust
        run: cd generated/rust && cargo test

      - name: Lint Rust
        run: cd generated/rust && cargo clippy -- -D warnings

  test-typescript:
    needs: generate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18

      - uses: actions/download-artifact@v3
        with:
          name: generated-code
          path: generated/

      - name: Install dependencies
        run: cd generated/typescript && npm install

      - name: Build TypeScript
        run: cd generated/typescript && npm run build

      - name: Test TypeScript
        run: cd generated/typescript && npm test

      - name: Type check
        run: cd generated/typescript && npx tsc --noEmit

  test-python:
    needs: generate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - uses: actions/download-artifact@v3
        with:
          name: generated-code
          path: generated/

      - name: Install dependencies
        run: |
          cd generated/python
          pip install -e .
          pip install pytest mypy

      - name: Test Python
        run: cd generated/python && pytest

      - name: Type check Python
        run: cd generated/python && mypy src/

  publish:
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    needs: [test-rust, test-typescript, test-python]
    runs-on: ubuntu-latest
    steps:
      - name: Publish packages
        run: make publish
```

## Version Synchronization

**File**: `scripts/sync-versions.sh`

```bash
#!/bin/bash
# Ensure all packages have the same version

set -e

VERSION_FILE="VERSION"
VERSION=$(cat "$VERSION_FILE")

echo "Syncing version to $VERSION"

# Update Cargo.toml
sed -i "s/version = \".*\"/version = \"$VERSION\"/" \
    generated/rust/Cargo.toml

# Update package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" \
    generated/typescript/package.json

# Update setup.py
sed -i "s/version=\".*\"/version=\"$VERSION\"/" \
    generated/python/setup.py

echo "✓ All versions synced to $VERSION"
```

## Package Publishing

**File**: `scripts/publish.sh`

```bash
#!/bin/bash
# Publish packages to registries

set -e

# Check if on main branch
if [ "$(git branch --show-current)" != "main" ]; then
    echo "Error: Must be on main branch to publish"
    exit 1
fi

# Publish Rust to crates.io
echo "Publishing Rust package..."
cd generated/rust
cargo publish

# Publish TypeScript to npm
echo "Publishing TypeScript package..."
cd ../typescript
npm publish

# Publish Python to PyPI
echo "Publishing Python package..."
cd ../python
python -m build
twine upload dist/*

echo "✅ All packages published!"
```

## Common Pitfalls

### Pitfall 1: Version Drift

```bash
# ❌ Wrong: Manual version updates
# Versions get out of sync!

# ✅ Correct: Single source of truth
echo "1.0.0" > VERSION
make sync-versions
```

### Pitfall 2: Breaking Changes

```bash
# Check for API compatibility before publishing
cargo semver-checks --baseline-version 0.9.0
```

## Performance Tips

1. **Parallel generation**: Generate targets in parallel
2. **Incremental builds**: Only regenerate changed schemas
3. **Caching**: Cache generated code in CI

---

**Next**: [Tutorial 10: Production Deployment](./10-Production-Deployment-Guide.md)

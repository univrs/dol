#!/bin/bash
#
# WASM Module Registry Publication Script
#
# Publishes compiled WASM modules to a registry for distribution.
# Supports multiple registry backends: S3, npm, custom HTTP endpoint.
#
# Usage: ./scripts/publish-wasm.sh <module-name> <version> [registry-type]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <module-name> <version> [registry-type]"
    echo ""
    echo "Publishes WASM module to registry after compilation and optimization."
    echo ""
    echo "Arguments:"
    echo "  module-name    Name of the WASM module (e.g., vudo-state)"
    echo "  version        Semantic version (e.g., 0.1.0)"
    echo "  registry-type  Registry backend: s3, npm, http (default: s3)"
    echo ""
    echo "Environment variables:"
    echo "  WASM_REGISTRY_S3_BUCKET    S3 bucket name (for s3 registry)"
    echo "  WASM_REGISTRY_NPM_SCOPE    npm scope (for npm registry)"
    echo "  WASM_REGISTRY_HTTP_URL     HTTP endpoint (for http registry)"
    echo ""
    echo "Examples:"
    echo "  $0 vudo-state 0.1.0 s3"
    echo "  $0 vudo-state 0.1.0 npm"
    echo "  WASM_REGISTRY_S3_BUCKET=my-wasm-bucket $0 vudo-state 0.1.0"
    exit 1
}

if [ $# -lt 2 ]; then
    usage
fi

MODULE_NAME="$1"
VERSION="$2"
REGISTRY_TYPE="${3:-s3}"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}WASM Module Registry Publisher${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Module:   $MODULE_NAME"
echo "Version:  $VERSION"
echo "Registry: $REGISTRY_TYPE"
echo ""

# Validate version format (semver)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
    echo -e "${RED}❌ Error: Invalid version format. Must be semver (e.g., 0.1.0)${NC}"
    exit 1
fi

# Step 1: Build WASM module
echo -e "${YELLOW}Step 1: Building WASM module...${NC}"

cargo build --target wasm32-unknown-unknown --release --features wasm-compile

WASM_FILE="target/wasm32-unknown-unknown/release/${MODULE_NAME}.wasm"

if [ ! -f "$WASM_FILE" ]; then
    # Try with lib prefix
    WASM_FILE="target/wasm32-unknown-unknown/release/lib${MODULE_NAME}.wasm"
    if [ ! -f "$WASM_FILE" ]; then
        echo -e "${RED}❌ Error: WASM file not found after build${NC}"
        echo "Expected: target/wasm32-unknown-unknown/release/${MODULE_NAME}.wasm"
        exit 1
    fi
fi

echo -e "${GREEN}✓ WASM module built: $WASM_FILE${NC}"

# Step 2: Check size budget
echo ""
echo -e "${YELLOW}Step 2: Checking size budget...${NC}"

if ! ./scripts/wasm-size-budget.sh "$WASM_FILE"; then
    echo -e "${RED}❌ Error: WASM module exceeds size budget${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Size budget met${NC}"

# Optimized files
OPT_WASM="${WASM_FILE}.opt"
OPT_WASM_GZ="${WASM_FILE}.opt.gz"

# Step 3: Generate metadata
echo ""
echo -e "${YELLOW}Step 3: Generating module metadata...${NC}"

METADATA_FILE="target/wasm32-unknown-unknown/release/${MODULE_NAME}-${VERSION}.json"

cat > "$METADATA_FILE" <<EOF
{
  "name": "$MODULE_NAME",
  "version": "$VERSION",
  "buildDate": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "size": {
    "original": $(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE"),
    "optimized": $(stat -c%s "$OPT_WASM" 2>/dev/null || stat -f%z "$OPT_WASM"),
    "gzipped": $(stat -c%s "$OPT_WASM_GZ" 2>/dev/null || stat -f%z "$OPT_WASM_GZ")
  },
  "rustVersion": "$(rustc --version | cut -d' ' -f2)",
  "targets": ["wasm32-unknown-unknown"],
  "features": ["wasm-compile"]
}
EOF

echo -e "${GREEN}✓ Metadata generated: $METADATA_FILE${NC}"

# Step 4: Publish to registry
echo ""
echo -e "${YELLOW}Step 4: Publishing to ${REGISTRY_TYPE} registry...${NC}"

case "$REGISTRY_TYPE" in
    s3)
        # S3 registry
        if [ -z "${WASM_REGISTRY_S3_BUCKET:-}" ]; then
            echo -e "${RED}❌ Error: WASM_REGISTRY_S3_BUCKET not set${NC}"
            echo "Set environment variable: export WASM_REGISTRY_S3_BUCKET=my-bucket"
            exit 1
        fi

        if ! command -v aws &> /dev/null; then
            echo -e "${RED}❌ Error: AWS CLI not installed${NC}"
            exit 1
        fi

        S3_PREFIX="s3://${WASM_REGISTRY_S3_BUCKET}/modules/${MODULE_NAME}/${VERSION}"

        echo "Uploading to: $S3_PREFIX"

        # Upload optimized WASM
        aws s3 cp "$OPT_WASM" "${S3_PREFIX}/${MODULE_NAME}.wasm" \
            --content-type "application/wasm" \
            --metadata "version=${VERSION},module=${MODULE_NAME}"

        # Upload gzipped WASM
        aws s3 cp "$OPT_WASM_GZ" "${S3_PREFIX}/${MODULE_NAME}.wasm.gz" \
            --content-type "application/wasm" \
            --content-encoding "gzip" \
            --metadata "version=${VERSION},module=${MODULE_NAME}"

        # Upload metadata
        aws s3 cp "$METADATA_FILE" "${S3_PREFIX}/metadata.json" \
            --content-type "application/json"

        echo -e "${GREEN}✓ Published to S3: ${S3_PREFIX}${NC}"
        ;;

    npm)
        # npm registry
        if [ -z "${WASM_REGISTRY_NPM_SCOPE:-}" ]; then
            echo -e "${YELLOW}⚠️  Warning: WASM_REGISTRY_NPM_SCOPE not set, using default scope${NC}"
            NPM_SCOPE="@vudo"
        else
            NPM_SCOPE="${WASM_REGISTRY_NPM_SCOPE}"
        fi

        NPM_PACKAGE_NAME="${NPM_SCOPE}/${MODULE_NAME}-wasm"
        NPM_DIR="target/npm/${NPM_PACKAGE_NAME}"

        mkdir -p "$NPM_DIR"

        # Copy WASM files
        cp "$OPT_WASM" "$NPM_DIR/${MODULE_NAME}.wasm"
        cp "$OPT_WASM_GZ" "$NPM_DIR/${MODULE_NAME}.wasm.gz"

        # Generate package.json
        cat > "$NPM_DIR/package.json" <<EOF
{
  "name": "$NPM_PACKAGE_NAME",
  "version": "$VERSION",
  "description": "WASM module for $MODULE_NAME (DOL 2.0)",
  "main": "${MODULE_NAME}.wasm",
  "files": [
    "${MODULE_NAME}.wasm",
    "${MODULE_NAME}.wasm.gz",
    "README.md"
  ],
  "keywords": ["wasm", "dol", "crdt", "local-first"],
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/univrs-io/dol.git"
  }
}
EOF

        # Generate README
        cat > "$NPM_DIR/README.md" <<EOF
# $NPM_PACKAGE_NAME

WASM module for \`$MODULE_NAME\` (DOL 2.0 Local-First Runtime).

## Installation

\`\`\`bash
npm install $NPM_PACKAGE_NAME
\`\`\`

## Usage

\`\`\`javascript
import wasmModule from '$NPM_PACKAGE_NAME/${MODULE_NAME}.wasm';

// Initialize WASM module
const module = await WebAssembly.instantiateStreaming(
  fetch(wasmModule)
);
\`\`\`

## Size

- Optimized: $(stat -c%s "$OPT_WASM" 2>/dev/null || stat -f%z "$OPT_WASM") bytes
- Gzipped: $(stat -c%s "$OPT_WASM_GZ" 2>/dev/null || stat -f%z "$OPT_WASM_GZ") bytes

## License

MIT OR Apache-2.0
EOF

        # Publish to npm
        cd "$NPM_DIR"
        npm publish --access public

        echo -e "${GREEN}✓ Published to npm: $NPM_PACKAGE_NAME@$VERSION${NC}"
        ;;

    http)
        # HTTP registry
        if [ -z "${WASM_REGISTRY_HTTP_URL:-}" ]; then
            echo -e "${RED}❌ Error: WASM_REGISTRY_HTTP_URL not set${NC}"
            echo "Set environment variable: export WASM_REGISTRY_HTTP_URL=https://registry.example.com"
            exit 1
        fi

        UPLOAD_URL="${WASM_REGISTRY_HTTP_URL}/modules/${MODULE_NAME}/${VERSION}"

        echo "Uploading to: $UPLOAD_URL"

        # Upload using curl (multipart form data)
        curl -X POST "$UPLOAD_URL" \
            -F "wasm=@${OPT_WASM}" \
            -F "wasm_gz=@${OPT_WASM_GZ}" \
            -F "metadata=@${METADATA_FILE}" \
            -H "Content-Type: multipart/form-data"

        echo ""
        echo -e "${GREEN}✓ Published to HTTP registry: $UPLOAD_URL${NC}"
        ;;

    *)
        echo -e "${RED}❌ Error: Unknown registry type: $REGISTRY_TYPE${NC}"
        echo "Supported types: s3, npm, http"
        exit 1
        ;;
esac

# Step 5: Verify publication
echo ""
echo -e "${YELLOW}Step 5: Verification...${NC}"

case "$REGISTRY_TYPE" in
    s3)
        if aws s3 ls "${S3_PREFIX}/" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Module verified in S3 registry${NC}"
        else
            echo -e "${RED}❌ Verification failed${NC}"
            exit 1
        fi
        ;;
    npm)
        if npm view "$NPM_PACKAGE_NAME@$VERSION" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Module verified in npm registry${NC}"
        else
            echo -e "${YELLOW}⚠️  Verification skipped (npm propagation delay)${NC}"
        fi
        ;;
    http)
        echo -e "${YELLOW}⚠️  Manual verification required for HTTP registry${NC}"
        ;;
esac

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Publication complete!${NC}"
echo ""
echo "Module:  $MODULE_NAME"
echo "Version: $VERSION"
echo "Registry: $REGISTRY_TYPE"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

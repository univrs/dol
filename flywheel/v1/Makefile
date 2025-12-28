# ═══════════════════════════════════════════════════════════════════════════════
# DOL Makefile - Flywheel Edition
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: all flywheel bootstrap check test clean help metrics

PROJECT_ROOT := $(shell pwd)
BOOTSTRAP_DIR := $(PROJECT_ROOT)/target/bootstrap
BOOTSTRAP_SRC := $(BOOTSTRAP_DIR)/src
DOL_DIR := $(PROJECT_ROOT)/dol
MODULES := types token ast lexer

# Default: run the flywheel
all: flywheel

# ═══════════════════════════════════════════════════════════════════════════════
# FLYWHEEL TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Run complete flywheel pipeline
flywheel:
	@./scripts/flywheel.sh

## Quick flywheel (skip tests)
flywheel-quick:
	@./scripts/flywheel.sh --quick

## Verbose flywheel
flywheel-verbose:
	@./scripts/flywheel.sh --verbose

## Show metrics dashboard
metrics:
	@./scripts/metrics.sh

## Show metrics as CSV
metrics-csv:
	@./scripts/metrics.sh --csv

# ═══════════════════════════════════════════════════════════════════════════════
# BOOTSTRAP TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Full bootstrap: regenerate, fix, and build
bootstrap: regen fix verify
	@echo "✓ Bootstrap complete"

## Regenerate Rust files from DOL source
regen:
	@echo "Regenerating Rust files from DOL source..."
	@mkdir -p $(BOOTSTRAP_SRC)
	@for module in $(MODULES); do \
		echo "  Generating $${module}.rs..."; \
		cargo run --release --features cli --bin dol-codegen -- \
			--target rust "$(DOL_DIR)/$${module}.dol" 2>/dev/null \
			> "$(BOOTSTRAP_SRC)/$${module}.rs"; \
	done

## Apply fixes to generated code
fix:
	@./scripts/bootstrap-fix.sh

## Verify bootstrap compiles
verify:
	@cd $(BOOTSTRAP_DIR) && cargo build --release 2>&1 | tail -3

# ═══════════════════════════════════════════════════════════════════════════════
# CHECK TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Check DOL source files parse correctly
check:
	@echo "Checking DOL source files..."
	@cargo run --features cli --bin dol-check -- $(DOL_DIR)/*.dol

## Check bootstrap compiles
check-bootstrap:
	@cd $(BOOTSTRAP_DIR) && cargo check 2>&1 | grep "^error\[" | wc -l | \
		xargs -I {} sh -c 'if [ {} -eq 0 ]; then echo "✓ Bootstrap OK"; else echo "✗ {} errors"; fi'

## Count bootstrap errors
errors:
	@cd $(BOOTSTRAP_DIR) && cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn

## Count raw errors (before fixes)
errors-raw:
	@$(MAKE) regen --quiet
	@cd $(BOOTSTRAP_DIR) && cargo check 2>&1 | grep "^error\[" | wc -l

# ═══════════════════════════════════════════════════════════════════════════════
# TEST TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Run all tests
test:
	@cargo test --lib

## Run tests with output
test-verbose:
	@cargo test --lib -- --nocapture

## Run specific test
test-one:
	@cargo test $(TEST) -- --nocapture

# ═══════════════════════════════════════════════════════════════════════════════
# CLEAN TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Clean all build artifacts
clean:
	@cargo clean
	@rm -rf $(BOOTSTRAP_DIR)/target

## Clean only bootstrap
clean-bootstrap:
	@rm -rf $(BOOTSTRAP_DIR)/target
	@rm -f $(BOOTSTRAP_SRC)/*.rs

## Clean metrics
clean-metrics:
	@rm -rf .metrics

# ═══════════════════════════════════════════════════════════════════════════════
# RELEASE TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

## Build release
release:
	@cargo build --release

## Create a new version tag
tag:
	@echo "Current tags:"
	@git tag -l "v*" | tail -5
	@echo ""
	@read -p "New version (e.g., 0.3.2): " ver; \
		git tag -a "v$$ver" -m "DOL v$$ver"
	@echo "Created tag. Push with: git push origin --tags"

# ═══════════════════════════════════════════════════════════════════════════════
# HELP
# ═══════════════════════════════════════════════════════════════════════════════

## Show this help
help:
	@echo ""
	@echo "DOL Makefile - Flywheel Edition"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo ""
	@echo "Flywheel (recommended):"
	@echo "  make flywheel         Run complete pipeline"
	@echo "  make flywheel-quick   Skip tests"
	@echo "  make metrics          Show progress dashboard"
	@echo ""
	@echo "Bootstrap:"
	@echo "  make bootstrap        Regenerate + fix + build"
	@echo "  make regen            Regenerate Rust from DOL"
	@echo "  make fix              Apply codegen fixes"
	@echo ""
	@echo "Check:"
	@echo "  make check            Parse DOL source"
	@echo "  make errors           Show bootstrap error breakdown"
	@echo "  make errors-raw       Count errors before fixes"
	@echo ""
	@echo "Test:"
	@echo "  make test             Run all tests"
	@echo "  make test-verbose     Tests with output"
	@echo ""
	@echo "Other:"
	@echo "  make clean            Clean all"
	@echo "  make release          Build release"
	@echo "  make tag              Create version tag"
	@echo ""

#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# HIR v0.4.0 COMPLETION CHECKLIST
# ═══════════════════════════════════════════════════════════════════════════════
#
# Run this before merging feature/hir-clean-v0.4.0 to main
#
# ═══════════════════════════════════════════════════════════════════════════════

set -e

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "              HIR v0.4.0 COMPLETION CHECKLIST"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

PASS=0
FAIL=0
WARN=0

check_pass() {
    echo "  ✅ $1"
    PASS=$((PASS + 1))
}

check_fail() {
    echo "  ❌ $1"
    FAIL=$((FAIL + 1))
}

check_warn() {
    echo "  ⚠️  $1"
    WARN=$((WARN + 1))
}

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 1: COMPILATION
# ─────────────────────────────────────────────────────────────────────────────────
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 1. COMPILATION                                                              │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 1.1 Zero errors
ERRORS=$(cargo check 2>&1 | grep "^error\[" | wc -l)
if [ "$ERRORS" -eq 0 ]; then
    check_pass "Zero compilation errors"
else
    check_fail "Has $ERRORS compilation errors"
fi

# 1.2 Zero warnings (optional but good)
WARNINGS=$(cargo check 2>&1 | grep "^warning:" | wc -l)
if [ "$WARNINGS" -eq 0 ]; then
    check_pass "Zero compilation warnings"
else
    check_warn "Has $WARNINGS compilation warnings (non-blocking)"
fi

# 1.3 Clippy passes
if cargo clippy --quiet 2>&1 | grep -q "^error"; then
    check_fail "Clippy has errors"
else
    check_pass "Clippy passes"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 2: TESTS
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 2. TESTS                                                                    │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 2.1 Library tests pass
if cargo test --lib --quiet 2>&1 | grep -q "^test result: ok"; then
    LIB_TESTS=$(cargo test --lib 2>&1 | grep "passed" | tail -1 | grep -oP '\d+ passed' | grep -oP '\d+')
    check_pass "Library tests pass ($LIB_TESTS tests)"
else
    check_fail "Library tests fail"
fi

# 2.2 Doc tests pass
if cargo test --doc --quiet 2>&1 | grep -q "^test result: ok"; then
    DOC_TESTS=$(cargo test --doc 2>&1 | grep "passed" | tail -1 | grep -oP '\d+ passed' | grep -oP '\d+')
    check_pass "Doc tests pass ($DOC_TESTS tests)"
else
    check_fail "Doc tests fail"
fi

# 2.3 HIR-specific tests exist
HIR_TESTS=$(cargo test hir 2>&1 | grep -oP '\d+ passed' | head -1 | grep -oP '\d+' || echo "0")
if [ "$HIR_TESTS" -gt 0 ]; then
    check_pass "HIR-specific tests exist ($HIR_TESTS tests)"
else
    check_warn "No HIR-specific tests found"
fi

# 2.4 Validation tests exist
VAL_TESTS=$(cargo test validate 2>&1 | grep -oP '\d+ passed' | head -1 | grep -oP '\d+' || echo "0")
if [ "$VAL_TESTS" -gt 0 ]; then
    check_pass "Validation tests exist ($VAL_TESTS tests)"
else
    check_warn "No validation-specific tests found"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 3: HIR MODULE COMPLETENESS
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 3. HIR MODULE COMPLETENESS                                                  │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 3.1 Core files exist
for f in types.rs mod.rs validate.rs desugar.rs; do
    if [ -f "src/hir/$f" ]; then
        LINES=$(wc -l < "src/hir/$f")
        check_pass "src/hir/$f exists ($LINES lines)"
    else
        check_fail "src/hir/$f missing"
    fi
done

# 3.2 No disabled files
DISABLED=$(find src -name "*.disabled" | wc -l)
if [ "$DISABLED" -eq 0 ]; then
    check_pass "No .disabled files in src/"
else
    check_fail "Found $DISABLED .disabled files"
fi

# 3.3 No todo!() in HIR code
TODOS=$(grep -r "todo!()" src/hir/ 2>/dev/null | wc -l)
if [ "$TODOS" -eq 0 ]; then
    check_pass "No todo!() in HIR modules"
else
    check_warn "Found $TODOS todo!() in HIR (review needed)"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 4: LOWERING PIPELINE
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 4. LOWERING PIPELINE                                                        │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 4.1 Lower module files exist
for f in mod.rs desugar.rs; do
    if [ -f "src/lower/$f" ]; then
        check_pass "src/lower/$f exists"
    else
        check_fail "src/lower/$f missing"
    fi
done

# 4.2 Lower exports work
if grep -q "pub use.*lower" src/lower/mod.rs 2>/dev/null; then
    check_pass "Lowering exports defined"
else
    check_warn "Check lowering exports"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 5: CODEGEN INTEGRATION
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 5. CODEGEN INTEGRATION                                                      │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 5.1 HIR codegen exists and exported
if [ -f "src/codegen/hir_rust.rs" ]; then
    LINES=$(wc -l < "src/codegen/hir_rust.rs")
    check_pass "src/codegen/hir_rust.rs exists ($LINES lines)"
else
    check_fail "src/codegen/hir_rust.rs missing"
fi

if grep -q "pub mod hir_rust" src/codegen/mod.rs 2>/dev/null; then
    check_pass "hir_rust module exported"
else
    check_fail "hir_rust module not exported"
fi

if grep -q "pub use hir_rust" src/codegen/mod.rs 2>/dev/null; then
    check_pass "HirRustCodegen exported"
else
    check_warn "HirRustCodegen not re-exported (may be ok)"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 6: DOCUMENTATION
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 6. DOCUMENTATION                                                            │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 6.1 HIR specification exists
if [ -f "docs/hir/HIR-SPECIFICATION.md" ]; then
    LINES=$(wc -l < "docs/hir/HIR-SPECIFICATION.md")
    check_pass "HIR-SPECIFICATION.md exists ($LINES lines)"
else
    check_fail "HIR-SPECIFICATION.md missing"
fi

# 6.2 Spec has key sections
if [ -f "docs/hir/HIR-SPECIFICATION.md" ]; then
    for section in "Purpose" "Type Hierarchy" "Desugaring" "Codegen"; do
        if grep -qi "$section" docs/hir/HIR-SPECIFICATION.md; then
            check_pass "Spec has $section section"
        else
            check_warn "Spec missing $section section"
        fi
    done
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 7: SELF-VALIDATION
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 7. SELF-VALIDATION (DOL compiles DOL)                                       │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 7.1 DOL files exist
DOL_COUNT=$(find dol -name "*.dol" 2>/dev/null | wc -l)
if [ "$DOL_COUNT" -gt 0 ]; then
    check_pass "Found $DOL_COUNT DOL specification files"
else
    check_fail "No DOL files found in dol/"
fi

# 7.2 dol-check validates all DOL files
if cargo run --bin dol-check --features cli -- dol/ 2>&1 | grep -q "Passed.*$DOL_COUNT"; then
    check_pass "All DOL files pass validation"
else
    # Check for any failures
    FAILED=$(cargo run --bin dol-check --features cli -- dol/ 2>&1 | grep -oP 'Failed:\s+\d+' | grep -oP '\d+' || echo "0")
    if [ "$FAILED" -eq 0 ]; then
        check_pass "DOL validation passes (with warnings)"
    else
        check_fail "$FAILED DOL files fail validation"
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SECTION 8: GIT STATUS
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────────────────────┐"
echo "│ 8. GIT STATUS                                                               │"
echo "└─────────────────────────────────────────────────────────────────────────────┘"

# 8.1 On correct branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" = "feature/hir-clean-v0.4.0" ]; then
    check_pass "On feature/hir-clean-v0.4.0 branch"
else
    check_warn "On $BRANCH (expected feature/hir-clean-v0.4.0)"
fi

# 8.2 Clean working directory
if [ -z "$(git status --porcelain)" ]; then
    check_pass "Working directory clean"
else
    check_warn "Uncommitted changes present"
fi

# 8.3 Ahead of main
AHEAD=$(git rev-list main..HEAD --count 2>/dev/null || echo "?")
if [ "$AHEAD" != "?" ] && [ "$AHEAD" -gt 0 ]; then
    check_pass "Branch has $AHEAD commits ahead of main"
else
    check_warn "Branch status unclear"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# SUMMARY
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                              SUMMARY"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "  ✅ Passed:   $PASS"
echo "  ⚠️  Warnings: $WARN"
echo "  ❌ Failed:   $FAIL"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "┌─────────────────────────────────────────────────────────────────────────────┐"
    echo "│                    ✅ READY TO MERGE TO MAIN                               │"
    echo "└─────────────────────────────────────────────────────────────────────────────┘"
    echo ""
    echo "Merge commands:"
    echo "  git checkout main"
    echo "  git merge --no-ff feature/hir-clean-v0.4.0 -m \"feat(hir): HIR v0.4.0 implementation\""
    echo "  git tag -a v0.4.0 -m \"HIR v0.4.0 - Clean implementation\""
    echo "  git push origin main --tags"
    echo ""
    exit 0
else
    echo "┌─────────────────────────────────────────────────────────────────────────────┐"
    echo "│                    ❌ NOT READY - FIX FAILURES FIRST                       │"
    echo "└─────────────────────────────────────────────────────────────────────────────┘"
    echo ""
    exit 1
fi

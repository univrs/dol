#!/bin/bash
# =============================================================================
# DOL Spirit Test Runner
# =============================================================================
# Runs test fixtures for the module and spirits system.
#
# Usage:
#   ./scripts/run_spirit_tests.sh              # Run all tests
#   ./scripts/run_spirit_tests.sh --category modules  # Run specific category
#   ./scripts/run_spirit_tests.sh --report     # Generate test report
#   ./scripts/run_spirit_tests.sh --verbose    # Verbose output
#
# Test Categories:
#   - modules     : Module declaration and path resolution tests
#   - visibility  : Visibility modifier tests (pub, pub(spirit), pub(parent))
#   - imports     : Import resolution tests (local, registry, git)
#   - compilation : Spirit and System compilation tests
#   - errors      : Error case validation tests
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TESTS_DIR="$PROJECT_ROOT/tests/spirits"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Options
CATEGORY=""
VERBOSE=false
REPORT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --category|-c)
            CATEGORY="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --report|-r)
            REPORT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--category <name>] [--verbose] [--report]"
            echo ""
            echo "Options:"
            echo "  --category, -c <name>  Run tests for specific category"
            echo "  --verbose, -v          Show detailed output"
            echo "  --report, -r           Generate test report"
            echo "  --help, -h             Show this help"
            echo ""
            echo "Categories: modules, visibility, imports, compilation, errors"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Print header
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}       DOL Spirit Test Runner              ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Function to run a single test file
run_test() {
    local file="$1"
    local category="$2"
    local filename=$(basename "$file")
    local expected_result="pass"

    # Check if this is an error test (expected to fail)
    if [[ "$category" == "errors" ]]; then
        expected_result="fail"
    fi

    if $VERBOSE; then
        echo -e "  ${BLUE}Running:${NC} $filename"
    fi

    # Try to parse the file with dol-parse
    if cargo run --bin dol-parse -- "$file" > /dev/null 2>&1; then
        if [[ "$expected_result" == "pass" ]]; then
            echo -e "  ${GREEN}✓${NC} $filename"
            ((PASSED++))
        else
            echo -e "  ${RED}✗${NC} $filename (expected to fail but passed)"
            ((FAILED++))
        fi
    else
        if [[ "$expected_result" == "fail" ]]; then
            echo -e "  ${GREEN}✓${NC} $filename (correctly rejected)"
            ((PASSED++))
        else
            echo -e "  ${RED}✗${NC} $filename"
            ((FAILED++))
        fi
    fi
}

# Function to run tests for a category
run_category() {
    local category="$1"
    local category_dir="$TESTS_DIR/$category"

    if [[ ! -d "$category_dir" ]]; then
        echo -e "${YELLOW}Warning: Category directory not found: $category${NC}"
        return
    fi

    echo -e "${BLUE}[$category]${NC}"

    for file in "$category_dir"/*.dol; do
        if [[ -f "$file" ]]; then
            run_test "$file" "$category"
        fi
    done

    echo ""
}

# Main test execution
if [[ -n "$CATEGORY" ]]; then
    # Run specific category
    run_category "$CATEGORY"
else
    # Run all categories
    for category in modules visibility imports compilation errors; do
        run_category "$category"
    done
fi

# Print summary
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}                Summary                    ${NC}"
echo -e "${BLUE}============================================${NC}"
echo -e "  ${GREEN}Passed:${NC}  $PASSED"
echo -e "  ${RED}Failed:${NC}  $FAILED"
echo -e "  ${YELLOW}Skipped:${NC} $SKIPPED"
echo ""

TOTAL=$((PASSED + FAILED))
if [[ $TOTAL -gt 0 ]]; then
    PERCENT=$((PASSED * 100 / TOTAL))
    echo -e "  Pass Rate: ${PERCENT}%"
fi

# Generate report if requested
if $REPORT; then
    REPORT_FILE="$PROJECT_ROOT/target/spirit_test_report.txt"
    mkdir -p "$(dirname "$REPORT_FILE")"

    {
        echo "DOL Spirit Test Report"
        echo "======================"
        echo "Date: $(date)"
        echo ""
        echo "Results:"
        echo "  Passed:  $PASSED"
        echo "  Failed:  $FAILED"
        echo "  Skipped: $SKIPPED"
        echo ""
        echo "Categories Tested:"
        for category in modules visibility imports compilation errors; do
            count=$(find "$TESTS_DIR/$category" -name "*.dol" 2>/dev/null | wc -l)
            echo "  - $category: $count files"
        done
    } > "$REPORT_FILE"

    echo -e "${GREEN}Report generated:${NC} $REPORT_FILE"
fi

# Exit with appropriate code
if [[ $FAILED -gt 0 ]]; then
    exit 1
fi
exit 0

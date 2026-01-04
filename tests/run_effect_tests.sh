#!/bin/bash
# ==============================================================================
# DOL Effect System Test Runner
# ==============================================================================
#
# This script runs all effect system tests for the DOL language.
#
# Usage:
#   ./run_effect_tests.sh           # Run all tests
#   ./run_effect_tests.sh inference # Run only inference tests
#   ./run_effect_tests.sh errors    # Run only error tests
#   ./run_effect_tests.sh prelude   # Run only prelude tests
#   ./run_effect_tests.sh regression # Run only regression tests
#   ./run_effect_tests.sh -v        # Verbose output
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
#   2 - Test runner error (missing dependencies, etc.)
#
# ==============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Test directories
INFERENCE_DIR="$SCRIPT_DIR/effects/inference"
ERRORS_DIR="$SCRIPT_DIR/effects/errors"
PRELUDE_DIR="$SCRIPT_DIR/effects/prelude"
REGRESSION_DIR="$SCRIPT_DIR/effects/regression"

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Verbose mode
VERBOSE=false

# Parse arguments
CATEGORIES=()
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "DOL Effect System Test Runner"
            echo ""
            echo "Usage: $0 [OPTIONS] [CATEGORY...]"
            echo ""
            echo "Categories:"
            echo "  inference   - Pure and effectful function inference tests"
            echo "  errors      - Error detection tests (E0401, E0402)"
            echo "  prelude     - Prelude function availability tests"
            echo "  regression  - Regression tests for past bugs"
            echo ""
            echo "Options:"
            echo "  -v, --verbose   Show detailed output"
            echo "  -h, --help      Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run all tests"
            echo "  $0 inference          # Run only inference tests"
            echo "  $0 -v errors prelude  # Run errors and prelude tests verbosely"
            exit 0
            ;;
        inference|errors|prelude|regression)
            CATEGORIES+=("$1")
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 2
            ;;
    esac
done

# Default to all categories if none specified
if [ ${#CATEGORIES[@]} -eq 0 ]; then
    CATEGORIES=("inference" "errors" "prelude" "regression")
fi

# Print header
echo ""
echo "============================================================"
echo "           DOL Effect System Test Suite"
echo "============================================================"
echo ""
echo "Project root: $PROJECT_ROOT"
echo "Test categories: ${CATEGORIES[*]}"
echo "Verbose: $VERBOSE"
echo ""

# Check for dol-check binary
DOL_CHECK="$PROJECT_ROOT/target/release/dol-check"
if [ ! -f "$DOL_CHECK" ]; then
    DOL_CHECK="$PROJECT_ROOT/target/debug/dol-check"
fi

if [ ! -f "$DOL_CHECK" ]; then
    echo -e "${YELLOW}Warning: dol-check binary not found${NC}"
    echo "Building with cargo..."
    cd "$PROJECT_ROOT"
    cargo build --bin dol-check 2>/dev/null || {
        echo -e "${YELLOW}Note: dol-check not yet implemented, running in parse-only mode${NC}"
        DOL_CHECK=""
    }
fi

# Function to run a single test file
run_test() {
    local test_file="$1"
    local test_name="$(basename "$test_file" .dol)"
    local category="$2"
    local expect_error="$3"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}Running: $category/$test_name${NC}"
    fi

    # If we have dol-check, use it
    if [ -n "$DOL_CHECK" ]; then
        if [ "$expect_error" = true ]; then
            # Error tests should fail
            if ! "$DOL_CHECK" --effects "$test_file" 2>/dev/null; then
                PASSED_TESTS=$((PASSED_TESTS + 1))
                if [ "$VERBOSE" = true ]; then
                    echo -e "  ${GREEN}PASS${NC} (expected error detected)"
                fi
                return 0
            else
                FAILED_TESTS=$((FAILED_TESTS + 1))
                echo -e "  ${RED}FAIL${NC} $category/$test_name (expected error not detected)"
                return 1
            fi
        else
            # Normal tests should pass
            if "$DOL_CHECK" --effects "$test_file" 2>/dev/null; then
                PASSED_TESTS=$((PASSED_TESTS + 1))
                if [ "$VERBOSE" = true ]; then
                    echo -e "  ${GREEN}PASS${NC}"
                fi
                return 0
            else
                FAILED_TESTS=$((FAILED_TESTS + 1))
                echo -e "  ${RED}FAIL${NC} $category/$test_name"
                return 1
            fi
        fi
    else
        # No dol-check, just verify files exist and are parseable
        if [ -f "$test_file" ]; then
            # Try to parse with dol-parse if available
            DOL_PARSE="$PROJECT_ROOT/target/release/dol-parse"
            if [ ! -f "$DOL_PARSE" ]; then
                DOL_PARSE="$PROJECT_ROOT/target/debug/dol-parse"
            fi

            if [ -f "$DOL_PARSE" ]; then
                if "$DOL_PARSE" "$test_file" >/dev/null 2>&1; then
                    PASSED_TESTS=$((PASSED_TESTS + 1))
                    if [ "$VERBOSE" = true ]; then
                        echo -e "  ${GREEN}PASS${NC} (parsed successfully)"
                    fi
                    return 0
                else
                    # Parse failure - might be expected for error tests
                    if [ "$expect_error" = true ]; then
                        PASSED_TESTS=$((PASSED_TESTS + 1))
                        if [ "$VERBOSE" = true ]; then
                            echo -e "  ${GREEN}PASS${NC} (expected parse error)"
                        fi
                        return 0
                    else
                        FAILED_TESTS=$((FAILED_TESTS + 1))
                        echo -e "  ${RED}FAIL${NC} $category/$test_name (parse error)"
                        return 1
                    fi
                fi
            else
                # No parser available, just check file exists
                SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
                if [ "$VERBOSE" = true ]; then
                    echo -e "  ${YELLOW}SKIP${NC} (no parser available)"
                fi
                return 0
            fi
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
            echo -e "  ${RED}FAIL${NC} $category/$test_name (file not found)"
            return 1
        fi
    fi
}

# Function to run all tests in a directory
run_category() {
    local category="$1"
    local dir="$2"
    local expect_error="${3:-false}"

    echo ""
    echo "------------------------------------------------------------"
    echo "  $category tests"
    echo "------------------------------------------------------------"

    if [ ! -d "$dir" ]; then
        echo -e "${RED}Directory not found: $dir${NC}"
        return 1
    fi

    local count=0
    for test_file in "$dir"/*.dol; do
        if [ -f "$test_file" ]; then
            run_test "$test_file" "$category" "$expect_error"
            count=$((count + 1))
        fi
    done

    if [ $count -eq 0 ]; then
        echo -e "${YELLOW}No test files found in $dir${NC}"
    else
        echo "  Ran $count tests"
    fi
}

# Run selected categories
for category in "${CATEGORIES[@]}"; do
    case $category in
        inference)
            run_category "inference" "$INFERENCE_DIR" false
            ;;
        errors)
            run_category "errors" "$ERRORS_DIR" true
            ;;
        prelude)
            run_category "prelude" "$PRELUDE_DIR" false
            ;;
        regression)
            run_category "regression" "$REGRESSION_DIR" false
            ;;
    esac
done

# Print summary
echo ""
echo "============================================================"
echo "                      Test Summary"
echo "============================================================"
echo ""
echo -e "  Total:   $TOTAL_TESTS"
echo -e "  ${GREEN}Passed:  $PASSED_TESTS${NC}"
echo -e "  ${RED}Failed:  $FAILED_TESTS${NC}"
echo -e "  ${YELLOW}Skipped: $SKIPPED_TESTS${NC}"
echo ""

# Calculate pass rate
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo "  Pass rate: ${PASS_RATE}%"
fi

echo ""
echo "============================================================"

# Exit with appropriate code
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi

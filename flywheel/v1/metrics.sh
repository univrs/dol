#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# DOL FLYWHEEL METRICS DASHBOARD
# ═══════════════════════════════════════════════════════════════════════════════
#
# Displays flywheel progress metrics over time.
#
# Usage:
#   ./scripts/metrics.sh [--last N] [--csv]
#
# ═══════════════════════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
METRICS_FILE="$PROJECT_ROOT/.metrics/flywheel.local.csv"

LAST=10
CSV=false

for arg in "$@"; do
    case $arg in
        --last=*) LAST="${arg#*=}" ;;
        --last) shift; LAST="$1" ;;
        --csv) CSV=true ;;
    esac
done

if [ ! -f "$METRICS_FILE" ]; then
    echo "No metrics recorded yet. Run: ./scripts/flywheel.sh"
    exit 0
fi

if $CSV; then
    echo "timestamp,source,dol_files,dol_lines,raw_errors,fixed_errors,tests_passed,tests_total,duration"
    tail -n "$LAST" "$METRICS_FILE"
    exit 0
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                    📊 DOL FLYWHEEL METRICS                                    "
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# Current status
LATEST=$(tail -1 "$METRICS_FILE")
IFS=',' read -r TIMESTAMP SOURCE DOL_FILES DOL_LINES RAW_ERRORS FIXED_ERRORS TESTS_PASSED TESTS_TOTAL DURATION <<< "$LATEST"

echo "Latest Run: $TIMESTAMP"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│                    CURRENT STATUS                          │"
echo "├─────────────────────────────────────────────────────────────┤"
printf "│  DOL Source:       %6s files, %6s lines              │\n" "$DOL_FILES" "$DOL_LINES"
printf "│  Raw Errors:       %6s                                  │\n" "$RAW_ERRORS"
printf "│  Fixed Errors:     %6s                                  │\n" "$FIXED_ERRORS"
printf "│  Tests:            %6s / %s                            │\n" "$TESTS_PASSED" "$TESTS_TOTAL"
printf "│  Duration:         %6ss                                 │\n" "$DURATION"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""

# Progress indicators
if [ "$RAW_ERRORS" -gt 0 ] && [ "$FIXED_ERRORS" -eq 0 ]; then
    echo "🔧 Fix Script: REQUIRED"
    echo "   Codegen produces $RAW_ERRORS errors, fix script resolves all"
    echo ""
    echo "📈 Next Goal: Implement codegen fixes to reduce raw errors"
elif [ "$RAW_ERRORS" -eq 0 ]; then
    echo "🎉 Fix Script: NOT NEEDED"
    echo "   Codegen produces clean output!"
    echo ""
    echo "📈 Next Goal: Add more DOL modules (parser, codegen)"
else
    REMAINING=$FIXED_ERRORS
    echo "⚠️ Fix Script: PARTIAL"
    echo "   Reduces errors from $RAW_ERRORS to $FIXED_ERRORS"
    echo ""
    echo "📈 Next Goal: Fix remaining $REMAINING errors"
fi

echo ""

# Trend analysis
RECORD_COUNT=$(wc -l < "$METRICS_FILE")
if [ "$RECORD_COUNT" -gt 1 ]; then
    echo "┌─────────────────────────────────────────────────────────────┐"
    echo "│                    RECENT HISTORY                          │"
    echo "├─────────────────────────────────────────────────────────────┤"
    echo "│  Timestamp            Raw    Fixed   Tests                 │"
    echo "├─────────────────────────────────────────────────────────────┤"
    
    tail -n "$LAST" "$METRICS_FILE" | while IFS=',' read -r TS SRC DF DL RE FE TP TT DUR; do
        # Truncate timestamp
        SHORT_TS=$(echo "$TS" | cut -c1-19)
        printf "│  %-19s  %5s  %5s   %s/%s                │\n" "$SHORT_TS" "$RE" "$FE" "$TP" "$TT"
    done
    
    echo "└─────────────────────────────────────────────────────────────┘"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"

# Self-hosting progress
echo ""
echo "🔄 SELF-HOSTING PROGRESS"
echo ""
echo "  Phase 1: Bootstrap Compiles     ✅ COMPLETE"
echo "  Phase 2: No Fix Script Needed   ⏳ IN PROGRESS ($RAW_ERRORS errors to fix)"
echo "  Phase 3: Parser in DOL          ⬜ TODO"
echo "  Phase 4: Full Self-Hosting      ⬜ TODO"
echo ""

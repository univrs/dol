#!/usr/bin/env bash
# DOL Lean verification gate (docs/design/DOL-LEAN4-ALIGNMENT.md, Phase L2).
#   scripts/verify.sh              build DolLean and reject sorry / non-standard axioms
#   scripts/verify.sh --self-test  check that the gate rejects bad fixtures
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.elan/bin:$PATH"

gate() { lake env lean --run scripts/CheckAxioms.lean "$1"; }

if [[ "${1:-}" == "--self-test" ]]; then
  lake build GateFixtures.Good GateFixtures.BadSorry GateFixtures.BadAxiom \
    GateFixtures.BadNativeDecide GateFixtures.BadImplementedBy GateFixtures.BadExtern >/dev/null 2>&1 || true
  fail=0
  gate GateFixtures.Good >/dev/null || { echo "✗ gate rejected a proved theorem"; fail=1; }
  if gate GateFixtures.BadSorry >/dev/null 2>&1; then echo "✗ gate accepted sorry"; fail=1; fi
  if gate GateFixtures.BadAxiom >/dev/null 2>&1; then echo "✗ gate accepted a custom axiom"; fail=1; fi
  if gate GateFixtures.BadNativeDecide >/dev/null 2>&1; then echo "✗ gate accepted native_decide"; fail=1; fi
  if gate GateFixtures.BadImplementedBy >/dev/null 2>&1; then echo "✗ gate accepted @[implemented_by]"; fail=1; fi
  if gate GateFixtures.BadExtern >/dev/null 2>&1; then echo "✗ gate accepted @[extern]"; fail=1; fi
  [[ $fail == 0 ]] && echo "✓ gate self-test passed (accepts proofs; rejects sorry, custom axioms, native_decide, implemented_by, extern)"
  exit $fail
fi

lake build DolLean
gate DolLean

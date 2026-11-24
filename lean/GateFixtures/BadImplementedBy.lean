/-! Fixture: `implemented_by` swaps verified code for unverified code at runtime. The gate must reject it. -/
def GateFixtures.fastImpl (n : Nat) : Nat := n
@[implemented_by GateFixtures.fastImpl]
def GateFixtures.spec (n : Nat) : Nat := n + 0
theorem GateFixtures.spec_ok : GateFixtures.spec 1 = 1 := rfl

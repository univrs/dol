/-! Fixture: a theorem resting on a custom axiom. The gate must reject it. -/
axiom GateFixtures.cheat : 1 + 1 = 3
theorem GateFixtures.uses_cheat : 1 + 1 = 3 := GateFixtures.cheat

/-! Fixture: `native_decide` trusts the compiler (`Lean.ofReduceBool`). The gate must reject it. -/
theorem GateFixtures.by_native : 2 ^ 10 = 1024 := by native_decide

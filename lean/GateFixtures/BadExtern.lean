/-! Fixture: `@[extern]` binds to unverified native code. The gate must reject it. -/
@[extern "dol_fake_native"]
def GateFixtures.native (n : Nat) : Nat := n

import Lean

/-!
Verification gate: every theorem in modules under `DolLean` may depend only on
Lean's standard axioms. `sorry` shows up as `sorryAx` and fails the gate.
Usage: `lake env lean --run scripts/CheckAxioms.lean [RootModule]`
-/
open Lean

def allowed : List Name := [``propext, ``Classical.choice, ``Quot.sound]

unsafe def main (args : List String) : IO UInt32 := do
  let root := (args.headD "DolLean").toName
  initSearchPath (← findSysroot)
  let env ← importModules #[{ module := root }] {} (trustLevel := 0)
  let mut bad := 0
  let mut n := 0
  for (name, ci) in env.constants.toList do
    let some idx := env.getModuleIdxFor? name | continue
    let modName := env.header.moduleNames[idx.toNat]!
    unless root.isPrefixOf modName do continue
    if let .thmInfo _ := ci then
      n := n + 1
      let (axs, _) ← (collectAxioms name : CoreM (Array Name)).toIO
        { fileName := "<check-axioms>", fileMap := default } { env := env }
      for ax in axs do
        unless allowed.contains ax do
          IO.eprintln s!"✗ {name} depends on {ax}"
          bad := bad + 1
  IO.println s!"checked {n} theorems under {root}: {bad} violation(s)"
  return if bad == 0 && n > 0 then 0 else 1

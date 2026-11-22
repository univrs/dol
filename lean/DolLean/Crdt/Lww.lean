/-!
# Last-writer-wins register

Lean model of DOL's `@crdt(lww)` strategy. Commutativity holds only for
`Coherent` pairs: one (ts, replica) pair must identify exactly one write
(see `not_comm_without_coherence`).
-/

namespace DolLean

/-- A last-writer-wins register: highest (timestamp, replica) wins. -/
structure Lww (α : Type) where
  ts      : Nat
  replica : Nat
  value   : α
deriving Repr, DecidableEq

namespace Lww
variable {α : Type}

/-- Lexicographic "later than or equal" on (ts, replica). -/
def le (a b : Lww α) : Prop := a.ts < b.ts ∨ (a.ts = b.ts ∧ a.replica ≤ b.replica)

instance (a b : Lww α) : Decidable (le a b) := by unfold le; infer_instance

/-- Merge keeps the later write. -/
def merge (a b : Lww α) : Lww α := if le a b then b else a

/-- Well-formedness: one (ts, replica) pair identifies one write. -/
def Coherent (a b : Lww α) : Prop := a.ts = b.ts → a.replica = b.replica → a = b

theorem merge_idem (a : Lww α) : merge a a = a := by
  simp [merge]

theorem merge_comm (a b : Lww α) (h : Coherent a b) : merge a b = merge b a := by
  unfold merge le
  by_cases h1 : a.ts < b.ts ∨ a.ts = b.ts ∧ a.replica ≤ b.replica <;>
  by_cases h2 : b.ts < a.ts ∨ b.ts = a.ts ∧ b.replica ≤ a.replica <;>
  simp only [h1, h2, ↓reduceIte]
  all_goals first
    | rfl
    | omega
    | exact (h (by omega) (by omega)).symm
    | exact h (by omega) (by omega)

theorem merge_assoc (a b c : Lww α) :
    merge (merge a b) c = merge a (merge b c) := by
  unfold merge le
  by_cases hab : a.ts < b.ts ∨ a.ts = b.ts ∧ a.replica ≤ b.replica <;>
  by_cases hbc : b.ts < c.ts ∨ b.ts = c.ts ∧ b.replica ≤ c.replica <;>
  by_cases hac : a.ts < c.ts ∨ a.ts = c.ts ∧ a.replica ≤ c.replica <;>
  simp only [hab, hbc, hac, ↓reduceIte] <;> first | rfl | omega

end Lww

/-- Without coherence, merge is not commutative: two distinct writes that share
(ts, replica) are resolved by argument order. -/
theorem Lww.not_comm_without_coherence :
    ¬ ∀ a b : Lww String, Lww.merge a b = Lww.merge b a := by
  intro h
  have := congrArg Lww.value (h ⟨1, 7, "x"⟩ ⟨1, 7, "y"⟩)
  simp [Lww.merge, Lww.le] at this

end DolLean

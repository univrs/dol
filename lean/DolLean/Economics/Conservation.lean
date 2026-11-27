/-!
# Credit Conservation

Lean model of the ENR credit ledger and proof that credits are conserved:
no operation creates credits except an explicit, recorded `fund` (mint).

Mirrors (univrs-enr):
- `CreditConservation` (`src/core/invariants.rs`):
  `sum(all_credits) == genesis + minted - burned`. Here `funded` is the
  conservation anchor (genesis + minted); nothing is burned — consumed and
  fee credits stay accounted in their own buckets.
- `ReservationLedger` (`src/core/reservation.rs`, H1c):
  `spendable + reserved + consumed == funded` after every operation, with
  `reserve` (Active → Reserved), `commit` (Reserved → Consumed) and
  `release` (Reserved → Released → Active).
- `CreditTransfer` (`src/core/types.rs`): debits `amount + entropy_cost`
  from the sender, credits `amount` to the receiver.

Modelling choices:
- Balances are `Nat`, not saturating `u64`; this models the non-overflowing
  regime. (Rust `saturating_add` can silently lose credits at `u64::MAX`.)
- Accounts are indices into `balances`; reservation ids are positions in `held`.
- `CreditTransfer.entropy_cost` is routed to a `revival` bucket. The Rust type
  only defines `total_cost`; the sink is a modelling assumption.
- An expired `commit` in Rust releases instead; here that is just `release`.
- Failed operations return `none` and leave the ledger unchanged (Rust `Err`).
-/

namespace DolLean.Economics

/-! ## Credit lifecycle (`CreditState`, `src/core/state.rs`) -/

-- `genCtorIdx false`: the auto-generated `ctorIdx` carries `@[implemented_by]`,
-- which the verification gate (scripts/CheckAxioms.lean) rejects.
set_option genCtorIdx false in
/-- Credit lifecycle states. -/
inductive CreditState where
  | active | reserved | consumed | released | inRevival

/-- Valid edges; mirrors `CreditState::can_transition_to`. -/
def CreditState.canTransition : CreditState → CreditState → Bool
  | .active, .reserved | .reserved, .consumed | .reserved, .released
  | .released, .active | .consumed, .inRevival | .inRevival, .active => true
  | _, _ => false

/-- The edges used by `reserve`, `commit` and `release` are valid, and no credit
can be spent from `reserved` without passing through `consumed` or `released`. -/
theorem lifecycle_edges :
    CreditState.canTransition .active .reserved = true ∧
    CreditState.canTransition .reserved .consumed = true ∧
    CreditState.canTransition .reserved .released = true ∧
    CreditState.canTransition .released .active = true ∧
    CreditState.canTransition .reserved .active = false ∧
    CreditState.canTransition .consumed .active = false := by
  decide

/-! ## Balance lists -/

/-- Sum of a list of balances. -/
def sum : List Nat → Nat
  | [] => 0
  | b :: bs => b + sum bs

/-- Balance of account `i` (`0` if unknown). -/
def bal : List Nat → Nat → Nat
  | [], _ => 0
  | b :: _, 0 => b
  | _ :: bs, i + 1 => bal bs i

/-- Add `n` to account `i` (no-op if unknown). -/
def credit : List Nat → Nat → Nat → List Nat
  | [], _, _ => []
  | b :: bs, 0, n => (b + n) :: bs
  | b :: bs, i + 1, n => b :: credit bs i n

/-- Subtract `n` from account `i` (no-op if unknown). -/
def debit : List Nat → Nat → Nat → List Nat
  | [], _, _ => []
  | b :: bs, 0, n => (b - n) :: bs
  | b :: bs, i + 1, n => b :: debit bs i n

theorem sum_credit : ∀ (l : List Nat) (i n : Nat), i < l.length →
    sum (credit l i n) = sum l + n
  | [], _, _, h => absurd h (Nat.not_lt_zero _)
  | b :: bs, 0, n, _ => by simp only [credit, sum]; omega
  | b :: bs, i + 1, n, h => by
    have := sum_credit bs i n (by simp at h; omega)
    simp only [credit, sum]; omega

theorem sum_debit : ∀ (l : List Nat) (i n : Nat), n ≤ bal l i →
    sum (debit l i n) + n = sum l
  | [], _, _, h => by simp only [bal] at h; simp only [debit, sum]; omega
  | b :: bs, 0, n, h => by simp only [bal] at h; simp only [debit, sum]; omega
  | b :: bs, i + 1, n, h => by
    have := sum_debit bs i n (by simpa only [bal] using h)
    simp only [debit, sum]; omega

theorem length_credit : ∀ (l : List Nat) (i n : Nat), (credit l i n).length = l.length
  | [], _, _ => rfl
  | _ :: _, 0, _ => rfl
  | _ :: bs, i + 1, n => by simp [credit, length_credit bs i n]

theorem length_debit : ∀ (l : List Nat) (i n : Nat), (debit l i n).length = l.length
  | [], _, _ => rfl
  | _ :: _, 0, _ => rfl
  | _ :: bs, i + 1, n => by simp [debit, length_debit bs i n]

/-! ## Held reservations -/

/-- Total amount held in reservations `(account, amount)`. -/
def heldSum : List (Nat × Nat) → Nat
  | [] => 0
  | r :: rs => r.2 + heldSum rs

/-- Remove reservation `k`, returning it and the remaining reservations. -/
def pick : List (Nat × Nat) → Nat → Option ((Nat × Nat) × List (Nat × Nat))
  | [], _ => none
  | r :: rs, 0 => some (r, rs)
  | r :: rs, k + 1 =>
    match pick rs k with
    | some (x, rest) => some (x, r :: rest)
    | none => none

theorem heldSum_pick : ∀ (l : List (Nat × Nat)) (k : Nat) (x : Nat × Nat)
    (rest : List (Nat × Nat)), pick l k = some (x, rest) → heldSum l = x.2 + heldSum rest
  | [], _, _, _, h => by simp [pick] at h
  | r :: rs, 0, x, rest, h => by
    simp only [pick, Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    rfl
  | r :: rs, k + 1, x, rest, h => by
    simp only [pick] at h
    split at h
    · rename_i x' rest' hp
      simp only [Option.some.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      have := heldSum_pick rs k x' rest' hp
      simp only [heldSum]; omega
    · exact absurd h (by simp)

theorem heldSum_append (l : List (Nat × Nat)) (r : Nat × Nat) :
    heldSum (l ++ [r]) = heldSum l + r.2 := by
  induction l with
  | nil => simp [heldSum]
  | cons a as ih => simp only [List.cons_append, heldSum, ih]; omega

/-! ## Ledger and operations -/

/-- Global credit ledger. -/
structure Ledger where
  /-- Active (spendable) balance per account. -/
  balances : List Nat
  /-- Held reservations `(account, amount)` — the Reserved state. -/
  held     : List (Nat × Nat)
  /-- Credits terminally committed (Consumed state). -/
  consumed : Nat
  /-- Entropy fees collected from transfers (revival pool). -/
  revival  : Nat
  /-- Total ever minted/funded — the conservation anchor. -/
  funded   : Nat

/-- Every credit in existence, in any lifecycle state. -/
def total (s : Ledger) : Nat :=
  sum s.balances + heldSum s.held + s.consumed + s.revival

/-- Conservation invariant: `spendable + reserved + consumed (+ fees) == funded`.
Mirrors `ReservationLedger::conservation_holds` / `CreditConservation::check`. -/
def Conserved (s : Ledger) : Prop := total s = s.funded

set_option genCtorIdx false in
/-- Ledger operations. -/
inductive Op where
  /-- Mint `amount` into `acct` (the only source of new credits; `ReservationLedger::fund`). -/
  | fund (acct amount : Nat)
  /-- `CreditTransfer { from, to, amount, entropy_cost }`. -/
  | transfer (src dst amount entropyCost : Nat)
  /-- `ReservationLedger::reserve`: Active → Reserved. -/
  | reserve (acct amount : Nat)
  /-- `ReservationLedger::commit`: Reserved → Consumed. -/
  | commit (id : Nat)
  /-- `ReservationLedger::release`: Reserved → Released → Active. -/
  | release (id : Nat)

/-- Credits an operation is allowed to create. Only `fund` mints. -/
def Op.minted : Op → Nat
  | .fund _ n => n
  | _ => 0

/-- Apply one operation; `none` means rejected (state unchanged). -/
def step (s : Ledger) : Op → Option Ledger
  | .fund a n =>
    if a < s.balances.length then
      some { s with balances := credit s.balances a n, funded := s.funded + n }
    else none
  | .transfer src dst n fee =>
    if dst < s.balances.length ∧ n + fee ≤ bal s.balances src then
      some { s with balances := credit (debit s.balances src (n + fee)) dst n,
                    revival := s.revival + fee }
    else none
  | .reserve a n =>
    if n ≠ 0 ∧ n ≤ bal s.balances a then
      some { s with balances := debit s.balances a n, held := s.held ++ [(a, n)] }
    else none
  | .commit k =>
    match pick s.held k with
    | some ((_, n), rest) => some { s with held := rest, consumed := s.consumed + n }
    | none => none
  | .release k =>
    match pick s.held k with
    | some ((a, n), rest) =>
      if a < s.balances.length then
        some { s with held := rest, balances := credit s.balances a n }
      else none
    | none => none

/-! ## Conservation theorems -/

/-- Every successful step changes the total by exactly what it mints.
Mirrors `CreditConservation`: `sum == genesis + minted - burned`. -/
theorem step_total (s s' : Ledger) (op : Op) (h : step s op = some s') :
    total s' = total s + op.minted := by
  cases op with
  | fund a n =>
    simp only [step] at h
    split at h
    · rename_i ha
      simp only [Option.some.injEq] at h; subst h
      have := sum_credit s.balances a n ha
      simp only [total, Op.minted]; omega
    · simp at h
  | transfer src dst n fee =>
    simp only [step] at h
    split at h
    · rename_i hv
      simp only [Option.some.injEq] at h; subst h
      have h1 := sum_debit s.balances src (n + fee) hv.2
      have h2 := sum_credit (debit s.balances src (n + fee)) dst n
        (by rw [length_debit]; exact hv.1)
      simp only [total, Op.minted]; omega
    · simp at h
  | reserve a n =>
    simp only [step] at h
    split at h
    · rename_i hv
      simp only [Option.some.injEq] at h; subst h
      have h1 := sum_debit s.balances a n hv.2
      have h2 := heldSum_append s.held (a, n)
      simp only [total, Op.minted]; omega
    · simp at h
  | commit k =>
    simp only [step] at h
    split at h
    · rename_i a n rest hp
      simp only [Option.some.injEq] at h; subst h
      have := heldSum_pick s.held k (a, n) rest hp
      simp only [total, Op.minted]; omega
    · simp at h
  | release k =>
    simp only [step] at h
    split at h
    · rename_i a n rest hp
      split at h
      · rename_i ha
        simp only [Option.some.injEq] at h; subst h
        have h1 := heldSum_pick s.held k (a, n) rest hp
        have h2 := sum_credit s.balances a n ha
        simp only [total, Op.minted]; omega
      · simp at h
    · simp at h

/-- A valid `CreditTransfer` preserves the total (amount moves to the receiver,
`entropy_cost` to the revival pool). Mirrors `CreditConservation::check`. -/
theorem transfer_preserves_total (s s' : Ledger) (src dst n fee : Nat)
    (h : step s (.transfer src dst n fee) = some s') : total s' = total s := by
  simpa [Op.minted] using step_total s s' _ h

/-- A transfer exceeding the sender's spendable balance is rejected.
Mirrors `EnrError::InsufficientCredits`. -/
theorem transfer_insufficient_rejected (s : Ledger) (src dst n fee : Nat)
    (h : bal s.balances src < n + fee) : step s (.transfer src dst n fee) = none := by
  simp only [step]
  split
  · rename_i hv; omega
  · rfl

/-- No operation other than an explicit `fund` (mint) changes the total: credits
cannot be created from nothing. -/
theorem no_mint (s s' : Ledger) (op : Op) (hop : op.minted = 0)
    (h : step s op = some s') : total s' = total s := by
  have := step_total s s' op h; omega

/-- No operation creates more credits than it records as minted. -/
theorem no_unrecorded_mint (s s' : Ledger) (op : Op) (h : step s op = some s') :
    total s' ≤ total s + op.minted :=
  Nat.le_of_eq (step_total s s' op h)

/-- The conservation invariant holds after every step (including `fund`, which
raises `funded` by exactly what it mints). Mirrors
`ReservationLedger::conservation_holds` holding after every operation. -/
theorem step_conserved (s s' : Ledger) (op : Op) (hs : Conserved s)
    (h : step s op = some s') : Conserved s' := by
  have ht := step_total s s' op h
  unfold Conserved at *
  cases op <;> simp only [step] at h <;> (split at h <;> try split at h) <;>
    first
    | (simp only [Option.some.injEq] at h; subst h; simp only [Op.minted] at ht ⊢; omega)
    | (simp at h; done)

/-- Reserve followed by commit (Active → Reserved → Consumed) conserves the total.
Mirrors the H1c reserve/commit path. -/
theorem reserve_consume_conserves (s s₁ s₂ : Ledger) (a n k : Nat)
    (h₁ : step s (.reserve a n) = some s₁) (h₂ : step s₁ (.commit k) = some s₂) :
    total s₂ = total s := by
  have e₁ := no_mint s s₁ _ rfl h₁
  have e₂ := no_mint s₁ s₂ _ rfl h₂
  omega

/-- Reserve followed by release (Active → Reserved → Released → Active) conserves
the total. Mirrors the H1c reserve/release path. -/
theorem reserve_release_conserves (s s₁ s₂ : Ledger) (a n k : Nat)
    (h₁ : step s (.reserve a n) = some s₁) (h₂ : step s₁ (.release k) = some s₂) :
    total s₂ = total s := by
  have e₁ := no_mint s s₁ _ rfl h₁
  have e₂ := no_mint s₁ s₂ _ rfl h₂
  omega

/-! ## Traces -/

/-- Run a sequence of operations, skipping rejected ones (Rust returns `Err`
without mutating). -/
def run (s : Ledger) : List Op → Ledger
  | [] => s
  | op :: ops => run ((step s op).getD s) ops

/-- Credits a trace is allowed to mint (an upper bound: rejected `fund`s mint nothing). -/
def mintedBy : List Op → Nat
  | [] => 0
  | op :: ops => op.minted + mintedBy ops

/-- Conservation holds across any trace. -/
theorem run_conserved (s : Ledger) (ops : List Op) (hs : Conserved s) :
    Conserved (run s ops) := by
  induction ops generalizing s with
  | nil => exact hs
  | cons op ops ih =>
    apply ih
    cases h : step s op with
    | none => exact hs
    | some s' => exact step_conserved s s' op hs h

/-- Over any trace, the total never exceeds the starting total plus what the trace
explicitly mints. -/
theorem run_total_le (s : Ledger) (ops : List Op) :
    total (run s ops) ≤ total s + mintedBy ops := by
  induction ops generalizing s with
  | nil => simp [run, mintedBy]
  | cons op ops ih =>
    have := ih ((step s op).getD s)
    have hstep : total ((step s op).getD s) ≤ total s + op.minted := by
      cases h : step s op with
      | none => simp only [Option.getD_none]; omega
      | some s' => simp only [Option.getD_some]; exact no_unrecorded_mint s s' op h
    simp only [run, mintedBy]; omega

/-- A trace with no `fund` operations leaves the total unchanged. -/
theorem run_no_mint (s : Ledger) (ops : List Op) (hops : mintedBy ops = 0) :
    total (run s ops) = total s := by
  induction ops generalizing s with
  | nil => rfl
  | cons op ops ih =>
    simp only [mintedBy] at hops
    have hrest := ih ((step s op).getD s) (by omega)
    simp only [run]; rw [hrest]
    cases h : step s op with
    | none => rfl
    | some s' => exact no_mint s s' op (by omega) h

end DolLean.Economics

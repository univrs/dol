/-!
# Ledger example (docs/design/DOL-LEAN4-ALIGNMENT.md §5, Phase L1)

Hand-written target for what `dol-codegen -t lean` should emit for the
`ledger.account` gen and `ledger.transfer` trait. Kept here so the design
doc's example stays type-checked.
-/

namespace Ledger

/-- A bounded account balance. -/
structure Account where
  owner   : String
  balance : UInt64
  h_balance : balance ≤ 1000000

/-- Transfers move value between accounts without creating or destroying it. -/
class Transfer where
  transfer : Account → Account → UInt64 → Account × Account
  conserves : ∀ (a b : Account) (n : UInt64),
    n ≤ a.balance →
      (transfer a b n).1.balance.toNat + (transfer a b n).2.balance.toNat
        = a.balance.toNat + b.balance.toNat

end Ledger

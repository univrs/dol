# DOL × Lean 4: Design for the Next DOL Feature Set

> **Status:** Accepted direction (2026-11-21). Progress is tracked in §9. Everything else in this document is a proposal unless marked **today**.
> **Inputs:** `Lean4.md` (Lean 4 survey), `docs/DOL-STATUS-2026-11.md` (verified DOL v0.8.2 status), and probes run against v0.8.2 (§2).
> **Scope:** which missing DOL features should be built *by leaning on Lean 4*, what DOL must change to make that possible, and the goal posts.

**About the source material.** `Lean4.md` mixes stable technical facts about Lean 4 with very recent news, such as AI-produced proofs of Fermat's Last Theorem and the Riemann Hypothesis, the 2026 kernel CVEs, and `lake check --paranoid` in v4.35.0. This design relies only on the stable core:

- the CIC type theory and propositions-as-types;
- the small trusted kernel (the de Bruijn criterion);
- `structure` / `inductive` / `class` / `theorem` / `sorry`;
- subtypes, `partial`, termination checking, `Decidable`;
- `#print axioms` and Lake.

Anything that depends on the 2026 trust tooling (comparator, con-leche, `--paranoid`) is marked **"per Lean4.md, verify before adopting."**

---

## 1. Thesis

DOL describes itself as *ontology-first*: you declare what is true about a system (genes, traits, rules, laws, evolutions), and code follows. Lean 4 is the language where **specification, program and proof are one artifact, and a small kernel decides truth.**

DOL already has the vocabulary of a specification language:

- `rule`, `law`, `forall`, `exists`, `implies`
- `never`, `matches`
- `evo` lineage, `where` refinements
- mandatory `docs`

But **none of that vocabulary has semantics** today (§2). The cheapest, most credible way to give it meaning is:

> **Do not build a proof kernel in DOL. Make Lean 4 DOL's verification backend.**
> DOL stays the ontology/systems DSL that compiles to Rust, TypeScript and WASM for the VUDO VM. Its declarative half (laws, rules, refinements, evolutions, CRDT annotations) compiles to **Lean 4 propositions**, checked by Lean's kernel. The DOL front-end stays *untrusted*, exactly as Lean's own elaborator is (de Bruijn criterion).

This is AWS Cedar's *verification-guided development* pattern (Lean4.md §5). A small Lean model is proved correct, the production code is generated separately, and differential testing ties the two together. DOL is unusually well placed for it, because it **already emits both sides**: Rust/WASM for production, and (proposed) Lean for the model. That gives the Univrs **Oracle Pattern** a real oracle.

---

## 2. Where DOL is today (verified at v0.8.2)

Probes were run with `dol-parse` built from main (`9959d8b`). Code references are in `src/`.

| Spec-language feature | Today | Evidence |
|---|---|---|
| `law name(params) { expr }` inside a trait | **Parsed, then discarded** | `parser.rs:922` collects into `_laws` and drops them; `dol-parse` reports "0 behaviors" |
| `law name { forall x: T. … }` (the form used in `examples/genes/graph.dol`, `examples/stdlib/**`) | **Does not parse** | "expected (, found '{'" (`parse_law_decl` requires parens, `parser.rs:4006`) |
| `a implies b` in any expression | **Does not parse** | "invalid binary operator: Implies" (`parser.rs:2772`); `pratt.rs:53` gives it a binding power but the binary-op conversion rejects it |
| `exists x: T. p` in expression position | **Does not parse** | "expected expression, found 'exists'" |
| `forall x: T. p` | Parses | `ForallExpr` (`ast.rs:1190`) |
| `forall`/`exists`/`implies` semantics | Typed as `Bool`; Rust codegen emits `/* logical expression not yet supported */` | `typechecker.rs:575-592`, `codegen/rust.rs:1838` |
| `has x: T where c` (refinement) | Parses; used only for CRDT-compatibility warnings; **not enforced** | `ast.rs:1963 HasField.constraint`, `validator.rs:1208` |
| `rule` statements (`x never y`, `x matches y`) | Parsed as **subject/predicate strings**, so they have no formal meaning | `ast.rs:806 Statement` |
| `evo X @ v2 > v1` (adds/deprecates/removes) | Lineage metadata only; no migration function, no preservation check | `ast.rs:754 Evo` |
| Type checking | HM-style inference with `Type::Var` exists, but **the validator does not enforce it**: `return "hello" + true` from an `i64` function passes | `validator.rs:741-772` |
| Generics on `fun` | Do not parse; gen/trait generics are skipped | `parser.rs:222 skip_type_params` |
| Enum fields, `match` | Parse; exhaustiveness checking not established | probe |
| Purity/effects | `sex` is a syntactic marker; host imports compile to the VUDO ABI (`src/wasm/host_abi.rs`) | — |
| Mandatory `docs` | **Real** and enforced | — |

**Conclusion:** DOL has the *shape* of a specification language and almost none of its substance. Lean supplies the substance. DOL only needs to (a) stop dropping and rejecting what users write, and (b) translate it faithfully.

---

## 3. Concept mapping: DOL construct → Lean 4 target

| DOL (v0.8 syntax) | Lean 4 target | Notes |
|---|---|---|
| `gen a.b { has f: T … }` | `structure A.B where f : T` | Dotted names map to Lean namespaces |
| `has f: T where c` | `{f : T // c}` (Subtype), or a proof field `h_f : c` | A proof field keeps runtime layout identical to the Rust/WASM output |
| `has k: enum { A, B }` / `type X { … }` | `inductive K \| A \| B deriving DecidableEq, Repr` | Payload-carrying variants need a DOL syntax extension (§5.3) |
| `trait t { fun …; law …(…) { … } }` | `class T (α : Type) where op : …; law : ∀ …, …` | This is Lean's "lawful class" pattern (cf. `LawfulMonad`) |
| impl of a trait (not in DOL today) | `instance : T X where … law := by …` | Proof obligations attach to instances |
| `rule r { … }` (formalised) | `def r : Prop := …` plus `theorem r_holds : r` | Only *formal* rule bodies translate (§5.2) |
| `forall x: T. p` / `exists` / `implies` | `∀ x : T, p` / `∃ x : T, p` / `p → q` | In `Prop`, not `Bool` |
| pure `fun` | `def` (total) or `partial def` | Totality is checked by Lean |
| `sex fun` / `sex var` | `IO` / `EStateM` actions; host functions as `opaque` axioms-free stubs | Effects are visible in types |
| `evo X @ v2 > v1` | `def migrate : X_v1 → X_v2` plus `theorem migrate_preserves_R` | Evolution becomes a *proof obligation* |
| `@crdt(lww\|or_set\|…)` | Instance of a `CRDT`/semilattice class with `merge_comm`, `merge_assoc`, `merge_idem` | Proved **once** per strategy in a DOL-Lean library |
| `system s @ v { requires t >= v' }` | Lake package plus `import`s | Version constraints stay in DOL; Lean checks the composition |
| `docs { … }` | `/-- … -/` docstrings | Keeps statements human-auditable (statement fidelity) |
| `.dol.test` given/when/then | `example`/`#guard` plus property tests (Plausible) | Concrete tests next to proofs |
| `'quote`, `!eval`, macros | Lean `syntax`/`macro_rules`/`elab` | No change proposed; DOL keeps its own macro system |

---

## 4. Design principles

1. **Lean is the kernel; DOL is untrusted.**
   - DOL never claims "verified" on its own authority.
   - "Verified" means: `lake build` passes, there are **zero `sorry`s**, and `#print axioms` shows only `propext`, `Classical.choice` and `Quot.sound`.
2. **Separate `bool` (computation) from `prop` (specification).**
   - Today `forall` is typed as `Bool`, which is unsound as a specification and uncomputable as code.
   - Laws and rules live in `prop`.
   - `prop`s that are `Decidable` may also be compiled to runtime checks in Rust/WASM.
3. **Statement fidelity first** (Lean4.md §8: "did we prove the right thing?").
   - Every law and rule keeps mandatory `docs`.
   - The generated Lean must be readable, with one DOL law mapping to one named Lean theorem.
   - Translation is covered by golden tests.
   - The statement is fixed and human-reviewed *before* anyone (human or AI agent) writes proofs.
4. **Proofs are optional; statements are not.**
   - A DOL law with no proof still compiles: it becomes a Lean `theorem … := sorry`, and the `sorry` count is reported as open obligations.
   - This lets ontologies grow without blocking on proof effort.
5. **One semantics, two outputs.**
   - The same DOL source produces (a) Rust/WASM for VUDO and (b) a Lean model.
   - Differential tests (the Oracle Pattern) check that they agree.
6. **Lean is an optional toolchain.**
   - It sits behind a `lean` Cargo feature and a separate CI job with a pinned `lean-toolchain`.
   - Users who never verify pay nothing.

---

## 5. Proposed feature set

### Phase L0: fix the floor (no Lean required)

These are prerequisites. Without them, nothing written in DOL's specification vocabulary survives to a backend.

| # | Change | Oracle (test) |
|---|---|---|
| L0.1 | `implies` works as a binary operator (`parser.rs:2772`) | Parse test `a implies b`; negative oracle: precedence is lower than `\|\|` |
| L0.2 | `exists x: T. p` parses in expression position | Parse and round-trip test |
| L0.3 | `law name { … }` (no params) parses; params become optional | All `law`s in `examples/**` parse |
| L0.4 | **Keep laws in the AST**: add `Trait.laws: Vec<LawDecl>` instead of `_laws` | Negative oracle: a trait with N laws never reports "0 laws" |
| L0.5 | Whole-file parsing in `dol-parse`/`dol-check` (`parse_file_all`) | Negative oracle: trailing garbage after the first declaration fails |
| L0.6 | Introduce a `prop` type; `forall`/`exists`/`implies`/law bodies are `prop`; `bool → prop` coercion allowed, `prop → bool` only via `decide` | Type tests in both directions |
| L0.7 | Enforce the typechecker in the validator for `fun` bodies | Negative oracle: `return "hello" + true` from `-> i64` fails |
| L0.8 | Stop dropping gen/trait generics; parse `fun id<T>(x: T) -> T` | Parse and HIR tests |

### Phase L1: `dol-lean`, specification extraction

New codegen target: `dol-codegen -t lean` (feature `lean`) emits a Lake package.

```
out/
  lakefile.lean            -- pinned toolchain, no Mathlib by default
  DolGen/<Module>.lean     -- structures, inductives, classes, defs
  DolGen/<Module>/Laws.lean -- one theorem per law/rule, body := sorry
```

Example input (v0.8 syntax; the `law`/`implies` lines need L0.1–L0.4 first):

```dol
gen ledger.account {
  has owner: string
  has balance: u64 where balance <= 1000000
}
docs { A bounded account balance. }

trait ledger.transfer {
  fun transfer(from: ledger.account, to: ledger.account, amt: u64) -> (ledger.account, ledger.account)
  law conserves(a: ledger.account, b: ledger.account, n: u64) {
    n <= a.balance implies
      transfer(a, b, n).0.balance + transfer(a, b, n).1.balance == a.balance + b.balance
  }
}
docs { Transfers move value between accounts without creating or destroying it. }
```

Generated Lean:

```lean
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
```

This listing type-checks on Lean v4.36.0. It lives at `lean/DolLean/Examples/Ledger.lean`. The arithmetic is deliberately lifted to `Nat` in the law. **Fixed-width overflow semantics is a key fidelity decision** (§7).

**Exit criteria (L1):**
- Every `law`/`rule`/`where` in `examples/**` either translates or produces a precise "not translatable" diagnostic.
- `lake build` on the output succeeds, with `sorry` warnings only.
- Golden-file tests pin the translation.
- Negative oracle: an ill-typed law (e.g. comparing `string` to `u64`) fails `lake build` and the error maps back to the DOL span.

### Phase L2: proofs and the verification gate

1. **Proof attachment.** Recommended start: **external proof files**, i.e. `proofs/<Module>.lean` containing `theorem <law_name> : <stmt> := by …`. `dol-lean` checks that each proof's statement is *syntactically identical* to the generated statement, a lightweight comparator.
   - This keeps DOL's grammar simple, lets Lean experts and AI agents work in native Lean with the infoview, and gives a clear review boundary.
   - An inline `law … proof by { <lean tactics> }` escape hatch can come later.
2. **`dol-check --verify`** runs:
   1. `dol-codegen -t lean`
   2. `lake build`
   3. a `#print axioms` gate that rejects `sorry` and non-standard axioms
   4. a report: proved / open (`sorry`) / failed per law
3. **CI job** with a pinned Lean version. Per Lean4.md, add the 2026 trust steps (`lake check`, multiple kernels, comparator) **after verifying they exist in the pinned release**.

**Exit criteria (L2):**
- At least one real DOL trait with laws is fully proved with no `sorry`.
- The gate fails on `sorry`.
- The gate fails on an added `axiom` (negative oracle).

### Phase L3: language features that move DOL toward Lean

Only features with a clear payoff for VUDO/DOLOS are listed. Each is specified so that it lowers both to Rust/WASM and to Lean.

| # | Feature | DOL surface (proposed) | Lean lowering | Why it matters for Univrs |
|---|---|---|---|---|
| L3.1 | **Algebraic data types** (enums with payloads) plus **exhaustive `match`** | `type Msg = Ping \| Data(bytes: Vec<u8>)` | `inductive` + `match` (Lean checks exhaustiveness) | Spirit messages and protocol states |
| L3.2 | **Trait implementations with law obligations** | `impl ledger.transfer for MyLedger { … }` | `instance` + per-law proof obligation | Laws attach to concrete code, not just interfaces |
| L3.3 | **Refinement types** | `has n: u64 where n > 0` (exists today) | Subtype / proof field; a runtime check in Rust/WASM when unproved | Validated Spirit inputs and the host ABI |
| L3.4 | **Totality by default** | `fun` = total; `partial fun` opts out | `def` with termination checking / `partial def` | Total Spirit handlers; fuel limits become a backstop, not the only guarantee |
| L3.5 | **Effect typing** | `sex fun` types become effectful; a pure `fun` cannot call a `sex fun` | `IO`/`EStateM`; host functions as `opaque` | Capability reasoning about what a Spirit can do |
| L3.6 | **Evolution obligations** | `evo X @ 2 > 1 { migrate(x) = … preserves rule.r }` | `def migrate` + `theorem migrate_preserves_r` | Safe schema evolution for local-first data: DOL's founding idea, made checkable |
| L3.7 | **CRDT laws** | `@crdt(lww)` (exists today) | Instance of a `CRDT` class; `merge_comm/assoc/idem` proved once per strategy | Proved convergence for DOLOS local-first state |
| L3.8 | **Decidable props as runtime guards** | a `rule` whose body is decidable | `Decidable` instance → a generated check in Rust/WASM | One rule is both a proof obligation and a production assertion |
| L3.9 | **Generic functions** | `fun map<A, B>(…)` | implicit args `{α β : Type}` | Required for any real stdlib |

**Explicit non-goals:**
- Full dependent types in DOL's surface syntax. Dependent reasoning lives in Lean proof files; DOL stays approachable.
- Tactics or a proof kernel inside DOL.
- Self-hosting DOL in Lean.
- Requiring Mathlib by default. Opt in only when a domain needs reals or advanced mathematics.

### Phase L4: ecosystem payoffs

1. **Verified Spirit capability policies.** Model each Spirit's declared capabilities and the VUDO VM's 15 host functions in Lean. Prove properties such as "a Spirit without `storage` capability never reaches `host_storage_write`". This is Cedar's pattern applied to the VUDO host ABI.
2. **Differential testing (Oracle Pattern).** For decidable laws, generate inputs, run the Lean model (`#eval`/compiled) and the DOL→WASM Spirit in the VUDO VM, and compare. Use binary search over the declaration graph to localise divergence.
3. **Proof DAG for agents.** Laws form a dependency graph (`uses`, `requires`, `evo`). Expose it as the shared, verifiable state for multi-agent proof work (the Prove2Me pattern, Lean4.md §8): agents fill `sorry`s and the gate accepts only kernel-checked nodes.
4. **A `dol-lean` standard library** of proved CRDT strategies, the ledger/credit invariants used by `host_credit_*`, and the ordering and lattice classes reused by DOL traits.

---

## 6. Capability gap table: what DOL can't do, and how Lean closes it

| Capability | DOL today | Via the Lean backend (L1–L2) | Needs a DOL language change |
|---|---|---|---|
| State a law and have it mean something | ✗ (dropped) | ✓ theorem statement | L0.3–L0.6 |
| Prove a law | ✗ | ✓ external proof plus gate | — |
| Catch ill-formed specifications | ✗ | ✓ Lean elaboration | L0.6 (`prop`) |
| Refinement types | parsed, unenforced | ✓ subtypes | L3.3 for runtime checks |
| Exhaustive pattern matching | unverified | ✓ | L3.1 (payload enums) |
| Termination/totality | ✗ | ✓ | L3.4 |
| Effect isolation | marker only | ✓ via monads | L3.5 |
| Safe schema evolution | metadata only | ✓ preservation theorems | L3.6 |
| CRDT convergence guarantees | annotation only | ✓ semilattice laws | L3.7 |
| Generic programming | ✗ for `fun` | ✓ | L3.9, L0.8 |
| Machine-checkable trust for agents and AI | ✗ | ✓ binary kernel verdict | — |

---

## 7. Risks and open decisions

| Risk | Mitigation |
|---|---|
| **Translation fidelity** (DOL→Lean is untrusted code) | Readable output, one-to-one naming, golden tests, statements human-reviewed before proving |
| **Fixed-width integers vs `Nat`.** Rust/WASM wrap or trap; mathematics does not. | **Decision needed:** model `u64` as `UInt64` (faithful, harder proofs) or as `Nat` with explicit bound hypotheses. Recommendation: `UInt64` in structures, `toNat` in law statements, plus generated no-overflow side conditions. |
| **Lean/Mathlib version churn** (Lean4.md §8) | Pin `lean-toolchain`; no Mathlib by default; a scheduled upgrade job |
| **Toolchain weight** | `lean` Cargo feature, separate CI job, never in the default build |
| **Model/implementation divergence** (Rust codegen today compiles only 14/41 files standalone) | L4.2 differential tests; fix Rust codegen correctness in parallel |
| **Scope creep into a dependently typed DOL** | The non-goals in §5; L3 features must each lower to both Rust/WASM and Lean |

**Decisions for the maintainer:**
1. Proof attachment: external `.lean` files first (recommended), or an inline `proof by {}` from day one?
2. Integer model for laws (above).
3. Is `prop` a new keyword type, or should `law`/`rule` bodies be implicitly `prop`? Recommendation: an implicit `prop` context, plus an explicit `prop` type for `fun`s that return propositions.
4. Should L0 ship as **v0.9** together with the planned removal of legacy syntax? Recommendation: yes. L0 is all parser and validator correctness, and v0.9 is already a breaking release.

---

## 8. Goal posts

| Milestone | Definition of done (all test-oracled) |
|---|---|
| **v0.9: "Truthful front-end"** (L0) | Laws and quantifiers parse and are kept; whole-file CLIs; types enforced; `prop` separated from `bool`; every `law` in `examples/**` parses |
| **v0.10: "Statements in Lean"** (L1) | `dol-codegen -t lean`; all example laws produce a Lean statement or a precise diagnostic; `lake build` green with `sorry`s only |
| **v0.11: "First proofs"** (L2) | `dol-check --verify` gate; one real Univrs trait (credit/ledger, or one CRDT strategy) proved with zero `sorry`; negative oracles for `sorry`/`axiom` |
| **v0.12+: "Lean-shaped DOL"** (L3) | ADTs plus exhaustive match, impl obligations, totality, effect typing, evo preservation, proved CRDT strategies |
| **1.0 candidate** | A spec document generated from (and tested against) the parser; Spirits compiled to the VUDO VM carry Lean-checked capability and CRDT guarantees; differential tests link Lean model and WASM |

**The goal post in one sentence:** a DOL ontology, written once, yields running Spirits for the VUDO VM **and** a Lean 4 model whose laws a kernel has checked, so that "what the system *is*" (DOL's promise) becomes a machine-checked claim rather than documentation.

---

## 9. Progress and implementation steps

### Done (2026-11-21)

- **Lean toolchain pinned.** `lean/` is a Lake project on Lean v4.36.0 with no Mathlib.
- **L2 gate prototype.**
  - `lean/scripts/verify.sh` builds `DolLean` and rejects any theorem that depends on `sorryAx` or on axioms other than `propext`, `Classical.choice` and `Quot.sound`.
  - `--self-test` checks the gate against fixtures: it accepts a proof, and rejects `sorry` and a custom `axiom`.
- **First L3.7 result: LWW register laws.** `DolLean.Crdt.Lww` proves `merge` idempotent, associative, and commutative for *coherent* writes.
  - Lean also proves commutativity is **false** when two distinct writes share a (timestamp, replica) pair.
  - **Design requirement for DOLOS:** each replica must never issue two writes with the same timestamp (a strictly increasing per-replica clock). The Rust/WASM LWW implementations must be checked against this.
- **The §5 ledger example type-checks** (`DolLean.Examples.Ledger`).

### Next steps, in order

1. **L0 (parser truthfulness):** `implies`, `exists`, parameterless `law`, keeping trait laws in the AST, and whole-file CLIs. This is in progress as a separate PR, together with the numeric-literal, `docs`-brace and `use`/`evo` parser fixes.
2. **L0.6–L0.8:** a `prop` type distinct from `bool`; enforce the typechecker in the validator; generics on `fun`. This depends on step 1.
3. **L1 skeleton:**
   - Add `src/codegen/lean.rs` behind a `lean` feature, and `dol-codegen -t lean` emitting a Lake package.
   - Translate `gen` → `structure`, field `where` → proof field, trait + laws → `class`, and `law` → `theorem … := sorry`.
   - Golden tests compare the output with `lean/DolLean/Examples/*.lean`.
4. **L1 corpus:** every `law` in `examples/**` either translates and type-checks, or gets a precise diagnostic.
5. **L2 wiring:**
   - `dol-check --verify` runs `dol-codegen -t lean`, then `lean/scripts/verify.sh`.
   - External proof files are keyed by law name, with a statement-equality check.
   - Add a CI job with a pinned toolchain (needs maintainer approval for `.github/workflows`).
6. **L3.7 CRDT library:** prove the remaining `@crdt` strategies (`or_set`, counters, …) and map each strategy annotation to its proved instance.
7. **L3.1–L3.6:** ADTs, impl obligations, totality, effect typing, evo preservation, in the order of §5.

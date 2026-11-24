> **Source note (added 2026-11-24).** This is a third-party survey of Lean 4, kept as the input cited by `docs/design/DOL-LEAN4-ALIGNMENT.md`.
> Its technical description of Lean is stable background: CIC, the kernel, elaboration, tactics, Lake.
> Its **2026 news items are unverified** and DOL design decisions do not rely on them: the AI formalizations of FLT and the Riemann Hypothesis, the kernel CVEs, con-leche/con-ron, `lake check --paranoid`, comparator, funding and roadmap claims.
> Toolchain facts used by DOL were checked against Lean v4.36.0 (`lean/lean-toolchain`).

# Lean 4 in Late 2026: What It Is, Where It Came From, Who Runs It, and Why AI Models Are Converging on It

Lean is an open-source, dependently typed functional programming language and interactive theorem prover. It was created by Leonardo de Moura at Microsoft Research in 2013. Today it is stewarded by the Lean Focused Research Organization (Lean FRO), a nonprofit under Convergent Research, not by a company. By November 2026 it had become the de facto "verification layer" for AI-generated mathematics and, more and more, for AI-generated software. The clearest signs are AI-produced, machine-checked formalizations of Fermat's Last Theorem (September 2026) and a claimed proof of the Riemann Hypothesis that the Lean FRO and Mathlib Initiative certified on November 21, 2026.

## TL;DR

- **What and who:** Lean is a programming language whose type checker is also a proof checker: a proof is a program, and a small trusted kernel accepts or rejects it. Lean was born at Microsoft Research (2013). It is now maintained by the nonprofit Lean FRO (founded July 2023 by Leonardo de Moura and Sebastian Ullrich), with the community-run Mathlib library and the Mathlib Initiative (director Johan Commelin) alongside. De Moura is also a senior scientist at AWS, and Amazon made the single largest donation in the FRO's history in July 2026, but no company owns Lean.
- **Ecosystem:** Mathlib holds roughly 290,000 theorems and 140,000 definitions from 772 contributors. Lean is used for frontier mathematics (Liquid Tensor Experiment, PFR, FLT, sphere packing) and production verification (AWS Cedar, Aurora, SampCert, Microsoft SymCrypt via Aeneas). It is also the standard environment for AI theorem provers: AlphaProof, Aristotle, Seed-Prover, DeepSeek-Prover, Kimina, Goedel, Gauss, and Claude-based agent swarms.
- **Why it matters for models:** Lean turns "is this answer correct?" into a cheap, deterministic, binary check. That makes it an RL reward signal, a hallucination filter, and a trust boundary for multi-agent systems. The 2026 "Summer of Soundness Bugs" showed the trust boundary must be hardened: multiple kernels, verified checkers (con-leche), `lake check --paranoid`, and human audits of statement fidelity. Teams building agentic systems should design around these practices.

## Key Findings

1. **Lean is a language first and a proof assistant second, and that is the point.** Programs, specifications and proofs live in one language. Lean 4 is implemented largely in Lean itself, so users can extend syntax, tactics and automation without touching C++.
2. **Governance is nonprofit and multi-funded.** The Lean FRO lists Alex Gerko, the Alfred P. Sloan Foundation, the Richard Merkin Foundation, Simons Foundation International and Convergent Research as funders. Amazon (July 2026), Harmonic (February 2026) and XTX Markets (September 2026) have added support. In September 2026 the FRO moved from annual to six-month roadmaps, because "our field is changing too quickly to commit to a plan a year ahead."
3. **AI has flipped the economics of formalization.** Hales's Flyspeck project, which formally verified the Kepler conjecture in HOL Light and Isabelle between 2003 and 2014, was originally estimated to require "about 20 work-years." In 2026, Math Inc.'s Gauss finished the 8- and 24-dimensional sphere-packing formalizations in weeks, Anthropic's Claude agents produced a 13-million-line formal proof of FLT in 11 days, and a 17-million-line Lean proof of the Riemann Hypothesis was released and community-certified within about a day.
4. **The trust story changed in 2026.** AI-assisted researchers found several kernel soundness bugs (July–August 2026). The response was a multi-kernel architecture: con-leche and con-ron, checkers with formal consistency proofs relative to set theory; the Lean Kernel Arena; and Lean v4.35.0 shipping verified kernels with `lake check --paranoid`.
5. **The bottleneck is moving from "can we prove it?" to "did we prove the right thing?"** Statement fidelity, definition correctness, and human understanding of machine-generated proofs are now the hard problems.

## 1. What Lean Is and How It Works

### The core idea: propositions as types, proofs as programs

Lean's logic is a variant of the **Calculus of Inductive Constructions (CIC)**, a dependent type theory. The specific dialect has non-cumulative universes and proof irrelevance. In a dependent type theory, types can depend on values: `Vector α n` is the type of lists of length exactly `n`. Most importantly, *propositions are types*.

This is the **Curry–Howard correspondence**:

| Logic | Programming |
|---|---|
| Proposition `p` | Type `p : Prop` |
| Proof of `p` | A term (program) of type `p` |
| `p → q` (implication) | Function from proofs of `p` to proofs of `q` |
| `p ∧ q` (and) | Pair of proofs |
| `∀ x, P x` | Dependent function |
| Checking a proof | Type-checking a program |

So "does this proof prove this theorem?" reduces to "does this term have this type?" That is a mechanical, decidable-in-practice check. One caveat: Mario Carneiro's thesis showed that definitional equality in Lean is undecidable in general, so the checker can fail to recognize some terms as equal that actually are.

### The small trusted kernel

Lean separates a large, untrusted front-end from a small trusted **kernel**. The kernel is "several thousand lines of C++ code," in Thomas Hales's description. Everything else can be buggy without compromising soundness, as long as the kernel rejects invalid terms. That includes the parser, elaborator, tactics, AI-written metaprograms and automation like `grind`. This is the de Bruijn criterion, and it is why AI-generated proofs can be trusted at all: the AI can be wrong, but the kernel won't accept a wrong proof unless the kernel itself has a bug.

Proofs can also be exported and re-checked by **independent kernels**. Hales counts about 25 Lean kernels in the "Lean Kernel Arena," including nanoda (Rust), lean4lean (Lean, by Mario Carneiro), and the formally verified con-leche (by Joachim Breitner).

### Elaboration

Users write high-level syntax with implicit arguments, type-class instances, coercions, notation and tactics. The **elaborator** turns this into fully explicit kernel terms. It is "a sort of compilation process for mathematics," and the kernel then checks its output. Elaboration is where most of Lean's cleverness and performance cost lives: unification, type-class resolution, and the automatic coercion from `ℕ` to `ℝ`.

### Term mode vs. tactic mode

- **Term mode:** you write the proof term directly, like a functional program.
- **Tactic mode** (`by ...`): you issue commands (`intro`, `rw`, `simp`, `induction`, `omega`, `linarith`, `grind`) that transform a goal state step by step. Each tactic is a metaprogram that builds the proof term for you.

Tactic mode dominates in Mathlib and in AI provers, because each step produces an observable goal state. That is exactly the (state, action) structure that reinforcement learning and tree search need.

### Metaprogramming: Lean written in Lean

Lean 4 is a ground-up rewrite. In its own words, "Lean and its tooling are implemented in Lean." The same language that states theorems also implements the parser, elaborator, tactic framework, language server, build system and documentation tools. Users can therefore define new syntax, domain-specific languages and decision procedures as ordinary Lean code. Recent FRO automation includes:

- **`grind`**, an SMT-inspired tactic combining E-matching, congruence closure and linear arithmetic, shipped August 2025.
- **`bv_decide`**, for bit-vector reasoning.
- **`SymM`**, a framework for high-performance software verification tooling.

### Lake, the editor and the infoview

- **Lake** is Lean's build system and package manager. Projects declare dependencies such as Mathlib, and Lake fetches pre-built artifacts. The FRO's current roadmap adds `lake check` (export and recheck with the official kernel plus comparator checks), `lake check <kernel>` (recheck with an alternative checker such as nanoda), and `lake check --paranoid` (recheck with every bundled kernel).
- **VS Code extension and infoview:** as you move the cursor through a proof, a side panel shows the current hypotheses and goals. That tight feedback loop is the heart of the Lean user experience. The extension passed 100,000 installs by September 2025.
- **Web tooling:** live.lean-lang.org (playground), Loogle (search), Reservoir (package index), Verso (literate documents and papers written and checked in Lean), Verso Blueprints (dependency-graph views of large projects), and the beta Lean Workbench (a zero-install web environment).

### Lean as a programming language

Lean compiles to efficient native code via C. It uses reference counting with in-place updates (the "Perceus" work won a PLDI 2021 distinguished paper award). AWS reports that its Lean model of Cedar runs differential tests in about 5 microseconds per input. The FRO is designing deeper compiler work (stack-allocated objects, uniqueness annotations, better array code), with implementation slated to start in early 2027. The FRO has explicitly decided **not** to invest in debugger infrastructure for now, because "AI agents are changing how people debug code."

## 2. Concrete Lean 4 Examples

### A simple function

```lean
def double (n : Nat) : Nat := n + n

#eval double 21   -- 42
```

- `def double (n : Nat) : Nat` declares a function named `double` that takes a natural number `n` and returns a natural number.
- `:= n + n` is the body. Lean checks that `n + n` has type `Nat`.
- `#eval` runs the compiled function and prints the result in the editor.

### A theorem about that function (tactic mode)

```lean
theorem double_eq_two_mul (n : Nat) : double n = 2 * n := by
  unfold double
  omega
```

- `theorem double_eq_two_mul (n : Nat) : double n = 2 * n` states a proposition. The statement *is a type*, and the proof will be a term of that type.
- `:= by` switches to tactic mode. The infoview now shows the goal `⊢ double n = 2 * n`.
- `unfold double` replaces `double n` with its definition, so the goal becomes `n + n = 2 * n`.
- `omega` is a decision procedure for linear arithmetic over integers and naturals. It closes the goal by constructing a proof term automatically.
- When the file compiles, the kernel has re-checked the generated term. If any step were wrong, you would see a red error instead.

### Curry–Howard in term mode

```lean
theorem and_swap (p q : Prop) (h : p ∧ q) : q ∧ p :=
  ⟨h.2, h.1⟩
```

- The hypothesis `h : p ∧ q` is literally a pair of proofs.
- `⟨h.2, h.1⟩` builds a new pair with the components swapped. This "proof" is just a two-line functional program, and type-checking it *is* proof-checking it.

### Commutativity of addition, by induction

```lean
theorem my_add_comm (a b : Nat) : a + b = b + a := by
  induction b with
  | zero => simp
  | succ k ih => omega
```

- `induction b` splits into a base case `b = 0` and a step case `b = k + 1`, and gives the hypothesis `ih : a + k = k + a` in the step case.
- `simp` closes `a + 0 = 0 + a` using simplification lemmas from the core library.
- `omega` closes the step case using linear arithmetic (with `ih` available).
- In practice, you would just cite the existing library lemma in term mode: `Nat.add_comm a b`.

### A Mathlib lemma over the reals

```lean
import Mathlib

example (a b : ℝ) : 2 * a * b ≤ a ^ 2 + b ^ 2 := by
  nlinarith [sq_nonneg (a - b)]
```

- `import Mathlib` brings in the community library, including the construction of the real numbers `ℝ`.
- `example` is an anonymous theorem.
- `sq_nonneg (a - b)` is a Mathlib lemma giving `0 ≤ (a - b) ^ 2`.
- `nlinarith` (nonlinear arithmetic) expands that hint and finds a linear combination proving the inequality, i.e. the AM–GM inequality for two terms.

This last pattern is what AI provers do at scale. They propose tactic calls and lemma hints, and Lean either accepts or returns a precise error message, which becomes training signal.

## 3. History and Origins

### Lean 0.1 through Lean 3 (2013–2017)

- **July 2013:** de Moura makes the first commit to the Lean repository at Microsoft Research. He was already the main architect of the Z3 SMT solver. Hales notes that de Moura persuaded Microsoft to release Lean as open source.
- **June 2014:** Lean 0.1 is released. Jeremy Avigad (Carnegie Mellon) was the first user. **January 2015:** the first university course using Lean, CMU's 15-815 Interactive Theorem Proving, is taught.
- **August 2015:** "The Lean Theorem Prover (System Description)" appears at CADE-25. This is the Lean 2 era. The paper later won the 2025 Skolem test-of-time award.
- **January 2017:** Lean 3.0 is released, the first moderately stable version. Lean 3 introduced a metaprogramming framework that let users write tactics in Lean.
- **July 2017:** Mario Carneiro (Avigad's student) and Johannes Hölzl split parts of Lean's core library into a separate mathematical library, **mathlib**.
- **February 2018:** the Lean Zulip chat is established. It becomes the community's central nervous system and passed 10,000 members by September 2024.

### Lean 4 and the community fork (2018–2023)

- **April 2018:** Lean 4 development begins as a complete reimplementation by de Moura and Sebastian Ullrich, designed to be implemented largely in Lean and to be an efficient general-purpose programming language.
- Lean 3 was officially end-of-lifed after version 3.4.2. The **leanprover-community** then maintained its own fork, releasing community versions up to 3.51.1 to keep mathlib alive during the transition.
- **2019:** the first Lean Together workshop is held (January), and Kevin Buzzard's group releases the **Natural Number Game** (October).
- **December 2020:** Peter Scholze issues the **Liquid Tensor Experiment** challenge. In June 2021 he wrote that the Experiment "has verified the entire part of the argument that I was unsure about." The full formalization was completed on July 14, 2022, led by Johan Commelin with major contributions from Adam Topaz.
- **2021:** Lean 4 is released (CADE 2021 paper by de Moura and Ullrich). The Mathlib4 repository is created in May 2021. Lean 4 is not backwards compatible with Lean 3.
- **2022–2023:** the community ports mathlib to Lean 4 using the semi-automated **Mathport** tool. **July 2023:** the port of Mathlib from Lean 3 to Lean 4 is complete.
- **September 8, 2023:** Lean 4.0.0, the first stable Lean 4 release, ships. Since then Lean has shipped roughly monthly numbered releases, from 4.1.0 (September 26, 2023) to **4.35.0 (October 12, 2026)**. Version 4.35.0 ships with verified kernels.

### Key people

- **Leonardo de Moura**: creator and Chief Architect, also the main architect of Z3. He received the ACM SIGPLAN Programming Languages Software Award for both Z3 and Lean, plus the Herbrand and CAV awards.
- **Sebastian Ullrich**: co-designer of Lean 4, co-founder and Head of Engineering of the Lean FRO.
- **Jeremy Avigad**: first user, co-author of *Theorem Proving in Lean*, and on the Lean community admin team.
- **Mario Carneiro and Johannes Hölzl**: founded mathlib. Carneiro also wrote the foundational type-theory thesis and lean4lean.
- **Kevin Buzzard** (Imperial College): evangelist to mathematicians, creator of the Xena Project, and leader of the EPSRC-funded Fermat's Last Theorem project (since 2024).
- **Patrick Massot**: co-author of *Mathematics in Lean*, creator of the Blueprint tooling used by large projects, and of Lean Verbose.
- **Johan Commelin**: led the Liquid Tensor Experiment and now directs the Mathlib Initiative.
- **Kim Morrison**: long-time Mathlib contributor and FRO engineer, launched the Palomar registry.
- **Terence Tao**: led the Polynomial Freiman–Ruzsa (PFR) formalization (2023) and the Equational Theories Project (2024–2025), and wrote "Machine-Assisted Proof" in the *Notices of the AMS* (January 2025).
- **Gabriel Ebner and Soonho Kong**: co-recipients with de Moura and Ullrich of the 2025 ACM SIGPLAN Software Award for Lean.

In June 2026 Quanta Books published Kevin Hartnett's *The Proof in the Code*, described as "the definitive account of the birth and rise of Lean."

## 4. Who Manages Lean Today

### The Lean FRO

- **Structure:** a nonprofit Focused Research Organization formed in July 2023 under **Convergent Research**, an incubator for time-limited, mission-driven research nonprofits. The FRO's founding mission was five years, aimed at steering Lean toward "long-term self-sustainability."
- **Leadership:** Chief Architect and co-founder **Leonardo de Moura**, who also sits on the FRO's board, and Head of Engineering and co-founder **Sebastian Ullrich**. The team is "more than ten employees," many recruited from the community.
- **De Moura and AWS:** de Moura left Microsoft Research in 2023 after 17 years and joined AWS's Automated Reasoning Group. The FRO site describes him as a senior principal applied scientist. His personal site, updated more recently, describes him as a Vice President and Distinguished Scientist at AWS. His FRO role is a separate, nonprofit position.
- **Funders:** Alex Gerko, the Alfred P. Sloan Foundation, the Richard Merkin Foundation, Simons Foundation International and Convergent Research. Major recent additions:
  - **July 2025:** Alex Gerko, founder of XTX Markets, gives $10M, with $5M to the Lean FRO and $5M to found the **Mathlib Initiative**.
  - **February 2026:** Harmonic makes an inaugural donation.
  - **July 26, 2026:** Amazon announces "substantial, long-term financial support," which it calls "the single largest donation in the FRO's history." Amazon explicitly argues that Lean's foundations should be developed in the open, in an independent nonprofit, so that "customers, auditors, and regulators" can independently validate proof tools.
  - **September 2026:** XTX Markets announces additional support.

**Bottom line on "which company manages it":** none. Lean is a nonprofit, open-source project. AWS is the most deeply entangled company: it employs the founder, is the largest donor, and uses Lean in production. Startups such as Harmonic, Axiom and Math Inc., and labs such as Google DeepMind, OpenAI and Anthropic, are heavy users and contributors but do not control it. Mathlib is governed separately by the community's admin team and maintainers.

### The Mathlib Initiative

The Mathlib Initiative officially launched in September 2025 with Gerko's $5M. It is directed by **Johan Commelin** (Utrecht University), whose own page places it at Renaissance Philanthropy, with Oliver Nash as a senior manager. It funds professional maintenance, review and editorial support for Mathlib. Mathlib itself has about 60 reviewers, of whom 28 are maintainers who accept contributions. On November 21, 2026, the Mathlib Initiative and the Lean FRO jointly signed the certification of the Riemann Hypothesis formalization.

### The current roadmap (Year 4, Part 1: September 2026 – February 2027)

- **Trust:** make external checking routine via `lake check`, alternative kernels, `--paranoid` mode, and comparator bundled with Lean. Add a mode where `lake build` skips kernel checks, so that validation becomes an explicit separate step.
- **UX:** harden Verso, Verso Blueprints, Games and the Workbench for everyday use.
- **Scalability:** granular caching for type-class inference across Mathlib; a new `do` notation with intrinsic verification; better verification-condition generation; faster `SymM` (used by Velvet and, in future, Aeneas).
- **Proof automation:** make `grind` and `bv_decide` robust and fail fast, and "ensure AI agents can use them effectively." The roadmap notes that "AI-generated proofs are becoming a major source of `grind` and `bv_decide` usage."
- **Tooling:** a code formatter; **Nerodia**, a Lean/Python FFI inspired by PyO3, because "the AI ecosystem is dominated by Python"; and productizing **lean-beam**, "how AI agents interact with Lean," to ship with Lean itself.
- **FRO-run AI projects:**
  - **Tau Ceti**: AI-authored mathematics with human-owned roadmaps and AI adversarial review.
  - **Hex**: spec-driven verified computational algebra, where humans write specifications and AI writes code and proofs.
  - **Palomar**: a registry of Lean-verified results.
- **Compiler:** design work now, implementation starting in early 2027.

## 5. Ecosystem and Usage

### Mathlib

- **Scale:** the Mathlib statistics page reports about **139,679 definitions, 291,414 theorems and 772 contributors**, with about 2.5 million lines of code in Hales's count. Later readings in November 2026 put it near 296,000 theorems. For comparison, Mathlib reported over 210,000 theorems in May 2025.
- **Scope:** algebra, number theory, analysis, measure theory, topology, category theory, geometry, combinatorics and probability. It covers the undergraduate curriculum and much graduate mathematics, plus the definitions needed to *state* frontier results. For example, `RiemannHypothesis` is a vetted Mathlib definition.
- **Why it matters for AI:** Commelin argues that because Mathlib is human-curated, "when a statement uses Mathlib's definitions, you know they are not hallucinated; they mean what you think they mean." That makes Mathlib both the vocabulary and the ground truth for AI provers.
- **Sister libraries:** CSLib (computer science, 2025), StatLib (statistics), PhysLib (physics), SciLean (scientific computing), Iris-Lean (separation logic), TorchLean (verified neural networks), Axiom's EconLib (economics) and Hex (computational algebra).

### Landmark formalization projects

| Project | Dates | Significance |
|---|---|---|
| Liquid Tensor Experiment | Dec 2020 – Jul 2022 | Scholze's challenge; showed Lean could handle cutting-edge research mathematics |
| Polynomial Freiman–Ruzsa (Tao et al.) | Nov–Dec 2023 | Paper posted Nov 9, 2023; Tao announced on Dec 5, 2023 that the formalization "has succeeded after three weeks," using Massot's Blueprint |
| Fermat's Last Theorem (Buzzard) | Dec 2023 – (planned 2029) | Human-led, modern route; overtaken in September 2026 by Anthropic's AI formalization of a classical route |
| Equational Theories Project (Tao) | Sep 2024 – Dec 2025 | Massively collaborative human+AI project settling all 22,028,942 implications between the 4,694 simplest equational laws on magmas (primary goal reached April 14, 2025) |
| Carleson's theorem | Jun 2024 – Aug 2025 | Major harmonic-analysis formalization |
| Sphere packing in dimensions 8 and 24 | Mar 2024 – Mar 2026 | Viazovska/Hariharan project completed with Math Inc.'s Gauss; about 200K lines |
| Classification of Finite Simple Groups (Axiom) | Oct 2026 | Announced as the first complete Lean formalization |
| Fermat's Last Theorem (Anthropic) | Aug–Sep 2026 | 13M lines in 11 days by Claude agents |
| Riemann Hypothesis (Alpöge) | Nov 20–21, 2026 | 17M-line claimed proof; Lean FRO and Mathlib Initiative certified the statement and proof |

### Education

- **Natural Number Game** (2019): gamified introduction that builds arithmetic from Peano axioms.
- **Theorem Proving in Lean 4** (Avigad, de Moura, Kong et al.): the canonical introduction to the logic.
- **Mathematics in Lean** (Avigad and Massot): Mathlib-based mathematics.
- **Functional Programming in Lean** (David Thrane Christiansen): Lean as a programming language.
- The **Lean Language Reference** (written in Verso) covers the type system, elaboration and tactics.
- Over 50 university courses had been taught with Lean by May 2025.

### Industry: software and hardware verification

- **AWS Cedar:** Cedar is the authorization-policy language behind Amazon Verified Permissions and AWS Verified Access. AWS uses "verification-guided development": an executable Lean model about 10 times smaller than the Rust production code, proofs of security properties (such as forbid-overrides-permit), and differential random testing on millions of inputs to check that Rust matches the model.
- **Other Amazon uses (per Amazon Science, July 2026):**
  - Bedrock AgentCore, for agentic safety.
  - SampCert, verified differential-privacy primitives deployed in AWS Clean Rooms (PLDI 2025).
  - AWS Neuron, for AI-chip compilation.
  - A proof of correctness for Aurora's segment-repair protocol, built with an LLM plus Lean "in a fraction of the time it would have taken manually."
- **Microsoft Research:** verifies production Rust cryptography in **SymCrypt** using Lean through the **Aeneas** toolchain (July 2026).
- **Startups:** Harmonic (2024, the first Lean-based startup), Axiom (2025), Math Inc. (2025), Pramaana Labs ($27M seed from Khosla, June 2026), and &Truth ($70M seed for Lean-based software verification, October 2026).
- **Other tools and projects:** Veil (distributed protocols, CAV 2025), Velvet (imperative programs), Bob DyLean (cryptographic protocols), and the Signal Shot moonshot to verify the Signal protocol and its Rust implementation in Lean.

### How Lean compares with other proof assistants

| System | Logic | Strengths | Relative position |
|---|---|---|---|
| **Lean 4** | Dependent type theory (CIC variant) | Unified language and prover, extensible metaprogramming, Mathlib, AI tooling | Most popular among mathematicians; default target for AI provers |
| **Rocq** (formerly Coq, renamed 2025) | CIC | Mature, CompCert, MetaCoq meta-theory work, strong program-verification heritage | Closest relative; stronger formal meta-theory of its own kernel |
| **Isabelle/HOL** | Simple type theory (HOL) | Powerful automation (Sledgehammer), Archive of Formal Proofs, seL4 | Weaker logic but very productive; Paulson argues most mathematics fits HOL |
| **HOL Light** | HOL | Tiny kernel (a few hundred lines), Kepler/Flyspeck | Gold standard for kernel minimalism |
| **Agda** | Dependent type theory | Elegant dependently typed programming, constructive/HoTT work | Research-oriented, smaller libraries |

Lean's advantages are network effects (Mathlib, Zulip, AI training data) and its "language-first" design. Its relative weakness, which Hales stresses, is meta-theory: there is not yet a complete, public, human-written relative-consistency proof covering Lean's abstract type theory, and properties like unique typing remain open conjectures. Carneiro's work and con-leche are closing that gap.

## 6. AI and Lean: Lean as Verifier, Environment and Reward Signal

### Why Lean fits AI

An LLM's mathematical argument in natural language is expensive to check and prone to plausible-sounding errors. A Lean proof is checked deterministically in seconds to hours, with a binary verdict and structured error messages. This gives four properties that matter:

1. **A non-hackable reward** (modulo kernel bugs) for reinforcement learning.
2. **Step-level environment feedback**: goal states and errors that support search, self-correction and agent loops.
3. **Hallucination elimination at the proof level.** Hallucination remains possible at the statement level.
4. **Composability**: verified lemmas become reusable building blocks for other agents, which is exactly what multi-agent coordination needs.

### Timeline of AI systems built on Lean

- **OpenAI GPT-f / lean-gym (2022):** Polu et al.'s "Formal Mathematics Statement Curriculum Learning" introduced lean-gym, a REPL interface to Lean, and expert iteration. They reported "a 36.6% pass rate on miniF2F-test" (and 47.3% on miniF2F-valid), up from the previous best of 29.2% held by PACT (Zheng et al., 2021), including "26 AMC12 problems, 6 AIME problems and 2 problems adapted from the IMO."
- **Meta AI HyperTree Proof Search (2022):** the Evariste system reached 41% pass@64 on miniF2F-test in Lean, against 36.6% for GPT-f.
- **LeanDojo / ReProver (NeurIPS 2023):** open-source data extraction from Lean repositories and a retrieval-augmented prover that selects premises from Mathlib. Follow-ups: **Lean Copilot** (LLM inference inside Lean's editor), **LeanAgent** (lifelong learning across repositories, ICLR 2025), and **LeanDojo-v2** (a unified training/proving framework).
- **Google DeepMind AlphaProof (July 2024; Nature, November 2025):** an AlphaZero-style RL agent whose training pipeline, per the Nature paper, "auto-formalized approximately 1 million natural-language problems into a dataset of around 80 million formal Lean problems." It achieved IMO 2024 silver-medal performance, solving four of six problems including the hardest. In May 2026, **AlphaProof Nexus** reported autonomous proofs of 9 Erdős problems.
- **DeepSeek-Prover V1/V1.5/V2:** V1.5-RL reached about 58–63% on miniF2F. **V2-671B** (April 2025) used DeepSeek-V3 to decompose problems into subgoals and RL to train the prover. It reached 88.9% on miniF2F-test (pass@8192) and 49/658 on PutnamBench.
- **Kimina-Prover Preview (Moonshot/Numina, April 2025):** RL from Qwen2.5-72B with a "formal reasoning pattern," reaching 80.7% on miniF2F (pass@8192). It also released the Kimina Lean Server, high-throughput verification infrastructure for RL.
- **Goedel-Prover-V2 (Princeton, August 2025):** a 32B model reaching 88.1% on miniF2F at pass@32 (90.4% with self-correction) and 86 PutnamBench problems. It showed that small models with compiler-guided self-correction can rival giant ones.
- **Harmonic Aristotle (July 2025):** gold-medal-equivalent at IMO 2025 with formally verified Lean solutions to five of six problems. It combines Lean proof search, informal lemma generation with autoformalization, and a geometry solver. Since October 2025 it has been available with a Lean API.
- **ByteDance Seed-Prover (July 2025) and Seed-Prover 1.5 (December 2025):** 1.0 reached five of six at IMO 2025 in "heavy" mode. **1.5** is an agentic prover trained with large-scale agentic RL. It solves **580/660 (87.9%) of PutnamBench**, 80% of FATE-H (graduate level), 33% of FATE-X (PhD level), and 11 of 12 Putnam 2025 problems within 9 hours, at about 10 H20-GPU-days per problem.
- **AxiomProver (Axiom):** 8 of 12 Putnam 2025 problems during the competition (January 2026); a proof of Fel's Conjecture (February 2026); a complete Lean formalization of the Classification of Finite Simple Groups (October 2026).
- **Math Inc.'s Gauss:** a quasi-autoformalization of the prime number theorem (September 2025). Gauss then completed the 8-dimensional sphere-packing proof (February 2026) and, about a week later, the 24-dimensional case, with a peak of about 450–500K lines refactored to about 200K. Accounts differ on how much the 24-dimensional run relied on human-written scaffolding. One Math Inc. engineer told Scientific American they "only gave Gauss the paper" and it produced about 120,000 lines, while Math Inc.'s own account gives larger peak figures.
- **Mistral Leanstral (March 2026):** the first open-source code agent designed specifically for Lean 4.
- **Meta's ATLAS (May 2026):** autoformalized large parts of 26 mathematics textbooks.
- **OpenAI (2026):**
  - A Lean-verified result on off-diagonal Ramsey numbers (April).
  - Lean certificates for progress on 10 research problems (August).
  - Lean-verified results on the Navier–Stokes Millennium Problem with Alpöge and Buckmaster (September).
  - A prime-gap bound of 186 with Lean formalization via GPT-6 Astra (September).
  - A Lean-verified refutation of the Exponential Time Hypothesis (October).
  
  These are company announcements. Independent mathematical assessment of some of them is ongoing.
- **Anthropic / Claude:**
  - **August 2026:** results related to the Riemann zeta function, with Lean formalizations that pass Comparator validation.
  - **September 4, 2026:** dozens of Claude agents coordinated through **Prove2Me**, an open DAG-based collaboration platform built by Tianyi Peng's group at Columbia. They produced the first complete computer-checked proof of **Fermat's Last Theorem**: about 13 million lines of Lean, 30,300 intermediate theorems (about 29,500 used), and about 6 billion output tokens over 11 days. Kevin Buzzard compiled and checked it, confirmed it uses only Lean's three standard axioms, and noted that it formalizes a classical route rather than his project's modern one. He said it tells us "essentially nothing" new about mathematics.
  - **Con-leche**, the verified Lean checker, had its code and proofs generated by Claude under Joachim Breitner's direction.
  - **November 20–21, 2026:** Anthropic mathematician Levent Alpöge released a 17-million-line Lean proof of the **Riemann Hypothesis**, developed with Claude, as a 175,187-module build. Within a day, the Lean FRO and Mathlib Initiative certified that the statement matches Mathlib's `RiemannHypothesis` (checked with two independent tools, `comparator` and `con-leche --compare`). They also confirmed that the exported proof passes Lean's kernel, con-leche, con-ron and several third-party checkers, all run "on an inert export of the proof, out of reach of hypothetical malicious code."
  
  **Caveat:** this certification is less than a day old. It certifies formal correctness, and human understanding and review of the argument are still to come. Alpöge himself calls the argument "incredibly long at the moment."

### Benchmarks

- **miniF2F:** 488 olympiad-style statements (244 validation, 244 test). It is now essentially saturated: Seed-Prover "saturates MiniF2F," and top systems exceed 88–90%.
- **PutnamBench** (2024): originally 640 Putnam problems with 1,692 formalizations in Lean 4, Isabelle and (partly) Coq. It went from "only a handful" solved in 2024 to 88% by Seed-Prover 1.5 in December 2025.
- **Harder successors:** FATE (graduate and PhD algebra), ProofNet, FormalQualBench, and the FRO's **Lean Eval** leaderboard of hard formalization problems (June 2026).

The benchmark story is one of rapid saturation. Practitioners should treat miniF2F scores as historical and look to research-level formalization as the real frontier.

### Verified code generation

The same loop that proves theorems can prove programs correct. In spec-driven development, humans write the specification in Lean, AI writes the implementation and the proof, and the kernel checks both. Examples include the FRO's Hex libraries, AWS's Aurora protocol proof, Microsoft's SymCrypt work via Aeneas, and the Signal Shot challenge. Amazon frames this as "neurosymbolic AI": generative models paired with Lean so that agents' behavior can be proved safe "no matter what inputs it gets." The FRO's roadmap now treats AI agents as first-class Lean users, through lean-beam, the Nerodia Python FFI, and agent-friendly error messages for `grind` and `bv_decide`.

## 7. The 2026 Trust Crisis and Its Resolution

The "Summer of Soundness Bugs" is the most important lesson for anyone building systems that rely on Lean as a verifier.

- **July 25, 2026:** Ramana Kumar published an AI-assisted, sorry-free "disproof" of the Collatz conjecture. It exploited a kernel bug in the handling of nested inductive types (phantom parameters escaping type checking). Kiran Gopinathan reduced it to a small proof of `False` (issue #14576, July 28), and de Moura fixed it within about an hour (PR #14577).
- The independent checker nanoda *also* accepted the exploit because of an unrelated bug of its own, so cross-checking with one stale external kernel was not enough.
- Further bugs followed in July and August, several found by Daniel Selsam at OpenAI using a cybersecurity-specialized AI. The collaboration ended "when the internal AI reported it could not find additional issues." A Phrack article later catalogued kernel CVEs across Lean and Rocq, including CVE-2026-72844 for #14576.
- All the bugs required **metaprogramming**, meaning sending crafted declarations directly to the kernel. They are not reachable by ordinary tactic proofs, and de Moura stresses that this was "an implementation bug, not a hole in Lean's meta-theory."
- **The response:**
  - Regression tests in the Lean Kernel Arena.
  - Hardened kernel invariants.
  - **Con-leche** (September 10, 2026), an external checker proven in Lean to be consistent relative to ZF set theory with inaccessible cardinals.
  - **Con-ron**, a second such checker.
  - **Lean v4.35.0** (October 2026) shipping verified kernels and `lake check --paranoid`.
  - **Comparator**, a sandboxed judge that exports proofs and checks that the proved statement matches a trusted reference statement.

Hales's verdict is that this is "a positive development": the bugs were found by researchers using frontier AI, not by attackers, and con-leche is "one of the most important milestones in Lean's history." He also raises a "trusting trust" worry: if AI swept the kernel for bugs and also helped write the verified checker, how do we know it did not plant a subtle backdoor? The current answer is diversity: many kernels, in different languages, run on inert exports, plus human audits.

## 8. Future Uses for Models: Prospects and Open Problems

### Where this is heading

1. **Autoformalization of the mathematical literature.** Josef Urban predicted in January 2026 that "(auto)formalization may become quite easy and ubiquitous in 2026." Jared Lichtman's Mathematics Autoformalization Project (launched September 8, 2026) aims to translate "all known math into formal code." Expect Mathlib-compatible libraries orders of magnitude larger than today's, curated by AI with human-owned roadmaps (the Tau Ceti model) and recorded in registries (Palomar).
2. **Machine-checkable AI research output.** Lean certificates are becoming the norm for AI mathematical claims from OpenAI, Anthropic, Google DeepMind and Axiom. Google DeepMind explicitly frames Lean as "the validation layer for AI-generated science."
3. **Scaling RL with formal verifiers.** Every major prover since AlphaProof uses Lean as the reward function. Lean's cheap, binary verification is one of the few RL environments that scales without reward hacking, as long as the kernel is sound and statements are fixed.
4. **Verified software and agent safety.** This is probably the largest economic opportunity: specifications in Lean, AI-generated implementations and proofs, independent checking, and deployment into high-stakes systems such as authorization, cryptography, storage protocols and agent guardrails.
5. **Multi-agent orchestration around a shared proof DAG.** Anthropic's FLT run is a template for multi-agent systems engineers. Earlier attempts "struggled to maintain coordination," and the breakthrough came from Prove2Me's directed acyclic graph of theorem statements, which let dozens of agents work in parallel against a shared, verifiable state. Lean supplies the ground truth that makes such coordination tractable.

### Open challenges

- **Statement fidelity.** A perfectly checked proof of the wrong statement is worthless. Comparator, vetted Mathlib definitions and human audits are the defense, and critics argue that semantically faithful autoformalization is fundamentally hard.
- **Human understanding.** Thirteen- and seventeen-million-line proofs are verified but not *understood*. Buzzard's "essentially nothing" remark and Tao's recent writing on AI-generated papers reflect a real tension between certification and insight. Verso Blueprints and "informalization" tools are early answers.
- **Meta-theory gaps.** Lean still lacks a complete, human-written consistency proof for its abstract type theory. Carneiro's abstract definition and lean4lean verification work are ongoing.
- **Data scarcity and distribution shift.** Formal data is tiny compared with code or prose. Current provers rely on autoformalized synthetic data, which inherits translation errors.
- **Mathlib version churn.** Mathlib changes daily and Lean ships monthly. AI-generated proofs break on upgrade, and benchmark results are tied to specific versions (for example, DeepSeek-Prover-V2 used Lean 4.9.0, while Anthropic's FLT used Lean 4.33.1 with Mathlib v4.33.0). Maintaining AI-generated code at scale is unsolved.
- **Compute cost.** FLT consumed about 6 billion output tokens. Seed-Prover 1.5 uses about 10 GPU-days per hard problem. Cheaper inference and better automation (`grind`, fail-fast tactics) matter.
- **Adversarial robustness.** As AI becomes better at finding kernel exploits, verification infrastructure must be treated as security-critical: sandboxed builds, inert exports, multiple kernels, and CVE processes.

## Timeline of Key Milestones

| Date | Milestone |
|---|---|
| Jul 2013 | First commit to Lean at Microsoft Research (de Moura) |
| Jun 2014 | Lean 0.1 released |
| Jan 2015 | First university course using Lean (CMU) |
| Aug 2015 | "The Lean Theorem Prover" system description at CADE-25 |
| Jan 2017 | Lean 3.0 released |
| Jul 2017 | mathlib created (Carneiro, Hölzl) |
| Feb 2018 | Lean Zulip established |
| Apr 2018 | Lean 4 development begins |
| Oct 2019 | Natural Number Game v1.0 |
| Dec 2020 | Scholze poses the Liquid Tensor Experiment |
| 2021 | Lean 4 released; Mathlib4 repository created |
| Feb 2022 | OpenAI's lean-gym and curriculum-learning prover |
| Jul 2022 | Liquid Tensor Experiment completed |
| Jul 2023 | Lean FRO founded; Mathlib port to Lean 4 completed; LeanDojo |
| Sep 2023 | Lean 4.0.0 stable release |
| Dec 2023 | PFR formalized in about 3 weeks; FLT project launched |
| Apr 2024 | AWS Cedar Lean verification published |
| Jul 2024 | AlphaProof achieves IMO silver with Lean |
| Apr 2025 | DeepSeek-Prover-V2 and Kimina-Prover |
| Jun 2025 | ACM SIGPLAN Software Award for Lean |
| Jul 2025 | Aristotle (IMO gold, formal) and Seed-Prover; Gerko's $10M gift |
| Aug 2025 | `grind` tactic and new compiler; CSLib announced |
| Sep 2025 | Mathlib Initiative launched |
| Dec 2025 | Seed-Prover 1.5 (88% PutnamBench); Equational Theories Project completed |
| Feb–Mar 2026 | Sphere packing in dimensions 8 and 24 completed with Gauss |
| Jun 2026 | Comparator and Lean Eval launched |
| Jul 2026 | Kernel soundness bug #14576 (Collatz exploit); Amazon's record donation |
| Aug 2026 | Year 4 Part 1 roadmap; Palomar registry |
| Sep 2026 | Claude agents formalize FLT; con-leche verified checker released |
| Oct 2026 | Lean v4.35.0 ships verified kernels; Axiom formalizes CFSG |
| Nov 21, 2026 | Lean FRO and Mathlib Initiative certify the 17M-line Riemann Hypothesis formalization |

## Recommendations for a Cloud/AI Platform Engineer Building Multi-Agent Systems

1. **Use Lean as your agents' ground-truth oracle, not their judge of meaning.** Fix the specification or theorem statement first, with human review, using Mathlib definitions wherever possible. Then let agents search freely. Gate acceptance on Comparator-style statement matching plus kernel checking.
2. **Adopt the post-2026 trust pipeline by default:**
   - Build in a sandbox.
   - Export an inert proof artifact.
   - Run `lake check --paranoid` (multiple kernels, including con-leche).
   - Run `#print axioms` and reject anything beyond `propext`, `Classical.choice` and `Quot.sound`.
   - Never trust `lake build` output alone. Treat kernels as security-critical dependencies and track their CVEs.
3. **Copy the Prove2Me pattern for coordination.** A shared DAG of lemma statements, with verified nodes as the only shared state, scales to dozens of agents and avoids the context-loss failures Anthropic hit early on.
4. **Integrate through the FRO's agent-facing stack:** lean-beam for agent interaction, Nerodia for Python, and high-throughput servers such as the Kimina Lean Server for RL rollouts. Expect these to stabilize through early 2027.
5. **Pin versions and budget for churn.** Lock Lean and Mathlib versions per project, schedule upgrades, and use the zero-`sorry` rebuild as your regression test.
6. **Start with software specifications where the ROI is clearest.** Good candidates are authorization policies (Cedar-style), protocol invariants, data-privacy guarantees and agent guardrails. Use verification-guided development: a small Lean model, proofs about the model, and differential testing against production code.

## Caveats

- Many 2026 milestones (FLT, Navier–Stokes, ETH refutation, CFSG, Riemann Hypothesis) are recent announcements from companies or individuals with commercial or reputational stakes. Lean certifies that a formal statement follows from Lean's axioms. It does not certify that the statement means what a press release says, that the mathematics is understood, or that no checker bug was exploited. The Riemann Hypothesis certification in particular is less than a day old as of this writing, and human review has not yet happened.
- Mathlib statistics change daily. The figures here are snapshots from mid-to-late 2026.
- Sources disagree on de Moura's AWS title (senior principal applied scientist on the FRO site vs. VP and Distinguished Scientist on his own site), and on line counts for the 24-dimensional sphere-packing formalization (about 120K vs. 450K raw lines, released at about 200K).
- Benchmark numbers come from different sample budgets (pass@32 vs. pass@8192), Lean versions and benchmark revisions, so they are not directly comparable.
- Lean's foundational meta-theory (unique typing, a complete relative-consistency proof of the abstract type theory) remains incomplete, although con-leche gives a formally verified consistency guarantee for a specific checker relative to set theory.
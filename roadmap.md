# Metal DOL Remediation Roadmap

## Executive Summary

Transform Metal DOL from prototype to production in 4 phases over 8 weeks.
Each phase builds on the previous, with clear deliverables and success metrics.

---

## Phase 1: Foundation (Week 1-2)

### Objective
Establish solid codebase with tests, documentation, and proper project structure.

### Tasks

#### 1.1 Project Structure Setup
**Agent**: orchestrator
**Priority**: P0
**Effort**: 2h
**Dependencies**: None

```
Actions:
- Create proper src/ directory structure
- Move existing files into correct locations
- Set up tests/ directory
- Create examples/ with subdirectories
- Create docs/ directory
```

**Deliverables**:
- [ ] `src/lib.rs` with module declarations
- [ ] `src/lexer.rs`, `src/parser.rs`, `src/ast.rs`
- [ ] `src/bin/` directory for CLI tools
- [ ] `tests/` directory structure
- [ ] `examples/` with genes/, traits/, constraints/, systems/

---

#### 1.2 Cargo.toml Enhancement
**Agent**: orchestrator
**Priority**: P0
**Effort**: 1h
**Dependencies**: 1.1

```toml
Key additions:
- Full metadata (authors, description, repository, license)
- Binary targets for CLI tools
- Feature flags (cli, serde)
- Dev dependencies (pretty_assertions, insta, criterion)
- Release profile optimizations
```

**Deliverables**:
- [ ] Complete Cargo.toml with all metadata
- [ ] Binary targets configured
- [ ] Features defined
- [ ] `cargo build` succeeds

---

#### 1.3 Error Types Definition
**Agent**: lexer-agent
**Priority**: P0
**Effort**: 2h
**Dependencies**: 1.1

```rust
// src/error.rs
Create:
- LexError with variants
- ParseError with span information
- ValidationError for semantic checks
- Implement Display, Error traits via thiserror
```

**Deliverables**:
- [ ] `src/error.rs` with all error types
- [ ] Span information in all errors
- [ ] Helpful error messages
- [ ] thiserror integration

---

#### 1.4 Lexer Documentation & Tests
**Agent**: lexer-agent
**Priority**: P0
**Effort**: 6h
**Dependencies**: 1.3

```rust
Documentation:
- Module-level docs with examples
- Doc comments on Token, TokenKind, Lexer
- Doc comments on all public methods

Tests (20+ tests):
- Keyword recognition (all 20+ keywords)
- Identifier parsing (simple, qualified, with underscores)
- Version number parsing
- Operator parsing
- String literal parsing
- Whitespace/comment handling
- Span tracking accuracy
- Error cases
```

**Deliverables**:
- [ ] `src/lexer.rs` fully documented
- [ ] `tests/lexer_tests.rs` with 20+ tests
- [ ] All tests passing
- [ ] Doc tests passing

---

#### 1.5 AST Documentation
**Agent**: parser-agent
**Priority**: P0
**Effort**: 3h
**Dependencies**: 1.1

```rust
Documentation:
- Module-level docs explaining AST structure
- Doc comments on Declaration enum and all variants
- Doc comments on Statement enum and all variants
- Doc comments on Span, Gene, Trait, Constraint, System, Evolution
- Examples showing AST construction
```

**Deliverables**:
- [ ] `src/ast.rs` fully documented
- [ ] Examples in doc comments
- [ ] `cargo doc` generates clean documentation

---

#### 1.6 Parser Documentation & Tests
**Agent**: parser-agent
**Priority**: P0
**Effort**: 8h
**Dependencies**: 1.4, 1.5

```rust
Documentation:
- Module-level docs with parsing examples
- Doc comments on Parser struct and methods
- Error handling documentation

Tests (20+ tests):
- Gene parsing (simple, with derives, with requires)
- Trait parsing (with uses, with emits)
- Constraint parsing (matches, never)
- System parsing (with version, with requirements)
- Evolution parsing (adds, deprecates, because)
- Exegesis parsing (required, multiline)
- Error recovery tests
- Span accuracy tests
```

**Deliverables**:
- [ ] `src/parser.rs` fully documented
- [ ] `tests/parser_tests.rs` with 20+ tests
- [ ] All tests passing
- [ ] Error messages include source location

---

#### 1.7 EBNF Grammar Specification
**Agent**: docs-agent
**Priority**: P1
**Effort**: 3h
**Dependencies**: 1.4, 1.6

```
Create formal grammar:
- Lexical rules (all token types)
- Syntactic rules (all declarations)
- Statement rules (all predicates)
- Test file syntax
- Comments and examples
```

**Deliverables**:
- [ ] `docs/grammar.ebnf` complete
- [ ] All language constructs covered
- [ ] Examples included
- [ ] Cross-referenced with parser

---

### Phase 1 Success Metrics

| Metric | Target |
|--------|--------|
| Lexer tests | 20+ passing |
| Parser tests | 20+ passing |
| Doc coverage | 100% public items |
| Build status | `cargo build` clean |
| Lint status | `cargo clippy` clean |

---

## Phase 2: Tooling (Week 3-4)

### Objective
Implement CLI tools that make DOL usable for developers.

### Tasks

#### 2.1 dol-parse CLI
**Agent**: cli-agent
**Priority**: P0
**Effort**: 8h
**Dependencies**: Phase 1 complete

```rust
// src/bin/dol-parse.rs
Features:
- Parse single file or directory
- Output formats: pretty, json, ast-debug
- Validation mode (--validate)
- Recursive directory scanning
- Exit codes for CI integration
```

**Deliverables**:
- [ ] `dol-parse` binary functional
- [ ] JSON output format
- [ ] Directory scanning
- [ ] CI-friendly exit codes
- [ ] Help text and usage examples

---

#### 2.2 Validator Implementation
**Agent**: parser-agent
**Priority**: P0
**Effort**: 6h
**Dependencies**: 2.1

```rust
// src/validator.rs
Validations:
- Exegesis present and non-empty
- Qualified identifiers follow naming conventions
- Version numbers are valid semver
- Uses references exist (when cross-file validation enabled)
- Evolution parent versions exist
- No duplicate statements
```

**Deliverables**:
- [ ] `src/validator.rs` with all validations
- [ ] Integration with dol-parse
- [ ] Helpful validation messages
- [ ] Tests for each validation rule

---

#### 2.3 dol-test Code Generator
**Agent**: cli-agent
**Priority**: P1
**Effort**: 12h
**Dependencies**: 2.2

```rust
// src/bin/dol-test.rs
Features:
- Parse .dol.test files
- Generate Rust test functions
- Support given/when/then/always syntax
- Output to specified directory
- Stub mode for scaffolding only
```

**Test File Parsing**:
```rust
// src/test_parser.rs
Parse test syntax:
- test declarations
- given clauses
- when clauses  
- then clauses
- always marker
```

**Code Generation**:
```rust
// src/codegen.rs
Generate:
- #[test] functions
- Arrange/Act/Assert structure
- Placeholder assertions
- Module structure
```

**Deliverables**:
- [ ] `dol-test` binary functional
- [ ] Test file parser
- [ ] Rust code generator
- [ ] Generated tests compile
- [ ] Stub mode working

---

#### 2.4 dol-check Validation Tool
**Agent**: cli-agent
**Priority**: P1
**Effort**: 8h
**Dependencies**: 2.2

```rust
// src/bin/dol-check.rs
Features:
- Validate all .dol files in directory
- Check exegesis coverage
- Compare against source directory (optional)
- Output coverage percentage
- CI gate mode (exit 1 on failure)
```

**Deliverables**:
- [ ] `dol-check` binary functional
- [ ] Exegesis enforcement
- [ ] Coverage reporting
- [ ] CI integration ready

---

#### 2.5 CLI Integration Tests
**Agent**: cli-agent
**Priority**: P1
**Effort**: 4h
**Dependencies**: 2.1, 2.3, 2.4

```rust
// tests/cli_tests.rs
Tests:
- dol-parse with valid files
- dol-parse with invalid files (exit codes)
- dol-parse JSON output correctness
- dol-test generation and compilation
- dol-check validation reporting
```

**Deliverables**:
- [ ] CLI integration tests
- [ ] Exit code verification
- [ ] Output format verification

---

### Phase 2 Success Metrics

| Metric | Target |
|--------|--------|
| dol-parse | Functional with JSON output |
| dol-test | Generates compiling Rust tests |
| dol-check | CI-ready with coverage |
| CLI tests | All passing |

---

## Phase 3: Documentation (Week 5-6)

### Objective
Create comprehensive documentation and examples for developer adoption.

### Tasks

#### 3.1 Example DOL Files
**Agent**: docs-agent
**Priority**: P0
**Effort**: 4h
**Dependencies**: Phase 2 complete

```
Create examples:
examples/
├── genes/
│   ├── container.exists.dol
│   ├── identity.cryptographic.dol
│   ├── state.finite.dol
│   └── node.exists.dol
├── traits/
│   ├── container.lifecycle.dol
│   ├── container.lifecycle.dol.test
│   ├── node.discovery.dol
│   └── node.discovery.dol.test
├── constraints/
│   ├── container.integrity.dol
│   └── cluster.consistency.dol
└── systems/
    ├── univrs.orchestrator.dol
    └── univrs.scheduler.dol
```

**Deliverables**:
- [ ] 4+ gene examples
- [ ] 4+ trait examples with tests
- [ ] 2+ constraint examples
- [ ] 2+ system examples
- [ ] All examples parse successfully

---

#### 3.2 Tutorial: First Gene
**Agent**: docs-agent
**Priority**: P1
**Effort**: 3h
**Dependencies**: 3.1

```markdown
docs/tutorials/01-first-gene.md
Contents:
- What is a gene?
- Creating your first gene file
- Parsing and validating
- Understanding the structure
- Adding exegesis
```

**Deliverables**:
- [ ] Tutorial written
- [ ] Code examples included
- [ ] Commands verified working

---

#### 3.3 Tutorial: Composing Traits
**Agent**: docs-agent
**Priority**: P1
**Effort**: 3h
**Dependencies**: 3.2

```markdown
docs/tutorials/02-composing-traits.md
Contents:
- What is a trait?
- Using genes with 'uses'
- Declaring behaviors with 'is'
- Event emission with 'emits'
- Writing trait tests
```

**Deliverables**:
- [ ] Tutorial written
- [ ] Builds on Tutorial 1
- [ ] Test examples included

---

#### 3.4 Tutorial: Writing Constraints
**Agent**: docs-agent
**Priority**: P1
**Effort**: 3h
**Dependencies**: 3.3

```markdown
docs/tutorials/03-writing-constraints.md
Contents:
- What is a constraint?
- Using 'matches' for equivalence
- Using 'never' for negative constraints
- Constraint testing patterns
```

**Deliverables**:
- [ ] Tutorial written
- [ ] Constraint examples
- [ ] Testing patterns

---

#### 3.5 Tutorial: Evolution & Versioning
**Agent**: docs-agent
**Priority**: P2
**Effort**: 3h
**Dependencies**: 3.4

```markdown
docs/tutorials/04-evolution.md
Contents:
- Why accumulative evolution?
- Creating evolution declarations
- Using adds/deprecates/removes
- Version lineage with >
- Rationale with 'because'
```

**Deliverables**:
- [ ] Tutorial written
- [ ] Evolution examples
- [ ] Version management guidance

---

#### 3.6 Language Specification Document
**Agent**: docs-agent
**Priority**: P1
**Effort**: 4h
**Dependencies**: 3.1

```markdown
docs/specification.md
Contents:
- Introduction
- Lexical grammar
- Syntactic grammar (reference EBNF)
- Semantic rules
- File structure
- Test syntax
- Examples
```

**Deliverables**:
- [ ] Complete specification
- [ ] Cross-references grammar.ebnf
- [ ] All constructs documented

---

#### 3.7 README Enhancement
**Agent**: docs-agent
**Priority**: P0
**Effort**: 4h
**Dependencies**: 3.1, 3.2

```markdown
README.md additions:
- Quick Start section
- Installation instructions
- Basic usage examples
- Project structure documentation
- Link to tutorials
- Contributing section
- License information
```

**Deliverables**:
- [ ] Comprehensive README
- [ ] Working quick start
- [ ] All links valid

---

### Phase 3 Success Metrics

| Metric | Target |
|--------|--------|
| Example files | 12+ DOL files |
| Tutorials | 4 complete |
| Specification | Complete |
| README | Production-ready |

---

## Phase 4: Community (Week 7-8)

### Objective
Prepare for public release with CI/CD and community infrastructure.

### Tasks

#### 4.1 GitHub Actions CI
**Agent**: ci-agent
**Priority**: P0
**Effort**: 4h
**Dependencies**: Phase 3 complete

```yaml
.github/workflows/ci.yml
Jobs:
- check (cargo check)
- test (cargo test)
- fmt (cargo fmt --check)
- clippy (cargo clippy)
- docs (cargo doc)
- dol-validate (validate all examples)
```

**Deliverables**:
- [ ] CI workflow file
- [ ] All jobs passing
- [ ] Branch protection ready

---

#### 4.2 Release Workflow
**Agent**: ci-agent
**Priority**: P1
**Effort**: 3h
**Dependencies**: 4.1

```yaml
.github/workflows/release.yml
Triggers: tags v*
Actions:
- Build release binaries
- Generate changelog
- Create GitHub release
- Publish to crates.io (optional)
```

**Deliverables**:
- [ ] Release workflow
- [ ] Changelog generation
- [ ] Binary artifacts

---

#### 4.3 CHANGELOG.md
**Agent**: docs-agent
**Priority**: P1
**Effort**: 2h
**Dependencies**: 4.1

```markdown
CHANGELOG.md
Format: Keep a Changelog
Contents:
- Unreleased section
- v0.0.1 release notes
- All changes categorized
```

**Deliverables**:
- [ ] CHANGELOG.md created
- [ ] Follows standard format
- [ ] All changes documented

---

#### 4.4 CONTRIBUTING.md
**Agent**: docs-agent
**Priority**: P1
**Effort**: 2h
**Dependencies**: 4.1

```markdown
CONTRIBUTING.md
Contents:
- Getting started
- Development setup
- Code style
- Testing requirements
- PR process
- Community guidelines
```

**Deliverables**:
- [ ] CONTRIBUTING.md complete
- [ ] Clear guidelines
- [ ] Links to relevant docs

---

#### 4.5 Issue Templates
**Agent**: ci-agent
**Priority**: P2
**Effort**: 1h
**Dependencies**: 4.1

```yaml
.github/ISSUE_TEMPLATE/
- bug_report.md
- feature_request.md
- question.md
```

**Deliverables**:
- [ ] Bug report template
- [ ] Feature request template
- [ ] Templates render correctly

---

#### 4.6 First Release (v0.0.1)
**Agent**: orchestrator
**Priority**: P0
**Effort**: 2h
**Dependencies**: All above

```
Release checklist:
- All CI checks passing
- CHANGELOG updated
- Version bumped in Cargo.toml
- Git tag created
- Release notes written
- Binaries attached
```

**Deliverables**:
- [ ] v0.0.1 tag created
- [ ] GitHub release published
- [ ] Release notes complete

---

### Phase 4 Success Metrics

| Metric | Target |
|--------|--------|
| CI pipeline | All green |
| Release automation | Functional |
| Community files | Complete |
| First release | Published |

---

## Task Dependencies Graph

```
Phase 1: Foundation
├── 1.1 Project Structure ──┬──► 1.2 Cargo.toml
│                           ├──► 1.3 Error Types ──► 1.4 Lexer Docs/Tests
│                           └──► 1.5 AST Docs ──────► 1.6 Parser Docs/Tests
│                                                          │
└── 1.7 EBNF Grammar ◄─────────────────────────────────────┘

Phase 2: Tooling
├── 2.1 dol-parse ──► 2.2 Validator ──┬──► 2.3 dol-test
│                                      └──► 2.4 dol-check
└── 2.5 CLI Tests ◄────────────────────────┘

Phase 3: Documentation
├── 3.1 Examples ──┬──► 3.2 Tutorial 1 ──► 3.3 Tutorial 2
│                  ├──► 3.4 Tutorial 3 ──► 3.5 Tutorial 4
│                  └──► 3.6 Specification
└── 3.7 README ◄───────┘

Phase 4: Community
├── 4.1 CI ──┬──► 4.2 Release Workflow
│            ├──► 4.3 CHANGELOG
│            ├──► 4.4 CONTRIBUTING
│            └──► 4.5 Issue Templates
└── 4.6 First Release ◄──────────────┘
```

---

## Agent Assignments Summary

| Agent | Tasks | Total Effort |
|-------|-------|--------------|
| orchestrator | 1.1, 1.2, 4.6 | 5h |
| lexer-agent | 1.3, 1.4 | 8h |
| parser-agent | 1.5, 1.6, 2.2 | 17h |
| cli-agent | 2.1, 2.3, 2.4, 2.5 | 32h |
| docs-agent | 1.7, 3.1-3.7, 4.3, 4.4 | 28h |
| ci-agent | 4.1, 4.2, 4.5 | 8h |

**Total Estimated Effort**: ~98 hours

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Parser complexity | Start with happy path, add error recovery incrementally |
| Test generation complexity | Begin with stub mode, add full generation later |
| Documentation drift | Validate all code examples in CI |
| Scope creep | Strict phase boundaries, defer enhancements to v0.0.2 |

---

## Definition of Done

A task is complete when:
1. Code compiles without warnings
2. All tests pass
3. Documentation is complete
4. Code review approved (self-review for solo work)
5. Integrated into main branch

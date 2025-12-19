# Claude-Flow Kickoff: Metal DOL Remediation

## Mission Brief

You are orchestrating the transformation of **Metal DOL** from an early-stage prototype into a production-ready DSL toolchain. This project implements a Design Ontology Language for ontology-first software development.

## Current State Assessment

The repository at `https://github.com/univrs/metadol` contains:
- Basic lexer implementation (needs tests and docs)
- Basic parser implementation (needs tests and docs)
- AST definitions (needs docs)
- No tests, no CI, no examples

## Target State

A production-ready Rust crate with:
- 40+ unit tests (lexer + parser)
- Complete documentation (doc comments on all public items)
- 3 CLI tools (dol-parse, dol-test, dol-check)
- Formal EBNF grammar
- 12+ example DOL files
- 4 tutorials
- GitHub Actions CI/CD
- First release (v0.0.1)

## Phase 1 Execution Plan

### Immediate Actions (Start Now)

#### 1. Project Structure Setup
```bash
# Ensure directory structure exists
mkdir -p src/bin tests examples/{genes,traits,constraints,systems} docs/tutorials .github/workflows
```

#### 2. Copy Scaffolding
The following files are ready to integrate:
- `src/lib.rs` - Library entry with module declarations
- `src/error.rs` - Error types with thiserror
- `src/ast.rs` - Complete AST definitions
- `src/lexer.rs` - Lexer with documentation
- `src/parser.rs` - Parser with documentation
- `src/validator.rs` - Validation rules
- `tests/lexer_tests.rs` - Lexer test suite
- `tests/parser_tests.rs` - Parser test suite
- `docs/grammar.ebnf` - Formal grammar
- `Cargo.toml` - Enhanced manifest

### Agent Task Assignments

#### Lexer Agent Tasks
1. **Verify src/lexer.rs** - Ensure all doc comments are present
2. **Run tests/lexer_tests.rs** - Verify all 20+ tests pass
3. **Add missing tests** for edge cases:
   - Deeply nested qualified identifiers
   - Unicode in strings
   - Very long identifiers
   - Comments at EOF

#### Parser Agent Tasks
1. **Verify src/parser.rs** - Ensure all doc comments are present
2. **Run tests/parser_tests.rs** - Verify all 20+ tests pass
3. **Fix exegesis parsing** - Currently uses hack, needs proper implementation
4. **Add error recovery** - Don't fail on first error

#### Docs Agent Tasks
1. **Verify docs/grammar.ebnf** - Cross-reference with parser
2. **Create examples/genes/container.exists.dol**:
```dol
gene container.exists {
  container has identity
  container has state
  container has boundaries
  container has lifecycle
}

exegesis {
  A container is the fundamental unit of workload isolation in Univrs.
  It encapsulates a running process with its dependencies, providing
  resource constraints and security boundaries. Every container has
  a cryptographic identity that persists across its lifecycle.
}
```

3. **Create examples/traits/container.lifecycle.dol**:
```dol
trait container.lifecycle {
  uses container.exists
  
  container is created
  container is starting
  container is running
  container is stopping
  container is stopped
  container is removing
  container is removed
  
  each transition emits event
}

exegesis {
  The container lifecycle defines the state machine that governs
  container execution. Transitions between states are atomic and
  emit events for observability. The lifecycle ensures predictable
  behavior from creation through removal.
}
```

### Verification Commands

After each task, verify with:

```bash
# Build check
cargo check

# Run all tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build docs
cargo doc --no-deps

# Parse example files (once dol-parse exists)
cargo run --bin dol-parse -- examples/genes/container.exists.dol
```

### Phase 1 Success Criteria

Before proceeding to Phase 2:
- [ ] `cargo test` shows 40+ tests passing
- [ ] `cargo clippy` shows 0 warnings
- [ ] `cargo doc` builds without errors
- [ ] All public items have doc comments
- [ ] docs/grammar.ebnf exists and is complete
- [ ] At least 4 example .dol files parse successfully

## File Contents Reference

### Key Files Already Prepared

The orchestration package includes complete implementations of:

1. **src/lib.rs** (4KB) - Module declarations, parse_file(), parse_and_validate()
2. **src/error.rs** (11KB) - LexError, ParseError, ValidationError with spans
3. **src/ast.rs** (18KB) - Declaration, Gene, Trait, Constraint, System, Evolution, Statement
4. **src/lexer.rs** (22KB) - Full tokenizer with 30+ token types
5. **src/parser.rs** (21KB) - Recursive descent parser for all declaration types
6. **src/validator.rs** (14KB) - Semantic validation rules

### Test Suites Ready

1. **tests/lexer_tests.rs** (12KB) - 20+ tests covering:
   - All keyword types
   - Qualified identifiers
   - Version numbers
   - String literals
   - Operators and delimiters
   - Comments and whitespace
   - Error handling

2. **tests/parser_tests.rs** (14KB) - 20+ tests covering:
   - Gene parsing
   - Trait parsing
   - Constraint parsing
   - System parsing
   - Evolution parsing
   - Error recovery
   - Exegesis validation

## Coordination Protocol

### Agent Communication
- Use task IDs from tasks.yaml for reference
- Report completion with: `[TASK_COMPLETE] task_1_4_lexer_docs_tests`
- Report blockers with: `[BLOCKED] task_id: reason`
- Request review with: `[REVIEW_NEEDED] task_id: files`

### Dependency Order
```
task_1_1 (structure) ──┬──► task_1_2 (cargo)
                       ├──► task_1_3 (errors) ──► task_1_4 (lexer)
                       └──► task_1_5 (ast) ──────► task_1_6 (parser)
                                                        │
task_1_7 (grammar) ◄────────────────────────────────────┘
```

### Commit Convention
```
feat(lexer): add comprehensive test suite

- Add 20+ unit tests for tokenization
- Test all keyword types
- Test qualified identifiers
- Test error handling

Closes task_1_4_lexer_docs_tests
```

## Begin Execution

Start with parallel execution of:
1. **orchestrator**: task_1_1_project_structure
2. **lexer-agent**: task_1_3_error_types (after 1.1)
3. **parser-agent**: task_1_5_ast_docs (after 1.1)
4. **docs-agent**: Begin grammar review

Report status after each task completion. Aim for Phase 1 completion in 2 sessions.

---

**Project Reference**: CLAUDE.md
**Detailed Roadmap**: docs/roadmap.md
**Task Definitions**: .claude-flow/tasks.yaml
**Orchestration Config**: .claude-flow/orchestration.yaml

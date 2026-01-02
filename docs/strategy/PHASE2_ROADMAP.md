# Phase 2: Tooling Implementation

## Objective
Implement the three CLI tools that make DOL usable for developers.

## Prerequisites (✅ Complete from Phase 1)
- 141 tests passing
- Clean clippy/fmt
- Parser fixes implemented
- docs/specification.md complete

## Task Breakdown

### Task 2.1: dol-parse CLI (8 hours)
**Agent**: cli-agent
**File**: `src/bin/dol-parse.rs`

**Features to implement:**
```bash
dol-parse file.dol              # Parse single file
dol-parse --format json file.dol # JSON output
dol-parse --recursive dir/       # Parse directory
dol-parse --validate dir/        # Validate only
dol-parse --ci dir/              # CI mode (exit codes)
```

**Implementation template**: See `/phase2/src/bin/dol-parse.rs`

**Acceptance criteria:**
- [ ] Parse single file successfully
- [ ] Recursive directory traversal
- [ ] JSON output format works
- [ ] CI-friendly exit codes (0=success, 1=failure)
- [ ] Colored terminal output

---

### Task 2.2: Validator Enhancement (6 hours)
**Agent**: parser-agent
**File**: `src/validator.rs`

**Validations to add:**
1. Exegesis non-empty check
2. Exegesis minimum length check
3. Naming convention validation (dot notation)
4. Version format validation (semver)
5. Duplicate statement detection
6. Cross-reference resolution (optional)

**Acceptance criteria:**
- [ ] All validation rules implemented
- [ ] Helpful error messages
- [ ] Integration with dol-parse --validate

---

### Task 2.3: dol-test Generator (12 hours)
**Agent**: cli-agent
**Files**: 
- `src/bin/dol-test.rs`
- `src/test_parser.rs`
- `src/codegen.rs`

**Features to implement:**
```bash
dol-test file.dol.test           # Generate tests
dol-test --output tests/gen/     # Custom output dir
dol-test --stubs file.dol.test   # Generate stubs only
dol-test validate file.dol.test  # Validate syntax
dol-test plan file.dol.test      # Show generation plan
```

**Test file syntax:**
```
test container.lifecycle {
    test "container can be created" {
        given:
            container does not exist
        when:
            create container
        then:
            container exists
            container is created
    }
}
```

**Implementation template**: See `/phase2/src/bin/dol-test.rs`
**Example test file**: See `/phase2/examples/traits/container.lifecycle.dol.test`

**Acceptance criteria:**
- [ ] Parse .dol.test files
- [ ] Generate compiling Rust tests
- [ ] Stub mode works
- [ ] Generated tests are formatted

---

### Task 2.4: dol-check Validator (8 hours)
**Agent**: cli-agent
**File**: `src/bin/dol-check.rs`

**Features to implement:**
```bash
dol-check dir/                    # Check all DOL files
dol-check --require-exegesis dir/ # Enforce exegesis
dol-check --strict dir/           # Warnings as errors
dol-check --ci dir/               # CI mode
dol-check --coverage-source src/  # Coverage check
```

**Implementation template**: See `/phase2/src/bin/dol-check.rs`

**Acceptance criteria:**
- [ ] Validate all DOL files recursively
- [ ] Exegesis enforcement
- [ ] Coverage percentage output
- [ ] JSON output format
- [ ] CI-friendly exit codes

---

### Task 2.5: CLI Integration Tests (4 hours)
**Agent**: cli-agent  
**File**: `tests/cli_tests.rs`

**Tests to write:**
```rust
#[test] fn test_dol_parse_single_file() { ... }
#[test] fn test_dol_parse_json_output() { ... }
#[test] fn test_dol_parse_invalid_file() { ... }
#[test] fn test_dol_test_generates_rust() { ... }
#[test] fn test_dol_check_finds_errors() { ... }
#[test] fn test_dol_check_ci_exit_codes() { ... }
```

**Acceptance criteria:**
- [ ] All CLI tools have integration tests
- [ ] Exit codes verified
- [ ] Output formats verified

---

## Dependencies Required in Cargo.toml

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
colored = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
walkdir = "2.4"  # Optional: for recursive directory traversal
```

---

## Phase 2 Success Criteria

| Check | Target |
|-------|--------|
| `cargo build --features cli` | ✅ Compiles |
| `dol-parse examples/` | ✅ Parses all files |
| `dol-test examples/*.dol.test` | ✅ Generates Rust tests |
| `dol-check --ci examples/` | ✅ Exit code 0 |
| Integration tests | ✅ All passing |

---

## Parallel Execution Plan

```
┌─────────────────────────────────────────┐
│  cli-agent                              │
│  ├── task_2_1_dol_parse (8h)           │
│  │       ↓                              │
│  ├── task_2_3_dol_test (12h)           │
│  │       ↓                              │
│  ├── task_2_4_dol_check (8h)           │
│  │       ↓                              │
│  └── task_2_5_cli_tests (4h)           │
└─────────────────────────────────────────┘
         ↑
┌─────────────────────────────────────────┐
│  parser-agent                           │
│  └── task_2_2_validator (6h)           │
│      (can run in parallel)              │
└─────────────────────────────────────────┘
```

---

## File Locations

Templates are ready at:
- `/home/claude/phase2/src/bin/dol-parse.rs`
- `/home/claude/phase2/src/bin/dol-test.rs`
- `/home/claude/phase2/src/bin/dol-check.rs`
- `/home/claude/phase2/examples/traits/container.lifecycle.dol.test`

Copy to metadol repo and adapt as needed.

# DOL Development Workflow

> **Branch Strategy for Self-Hosting Development**
> *Every commit tested. Every merge verified. Every release proven.*

---

## Branch Structure

```
main                           # Production - always deployable
│
├── develop                    # Integration branch
│   │
│   ├── feature/HIR-dolv3      # Current: HIR development
│   │   ├── hir-types          # Sub-feature branches
│   │   ├── hir-desugar
│   │   ├── hir-validate
│   │   └── hir-emit
│   │
│   ├── feature/mlir-dialect   # Future: MLIR integration
│   ├── feature/mcp-server     # Future: MCP tools
│   └── feature/self-host      # Future: DOL-in-DOL
│
├── release/v0.4.0             # Release candidates
└── hotfix/critical-fix        # Emergency fixes
```

---

## Branch Naming Convention

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feature/<scope>-<description>` | `feature/HIR-dolv3` |
| Sub-feature | `feature/<parent>/<sub>` | `feature/HIR-dolv3/hir-types` |
| Bugfix | `fix/<issue>-<description>` | `fix/42-parser-panic` |
| Release | `release/v<semver>` | `release/v0.4.0` |
| Hotfix | `hotfix/<description>` | `hotfix/bootstrap-regression` |

---

## Workflow Rules

### 1. Feature Development

```bash
# Start new feature from develop
git checkout develop
git pull origin develop
git checkout -b feature/HIR-dolv3

# Work in sub-branches for isolation
git checkout -b feature/HIR-dolv3/hir-types

# Regular commits with conventional format
git commit -m "feat(hir): add HirType canonical forms"
git commit -m "test(hir): add type validation tests"
git commit -m "docs(hir): document desugaring rules"

# Push and create PR to parent feature branch
git push -u origin feature/HIR-dolv3/hir-types
# PR: feature/HIR-dolv3/hir-types → feature/HIR-dolv3

# After review, merge to feature branch
# Then PR: feature/HIR-dolv3 → develop
```

### 2. Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding/updating tests
- `docs`: Documentation
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `ci`: CI/CD changes
- `chore`: Maintenance

**Scopes:**
- `hir`: HIR-related changes
- `parser`: Parser changes
- `lexer`: Lexer changes
- `codegen`: Code generation
- `mlir`: MLIR dialect
- `mcp`: MCP server
- `bootstrap`: Bootstrap compilation

### 3. PR Requirements

**Before creating PR:**
- [ ] All tests pass locally (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] DOL files parse (`make check`)
- [ ] Bootstrap compiles (`make bootstrap`)

**PR checklist:**
- [ ] Descriptive title with conventional commit format
- [ ] Description explains what and why
- [ ] Tests added for new functionality
- [ ] Documentation updated if needed
- [ ] No unrelated changes

### 4. Merge Strategy

| Merge Type | When | Command |
|------------|------|---------|
| Squash | Sub-feature → Feature | Combines commits |
| Merge | Feature → Develop | Preserves history |
| Rebase | Hotfix → Main | Linear history |

---

## Testing Requirements

### Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │  ← Bootstrap verification
                    │   (Few, Slow)   │
                    ├─────────────────┤
                    │ Integration     │  ← Multi-module tests
                    │ Tests (Medium)  │
                    ├─────────────────┤
                    │   Unit Tests    │  ← Per-function tests
                    │  (Many, Fast)   │
                    └─────────────────┘
```

### Test Requirements by Scope

| Scope | Unit Tests | Integration | E2E |
|-------|------------|-------------|-----|
| HIR Types | Required | Required | - |
| Desugaring | Required | Required | Required |
| Validation | Required | Required | - |
| Codegen | Required | Required | Required |
| Bootstrap | - | Required | Required |

### Coverage Targets

| Module | Target | Current |
|--------|--------|---------|
| Lexer | 90% | ~85% |
| Parser | 90% | ~80% |
| HIR | 95% | 0% |
| Codegen | 85% | ~60% |
| Overall | 85% | ~70% |

---

## CI/CD Gates

### On Every Push

```yaml
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test --lib
- make check  # DOL files parse
```

### On PR to develop

```yaml
- All push checks
- cargo test --all
- make bootstrap
- Coverage report
```

### On PR to main

```yaml
- All develop checks
- make flywheel  # Full pipeline
- Performance benchmarks
- Release notes generated
```

---

## Quick Commands

```bash
# Start HIR feature branch
git checkout develop && git pull
git checkout -b feature/HIR-dolv3

# Daily workflow
git fetch origin
git rebase origin/develop  # Keep up to date
cargo test && cargo fmt && cargo clippy
git add . && git commit -m "feat(hir): ..."
git push

# Before PR
make flywheel  # Full verification
git push

# After merge, cleanup
git checkout develop && git pull
git branch -d feature/HIR-dolv3/hir-types
```

---

## Release Process

### Version Bumping

| Change | Version | Example |
|--------|---------|---------|
| Breaking | Major | 0.3.1 → 1.0.0 |
| Feature | Minor | 0.3.1 → 0.4.0 |
| Fix | Patch | 0.3.1 → 0.3.2 |

### Release Checklist

1. [ ] All tests pass on develop
2. [ ] Create release branch: `release/v0.4.0`
3. [ ] Update CHANGELOG.md
4. [ ] Update version in Cargo.toml
5. [ ] PR release → main
6. [ ] Tag after merge: `git tag -a v0.4.0`
7. [ ] Push tag: `git push origin v0.4.0`
8. [ ] GitHub Release created automatically

---

*"Every commit tested. Every merge verified. Every release proven."*

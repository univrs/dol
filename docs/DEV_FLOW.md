### Development Flow
```
1. Work in sub-branches:
   git checkout -b feature/HIR-dolv3/desugar-expressions

2. Write failing test first:
   cargo test hir::desugar_tests::pipe_desugars_to_call

3. Implement to pass:
   // src/hir/desugar.rs

4. Pre-commit hook validates:
   ✓ cargo fmt
   ✓ cargo clippy
   ✓ cargo test
   ✓ DOL files parse

5. Commit with conventional format:
   git commit -m "feat(hir): implement pipe operator desugaring"

6. PR to feature branch:
   feature/HIR-dolv3/desugar-expressions → feature/HIR-dolv3

7. PR to develop when feature complete:
   feature/HIR-dolv3 → develop
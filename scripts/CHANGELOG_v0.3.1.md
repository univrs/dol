# DOL v0.3.1 - Bootstrap Compilation

## Summary

DOL source now compiles to working Rust code. This is the first milestone toward full self-hosting.

## Changes

### dol/ast.dol
- Added 25 missing gene definitions (ThisExpr, BinaryExpr, CallExpr, etc.)

### dol/token.dol
- Added 9 missing TokenKind variants (Ampersand, Bar, Caret, Eof, Error, etc.)

### scripts/bootstrap-fix.sh
- New script to apply codegen workarounds after regeneration

### Makefile
- Added bootstrap targets (make bootstrap, make check, etc.)

### .github/workflows/bootstrap.yml
- CI workflow for bootstrap validation

## Metrics

| Before | After |
|--------|-------|
| 248 errors | 0 errors |

## Commit Message

```
feat(dol): v0.3.1 - Bootstrap compilation achieved

- Add 25 missing gene definitions to ast.dol
- Add 9 missing TokenKind variants to token.dol
- Add bootstrap-fix.sh script for codegen workarounds
- Add Makefile with bootstrap targets
- Add CI workflow for bootstrap validation

Bootstrap compiles: 248 → 0 errors (with fix script)
```

## Next Steps

1. Implement codegen fixes to eliminate fix script
2. Write parser.dol (parser in DOL)
3. Full self-hosting

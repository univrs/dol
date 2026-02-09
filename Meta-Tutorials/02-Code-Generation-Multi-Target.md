# Tutorial 02: Multi-Target Code Generation

> **Generate Rust, TypeScript, Python, WIT, and JSON Schema from single DOL schemas**
>
> **Level**: Intermediate | **Time**: 60 minutes | **Lines**: 180+

## Overview

DOL's code generation transforms schemas into multiple target languages for type-safe cross-platform development. This tutorial demonstrates all 5 code generation targets.

## Prerequisites

```bash
cargo install metadol
npm install -g typescript
pip install mypy pydantic
cargo install wit-bindgen-cli
```

## Complete Example: All 5 Targets

See `Examples/ecommerce.dol` for source schema generating:
- Rust with `#[derive(Serialize)]`
- TypeScript interfaces
- Python with Pydantic
- WIT for WASM Component Model
- JSON Schema for validation

## Code Generation Commands

```bash
# Rust
dol-codegen --target rust schema.dol > schema.rs

# TypeScript
dol-codegen --target typescript schema.dol > schema.ts

# Python
dol-codegen --target python schema.dol > schema.py

# WIT
dol-codegen --target wit schema.dol > schema.wit

# JSON Schema
dol-codegen --target json-schema schema.dol > schema.json
```

## Build Automation

```bash
#!/bin/bash
# generate_all.sh
for target in rust typescript python wit json-schema; do
    dol-codegen --target $target schema.dol > generated/schema.$target
done
```

---

**Next**: [Tutorial 03: AI-Assisted Schema Design](./03-AI-Assisted-Schema-Design.md)

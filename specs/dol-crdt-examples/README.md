# DOL CRDT Reference Examples

This directory contains reference examples for all seven CRDT strategies supported by DOL 2.0.

## Examples

| File | Strategy | Description |
|------|----------|-------------|
| [01-immutable.dol](./01-immutable.dol) | `immutable` | Permanent identity fields (IDs, timestamps) |
| [02-lww.dol](./02-lww.dol) | `lww` | User metadata and configuration |
| [03-or-set.dol](./03-or-set.dol) | `or_set` | Collections with add-wins semantics |
| [04-pn-counter.dol](./04-pn-counter.dol) | `pn_counter` | Distributed counters and metrics |
| [05-peritext.dol](./05-peritext.dol) | `peritext` | Collaborative rich text editing |
| [06-rga.dol](./06-rga.dol) | `rga` | Ordered sequences and lists |
| [07-mv-register.dol](./07-mv-register.dol) | `mv_register` | Conflict detection and multi-valued state |

## Using These Examples

### Validation

Validate examples against the DOL CRDT schema:

```bash
dol-check specs/dol-crdt-examples/*.dol
```

### Code Generation

Generate Rust code with CRDT support:

```bash
dol-codegen-rust specs/dol-crdt-examples/01-immutable.dol -o output/
```

### JSON Schema Export

Export to JSON schema format:

```bash
dol-export --format json specs/dol-crdt-examples/01-immutable.dol
```

## Quick Reference

### Strategy Selection

```
Does the field change after creation?
├─ NO → @crdt(immutable)
└─ YES
   ├─ Text content? → @crdt(peritext)
   ├─ Counter? → @crdt(pn_counter)
   ├─ Unordered collection? → @crdt(or_set)
   ├─ Ordered sequence? → @crdt(rga)
   ├─ Need conflict detection? → @crdt(mv_register)
   └─ Simple value → @crdt(lww)
```

### Type Compatibility

| Type | Compatible Strategies |
|------|---------------------|
| String | immutable, lww, peritext, mv_register |
| Int/UInt | immutable, lww, pn_counter, mv_register |
| Set<T> | or_set, mv_register |
| List<T> | rga, mv_register |
| Custom Struct | immutable, lww, mv_register |

## Learning Path

1. Start with **immutable** for IDs
2. Use **lww** for simple metadata
3. Add **or_set** for collections
4. Use **pn_counter** for metrics
5. Try **peritext** for collaborative text
6. Use **rga** for ordered lists
7. Use **mv_register** when you need conflict detection

## Additional Resources

- [Specification](../dol-crdt-schema-v1.0.md)
- [Implementation Guide](../dol-crdt-implementation-guide.md)
- [JSON Schema](../../schemas/dol-crdt.json)
- [DOL Book - CRDT Guide](../../docs/book/local-first/crdt-guide/)

## License

CC BY 4.0 - Univrs Foundation, 2026

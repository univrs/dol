# DOL Meta-Programming Examples

> **20+ example schemas demonstrating all DOL features**

## Example Index

### Basic Examples (01-05)

1. **[01-simple-user.dol](./01-simple-user.dol)** - Basic schema with LWW and immutable fields
2. **[02-collaborative-document.dol](./02-collaborative-document.dol)** - Peritext for rich text editing
3. **[03-analytics-counters.dol](./03-analytics-counters.dol)** - PN-Counter for distributed counting
4. **[04-shopping-cart.dol](./04-shopping-cart.dol)** - OR-Set for collection management
5. **[05-task-list-rga.dol](./05-task-list-rga.dol)** - RGA for ordered lists

### CRDT Strategy Examples (06-12)

6. **immutable-examples.dol** - All immutable field use cases
7. **lww-examples.dol** - Last-write-wins patterns
8. **or-set-examples.dol** - Observed-remove set patterns
9. **pn-counter-examples.dol** - Positive-negative counter patterns
10. **peritext-examples.dol** - Rich text editing patterns
11. **rga-examples.dol** - Replicated growable array patterns
12. **mv-register-examples.dol** - Multi-value register patterns

### Real-World Applications (13-20)

13. **chat-application.dol** - Complete chat with rooms and messages
14. **blog-platform.dol** - Blog with posts, comments, tags
15. **project-management.dol** - Projects, tasks, milestones
16. **social-network.dol** - Users, posts, follows, likes
17. **e-commerce-full.dol** - Products, orders, inventory
18. **wiki-system.dol** - Collaborative wiki with history
19. **code-editor.dol** - Collaborative code editing
20. **calendar-app.dol** - Events, recurring tasks, reminders

### Advanced Patterns (21-25)

21. **reflection-patterns.dol** - Runtime type introspection
22. **macro-examples.dol** - Custom macro usage
23. **generic-types.dol** - Generic programming patterns
24. **constraint-examples.dol** - Complex constraints
25. **migration-examples.dol** - Schema evolution patterns

## Usage

### Generate Rust Code

```bash
dol-codegen --target rust 01-simple-user.dol > user.rs
```

### Generate TypeScript

```bash
dol-codegen --target typescript 01-simple-user.dol > user.ts
```

### Generate All Targets

```bash
for file in *.dol; do
    dol-codegen --target rust "$file" > "generated/${file%.dol}.rs"
    dol-codegen --target typescript "$file" > "generated/${file%.dol}.ts"
    dol-codegen --target python "$file" > "generated/${file%.dol}.py"
done
```

### Compile to WASM

```bash
# From Rust generated code
dol-build-wasm 13-chat-application.dol
```

## Testing Examples

```bash
# Parse all examples
for file in *.dol; do
    echo "Parsing $file..."
    dol-parse "$file" || echo "Failed: $file"
done
```

## Feature Coverage

| Feature | Examples |
|---------|----------|
| Immutable CRDT | 01, 02, 03, 04, 05, 06 |
| LWW CRDT | 01, 02, 03, 04, 07 |
| OR-Set CRDT | 02, 04, 08 |
| PN-Counter CRDT | 03, 09 |
| Peritext CRDT | 02, 10 |
| RGA CRDT | 02, 05, 11 |
| MV-Register CRDT | 12 |
| Constraints | 04, 24 |
| Enums | 05, 13, 17 |
| Methods | 01, 02, 03, 04, 05 |
| Generics | 23 |
| Macros | 22 |

## Learning Path

### Beginner

Start with examples 01-05 to understand:
- Basic schema definition
- CRDT annotations
- Type system
- Method definitions

### Intermediate

Study examples 06-12 to master:
- All 7 CRDT strategies
- When to use each strategy
- Performance trade-offs

### Advanced

Explore examples 13-25 for:
- Real-world application design
- Complex patterns
- Advanced features
- Production considerations

## Building Applications

Use these examples as starting points:

```bash
# Start a chat app
cp 13-chat-application.dol my-chat/schema.dol
cd my-chat
dol-codegen --target rust schema.dol > src/generated.rs
wasm-pack build
```

## Contributing

Add your own examples! Follow the pattern:

1. Create `XX-descriptive-name.dol`
2. Include comprehensive docs block
3. Demonstrate specific feature
4. Add to this README

---

**See also**: [Meta-Programming Tutorials](../README.md)

# DOL Meta-Programming Tutorials

> **Comprehensive guides to meta-programming, reflection, code generation, and WASM compilation in Metal DOL**

This tutorial collection demonstrates all meta-programming capabilities in DOL, from runtime reflection to multi-language code generation to full WASM deployment pipelines.

## 📚 Tutorial Index

### Core Meta-Programming

1. **[Runtime Reflection](./01-Runtime-Reflection.md)** - Query schemas at runtime using `dol-reflect`
   - TypeRegistry and TypeInfo API
   - Dynamic schema introspection
   - Hot-reload patterns
   - CRDT strategy inspection
   - **Level**: Intermediate | **Lines**: 150+

2. **[Multi-Target Code Generation](./02-Code-Generation-Multi-Target.md)** - Generate Rust, TypeScript, Python, WIT, JSON Schema
   - All 5 code generation targets
   - Template customization
   - Build system integration
   - **Level**: Intermediate | **Lines**: 180+

3. **[AI-Assisted Schema Design](./03-AI-Assisted-Schema-Design.md)** - Natural language to DOL
   - MCP tool integration
   - LSP intelligent completion
   - Schema validation with AI
   - **Level**: Advanced | **Lines**: 120+

### Macro Systems

4. **[Declarative Macros](./04-Declarative-Macros.md)** - Pattern matching and hygienic expansion
   - Built-in macros (derive, stringify, concat)
   - Custom macro creation
   - Macro hygiene rules
   - **Level**: Intermediate | **Lines**: 140+

5. **[Procedural Macros](./05-Procedural-Macros.md)** - Advanced code transformation
   - Derive macros (Debug, Clone, Gen)
   - Attribute macros (cached, async, memoize)
   - Function-like macros (sql, json, regex)
   - **Level**: Advanced | **Lines**: 160+

### WASM Compilation

6. **[DOL to WASM Pipeline](./06-DOL-to-WASM-Complete-Pipeline.md)** - Full compilation workflow
   - DOL → Rust → WASM compilation
   - CRDT annotations in WASM
   - wasm-opt optimization
   - Browser and Node.js integration
   - **Level**: Intermediate | **Lines**: 220+

7. **[CRDT Schema Design](./07-CRDT-Schema-Design.md)** - Local-first patterns
   - All 7 CRDT strategies with examples
   - Type compatibility guide
   - Constraint + CRDT interaction
   - Merge behavior patterns
   - **Level**: Advanced | **Lines**: 170+

### Advanced Patterns

8. **[Advanced Reflection Patterns](./08-Advanced-Reflection-Patterns.md)** - Meta-programming techniques
   - Runtime schema evolution
   - Generic programming with reflection
   - Type-safe serialization
   - Self-modifying programs
   - **Level**: Expert | **Lines**: 160+

9. **[Multi-Language Workflow](./09-Multi-Language-Codegen-Workflow.md)** - Cross-platform development
   - Single schema → 5 languages
   - Build automation (Makefile, scripts)
   - Package manager integration
   - **Level**: Intermediate | **Lines**: 190+

10. **[Production Deployment](./10-Production-Deployment-Guide.md)** - Going to production
    - WASM bundle optimization
    - Performance profiling
    - CI/CD pipelines
    - Monitoring and debugging
    - **Level**: Advanced | **Lines**: 210+

## 🎯 Learning Paths

### Path 1: Beginner to WASM
```
01 → 04 → 06 → 10
```
Start with reflection basics, learn macros, compile to WASM, deploy to production.

### Path 2: Advanced Meta-Programming
```
01 → 04 → 05 → 08
```
Master reflection, declarative macros, procedural macros, advanced patterns.

### Path 3: Local-First Development
```
01 → 07 → 06 → 09
```
Understand reflection, design CRDT schemas, compile to WASM, deploy multi-platform.

### Path 4: Full-Stack Type Safety
```
02 → 09 → 06 → 10
```
Generate code for all targets, automate builds, compile to WASM, production deployment.

## 📦 Prerequisites

### Software Requirements

```bash
# Required
rustc >= 1.70.0
cargo >= 1.70.0
wasm-pack >= 0.12.0

# Optional (for specific tutorials)
node >= 18.0.0        # Tutorial 06, 09
python >= 3.10        # Tutorial 02, 09
deno >= 1.30.0        # Tutorial 06
wasmtime >= 15.0.0    # Tutorial 06
```

### DOL Installation

```bash
# Install DOL from source
git clone https://github.com/univrs/dol
cd dol
cargo install --path .

# Verify installation
dol --version
```

### Example Files

All tutorials reference example files in `Meta-Tutorials/Examples/`:
- 20+ DOL schemas demonstrating all features
- Build scripts and automation
- Test harnesses

## 🔧 Quick Start

```bash
# Clone and navigate
cd Meta-Tutorials

# Run a simple example
cd Examples
dol-codegen --target rust user_profile.dol > user_profile.rs

# Compile to WASM
dol-build-wasm chat_room.dol
```

## 🌟 Feature Coverage

| Feature | Tutorials | Examples |
|---------|-----------|----------|
| Runtime Reflection | 01, 08 | 5+ |
| Code Generation | 02, 09 | 8+ |
| Declarative Macros | 04 | 6+ |
| Procedural Macros | 05 | 5+ |
| WASM Compilation | 06, 10 | 7+ |
| CRDT Strategies | 07 | 7+ |
| AI Integration | 03 | 3+ |

## 📖 Using These Tutorials

### Reading Format

Each tutorial follows this structure:

1. **Overview**: What you'll learn and prerequisites
2. **Concepts**: Core concepts explained
3. **Complete Examples**: 100-200+ lines of working code
4. **Step-by-Step Walkthrough**: Detailed explanations
5. **Common Pitfalls**: What to avoid
6. **Performance Tips**: Optimization strategies
7. **Further Reading**: Related resources

### Code Conventions

```dol
// DOL code is syntax-highlighted
gen Example {
    has field: Type
}
```

```rust
// Rust code examples are complete and runnable
fn example() {
    println!("Working code");
}
```

```bash
# Shell commands are prefixed with $
$ dol-codegen example.dol
```

### Difficulty Levels

- **Beginner**: Basic DOL knowledge required
- **Intermediate**: Familiar with DOL syntax and concepts
- **Advanced**: Deep understanding of type systems
- **Expert**: Meta-programming experience

## 🤝 Contributing

Found an issue or want to improve a tutorial?

```bash
# Open an issue
https://github.com/univrs/dol/issues

# Submit a PR
git checkout -b improve-tutorial-01
# Make changes
git commit -m "Improve Tutorial 01 with XYZ"
git push origin improve-tutorial-01
```

## 📚 Additional Resources

- [DOL Language Specification](../docs/specification.md)
- [DOL Grammar (EBNF)](../docs/grammar.ebnf)
- [API Documentation](https://docs.rs/metadol)
- [Community Discord](https://discord.gg/univrs)

## 🎓 What You'll Build

By completing these tutorials, you'll build:

1. **Reflection Engine** - Runtime schema browser (Tutorial 01)
2. **Code Generator** - Multi-language output (Tutorial 02)
3. **Schema Designer** - AI-assisted tool (Tutorial 03)
4. **Macro Library** - Custom macros (Tutorials 04-05)
5. **WASM Chat App** - Real-time collaboration (Tutorial 06)
6. **CRDT Editor** - Conflict-free text editing (Tutorial 07)
7. **Type-Safe Serializer** - Reflection-based (Tutorial 08)
8. **Polyglot SDK** - One schema, many languages (Tutorial 09)
9. **Production Service** - Deployed WASM microservice (Tutorial 10)

## 📊 Tutorial Statistics

- **Total Lines of Code**: 1,700+
- **Example Schemas**: 20+
- **Runnable Examples**: 50+
- **Build Scripts**: 10+
- **Test Cases**: 30+

## 🚀 Next Steps

1. **Start with [Tutorial 01: Runtime Reflection](./01-Runtime-Reflection.md)**
2. **Follow your chosen learning path**
3. **Build the example projects**
4. **Contribute back to the community**

---

**Last Updated**: 2026-02-07
**DOL Version**: 0.8.0+
**Maintainer**: Univrs Team

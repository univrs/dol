# DOL Specifications

This directory contains formal specifications and standards materials for the Distributed Ontology Language (DOL) project.

## Contents

### Core Specifications

- **[dol-crdt-schema-v1.0.md](./dol-crdt-schema-v1.0.md)** - The DOL CRDT Schema Specification v1.0
  - Formal specification of CRDT annotation syntax
  - Semantics for all 7 CRDT strategies
  - Type compatibility matrix
  - Validation and conformance requirements
  - 60+ pages of detailed technical specification

### Implementation Materials

- **[dol-crdt-implementation-guide.md](./dol-crdt-implementation-guide.md)** - Implementation Guide
  - For library and tool developers
  - Parser implementation guide
  - Validator implementation guide
  - Code generator patterns
  - Runtime library requirements
  - Testing and conformance

### Reference Examples

- **[dol-crdt-examples/](./dol-crdt-examples/)** - Reference Examples
  - Complete examples for all 7 CRDT strategies
  - Demonstrates best practices
  - Includes merge examples and explanations
  - See [dol-crdt-examples/README.md](./dol-crdt-examples/README.md)

### Standards Submission

- **[dol-crdt-standards-submission.md](./dol-crdt-standards-submission.md)** - Standards Body Submission Materials
  - IETF submission package
  - W3C submission package
  - ISO/IEC submission package
  - Timeline and contacts

## Quick Links

### Specifications
- [Main Specification](./dol-crdt-schema-v1.0.md)
- [JSON Schema](../schemas/dol-crdt.json)
- [RFC-001: CRDT Annotations](../rfcs/RFC-001-dol-crdt-annotations.md)

### Guides
- [Implementation Guide](./dol-crdt-implementation-guide.md)
- [Example Catalog](./dol-crdt-examples/)
- [Developer Documentation](../docs/book/local-first/crdt-guide/)

### Community
- [GitHub Repository](https://github.com/univrs/dol)
- [Standards Forum](https://forum.univrs.org/c/standards)
- Email: standards@univrs.org

## CRDT Strategies

The DOL CRDT Schema supports seven core strategies:

| Strategy | Use Case | Example |
|----------|----------|---------|
| **immutable** | IDs, timestamps | `@crdt(immutable) id: Uuid` |
| **lww** | Metadata, settings | `@crdt(lww) name: String` |
| **or_set** | Tags, collections | `@crdt(or_set) tags: Set<String>` |
| **pn_counter** | Metrics, scores | `@crdt(pn_counter) likes: Int` |
| **peritext** | Rich text | `@crdt(peritext) content: String` |
| **rga** | Ordered lists | `@crdt(rga) tasks: List<TaskId>` |
| **mv_register** | Conflict detection | `@crdt(mv_register) theme: Theme` |

## Usage

### Validate a DOL schema

```bash
dol-check schemas/my-schema.dol
```

### Generate code

```bash
dol-codegen-rust schemas/my-schema.dol -o output/
```

### Export to JSON Schema

```bash
dol-export --format json schemas/my-schema.dol
```

## Conformance Levels

**Level 1 (Basic):**
- Supports 4+ strategies (immutable, lww, or_set, pn_counter)
- Type compatibility validation
- JSON schema export

**Level 2 (Full):**
- All 7 strategies
- Constraint categorization
- Schema evolution
- Passes conformance test suite

**Level 3 (Advanced):**
- Byzantine fault tolerance
- Cross-library interoperability
- Custom strategy extensions

## Contributing

We welcome contributions to the specification!

### Feedback

- **Issues:** https://github.com/univrs/dol/issues
- **Discussions:** https://github.com/univrs/dol/discussions
- **Email:** standards@univrs.org

### Process

1. Open an issue describing the problem or suggestion
2. Discuss with maintainers and community
3. Submit a pull request with proposed changes
4. Participate in review process

## License

All specifications in this directory are licensed under:

**Creative Commons Attribution 4.0 International (CC BY 4.0)**

You are free to:
- Share — copy and redistribute the material
- Adapt — remix, transform, and build upon the material

Under the following terms:
- Attribution — You must give appropriate credit

See https://creativecommons.org/licenses/by/4.0/ for details.

## Version History

### v1.0.0 (2026-02-05)
- Initial release of DOL CRDT Schema Specification
- 7 core CRDT strategies
- Complete validation rules
- JSON schema format
- Implementation guide
- Standards submission materials

## Contact

**Univrs Foundation**
- Website: https://univrs.org
- Email: standards@univrs.org
- GitHub: https://github.com/univrs/dol

**For implementation support:**
- Discord: https://discord.gg/univrs
- Forum: https://forum.univrs.org

**For standards submission:**
- Email: standards@univrs.org

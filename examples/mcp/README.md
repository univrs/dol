# DOL MCP Server - AI-Assisted CRDT Schema Design

This directory contains examples and documentation for using the DOL Model Context Protocol (MCP) server for AI-assisted CRDT schema design.

## Overview

The DOL MCP server provides AI assistants with tools to help developers design local-first, CRDT-annotated DOL schemas. It integrates seamlessly with MCP-compatible AI assistants like Claude Desktop.

## Features

### CRDT-Specific Tools

1. **validate_schema** - Validate DOL schemas with CRDT annotations
   - Detects type-strategy incompatibilities
   - Identifies anti-patterns
   - Suggests optimizations
   - Checks for performance issues

2. **recommend_crdt** - Get intelligent CRDT strategy recommendations
   - Analyzes field type and usage pattern
   - References RFC-001 compatibility matrix
   - Explains trade-offs
   - Provides alternatives

3. **explain_strategy** - Learn about CRDT strategies
   - Detailed explanations of each strategy
   - Use cases and trade-offs
   - Best practices

4. **generate_example** - Generate example schemas
   - Common use cases (chat, tasks, profiles)
   - Complete schemas with annotations
   - Best practice examples

### General DOL Tools

- **parse** - Parse DOL source code
- **compile_rust** - Generate Rust code
- **compile_typescript** - Generate TypeScript code
- **typecheck** - Type check expressions
- **reflect** - Get runtime type information

## Quick Start

### Installation

```bash
# From the univrs-dol repository root
cargo install --path . --features cli

# Verify installation
dol-mcp
```

### Configuration

#### Claude Desktop

Copy the configuration to your Claude Desktop config directory:

```bash
# macOS
cp examples/mcp/claude-desktop-config.json ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Linux
cp examples/mcp/claude-desktop-config.json ~/.config/Claude/claude_desktop_config.json

# Windows
copy examples\mcp\claude-desktop-config.json %APPDATA%\Claude\claude_desktop_config.json
```

Or manually add to your existing config:

```json
{
  "mcpServers": {
    "dol-mcp": {
      "command": "dol-mcp",
      "args": ["serve"],
      "description": "Metal DOL MCP Server - AI-assisted CRDT schema design"
    }
  }
}
```

### Usage Examples

#### Command Line

```bash
# Get CRDT recommendation
dol-mcp tool recommend_crdt \
  field_name=content \
  field_type=String \
  usage_pattern=collaborative-text

# Explain a strategy
dol-mcp tool explain_strategy strategy=peritext

# Generate example schema
dol-mcp tool generate_example use_case=chat_message

# Validate a schema
echo 'gen message { @crdt(lww) has text: String } exegesis { Message. }' | \
  dol-mcp tool validate_schema
```

#### With AI Assistant

See [example-queries.md](./example-queries.md) for detailed AI conversation examples.

**Example Conversation:**

```
User: I need a collaborative document editor. What fields should I have?

AI: Let me help design a CRDT schema for your document editor.
    [Uses recommend_crdt for each field]
    [Generates complete schema with generate_example]

    Here's a recommended schema:

    gen document.collaborative {
      @crdt(immutable)
      has id: Uuid

      @crdt(lww)
      has title: String

      @crdt(peritext, formatting="full", max_length=1000000)
      has content: RichText

      @crdt(or_set)
      has collaborators: Set<Identity>
    }

    This schema uses:
    - Immutable ID for distributed identity
    - LWW for simple title field
    - Peritext for collaborative rich text editing
    - OR-Set for multi-user collaborator management
```

## Documentation

- [Example Queries](./example-queries.md) - Detailed AI interaction examples
- [Schema Design Workflow](./schema-design-workflow.md) - Step-by-step guide
- [RFC-001](../../rfcs/RFC-001-dol-crdt-annotations.md) - CRDT annotation specification

## Tool Reference

### validate_schema

Validates DOL schemas for CRDT correctness and best practices.

**Parameters:**
- `source` (required): DOL source code to validate

**Example:**
```bash
dol-mcp tool validate_schema source="$(cat my-schema.dol)"
```

**Returns:**
```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "crdt_issues": [
    {
      "severity": "Warning",
      "category": "Performance",
      "message": "Field 'content' uses Peritext without max_length",
      "suggestion": "Add max_length option: @crdt(peritext, max_length=1000000)",
      "field": "content"
    }
  ]
}
```

### recommend_crdt

Recommends optimal CRDT strategy based on usage pattern.

**Parameters:**
- `field_name` (required): Name of the field
- `field_type` (required): Type (e.g., "String", "i32", "Set<String>")
- `usage_pattern` (required): One of:
  - `write-once` - Set once, never modified
  - `last-write-wins` - Simple updates, LWW resolution
  - `collaborative-text` - Real-time text editing
  - `multi-user-set` - Collaborative collections
  - `counter` - Numeric counters
  - `ordered-list` - Sequences with ordering
- `consistency_requirement` (optional): "eventual" (default), "causal", or "strong"

**Example:**
```bash
dol-mcp tool recommend_crdt \
  field_name=content \
  field_type=String \
  usage_pattern=collaborative-text \
  consistency_requirement=eventual
```

**Returns:**
```json
{
  "field_name": "content",
  "field_type": "String",
  "recommended_strategy": "peritext",
  "confidence": "High",
  "reasoning": "Peritext CRDT enables conflict-free collaborative text editing...",
  "example": "@crdt(peritext, formatting=\"full\") has content: String",
  "alternatives": [...],
  "trade_offs": {
    "pros": ["Conflict-free concurrent editing", ...],
    "cons": ["Higher storage overhead", ...]
  }
}
```

### explain_strategy

Explains CRDT strategy semantics and trade-offs.

**Parameters:**
- `strategy` (required): One of: immutable, lww, peritext, or_set, pn_counter, rga, mv_register

**Example:**
```bash
dol-mcp tool explain_strategy strategy=peritext
```

**Returns:**
```
Peritext Strategy:
- Collaborative rich text editing
- Conflict-free concurrent editing
- Preserves formatting and user intent
- Based on RGA + formatting marks
- Trade-off: Higher storage/merge overhead
- Best-in-class for document collaboration
```

### generate_example

Generates example DOL schemas for common use cases.

**Parameters:**
- `use_case` (optional): One of:
  - `chat_message` (default)
  - `task_board`
  - `user_profile`
  - `counter`

**Example:**
```bash
dol-mcp tool generate_example use_case=task_board
```

**Returns:**
Complete DOL schema with CRDT annotations and exegesis.

## Best Practices

1. **Always validate** schemas after making changes
2. **Start with immutable IDs** for every replicated entity
3. **Use LWW as default** for simple fields that rarely conflict
4. **Reserve Peritext** for actual collaborative text editing
5. **Add size constraints** to collections to prevent unbounded growth
6. **Document strategy choices** in exegesis blocks
7. **Consider consistency levels** when choosing strategies

## Advanced Features

### Diagnostic Categories

The validator categorizes issues by severity and type:

- **Error**: Must be fixed (e.g., incompatible type-strategy)
- **Warning**: Should be addressed (e.g., anti-patterns)
- **Info**: Suggestions for improvement (e.g., missing constraints)

**Categories:**
- **AntiPattern**: Common mistakes (e.g., LWW on collections)
- **Performance**: Performance concerns (e.g., unbounded text)
- **Correctness**: Semantic issues (e.g., conflicting constraints)
- **Consistency**: Consistency concerns (e.g., mixed CRDT/non-CRDT)
- **BestPractice**: Recommendations (e.g., missing ID field)

### Optimization Suggestions

The diagnostics engine provides optimization suggestions:

```json
{
  "category": "Timestamp",
  "title": "Add Hybrid Logical Clock",
  "description": "Gene has 5 LWW fields but no timestamp. Consider adding an HLC for better causality tracking.",
  "impact": "Medium",
  "implementation": "@crdt(immutable) has hlc: HybridLogicalClock"
}
```

## Troubleshooting

### MCP Server Not Found

If the AI assistant cannot find the server:

1. Verify installation: `which dol-mcp`
2. Check config file syntax
3. Restart the AI assistant
4. Check logs (location varies by client)

### Type Compatibility Errors

```
Error: IncompatibleCrdtStrategy
Field 'count' uses pn_counter with type String
```

**Solution:** PN-Counter requires numeric types. Change to `i32` or use `lww`.

### Anti-Pattern Warnings

```
Warning: Field 'tags' uses LWW on collection type Set<String>
```

**Solution:** Use `@crdt(or_set)` for Set types to preserve concurrent additions.

## Contributing

To add new CRDT tools or improve recommendations:

1. Edit `src/mcp/recommendations.rs` for recommendation logic
2. Edit `src/mcp/diagnostics.rs` for validation rules
3. Edit `src/mcp/server.rs` to add new tool handlers
4. Add tests in the respective modules
5. Update documentation

## References

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [RFC-001: DOL CRDT Annotations](../../rfcs/RFC-001-dol-crdt-annotations.md)
- [DOL Documentation](../../docs/)
- [Automerge](https://automerge.org/) - CRDT implementation library

## License

MIT OR Apache-2.0 (see repository root)

## Support

For issues and questions:
- GitHub Issues: https://github.com/univrs/dol/issues
- Documentation: https://docs.rs/dol
- RFC-001: Full CRDT specification

# Tutorial 03: AI-Assisted Schema Design

> **Natural language to DOL using MCP and LSP**
>
> **Level**: Advanced | **Time**: 50 minutes | **Lines**: 120+

## Overview

AI-assisted tools accelerate schema design through:
- Natural language → DOL conversion via MCP
- Intelligent completion via LSP
- Schema validation with AI suggestions
- Automated refactoring

## Prerequisites

```bash
cargo install dol-mcp dol-lsp
npm install -g @anthropic-ai/sdk
```

## MCP Integration Example

```rust
// mcp_schema_assistant.rs
use anthropic_mcp::Client;
use metadol::parse_file;

async fn generate_schema_from_nl(description: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new()?;

    let prompt = format!(
        "Generate a DOL schema for: {}. Include appropriate CRDT annotations.",
        description
    );

    let response = client.complete(&prompt).await?;

    // Validate generated DOL
    match parse_file(&response) {
        Ok(_) => Ok(response),
        Err(e) => Err(format!("Invalid DOL generated: {}", e).into())
    }
}

#[tokio::main]
async fn main() {
    let schema = generate_schema_from_nl(
        "A collaborative document editor with real-time sync"
    ).await.unwrap();

    println!("Generated schema:\n{}", schema);
}
```

## LSP Features

```json
{
  "dol.lsp": {
    "completion": {
      "crdt_strategies": ["immutable", "lww", "or_set", "pn_counter", "peritext", "rga", "mv_register"],
      "smart_field_types": true,
      "constraint_suggestions": true
    },
    "validation": {
      "type_checking": true,
      "crdt_compatibility": true
    },
    "refactoring": {
      "extract_trait": true,
      "inline_gene": true,
      "rename_symbol": true
    }
  }
}
```

## AI-Powered Validation

```rust
// ai_validator.rs
use metadol::{parse_file, ast::Declaration};

fn validate_with_ai(schema: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    match parse_file(schema) {
        Ok(decls) => {
            for decl in &decls {
                // Check CRDT compatibility
                if let Declaration::Gene(gen) = decl {
                    for stmt in &gen.statements {
                        if let metadol::ast::Statement::HasField(field) = stmt {
                            if let Some(crdt) = &field.crdt_annotation {
                                // AI suggests optimal CRDT based on field type
                                match (field.type_, crdt.strategy) {
                                    (TypeExpr::Named(ref t), _) if t == "Set" => {
                                        suggestions.push(format!(
                                            "Consider or_set for field '{}'", field.name
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            suggestions.push(format!("Parse error: {}", e));
        }
    }

    suggestions
}
```

## Example: Chat Schema Generation

```bash
$ dol-ai-generate "chat application with rooms, messages, and reactions"

Generated schema:

gen ChatRoom {
    @crdt(immutable)
    has id: string

    @crdt(lww)
    has name: string

    @crdt(rga)
    has messages: Vec<ChatMessage>

    @crdt(or_set)
    has members: Set<string>
}

gen ChatMessage {
    @crdt(immutable)
    has id: string

    @crdt(peritext)
    has content: string

    @crdt(or_set)
    has reactions: Set<Reaction>
}

docs {
    AI-generated chat schema with optimal CRDT strategies
    for collaborative editing and real-time sync.
}
```

## Common Patterns

### Pattern 1: Iterative Refinement

```bash
$ dol-ai-refine schema.dol --suggestion "add user authentication"
$ dol-ai-refine schema.dol --suggestion "optimize for offline-first"
$ dol-ai-refine schema.dol --suggestion "add data migration support"
```

### Pattern 2: Schema Completion

```dol
// Type partially, get AI completion
gen UserProfile {
    has id: string  // AI suggests: @crdt(immutable)
    has name:       // AI suggests: string with @crdt(lww)
```

## Performance Tips

1. **Cache AI responses** for common patterns
2. **Use LSP incrementally** as you type
3. **Batch validation** for large schemas

---

**Next**: [Tutorial 04: Declarative Macros](./04-Declarative-Macros.md)

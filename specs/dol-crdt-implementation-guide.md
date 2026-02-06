# DOL CRDT Implementation Guide v1.0

**Version:** 1.0.0
**Date:** 2026-02-05
**Audience:** Library and tool developers implementing DOL CRDT support
**License:** CC BY 4.0

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Parser Implementation](#2-parser-implementation)
3. [Validator Implementation](#3-validator-implementation)
4. [Code Generator Implementation](#4-code-generator-implementation)
5. [Runtime Library Requirements](#5-runtime-library-requirements)
6. [Interoperability](#6-interoperability)
7. [Testing and Conformance](#7-testing-and-conformance)
8. [Best Practices](#8-best-practices)

---

## 1. Introduction

### 1.1 Purpose

This guide helps developers implement DOL CRDT schema support in their libraries and tools. Whether you're building:
- A new CRDT library
- A code generator for a specific language
- A schema validator
- A data sync protocol

This guide provides the necessary information for interoperability with DOL's CRDT annotation format.

### 1.2 Prerequisites

**Required Knowledge:**
- CRDT theory (Strong Eventual Consistency, merge semantics)
- Parsing and validation techniques
- Your target language's type system

**Recommended Reading:**
- [DOL CRDT Schema Specification](./dol-crdt-schema-v1.0.md)
- [RFC-001: DOL CRDT Annotations](../rfcs/RFC-001-dol-crdt-annotations.md)
- Shapiro et al. (2011): "Conflict-free Replicated Data Types"

### 1.3 Implementation Levels

**Level 1 (Basic):**
- Parse CRDT annotations
- Validate type compatibility
- Support 4+ strategies (immutable, lww, or_set, pn_counter)

**Level 2 (Full):**
- Support all 7 strategies
- Implement constraint categorization
- Schema evolution support
- Pass conformance test suite

**Level 3 (Advanced):**
- Cross-library interoperability
- Custom strategy extensions
- Performance optimizations

---

## 2. Parser Implementation

### 2.1 Lexer Extensions

Add tokens for CRDT annotations:

```
TOKEN_AT         = '@'
TOKEN_CRDT       = 'crdt'
TOKEN_LPAREN     = '('
TOKEN_RPAREN     = ')'
TOKEN_COMMA      = ','
TOKEN_EQUALS     = '='
TOKEN_IDENTIFIER = [a-z_][a-z0-9_]*
TOKEN_STRING     = '"' ... '"'
TOKEN_NUMBER     = [0-9]+
TOKEN_BOOL       = 'true' | 'false'
```

### 2.2 Grammar Rules

EBNF grammar for CRDT annotations:

```ebnf
crdt_annotation = '@crdt', '(', crdt_strategy, [ ',', crdt_options ], ')' ;

crdt_strategy = 'immutable'
              | 'lww'
              | 'or_set'
              | 'pn_counter'
              | 'peritext'
              | 'rga'
              | 'mv_register' ;

crdt_options = option_pair, { ',', option_pair } ;
option_pair = identifier, '=', value ;
value = string_literal | number | boolean ;

annotated_field = [ crdt_annotation ], field_name, ':', type_spec ;
```

### 2.3 AST Representation

Recommended AST node structure:

```rust
// Rust example
struct Field {
    name: String,
    type_spec: TypeSpec,
    crdt_annotation: Option<CrdtAnnotation>,
    documentation: Option<String>,
}

struct CrdtAnnotation {
    strategy: CrdtStrategy,
    options: HashMap<String, AnnotationValue>,
    span: Span,  // For error reporting
}

enum CrdtStrategy {
    Immutable,
    Lww,
    OrSet,
    PnCounter,
    Peritext,
    Rga,
    MvRegister,
}

enum AnnotationValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}
```

```python
# Python example
from dataclasses import dataclass
from typing import Optional, Dict, Union
from enum import Enum

class CrdtStrategy(Enum):
    IMMUTABLE = "immutable"
    LWW = "lww"
    OR_SET = "or_set"
    PN_COUNTER = "pn_counter"
    PERITEXT = "peritext"
    RGA = "rga"
    MV_REGISTER = "mv_register"

AnnotationValue = Union[str, int, bool]

@dataclass
class CrdtAnnotation:
    strategy: CrdtStrategy
    options: Dict[str, AnnotationValue]
    span: Optional[Span] = None

@dataclass
class Field:
    name: str
    type_spec: TypeSpec
    crdt_annotation: Optional[CrdtAnnotation] = None
    documentation: Optional[str] = None
```

### 2.4 Parsing Algorithm

**Step 1: Recognize annotation prefix**
```python
def parse_field(tokens):
    crdt_annotation = None

    # Check for @crdt annotation
    if peek_token().type == TOKEN_AT:
        crdt_annotation = parse_crdt_annotation(tokens)

    # Parse field declaration
    field_name = expect(TOKEN_IDENTIFIER)
    expect(TOKEN_COLON)
    type_spec = parse_type_spec(tokens)

    return Field(field_name, type_spec, crdt_annotation)
```

**Step 2: Parse annotation**
```python
def parse_crdt_annotation(tokens):
    expect(TOKEN_AT)
    expect(TOKEN_CRDT)
    expect(TOKEN_LPAREN)

    # Parse strategy
    strategy_token = expect(TOKEN_IDENTIFIER)
    strategy = parse_strategy(strategy_token.value)

    # Parse options (if any)
    options = {}
    if peek_token().type == TOKEN_COMMA:
        consume(TOKEN_COMMA)
        options = parse_options(tokens)

    expect(TOKEN_RPAREN)

    return CrdtAnnotation(strategy, options)
```

**Step 3: Parse options**
```python
def parse_options(tokens):
    options = {}

    while True:
        # Parse key=value
        key = expect(TOKEN_IDENTIFIER).value
        expect(TOKEN_EQUALS)
        value = parse_value(tokens)

        options[key] = value

        # Check for more options
        if peek_token().type != TOKEN_COMMA:
            break
        consume(TOKEN_COMMA)

    return options

def parse_value(tokens):
    token = peek_token()

    if token.type == TOKEN_STRING:
        return consume(TOKEN_STRING).value
    elif token.type == TOKEN_NUMBER:
        return int(consume(TOKEN_NUMBER).value)
    elif token.type == TOKEN_BOOL:
        return consume(TOKEN_BOOL).value == "true"
    else:
        raise ParseError(f"Expected value, got {token.type}")
```

### 2.5 Error Handling

Provide clear error messages with source location:

```python
class CrdtParseError(Exception):
    def __init__(self, message, span):
        self.message = message
        self.span = span

    def format(self, source_code):
        line = source_code.lines[span.line]
        return f"""
Error: {self.message}
  --> {span.file}:{span.line}:{span.column}
   |
{span.line} | {line}
   | {' ' * span.column}^ invalid CRDT annotation
"""
```

**Example errors:**
```
Error: Unknown CRDT strategy 'last_write_wins'
  --> schema.dol:5:8
   |
5  | @crdt(last_write_wins)
   |       ^^^^^^^^^^^^^^^^ unknown strategy
   |
   = help: Valid strategies: immutable, lww, or_set, pn_counter, peritext, rga, mv_register
```

---

## 3. Validator Implementation

### 3.1 Validation Pipeline

```
Parser → AST
  ↓
Type Checking
  ↓
CRDT Validation
  ├─ Strategy-Type Compatibility (§3.2)
  ├─ Option Validation (§3.3)
  ├─ Constraint Compatibility (§3.4)
  └─ Evolution Safety (§3.5)
  ↓
Validated Schema
```

### 3.2 Strategy-Type Compatibility

Implement the compatibility matrix from the specification (§6):

```rust
fn validate_strategy_type_compatibility(
    field: &Field,
    strategy: CrdtStrategy,
    type_spec: &TypeSpec,
) -> Result<(), ValidationError> {
    match (strategy, type_spec) {
        // Immutable: compatible with any type
        (CrdtStrategy::Immutable, _) => Ok(()),

        // LWW: compatible with scalars and structs
        (CrdtStrategy::Lww, TypeSpec::Scalar(_)) => Ok(()),
        (CrdtStrategy::Lww, TypeSpec::Struct(_)) => Ok(()),
        (CrdtStrategy::Lww, TypeSpec::Collection(_)) => {
            Err(ValidationError::TypeStrategyMismatch {
                field: field.name.clone(),
                strategy: "lww",
                type_name: format!("{:?}", type_spec),
                hint: "LWW is not compatible with collections. Use or_set for Set<T> or rga for List<T>",
            })
        }

        // OR-Set: only compatible with Set<T>
        (CrdtStrategy::OrSet, TypeSpec::Set(_)) => Ok(()),
        (CrdtStrategy::OrSet, _) => {
            Err(ValidationError::TypeStrategyMismatch {
                field: field.name.clone(),
                strategy: "or_set",
                type_name: format!("{:?}", type_spec),
                hint: "or_set requires Set<T> type",
            })
        }

        // PN-Counter: compatible with integers
        (CrdtStrategy::PnCounter, TypeSpec::Int | TypeSpec::UInt) => Ok(()),
        (CrdtStrategy::PnCounter, TypeSpec::Float) => {
            // Warning: works but has precision issues
            Ok(())  // Could emit warning
        }
        (CrdtStrategy::PnCounter, _) => {
            Err(ValidationError::TypeStrategyMismatch {
                field: field.name.clone(),
                strategy: "pn_counter",
                type_name: format!("{:?}", type_spec),
                hint: "pn_counter requires Int or UInt type",
            })
        }

        // Peritext: String or RichText
        (CrdtStrategy::Peritext, TypeSpec::String | TypeSpec::RichText) => Ok(()),
        (CrdtStrategy::Peritext, _) => {
            Err(ValidationError::TypeStrategyMismatch {
                field: field.name.clone(),
                strategy: "peritext",
                type_name: format!("{:?}", type_spec),
                hint: "peritext requires String or RichText type",
            })
        }

        // RGA: List, Vec, Array
        (CrdtStrategy::Rga, TypeSpec::List(_) | TypeSpec::Vec(_)) => Ok(()),
        (CrdtStrategy::Rga, _) => {
            Err(ValidationError::TypeStrategyMismatch {
                field: field.name.clone(),
                strategy: "rga",
                type_name: format!("{:?}", type_spec),
                hint: "rga requires List<T> or Vec<T> type",
            })
        }

        // MV-Register: compatible with any type
        (CrdtStrategy::MvRegister, _) => Ok(()),
    }
}
```

### 3.3 Option Validation

Validate strategy-specific options:

```rust
fn validate_options(
    strategy: CrdtStrategy,
    options: &HashMap<String, AnnotationValue>,
) -> Result<(), ValidationError> {
    match strategy {
        CrdtStrategy::Lww => {
            // Valid: tie_break
            if let Some(tie_break) = options.get("tie_break") {
                match tie_break {
                    AnnotationValue::String(s) if s == "actor_id" => Ok(()),
                    AnnotationValue::String(s) if s == "content_hash" => Ok(()),
                    AnnotationValue::String(s) if s == "custom" => Ok(()),
                    _ => Err(ValidationError::InvalidOption {
                        option: "tie_break",
                        value: format!("{:?}", tie_break),
                        valid_values: vec!["actor_id", "content_hash", "custom"],
                    }),
                }
            } else {
                Ok(())
            }
        }

        CrdtStrategy::PnCounter => {
            // Valid: min_value, max_value, overflow_strategy
            if let Some(min) = options.get("min_value") {
                if !matches!(min, AnnotationValue::Integer(_)) {
                    return Err(ValidationError::InvalidOptionType {
                        option: "min_value",
                        expected: "integer",
                        got: format!("{:?}", min),
                    });
                }
            }

            if let Some(max) = options.get("max_value") {
                if !matches!(max, AnnotationValue::Integer(_)) {
                    return Err(ValidationError::InvalidOptionType {
                        option: "max_value",
                        expected: "integer",
                        got: format!("{:?}", max),
                    });
                }
            }

            // Check min < max
            if let (Some(AnnotationValue::Integer(min)), Some(AnnotationValue::Integer(max))) =
                (options.get("min_value"), options.get("max_value"))
            {
                if min >= max {
                    return Err(ValidationError::InvalidConstraint {
                        message: "min_value must be < max_value",
                    });
                }
            }

            Ok(())
        }

        CrdtStrategy::Peritext => {
            // Valid: formatting, max_length
            if let Some(fmt) = options.get("formatting") {
                match fmt {
                    AnnotationValue::String(s) if s == "full" => Ok(()),
                    AnnotationValue::String(s) if s == "markdown" => Ok(()),
                    AnnotationValue::String(s) if s == "plain" => Ok(()),
                    _ => Err(ValidationError::InvalidOption {
                        option: "formatting",
                        value: format!("{:?}", fmt),
                        valid_values: vec!["full", "markdown", "plain"],
                    }),
                }
            } else {
                Ok(())
            }
        }

        // Strategies with no options
        CrdtStrategy::Immutable |
        CrdtStrategy::OrSet |
        CrdtStrategy::Rga |
        CrdtStrategy::MvRegister => {
            if !options.is_empty() {
                Err(ValidationError::UnexpectedOptions {
                    strategy: format!("{:?}", strategy),
                    options: options.keys().cloned().collect(),
                })
            } else {
                Ok(())
            }
        }
    }
}
```

### 3.4 Constraint Compatibility

Categorize constraints and detect conflicts:

```rust
enum ConstraintCategory {
    CrdtSafe,             // Category A: enforced by CRDT
    EventuallyConsistent, // Category B: eventually consistent
    StrongConsistency,    // Category C: requires coordination
}

fn categorize_constraint(constraint: &Constraint, field: &Field) -> ConstraintCategory {
    // Category A: Immutability constraints
    if constraint.is_immutability() && field.crdt_strategy == Some(CrdtStrategy::Immutable) {
        return ConstraintCategory::CrdtSafe;
    }

    // Category A: Monotonicity constraints
    if constraint.is_monotonic() && field.crdt_strategy == Some(CrdtStrategy::PnCounter) {
        return ConstraintCategory::CrdtSafe;
    }

    // Category C: Cross-entity atomic constraints
    if constraint.is_cross_entity_atomic() {
        return ConstraintCategory::StrongConsistency;
    }

    // Category B: Everything else
    ConstraintCategory::EventuallyConsistent
}

fn validate_constraint_compatibility(schema: &Schema) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    for constraint in &schema.constraints {
        let category = categorize_constraint(constraint, &schema.fields);

        match category {
            ConstraintCategory::CrdtSafe => {
                // No action needed, enforced by CRDT
            }
            ConstraintCategory::EventuallyConsistent => {
                warnings.push(ValidationWarning::EventuallyConsistentConstraint {
                    constraint: constraint.name.clone(),
                    hint: "This constraint may be temporarily violated during concurrent operations",
                });
            }
            ConstraintCategory::StrongConsistency => {
                warnings.push(ValidationWarning::StrongConsistencyRequired {
                    constraint: constraint.name.clone(),
                    hint: "This constraint requires coordination (BFT consensus, escrow, or locking)",
                });
            }
        }
    }

    warnings
}
```

### 3.5 Evolution Safety

Validate schema evolution and migrations:

```rust
fn validate_evolution(evolution: &Evolution) -> Result<(), ValidationError> {
    for change in &evolution.changes {
        let safety = check_migration_safety(&change.from_strategy, &change.to_strategy);

        match safety {
            MigrationSafety::Safe => {
                // No action needed
            }
            MigrationSafety::Unsafe { reason } => {
                // Require migration function
                if !evolution.has_migration_fn(&change.field) {
                    return Err(ValidationError::MissingMigrationFn {
                        field: change.field.clone(),
                        from: format!("{:?}", change.from_strategy),
                        to: format!("{:?}", change.to_strategy),
                        reason,
                    });
                }

                // Validate migration determinism
                validate_migration_determinism(evolution, &change.field)?;
            }
        }
    }

    Ok(())
}

enum MigrationSafety {
    Safe,
    Unsafe { reason: String },
}

fn check_migration_safety(from: &CrdtStrategy, to: &CrdtStrategy) -> MigrationSafety {
    match (from, to) {
        // Safe migrations
        (CrdtStrategy::Immutable, CrdtStrategy::Lww) => MigrationSafety::Safe,
        (CrdtStrategy::Lww, CrdtStrategy::MvRegister) => MigrationSafety::Safe,
        (CrdtStrategy::OrSet, CrdtStrategy::Rga) => MigrationSafety::Safe,

        // Unsafe migrations
        (CrdtStrategy::Lww, CrdtStrategy::Immutable) => {
            MigrationSafety::Unsafe {
                reason: "Cannot make mutable field immutable".to_string(),
            }
        }
        (CrdtStrategy::Peritext, CrdtStrategy::Lww) => {
            MigrationSafety::Unsafe {
                reason: "Loses rich text structure".to_string(),
            }
        }
        (CrdtStrategy::OrSet, CrdtStrategy::PnCounter) => {
            MigrationSafety::Unsafe {
                reason: "Semantic mismatch (set ≠ counter)".to_string(),
            }
        }

        // Default: require manual verification
        _ => MigrationSafety::Unsafe {
            reason: "Migration requires manual verification".to_string(),
        },
    }
}
```

---

## 4. Code Generator Implementation

### 4.1 Target Language Patterns

#### 4.1.1 Rust/Automerge

```rust
// Generated from:
// @crdt(lww)
// name: string

use automerge::{Automerge, AutoCommit, ReadDoc};
use autosurgeon::{Reconcile, Hydrate};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Entity {
    #[autosurgeon(lww)]
    pub name: String,
}

impl Entity {
    pub fn set_name(&mut self, name: String, doc: &mut AutoCommit) {
        self.name = name;
        autosurgeon::reconcile(doc, &self).expect("reconcile failed");
    }
}
```

#### 4.1.2 TypeScript/Automerge

```typescript
// Generated from DOL schema
import * as Automerge from '@automerge/automerge';

export interface Entity {
  name: string;  // @crdt(lww)
  tags: string[]; // @crdt(or_set)
}

export class EntityCRDT {
  private doc: Automerge.Doc<Entity>;

  constructor(initialData: Entity) {
    this.doc = Automerge.from(initialData);
  }

  setName(name: string): void {
    this.doc = Automerge.change(this.doc, doc => {
      doc.name = name;
    });
  }

  addTag(tag: string): void {
    this.doc = Automerge.change(this.doc, doc => {
      if (!doc.tags) doc.tags = [];
      doc.tags.push(tag);
    });
  }

  merge(other: Automerge.Doc<Entity>): void {
    this.doc = Automerge.merge(this.doc, other);
  }

  save(): Uint8Array {
    return Automerge.save(this.doc);
  }

  static load(data: Uint8Array): EntityCRDT {
    const doc = Automerge.load<Entity>(data);
    const instance = new EntityCRDT({} as Entity);
    instance.doc = doc;
    return instance;
  }
}
```

#### 4.1.3 Python/Automerge

```python
# Generated from DOL schema
from automerge import Automerge
from typing import List, Set
from dataclasses import dataclass

@dataclass
class Entity:
    name: str  # @crdt(lww)
    tags: Set[str]  # @crdt(or_set)

class EntityCRDT:
    def __init__(self, initial_data: Entity):
        self.doc = Automerge.from_dict(initial_data.__dict__)

    def set_name(self, name: str):
        with self.doc.transaction() as tx:
            tx["name"] = name

    def add_tag(self, tag: str):
        with self.doc.transaction() as tx:
            if "tags" not in tx:
                tx["tags"] = []
            tx["tags"].append(tag)

    def merge(self, other: Automerge):
        self.doc = self.doc.merge(other)

    def save(self) -> bytes:
        return self.doc.save()

    @staticmethod
    def load(data: bytes) -> 'EntityCRDT':
        doc = Automerge.load(data)
        instance = EntityCRDT(Entity(name="", tags=set()))
        instance.doc = doc
        return instance
```

### 4.2 Strategy-Specific Code Generation

Implement code generation for each strategy:

```rust
fn generate_field_code(field: &Field, target_lang: &Language) -> String {
    match (&field.crdt_annotation, target_lang) {
        (Some(ann), Language::Rust) => {
            generate_rust_field(field, ann)
        }
        (Some(ann), Language::TypeScript) => {
            generate_typescript_field(field, ann)
        }
        (None, _) => {
            // No CRDT annotation, generate plain field
            generate_plain_field(field, target_lang)
        }
    }
}

fn generate_rust_field(field: &Field, ann: &CrdtAnnotation) -> String {
    let attr = match ann.strategy {
        CrdtStrategy::Immutable => "#[autosurgeon(immutable)]",
        CrdtStrategy::Lww => "#[autosurgeon(lww)]",
        CrdtStrategy::OrSet => "#[autosurgeon(set)]",
        CrdtStrategy::PnCounter => "#[autosurgeon(counter)]",
        CrdtStrategy::Peritext => "#[autosurgeon(text)]",
        CrdtStrategy::Rga => "#[autosurgeon(list)]",
        CrdtStrategy::MvRegister => "#[autosurgeon(mv_register)]",
    };

    format!("{}\npub {}: {},", attr, field.name, field.type_spec)
}
```

### 4.3 Operation Method Generation

Generate methods for CRDT operations:

```rust
fn generate_operations(schema: &Schema, target_lang: &Language) -> String {
    let mut methods = Vec::new();

    for field in &schema.fields {
        if let Some(ann) = &field.crdt_annotation {
            methods.push(generate_operation_method(field, ann, target_lang));
        }
    }

    methods.join("\n\n")
}

fn generate_operation_method(
    field: &Field,
    ann: &CrdtAnnotation,
    lang: &Language,
) -> String {
    match (ann.strategy, lang) {
        (CrdtStrategy::Lww, Language::Rust) => {
            format!(r#"
pub fn set_{field_name}(&mut self, value: {field_type}, doc: &mut AutoCommit) {{
    self.{field_name} = value;
    autosurgeon::reconcile(doc, &self).expect("reconcile failed");
}}
"#, field_name = field.name, field_type = field.type_spec)
        }

        (CrdtStrategy::OrSet, Language::Rust) => {
            format!(r#"
pub fn add_{field_name}(&mut self, item: {element_type}, doc: &mut AutoCommit) {{
    self.{field_name}.insert(item);
    autosurgeon::reconcile(doc, &self).expect("reconcile failed");
}}

pub fn remove_{field_name}(&mut self, item: &{element_type}, doc: &mut AutoCommit) {{
    self.{field_name}.remove(item);
    autosurgeon::reconcile(doc, &self).expect("reconcile failed");
}}
"#,
                field_name = field.name,
                element_type = extract_element_type(&field.type_spec)
            )
        }

        (CrdtStrategy::PnCounter, Language::Rust) => {
            format!(r#"
pub fn increment_{field_name}(&mut self, amount: {field_type}, doc: &mut AutoCommit) {{
    self.{field_name} += amount;
    autosurgeon::reconcile(doc, &self).expect("reconcile failed");
}}

pub fn decrement_{field_name}(&mut self, amount: {field_type}, doc: &mut AutoCommit) {{
    self.{field_name} -= amount;
    autosurgeon::reconcile(doc, &self).expect("reconcile failed");
}}
"#, field_name = field.name, field_type = field.type_spec)
        }

        // ... other strategies and languages
        _ => String::new(),
    }
}
```

---

## 5. Runtime Library Requirements

### 5.1 Core CRDT Operations

Implement merge semantics for each strategy:

```rust
pub trait CrdtMerge {
    fn merge(&mut self, other: &Self);
}

// Immutable
impl CrdtMerge for ImmutableRegister<T> {
    fn merge(&mut self, other: &Self) {
        // Keep value with earliest timestamp
        if other.timestamp < self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        } else if other.timestamp == self.timestamp {
            // Tie-break by actor_id
            if other.actor_id < self.actor_id {
                self.value = other.value.clone();
                self.actor_id = other.actor_id;
            }
        }
    }
}

// LWW
impl CrdtMerge for LwwRegister<T> {
    fn merge(&mut self, other: &Self) {
        // Keep value with latest timestamp
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.actor_id = other.actor_id;
        } else if other.timestamp == self.timestamp {
            // Tie-break by actor_id
            if other.actor_id > self.actor_id {
                self.value = other.value.clone();
                self.actor_id = other.actor_id;
            }
        }
    }
}

// OR-Set
impl<T: Hash + Eq + Clone> CrdtMerge for ORSet<T> {
    fn merge(&mut self, other: &Self) {
        // Union all elements and tombstones
        for (element, tags) in &other.elements {
            self.elements.entry(element.clone())
                .or_insert_with(HashSet::new)
                .extend(tags);
        }
        self.tombstones.extend(&other.tombstones);
    }
}

// PN-Counter
impl CrdtMerge for PNCounter {
    fn merge(&mut self, other: &Self) {
        // Take max per actor
        for (actor, count) in &other.increments {
            let current = self.increments.entry(*actor).or_insert(0);
            *current = (*current).max(*count);
        }
        for (actor, count) in &other.decrements {
            let current = self.decrements.entry(*actor).or_insert(0);
            *current = (*current).max(*count);
        }
    }
}
```

### 5.2 Serialization

Implement serialization for sync:

```rust
pub trait CrdtSerialize {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<Self, Error> where Self: Sized;
}

// Example: LWW Register
impl<T: Serialize + DeserializeOwned> CrdtSerialize for LwwRegister<T> {
    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&(
            &self.value,
            &self.timestamp,
            &self.actor_id,
        )).expect("serialization failed")
    }

    fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let (value, timestamp, actor_id) = bincode::deserialize(data)?;
        Ok(LwwRegister { value, timestamp, actor_id })
    }
}
```

### 5.3 Delta Compression

Optimize sync with delta compression:

```rust
pub trait CrdtDelta {
    type Delta;

    fn delta_since(&self, version: Version) -> Self::Delta;
    fn apply_delta(&mut self, delta: Self::Delta);
}

// Example: OR-Set delta
impl<T: Hash + Eq + Clone> CrdtDelta for ORSet<T> {
    type Delta = ORSetDelta<T>;

    fn delta_since(&self, version: Version) -> Self::Delta {
        // Return only elements/tombstones added since version
        ORSetDelta {
            added_elements: self.elements.iter()
                .filter(|(_, tags)| tags.iter().any(|t| t.version > version))
                .cloned()
                .collect(),
            added_tombstones: self.tombstones.iter()
                .filter(|t| t.version > version)
                .cloned()
                .collect(),
        }
    }

    fn apply_delta(&mut self, delta: Self::Delta) {
        // Apply delta (same as merge but only for delta)
        for (element, tags) in delta.added_elements {
            self.elements.entry(element)
                .or_insert_with(HashSet::new)
                .extend(tags);
        }
        self.tombstones.extend(delta.added_tombstones);
    }
}
```

---

## 6. Interoperability

### 6.1 JSON Schema Export

Support exporting to JSON schema format:

```rust
pub fn export_to_json_schema(schema: &Schema) -> serde_json::Value {
    json!({
        "name": schema.name,
        "version": schema.version.to_string(),
        "fields": schema.fields.iter().map(|field| {
            json!({
                "name": field.name,
                "type": field.type_spec.to_string(),
                "crdt": field.crdt_annotation.as_ref().map(|ann| {
                    json!({
                        "strategy": format!("{:?}", ann.strategy).to_lowercase(),
                        "options": ann.options,
                    })
                }),
            })
        }).collect::<Vec<_>>(),
    })
}
```

### 6.2 Cross-Library Compatibility

Document how to convert between CRDT libraries:

```rust
// Example: Automerge <-> Yjs conversion
pub trait CrdtConvert<T> {
    fn from_automerge(doc: &Automerge) -> T;
    fn to_automerge(&self) -> Automerge;
    fn from_yjs(doc: &YDoc) -> T;
    fn to_yjs(&self) -> YDoc;
}

impl CrdtConvert<MySchema> for MySchema {
    fn from_automerge(doc: &Automerge) -> MySchema {
        // Extract fields from Automerge document
        MySchema {
            name: doc.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tags: doc.get("tags").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                .unwrap_or_default(),
        }
    }

    fn to_automerge(&self) -> Automerge {
        let mut doc = Automerge::new();
        doc.transact(|tx| {
            tx.put("name", &self.name);
            tx.put("tags", &self.tags);
        });
        doc
    }

    // Similar for Yjs
    fn from_yjs(doc: &YDoc) -> MySchema {
        // ...
    }

    fn to_yjs(&self) -> YDoc {
        // ...
    }
}
```

---

## 7. Testing and Conformance

### 7.1 Property-Based Tests

Verify CRDT properties:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lww_commutativity(
        value1: String,
        value2: String,
        ts1: u64,
        ts2: u64,
    ) {
        let mut reg1 = LwwRegister::new(value1.clone(), ts1, ActorId::from("A"));
        let mut reg2 = LwwRegister::new(value2.clone(), ts2, ActorId::from("B"));

        // Merge in both orders
        let mut result1 = reg1.clone();
        result1.merge(&reg2);

        let mut result2 = reg2.clone();
        result2.merge(&reg1);

        // Should be identical (commutativity)
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_or_set_idempotence(
        elements: Vec<String>,
    ) {
        let mut set = ORSet::new();
        for elem in elements {
            set.add(elem);
        }

        // Merge with self
        let original = set.clone();
        set.merge(&original);

        // Should be identical (idempotence)
        assert_eq!(set, original);
    }
}
```

### 7.2 Convergence Tests

Test that replicas converge:

```rust
#[test]
fn test_convergence() {
    let mut replica_a = MySchema::new();
    let mut replica_b = replica_a.fork();
    let mut replica_c = replica_a.fork();

    // Concurrent operations
    replica_a.set_name("Alice");
    replica_b.set_name("Bob");
    replica_c.add_tag("urgent");

    // Merge in different orders
    replica_a.merge(&replica_b);
    replica_a.merge(&replica_c);

    replica_b.merge(&replica_c);
    replica_b.merge(&replica_a);

    replica_c.merge(&replica_a);
    replica_c.merge(&replica_b);

    // All should converge to same state
    assert_eq!(replica_a, replica_b);
    assert_eq!(replica_b, replica_c);
}
```

### 7.3 Conformance Test Suite

Run the official DOL CRDT test suite:

```bash
# Download test suite
git clone https://github.com/univrs/dol-crdt-test-suite

# Run tests
cd dol-crdt-test-suite
./run-tests.sh --impl-path /path/to/your/implementation
```

---

## 8. Best Practices

### 8.1 Error Messages

Provide clear, actionable error messages:

```
✗ Good:
Error: Invalid CRDT strategy
  --> schema.dol:5:8
   |
5  | @crdt(last_write_wins)
   |       ^^^^^^^^^^^^^^^^ unknown strategy
   |
   = help: Valid strategies: immutable, lww, or_set, pn_counter, peritext, rga, mv_register
   = note: Did you mean 'lww'?

✗ Bad:
Error: Parse error at line 5
```

### 8.2 Performance

Optimize for common cases:

1. **Use delta compression** for sync (only send changes)
2. **Garbage collect tombstones** periodically
3. **Batch operations** to reduce merge overhead
4. **Cache merged state** to avoid redundant computation

### 8.3 Documentation

Generate documentation from DOL schemas:

```rust
pub fn generate_docs(schema: &Schema) -> String {
    let mut docs = String::new();

    docs.push_str(&format!("# {}\n\n", schema.name));
    docs.push_str(&format!("Version: {}\n\n", schema.version));

    if let Some(doc) = &schema.documentation {
        docs.push_str(&format!("{}\n\n", doc));
    }

    docs.push_str("## Fields\n\n");
    for field in &schema.fields {
        docs.push_str(&format!("### `{}`\n\n", field.name));
        docs.push_str(&format!("Type: `{}`\n\n", field.type_spec));

        if let Some(ann) = &field.crdt_annotation {
            docs.push_str(&format!("CRDT Strategy: `{:?}`\n\n", ann.strategy));
            if !ann.options.is_empty() {
                docs.push_str("Options:\n");
                for (key, value) in &ann.options {
                    docs.push_str(&format!("- `{}`: `{:?}`\n", key, value));
                }
                docs.push_str("\n");
            }
        }

        if let Some(doc) = &field.documentation {
            docs.push_str(&format!("{}\n\n", doc));
        }
    }

    docs
}
```

### 8.4 Versioning

Support schema versioning and evolution:

```rust
pub struct SchemaRegistry {
    schemas: HashMap<(String, Version), Schema>,
}

impl SchemaRegistry {
    pub fn register(&mut self, schema: Schema) {
        let key = (schema.name.clone(), schema.version.clone());
        self.schemas.insert(key, schema);
    }

    pub fn get(&self, name: &str, version: &Version) -> Option<&Schema> {
        self.schemas.get(&(name.to_string(), version.clone()))
    }

    pub fn get_latest(&self, name: &str) -> Option<&Schema> {
        self.schemas.iter()
            .filter(|((n, _), _)| n == name)
            .max_by_key(|((_, v), _)| v)
            .map(|(_, schema)| schema)
    }

    pub fn migrate(&self, data: &[u8], from: &Version, to: &Version) -> Result<Vec<u8>, Error> {
        // Implement migration logic
        todo!()
    }
}
```

---

## 9. Resources

### 9.1 Reference Implementations

- **DOL Rust**: https://github.com/univrs/dol (canonical implementation)
- **DOL TypeScript**: https://github.com/univrs/dol-ts
- **DOL Python**: https://github.com/univrs/dol-py

### 9.2 CRDT Libraries

- **Automerge**: https://automerge.org
- **Yjs**: https://github.com/yjs/yjs
- **Loro**: https://loro.dev
- **Diamond Types**: https://github.com/josephg/diamond-types

### 9.3 Specifications

- [DOL CRDT Schema Specification](./dol-crdt-schema-v1.0.md)
- [RFC-001: DOL CRDT Annotations](../rfcs/RFC-001-dol-crdt-annotations.md)
- [JSON Schema](../schemas/dol-crdt.json)

### 9.4 Community

- **Forum**: https://forum.univrs.org
- **Discord**: https://discord.gg/univrs
- **GitHub**: https://github.com/univrs/dol

---

## 10. Contact

For questions, feedback, or implementation support:
- Email: standards@univrs.org
- Issues: https://github.com/univrs/dol/issues
- Discussions: https://github.com/univrs/dol/discussions

---

**Document Version:** 1.0.0
**Published:** 2026-02-05
**License:** Creative Commons Attribution 4.0 International (CC BY 4.0)
**Copyright:** Univrs Foundation, 2026

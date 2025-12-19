# Metal DOL Remediation Plan

**Project**: univrs/metadol  
**Status**: Early-stage prototype requiring systematic improvement  
**Goal**: Transform from aspirational concept to production-ready DSL toolchain

---

## Executive Summary

The critical analysis identified 10 major areas requiring remediation. This plan provides specific, actionable steps to address each critique, transforming Metadol from an early prototype into a mature, usable language and toolchain that fulfills its ambitious vision of ontology-first development.

---

## Critique 1: Lack of Tests / Examples

**Issue**: No Rust tests, example usage, CI config, or demos visible. `cargo test` would fail with nothing to execute.

### Remediation Actions

#### 1.1 Create Unit Tests for Lexer (`tests/lexer_tests.rs`)

```rust
//! Lexer unit tests for Metal DOL tokenization

use metadol::lexer::{Lexer, Token, TokenKind};

#[test]
fn test_lexer_keywords() {
    let input = "gene trait constraint system evolves";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().kind, TokenKind::Gene);
    assert_eq!(lexer.next_token().kind, TokenKind::Trait);
    assert_eq!(lexer.next_token().kind, TokenKind::Constraint);
    assert_eq!(lexer.next_token().kind, TokenKind::System);
    assert_eq!(lexer.next_token().kind, TokenKind::Evolves);
}

#[test]
fn test_lexer_identifiers() {
    let input = "container.exists identity.cryptographic";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Identifier);
    assert_eq!(token.lexeme, "container.exists");
}

#[test]
fn test_lexer_operators() {
    let input = "has is requires uses derives from";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().kind, TokenKind::Has);
    assert_eq!(lexer.next_token().kind, TokenKind::Is);
    // ... continue for all operators
}

#[test]
fn test_lexer_braces_and_delimiters() {
    let input = "{ } @ > >= >";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
    assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    // ... continue
}

#[test]
fn test_lexer_version_numbers() {
    let input = "@ 0.0.1 > 0.0.2";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().kind, TokenKind::At);
    let version = lexer.next_token();
    assert_eq!(version.kind, TokenKind::Version);
    assert_eq!(version.lexeme, "0.0.1");
}

#[test]
fn test_lexer_complete_gene() {
    let input = r#"
gene container.exists {
  container has identity
  container has state
}
"#;
    let lexer = Lexer::new(input);
    let tokens: Vec<Token> = lexer.collect();
    
    assert!(tokens.len() > 0);
    assert_eq!(tokens[0].kind, TokenKind::Gene);
}
```

#### 1.2 Create Parser Tests (`tests/parser_tests.rs`)

```rust
//! Parser unit tests for Metal DOL AST generation

use metadol::parser::Parser;
use metadol::ast::{Declaration, Statement};

#[test]
fn test_parse_gene() {
    let input = r#"
gene container.exists {
  container has identity
  container has state
  container has boundaries
}

exegesis {
  A container is the fundamental unit of workload isolation.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    assert!(result.is_ok());
    let decl = result.unwrap();
    
    match decl {
        Declaration::Gene { name, statements, exegesis } => {
            assert_eq!(name, "container.exists");
            assert_eq!(statements.len(), 3);
            assert!(exegesis.contains("fundamental unit"));
        }
        _ => panic!("Expected Gene declaration"),
    }
}

#[test]
fn test_parse_trait_with_uses() {
    let input = r#"
trait container.lifecycle {
  uses container.exists
  uses identity.cryptographic
  
  container is created
  container is started
  container is stopped
}

exegesis {
  The container lifecycle defines state transitions.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    assert!(result.is_ok());
    // Validate trait structure
}

#[test]
fn test_parse_constraint() {
    let input = r#"
constraint container.integrity {
  container state matches declared state
  container identity never changes
}

exegesis {
  Ensures runtime matches declared ontology.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    assert!(result.is_ok());
}

#[test]
fn test_parse_evolution() {
    let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed
  
  because "workload migration requires state preservation"
}

exegesis {
  Version 0.0.2 adds pause/resume capabilities.
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    assert!(result.is_ok());
}

#[test]
fn test_parser_error_recovery() {
    let input = "gene missing.braces container has state";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("expected"));
}
```

#### 1.3 Create Integration Tests (`tests/integration_tests.rs`)

```rust
//! End-to-end integration tests

use std::fs;
use metadol::{parse_file, validate_dol};

#[test]
fn test_parse_example_files() {
    let examples = [
        "examples/genes/container.exists.dol",
        "examples/traits/container.lifecycle.dol",
        "examples/constraints/container.integrity.dol",
    ];
    
    for path in examples {
        let content = fs::read_to_string(path)
            .expect(&format!("Failed to read {}", path));
        
        let result = parse_file(&content);
        assert!(result.is_ok(), "Failed to parse {}: {:?}", path, result.err());
    }
}

#[test]
fn test_dol_validation() {
    let content = fs::read_to_string("examples/genes/container.exists.dol")
        .expect("Failed to read example");
    
    let ast = parse_file(&content).unwrap();
    let validation = validate_dol(&ast);
    
    assert!(validation.is_valid());
    assert!(validation.has_exegesis());
}
```

---

## Critique 2: No Documentation for Build/Usage

**Issue**: No build instructions, usage examples, or how to run the parser.

### Remediation Actions

#### 2.1 Enhanced README.md Structure

Add the following sections to README.md:

```markdown
## Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/univrs/metadol.git
cd metadol

# Build the project
cargo build --release

# Run tests
cargo test

# Install CLI tools
cargo install --path .
```

### Basic Usage

#### Parse a DOL file

```bash
# Parse and validate a DOL file
dol-parse examples/genes/container.exists.dol

# Output as JSON AST
dol-parse examples/genes/container.exists.dol --format json

# Validate all files in a directory
dol-parse examples/ --validate --recursive
```

#### Library Usage

```rust
use metadol::{parse_file, Lexer, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
gene container.exists {
  container has identity
  container has state
}

exegesis {
  A container is the fundamental unit.
}
"#;
    
    // Parse to AST
    let ast = parse_file(input)?;
    
    // Access parsed elements
    println!("Parsed: {:?}", ast.name());
    
    Ok(())
}
```

## Development

### Project Structure

```
metadol/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── lexer.rs         # Tokenizer for DOL syntax
│   ├── parser.rs        # Recursive descent parser
│   ├── ast.rs           # Abstract syntax tree definitions
│   ├── validator.rs     # Semantic validation
│   └── bin/
│       ├── dol-parse.rs # Parser CLI
│       ├── dol-test.rs  # Test generator CLI
│       └── dol-check.rs # Validation CLI
├── tests/
│   ├── lexer_tests.rs   # Lexer unit tests
│   ├── parser_tests.rs  # Parser unit tests
│   └── integration_tests.rs
├── examples/
│   ├── genes/           # Example gene definitions
│   ├── traits/          # Example trait definitions
│   ├── constraints/     # Example constraint definitions
│   └── systems/         # Example system definitions
└── docs/
    ├── grammar.ebnf     # Formal grammar specification
    └── tutorials/       # Step-by-step guides
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test module
cargo test lexer_tests

# Run tests matching pattern
cargo test parse_gene
```

### Building Documentation

```bash
# Generate API documentation
cargo doc --open

# Generate with private items
cargo doc --document-private-items --open
```
```

---

## Critique 3: Lack of Documentation Comments

**Issue**: None of the source code contains inline doc comments (`///`) explaining structs, enums, or functions.

### Remediation Actions

#### 3.1 Add Comprehensive Doc Comments to `ast.rs`

```rust
//! Abstract Syntax Tree definitions for Metal DOL.
//!
//! This module defines the complete AST representation for parsed DOL files,
//! including genes, traits, constraints, systems, and evolution declarations.
//!
//! # Example
//!
//! ```rust
//! use metadol::ast::{Declaration, Gene, Statement};
//!
//! let gene = Gene {
//!     name: "container.exists".to_string(),
//!     statements: vec![
//!         Statement::Has { subject: "container", property: "identity" },
//!     ],
//!     exegesis: "A container is the fundamental unit.".to_string(),
//! };
//! ```

/// The top-level declaration types in Metal DOL.
///
/// Every DOL file contains exactly one primary declaration followed by
/// an exegesis section. This enum represents all possible declaration types.
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    /// A gene declaration - the atomic unit of DOL.
    ///
    /// Genes declare fundamental truths that cannot be decomposed further.
    /// They are named with dot notation: `domain.property`.
    Gene(Gene),
    
    /// A trait declaration - composable behaviors built from genes.
    ///
    /// Traits declare what a component does, building on genes via `uses`.
    Trait(Trait),
    
    /// A constraint declaration - invariants that must always hold.
    ///
    /// Constraints define the laws of the system using predicates like
    /// `matches` and `never`.
    Constraint(Constraint),
    
    /// A system declaration - top-level composition of a complete subsystem.
    ///
    /// Systems combine genes, traits, and constraints with version requirements.
    System(System),
    
    /// An evolution declaration - lineage record of ontology changes.
    ///
    /// Evolutions use the `>` operator to denote version lineage.
    Evolution(Evolution),
}

/// A gene declaration representing atomic ontological truths.
///
/// # Example DOL Syntax
///
/// ```dol
/// gene container.exists {
///   container has identity
///   container has state
///   container has boundaries
/// }
///
/// exegesis {
///   A container is the fundamental unit of workload isolation.
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Gene {
    /// The fully qualified name using dot notation (e.g., "container.exists")
    pub name: String,
    
    /// The declarative statements within the gene body
    pub statements: Vec<Statement>,
    
    /// The mandatory exegesis explaining intent and context
    pub exegesis: String,
    
    /// Source location for error reporting
    pub span: Span,
}

/// A statement within a DOL declaration.
///
/// Statements use simple predicates to declare relationships and properties.
/// The predicate determines the semantic meaning of the statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Property possession: `subject has property`
    ///
    /// # Example
    /// ```dol
    /// container has identity
    /// ```
    Has {
        subject: String,
        property: String,
        span: Span,
    },
    
    /// State or behavior: `subject is state`
    ///
    /// # Example
    /// ```dol
    /// container is created
    /// ```
    Is {
        subject: String,
        state: String,
        span: Span,
    },
    
    /// Origin relationship: `subject derives from origin`
    ///
    /// # Example
    /// ```dol
    /// identity derives from ed25519 keypair
    /// ```
    DerivesFrom {
        subject: String,
        origin: String,
        span: Span,
    },
    
    /// Dependency: `subject requires dependency`
    ///
    /// # Example
    /// ```dol
    /// identity requires no authority
    /// ```
    Requires {
        subject: String,
        requirement: String,
        span: Span,
    },
    
    /// Composition: `uses reference`
    ///
    /// # Example
    /// ```dol
    /// uses container.exists
    /// ```
    Uses {
        reference: String,
        span: Span,
    },
    
    /// Event production: `action emits event`
    ///
    /// # Example
    /// ```dol
    /// each transition emits event
    /// ```
    Emits {
        action: String,
        event: String,
        span: Span,
    },
    
    /// Equivalence constraint: `subject matches target`
    ///
    /// # Example
    /// ```dol
    /// container state matches declared state
    /// ```
    Matches {
        subject: String,
        target: String,
        span: Span,
    },
    
    /// Negative constraint: `subject never action`
    ///
    /// # Example
    /// ```dol
    /// container identity never changes
    /// ```
    Never {
        subject: String,
        action: String,
        span: Span,
    },
}

/// Source location information for error reporting and tooling.
///
/// Spans track the byte offsets of AST nodes in the original source,
/// enabling precise error messages and IDE integration.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    /// Starting byte offset (inclusive)
    pub start: usize,
    
    /// Ending byte offset (exclusive)
    pub end: usize,
    
    /// Line number (1-indexed)
    pub line: usize,
    
    /// Column number (1-indexed)
    pub column: usize,
}
```

#### 3.2 Add Doc Comments to `lexer.rs`

```rust
//! Lexical analysis for Metal DOL.
//!
//! This module provides tokenization of DOL source text into a stream of tokens
//! that can be consumed by the parser. The lexer handles keywords, identifiers,
//! operators, version numbers, and string literals.
//!
//! # Example
//!
//! ```rust
//! use metadol::lexer::{Lexer, TokenKind};
//!
//! let mut lexer = Lexer::new("gene container.exists { }");
//! 
//! assert_eq!(lexer.next_token().kind, TokenKind::Gene);
//! assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
//! assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
//! ```

/// A lexical token produced by the lexer.
///
/// Tokens carry their kind, the original source text (lexeme), and
/// source location information for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The category of this token
    pub kind: TokenKind,
    
    /// The original source text that produced this token
    pub lexeme: String,
    
    /// Source location for error reporting
    pub span: Span,
}

/// The category of a lexical token.
///
/// TokenKind distinguishes between keywords, operators, literals,
/// and other syntactic elements of the DOL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // === Keywords ===
    /// The `gene` keyword for atomic declarations
    Gene,
    /// The `trait` keyword for composable behaviors
    Trait,
    /// The `constraint` keyword for invariants
    Constraint,
    /// The `system` keyword for top-level composition
    System,
    /// The `evolves` keyword for evolution declarations
    Evolves,
    /// The `exegesis` keyword for explanation sections
    Exegesis,
    /// The `test` keyword for test declarations
    Test,
    
    // === Predicates ===
    /// The `has` predicate for property possession
    Has,
    /// The `is` predicate for state/behavior
    Is,
    /// The `derives` keyword (followed by `from`)
    Derives,
    /// The `from` keyword (after `derives`)
    From,
    /// The `requires` predicate for dependencies
    Requires,
    /// The `uses` predicate for composition
    Uses,
    /// The `emits` predicate for event production
    Emits,
    /// The `matches` predicate for equivalence
    Matches,
    /// The `never` predicate for negative constraints
    Never,
    
    // === Evolution Operators ===
    /// The `adds` operator for new capabilities
    Adds,
    /// The `deprecates` operator for soft removal
    Deprecates,
    /// The `removes` operator for hard removal
    Removes,
    /// The `because` keyword for rationale
    Because,
    
    // === Delimiters ===
    /// Left brace `{`
    LeftBrace,
    /// Right brace `}`
    RightBrace,
    /// At symbol `@` for version annotations
    At,
    /// Greater-than `>` for lineage
    Greater,
    /// Greater-than-or-equal `>=` for version constraints
    GreaterEqual,
    
    // === Literals ===
    /// A dot-notation identifier like `container.exists`
    Identifier,
    /// A semantic version number like `0.0.1`
    Version,
    /// A quoted string literal
    String,
    
    // === Special ===
    /// End of file
    Eof,
    /// Unrecognized input (for error recovery)
    Error,
}

/// The lexer for Metal DOL source text.
///
/// The lexer maintains internal state as it scans through source text,
/// producing tokens on demand. It handles whitespace and comments
/// automatically, and provides source location tracking.
///
/// # Example
///
/// ```rust
/// use metadol::lexer::Lexer;
///
/// let input = r#"
/// gene container.exists {
///   container has identity
/// }
/// "#;
///
/// let lexer = Lexer::new(input);
/// let tokens: Vec<_> = lexer.collect();
///
/// assert!(tokens.len() > 0);
/// ```
pub struct Lexer<'a> {
    /// The source text being tokenized
    source: &'a str,
    
    /// Iterator over source characters with byte positions
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    
    /// Current byte position in source
    position: usize,
    
    /// Current line number (1-indexed)
    line: usize,
    
    /// Current column number (1-indexed)
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source text.
    ///
    /// # Arguments
    ///
    /// * `source` - The DOL source text to tokenize
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance positioned at the start of the source
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.char_indices().peekable(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Produces the next token from the source.
    ///
    /// Advances the lexer position and returns the next token.
    /// Returns `TokenKind::Eof` when the source is exhausted.
    ///
    /// # Returns
    ///
    /// The next `Token` from the source
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        
        // Implementation details...
        todo!()
    }
    
    /// Skips whitespace and comments.
    ///
    /// DOL uses `//` for single-line comments. Whitespace includes
    /// spaces, tabs, and newlines.
    fn skip_whitespace_and_comments(&mut self) {
        // Implementation...
    }
}
```

---

## Critique 4: Specification Document Rendering Issues

**Issue**: The metal-dol-specification.md may not render properly on GitHub.

### Remediation Actions

#### 4.1 Create Properly Formatted Specification

```markdown
---
title: Metal DOL Specification
version: 0.0.1
status: Draft
---

# Metal DOL Language Specification

Version 0.0.1 — Foundation

## 1. Introduction

Metal DOL (Design Ontology Language) is a declarative specification language
for ontology-first software development. This document provides the formal
specification of the language syntax and semantics.

## 2. Lexical Grammar

### 2.1 Character Set

DOL source files are encoded in UTF-8. The following character classes are defined:

- `letter` = `[a-zA-Z]`
- `digit` = `[0-9]`
- `whitespace` = `[ \t\r\n]`

### 2.2 Comments

Single-line comments begin with `//` and extend to end of line:

```
// This is a comment
gene container.exists { ... }  // Trailing comment
```

### 2.3 Keywords

Reserved keywords (case-sensitive):

| Category | Keywords |
|----------|----------|
| Declarations | `gene`, `trait`, `constraint`, `system`, `evolves` |
| Predicates | `has`, `is`, `derives`, `from`, `requires`, `uses`, `emits`, `matches`, `never` |
| Evolution | `adds`, `deprecates`, `removes`, `because` |
| Structure | `exegesis`, `test`, `given`, `when`, `then`, `always` |

### 2.4 Identifiers

Identifiers use dot notation for namespacing:

```
identifier = letter (letter | digit | '_')*
qualified-id = identifier ('.' identifier)*
```

Examples: `container`, `container.exists`, `identity.cryptographic`

### 2.5 Version Numbers

Semantic versioning format:

```
version = digit+ '.' digit+ '.' digit+
```

Examples: `0.0.1`, `1.2.3`, `10.0.0`

## 3. Syntactic Grammar (EBNF)

```ebnf
(* Top-level structure *)
file = declaration exegesis-block ;

declaration = gene-decl | trait-decl | constraint-decl 
            | system-decl | evolution-decl ;

(* Gene declaration *)
gene-decl = 'gene' qualified-id '{' statement* '}' ;

(* Trait declaration *)
trait-decl = 'trait' qualified-id '{' (uses-stmt | statement)* '}' ;

(* Constraint declaration *)
constraint-decl = 'constraint' qualified-id '{' statement* '}' ;

(* System declaration *)
system-decl = 'system' qualified-id '@' version 
              '{' (requires-clause | statement)* '}' ;

(* Evolution declaration *)
evolution-decl = 'evolves' qualified-id '@' version '>' version
                 '{' evolution-stmt* '}' ;

(* Statements *)
statement = has-stmt | is-stmt | derives-stmt | requires-stmt
          | uses-stmt | emits-stmt | matches-stmt | never-stmt ;

has-stmt = identifier 'has' identifier ;
is-stmt = identifier 'is' identifier ;
derives-stmt = identifier 'derives' 'from' phrase ;
requires-stmt = identifier 'requires' phrase ;
uses-stmt = 'uses' qualified-id ;
emits-stmt = phrase 'emits' identifier ;
matches-stmt = phrase 'matches' phrase ;
never-stmt = identifier 'never' identifier ;

(* Evolution statements *)
evolution-stmt = adds-stmt | deprecates-stmt | removes-stmt | because-stmt ;
adds-stmt = 'adds' statement ;
deprecates-stmt = 'deprecates' statement ;
removes-stmt = 'removes' qualified-id ;
because-stmt = 'because' string-literal ;

(* Requirements with version constraints *)
requires-clause = 'requires' qualified-id version-constraint ;
version-constraint = '>=' version | '>' version | '=' version ;

(* Exegesis block *)
exegesis-block = 'exegesis' '{' text '}' ;

(* Primitives *)
phrase = identifier+ ;
string-literal = '"' [^"]* '"' ;
text = [^}]* ;
```

## 4. Semantic Rules

### 4.1 Naming Conventions

| Declaration Type | Naming Pattern | Example |
|-----------------|----------------|---------|
| Gene | `domain.property` | `container.exists` |
| Trait | `domain.behavior` | `container.lifecycle` |
| Constraint | `domain.invariant` | `container.integrity` |
| System | `product.component` | `univrs.orchestrator` |

### 4.2 Exegesis Requirements

Every declaration MUST have an exegesis block. A DOL file without exegesis
is considered incomplete and MUST be rejected by validators.

### 4.3 Evolution Lineage

The `>` operator in evolution declarations denotes "descends from":

```
evolves container.lifecycle @ 0.0.2 > 0.0.1
```

Reads as: "version 0.0.2 descends from 0.0.1"

## 5. File Structure

### 5.1 Repository Layout

```
dol/
├── genes/
│   └── *.dol
├── traits/
│   ├── *.dol
│   └── *.dol.test
├── constraints/
│   ├── *.dol
│   └── *.dol.test
└── systems/
    ├── *.dol
    └── *.dol.test
```

### 5.2 File Naming

- Declaration files: `<qualified-id>.dol` (e.g., `container.exists.dol`)
- Test files: `<qualified-id>.dol.test` (e.g., `container.lifecycle.dol.test`)

## 6. Test Syntax

```ebnf
test-file = test-decl+ ;

test-decl = 'test' identifier '{' 
            given-clause* 
            when-clause* 
            then-clause+ 
            ['always'] 
            '}' ;

given-clause = 'given' phrase ;
when-clause = 'when' phrase ;
then-clause = 'then' phrase ;
```

Example:

```dol
test container.creation {
  given no container
  when container is created
  then container has identity
  then container state is created
}
```
```

---

## Critique 5: No Formal Grammar (BNF/EBNF)

**Issue**: Only prose description, no formal grammar for parser implementation.

### Remediation Actions

#### 5.1 Create `docs/grammar.ebnf`

```ebnf
(* 
 * Metal DOL Grammar Specification
 * Version: 0.0.1
 * Format: Extended Backus-Naur Form (EBNF)
 *)

(* ============================================ *)
(* Lexical Rules                                *)
(* ============================================ *)

letter = 'a' | 'b' | ... | 'z' | 'A' | 'B' | ... | 'Z' ;
digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' ;
underscore = '_' ;

identifier = letter , { letter | digit | underscore } ;
qualified_identifier = identifier , { '.' , identifier } ;
version = digit , { digit } , '.' , digit , { digit } , '.' , digit , { digit } ;

string_literal = '"' , { string_char } , '"' ;
string_char = ? any character except '"' or newline ? ;

whitespace = ' ' | '\t' | '\r' | '\n' ;
comment = '//' , { ? any character except newline ? } , '\n' ;

(* ============================================ *)
(* Keywords                                     *)
(* ============================================ *)

keyword = 'gene' | 'trait' | 'constraint' | 'system' | 'evolves'
        | 'exegesis' | 'test' | 'given' | 'when' | 'then' | 'always'
        | 'has' | 'is' | 'derives' | 'from' | 'requires' | 'uses'
        | 'emits' | 'matches' | 'never'
        | 'adds' | 'deprecates' | 'removes' | 'because' ;

(* ============================================ *)
(* Top-Level Structure                          *)
(* ============================================ *)

dol_file = declaration , exegesis_block ;

declaration = gene_declaration
            | trait_declaration
            | constraint_declaration
            | system_declaration
            | evolution_declaration ;

exegesis_block = 'exegesis' , '{' , exegesis_text , '}' ;
exegesis_text = { ? any character except '}' ? } ;

(* ============================================ *)
(* Gene Declaration                             *)
(* ============================================ *)

gene_declaration = 'gene' , qualified_identifier , '{' , { statement } , '}' ;

(* ============================================ *)
(* Trait Declaration                            *)
(* ============================================ *)

trait_declaration = 'trait' , qualified_identifier , '{' , 
                    { uses_statement | statement } , '}' ;

(* ============================================ *)
(* Constraint Declaration                       *)
(* ============================================ *)

constraint_declaration = 'constraint' , qualified_identifier , '{' , 
                         { constraint_statement } , '}' ;

constraint_statement = matches_statement | never_statement | statement ;

(* ============================================ *)
(* System Declaration                           *)
(* ============================================ *)

system_declaration = 'system' , qualified_identifier , '@' , version , '{' ,
                     { requires_clause | statement } , '}' ;

requires_clause = 'requires' , qualified_identifier , version_constraint ;
version_constraint = ( '>=' | '>' | '=' ) , version ;

(* ============================================ *)
(* Evolution Declaration                        *)
(* ============================================ *)

evolution_declaration = 'evolves' , qualified_identifier , '@' , version , 
                        '>' , version , '{' , { evolution_statement } , '}' ;

evolution_statement = adds_statement
                    | deprecates_statement
                    | removes_statement
                    | because_statement ;

adds_statement = 'adds' , statement ;
deprecates_statement = 'deprecates' , statement ;
removes_statement = 'removes' , qualified_identifier ;
because_statement = 'because' , string_literal ;

(* ============================================ *)
(* Statements                                   *)
(* ============================================ *)

statement = has_statement
          | is_statement
          | derives_statement
          | requires_statement
          | uses_statement
          | emits_statement
          | matches_statement
          | never_statement ;

has_statement = subject , 'has' , property ;
is_statement = subject , 'is' , state ;
derives_statement = subject , 'derives' , 'from' , phrase ;
requires_statement = subject , 'requires' , phrase ;
uses_statement = 'uses' , qualified_identifier ;
emits_statement = phrase , 'emits' , identifier ;
matches_statement = phrase , 'matches' , phrase ;
never_statement = subject , 'never' , action ;

subject = identifier ;
property = identifier ;
state = identifier ;
action = identifier ;
phrase = identifier , { identifier } ;

(* ============================================ *)
(* Test File Syntax                             *)
(* ============================================ *)

test_file = { test_declaration } ;

test_declaration = 'test' , identifier , '{' ,
                   { given_clause } ,
                   { when_clause } ,
                   { then_clause } ,
                   [ 'always' ] , '}' ;

given_clause = 'given' , phrase ;
when_clause = 'when' , phrase ;
then_clause = 'then' , phrase ;
```

---

## Critique 6: No CI/CD or Enforcement Mechanisms

**Issue**: No GitHub Actions, linters, validators, or enforcement visible.

### Remediation Actions

#### 6.1 Create `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        
      - name: Run cargo check
        run: cargo check --all-features

  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        
      - name: Run tests
        run: cargo test --all-features --verbose
        
      - name: Run doc tests
        run: cargo test --doc

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt
          
      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: clippy
          
      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        
      - name: Build documentation
        run: cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: -Dwarnings

  dol-validate:
    name: DOL Validation
    runs-on: ubuntu-latest
    needs: [check, test]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        
      - name: Build DOL tools
        run: cargo build --release
        
      - name: Validate all DOL files
        run: |
          for file in $(find examples -name "*.dol"); do
            echo "Validating $file..."
            ./target/release/dol-parse "$file" --validate
          done
          
      - name: Check exegesis coverage
        run: ./target/release/dol-check --require-exegesis examples/

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
        
      - name: Generate coverage report
        run: cargo tarpaulin --out xml --output-dir coverage
        
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml
          fail_ci_if_error: true
```

#### 6.2 Add Pre-commit Hooks (`.pre-commit-config.yaml`)

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: dol-validate
        name: DOL validation
        entry: bash -c 'for f in $(git diff --cached --name-only --diff-filter=ACM | grep "\.dol$"); do cargo run --bin dol-parse -- "$f" --validate || exit 1; done'
        language: system
        types: [file]
        files: \.dol$
```

---

## Critique 7: Aspirational vs. Functional Workflow

**Issue**: Claimed workflow (DOL → Tests → Code) has no implementation.

### Remediation Actions

#### 7.1 Implement `dol-test` Code Generator

Create `src/bin/dol-test.rs`:

```rust
//! DOL Test Generator
//!
//! Transforms `.dol.test` files into executable Rust tests.

use std::fs;
use std::path::PathBuf;
use clap::Parser;
use metadol::test_parser::parse_test_file;
use metadol::codegen::generate_rust_tests;

#[derive(Parser)]
#[command(name = "dol-test")]
#[command(about = "Generate Rust tests from DOL test specifications")]
struct Cli {
    /// Input .dol.test file or directory
    input: PathBuf,
    
    /// Output directory for generated tests
    #[arg(short, long, default_value = "src/generated_tests")]
    output: PathBuf,
    
    /// Generate test stubs only (no assertions)
    #[arg(long)]
    stubs: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let test_files = if cli.input.is_dir() {
        find_test_files(&cli.input)?
    } else {
        vec![cli.input.clone()]
    };
    
    fs::create_dir_all(&cli.output)?;
    
    for test_file in test_files {
        println!("Processing: {}", test_file.display());
        
        let content = fs::read_to_string(&test_file)?;
        let tests = parse_test_file(&content)?;
        let rust_code = generate_rust_tests(&tests, cli.stubs)?;
        
        let output_name = test_file
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace('.', "_");
        let output_path = cli.output.join(format!("{}_test.rs", output_name));
        
        fs::write(&output_path, rust_code)?;
        println!("  Generated: {}", output_path.display());
    }
    
    println!("\nDone! Generated {} test file(s)", test_files.len());
    Ok(())
}

fn find_test_files(dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.to_string_lossy().ends_with(".dol.test") {
            files.push(path);
        } else if path.is_dir() {
            files.extend(find_test_files(&path)?);
        }
    }
    Ok(files)
}
```

#### 7.2 Implement `dol-check` Validator

Create `src/bin/dol-check.rs`:

```rust
//! DOL Validation Tool
//!
//! CI gate ensuring code changes have corresponding DOL coverage.

use std::fs;
use std::path::PathBuf;
use clap::Parser;
use metadol::validator::{validate_directory, ValidationResult};

#[derive(Parser)]
#[command(name = "dol-check")]
#[command(about = "Validate DOL specifications and coverage")]
struct Cli {
    /// DOL directory to validate
    dol_dir: PathBuf,
    
    /// Source directory to check coverage against
    #[arg(short, long)]
    against: Option<PathBuf>,
    
    /// Require exegesis in all DOL files
    #[arg(long)]
    require_exegesis: bool,
    
    /// Minimum coverage percentage (0-100)
    #[arg(long, default_value = "0")]
    min_coverage: u8,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let result = validate_directory(&cli.dol_dir, &ValidationOptions {
        require_exegesis: cli.require_exegesis,
        source_dir: cli.against.clone(),
        min_coverage: cli.min_coverage,
    })?;
    
    print_results(&result);
    
    if result.has_errors() {
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_results(result: &ValidationResult) {
    println!("DOL Validation Results");
    println!("======================");
    println!("Files validated: {}", result.files_checked);
    println!("Errors: {}", result.errors.len());
    println!("Warnings: {}", result.warnings.len());
    
    if !result.errors.is_empty() {
        println!("\nErrors:");
        for error in &result.errors {
            println!("  ❌ {}: {}", error.file, error.message);
        }
    }
    
    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  ⚠️  {}: {}", warning.file, warning.message);
        }
    }
    
    if let Some(coverage) = result.coverage {
        println!("\nCoverage: {:.1}%", coverage);
    }
}
```

---

## Critique 8: Missing Examples and Tutorials

**Issue**: No quickstart guide, tutorial walkthroughs, or example DOL files.

### Remediation Actions

#### 8.1 Create Example Directory Structure

```
examples/
├── genes/
│   ├── container.exists.dol
│   ├── identity.cryptographic.dol
│   ├── state.finite.dol
│   └── node.exists.dol
├── traits/
│   ├── container.lifecycle.dol
│   ├── container.lifecycle.dol.test
│   ├── node.discovery.dol
│   └── node.discovery.dol.test
├── constraints/
│   ├── container.integrity.dol
│   ├── container.integrity.dol.test
│   └── cluster.consistency.dol
├── systems/
│   ├── univrs.orchestrator.dol
│   └── univrs.orchestrator.dol.test
└── tutorials/
    ├── 01-first-gene.md
    ├── 02-composing-traits.md
    ├── 03-writing-constraints.md
    └── 04-test-driven-dol.md
```

#### 8.2 Create Tutorial: `docs/tutorials/01-first-gene.md`

```markdown
# Tutorial 1: Your First Gene

This tutorial walks you through creating your first Metal DOL gene declaration.

## What is a Gene?

A gene is the atomic unit of DOL. It declares fundamental truths that cannot
be decomposed further. Think of genes as the DNA of your system specification.

## Step 1: Create the File

Create a new file called `user.exists.dol`:

```dol
gene user.exists {
  user has identity
  user has email
  user has created_at
}

exegesis {
  A user represents a human actor in the system. Every user has a unique
  cryptographic identity, an email address for communication, and a
  timestamp recording when the user account was created.
}
```

## Step 2: Parse and Validate

```bash
dol-parse user.exists.dol --validate
```

Expected output:

```
✓ Parsed: user.exists
✓ Exegesis present
✓ 3 statements validated
```

## Step 3: Understanding the Structure

Let's break down the DOL file:

### Declaration Header

```dol
gene user.exists {
```

- `gene` - The declaration type
- `user.exists` - Qualified name using dot notation
- `{` - Opens the statement block

### Statements

```dol
  user has identity
  user has email
  user has created_at
```

Each statement uses the `has` predicate to declare properties.
The pattern is: `subject has property`

### Exegesis

```dol
exegesis {
  A user represents a human actor...
}
```

The exegesis explains the intent and provides context. It's required
for every DOL declaration.

## Step 4: Create a Test

Create `user.exists.dol.test`:

```dol
test user.has.identity {
  given new user
  when user is created
  then user has identity
}

test user.has.email {
  given new user
  when user is created
  then user has email
  then email is valid format
}
```

## Step 5: Generate Rust Tests

```bash
dol-test user.exists.dol.test --output src/tests/
```

This generates executable Rust test scaffolding.

## Next Steps

- [Tutorial 2: Composing Traits](02-composing-traits.md)
- [Tutorial 3: Writing Constraints](03-writing-constraints.md)
```

---

## Critique 9: Low Maturity Signals

**Issue**: No releases, no community engagement, sparse commits.

### Remediation Actions

#### 9.1 Create Release Process

1. **Add `CHANGELOG.md`**:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial lexer implementation
- Initial parser implementation
- AST definitions for all declaration types
- Basic CLI tools: dol-parse, dol-test, dol-check

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Nothing yet

## [0.0.1] - 2024-XX-XX

### Added
- Initial release with core language specification
- Lexer for DOL tokenization
- Recursive descent parser
- Complete AST representation
- Exegesis parsing and validation
```

2. **Create Release Workflow** (`.github/workflows/release.yml`):

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    name: Create Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build changelog
        id: changelog
        run: |
          # Extract changelog for this version
          version=${GITHUB_REF#refs/tags/v}
          # ... extract section from CHANGELOG.md
          
      - name: Create Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          body: ${{ steps.changelog.outputs.content }}
          draft: false
          prerelease: false
```

#### 9.2 Add Community Files

1. **`CONTRIBUTING.md`**:

```markdown
# Contributing to Metal DOL

Thank you for your interest in contributing to Metal DOL!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/metadol.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make changes and test: `cargo test`
5. Submit a pull request

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/univrs/metadol.git
cd metadol
cargo build
cargo test
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add doc comments to public items
- Write tests for new functionality

## Pull Request Process

1. Update documentation for any changed functionality
2. Add tests covering your changes
3. Ensure all CI checks pass
4. Request review from maintainers
```

2. **`CODE_OF_CONDUCT.md`** (Contributor Covenant)

3. **Issue Templates** (`.github/ISSUE_TEMPLATE/`)

---

## Critique 10: Overall Assessment

**Issue**: Early-stage prototype, conceptually intriguing but not practically applicable.

### Comprehensive Implementation Plan

#### Phase 1: Foundation (Week 1-2)

| Task | Priority | Effort |
|------|----------|--------|
| Add comprehensive doc comments to all source files | High | 4h |
| Create unit tests for lexer (20+ tests) | High | 6h |
| Create unit tests for parser (20+ tests) | High | 6h |
| Add proper Cargo.toml with metadata | High | 1h |
| Create formal EBNF grammar | High | 3h |

#### Phase 2: Tooling (Week 3-4)

| Task | Priority | Effort |
|------|----------|--------|
| Implement `dol-parse` CLI | High | 8h |
| Implement `dol-test` generator | High | 12h |
| Implement `dol-check` validator | Medium | 8h |
| Create GitHub Actions CI/CD | High | 4h |

#### Phase 3: Documentation (Week 5-6)

| Task | Priority | Effort |
|------|----------|--------|
| Rewrite README with quick start | High | 4h |
| Create 4 tutorials | Medium | 8h |
| Create API documentation | Medium | 4h |
| Add example DOL files | High | 4h |

#### Phase 4: Community (Week 7-8)

| Task | Priority | Effort |
|------|----------|--------|
| Create v0.0.1 release | High | 2h |
| Add CONTRIBUTING.md | Medium | 2h |
| Add issue templates | Low | 1h |
| Write blog post announcement | Low | 4h |

---

## Appendix: Recommended Project Structure

```
metadol/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   └── release.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
├── docs/
│   ├── grammar.ebnf
│   ├── specification.md
│   └── tutorials/
│       ├── 01-first-gene.md
│       ├── 02-composing-traits.md
│       ├── 03-writing-constraints.md
│       └── 04-test-driven-dol.md
├── examples/
│   ├── genes/
│   ├── traits/
│   ├── constraints/
│   └── systems/
├── src/
│   ├── lib.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── ast.rs
│   ├── validator.rs
│   ├── codegen.rs
│   └── bin/
│       ├── dol-parse.rs
│       ├── dol-test.rs
│       └── dol-check.rs
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   └── integration_tests.rs
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
└── LICENSE
```

---

## Success Metrics

After implementing this remediation plan, the project should achieve:

- [ ] **100% test coverage** for lexer and parser core functions
- [ ] **Green CI/CD** on every pull request
- [ ] **Documentation score** of 100% (all public items documented)
- [ ] **Clippy clean** with no warnings
- [ ] **At least 10 example DOL files** demonstrating all features
- [ ] **4 tutorials** covering end-to-end workflow
- [ ] **First release** tagged and published
- [ ] **README** with working quick-start commands

---

*This remediation plan addresses all 10 critique points systematically. Implementation should proceed phase by phase, with each phase building on the previous. The goal is to transform Metal DOL from an aspirational prototype into a production-ready DSL toolchain.*

## TypeChecker Overview

  The typechecker.rs implements bidirectional type checking with two modes:

  1. Inference mode - Synthesizes a type from an expression (bottom-up)
  2. Checking mode - Verifies an expression against an expected type (top-down)

### Type Environment (Scoping)

```
// Core Types

  pub enum Type {
      // Primitives
      Void, Bool, String,
      Int8, Int16, Int32, Int64,      // Signed integers
      UInt8, UInt16, UInt32, UInt64,  // Unsigned integers
      Float32, Float64,               // Floats

      // Compound types
      Function { params: Vec<Type>, ret: Box<Type> },
      Tuple { elements: Vec<Type> },
      Generic { name: String, type_params: Vec<String>, args: Vec<Type> },

      // Type variables for inference
      Var { id: u32 },     // Unification variable
      Unknown,             // Placeholder during inference

      // Special types
      Any,                 // Top type (accepts everything)
      Never,               // Bottom type (unreachable)
      Error,               // Error recovery
  }

  // Type Environment (Scoping)

  pub struct TypeEnv {
      bindings: HashMap<String, Type>,
      parent: Option<Box<TypeEnv>>,  // Lexical scoping chain
  }

  When entering a new scope (function body, block), a child environment is created:
  fn child(&self) -> TypeEnv {
      TypeEnv { bindings: HashMap::new(), parent: Some(Box::new(self.clone())) }
  }

  // Lookup walks up the parent chain until it finds the binding or reaches the root.

  #**Effect Context (Purity)**

  pub enum EffectContext {
      Pure,   // No side effects allowed
      Sex,    // Side effects permitted
  }

```
## This tracks whether the current context allows side effects, enabling the type checker to verify purity constraints.
   
  ---
#  DOL Module Structure


  Here's dol/token.dol as a simple example:
- 1. Module declaration with semantic version
- 2. Documentation block (required for all public items)
- 3. Gene with enum type (like a Rust enum)
- 4. Gene with fields (like a Rust struct)
- 5. Gene with fields AND methods
- 6. Standalone function

```  
  // - 1. Module declaration with semantic version
 
 
  module dol.token @ 0.4.0

  // - 2. Documentation block (required for all public items)
  exegesis {
      Token definitions for the DOL lexer.
  }

  // 3. Gene with enum type (like a Rust enum)

pub gene TokenKind {
      type: enum {
          Module, Use, Pub, Gene, Trait, System,
          Has, Is, Derives, From, Requires, Uses,
          Fun, Let, Const, Return, If, Else, Match,
          // ... more variants
      }
  }


  // 4. Gene with fields (like a Rust struct)

pub gene Span {
      has start: UInt64
      has end: UInt64
      has line: UInt32
      has column: UInt32
  }

  // 5. Gene with fields AND methods
  pub gene Token {
      has kind: TokenKind
      has span: Span
      has text: String

      // Method using `this` (like `self` in Rust)
      fun is_keyword() -> Bool {
          match this.kind {
              TokenKind::Module => true,
              TokenKind::Use => true,
              // ... more cases
              _ => false
          }
      }
  }

  // 6. Standalone function
  pub fun span_contains(outer: Span, inner: Span) -> Bool {
      return outer.start <= inner.start && inner.end <= outer.end;
  }
```
  Generated Rust Structure

  When processed by dol-build-crate, this produces stage2/src/token.rs:
```
  //! Module: dol.token
  //! Generated from DOL source - do not edit

  use crate::compat::*;
  use crate::ast::*;
  // ... sibling imports

  #[derive(Debug, Clone, PartialEq)]
  pub enum TokenKind {
      Module, Use, Pub, Gene, Trait, System,
      // ...
  }

  #[derive(Debug, Clone, PartialEq)]
  pub struct Span {
      pub start: u64,
      pub end: u64,
      pub line: u32,
      pub column: u32,
  }

  #[derive(Debug, Clone, PartialEq)]
  pub struct Token {
      pub kind: TokenKind,
      pub span: Span,
      pub text: String,
  }

  impl Token {
      pub fn is_keyword(&self) -> bool {
          match self.kind {
              TokenKind::Module => true,
              // ...
          }
      }
  }

  pub fn span_contains(outer: Span, inner: Span) -> bool {
      outer.start <= inner.start && inner.end <= outer.end
  }

```
 
## Module Dependencies

  The crate generator automatically:
  1. Extracts module name from module dol.token → token
  2. Generates sibling imports (use crate::ast::*;, etc.)
  3. Creates lib.rs with pub mod token;
  4. Creates prelude.rs for convenient re-exports

─────────────────────────────────────────────────────────

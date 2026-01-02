!REJECTED # BAD Design: `provides` Keyword for Static Constructors

/// NOT TO BE IMPLEMENTED 


## Overview

The `provides` keyword introduces static factory methods (constructors) to DOL genes. Unlike instance methods declared with `fun`/`is`, static methods have no `self` parameter and can create new instances of the gene type.

**Status**: Design Document
**Author**: Type System Architect
**Date**: 2026-01-02

## Motivation

Currently, DOL genes support instance methods via the `fun` keyword inside gene bodies:

```dol
gene Counter {
    has value: Int64

    fun increment() -> Int64 {
        return value + 1  // implicitly self.value
    }
}
```

However, there is no way to create gene instances from within DOL itself. Users must rely on external runtime mechanisms to instantiate genes. The `provides` keyword enables genes to define their own constructors:

```dol
gene Credits {
    has amount: UInt64

    provides zero() -> Self {
        Credits { amount: 0 }
    }

    provides from_amount(amt: UInt64) -> Self {
        Credits { amount: amt }
    }
}

// Usage:
let c = Credits::zero()
let d = Credits::from_amount(100)
```

## Design Goals

1. **Clear semantic distinction**: `provides` = static, `fun` = instance
2. **Self type resolution**: `Self` in return type resolves to the containing gene
3. **No implicit parameters**: Static methods have only their declared parameters
4. **Consistent export naming**: `Gene_method` format for WASM exports
5. **Call syntax**: `Gene::method()` for invocation (consistent with Rust)

---

## 1. AST Changes

### File: `/home/ardeshir/repos/univrs-dol/src/ast.rs`

#### Option A: New Statement Variant (Recommended)

Add a new variant to the `Statement` enum specifically for static methods:

```rust
/// A statement within a DOL declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Statement {
    // ... existing variants ...

    /// Function declaration inside a gene or trait: `fun name(...) -> Type { ... }`
    Function(Box<FunctionDecl>),

    /// Static factory method: `provides name(...) -> Self { ... }`
    /// Static methods have no implicit `self` parameter.
    Provides(Box<FunctionDecl>),
}
```

**Pros**:
- Clear distinction at AST level
- No changes to `FunctionDecl` structure
- Pattern matching naturally separates static vs instance

**Cons**:
- Slight code duplication in handling

#### Option B: Add `is_static` field to FunctionDecl

```rust
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FunctionDecl {
    pub visibility: Visibility,
    pub purity: Purity,
    /// Whether this is a static method (declared with `provides`)
    pub is_static: bool,
    pub name: String,
    pub type_params: Option<TypeParams>,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeExpr>,
    pub body: Vec<Stmt>,
    pub exegesis: String,
    pub span: Span,
}
```

**Pros**:
- Single structure for all functions
- Easier serialization

**Cons**:
- Less explicit at AST level
- Requires checking a boolean everywhere

### Recommendation: Option A

A separate `Statement::Provides` variant makes the distinction explicit and enables cleaner pattern matching in the compiler. The `FunctionDecl` structure remains unchanged, reused for both instance and static methods.

---

## 2. Lexer Changes

### File: `/home/ardeshir/repos/univrs-dol/src/lexer.rs`

Add the `Provides` token kind:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ... existing tokens ...

    // === Static Method Keyword ===
    /// The `provides` keyword for static factory methods
    Provides,
}
```

Add to `keyword_kind` function:

```rust
fn keyword_kind(&self, lexeme: &str) -> Option<TokenKind> {
    match lexeme {
        // ... existing keywords ...
        "provides" => Some(TokenKind::Provides),
        _ => None,
    }
}
```

Add to `Display` implementation:

```rust
impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing ...
            TokenKind::Provides => write!(f, "provides"),
        }
    }
}
```

Add to `is_keyword()`:

```rust
pub fn is_keyword(&self) -> bool {
    matches!(
        self,
        // ... existing ...
        | TokenKind::Provides
    )
}
```

---

## 3. Parser Changes

### File: `/home/ardeshir/repos/univrs-dol/src/parser.rs`

Modify `parse_statement()` to handle `provides`:

```rust
fn parse_statement(&mut self) -> Result<Statement, ParseError> {
    let start_span = self.current.span;

    // ... existing statement handling ...

    // Handle DOL 2.0 function declarations inside genes: [pub] [sex] fun name(...) -> Type { ... }
    // Check for optional visibility modifier
    let mut visibility = Visibility::Private;
    let mut purity = Purity::Pure;

    if self.current.kind == TokenKind::Pub {
        visibility = Visibility::Public;
        self.advance();
    }

    // Check for optional purity modifier (sex = side-effecting)
    if self.current.kind == TokenKind::Sex {
        purity = Purity::Sex;
        self.advance();
    }

    // Handle instance methods (fun)
    if self.current.kind == TokenKind::Function {
        let mut func = self.parse_function_decl()?;
        func.visibility = visibility;
        func.purity = purity;
        return Ok(Statement::Function(Box::new(func)));
    }

    // Handle static factory methods (provides)
    if self.current.kind == TokenKind::Provides {
        let func = self.parse_provides_decl(visibility, purity)?;
        return Ok(Statement::Provides(Box::new(func)));
    }

    // ... rest of statement parsing ...
}
```

Add new parsing function for `provides`:

```rust
/// Parses a `provides` declaration (static factory method).
///
/// Syntax: `[pub] [sex] provides name(...) -> Type { ... }`
fn parse_provides_decl(
    &mut self,
    visibility: Visibility,
    purity: Purity,
) -> Result<FunctionDecl, ParseError> {
    let start_span = self.current.span;
    self.expect(TokenKind::Provides)?;

    let name = self.expect_identifier_or_keyword()?;

    // Parse optional type parameters: <T, U>
    let type_params = if self.current.kind == TokenKind::Lt {
        Some(self.parse_type_params()?)
    } else {
        None
    };

    // Parse parameters
    self.expect(TokenKind::LeftParen)?;
    let params = self.parse_function_params()?;
    self.expect(TokenKind::RightParen)?;

    // Parse return type (required for provides - must return Self or gene type)
    let return_type = if self.current.kind == TokenKind::Arrow {
        self.advance();
        Some(self.parse_type()?)
    } else {
        // Default to Self if no return type specified
        Some(TypeExpr::Named("Self".to_string()))
    };

    // Parse body
    let body = if self.current.kind == TokenKind::LeftBrace {
        self.advance();
        let mut stmts = Vec::new();
        while self.current.kind != TokenKind::RightBrace && self.current.kind != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;
        stmts
    } else {
        Vec::new()
    };

    let span = start_span.merge(&self.previous.span);

    Ok(FunctionDecl {
        visibility,
        purity,
        name,
        type_params,
        params,
        return_type,
        body,
        exegesis: String::new(),
        span,
    })
}
```

### Parsing `Self` Type

The `Self` type should be parsed as a named type in `parse_type()`:

```rust
fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
    // Handle Self keyword
    if self.current.lexeme == "Self" {
        self.advance();
        return Ok(TypeExpr::Named("Self".to_string()));
    }

    // ... rest of type parsing ...
}
```

### Parsing Static Method Calls: `Gene::method()`

Add support for the `::` path separator in expression parsing:

```rust
fn parse_primary(&mut self) -> Result<Expr, ParseError> {
    // ... existing primary parsing ...

    // After parsing an identifier, check for :: (static method call)
    if self.current.kind == TokenKind::PathSep {
        self.advance(); // consume ::
        let method_name = self.expect_identifier()?;

        // Parse arguments
        self.expect(TokenKind::LeftParen)?;
        let args = self.parse_call_args()?;
        self.expect(TokenKind::RightParen)?;

        // Return a static call expression
        return Ok(Expr::Call {
            callee: Box::new(Expr::Identifier(format!("{}::{}", gene_name, method_name))),
            args,
        });
    }
}
```

---

## 4. WASM Codegen Changes

### File: `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs`

### 4.1 Modified `extract_functions`

Update to handle both instance and static methods:

```rust
fn extract_functions<'a>(
    &self,
    module: &'a Declaration,
) -> Result<Vec<ExtractedFunction<'a>>, WasmError> {
    use crate::ast::{Declaration, Statement};

    match module {
        Declaration::Gene(gene) => {
            // Collect field names for implicit self access (instance methods only)
            let mut field_names: Vec<String> = Vec::new();

            // ... existing field collection code ...

            let gene_context = if field_names.is_empty() {
                None
            } else {
                Some(GeneContext {
                    gene_name: gene.name.clone(),
                    field_names,
                })
            };

            let mut funcs = Vec::new();
            for stmt in &gene.statements {
                match stmt {
                    // Instance method: has implicit self parameter
                    Statement::Function(func) => {
                        funcs.push(ExtractedFunction {
                            func: func.as_ref(),
                            exported_name: format!("{}.{}", gene.name, func.name),
                            gene_context: gene_context.clone(),
                            is_static: false,
                        });
                    }
                    // Static method: NO implicit self parameter
                    Statement::Provides(func) => {
                        funcs.push(ExtractedFunction {
                            func: func.as_ref(),
                            exported_name: format!("{}_{}", gene.name, func.name),
                            gene_context: Some(GeneContext {
                                gene_name: gene.name.clone(),
                                field_names: field_names.clone(),
                            }),
                            is_static: true,
                        });
                    }
                    _ => {}
                }
            }
            Ok(funcs)
        }
        // ... other declaration types ...
    }
}
```

### 4.2 Modified `ExtractedFunction` Structure

```rust
struct ExtractedFunction<'a> {
    func: &'a crate::ast::FunctionDecl,
    exported_name: String,
    gene_context: Option<GeneContext>,
    /// Whether this is a static method (no implicit self)
    is_static: bool,
}
```

### 4.3 Modified Type Section Generation

```rust
// Add user function types
for extracted in &functions {
    let mut params: Vec<ValType> = Vec::new();

    // For instance methods (non-static), add implicit 'self' parameter
    if !extracted.is_static && extracted.gene_context.is_some() {
        params.push(ValType::I32); // self is a pointer
    }

    // Add declared parameters
    for p in &extracted.func.params {
        params.push(self.dol_type_to_wasm(&p.type_ann)?);
    }

    // Handle return type
    let results = if let Some(ref ret_type) = extracted.func.return_type {
        // Resolve Self type to i32 (pointer)
        let wasm_type = match ret_type {
            TypeExpr::Named(name) if name == "Self" => ValType::I32,
            other => self.dol_type_to_wasm(other)?,
        };
        vec![wasm_type]
    } else {
        vec![]
    };

    types.function(params, results);
}
```

### 4.4 Modified `LocalsTable::new_with_gene_context`

Add parameter to indicate static vs instance:

```rust
fn new_with_gene_context(
    params: &[crate::ast::FunctionParam],
    gene_context: Option<&GeneContext>,
    is_static: bool,
) -> Self {
    // Only add implicit self for non-static methods
    let has_self = gene_context.is_some() && !is_static;
    let self_offset = if has_self { 1u32 } else { 0u32 };

    let mut table = Self {
        param_count: params.len() as u32 + self_offset,
        // ... rest of initialization ...
    };

    // Add implicit 'self' parameter at index 0 for instance methods only
    if has_self {
        table.name_to_index.insert("self".to_string(), 0);
        table.var_types.insert("self".to_string(), wasm_encoder::ValType::I32);
    }

    // ... rest of method ...
}
```

### 4.5 WASM Instruction Sequence for Static Constructor

For a `provides` method like:

```dol
provides zero() -> Self {
    Credits { amount: 0 }
}
```

The generated WASM should:

1. Calculate struct size from gene layout
2. Call `alloc(size)` to get a pointer
3. Store field values at appropriate offsets
4. Return the pointer

```wasm
;; Credits_zero() -> i32
(func $Credits_zero (result i32)
  (local $ptr i32)

  ;; Allocate memory for Credits struct
  i32.const 8         ;; size of Credits (1 UInt64 field)
  call $alloc
  local.set $ptr

  ;; Initialize amount field at offset 0
  local.get $ptr
  i64.const 0         ;; amount: 0
  i64.store offset=0

  ;; Return pointer
  local.get $ptr
)
```

### 4.6 Compile Struct Literal Expression

Add handling for struct literal expressions in `emit_expr`:

```rust
fn emit_expr(
    &self,
    func: &mut Function,
    expr: &Expr,
    locals: &LocalsTable,
) -> Result<(), WasmError> {
    match expr {
        // ... existing cases ...

        Expr::StructLiteral { type_name, fields } => {
            // Resolve type name (handle Self)
            let actual_type = if type_name == "Self" {
                // Get gene name from context
                locals.lookup_dol_type("self")
                    .or_else(|| locals.lookup_dol_type("__gene_name"))
                    .ok_or_else(|| WasmError::new("Self used outside gene context"))?
                    .to_string()
            } else {
                type_name.clone()
            };

            // Look up gene layout
            let layout = self.gene_layouts.get(&actual_type)
                .ok_or_else(|| WasmError::new(format!(
                    "Unknown gene type: {}", actual_type
                )))?;

            // Emit allocation call
            func.instruction(&Instruction::I32Const(layout.total_size as i32));
            func.instruction(&Instruction::Call(0)); // alloc function index

            // Store pointer in temp local
            let ptr_local = locals.lookup("__struct_ptr")
                .ok_or_else(|| WasmError::new("Missing __struct_ptr local"))?;
            func.instruction(&Instruction::LocalTee(ptr_local));

            // Initialize each field
            for (field_name, field_expr) in fields {
                let field_layout = layout.get_field(field_name)
                    .ok_or_else(|| WasmError::new(format!(
                        "Unknown field: {}", field_name
                    )))?;

                // Load pointer
                func.instruction(&Instruction::LocalGet(ptr_local));

                // Emit field value expression
                self.emit_expr(func, field_expr, locals)?;

                // Store based on field type
                match field_layout.field_type {
                    WasmFieldType::I32 => {
                        func.instruction(&Instruction::I32Store(
                            wasm_encoder::MemArg {
                                offset: field_layout.offset as u64,
                                align: 2,
                                memory_index: 0,
                            }
                        ));
                    }
                    WasmFieldType::I64 => {
                        func.instruction(&Instruction::I64Store(
                            wasm_encoder::MemArg {
                                offset: field_layout.offset as u64,
                                align: 3,
                                memory_index: 0,
                            }
                        ));
                    }
                    // ... other types ...
                }
            }

            // Leave pointer on stack as result
            func.instruction(&Instruction::LocalGet(ptr_local));
        }
    }
}
```

---

## 5. Type Resolution for `Self`

### Context Tracking

When compiling functions, track the current gene context:

```rust
struct CompilationContext<'a> {
    /// Current gene name (for Self resolution)
    current_gene: Option<&'a str>,
    /// Gene layout for struct operations
    gene_layout: Option<&'a GeneLayout>,
}
```

### Self Resolution Rules

1. In `provides` methods: `Self` = containing gene type
2. In `fun` methods: `Self` = containing gene type
3. In top-level functions: `Self` = error (no containing gene)
4. In trait methods: `Self` = implementing gene type (future)

---

## 6. Export Naming Convention

| Declaration | Export Name |
|-------------|-------------|
| `gene Counter { fun increment() }` | `Counter.increment` |
| `gene Counter { provides new() }` | `Counter_new` |
| `fun add(a, b)` (top-level) | `add` |

The underscore separator for static methods (`Gene_method`) distinguishes them from instance methods (`Gene.method`) and follows common conventions.

---

## 7. Test Cases

### 7.1 Parser Tests

```rust
#[test]
fn test_parse_provides() {
    let input = r#"
    gene Credits {
        has amount: UInt64

        provides zero() -> Self {
            Credits { amount: 0 }
        }
    }
    "#;

    let mut parser = Parser::new(input);
    let decl = parser.parse().unwrap();

    if let Declaration::Gene(gene) = decl {
        assert_eq!(gene.statements.len(), 2); // has + provides

        if let Statement::Provides(func) = &gene.statements[1] {
            assert_eq!(func.name, "zero");
            assert!(matches!(
                func.return_type,
                Some(TypeExpr::Named(ref n)) if n == "Self"
            ));
        } else {
            panic!("Expected Provides statement");
        }
    }
}

#[test]
fn test_parse_static_call() {
    let input = "let c = Credits::zero()";
    let mut parser = Parser::new(input);
    let stmt = parser.parse_stmt().unwrap();

    // Verify the call expression parses correctly
}
```

### 7.2 WASM Codegen Tests

```rust
#[test]
fn test_compile_provides() {
    let input = r#"
    gene Credits {
        has amount: UInt64

        provides zero() -> Self {
            return Credits { amount: 0 }
        }
    }
    "#;

    let mut parser = Parser::new(input);
    let decl = parser.parse().unwrap();

    let mut compiler = WasmCompiler::new();
    let wasm = compiler.compile(&decl).unwrap();

    // Verify WASM is valid
    wasmparser::validate(&wasm).expect("Invalid WASM");

    // Verify export exists
    // Credits_zero should be exported
}

#[test]
fn test_static_method_no_self_param() {
    // Verify that static methods don't have implicit self
    let input = r#"
    gene Point {
        has x: Int64
        has y: Int64

        provides origin() -> Self {
            return Point { x: 0, y: 0 }
        }

        fun distance() -> Int64 {
            return x + y
        }
    }
    "#;

    let mut parser = Parser::new(input);
    let decl = parser.parse().unwrap();

    // origin() should have 0 params, distance() should have 1 (self)
}
```

### 7.3 Integration Tests

```dol
// test-cases/working/provides_test.dol

gene Vec2 {
    has x: Float64
    has y: Float64

    provides zero() -> Self {
        return Vec2 { x: 0.0, y: 0.0 }
    }

    provides new(x: Float64, y: Float64) -> Self {
        return Vec2 { x: x, y: y }
    }

    fun length_squared() -> Float64 {
        return x * x + y * y
    }
}

fun test_static_constructor() -> Float64 {
    let v = Vec2::zero()
    return v.length_squared()  // Should return 0.0
}

fun test_static_with_args() -> Float64 {
    let v = Vec2::new(3.0, 4.0)
    return v.length_squared()  // Should return 25.0
}
```

---

## 8. Implementation Phases

### Phase 1: Lexer and AST (1 day)
- Add `TokenKind::Provides` to lexer
- Add `Statement::Provides` to AST
- Update Display implementations

### Phase 2: Parser (1 day)
- Implement `parse_provides_decl`
- Handle `Self` type in type parser
- Add `Gene::method()` call syntax parsing

### Phase 3: WASM Codegen (2 days)
- Modify `extract_functions` for static methods
- Update type section generation (no self param)
- Implement struct literal expression compilation
- Handle `Self` return type resolution

### Phase 4: Testing (1 day)
- Parser unit tests
- WASM codegen tests
- Integration tests with runtime

---

## 9. Future Considerations

### 9.1 Trait Static Methods

In the future, traits may also have `provides` methods:

```dol
trait Default {
    provides default() -> Self
}

gene Counter impl Default {
    has value: Int64

    provides default() -> Self {
        return Counter { value: 0 }
    }
}
```

### 9.2 Builder Pattern Support

The `provides` keyword naturally enables builder patterns:

```dol
gene Config {
    has timeout: Int64
    has retries: Int32

    provides builder() -> ConfigBuilder {
        return ConfigBuilder { config: Config { timeout: 0, retries: 0 } }
    }
}
```

### 9.3 Static vs Instance Callable on Instance

Decision: Static methods should NOT be callable on instances.

```dol
let c = Counter::new(0)   // OK - static call
let d = c::new(0)         // ERROR - can't call static on instance
let e = c.new(0)          // ERROR - new is not an instance method
```

This keeps the semantics clear and avoids confusion.

---

## 10. Summary

The `provides` keyword introduces static factory methods to DOL genes, enabling genes to define their own constructors. Key design decisions:

1. **New AST variant**: `Statement::Provides` for clarity
2. **No implicit self**: Static methods have only declared parameters
3. **Self type**: Resolves to containing gene type
4. **Export naming**: `Gene_method` for static, `Gene.method` for instance
5. **Call syntax**: `Gene::method()` using path separator

This design maintains consistency with existing DOL patterns while enabling powerful new capabilities for gene instantiation.

/// END OF NOT TO BE IMPLEMENTED
  
See `self-presence-inference.md` for the actual design.
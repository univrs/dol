# Option<T> Type Design Document

## Overview

This document specifies the design for DOL's `Option<T>` tagged union type, representing optional values that are either `Some(value)` or `None`.

## 1. DOL Syntax

### 1.1 Type Annotations

```dol
// Explicit type annotation
let maybe_num: Option<Int64> = Some(42)
let nothing: Option<Int64> = None

// Nested options
let nested: Option<Option<Int32>> = Some(None)

// In function signatures
fun find_index(arr: List<Int64>, target: Int64) -> Option<Int64> {
    // ...
}
```

### 1.2 Expression Syntax

```dol
// Construction
let opt1 = Some(42)           // Some variant
let opt2: Option<Int64> = None // None variant (requires type context)

// Pattern matching (primary way to access)
match opt1 {
    Some(n) => n * 2,
    None => 0
}

// Method-style access
opt1.is_some()              // -> Bool (true if Some)
opt1.is_none()              // -> Bool (true if None)
opt1.unwrap()               // -> T (traps if None)
opt1.unwrap_or(default)     // -> T (returns default if None)
opt1.map(|x| x * 2)         // -> Option<U> (future)
```

## 2. Memory Layout

### 2.1 Tagged Union Structure

`Option<T>` is represented as a tagged union in linear memory:

```
+------+------------------+
| tag  |     payload      |
| (1B) |   (sizeof(T))    |
+------+------------------+

tag = 0: None (payload bytes unused but allocated)
tag = 1: Some (payload contains value of type T)
```

### 2.2 Layout Rules

1. **Tag byte**: Always 1 byte at offset 0
2. **Payload alignment**: Starts at offset = align_up(1, alignof(T))
3. **Total size**: align_up(1 + sizeof(T), alignof(T)) for proper array layouts
4. **Minimum size**: At least 4 bytes (for i32 alignment compatibility)

### 2.3 Layout Examples

#### Option<Int32>
```
Offset  Size  Field
------  ----  -----
0       1     tag (0=None, 1=Some)
1-3     3     padding (for 4-byte alignment)
4       4     payload (Int32)
------  ----
Total:  8 bytes, alignment: 4
```

#### Option<Int64>
```
Offset  Size  Field
------  ----  -----
0       1     tag
1-7     7     padding (for 8-byte alignment)
8       8     payload (Int64)
------  ----
Total:  16 bytes, alignment: 8
```

#### Option<Float64>
```
Offset  Size  Field
------  ----  -----
0       1     tag
1-7     7     padding
8       8     payload (Float64)
------  ----
Total:  16 bytes, alignment: 8
```

#### Option<Bool>
```
Offset  Size  Field
------  ----  -----
0       1     tag
1-3     3     padding
4       4     payload (Bool as i32)
------  ----
Total:  8 bytes, alignment: 4
```

#### Option<String> (pointer)
```
Offset  Size  Field
------  ----  -----
0       1     tag
1-3     3     padding
4       4     payload (pointer to string data)
------  ----
Total:  8 bytes, alignment: 4
```

#### Option<Gene> (pointer to struct)
```
Offset  Size  Field
------  ----  -----
0       1     tag
1-3     3     padding
4       4     payload (pointer to gene instance)
------  ----
Total:  8 bytes, alignment: 4
```

#### Option<Option<Int32>> (nested)
```
Outer Option:
Offset  Size  Field
------  ----  -----
0       1     outer_tag
1-3     3     padding
4       1     inner_tag      --|
5-7     3     padding          | Inner Option<Int32> = 8 bytes
8       4     inner_payload  --|
------  ----
Total:  12 bytes (aligned to 4 = 12), alignment: 4
```

## 3. AST Changes

### 3.1 TypeExpr Enum (src/ast.rs)

The existing `TypeExpr::Generic` variant already handles `Option<T>`:

```rust
pub enum TypeExpr {
    // ... existing variants ...

    /// Generic type with arguments (e.g., `List<T>`, `Option<T>`)
    Generic {
        name: String,       // "Option"
        args: Vec<TypeExpr>, // [T]
    },
    // ...
}
```

No changes needed - the generic system already supports Option.

### 3.2 Expr Enum - Add Some/None Variants

```rust
pub enum Expr {
    // ... existing variants ...

    /// Some variant of Option type: `Some(expr)`
    ///
    /// Creates an Option in the Some state with the given value.
    /// The type parameter is inferred from the inner expression.
    ///
    /// # Example
    /// ```dol
    /// let x = Some(42)  // Option<Int64>
    /// ```
    Some(Box<Expr>),

    /// None variant of Option type
    ///
    /// Creates an Option in the None state.
    /// Requires type context for inference (type annotation or expected type).
    ///
    /// # Example
    /// ```dol
    /// let x: Option<Int64> = None
    /// ```
    None,

    // ... rest of variants ...
}
```

### 3.3 Pattern Enum - Add Some/None Patterns

```rust
pub enum Pattern {
    // ... existing variants ...

    /// Some pattern for Option matching: `Some(binding)` or `Some(pattern)`
    ///
    /// Matches the Some variant of an Option and binds/matches the inner value.
    ///
    /// # Example
    /// ```dol
    /// match opt {
    ///     Some(n) => n * 2,  // n binds to inner value
    ///     None => 0
    /// }
    /// ```
    Some(Box<Pattern>),

    /// None pattern for Option matching
    ///
    /// Matches the None variant of an Option.
    ///
    /// # Example
    /// ```dol
    /// match opt {
    ///     Some(_) => true,
    ///     None => false
    /// }
    /// ```
    None,

    // ... rest of variants ...
}
```

### 3.4 Complete AST Additions

```rust
// In src/ast.rs

// Add to Expr enum:
/// Some variant of Option type
Some(Box<Expr>),
/// None variant of Option type
None,

// Add to Pattern enum:
/// Some pattern with inner binding
Some(Box<Pattern>),
/// None pattern
None,

// Implement display traits
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing cases ...
            Expr::Some(inner) => write!(f, "Some({})", inner),
            Expr::None => write!(f, "None"),
            // ...
        }
    }
}
```

## 4. Parser Changes

### 4.1 Parsing Option<T> Type (Already Supported)

The parser already handles generic types via `TypeExpr::Generic`:

```rust
// In parse_type_expr()
// "Option" is parsed as identifier, then "<" triggers generic parsing
// Result: TypeExpr::Generic { name: "Option", args: [inner_type] }
```

### 4.2 Parsing Some(expr) Expression

```rust
// In parse_primary_expr() or parse_call_expr()
fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
    match self.current_token() {
        Token::Some => {
            self.advance(); // consume 'Some'
            self.expect(Token::LeftParen)?;
            let inner = self.parse_expr()?;
            self.expect(Token::RightParen)?;
            Ok(Expr::Some(Box::new(inner)))
        }
        Token::None => {
            self.advance(); // consume 'None'
            Ok(Expr::None)
        }
        // ... other cases ...
    }
}
```

### 4.3 Parsing Some/None Patterns

```rust
// In parse_pattern()
fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
    match self.current_token() {
        Token::Some => {
            self.advance(); // consume 'Some'
            self.expect(Token::LeftParen)?;
            let inner = self.parse_pattern()?;
            self.expect(Token::RightParen)?;
            Ok(Pattern::Some(Box::new(inner)))
        }
        Token::None => {
            self.advance(); // consume 'None'
            Ok(Pattern::None)
        }
        // ... other pattern cases ...
    }
}
```

### 4.4 Lexer Additions

```rust
// Add to Token enum
pub enum Token {
    // ... existing tokens ...
    Some,  // keyword "Some"
    None,  // keyword "None"
    // ...
}

// In lexer keyword recognition
fn keyword_or_ident(s: &str) -> Token {
    match s {
        // ... existing keywords ...
        "Some" => Token::Some,
        "None" => Token::None,
        _ => Token::Identifier(s.to_string()),
    }
}
```

## 5. WASM Codegen

### 5.1 Option Type Layout Computation

Add to `src/wasm/layout.rs`:

```rust
/// Compute the layout for Option<T>.
///
/// Returns (tag_offset, payload_offset, total_size, alignment).
pub fn option_layout(inner_type: &TypeExpr, registry: &GeneLayoutRegistry)
    -> Result<OptionLayout, WasmError>
{
    let inner_info = type_to_wasm_info(inner_type, registry)?;

    let tag_offset = 0u32;
    let payload_offset = align_up(1, inner_info.alignment);
    let raw_size = payload_offset + inner_info.size;
    let alignment = inner_info.alignment.max(4); // Minimum 4-byte alignment
    let total_size = align_up(raw_size, alignment);

    Ok(OptionLayout {
        tag_offset,
        payload_offset,
        total_size,
        alignment,
        inner_wasm_type: inner_info.wasm_type,
        inner_size: inner_info.size,
    })
}

#[derive(Debug, Clone)]
pub struct OptionLayout {
    pub tag_offset: u32,
    pub payload_offset: u32,
    pub total_size: u32,
    pub alignment: u32,
    pub inner_wasm_type: WasmFieldType,
    pub inner_size: u32,
}
```

### 5.2 Some(expr) Compilation

```rust
// In emit_expression() for Expr::Some
Expr::Some(inner) => {
    // 1. Compute Option layout for inner type
    let inner_type = self.infer_type(inner, locals)?;
    let layout = option_layout(&inner_type, &self.gene_layouts)?;

    // 2. Allocate space for Option
    // Push size argument for alloc
    function.instruction(&Instruction::I32Const(layout.total_size as i32));
    function.instruction(&Instruction::Call(alloc_func_idx));

    // 3. Store pointer in temp local
    let temp = locals.lookup("__option_ptr").unwrap();
    function.instruction(&Instruction::LocalTee(temp));

    // 4. Write tag = 1 (Some)
    function.instruction(&Instruction::I32Const(1));
    function.instruction(&Instruction::I32Store8(MemArg {
        offset: layout.tag_offset as u64,
        align: 0,
        memory_index: 0,
    }));

    // 5. Write payload
    function.instruction(&Instruction::LocalGet(temp));
    self.emit_expression(function, inner, locals, loop_ctx)?;

    // 6. Store payload based on type
    let store_instr = match layout.inner_wasm_type {
        WasmFieldType::I32 => Instruction::I32Store(MemArg {
            offset: layout.payload_offset as u64,
            align: 2, // log2(4)
            memory_index: 0,
        }),
        WasmFieldType::I64 => Instruction::I64Store(MemArg {
            offset: layout.payload_offset as u64,
            align: 3, // log2(8)
            memory_index: 0,
        }),
        WasmFieldType::F32 => Instruction::F32Store(MemArg {
            offset: layout.payload_offset as u64,
            align: 2,
            memory_index: 0,
        }),
        WasmFieldType::F64 => Instruction::F64Store(MemArg {
            offset: layout.payload_offset as u64,
            align: 3,
            memory_index: 0,
        }),
        WasmFieldType::Ptr => Instruction::I32Store(MemArg {
            offset: layout.payload_offset as u64,
            align: 2,
            memory_index: 0,
        }),
    };
    function.instruction(&store_instr);

    // 7. Leave pointer on stack as result
    function.instruction(&Instruction::LocalGet(temp));

    Ok(())
}
```

### 5.3 None Compilation

```rust
// In emit_expression() for Expr::None
Expr::None => {
    // Type must be known from context (inferred from annotation)
    let option_type = self.get_expected_type()?;
    let inner_type = self.extract_option_inner(&option_type)?;
    let layout = option_layout(&inner_type, &self.gene_layouts)?;

    // 1. Allocate space for Option
    function.instruction(&Instruction::I32Const(layout.total_size as i32));
    function.instruction(&Instruction::Call(alloc_func_idx));

    // 2. Store pointer in temp
    let temp = locals.lookup("__option_ptr").unwrap();
    function.instruction(&Instruction::LocalTee(temp));

    // 3. Write tag = 0 (None)
    function.instruction(&Instruction::I32Const(0));
    function.instruction(&Instruction::I32Store8(MemArg {
        offset: layout.tag_offset as u64,
        align: 0,
        memory_index: 0,
    }));

    // 4. Leave pointer on stack
    function.instruction(&Instruction::LocalGet(temp));

    Ok(())
}
```

### 5.4 Pattern Matching on Option

```rust
// In emit_match_arms() - add Option pattern handling
fn emit_option_match(
    &self,
    function: &mut Function,
    scrutinee_ptr: u32,  // local holding Option pointer
    arms: &[MatchArm],
    locals: &LocalsTable,
    loop_ctx: LoopContext,
) -> Result<(), WasmError> {
    // Find Some and None arms
    let some_arm = arms.iter().find(|a| matches!(&a.pattern, Pattern::Some(_)));
    let none_arm = arms.iter().find(|a| matches!(&a.pattern, Pattern::None));

    // Load tag byte
    function.instruction(&Instruction::LocalGet(scrutinee_ptr));
    function.instruction(&Instruction::I32Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));

    // Branch on tag
    // if (tag != 0) { Some arm } else { None arm }
    function.instruction(&Instruction::If(BlockType::Result(result_type)));

    // === Some branch ===
    if let Some(arm) = some_arm {
        if let Pattern::Some(inner_pattern) = &arm.pattern {
            // Bind inner value if pattern is identifier
            if let Pattern::Identifier(name) = inner_pattern.as_ref() {
                // Load payload value
                function.instruction(&Instruction::LocalGet(scrutinee_ptr));
                let load_instr = self.make_load_instruction(layout)?;
                function.instruction(&load_instr);

                // Store in bound variable
                let local_idx = locals.lookup(name)
                    .ok_or_else(|| WasmError::new(format!("Unbound: {}", name)))?;
                function.instruction(&Instruction::LocalSet(local_idx));
            }

            // Emit arm body
            self.emit_expression(function, &arm.body, locals, loop_ctx)?;
        }
    } else {
        // No Some arm - unreachable
        function.instruction(&Instruction::Unreachable);
    }

    function.instruction(&Instruction::Else);

    // === None branch ===
    if let Some(arm) = none_arm {
        self.emit_expression(function, &arm.body, locals, loop_ctx)?;
    } else {
        // No None arm - unreachable
        function.instruction(&Instruction::Unreachable);
    }

    function.instruction(&Instruction::End);

    Ok(())
}
```

### 5.5 Method Compilation

#### is_some() / is_none()

```rust
// opt.is_some() - returns tag != 0
fn emit_is_some(&self, function: &mut Function, opt_ptr: u32) {
    function.instruction(&Instruction::LocalGet(opt_ptr));
    function.instruction(&Instruction::I32Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    // Result is already 0 or 1 (boolean)
}

// opt.is_none() - returns tag == 0
fn emit_is_none(&self, function: &mut Function, opt_ptr: u32) {
    function.instruction(&Instruction::LocalGet(opt_ptr));
    function.instruction(&Instruction::I32Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    function.instruction(&Instruction::I32Eqz);
}
```

#### unwrap()

```rust
// opt.unwrap() - trap if None, return inner value if Some
fn emit_unwrap(
    &self,
    function: &mut Function,
    opt_ptr: u32,
    layout: &OptionLayout,
) {
    // Check tag
    function.instruction(&Instruction::LocalGet(opt_ptr));
    function.instruction(&Instruction::I32Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));

    // If tag == 0, trap
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);  // Trap!
    function.instruction(&Instruction::End);

    // Load and return payload
    function.instruction(&Instruction::LocalGet(opt_ptr));
    let load_instr = match layout.inner_wasm_type {
        WasmFieldType::I32 | WasmFieldType::Ptr => Instruction::I32Load(MemArg {
            offset: layout.payload_offset as u64,
            align: 2,
            memory_index: 0,
        }),
        WasmFieldType::I64 => Instruction::I64Load(MemArg {
            offset: layout.payload_offset as u64,
            align: 3,
            memory_index: 0,
        }),
        WasmFieldType::F32 => Instruction::F32Load(MemArg {
            offset: layout.payload_offset as u64,
            align: 2,
            memory_index: 0,
        }),
        WasmFieldType::F64 => Instruction::F64Load(MemArg {
            offset: layout.payload_offset as u64,
            align: 3,
            memory_index: 0,
        }),
    };
    function.instruction(&load_instr);
}
```

#### unwrap_or(default)

```rust
// opt.unwrap_or(default) - return inner if Some, default if None
fn emit_unwrap_or(
    &self,
    function: &mut Function,
    opt_ptr: u32,
    default_expr: &Expr,
    layout: &OptionLayout,
    locals: &LocalsTable,
    loop_ctx: LoopContext,
) -> Result<(), WasmError> {
    // Check tag
    function.instruction(&Instruction::LocalGet(opt_ptr));
    function.instruction(&Instruction::I32Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));

    // if (tag != 0) { load payload } else { evaluate default }
    let result_type = layout.inner_wasm_type.to_val_type();
    function.instruction(&Instruction::If(BlockType::Result(result_type)));

    // Some case: load payload
    function.instruction(&Instruction::LocalGet(opt_ptr));
    let load_instr = self.make_load_instruction(layout)?;
    function.instruction(&load_instr);

    function.instruction(&Instruction::Else);

    // None case: evaluate default
    self.emit_expression(function, default_expr, locals, loop_ctx)?;

    function.instruction(&Instruction::End);

    Ok(())
}
```

## 6. Type Inference Rules

### 6.1 Some(expr) Inference

```
G |- expr : T
-----------------------
G |- Some(expr) : Option<T>
```

The type of `Some(expr)` is `Option<T>` where `T` is inferred from `expr`.

### 6.2 None Inference

```
G |- expected_type = Option<T>
-----------------------
G |- None : Option<T>
```

`None` requires type context from:
1. Explicit type annotation: `let x: Option<Int64> = None`
2. Function return type: `fun f() -> Option<Int64> { None }`
3. Function argument type: `process(None)` where `process(opt: Option<T>)`

### 6.3 Pattern Matching Type Refinement

```
G |- opt : Option<T>
G, x : T |- body_some : R
G |- body_none : R
----------------------------------
G |- match opt {
       Some(x) => body_some,
       None => body_none
     } : R
```

Within the `Some(x)` arm, `x` is bound with type `T`.

### 6.4 Implementation Sketch

```rust
impl TypeChecker {
    fn infer_type(&self, expr: &Expr, ctx: &TypeContext) -> Result<TypeExpr, TypeError> {
        match expr {
            Expr::Some(inner) => {
                let inner_type = self.infer_type(inner, ctx)?;
                Ok(TypeExpr::Generic {
                    name: "Option".to_string(),
                    args: vec![inner_type],
                })
            }

            Expr::None => {
                // Must have expected type from context
                match ctx.expected_type {
                    Some(TypeExpr::Generic { name, args }) if name == "Option" => {
                        Ok(TypeExpr::Generic {
                            name: "Option".to_string(),
                            args: args.clone(),
                        })
                    }
                    Some(expected) => Err(TypeError::TypeMismatch {
                        expected: "Option<T>".to_string(),
                        found: format!("{:?}", expected),
                    }),
                    None => Err(TypeError::CannotInferType {
                        message: "None requires type annotation".to_string(),
                    }),
                }
            }

            // ... other cases ...
        }
    }
}
```

## 7. Test Cases

### 7.1 Basic Construction

```dol
// Test: Some construction
fun test_some() -> Int64 {
    let opt = Some(42)
    match opt {
        Some(n) => n,
        None => 0
    }
}
// Expected: returns 42

// Test: None construction
fun test_none() -> Int64 {
    let opt: Option<Int64> = None
    match opt {
        Some(n) => n,
        None => -1
    }
}
// Expected: returns -1
```

### 7.2 Type Inference

```dol
// Test: Some infers inner type
fun infer_some() -> Option<Int64> {
    Some(100)  // Inferred as Option<Int64>
}

// Test: None requires annotation
fun needs_annotation() -> Option<Int64> {
    let x: Option<Int64> = None  // OK
    // let y = None  // ERROR: cannot infer type
    x
}
```

### 7.3 Pattern Matching

```dol
// Test: Basic pattern matching
fun match_option(opt: Option<Int64>) -> Int64 {
    match opt {
        Some(x) => x * 2,
        None => 0
    }
}

// Test: Nested patterns
fun match_nested(opt: Option<Option<Int64>>) -> Int64 {
    match opt {
        Some(Some(x)) => x,
        Some(None) => -1,
        None => -2
    }
}

// Test: Wildcard in Some
fun match_wildcard(opt: Option<Int64>) -> Bool {
    match opt {
        Some(_) => true,
        None => false
    }
}
```

### 7.4 Methods

```dol
// Test: is_some / is_none
fun test_predicates() -> Bool {
    let some_val = Some(42)
    let none_val: Option<Int64> = None

    some_val.is_some() & none_val.is_none()
}
// Expected: returns true

// Test: unwrap
fun test_unwrap() -> Int64 {
    let opt = Some(99)
    opt.unwrap()
}
// Expected: returns 99

// Test: unwrap_or
fun test_unwrap_or() -> Int64 {
    let some_opt = Some(10)
    let none_opt: Option<Int64> = None

    some_opt.unwrap_or(0) + none_opt.unwrap_or(5)
}
// Expected: returns 15
```

### 7.5 Different Inner Types

```dol
// Test: Option<Float64>
fun option_float() -> Float64 {
    let opt = Some(3.14)
    match opt {
        Some(x) => x,
        None => 0.0
    }
}

// Test: Option<Bool>
fun option_bool() -> Bool {
    let opt = Some(true)
    match opt {
        Some(b) => b,
        None => false
    }
}

// Test: Option<String> (pointer)
fun option_string() -> Int64 {
    let opt = Some("hello")
    match opt {
        Some(_) => 1,
        None => 0
    }
}
```

### 7.6 WASM Execution Tests

```rust
#[cfg(test)]
mod option_tests {
    use super::*;

    #[test]
    fn test_option_some_i64() {
        let source = r#"
            fun get_some() -> Int64 {
                let opt = Some(42)
                match opt {
                    Some(n) => n,
                    None => 0
                }
            }
        "#;

        let wasm = compile_to_wasm(source).unwrap();
        let result = execute_wasm(&wasm, "get_some", &[]);
        assert_eq!(result, Val::I64(42));
    }

    #[test]
    fn test_option_none_i64() {
        let source = r#"
            fun get_none() -> Int64 {
                let opt: Option<Int64> = None
                match opt {
                    Some(n) => n,
                    None => -1
                }
            }
        "#;

        let wasm = compile_to_wasm(source).unwrap();
        let result = execute_wasm(&wasm, "get_none", &[]);
        assert_eq!(result, Val::I64(-1));
    }

    #[test]
    fn test_option_unwrap_or() {
        let source = r#"
            fun test_unwrap_or() -> Int64 {
                let none: Option<Int64> = None
                none.unwrap_or(99)
            }
        "#;

        let wasm = compile_to_wasm(source).unwrap();
        let result = execute_wasm(&wasm, "test_unwrap_or", &[]);
        assert_eq!(result, Val::I64(99));
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn test_option_unwrap_none_traps() {
        let source = r#"
            fun unwrap_none() -> Int64 {
                let opt: Option<Int64> = None
                opt.unwrap()  // Should trap!
            }
        "#;

        let wasm = compile_to_wasm(source).unwrap();
        execute_wasm(&wasm, "unwrap_none", &[]);
    }

    #[test]
    fn test_option_is_some() {
        let source = r#"
            fun check_some() -> Int64 {
                let opt = Some(1)
                if opt.is_some() { 1 } else { 0 }
            }
        "#;

        let wasm = compile_to_wasm(source).unwrap();
        let result = execute_wasm(&wasm, "check_some", &[]);
        assert_eq!(result, Val::I64(1));
    }
}
```

## 8. Implementation Phases

### Phase 1: AST and Parser
1. Add `Some` and `None` tokens to lexer
2. Add `Expr::Some` and `Expr::None` variants
3. Add `Pattern::Some` and `Pattern::None` variants
4. Update parser to handle Some/None expressions and patterns

### Phase 2: Layout
1. Add `OptionLayout` struct to layout.rs
2. Implement `option_layout()` function
3. Register Option type in `type_to_wasm_info`

### Phase 3: Basic Codegen
1. Implement `emit_some()` for Some expression compilation
2. Implement `emit_none()` for None expression compilation
3. Add Option pattern matching support in `emit_match_arms()`

### Phase 4: Methods
1. Implement `is_some()` method compilation
2. Implement `is_none()` method compilation
3. Implement `unwrap()` with trap on None
4. Implement `unwrap_or()` with default value

### Phase 5: Type Inference
1. Add Option type inference for Some
2. Add context-based inference for None
3. Add pattern binding type refinement

### Phase 6: Testing
1. Unit tests for AST construction
2. Parser tests for Some/None syntax
3. Layout computation tests
4. WASM execution tests for all cases

## 9. Open Questions and Decisions

### Q1: None Type Inference
**Decision**: None requires type context. Type annotations, expected types from function signatures, or generic constraints provide the necessary type information.

### Q2: Nested Option Layout
**Decision**: Use inline layout where the inner Option is embedded directly (not via pointer). This uses more memory but avoids double indirection.

### Q3: Zero-size Optimization for Unit
**Decision**: Defer. `Option<()>` could theoretically be 1 byte (just the tag), but for simplicity, use standard layout rules.

### Q4: Pattern Exhaustiveness
**Decision**: The compiler should warn if Option patterns are not exhaustive (missing Some or None arm) unless a wildcard is present.

### Q5: Method Resolution
**Decision**: Option methods (is_some, unwrap, etc.) are built-in and handled specially in the compiler, not via trait dispatch.

## 10. Future Extensions

1. **map/and_then/or_else**: Functional combinators for Option transformation
2. **? operator**: Early return on None (`value?` desugars to match + early return)
3. **Nullable types**: Interop with languages that use null pointers
4. **Result<T, E>**: Similar tagged union for error handling
5. **Exhaustiveness checker**: Warn on non-exhaustive pattern matches

---

*Document Version: 1.0*
*Author: Type System Architect*
*Date: 2026-01-02*

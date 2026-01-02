# Static Methods via Self-Presence Inference

**Philosophy Alignment**: No new keywords. Compiler infers intent.

## The Problem

ENR specs need static constructors:
- `Credits::zero()`
- `EntropyWeights::default()`
- `Duration::seconds(s)`

## The Wrong Solution ❌

Adding a `provides` keyword:
```dol
gene Credits {
    provides zero() -> Self { ... }  // BAD: Forces user annotation
}
```

This violates DOL philosophy:
- Users shouldn't annotate what compiler can infer
- We're removing `sex` from user-facing code, not adding keywords

## The Right Solution ✅

**Infer from self-presence** - no keyword needed:

```dol
gene Credits {
    has amount: UInt64
    
    // Static method: NO self parameter
    fun zero() -> Credits {
        Credits { amount: 0 }
    }
    
    // Static method: NO self parameter, has other params
    fun from_amount(amt: UInt64) -> Credits {
        Credits { amount: amt }
    }
    
    // Instance method: HAS self parameter
    fun double(self) -> Credits {
        Credits { amount: self.amount * 2 }
    }
    
    // Instance method: HAS self, mutates
    fun increment(self) {
        self.amount = self.amount + 1
    }
}
```

## Call Syntax

```dol
// Static calls use :: (type namespace)
let c = Credits::zero()
let d = Credits::from_amount(100)

// Instance calls use . (value namespace)
let e = c.double()
c.increment()
```

## Compiler Implementation

### 1. AST (No Changes to Gene Methods)

```rust
// src/ast.rs
pub struct GeneMethod {
    pub name: String,
    pub params: Vec<Param>,      // First param may or may not be "self"
    pub return_type: Option<Type>,
    pub body: Expr,
}

impl GeneMethod {
    /// Infer if this is a static method (no self parameter)
    pub fn is_static(&self) -> bool {
        self.params.first().map(|p| p.name != "self").unwrap_or(true)
    }
}
```

### 2. Parser (Minimal Changes)

```rust
// src/parser.rs
fn parse_gene_method(&mut self) -> Result<GeneMethod> {
    self.expect(TokenKind::Fun)?;
    let name = self.expect_ident()?;
    let params = self.parse_params()?;
    let return_type = self.parse_optional_return_type()?;
    let body = self.parse_block()?;
    
    // No special handling needed - just parse the params normally
    // is_static() will be inferred from param list
    
    Ok(GeneMethod { name, params, return_type, body })
}
```

### 3. WASM Codegen

```rust
// src/wasm/compiler.rs
fn compile_gene(&mut self, gene: &Gene) -> Result<()> {
    // Compute layout for the gene
    let layout = self.compute_gene_layout(gene);
    
    for method in &gene.methods {
        if method.is_static() {
            self.compile_static_method(&gene.name, method)?;
        } else {
            self.compile_instance_method(&gene.name, method, &layout)?;
        }
    }
    
    Ok(())
}

fn compile_static_method(&mut self, gene: &str, method: &GeneMethod) -> Result<()> {
    // Function name: Gene_methodName (exported)
    let func_name = format!("{}_{}", gene, method.name);
    
    // No implicit self - params are exactly as declared
    let params: Vec<_> = method.params.iter()
        .map(|p| self.type_to_valtype(&p.ty))
        .collect();
    
    let return_type = method.return_type.as_ref()
        .map(|t| self.type_to_valtype(t));
    
    self.begin_function(&func_name, &params, return_type);
    
    // Compile body
    self.compile_expr(&method.body)?;
    
    self.end_function();
    self.export_function(&func_name);
    
    Ok(())
}

fn compile_instance_method(&mut self, gene: &str, method: &GeneMethod, layout: &GeneLayout) -> Result<()> {
    // Function name: Gene.methodName (exported)
    let func_name = format!("{}.{}", gene, method.name);
    
    // First param is self (i32 pointer)
    let mut params = vec![ValType::I32]; // self pointer
    
    // Rest of params
    for param in method.params.iter().skip(1) { // skip "self"
        params.push(self.type_to_valtype(&param.ty));
    }
    
    let return_type = method.return_type.as_ref()
        .map(|t| self.type_to_valtype(t));
    
    self.begin_function(&func_name, &params, return_type);
    
    // self is local 0
    self.locals.bind("self", 0);
    
    // Compile body
    self.compile_expr(&method.body)?;
    
    self.end_function();
    self.export_function(&func_name);
    
    Ok(())
}
```

### 4. Call Site Compilation

```rust
fn compile_static_call(&mut self, gene: &str, method: &str, args: &[Expr]) -> Result<()> {
    // Compile arguments
    for arg in args {
        self.compile_expr(arg)?;
    }
    
    // Call Gene_method
    let func_name = format!("{}_{}", gene, method);
    let func_idx = self.get_func_index(&func_name)?;
    self.emit(Instruction::Call(func_idx));
    
    Ok(())
}

fn compile_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<()> {
    // Compile receiver (self pointer)
    self.compile_expr(receiver)?;
    
    // Compile additional arguments
    for arg in args {
        self.compile_expr(arg)?;
    }
    
    // Determine gene type from receiver
    let gene = self.infer_type(receiver)?.as_gene()?;
    
    // Call Gene.method
    let func_name = format!("{}.{}", gene, method);
    let func_idx = self.get_func_index(&func_name)?;
    self.emit(Instruction::Call(func_idx));
    
    Ok(())
}
```

## ENR Spec Compatibility

The ENR specs use `provides` syntax which we need to support during parsing but treat as regular methods:

```dol
// ENR syntax (backwards compat during transition)
gene Credits {
    has amount: UInt64
    
    provides zero() -> Self {
        Credits { amount: 0 }
    }
}

// Equivalent to (preferred going forward)
gene Credits {
    has amount: UInt64
    
    fun zero() -> Credits {
        Credits { amount: 0 }
    }
}
```

**Parser approach**: Treat `provides` as alias for `fun` (same AST), compiler still infers from self-presence.

```rust
fn parse_gene_body(&mut self) -> Result<Vec<GeneItem>> {
    let mut items = vec![];
    
    loop {
        match self.peek().kind {
            TokenKind::Has => items.push(self.parse_has_field()?),
            TokenKind::Fun | TokenKind::Provides => {
                // Both keywords produce the same AST
                self.advance(); // consume fun/provides
                items.push(GeneItem::Method(self.parse_method_body()?));
            }
            TokenKind::Is => items.push(self.parse_is_method()?),
            _ => break,
        }
    }
    
    Ok(items)
}
```

## Test Cases

### test-cases/level5/static_methods.dol
```dol
gene Point {
    has x: Int
    has y: Int
    
    // Static: no self
    fun origin() -> Point {
        Point { x: 0, y: 0 }
    }
    
    fun new(x: Int, y: Int) -> Point {
        Point { x: x, y: y }
    }
    
    // Instance: has self
    fun magnitude_squared(self) -> Int {
        self.x * self.x + self.y * self.y
    }
    
    fun translate(self, dx: Int, dy: Int) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}

fun test_static_methods() -> Int {
    let origin = Point::origin()
    let p = Point::new(3, 4)
    
    // origin.magnitude_squared() = 0
    // p.magnitude_squared() = 25
    origin.magnitude_squared() + p.magnitude_squared()
}

// Expected: 25
```

### test-cases/level5/credits_static.dol
```dol
// ENR-style Credits gene
gene Credits {
    has amount: UInt64
    
    fun zero() -> Credits {
        Credits { amount: 0 }
    }
    
    fun from_amount(amt: UInt64) -> Credits {
        Credits { amount: amt }
    }
    
    fun add(self, other: Credits) -> Credits {
        Credits { amount: self.amount + other.amount }
    }
}

fun test_credits() -> UInt64 {
    let a = Credits::zero()
    let b = Credits::from_amount(100)
    let c = a.add(b)
    
    c.amount
}

// Expected: 100
```

## Migration Path

1. **Phase 1**: Parser accepts both `fun` and `provides` for methods
2. **Phase 2**: Compiler infers static/instance from self-presence
3. **Phase 3**: Update ENR specs to use `fun` instead of `provides`
4. **Phase 4**: Deprecate `provides` keyword (warning)
5. **Phase 5**: Remove `provides` from parser

## Summary

| Aspect | Old Approach | New Approach |
|--------|--------------|--------------|
| Keyword | `provides` | None (inference) |
| User annotation | Required | Not needed |
| Philosophy | Violates | Aligns |
| Static detection | Explicit keyword | Self-presence |
| Complexity | Higher (new AST) | Lower (inference) |
| ENR compat | Direct | Via parser alias |

The compiler does the work, not the user. 🍄
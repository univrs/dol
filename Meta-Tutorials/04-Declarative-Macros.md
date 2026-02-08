# Tutorial 04: Declarative Macros

> **Pattern matching and hygienic macro expansion**
>
> **Level**: Intermediate | **Time**: 55 minutes | **Lines**: 140+

## Overview

DOL's declarative macro system provides compile-time code generation through pattern matching and template expansion.

## Built-in Macros

### #derive - Trait Implementation

```dol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
gen User {
    has id: string
    has name: string
}

// Expands to Rust trait implementations automatically
```

### #stringify - Code to String

```dol
gen Example {
    fun test() {
        let name = #stringify(Example)
        println("Type name: " + name)  // "Example"
    }
}
```

### #concat - String Concatenation

```dol
const TABLE_NAME: string = #concat("users_", VERSION, "_table")
// Expands to: "users_v1_table"
```

### #env - Environment Variables

```dol
const DATABASE_URL: string = #env("DATABASE_URL")
const DEBUG_MODE: Bool = #env("DEBUG") == "true"
```

### #cfg - Conditional Compilation

```dol
#[cfg(target = "wasm")]
gen WasmFeature {
    has wasm_specific: Int
}

#[cfg(target = "native")]
gen NativeFeature {
    has native_specific: Int
}
```

## Custom Macro Example

```rust
// custom_macros.rs
use metadol::macros::{Macro, MacroInput, MacroOutput, MacroError, MacroContext};
use metadol::ast::{Expr, Literal};

pub struct TimestampMacro;

impl Macro for TimestampMacro {
    fn name(&self) -> &str {
        "timestamp"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext)
        -> Result<MacroOutput, MacroError> {
        // Generate current timestamp at compile time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(MacroOutput::Expr(Box::new(
            Expr::Literal(Literal::Int(now as i64))
        )))
    }
}

pub struct ValidatorMacro;

impl Macro for ValidatorMacro {
    fn name(&self) -> &str {
        "validator"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext)
        -> Result<MacroOutput, MacroError> {
        // Input: field name and constraint
        let expr = input.as_expr()
            .ok_or_else(|| MacroError::invalid_argument("expected expression"))?;

        // Generate validation function
        Ok(MacroOutput::Function(Box::new(
            // ... generate validation logic
        )))
    }
}
```

## Using Custom Macros

```dol
// Register custom macros
use custom_macros::{TimestampMacro, ValidatorMacro}

gen Record {
    has created_at: Int = #timestamp()

    #[validator(min=0, max=100)]
    has score: Int
}

docs {
    Uses custom #timestamp macro to set creation time at compile time.
    Uses #validator to generate constraint checking.
}
```

## Macro Hygiene

```dol
// ❌ Wrong: Variable capture
#macro bad_macro(x) {
    let temp = x
    temp + temp
}

let temp = 5
let result = bad_macro(10)  // Captures outer 'temp'!

// ✅ Correct: Hygienic macro
#macro good_macro(x) {
    {
        let __temp = x  // Unique name
        __temp + __temp
    }
}
```

## Pattern Matching Macros

```rust
pub struct MatchMacro;

impl Macro for MatchMacro {
    fn expand(&self, input: MacroInput, ctx: &MacroContext)
        -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::Expr(e) => {
                // Match on expression type
                match *e {
                    Expr::Literal(Literal::Int(n)) => {
                        // Generate code for integer
                    }
                    Expr::Identifier(ref name) => {
                        // Generate code for identifier
                    }
                    _ => return Err(MacroError::type_error("int or ident", "other"))
                }
            }
            _ => return Err(MacroError::invalid_argument("expected expression"))
        }
    }
}
```

## Complete Example: Builder Pattern Macro

```rust
// builder_macro.rs
pub struct BuilderMacro;

impl Macro for BuilderMacro {
    fn name(&self) -> &str { "derive_builder" }

    fn expand(&self, input: MacroInput, ctx: &MacroContext)
        -> Result<MacroOutput, MacroError> {
        let decl = input.as_declaration()
            .ok_or_else(|| MacroError::invalid_argument("expected declaration"))?;

        if let Declaration::Gene(gen) = &**decl {
            let builder_name = format!("{}Builder", gen.name);

            // Generate builder struct and methods
            let builder_gen = Gen {
                name: builder_name,
                fields: gen.statements.iter()
                    .filter_map(|s| match s {
                        Statement::HasField(f) => Some(f.clone()),
                        _ => None
                    })
                    .map(|f| {
                        // Make fields optional in builder
                        HasField {
                            type_: TypeExpr::Option(Box::new(f.type_)),
                            default: Some(Expr::Literal(Literal::Null)),
                            ..f
                        }
                    })
                    .collect(),
                // ... generate with_* methods
            };

            Ok(MacroOutput::Declaration(Box::new(
                Declaration::Gene(builder_gen)
            )))
        } else {
            Err(MacroError::type_error("gene", "other"))
        }
    }
}
```

Usage:

```dol
#[derive_builder]
gen User {
    has id: string
    has name: string
    has email: string
}

// Expands to:

gen UserBuilder {
    has id: Option<string> = None
    has name: Option<string> = None
    has email: Option<string> = None

    fun with_id(mut self, id: string) -> UserBuilder {
        self.id = Some(id)
        return self
    }

    fun with_name(mut self, name: string) -> UserBuilder {
        self.name = Some(name)
        return self
    }

    fun with_email(mut self, email: string) -> UserBuilder {
        self.email = Some(email)
        return self
    }

    fun build(self) -> Result<User, string> {
        if self.id.is_none() {
            return Err("id is required")
        }
        // ... validate all required fields

        return Ok(User {
            id: self.id.unwrap(),
            name: self.name.unwrap_or(""),
            email: self.email.unwrap_or("")
        })
    }
}

// Usage:
let user = UserBuilder::new()
    .with_id("123")
    .with_name("Alice")
    .with_email("alice@example.com")
    .build()
    .unwrap()
```

## Common Pitfalls

### Pitfall 1: Infinite Recursion

```dol
// ❌ Wrong: Macro calls itself
#macro recursive(x) {
    recursive(x + 1)  // Infinite!
}

// ✅ Correct: Base case
#macro recursive(x) {
    if x > 100 {
        return x
    } else {
        return recursive(x + 1)
    }
}
```

### Pitfall 2: Type Mismatch

```rust
// ❌ Wrong: Generating incompatible types
fn expand(&self, input: MacroInput, ctx: &MacroContext)
    -> Result<MacroOutput, MacroError> {
    Ok(MacroOutput::Expr(
        Box::new(Expr::Literal(Literal::String("test".into())))
    ))
    // But context expects Int!
}

// ✅ Correct: Check expected type
fn expand(&self, input: MacroInput, ctx: &MacroContext)
    -> Result<MacroOutput, MacroError> {
    match ctx.expected_type() {
        Some(Type::String) => {
            Ok(MacroOutput::Expr(
                Box::new(Expr::Literal(Literal::String("test".into())))
            ))
        }
        Some(Type::I64) => {
            Ok(MacroOutput::Expr(
                Box::new(Expr::Literal(Literal::Int(42)))
            ))
        }
        _ => Err(MacroError::type_error("string or int", "unknown"))
    }
}
```

## Performance Tips

1. **Cache macro expansions** for identical inputs
2. **Avoid heavy computation** in macros (use build scripts instead)
3. **Limit macro depth** to prevent stack overflow

---

**Next**: [Tutorial 05: Procedural Macros](./05-Procedural-Macros.md)

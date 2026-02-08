# Tutorial 05: Procedural Macros

> **Advanced code transformation with derive, attribute, and function-like macros**
>
> **Level**: Advanced | **Time**: 65 minutes | **Lines**: 160+

## Overview

Procedural macros enable sophisticated code generation through:
- **Derive macros**: Automatic trait implementation
- **Attribute macros**: Code transformation and decoration
- **Function-like macros**: Domain-specific languages (DSLs)

## Derive Macros

### #[derive(Gen)] - CRDT Schema Generation

```rust
// dol_macro_proc/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Gen)]
pub fn derive_gen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn to_dol(&self) -> String {
                format!("gen {} {{ ... }}", stringify!(#name))
            }

            pub fn from_dol(source: &str) -> Result<Self, ParseError> {
                // Parse DOL and construct instance
                metadol::parse_and_construct(source)
            }
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```rust
#[derive(Gen, Debug, Clone)]
struct UserProfile {
    id: String,
    name: String,
    age: u32,
}

fn main() {
    let user = UserProfile {
        id: "123".to_string(),
        name: "Alice".to_string(),
        age: 30,
    };

    println!("{}", user.to_dol());
    // gen UserProfile {
    //   has id: string
    //   has name: string
    //   has age: Int
    // }
}
```

### #[derive(Crdt)] - Auto CRDT Implementation

```rust
#[proc_macro_derive(Crdt, attributes(crdt))]
pub fn derive_crdt(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Crdt can only be derived for structs"),
    };

    let merge_implementations: Vec<_> = fields.iter().map(|field| {
        let name = &field.ident;

        // Extract #[crdt(strategy)] attribute
        let strategy = field.attrs.iter()
            .find(|attr| attr.path().is_ident("crdt"))
            .and_then(|attr| {
                attr.parse_args::<syn::Ident>().ok()
            })
            .map(|ident| ident.to_string())
            .unwrap_or_else(|| "lww".to_string());

        match strategy.as_str() {
            "immutable" => quote! {
                // Immutable: keep first write
                if self.#name.is_none() {
                    self.#name = other.#name.clone();
                }
            },
            "lww" => quote! {
                // Last-write-wins: use timestamp
                if other.timestamp > self.timestamp {
                    self.#name = other.#name.clone();
                }
            },
            "or_set" => quote! {
                // OR-Set: union of elements
                self.#name.extend(other.#name.iter().cloned());
            },
            "pn_counter" => quote! {
                // PN Counter: add increments
                self.#name += other.#name;
            },
            _ => quote! {},
        }
    }).collect();

    let name = &input.ident;
    let expanded = quote! {
        impl Crdt for #name {
            fn merge(&mut self, other: &Self) {
                #(#merge_implementations)*
            }
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```rust
#[derive(Crdt, Clone)]
struct ChatMessage {
    #[crdt(immutable)]
    id: String,

    #[crdt(lww)]
    content: String,

    #[crdt(or_set)]
    reactions: HashSet<String>,

    timestamp: u64,
}

fn main() {
    let mut msg1 = ChatMessage {
        id: "1".into(),
        content: "Hello".into(),
        reactions: HashSet::from(["👍".into()]),
        timestamp: 100,
    };

    let msg2 = ChatMessage {
        id: "1".into(),
        content: "Hello World".into(),
        reactions: HashSet::from(["❤️".into()]),
        timestamp: 200,
    };

    msg1.merge(&msg2);
    // Result: content = "Hello World" (LWW), reactions = {"👍", "❤️"} (OR-Set)
}
```

## Attribute Macros

### #[cached] - Memoization

```rust
#[proc_macro_attribute]
pub fn cached(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as syn::ItemFn);
    let name = &func.sig.ident;
    let inputs = &func.sig.inputs;
    let output = &func.sig.output;
    let body = &func.block;

    let cache_name = syn::Ident::new(
        &format!("__CACHE_{}", name.to_string().to_uppercase()),
        name.span()
    );

    let expanded = quote! {
        use std::sync::LazyLock;
        use std::collections::HashMap;
        use std::sync::Mutex;

        static #cache_name: LazyLock<Mutex<HashMap<String, _>>> =
            LazyLock::new(|| Mutex::new(HashMap::new()));

        fn #name(#inputs) #output {
            let key = format!("{:?}", (#inputs));

            let mut cache = #cache_name.lock().unwrap();
            if let Some(result) = cache.get(&key) {
                return result.clone();
            }

            let result = {
                #body
            };

            cache.insert(key, result.clone());
            result
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```dol
gen Calculator {
    #[cached]
    fun fibonacci(n: Int) -> Int {
        if n <= 1 {
            return n
        }
        return fibonacci(n - 1) + fibonacci(n - 2)
    }
}

// First call: computes
let result1 = calculator.fibonacci(40)  // Takes time

// Second call: cached
let result2 = calculator.fibonacci(40)  // Instant!
```

### #[async_method] - Async Transformation

```rust
#[proc_macro_attribute]
pub fn async_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as syn::ItemFn);

    // Transform to async and wrap returns with await
    let expanded = quote! {
        async fn #func.sig.ident(#func.sig.inputs) #func.sig.output {
            // Insert .await for blocking calls
            #func.block
        }
    };

    TokenStream::from(expanded)
}
```

### #[wasm_export] - WASM Bindings

```rust
#[proc_macro_attribute]
pub fn wasm_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as syn::ItemFn);
    let name = &func.sig.ident;

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn #name(#func.sig.inputs) #func.sig.output {
            #func.block
        }

        // Generate TypeScript bindings
        const _: () = {
            const BINDING: &str = concat!(
                "export function ", stringify!(#name), "(): ... { ... }"
            );
        };
    };

    TokenStream::from(expanded)
}
```

Usage:

```dol
gen WasmService {
    #[wasm_export]
    fun process_data(data: string) -> string {
        return data.to_uppercase()
    }
}

// Automatically exports:
// - WASM function
// - TypeScript binding
```

## Function-like Macros

### sql! - Type-Safe SQL

```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let sql_query = input.to_string();

    // Parse SQL and generate type-safe query
    let parsed = parse_sql(&sql_query);

    let expanded = quote! {
        {
            struct Query;
            impl Query {
                fn execute(&self) -> QueryResult {
                    // Execute with runtime
                    execute_sql(#sql_query)
                }
            }
            Query
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```dol
gen UserRepository {
    fun find_by_email(email: string) -> Option<User> {
        let result = sql!("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_one()

        return result.map(|row| User::from_row(row))
    }
}
```

### json! - JSON Literal

```rust
#[proc_macro]
pub fn json(input: TokenStream) -> TokenStream {
    let json_str = input.to_string();

    // Parse and validate JSON at compile time
    let _: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Invalid JSON");

    let expanded = quote! {
        serde_json::json!(#json_str)
    };

    TokenStream::from(expanded)
}
```

Usage:

```dol
gen ApiClient {
    fun create_user(name: string) -> Response {
        let body = json!({
            "name": name,
            "created_at": timestamp()
        })

        return post("/users", body)
    }
}
```

### regex! - Compiled Regex

```rust
#[proc_macro]
pub fn regex(input: TokenStream) -> TokenStream {
    let pattern = input.to_string();

    // Validate regex at compile time
    regex::Regex::new(&pattern)
        .expect("Invalid regex pattern");

    let expanded = quote! {
        {
            use std::sync::LazyLock;
            static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
                regex::Regex::new(#pattern).unwrap()
            });
            &*RE
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```dol
gen Validator {
    fun is_valid_email(email: string) -> Bool {
        let email_regex = regex!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        return email_regex.is_match(email)
    }
}
```

## Complete Example: ORM Macro

```rust
// orm_macro.rs
#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract table name from attribute
    let table_name = extract_table_name(&input.attrs)
        .unwrap_or_else(|| name.to_string().to_lowercase());

    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Table can only be derived for structs"),
    };

    // Generate column mappings
    let column_defs: Vec<_> = fields.iter().map(|field| {
        let field_name = &field.ident;
        let column_name = extract_column_name(&field.attrs)
            .unwrap_or_else(|| field_name.to_string());

        quote! {
            (#column_name, stringify!(#field_name))
        }
    }).collect();

    let expanded = quote! {
        impl Table for #name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> Vec<(&'static str, &'static str)> {
                vec![#(#column_defs),*]
            }

            fn insert(&self) -> String {
                format!("INSERT INTO {} (...) VALUES (...)", Self::table_name())
            }

            fn update(&self) -> String {
                format!("UPDATE {} SET ... WHERE ...", Self::table_name())
            }

            fn delete(&self) -> String {
                format!("DELETE FROM {} WHERE ...", Self::table_name())
            }
        }
    };

    TokenStream::from(expanded)
}
```

Usage:

```rust
#[derive(Table, Clone)]
#[table(name = "users")]
struct User {
    #[column(name = "user_id", primary_key)]
    id: String,

    #[column(name = "user_name")]
    name: String,

    #[column(name = "email_address", unique)]
    email: String,
}

fn main() {
    let user = User {
        id: "123".into(),
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    println!("{}", user.insert());
    // INSERT INTO users (user_id, user_name, email_address)
    // VALUES ('123', 'Alice', 'alice@example.com')
}
```

## Common Pitfalls

### Pitfall 1: Hygiene Violations

```rust
// ❌ Wrong: User identifiers can clash
#[proc_macro]
pub fn bad_macro(input: TokenStream) -> TokenStream {
    quote! {
        let result = process(#input);  // 'result' might clash!
        result
    }.into()
}

// ✅ Correct: Use unique identifiers
#[proc_macro]
pub fn good_macro(input: TokenStream) -> TokenStream {
    let temp_var = syn::Ident::new(
        &format!("__temp_{}", random_suffix()),
        proc_macro2::Span::call_site()
    );

    quote! {
        let #temp_var = process(#input);
        #temp_var
    }.into()
}
```

### Pitfall 2: Span Information Loss

```rust
// ❌ Wrong: Loses original span
let ident = syn::Ident::new("field", Span::call_site());

// ✅ Correct: Preserve original span
let ident = syn::Ident::new("field", original_ident.span());
```

## Performance Tips

1. **Minimize generated code** - don't inline large functions
2. **Use `LazyLock`** for compile-time constants
3. **Cache parsed AST** when possible

---

**Next**: [Tutorial 06: DOL to WASM Pipeline](./06-DOL-to-WASM-Complete-Pipeline.md)

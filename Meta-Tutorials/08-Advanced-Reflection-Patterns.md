# Tutorial 08: Advanced Reflection Patterns

> **Meta-programming techniques with runtime type introspection**
>
> **Level**: Expert | **Time**: 65 minutes | **Lines**: 160+

## Overview

Advanced reflection enables sophisticated meta-programming:
- Generic programming with TypeInfo
- Type-safe serialization
- Runtime schema evolution
- Self-modifying programs
- Dynamic dispatch systems

## Pattern 1: Generic Serializer (50 lines)

```rust
use metadol::reflect::{TypeRegistry, TypeInfo, TypeKind, FieldInfo};
use serde_json::Value;

/// Generic serializer using reflection
pub struct ReflectSerializer {
    registry: TypeRegistry,
}

impl ReflectSerializer {
    pub fn new(registry: TypeRegistry) -> Self {
        Self { registry }
    }

    /// Serialize any type to JSON using reflection
    pub fn serialize<T>(&self, value: &T, type_name: &str) -> Result<Value, String> {
        let type_info = self.registry.lookup(type_name)
            .ok_or_else(|| format!("Unknown type: {}", type_name))?;

        match type_info.kind() {
            TypeKind::Primitive => self.serialize_primitive(value, type_info),
            TypeKind::Record => self.serialize_record(value, type_info),
            TypeKind::Enum => self.serialize_enum(value, type_info),
            _ => Err(format!("Unsupported type kind: {:?}", type_info.kind()))
        }
    }

    fn serialize_record<T>(&self, value: &T, type_info: &TypeInfo)
        -> Result<Value, String> {
        let mut map = serde_json::Map::new();

        for field in type_info.fields() {
            let field_value = self.get_field_value(value, field)?;
            let field_json = self.serialize(&field_value, field.type_name())?;
            map.insert(field.name().to_string(), field_json);
        }

        Ok(Value::Object(map))
    }

    fn serialize_enum<T>(&self, value: &T, type_info: &TypeInfo)
        -> Result<Value, String> {
        // Use std::mem::discriminant for enum variant
        let variant_name = self.get_enum_variant_name(value)?;
        Ok(Value::String(variant_name))
    }

    fn serialize_primitive<T>(&self, value: &T, type_info: &TypeInfo)
        -> Result<Value, String> {
        match type_info.name() {
            "Int32" | "Int64" => {
                let num = unsafe { *(value as *const T as *const i64) };
                Ok(Value::Number(num.into()))
            }
            "String" => {
                let s = unsafe { &*(value as *const T as *const String) };
                Ok(Value::String(s.clone()))
            }
            "Bool" => {
                let b = unsafe { *(value as *const T as *const bool) };
                Ok(Value::Bool(b))
            }
            _ => Err(format!("Unknown primitive: {}", type_info.name()))
        }
    }

    fn get_field_value<T>(&self, value: &T, field: &FieldInfo)
        -> Result<Box<dyn std::any::Any>, String> {
        // Use field offset to extract value
        // In real implementation, use proc macros or codegen
        unimplemented!("Field value extraction requires proc macros")
    }

    fn get_enum_variant_name<T>(&self, value: &T) -> Result<String, String> {
        // Use std::mem::discriminant
        unimplemented!("Enum introspection")
    }
}

// Usage:
fn example() {
    let mut registry = TypeRegistry::with_primitives();

    // Register User type
    let user_type = TypeInfo::record("User")
        .with_field(FieldInfo::new("id", "String"))
        .with_field(FieldInfo::new("name", "String"))
        .with_field(FieldInfo::new("age", "Int32"));

    registry.register(user_type);

    let serializer = ReflectSerializer::new(registry);

    struct User {
        id: String,
        name: String,
        age: i32,
    }

    let user = User {
        id: "123".into(),
        name: "Alice".into(),
        age: 30,
    };

    let json = serializer.serialize(&user, "User").unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
```

## Pattern 2: Schema Evolution (40 lines)

```rust
use metadol::reflect::{TypeRegistry, TypeInfo, FieldInfo};

/// Schema migration system
pub struct SchemaMigration {
    from_registry: TypeRegistry,
    to_registry: TypeRegistry,
}

impl SchemaMigration {
    pub fn new(from: TypeRegistry, to: TypeRegistry) -> Self {
        Self {
            from_registry: from,
            to_registry: to,
        }
    }

    /// Generate migration plan
    pub fn plan(&self, type_name: &str) -> MigrationPlan {
        let from_type = self.from_registry.lookup(type_name);
        let to_type = self.to_registry.lookup(type_name);

        match (from_type, to_type) {
            (Some(from), Some(to)) => self.diff_types(from, to),
            (Some(_), None) => MigrationPlan::TypeRemoved,
            (None, Some(_)) => MigrationPlan::TypeAdded,
            (None, None) => MigrationPlan::NoChange,
        }
    }

    fn diff_types(&self, from: &TypeInfo, to: &TypeInfo) -> MigrationPlan {
        let mut plan = MigrationPlan::default();

        // Find added fields
        for field in to.fields() {
            if from.field(field.name()).is_none() {
                plan.added_fields.push(field.clone());
            }
        }

        // Find removed fields
        for field in from.fields() {
            if to.field(field.name()).is_none() {
                plan.removed_fields.push(field.clone());
            }
        }

        // Find type changes
        for field in to.fields() {
            if let Some(old_field) = from.field(field.name()) {
                if old_field.type_name() != field.type_name() {
                    plan.changed_fields.push((old_field.clone(), field.clone()));
                }
            }
        }

        plan
    }

    /// Apply migration to data
    pub fn migrate(&self, data: &mut serde_json::Value, type_name: &str)
        -> Result<(), String> {
        let plan = self.plan(type_name);

        match data {
            Value::Object(ref mut map) => {
                // Add new fields with defaults
                for field in &plan.added_fields {
                    if let Some(default) = field.default() {
                        map.insert(
                            field.name().to_string(),
                            serde_json::from_str(default).unwrap_or(Value::Null)
                        );
                    }
                }

                // Remove old fields
                for field in &plan.removed_fields {
                    map.remove(field.name());
                }

                // Transform changed fields
                for (old_field, new_field) in &plan.changed_fields {
                    if let Some(value) = map.get(old_field.name()) {
                        let converted = self.convert_type(
                            value,
                            old_field.type_name(),
                            new_field.type_name()
                        )?;
                        map.insert(new_field.name().to_string(), converted);
                    }
                }

                Ok(())
            }
            _ => Err("Expected object".into())
        }
    }

    fn convert_type(&self, value: &Value, from_type: &str, to_type: &str)
        -> Result<Value, String> {
        // Type conversion logic
        match (from_type, to_type) {
            ("Int32", "String") => {
                let n = value.as_i64().ok_or("Not an int")?;
                Ok(Value::String(n.to_string()))
            }
            ("String", "Int32") => {
                let s = value.as_str().ok_or("Not a string")?;
                let n = s.parse::<i64>().map_err(|e| e.to_string())?;
                Ok(Value::Number(n.into()))
            }
            _ if from_type == to_type => Ok(value.clone()),
            _ => Err(format!("Cannot convert {} to {}", from_type, to_type))
        }
    }
}

#[derive(Debug, Default)]
pub struct MigrationPlan {
    added_fields: Vec<FieldInfo>,
    removed_fields: Vec<FieldInfo>,
    changed_fields: Vec<(FieldInfo, FieldInfo)>,
}

// Usage:
fn migrate_schema() {
    let old_registry = build_v1_registry();
    let new_registry = build_v2_registry();

    let migration = SchemaMigration::new(old_registry, new_registry);

    let mut data = serde_json::json!({
        "id": "123",
        "name": "Alice",
        // Missing "email" field added in v2
    });

    migration.migrate(&mut data, "User").unwrap();

    // Now has default email
    assert!(data["email"].is_string());
}
```

## Pattern 3: Dynamic Validation (35 lines)

```rust
use metadol::reflect::{TypeRegistry, TypeInfo};

pub struct DynamicValidator {
    registry: TypeRegistry,
}

impl DynamicValidator {
    pub fn validate(&self, data: &serde_json::Value, type_name: &str)
        -> Result<(), Vec<ValidationError>> {
        let type_info = self.registry.lookup(type_name)
            .ok_or_else(|| vec![ValidationError::UnknownType(type_name.into())])?;

        self.validate_value(data, type_info)
    }

    fn validate_value(&self, data: &Value, type_info: &TypeInfo)
        -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        match type_info.kind() {
            TypeKind::Record => {
                let obj = data.as_object()
                    .ok_or_else(|| vec![ValidationError::TypeMismatch {
                        expected: "object".into(),
                        got: data.to_string(),
                    }])?;

                // Validate all fields
                for field in type_info.fields() {
                    match obj.get(field.name()) {
                        Some(value) => {
                            // Recursively validate field
                            if let Some(field_type) = self.registry.lookup(field.type_name()) {
                                if let Err(field_errors) = self.validate_value(value, field_type) {
                                    errors.extend(field_errors);
                                }
                            }

                            // Validate constraints
                            if let Some(constraint) = field.doc() {
                                if constraint.contains("min=") {
                                    // Parse and check constraint
                                    // Example: age >= 18
                                }
                            }
                        }
                        None if !field.is_optional() => {
                            errors.push(ValidationError::MissingField {
                                field: field.name().into(),
                            });
                        }
                        None => {} // Optional field, OK
                    }
                }
            }
            TypeKind::Primitive => {
                // Validate primitive type
                self.validate_primitive(data, type_info.name())?;
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_primitive(&self, data: &Value, type_name: &str)
        -> Result<(), Vec<ValidationError>> {
        let valid = match type_name {
            "String" => data.is_string(),
            "Int32" | "Int64" => data.is_i64(),
            "Float32" | "Float64" => data.is_f64(),
            "Bool" => data.is_boolean(),
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(vec![ValidationError::TypeMismatch {
                expected: type_name.into(),
                got: format!("{:?}", data),
            }])
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    UnknownType(String),
    TypeMismatch { expected: String, got: String },
    MissingField { field: String },
    ConstraintViolation { field: String, constraint: String },
}
```

## Pattern 4: Type-Safe Builder (35 lines)

```rust
/// Generate builder at runtime using reflection
pub fn generate_builder(type_info: &TypeInfo) -> String {
    let type_name = type_info.name();
    let builder_name = format!("{}Builder", type_name);

    let mut code = format!(
        "pub struct {} {{\n",
        builder_name
    );

    // Generate optional fields
    for field in type_info.fields() {
        code.push_str(&format!(
            "    {}: Option<{}>,\n",
            field.name(),
            field.type_name()
        ));
    }

    code.push_str("}\n\n");

    // Generate impl block
    code.push_str(&format!("impl {} {{\n", builder_name));

    // with_* methods
    for field in type_info.fields() {
        code.push_str(&format!(
            "    pub fn with_{}(mut self, {}: {}) -> Self {{\n",
            field.name(),
            field.name(),
            field.type_name()
        ));
        code.push_str(&format!(
            "        self.{} = Some({});\n",
            field.name(),
            field.name()
        ));
        code.push_str("        self\n    }\n\n");
    }

    // build method
    code.push_str(&format!(
        "    pub fn build(self) -> Result<{}, String> {{\n",
        type_name
    ));

    code.push_str(&format!("        Ok({} {{\n", type_name));
    for field in type_info.fields() {
        if field.is_optional() {
            code.push_str(&format!(
                "            {}: self.{},\n",
                field.name(),
                field.name()
            ));
        } else {
            code.push_str(&format!(
                "            {}: self.{}.ok_or(\"Missing field: {}\")?,\n",
                field.name(),
                field.name(),
                field.name()
            ));
        }
    }
    code.push_str("        })\n    }\n");

    code.push_str("}\n");

    code
}

// Usage:
let user_type = TypeInfo::record("User")
    .with_field(FieldInfo::new("id", "String"))
    .with_field(FieldInfo::new("name", "String"));

let builder_code = generate_builder(&user_type);
println!("{}", builder_code);

// Output:
// pub struct UserBuilder {
//     id: Option<String>,
//     name: Option<String>,
// }
//
// impl UserBuilder {
//     pub fn with_id(mut self, id: String) -> Self { ... }
//     pub fn with_name(mut self, name: String) -> Self { ... }
//     pub fn build(self) -> Result<User, String> { ... }
// }
```

## Pattern 5: Reflection-Based Equality

```rust
/// Compare two values using reflection
pub fn reflect_eq(
    a: &dyn std::any::Any,
    b: &dyn std::any::Any,
    type_info: &TypeInfo
) -> bool {
    match type_info.kind() {
        TypeKind::Record => {
            // Compare all fields
            for field in type_info.fields() {
                let field_a = extract_field(a, field);
                let field_b = extract_field(b, field);

                if !reflect_eq(&field_a, &field_b, field_type) {
                    return false;
                }
            }
            true
        }
        TypeKind::Primitive => {
            // Direct comparison for primitives
            compare_primitives(a, b, type_info.name())
        }
        _ => false
    }
}
```

## Common Pitfalls

### Pitfall 1: Type Erasure

```rust
// ❌ Wrong: Lost type information
let value: Box<dyn Any> = Box::new(42);
// Can't get TypeInfo without type name!

// ✅ Correct: Store type metadata
struct TypedValue {
    value: Box<dyn Any>,
    type_name: String,
}
```

### Pitfall 2: Unsafe Code

```rust
// ❌ Dangerous: Incorrect cast
let value_ptr = &value as *const _ as *const String;
let s = unsafe { &*value_ptr };  // UB if not String!

// ✅ Safer: Use Any::downcast_ref
let s = value.downcast_ref::<String>()
    .ok_or("Not a String")?;
```

## Performance Tips

1. **Cache TypeInfo lookups**
2. **Use lazy evaluation** for validation
3. **Avoid unnecessary clones** in reflection code

---

**Next**: [Tutorial 09: Multi-Language Workflow](./09-Multi-Language-Codegen-Workflow.md)

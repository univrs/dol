//! Comprehensive tests for the DOL reflection API.
//!
//! This module verifies 100% coverage of the reflection API including:
//! - TypeInfo creation and inspection
//! - FieldInfo and MethodInfo metadata
//! - TypeRegistry operations
//! - Type hierarchies and trait relationships
//! - Edge cases and error conditions

use metadol::reflect::*;

// ============================================
// TypeInfo Creation Tests
// ============================================

#[test]
fn test_typeinfo_all_kinds() {
    // Test all TypeKind variants
    let primitive = TypeInfo::primitive("Int32");
    assert_eq!(primitive.kind(), TypeKind::Primitive);

    let record = TypeInfo::record("User");
    assert_eq!(record.kind(), TypeKind::Record);

    let function = TypeInfo::function("Handler");
    assert_eq!(function.kind(), TypeKind::Function);

    let generic = TypeInfo::generic("List");
    assert_eq!(generic.kind(), TypeKind::Generic);

    let enum_type = TypeInfo::new("Status", TypeKind::Enum);
    assert_eq!(enum_type.kind(), TypeKind::Enum);

    let tuple = TypeInfo::new("Pair", TypeKind::Tuple);
    assert_eq!(tuple.kind(), TypeKind::Tuple);

    let array = TypeInfo::new("Array", TypeKind::Array);
    assert_eq!(array.kind(), TypeKind::Array);

    let optional = TypeInfo::new("Maybe", TypeKind::Optional);
    assert_eq!(optional.kind(), TypeKind::Optional);

    let reference = TypeInfo::new("Ref", TypeKind::Reference);
    assert_eq!(reference.kind(), TypeKind::Reference);

    let unknown = TypeInfo::new("Unknown", TypeKind::Unknown);
    assert_eq!(unknown.kind(), TypeKind::Unknown);
}

#[test]
fn test_typeinfo_builder_pattern() {
    let complex_type = TypeInfo::record("ComplexType")
        .with_field(FieldInfo::new("id", "Int64"))
        .with_field(FieldInfo::new("name", "String").optional())
        .with_field(FieldInfo::new("count", "Int32").mutable())
        .with_method(MethodInfo::new("calculate").returns("Float64").pure())
        .with_method(MethodInfo::new("create").returns("Self").static_method())
        .with_type_param("T")
        .with_type_param("U")
        .with_parent("BaseType")
        .implements("Serializable")
        .implements("Comparable")
        .with_doc("A complex example type")
        .private();

    assert_eq!(complex_type.name(), "ComplexType");
    assert_eq!(complex_type.kind(), TypeKind::Record);
    assert_eq!(complex_type.fields().len(), 3);
    assert_eq!(complex_type.methods().len(), 2);
    assert_eq!(complex_type.type_params().len(), 2);
    assert_eq!(complex_type.parent(), Some("BaseType"));
    assert_eq!(complex_type.traits().len(), 2);
    assert_eq!(complex_type.doc(), Some("A complex example type"));
    assert!(!complex_type.is_public());
}

// ============================================
// FieldInfo Comprehensive Tests
// ============================================

#[test]
fn test_fieldinfo_all_attributes() {
    let field = FieldInfo::new("status", "Status")
        .optional()
        .mutable()
        .with_doc("Current status")
        .with_default("Status::Pending");

    assert_eq!(field.name(), "status");
    assert_eq!(field.type_name(), "Status");
    assert!(field.is_optional());
    assert!(field.is_mutable());
    assert_eq!(field.doc(), Some("Current status"));
    assert_eq!(field.default(), Some("Status::Pending"));
}

#[test]
fn test_fieldinfo_immutable_required() {
    let field = FieldInfo::new("id", "UUID");

    assert_eq!(field.name(), "id");
    assert_eq!(field.type_name(), "UUID");
    assert!(!field.is_optional());
    assert!(!field.is_mutable());
    assert_eq!(field.doc(), None);
    assert_eq!(field.default(), None);
}

#[test]
fn test_fieldinfo_with_complex_types() {
    let field1 = FieldInfo::new("items", "Vec<Item>");
    assert_eq!(field1.type_name(), "Vec<Item>");

    let field2 = FieldInfo::new("map", "HashMap<String, Value>");
    assert_eq!(field2.type_name(), "HashMap<String, Value>");

    let field3 = FieldInfo::new("result", "Result<T, Error>");
    assert_eq!(field3.type_name(), "Result<T, Error>");
}

// ============================================
// MethodInfo Comprehensive Tests
// ============================================

#[test]
fn test_methodinfo_builder_pattern() {
    let method = MethodInfo::new("process")
        .with_param("input", "String")
        .with_param("options", "Options")
        .with_param("callback", "Fn(Result) -> bool")
        .returns("Result<Output, Error>")
        .pure()
        .with_doc("Processes input with options");

    assert_eq!(method.name(), "process");
    assert_eq!(method.params().len(), 3);
    assert_eq!(
        method.params()[0],
        ("input".to_string(), "String".to_string())
    );
    assert_eq!(
        method.params()[1],
        ("options".to_string(), "Options".to_string())
    );
    assert_eq!(method.return_type(), "Result<Output, Error>");
    assert!(method.is_pure());
    assert!(!method.is_static());
    assert_eq!(method.doc(), Some("Processes input with options"));
}

#[test]
fn test_methodinfo_static_pure() {
    let method = MethodInfo::new("from_str")
        .with_param("s", "String")
        .returns("Self")
        .static_method()
        .pure();

    assert!(method.is_static());
    assert!(method.is_pure());
}

#[test]
fn test_methodinfo_no_params() {
    let method = MethodInfo::new("get_default")
        .returns("Value")
        .static_method();

    assert_eq!(method.params().len(), 0);
    assert_eq!(method.return_type(), "Value");
}

#[test]
fn test_methodinfo_void_return() {
    let method = MethodInfo::new("execute");

    assert_eq!(method.return_type(), "Void");
}

// ============================================
// TypeRegistry Comprehensive Tests
// ============================================

#[test]
fn test_registry_empty_state() {
    let registry = TypeRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.lookup("Anything").is_none());
    assert_eq!(registry.type_names().count(), 0);
    assert_eq!(registry.types().count(), 0);
}

#[test]
fn test_registry_primitives_completeness() {
    let registry = TypeRegistry::with_primitives();

    // Verify all expected primitives
    let expected = vec![
        "Void", "Bool", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
        "Float32", "Float64", "String",
    ];

    for prim in expected {
        assert!(registry.contains(prim), "Missing primitive: {}", prim);
        let info = registry.lookup(prim).unwrap();
        assert_eq!(info.kind(), TypeKind::Primitive);
        assert_eq!(info.name(), prim);
    }
}

#[test]
fn test_registry_register_and_lookup() {
    let mut registry = TypeRegistry::new();

    let user_type = TypeInfo::record("User")
        .with_field(FieldInfo::new("id", "Int64"))
        .with_field(FieldInfo::new("name", "String"));

    registry.register(user_type.clone());

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
    assert!(registry.contains("User"));

    let found = registry.lookup("User").unwrap();
    assert_eq!(found.name(), "User");
    assert_eq!(found.fields().len(), 2);
}

#[test]
fn test_registry_overwrite_type() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("Test").with_field(FieldInfo::new("v1", "Int32")));
    assert_eq!(registry.lookup("Test").unwrap().fields().len(), 1);

    // Overwrite with new definition
    registry.register(
        TypeInfo::record("Test")
            .with_field(FieldInfo::new("v2", "String"))
            .with_field(FieldInfo::new("v3", "Bool")),
    );

    assert_eq!(registry.len(), 1);
    assert_eq!(registry.lookup("Test").unwrap().fields().len(), 2);
}

#[test]
fn test_registry_remove() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("A"));
    registry.register(TypeInfo::record("B"));
    registry.register(TypeInfo::record("C"));

    assert_eq!(registry.len(), 3);

    let removed = registry.remove("B");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name(), "B");
    assert_eq!(registry.len(), 2);
    assert!(!registry.contains("B"));
    assert!(registry.contains("A"));
    assert!(registry.contains("C"));

    // Removing non-existent type returns None
    assert!(registry.remove("NonExistent").is_none());
}

#[test]
fn test_registry_clear() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("A"));
    registry.register(TypeInfo::record("B"));
    registry.register(TypeInfo::record("C"));

    assert_eq!(registry.len(), 3);

    registry.clear();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.lookup("A").is_none());
}

// ============================================
// Type Hierarchy Tests
// ============================================

#[test]
fn test_type_hierarchy_simple() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("Entity"));
    registry.register(TypeInfo::record("User").with_parent("Entity"));
    registry.register(TypeInfo::record("Product").with_parent("Entity"));

    let entity_subtypes = registry.subtypes("Entity");
    assert_eq!(entity_subtypes.len(), 2);

    let names: Vec<&str> = entity_subtypes.iter().map(|t| t.name()).collect();
    assert!(names.contains(&"User"));
    assert!(names.contains(&"Product"));
}

#[test]
fn test_type_hierarchy_deep() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("A"));
    registry.register(TypeInfo::record("B").with_parent("A"));
    registry.register(TypeInfo::record("C").with_parent("B"));
    registry.register(TypeInfo::record("D").with_parent("C"));

    assert_eq!(registry.subtypes("A").len(), 1); // B
    assert_eq!(registry.subtypes("B").len(), 1); // C
    assert_eq!(registry.subtypes("C").len(), 1); // D
    assert_eq!(registry.subtypes("D").len(), 0);
}

#[test]
fn test_type_hierarchy_orphans() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("Parent"));
    registry.register(TypeInfo::record("Orphan"));

    assert_eq!(registry.subtypes("Parent").len(), 0);
    assert_eq!(registry.subtypes("Orphan").len(), 0);
}

// ============================================
// Trait Implementation Tests
// ============================================

#[test]
fn test_trait_implementors_simple() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("A").implements("Serializable"));
    registry.register(TypeInfo::record("B").implements("Serializable"));
    registry.register(TypeInfo::record("C"));

    let implementors = registry.implementors("Serializable");
    assert_eq!(implementors.len(), 2);

    let names: Vec<&str> = implementors.iter().map(|t| t.name()).collect();
    assert!(names.contains(&"A"));
    assert!(names.contains(&"B"));
}

#[test]
fn test_trait_implementors_multiple_traits() {
    let mut registry = TypeRegistry::new();

    registry.register(
        TypeInfo::record("User")
            .implements("Serializable")
            .implements("Validatable")
            .implements("Auditable"),
    );

    registry.register(
        TypeInfo::record("Product")
            .implements("Serializable")
            .implements("Auditable"),
    );

    assert_eq!(registry.implementors("Serializable").len(), 2);
    assert_eq!(registry.implementors("Validatable").len(), 1);
    assert_eq!(registry.implementors("Auditable").len(), 2);
    assert_eq!(registry.implementors("NonExistent").len(), 0);
}

// ============================================
// TypeInfo Field/Method Lookup Tests
// ============================================

#[test]
fn test_field_lookup() {
    let info = TypeInfo::record("Person")
        .with_field(FieldInfo::new("name", "String"))
        .with_field(FieldInfo::new("age", "Int32"))
        .with_field(FieldInfo::new("email", "String"));

    assert!(info.field("name").is_some());
    assert_eq!(info.field("name").unwrap().type_name(), "String");

    assert!(info.field("age").is_some());
    assert_eq!(info.field("age").unwrap().type_name(), "Int32");

    assert!(info.field("nonexistent").is_none());
}

#[test]
fn test_method_lookup() {
    let info = TypeInfo::record("Calculator")
        .with_method(MethodInfo::new("add").returns("Int32"))
        .with_method(MethodInfo::new("subtract").returns("Int32"))
        .with_method(MethodInfo::new("multiply").returns("Int32"));

    assert!(info.method("add").is_some());
    assert_eq!(info.method("add").unwrap().return_type(), "Int32");

    assert!(info.method("subtract").is_some());
    assert!(info.method("divide").is_none());
}

// ============================================
// reflect_type Function Tests
// ============================================

#[test]
fn test_reflect_type_known() {
    let mut registry = TypeRegistry::with_primitives();
    registry.register(TypeInfo::record("Custom"));

    let info = reflect_type(&registry, "Custom");
    assert_eq!(info.name(), "Custom");
    assert_eq!(info.kind(), TypeKind::Record);
}

#[test]
fn test_reflect_type_primitive() {
    let registry = TypeRegistry::with_primitives();

    let info = reflect_type(&registry, "Int32");
    assert_eq!(info.name(), "Int32");
    assert_eq!(info.kind(), TypeKind::Primitive);
}

#[test]
fn test_reflect_type_unknown() {
    let registry = TypeRegistry::new();

    let info = reflect_type(&registry, "NonExistent");
    assert_eq!(info.kind(), TypeKind::Unknown);
}

// ============================================
// TypeKind Display Tests
// ============================================

#[test]
fn test_typekind_display() {
    assert_eq!(format!("{}", TypeKind::Primitive), "primitive");
    assert_eq!(format!("{}", TypeKind::Record), "record");
    assert_eq!(format!("{}", TypeKind::Enum), "enum");
    assert_eq!(format!("{}", TypeKind::Function), "function");
    assert_eq!(format!("{}", TypeKind::Generic), "generic");
    assert_eq!(format!("{}", TypeKind::Tuple), "tuple");
    assert_eq!(format!("{}", TypeKind::Array), "array");
    assert_eq!(format!("{}", TypeKind::Optional), "optional");
    assert_eq!(format!("{}", TypeKind::Reference), "reference");
    assert_eq!(format!("{}", TypeKind::Unknown), "unknown");
}

// ============================================
// Edge Cases and Error Conditions
// ============================================

#[test]
fn test_empty_type_name() {
    let info = TypeInfo::record("");
    assert_eq!(info.name(), "");
}

#[test]
fn test_type_with_no_fields_or_methods() {
    let info = TypeInfo::record("Empty");

    assert_eq!(info.fields().len(), 0);
    assert_eq!(info.methods().len(), 0);
    assert_eq!(info.type_params().len(), 0);
    assert_eq!(info.traits().len(), 0);
}

#[test]
fn test_registry_iteration() {
    let mut registry = TypeRegistry::new();

    registry.register(TypeInfo::record("Z"));
    registry.register(TypeInfo::record("A"));
    registry.register(TypeInfo::record("M"));

    let type_names: Vec<&str> = registry.type_names().collect();
    assert_eq!(type_names.len(), 3);
    assert!(type_names.contains(&"Z"));
    assert!(type_names.contains(&"A"));
    assert!(type_names.contains(&"M"));

    let types: Vec<&TypeInfo> = registry.types().collect();
    assert_eq!(types.len(), 3);
}

#[test]
fn test_field_and_method_uniqueness() {
    // Test that fields and methods are separate namespaces
    let info = TypeInfo::record("Test")
        .with_field(FieldInfo::new("value", "Int32"))
        .with_method(MethodInfo::new("value").returns("Int32"));

    assert!(info.field("value").is_some());
    assert!(info.method("value").is_some());
    assert_eq!(info.field("value").unwrap().type_name(), "Int32");
    assert_eq!(info.method("value").unwrap().return_type(), "Int32");
}

#[test]
fn test_complex_generic_type() {
    let info = TypeInfo::generic("HashMap")
        .with_type_param("K")
        .with_type_param("V")
        .with_method(
            MethodInfo::new("insert")
                .with_param("key", "K")
                .with_param("value", "V")
                .returns("Option<V>"),
        )
        .with_method(
            MethodInfo::new("get")
                .with_param("key", "K")
                .returns("Option<V>")
                .pure(),
        );

    assert_eq!(info.kind(), TypeKind::Generic);
    assert_eq!(info.type_params(), &["K", "V"]);
    assert_eq!(info.methods().len(), 2);
}

// ============================================
// Integration Scenario Tests
// ============================================

#[test]
fn test_complete_domain_model() {
    let mut registry = TypeRegistry::with_primitives();

    // Define base entity
    registry.register(
        TypeInfo::record("Entity")
            .with_field(FieldInfo::new("id", "Int64"))
            .with_field(FieldInfo::new("created_at", "DateTime"))
            .with_field(FieldInfo::new("updated_at", "DateTime"))
            .implements("Identifiable")
            .implements("Timestamped"),
    );

    // Define user entity
    registry.register(
        TypeInfo::record("User")
            .with_parent("Entity")
            .with_field(FieldInfo::new("username", "String"))
            .with_field(FieldInfo::new("email", "String"))
            .with_field(FieldInfo::new("role", "Role"))
            .with_method(
                MethodInfo::new("authenticate")
                    .with_param("password", "String")
                    .returns("Result<Token, AuthError>"),
            )
            .implements("Authenticatable")
            .implements("Serializable"),
    );

    // Define admin entity
    registry.register(
        TypeInfo::record("Admin")
            .with_parent("User")
            .with_field(FieldInfo::new("permissions", "Vec<Permission>"))
            .implements("Authorizable"),
    );

    // Verify the model
    assert_eq!(registry.len(), 16); // 13 primitives + 3 custom

    // Verify hierarchy
    assert_eq!(registry.subtypes("Entity").len(), 1);
    assert_eq!(registry.subtypes("User").len(), 1);

    // Verify trait implementations
    assert_eq!(registry.implementors("Authenticatable").len(), 1);
    assert_eq!(registry.implementors("Serializable").len(), 1);

    // Verify user type
    let user = registry.lookup("User").unwrap();
    assert_eq!(user.fields().len(), 3);
    assert_eq!(user.methods().len(), 1);
    assert_eq!(user.parent(), Some("Entity"));
    assert_eq!(user.traits().len(), 2);
}

#[test]
fn test_reflection_performance() {
    let mut registry = TypeRegistry::new();

    // Register 1000 types
    for i in 0..1000 {
        registry.register(
            TypeInfo::record(format!("Type{}", i))
                .with_field(FieldInfo::new("field1", "Int32"))
                .with_field(FieldInfo::new("field2", "String")),
        );
    }

    assert_eq!(registry.len(), 1000);

    // Verify lookups work
    for i in 0..1000 {
        let name = format!("Type{}", i);
        assert!(registry.lookup(&name).is_some());
    }
}

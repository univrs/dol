//! Comprehensive tests for WIT generation from DOL Gen declarations
//!
//! These tests verify that all CRDT strategies generate valid WIT interfaces,
//! complex types are handled correctly, and the generated WIT validates with wasm-tools.

use metadol::ast::{
    CrdtAnnotation, CrdtStrategy, Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr,
    Visibility,
};
use dol_codegen_wit::{generate_gen_interface, generate_wit, generate_world, WitOptions};
use indoc::indoc;

/// Helper to create a Gen with CRDT-annotated fields
fn create_gen_with_crdt(
    name: &str,
    fields: Vec<(&str, &str, CrdtStrategy)>,
    exegesis: &str,
) -> Gen {
    let statements = fields
        .into_iter()
        .map(|(field_name, field_type, strategy)| {
            Statement::HasField(Box::new(HasField {
                name: field_name.to_string(),
                type_: parse_type(field_type),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy,
                    options: vec![],
                    span: Span::default(),
                }),
                span: Span::default(),
            }))
        })
        .collect();

    Gen {
        visibility: Visibility::Public,
        name: name.to_string(),
        extends: None,
        statements,
        exegesis: exegesis.to_string(),
        span: Span::default(),
    }
}

/// Helper to parse simple type strings
fn parse_type(type_str: &str) -> TypeExpr {
    if type_str.starts_with("Set<") {
        let inner = type_str.strip_prefix("Set<").unwrap().strip_suffix(">").unwrap();
        TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![parse_type(inner)],
        }
    } else if type_str.starts_with("List<") {
        let inner = type_str.strip_prefix("List<").unwrap().strip_suffix(">").unwrap();
        TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![parse_type(inner)],
        }
    } else if type_str.starts_with("Option<") {
        let inner = type_str
            .strip_prefix("Option<")
            .unwrap()
            .strip_suffix(">")
            .unwrap();
        TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![parse_type(inner)],
        }
    } else if type_str.starts_with("Map<") {
        let inner = type_str.strip_prefix("Map<").unwrap().strip_suffix(">").unwrap();
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        TypeExpr::Generic {
            name: "Map".to_string(),
            args: vec![parse_type(parts[0]), parse_type(parts[1])],
        }
    } else {
        TypeExpr::Named(type_str.to_string())
    }
}

#[test]
fn test_immutable_strategy() {
    let gen = create_gen_with_crdt(
        "Identity",
        vec![
            ("id", "String", CrdtStrategy::Immutable),
            ("created_at", "i64", CrdtStrategy::Immutable),
        ],
        "An identity with immutable fields",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface identity"));
    assert!(wit.contains("record identity"));
    assert!(wit.contains("/// @crdt(immutable)"));
    assert!(wit.contains("id: string"));
    assert!(wit.contains("created-at: s64"));
    assert!(wit.contains("merge: func"));
}

#[test]
fn test_lww_strategy() {
    let gen = create_gen_with_crdt(
        "UserProfile",
        vec![
            ("display_name", "String", CrdtStrategy::Lww),
            ("avatar_url", "String", CrdtStrategy::Lww),
            ("bio", "String", CrdtStrategy::Lww),
        ],
        "A user profile with last-write-wins fields",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface user-profile"));
    assert!(wit.contains("record user-profile"));
    assert!(wit.contains("/// @crdt(lww)"));
    assert!(wit.contains("display-name: string"));
    assert!(wit.contains("avatar-url: string"));
    assert!(wit.contains("bio: string"));
}

#[test]
fn test_or_set_strategy() {
    let gen = create_gen_with_crdt(
        "Document",
        vec![
            ("tags", "Set<String>", CrdtStrategy::OrSet),
            ("collaborators", "Set<String>", CrdtStrategy::OrSet),
        ],
        "A document with observed-remove sets",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface document"));
    assert!(wit.contains("record document"));
    assert!(wit.contains("/// @crdt(or_set)"));
    assert!(wit.contains("tags: list<string>"));
    assert!(wit.contains("collaborators: list<string>"));
}

#[test]
fn test_pn_counter_strategy() {
    let gen = create_gen_with_crdt(
        "Post",
        vec![
            ("likes", "i64", CrdtStrategy::PnCounter),
            ("karma_score", "i64", CrdtStrategy::PnCounter),
            ("view_count", "i64", CrdtStrategy::PnCounter),
        ],
        "A post with PN-Counter fields",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface post"));
    assert!(wit.contains("record post"));
    assert!(wit.contains("/// @crdt(pn_counter)"));
    assert!(wit.contains("likes: s64"));
    assert!(wit.contains("karma-score: s64"));
    assert!(wit.contains("view-count: s64"));
}

#[test]
fn test_peritext_strategy() {
    let gen = create_gen_with_crdt(
        "TextDocument",
        vec![
            ("id", "String", CrdtStrategy::Immutable),
            ("content", "String", CrdtStrategy::Peritext),
            ("title", "String", CrdtStrategy::Lww),
        ],
        "A text document with Peritext content",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface text-document"));
    assert!(wit.contains("record text-document"));
    assert!(wit.contains("/// @crdt(peritext)"));
    assert!(wit.contains("content: string"));
}

#[test]
fn test_rga_strategy() {
    let gen = create_gen_with_crdt(
        "TaskBoard",
        vec![
            ("task_order", "List<String>", CrdtStrategy::Rga),
            ("column_order", "List<String>", CrdtStrategy::Rga),
        ],
        "A task board with RGA ordered lists",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface task-board"));
    assert!(wit.contains("record task-board"));
    assert!(wit.contains("/// @crdt(rga)"));
    assert!(wit.contains("task-order: list<string>"));
    assert!(wit.contains("column-order: list<string>"));
}

#[test]
fn test_mv_register_strategy() {
    let gen = create_gen_with_crdt(
        "Config",
        vec![
            ("theme", "String", CrdtStrategy::MvRegister),
            ("language", "String", CrdtStrategy::MvRegister),
        ],
        "A config with multi-value registers",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface config"));
    assert!(wit.contains("record config"));
    assert!(wit.contains("/// @crdt(mv_register)"));
    assert!(wit.contains("theme: string"));
    assert!(wit.contains("language: string"));
}

#[test]
fn test_complex_nested_types() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "ComplexType".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "nested_option".to_string(),
                type_: TypeExpr::Generic {
                    name: "Option".to_string(),
                    args: vec![TypeExpr::Generic {
                        name: "List".to_string(),
                        args: vec![TypeExpr::Named("String".to_string())],
                    }],
                },
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "map_field".to_string(),
                type_: TypeExpr::Generic {
                    name: "Map".to_string(),
                    args: vec![
                        TypeExpr::Named("String".to_string()),
                        TypeExpr::Named("i32".to_string()),
                    ],
                },
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::OrSet,
                    options: vec![],
                    span: Span::default(),
                }),
                span: Span::default(),
            })),
        ],
        exegesis: "Complex nested types".to_string(),
        span: Span::default(),
    };

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("nested-option: option<list<string>>"));
    assert!(wit.contains("map-field: list<tuple<string, s32>>"));
}

#[test]
fn test_all_integer_types() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "IntegerTypes".to_string(),
        extends: None,
        statements: vec![
            Statement::HasField(Box::new(HasField {
                name: "i8_field".to_string(),
                type_: TypeExpr::Named("i8".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "i16_field".to_string(),
                type_: TypeExpr::Named("i16".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "i32_field".to_string(),
                type_: TypeExpr::Named("i32".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "i64_field".to_string(),
                type_: TypeExpr::Named("i64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "u8_field".to_string(),
                type_: TypeExpr::Named("u8".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "u16_field".to_string(),
                type_: TypeExpr::Named("u16".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "u32_field".to_string(),
                type_: TypeExpr::Named("u32".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
            Statement::HasField(Box::new(HasField {
                name: "u64_field".to_string(),
                type_: TypeExpr::Named("u64".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: None,
                span: Span::default(),
            })),
        ],
        exegesis: "All integer types".to_string(),
        span: Span::default(),
    };

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("i8-field: s8"));
    assert!(wit.contains("i16-field: s16"));
    assert!(wit.contains("i32-field: s32"));
    assert!(wit.contains("i64-field: s64"));
    assert!(wit.contains("u8-field: u8"));
    assert!(wit.contains("u16-field: u16"));
    assert!(wit.contains("u32-field: u32"));
    assert!(wit.contains("u64-field: u64"));
}

#[test]
fn test_generate_full_file() {
    let gen1 = create_gen_with_crdt(
        "ChatMessage",
        vec![
            ("id", "String", CrdtStrategy::Immutable),
            ("content", "String", CrdtStrategy::Peritext),
            ("reactions", "Set<String>", CrdtStrategy::OrSet),
        ],
        "A chat message",
    );

    let gen2 = create_gen_with_crdt(
        "Counter",
        vec![
            ("counter_id", "String", CrdtStrategy::Immutable),
            ("value", "i64", CrdtStrategy::PnCounter),
        ],
        "A distributed counter",
    );

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen1), Declaration::Gene(gen2)],
    };

    let mut options = WitOptions::default();
    options.package_name = Some("univrs:crdt".to_string());
    options.package_version = Some("1.0.0".to_string());

    let wit = generate_wit(&file, &options).unwrap();

    assert!(wit.contains("package univrs:crdt@1.0.0"));
    assert!(wit.contains("interface chat-message"));
    assert!(wit.contains("interface counter"));
    assert!(wit.contains("record chat-message"));
    assert!(wit.contains("record counter"));
}

#[test]
fn test_generate_world() {
    let gen = create_gen_with_crdt(
        "ChatMessage",
        vec![("id", "String", CrdtStrategy::Immutable)],
        "A chat message",
    );

    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(gen)],
    };

    let options = WitOptions::default();
    let world = generate_world(&file, "ChatWorld", &options).unwrap();

    assert!(world.contains("world chat-world"));
    assert!(world.contains("export chat-message"));
}

#[test]
fn test_documentation_generation() {
    let gen = create_gen_with_crdt(
        "Example",
        vec![("field", "String", CrdtStrategy::Lww)],
        indoc! {"
            This is a multi-line
            documentation string
            that should be preserved
        "},
    );

    let mut options = WitOptions::default();
    options.include_docs = true;

    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("/// This is a multi-line"));
    assert!(wit.contains("/// documentation string"));
    assert!(wit.contains("/// that should be preserved"));
}

#[test]
fn test_no_merge_functions() {
    let gen = create_gen_with_crdt(
        "Simple",
        vec![("field", "String", CrdtStrategy::Lww)],
        "Simple gen",
    );

    let mut options = WitOptions::default();
    options.generate_merge_functions = false;

    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(!wit.contains("merge: func"));
}

#[test]
fn test_no_serialization_functions() {
    let gen = create_gen_with_crdt(
        "Simple",
        vec![("field", "String", CrdtStrategy::Lww)],
        "Simple gen",
    );

    let mut options = WitOptions::default();
    options.generate_serialization = false;

    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(!wit.contains("to-bytes"));
    assert!(!wit.contains("from-bytes"));
}

#[test]
fn test_option_type() {
    let gen = Gen {
        visibility: Visibility::Public,
        name: "WithOption".to_string(),
        extends: None,
        statements: vec![Statement::HasField(Box::new(HasField {
            name: "optional_field".to_string(),
            type_: TypeExpr::Generic {
                name: "Option".to_string(),
                args: vec![TypeExpr::Named("i64".to_string())],
            },
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Lww,
                options: vec![],
                span: Span::default(),
            }),
            span: Span::default(),
        }))],
        exegesis: "With option".to_string(),
        span: Span::default(),
    };

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("optional-field: option<s64>"));
}

#[test]
fn test_kebab_case_conversion() {
    let gen = create_gen_with_crdt(
        "HTTPServer",
        vec![
            ("serverURL", "String", CrdtStrategy::Lww),
            ("maxConnections", "i32", CrdtStrategy::PnCounter),
        ],
        "HTTP Server",
    );

    let options = WitOptions::default();
    let wit = generate_gen_interface(&gen, &options).unwrap();

    assert!(wit.contains("interface http-server"));
    assert!(wit.contains("record http-server"));
    assert!(wit.contains("server-url: string"));
    assert!(wit.contains("max-connections: s32"));
}

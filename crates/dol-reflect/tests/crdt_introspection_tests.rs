//! Integration tests for CRDT introspection

use metadol::ast::{CrdtAnnotation, CrdtOption, CrdtStrategy, Expr, Literal, Span};
use dol_reflect::crdt_introspection::{
    ConflictResolution, CrdtIntrospector, MergeSemantics, TypeCompatibility,
};
use dol_reflect::schema_api::SchemaRegistry;

#[test]
fn test_merge_semantics_immutable() {
    let semantics = MergeSemantics::for_strategy(CrdtStrategy::Immutable);

    assert!(semantics.is_commutative());
    assert!(semantics.is_associative());
    assert!(semantics.is_idempotent());
    assert!(semantics.is_sec());
    assert_eq!(
        semantics.conflict_resolution(),
        ConflictResolution::NoConflicts
    );
}

#[test]
fn test_merge_semantics_lww() {
    let semantics = MergeSemantics::for_strategy(CrdtStrategy::Lww);

    assert!(semantics.is_commutative());
    assert!(semantics.is_associative());
    assert!(semantics.is_idempotent());
    assert!(semantics.is_sec());
    assert_eq!(
        semantics.conflict_resolution(),
        ConflictResolution::LastWriteWins
    );
}

#[test]
fn test_merge_semantics_or_set() {
    let semantics = MergeSemantics::for_strategy(CrdtStrategy::OrSet);

    assert!(semantics.is_commutative());
    assert!(semantics.is_associative());
    assert!(semantics.is_idempotent());
    assert!(semantics.is_sec());
    assert_eq!(semantics.conflict_resolution(), ConflictResolution::AddWins);
}

#[test]
fn test_merge_semantics_pn_counter() {
    let semantics = MergeSemantics::for_strategy(CrdtStrategy::PnCounter);

    assert!(semantics.is_commutative());
    assert!(semantics.is_associative());
    assert!(semantics.is_idempotent());
    assert!(semantics.is_sec());
    assert_eq!(
        semantics.conflict_resolution(),
        ConflictResolution::NoConflicts
    );
}

#[test]
fn test_merge_semantics_mv_register() {
    let semantics = MergeSemantics::for_strategy(CrdtStrategy::MvRegister);

    assert!(semantics.is_commutative());
    assert!(semantics.is_associative());
    assert!(semantics.is_idempotent());
    assert!(semantics.is_sec());
    assert_eq!(
        semantics.conflict_resolution(),
        ConflictResolution::MultiValue
    );
}

#[test]
fn test_type_compatibility_string() {
    let compat = TypeCompatibility::for_type("String");

    assert!(compat.is_compatible(CrdtStrategy::Immutable));
    assert!(compat.is_compatible(CrdtStrategy::Lww));
    assert!(compat.is_compatible(CrdtStrategy::Peritext));
    assert!(compat.is_compatible(CrdtStrategy::MvRegister));
    assert!(!compat.is_compatible(CrdtStrategy::PnCounter));
    assert!(!compat.is_compatible(CrdtStrategy::OrSet));

    assert_eq!(compat.recommended, Some(CrdtStrategy::Lww));
}

#[test]
fn test_type_compatibility_integer() {
    let compat = TypeCompatibility::for_type("Int32");

    assert!(compat.is_compatible(CrdtStrategy::Immutable));
    assert!(compat.is_compatible(CrdtStrategy::Lww));
    assert!(compat.is_compatible(CrdtStrategy::PnCounter));
    assert!(compat.is_compatible(CrdtStrategy::MvRegister));
    assert!(!compat.is_compatible(CrdtStrategy::OrSet));
    assert!(!compat.is_compatible(CrdtStrategy::Peritext));

    assert_eq!(compat.recommended, Some(CrdtStrategy::PnCounter));
}

#[test]
fn test_type_compatibility_set() {
    let compat = TypeCompatibility::for_type("Set<String>");

    assert!(compat.is_compatible(CrdtStrategy::Immutable));
    assert!(compat.is_compatible(CrdtStrategy::OrSet));
    assert!(compat.is_compatible(CrdtStrategy::Rga));
    assert!(compat.is_compatible(CrdtStrategy::MvRegister));
    assert!(!compat.is_compatible(CrdtStrategy::PnCounter));

    assert_eq!(compat.recommended, Some(CrdtStrategy::OrSet));
}

#[test]
fn test_type_compatibility_list() {
    let compat = TypeCompatibility::for_type("List<Int32>");

    assert!(compat.is_compatible(CrdtStrategy::Immutable));
    assert!(compat.is_compatible(CrdtStrategy::Rga));
    assert!(compat.is_compatible(CrdtStrategy::MvRegister));
    assert!(!compat.is_compatible(CrdtStrategy::PnCounter));
    assert!(!compat.is_compatible(CrdtStrategy::OrSet));

    assert_eq!(compat.recommended, Some(CrdtStrategy::Rga));
}

#[test]
fn test_analyze_crdt_field() {
    let source = r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String

  @crdt(or_set)
  message has reactions: Set<String>
}

exegesis { Chat message }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gen = registry.get_gen("chat.message").unwrap();
    let mut introspector = CrdtIntrospector::new();

    // Analyze ID field
    let id_field = gen.get_field("id").unwrap();
    let analysis = introspector.analyze_field(id_field).unwrap();

    assert_eq!(analysis.field_name, "id");
    assert_eq!(analysis.field_type, "String");
    assert_eq!(analysis.strategy, CrdtStrategy::Immutable);
    assert!(analysis.compatible);
    assert!(analysis.issues.is_empty());

    // Analyze content field
    let content_field = gen.get_field("content").unwrap();
    let analysis = introspector.analyze_field(content_field).unwrap();

    assert_eq!(analysis.field_name, "content");
    assert_eq!(analysis.strategy, CrdtStrategy::Peritext);
    assert!(analysis.compatible);

    // Analyze reactions field
    let reactions_field = gen.get_field("reactions").unwrap();
    let analysis = introspector.analyze_field(reactions_field).unwrap();

    assert_eq!(analysis.field_name, "reactions");
    assert_eq!(analysis.strategy, CrdtStrategy::OrSet);
    assert!(analysis.compatible);
}

#[test]
fn test_analyze_gen_all_fields() {
    let source = r#"
gen counter.app {
  @crdt(immutable)
  app has id: String

  @crdt(pn_counter)
  app has counter: Int32

  @crdt(lww)
  app has name: String
}

exegesis { Counter app }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gen = registry.get_gen("counter.app").unwrap();
    let mut introspector = CrdtIntrospector::new();

    let analyses = introspector.analyze_gen(gen);

    assert_eq!(analyses.len(), 3);

    // Check all fields are analyzed
    let field_names: Vec<_> = analyses.iter().map(|a| a.field_name.as_str()).collect();
    assert!(field_names.contains(&"id"));
    assert!(field_names.contains(&"counter"));
    assert!(field_names.contains(&"name"));
}

#[test]
fn test_incompatible_crdt_strategy() {
    let source = r#"
gen bad.counter {
  @crdt(peritext)
  counter has value: Int32
}

exegesis { Bad counter with incompatible CRDT }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gen = registry.get_gen("bad.counter").unwrap();
    let field = gen.get_field("value").unwrap();

    let mut introspector = CrdtIntrospector::new();
    let analysis = introspector.analyze_field(field).unwrap();

    // Should be marked as incompatible
    assert!(!analysis.compatible);
    assert!(!analysis.issues.is_empty());
}

#[test]
fn test_validate_annotation_valid() {
    let mut introspector = CrdtIntrospector::new();

    let annotation = CrdtAnnotation {
        strategy: CrdtStrategy::Lww,
        options: vec![],
        span: Span::default(),
    };

    assert!(introspector
        .validate_annotation("String", &annotation)
        .is_ok());
}

#[test]
fn test_validate_annotation_invalid_strategy() {
    let mut introspector = CrdtIntrospector::new();

    let annotation = CrdtAnnotation {
        strategy: CrdtStrategy::PnCounter,
        options: vec![],
        span: Span::default(),
    };

    let result = introspector.validate_annotation("String", &annotation);
    assert!(result.is_err());
}

#[test]
fn test_validate_annotation_with_options() {
    let mut introspector = CrdtIntrospector::new();

    let annotation = CrdtAnnotation {
        strategy: CrdtStrategy::Lww,
        options: vec![CrdtOption {
            key: "tie_break".to_string(),
            value: Expr::Literal(Literal::String("actor_id".to_string())),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    assert!(introspector
        .validate_annotation("String", &annotation)
        .is_ok());
}

#[test]
fn test_validate_annotation_invalid_option() {
    let mut introspector = CrdtIntrospector::new();

    let annotation = CrdtAnnotation {
        strategy: CrdtStrategy::Lww,
        options: vec![CrdtOption {
            key: "invalid_option".to_string(),
            value: Expr::Literal(Literal::String("value".to_string())),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    let result = introspector.validate_annotation("String", &annotation);
    assert!(result.is_err());
}

#[test]
fn test_validate_pn_counter_options() {
    let mut introspector = CrdtIntrospector::new();

    let annotation = CrdtAnnotation {
        strategy: CrdtStrategy::PnCounter,
        options: vec![
            CrdtOption {
                key: "min_value".to_string(),
                value: Expr::Literal(Literal::Int(0)),
                span: Span::default(),
            },
            CrdtOption {
                key: "max_value".to_string(),
                value: Expr::Literal(Literal::Int(100)),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    assert!(introspector
        .validate_annotation("Int32", &annotation)
        .is_ok());
}

#[test]
fn test_recommend_strategy() {
    let mut introspector = CrdtIntrospector::new();

    assert_eq!(
        introspector.recommend_strategy("String"),
        Some(CrdtStrategy::Lww)
    );
    assert_eq!(
        introspector.recommend_strategy("Int32"),
        Some(CrdtStrategy::PnCounter)
    );
    assert_eq!(
        introspector.recommend_strategy("Int64"),
        Some(CrdtStrategy::PnCounter)
    );
    assert_eq!(
        introspector.recommend_strategy("Bool"),
        Some(CrdtStrategy::Lww)
    );
    assert_eq!(
        introspector.recommend_strategy("Set<String>"),
        Some(CrdtStrategy::OrSet)
    );
    assert_eq!(
        introspector.recommend_strategy("Vec<Int32>"),
        Some(CrdtStrategy::OrSet)
    );
    assert_eq!(
        introspector.recommend_strategy("List<String>"),
        Some(CrdtStrategy::Rga)
    );
}

#[test]
fn test_analyze_registry() {
    let source = r#"
gen user.profile {
  @crdt(immutable)
  user has id: String

  @crdt(lww)
  user has name: String
}

exegesis { User }

gen message.chat {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String
}

exegesis { Message }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let mut introspector = CrdtIntrospector::new();
    let analyses = introspector.analyze_registry(&registry);

    assert_eq!(analyses.len(), 2);
    assert!(analyses.contains_key("user.profile"));
    assert!(analyses.contains_key("message.chat"));

    assert_eq!(analyses["user.profile"].len(), 2);
    assert_eq!(analyses["message.chat"].len(), 2);
}

#[test]
fn test_crdt_field_with_constraint() {
    let source = r#"
gen bounded.counter {
  @crdt(lww)
  counter has value: Int32 = 0 where value >= 0 && value <= 100
}

exegesis { Bounded counter }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gen = registry.get_gen("bounded.counter").unwrap();
    let field = gen.get_field("value").unwrap();

    let mut introspector = CrdtIntrospector::new();
    let analysis = introspector.analyze_field(field).unwrap();

    // LWW with constraint should have a compatibility warning
    assert!(analysis.compatible); // LWW is compatible with Int32
    // But there might be a warning about constraint compatibility
}

#[test]
fn test_all_crdt_strategies_have_semantics() {
    let strategies = vec![
        CrdtStrategy::Immutable,
        CrdtStrategy::Lww,
        CrdtStrategy::OrSet,
        CrdtStrategy::PnCounter,
        CrdtStrategy::Peritext,
        CrdtStrategy::Rga,
        CrdtStrategy::MvRegister,
    ];

    for strategy in strategies {
        let semantics = MergeSemantics::for_strategy(strategy);
        assert_eq!(semantics.strategy, strategy);
        // All DOL CRDT strategies should satisfy SEC
        assert!(semantics.is_sec(), "Strategy {:?} should be SEC", strategy);
    }
}

#[test]
fn test_type_compatibility_caching() {
    let mut introspector = CrdtIntrospector::new();

    // First call should compute
    let strategy1 = introspector.recommend_strategy("String");

    // Second call should use cache
    let strategy2 = introspector.recommend_strategy("String");

    assert_eq!(strategy1, strategy2);
}

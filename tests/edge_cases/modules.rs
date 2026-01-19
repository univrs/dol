//! Module System Edge Case Tests for DOL
//!
//! Tests module system edge cases:
//! - Circular dependency detection
//! - Diamond dependency patterns
//! - Name shadowing across modules
//! - Re-export chains
//! - Version conflict resolution
//! - Missing dependency handling
//! - Visibility boundary violations
//!
//! These tests help discover bugs in the module resolver and import system.

use metadol::ast::{ImportSource, UseItems, Visibility};
use metadol::parser::Parser;
use metadol::{parse_dol_file, parse_file, parse_file_all};

// ============================================================================
// CIRCULAR DEPENDENCY TESTS
// ============================================================================

mod circular_deps {
    use super::*;

    #[test]
    fn direct_circular_reference() {
        // Module A uses Module B, Module B uses Module A
        let module_a = r#"
module a @ 1.0.0

use b.Thing

gen AThing {
    has b_ref: Thing
}
"#;

        let module_b = r#"
module b @ 1.0.0

use a.AThing

gen Thing {
    has a_ref: AThing
}
"#;

        // Both should parse independently
        let result_a = parse_dol_file(module_a);
        let result_b = parse_dol_file(module_b);

        assert!(
            result_a.is_ok(),
            "Module A should parse: {:?}",
            result_a.err()
        );
        assert!(
            result_b.is_ok(),
            "Module B should parse: {:?}",
            result_b.err()
        );

        // NOTE: Actual circular dependency detection happens at link time
        println!("NOTE: Circular deps should be detected at link/compile time, not parse time");
    }

    #[test]
    fn transitive_circular_reference() {
        // A -> B -> C -> A
        let module_a = r#"
module a @ 1.0.0

use b.BThing

gen AThing {
    has b: BThing
}
"#;

        let module_b = r#"
module b @ 1.0.0

use c.CThing

gen BThing {
    has c: CThing
}
"#;

        let module_c = r#"
module c @ 1.0.0

use a.AThing

gen CThing {
    has a: AThing
}
"#;

        // All should parse
        assert!(parse_dol_file(module_a).is_ok());
        assert!(parse_dol_file(module_b).is_ok());
        assert!(parse_dol_file(module_c).is_ok());

        println!("NOTE: Transitive circular deps (A->B->C->A) should be detected at link time");
    }

    #[test]
    fn self_referential_module() {
        // Module that tries to use itself
        let source = r#"
module self_ref @ 1.0.0

use self_ref.Thing

gen Thing {
    has value: i64
}
"#;

        let result = parse_dol_file(source);

        // Parser might accept this; semantic analysis should catch it
        match result {
            Ok(_) => println!("NOTE: Self-referential use parses (caught at semantic level?)"),
            Err(e) => println!("NOTE: Self-referential use rejected at parse: {:?}", e),
        }
    }

    #[test]
    fn circular_via_wildcard_import() {
        // Circular dependency via glob import
        let module_a = r#"
module a @ 1.0.0

use b.*

gen AThing {
    has data: i64
}
"#;

        let module_b = r#"
module b @ 1.0.0

use a.*

gen BThing {
    has data: i64
}
"#;

        assert!(parse_dol_file(module_a).is_ok());
        assert!(parse_dol_file(module_b).is_ok());

        println!("NOTE: Glob import circularity should be detected at resolve time");
    }
}

// ============================================================================
// DIAMOND DEPENDENCY TESTS
// ============================================================================

mod diamond_deps {
    use super::*;

    #[test]
    fn basic_diamond_pattern() {
        // Classic diamond: A depends on B and C, both B and C depend on D
        let module_d = r#"
module d @ 1.0.0

gen Shared {
    has value: i64
}
"#;

        let module_b = r#"
module b @ 1.0.0

use d.Shared

gen BType {
    has shared: Shared
}
"#;

        let module_c = r#"
module c @ 1.0.0

use d.Shared

gen CType {
    has shared: Shared
}
"#;

        let module_a = r#"
module a @ 1.0.0

use b.BType
use c.CType
use d.Shared

gen AType {
    has b: BType
    has c: CType
    has direct: Shared
}
"#;

        // All should parse
        assert!(parse_dol_file(module_d).is_ok());
        assert!(parse_dol_file(module_b).is_ok());
        assert!(parse_dol_file(module_c).is_ok());
        assert!(parse_dol_file(module_a).is_ok());

        println!("NOTE: Diamond pattern (A->B->D, A->C->D) should work with single instance of D");
    }

    #[test]
    fn diamond_with_version_conflict() {
        // Diamond where B uses D@1.0 and C uses D@2.0
        let module_b = r#"
module b @ 1.0.0

use @univrs/d @ 1.0.0

gen BType {
    has data: i64
}
"#;

        let module_c = r#"
module c @ 1.0.0

use @univrs/d @ 2.0.0

gen CType {
    has data: i64
}
"#;

        let module_a = r#"
module a @ 1.0.0

use b.BType
use c.CType

gen AType {
    has b: BType
    has c: CType
}
"#;

        // Should parse - version resolution happens at link time
        assert!(parse_dol_file(module_b).is_ok());
        assert!(parse_dol_file(module_c).is_ok());
        assert!(parse_dol_file(module_a).is_ok());

        println!("NOTE: Diamond with version conflict should be resolved by package manager");
    }

    #[test]
    fn diamond_with_re_export() {
        // Diamond where B re-exports from D, C imports from B
        let module_d = r#"
module d @ 1.0.0

pub gen Shared {
    has value: i64
}
"#;

        let module_b = r#"
module b @ 1.0.0

pub use d.Shared

gen BType {
    has shared: Shared
}
"#;

        let module_c = r#"
module c @ 1.0.0

use b.Shared

gen CType {
    has shared: Shared
}
"#;

        assert!(parse_dol_file(module_d).is_ok());
        assert!(parse_dol_file(module_b).is_ok());
        assert!(parse_dol_file(module_c).is_ok());

        println!("NOTE: Re-export chains should resolve to same type");
    }
}

// ============================================================================
// NAME SHADOWING TESTS
// ============================================================================

mod shadowing {
    use super::*;

    #[test]
    fn local_shadows_import() {
        let source = r#"
module test @ 1.0.0

use other.Thing

gen Thing {
    has value: i64
}

gen User {
    has t: Thing
}
"#;

        let result = parse_dol_file(source);
        assert!(result.is_ok(), "Local shadowing import should parse");

        println!("NOTE: Local 'Thing' should shadow imported 'Thing' - verify at semantic level");
    }

    #[test]
    fn import_shadows_builtin() {
        let source = r#"
module test @ 1.0.0

use custom.i64

gen Test {
    has value: i64
}
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Shadowing builtin 'i64' is allowed (verify semantics)"),
            Err(e) => println!("NOTE: Shadowing builtin 'i64' is forbidden: {:?}", e),
        }
    }

    #[test]
    fn qualified_vs_unqualified() {
        let source = r#"
module test @ 1.0.0

use a.Thing
use b.Thing as OtherThing

gen Local {
    has a: Thing
    has b: OtherThing
    has c: a.Thing
    has d: b.Thing
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "Qualified and unqualified names should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn nested_module_shadowing() {
        let source = r#"
module parent.child @ 1.0.0

use parent.Thing

gen Thing {
    has local: i64
}

gen Test {
    has child_thing: Thing
    has parent_thing: parent.Thing
}
"#;

        let result = parse_dol_file(source);
        match result {
            Ok(_) => println!("NOTE: Child module can shadow parent's types"),
            Err(e) => println!("NOTE: Child shadowing parent rejected: {:?}", e),
        }
    }

    #[test]
    fn multiple_glob_imports_conflict() {
        let source = r#"
module test @ 1.0.0

use a.*
use b.*

gen User {
    has t: Thing
}
"#;

        let result = parse_dol_file(source);

        // Parser accepts this; ambiguity would be caught at name resolution
        match result {
            Ok(_) => println!("NOTE: Multiple glob imports parse - ambiguity checked later"),
            Err(e) => println!("NOTE: Multiple glob imports rejected: {:?}", e),
        }
    }

    #[test]
    fn alias_same_as_existing() {
        let source = r#"
module test @ 1.0.0

gen Thing { has value: i64 }

use other.Different as Thing
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Aliasing to existing name allowed at parse time"),
            Err(e) => println!("NOTE: Aliasing to existing name rejected: {:?}", e),
        }
    }
}

// ============================================================================
// RE-EXPORT CHAIN TESTS
// ============================================================================

mod re_exports {
    use super::*;

    #[test]
    fn simple_re_export() {
        let source = r#"
module reexport @ 1.0.0

pub use original.Thing
pub use original.{A, B, C}
"#;

        let result = parse_dol_file(source);
        assert!(result.is_ok(), "Simple re-export should parse");

        if let Ok(file) = result {
            assert_eq!(file.uses.len(), 2);
            assert!(matches!(file.uses[0].visibility, Visibility::Public));
        }
    }

    #[test]
    fn re_export_with_rename() {
        let source = r#"
module facade @ 1.0.0

pub use internal.InternalThing as PublicThing
pub use internal.{A as Alpha, B as Beta}
"#;

        let result = parse_dol_file(source);
        assert!(result.is_ok(), "Re-export with rename should parse");
    }

    #[test]
    fn chained_re_exports() {
        // A re-exports from B, B re-exports from C
        let module_c = r#"
module c @ 1.0.0

pub gen Original {
    has value: i64
}
"#;

        let module_b = r#"
module b @ 1.0.0

pub use c.Original
"#;

        let module_a = r#"
module a @ 1.0.0

pub use b.Original
"#;

        assert!(parse_dol_file(module_c).is_ok());
        assert!(parse_dol_file(module_b).is_ok());
        assert!(parse_dol_file(module_a).is_ok());

        println!("NOTE: Chained re-exports (A->B->C) should resolve to same type");
    }

    #[test]
    fn re_export_glob() {
        let source = r#"
module reexport @ 1.0.0

pub use internal.*
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(file) => {
                assert!(matches!(file.uses[0].visibility, Visibility::Public));
                println!("NOTE: Glob re-export is supported");
            }
            Err(e) => println!("NOTE: Glob re-export rejected: {:?}", e),
        }
    }

    #[test]
    fn private_type_re_exported() {
        // Module that tries to re-export a private type
        let module_internal = r#"
module internal @ 1.0.0

gen PrivateThing {
    has secret: i64
}
"#;

        let module_facade = r#"
module facade @ 1.0.0

pub use internal.PrivateThing
"#;

        // Both parse - visibility check is semantic
        assert!(parse_dol_file(module_internal).is_ok());
        assert!(parse_dol_file(module_facade).is_ok());

        println!("NOTE: Re-exporting private types should fail at semantic check");
    }
}

// ============================================================================
// VERSION CONSTRAINT TESTS
// ============================================================================

mod version_constraints {
    use super::*;

    #[test]
    fn exact_version_requirement() {
        let source = r#"
module test @ 1.0.0

use @univrs/core = 2.3.4
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "Exact version requirement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn minimum_version_requirement() {
        let source = r#"
module test @ 1.0.0

use @univrs/core >= 1.0.0
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: >= version constraint is supported"),
            Err(e) => println!("NOTE: >= version constraint not supported: {:?}", e),
        }
    }

    #[test]
    fn caret_version() {
        let source = r#"
module test @ 1.0.0

use @univrs/core @ ^1.2.3
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Caret version (^1.2.3) is supported"),
            Err(e) => println!("NOTE: Caret version not supported: {:?}", e),
        }
    }

    #[test]
    fn tilde_version() {
        let source = r#"
module test @ 1.0.0

use @univrs/core @ ~1.2.3
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Tilde version (~1.2.3) is supported"),
            Err(e) => println!("NOTE: Tilde version not supported: {:?}", e),
        }
    }

    #[test]
    fn version_range() {
        let source = r#"
module test @ 1.0.0

use @univrs/core >= 1.0.0 < 2.0.0
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Version range is supported"),
            Err(e) => println!("NOTE: Version range not supported: {:?}", e),
        }
    }

    #[test]
    fn prerelease_version() {
        let source = r#"
module test @ 1.0.0-alpha.1

use @univrs/core @ 2.0.0-beta.5
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Prerelease versions are supported"),
            Err(e) => println!("NOTE: Prerelease versions not supported: {:?}", e),
        }
    }
}

// ============================================================================
// MISSING DEPENDENCY TESTS
// ============================================================================

mod missing_deps {
    use super::*;

    #[test]
    fn use_nonexistent_module() {
        let source = r#"
module test @ 1.0.0

use nonexistent.module.Thing

gen User {
    has t: Thing
}
"#;

        let result = parse_dol_file(source);

        // Parser accepts this - resolution happens later
        assert!(
            result.is_ok(),
            "Using nonexistent module should parse (checked at resolve time)"
        );
    }

    #[test]
    fn use_nonexistent_type_from_module() {
        let source = r#"
module test @ 1.0.0

use existing.module.NonexistentType

gen User {
    has t: NonexistentType
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "Using nonexistent type should parse (checked at resolve time)"
        );
    }

    #[test]
    fn optional_dependency() {
        // Test syntax for optional dependencies
        let source = r#"
module test @ 1.0.0

use @univrs/optional?

fun maybe_use() -> i64 {
    return 42
}
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Optional dependency (?) syntax is supported"),
            Err(e) => println!("NOTE: Optional dependency syntax not supported: {:?}", e),
        }
    }
}

// ============================================================================
// VISIBILITY BOUNDARY TESTS
// ============================================================================

mod visibility_boundaries {
    use super::*;

    #[test]
    fn pub_spirit_visibility() {
        let source = r#"
module test @ 1.0.0

pub(spirit) gen InternalType {
    has value: i64
}

pub gen PublicType {
    has internal: InternalType
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "pub(spirit) visibility should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn pub_parent_visibility() {
        let source = r#"
module parent.child @ 1.0.0

pub(parent) gen ParentVisible {
    has value: i64
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "pub(parent) visibility should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn private_field_access() {
        // Testing that parser accepts private field declarations
        let source = r#"
module test @ 1.0.0

gen Encapsulated {
    has public_field: i64
    has private_field: i64
}

fun access(e: Encapsulated) -> i64 {
    return e.public_field + e.private_field
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "Field access should parse (visibility checked later)"
        );
    }

    #[test]
    fn visibility_escalation() {
        // Public type exposing private type
        let source = r#"
module test @ 1.0.0

gen PrivateHelper {
    has data: i64
}

pub gen PublicType {
    has helper: PrivateHelper
}
"#;

        let result = parse_dol_file(source);

        // Parser accepts - semantic checker should warn/error
        assert!(
            result.is_ok(),
            "Visibility escalation parses (checked semantically)"
        );
        println!("NOTE: Public type with private field should produce semantic warning");
    }

    #[test]
    fn visibility_through_trait() {
        let source = r#"
module test @ 1.0.0

trait PrivateTrait {
    fun private_method() -> i64
}

pub gen PublicType {
    has value: i64
}

impl PrivateTrait for PublicType {
    fun private_method() -> i64 {
        return 42
    }
}
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: impl syntax is supported"),
            Err(e) => println!("NOTE: impl syntax not yet supported: {:?}", e),
        }
    }
}

// ============================================================================
// IMPORT SOURCE TESTS
// ============================================================================

mod import_sources {
    use super::*;

    #[test]
    fn local_import() {
        let source = r#"
module test @ 1.0.0

use local.module.Thing
"#;

        let result = parse_dol_file(source);
        assert!(result.is_ok());

        if let Ok(file) = result {
            assert!(matches!(file.uses[0].source, ImportSource::Local));
        }
    }

    #[test]
    fn registry_import() {
        let source = r#"
module test @ 1.0.0

use @univrs/std.io.println
"#;

        let result = parse_dol_file(source);
        assert!(result.is_ok());

        if let Ok(file) = result {
            if let ImportSource::Registry { org, package, .. } = &file.uses[0].source {
                assert_eq!(org, "univrs");
                assert_eq!(package, "std");
            } else {
                panic!("Expected registry import");
            }
        }
    }

    #[test]
    fn git_import() {
        let source = r#"
module test @ 1.0.0

use @git:https://github.com/example/repo.Thing
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(file) => {
                if let ImportSource::Git { url, .. } = &file.uses[0].source {
                    println!("NOTE: Git import URL: {}", url);
                }
            }
            Err(e) => println!("NOTE: Git import syntax not supported: {:?}", e),
        }
    }

    #[test]
    fn path_import() {
        let source = r#"
module test @ 1.0.0

use @path:../sibling/module.Thing
"#;

        let result = parse_dol_file(source);

        match result {
            Ok(_) => println!("NOTE: Path import (@path:) is supported"),
            Err(e) => println!("NOTE: Path import not supported: {:?}", e),
        }
    }

    #[test]
    fn mixed_import_sources() {
        let source = r#"
module test @ 1.0.0

use local_module.Thing
use @univrs/std.io
use @org/package.Type

gen Combined {
    has local: Thing
    has io_handle: io.Handle
    has external: Type
}
"#;

        let result = parse_dol_file(source);
        assert!(
            result.is_ok(),
            "Mixed import sources should parse: {:?}",
            result.err()
        );
    }
}

// ============================================================================
// SPIRIT AND SYSTEM EDGE CASES
// ============================================================================

mod spirit_system {
    use super::*;

    #[test]
    fn empty_spirit_manifest() {
        let source = r#"
spirit empty @ 1.0.0
"#;

        let result = parse_file(source);

        match result {
            Ok(_) => println!("NOTE: Empty spirit manifest is valid"),
            Err(e) => println!("NOTE: Empty spirit manifest rejected: {:?}", e),
        }
    }

    #[test]
    fn spirit_with_config() {
        let source = r#"
spirit configured @ 1.0.0

config {
    entry: "lib.dol"
    target: wasm32
    features: ["f64-precision", "debug"]
}

pub mod core
pub mod utils
"#;

        let result = parse_file(source);

        match result {
            Ok(_) => println!("NOTE: Spirit config block is supported"),
            Err(e) => println!("NOTE: Spirit config block error: {:?}", e),
        }
    }

    #[test]
    fn system_with_bindings() {
        let source = r#"
system production @ 1.0.0 {
    requires physics >= 0.9.0
    requires chemistry >= 0.9.0

    all operations is logged
}

exegesis {
    Production system composition.
}
"#;

        let result = parse_file(source);

        match result {
            Ok(_) => println!("NOTE: System with requirements parses"),
            Err(e) => println!("NOTE: System declaration error: {:?}", e),
        }
    }

    #[test]
    fn nested_module_declarations() {
        let source = r#"
module parent @ 1.0.0

pub mod child {
    gen ChildType {
        has value: i64
    }
}

pub mod sibling {
    use super::child.ChildType

    gen SiblingType {
        has child: ChildType
    }
}
"#;

        let result = parse_file(source);

        match result {
            Ok(_) => println!("NOTE: Inline nested modules are supported"),
            Err(e) => println!("NOTE: Inline nested modules not supported: {:?}", e),
        }
    }
}

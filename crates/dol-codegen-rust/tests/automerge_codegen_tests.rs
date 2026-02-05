//! Integration tests for Automerge code generation
//!
//! These tests verify that DOL files with CRDT annotations generate
//! correct Rust code with Automerge backing.

use dol_codegen_rust::{generate_rust, CodegenOptions, Target};
use dol::parse_dol_file;

#[test]
fn test_simple_gen_with_immutable_field() {
    let source = r#"
gen message {
  @crdt(immutable) has id: String
}

exegesis {
  A message with an immutable ID.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        include_docs: true,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Verify output contains expected elements
    assert!(
        code.contains("struct Message"),
        "Should generate Message struct"
    );
    assert!(
        code.contains("Reconcile"),
        "Should derive Reconcile trait"
    );
    assert!(code.contains("Hydrate"), "Should derive Hydrate trait");
    assert!(
        code.contains("autosurgeon") && code.contains("immutable"),
        "Should have immutable attribute"
    );
    assert!(code.contains("pub id") && code.contains("String"), "Should have id field");
}

#[test]
fn test_gen_with_multiple_crdt_strategies() {
    let source = r#"
gen chat_message {
  @crdt(immutable) has id: String
  @crdt(peritext) has content: String
  @crdt(or_set) has reactions: Set<String>
  @crdt(pn_counter) has likes: i64
}

exegesis {
  A collaborative chat message with reactions and likes.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Verify all CRDT strategies are represented
    assert!(code.contains("autosurgeon") && code.contains("immutable"));
    assert!(code.contains("autosurgeon") && code.contains("text"));
    assert!(code.contains("autosurgeon") && code.contains("set"));
    assert!(code.contains("autosurgeon") && code.contains("counter"));

    // Verify field types
    assert!(code.contains("pub id") && code.contains("String"));
    assert!(code.contains("pub content") && code.contains("String"));
    assert!(code.contains("pub reactions") && code.contains("HashSet") && code.contains("String"));
    assert!(code.contains("pub likes") && code.contains("i64"));
}

#[test]
fn test_gen_with_lww_strategy() {
    let source = r#"
gen user_profile {
  @crdt(lww) has username: String
  @crdt(lww) has bio: String
}

exegesis {
  A user profile with last-write-wins semantics.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // LWW should not have special attributes (it's the default)
    assert!(code.contains("struct UserProfile"));
    assert!(code.contains("pub username") && code.contains("String"));
    assert!(code.contains("pub bio") && code.contains("String"));
}

#[test]
fn test_gen_with_complex_types() {
    let source = r#"
gen document {
  @crdt(immutable) has id: String
  @crdt(rga) has paragraphs: Vec<String>
  @crdt(or_set) has tags: Set<String>
  has metadata: Map<String, String>
}

exegesis {
  A document with complex nested types.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    assert!(code.contains("struct Document"));
    assert!(code.contains("autosurgeon") && code.contains("list")); // RGA
    assert!(code.contains("pub paragraphs") && code.contains("Vec") && code.contains("String"));
    assert!(code.contains("pub tags") && code.contains("HashSet") && code.contains("String"));
    assert!(code.contains("pub metadata") && code.contains("HashMap") && code.contains("String"));
}

#[test]
fn test_merge_method_generation() {
    let source = r#"
gen counter {
  @crdt(pn_counter) has value: i64
}

exegesis {
  A distributed counter.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Verify merge method exists
    assert!(code.contains("pub fn merge"));
    assert!(code.contains("autosurgeon :: reconcile"));
}

#[test]
fn test_automerge_conversion_methods() {
    let source = r#"
gen note {
  @crdt(immutable) has id: String
  @crdt(peritext) has content: String
}

exegesis {
  A simple note.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Verify conversion methods
    assert!(code.contains("pub fn from_automerge"));
    assert!(code.contains("pub fn to_automerge"));
}

#[test]
fn test_standard_struct_without_crdt() {
    let source = r#"
gen simple_data {
  data has name: String
  data has age: i32
}

exegesis {
  Simple data without CRDT annotations.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::Rust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Should generate standard struct without Automerge
    assert!(code.contains("struct SimpleData"));
    assert!(!code.contains("Reconcile"));
    assert!(!code.contains("autosurgeon"));
}

#[test]
fn test_serde_derives() {
    let source = r#"
gen serializable {
  @crdt(immutable) has id: String
}

exegesis {
  Serializable data.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        derive_serde: true,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    assert!(code.contains("serde :: Serialize"));
    assert!(code.contains("serde :: Deserialize"));
}

#[cfg(feature = "wasm")]
#[test]
fn test_wasm_bindings_generation() {
    use dol_codegen_rust::wasm_bindings;

    let source = r#"
gen wasm_counter {
  @crdt(pn_counter) has value: i64
}

exegesis {
  A counter for WASM.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    assert!(!file.declarations.is_empty());

    if let dol::ast::Declaration::Gene(gen) = &file.declarations[0] {
        let options = CodegenOptions::default();
        let code = wasm_bindings::generate_wasm_bindings(gen, &options)
            .expect("Failed to generate WASM bindings");

        assert!(code.contains("WasmCounterWASM"));
        assert!(code.contains("#[wasm_bindgen]"));
        assert!(code.contains("pub fn new"));
        assert!(code.contains("pub fn merge"));
        assert!(code.contains("pub fn save"));
        assert!(code.contains("pub fn load"));
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_code_compiles() {
    // This test verifies that generated code is syntactically valid
    // by parsing it with syn
    let source = r#"
gen compilable {
  @crdt(immutable) has id: String
  @crdt(peritext) has text: String
}

exegesis {
  Compilable test.
}
"#;

    let file = parse_dol_file(source).expect("Failed to parse DOL file");
    let options = CodegenOptions {
        target: Target::AutomergeRust,
        ..Default::default()
    };

    let code = generate_rust(&file, &options).expect("Failed to generate code");

    // Try to parse the generated code as Rust
    syn::parse_file(&code).expect("Generated code should be valid Rust syntax");
}

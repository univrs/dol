//! Example: Generate WIT from DOL Gen declaration
//!
//! This example demonstrates how to use the dol-codegen-wit crate to generate
//! WIT interface definitions from DOL Gen declarations with CRDT annotations.
//!
//! Run with:
//! ```bash
//! cargo run --example generate_wit
//! ```

use dol_codegen_wit::{generate_wit, WitOptions};
use metadol::ast::{
    CrdtAnnotation, CrdtStrategy, Declaration, DolFile, Gen, HasField, Span, Statement, TypeExpr,
    Visibility,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Gen declaration for a chat message
    let chat_message = Gen {
        visibility: Visibility::Public,
        name: "ChatMessage".to_string(),
        extends: None,
        statements: vec![
            // Immutable ID field
            Statement::HasField(Box::new(HasField {
                name: "id".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::Immutable,
                    options: vec![],
                    span: Span::default(),
                }),
                span: Span::default(),
            })),
            // Peritext content field
            Statement::HasField(Box::new(HasField {
                name: "content".to_string(),
                type_: TypeExpr::Named("String".to_string()),
                default: None,
                constraint: None,
                crdt_annotation: Some(CrdtAnnotation {
                    strategy: CrdtStrategy::Peritext,
                    options: vec![],
                    span: Span::default(),
                }),
                span: Span::default(),
            })),
            // OR-Set reactions field
            Statement::HasField(Box::new(HasField {
                name: "reactions".to_string(),
                type_: TypeExpr::Generic {
                    name: "Set".to_string(),
                    args: vec![TypeExpr::Named("String".to_string())],
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
            // LWW edited_at field
            Statement::HasField(Box::new(HasField {
                name: "edited_at".to_string(),
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
            })),
        ],
        exegesis: "A collaborative chat message with CRDT-based editing.\n\nSupports real-time collaboration with automatic conflict resolution.".to_string(),
        span: Span::default(),
    };

    // Create a DolFile with the Gen
    let file = DolFile {
        module: None,
        uses: vec![],
        declarations: vec![Declaration::Gene(chat_message)],
    };

    // Configure WIT generation options
    let mut options = WitOptions::default();
    options.package_name = Some("univrs:chat".to_string());
    options.package_version = Some("1.0.0".to_string());
    options.include_docs = true;
    options.generate_merge_functions = true;
    options.generate_serialization = true;

    // Generate WIT
    println!("Generating WIT interface...\n");
    let wit_code = generate_wit(&file, &options)?;

    // Output the generated WIT
    println!("{}", wit_code);

    // Optionally write to file
    // std::fs::write("chat-message.wit", wit_code)?;
    // println!("\nWritten to chat-message.wit");

    Ok(())
}

//! Comprehensive tests for CRDT annotation parsing.
//!
//! These tests verify correct parsing of @crdt annotations on Gene fields
//! as specified in RFC-001: DOL 2.0 CRDT Annotations.

use metadol::ast::{CrdtStrategy, Declaration, Statement};
use metadol::error::ParseError;
use metadol::parser::Parser;

/// Helper to parse a string and return the declaration
fn parse(input: &str) -> Result<Declaration, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

// ============================================
// 1. All 7 CRDT Strategies
// ============================================

#[test]
fn test_parse_crdt_immutable() {
    let input = r#"
gen Message {
  @crdt(immutable)
  has id: String
}

docs {
  Message with immutable ID field.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.statements.len(), 1);
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "id");
            assert!(field.crdt_annotation.is_some());
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Immutable);
            assert_eq!(annotation.options.len(), 0);
        } else {
            panic!("Expected HasField statement");
        }
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_parse_crdt_lww() {
    let input = r#"
gen Profile {
  @crdt(lww)
  has display_name: String
}

docs {
  Profile with last-write-wins display name.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "display_name");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Lww);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_or_set() {
    let input = r#"
gen Document {
  @crdt(or_set)
  has tags: Set<String>
}

docs {
  Document with OR-Set tags.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "tags");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::OrSet);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_pn_counter() {
    let input = r#"
gen Post {
  @crdt(pn_counter)
  has likes: Int
}

docs {
  Post with PN-Counter for likes.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "likes");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::PnCounter);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_peritext() {
    let input = r#"
gen RichDocument {
  @crdt(peritext)
  has content: String
}

docs {
  Document with Peritext rich text CRDT.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "content");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Peritext);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_rga() {
    let input = r#"
gen TaskBoard {
  @crdt(rga)
  has task_order: List<String>
}

docs {
  Task board with RGA for ordered list.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "task_order");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Rga);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_mv_register() {
    let input = r#"
gen Config {
  @crdt(mv_register)
  has theme: String
}

docs {
  Config with multi-value register for theme.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "theme");
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::MvRegister);
        } else {
            panic!("Expected HasField statement");
        }
    }
}

// ============================================
// 2. CRDT Options Parsing
// ============================================

#[test]
fn test_parse_crdt_with_single_option() {
    let input = r#"
gen Profile {
  @crdt(lww, tie_break="actor_id")
  has bio: String
}

docs {
  Profile with LWW and tie-break option.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Lww);
            assert_eq!(annotation.options.len(), 1);
            assert_eq!(annotation.options[0].key, "tie_break");
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_with_multiple_options() {
    let input = r#"
gen Counter {
  @crdt(pn_counter, min_value=0, max_value=100)
  has count: Int
}

docs {
  Counter with min and max value constraints.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::PnCounter);
            assert_eq!(annotation.options.len(), 2);
            assert_eq!(annotation.options[0].key, "min_value");
            assert_eq!(annotation.options[1].key, "max_value");
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_crdt_peritext_with_options() {
    let input = r#"
gen Document {
  @crdt(peritext, formatting="full", max_length=1000000)
  has content: String
}

docs {
  Document with peritext formatting options.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Peritext);
            assert_eq!(annotation.options.len(), 2);
            assert_eq!(annotation.options[0].key, "formatting");
            assert_eq!(annotation.options[1].key, "max_length");
        } else {
            panic!("Expected HasField statement");
        }
    }
}

// ============================================
// 3. Multiple Fields with CRDT Annotations
// ============================================

#[test]
fn test_parse_multiple_crdt_fields() {
    let input = r#"
gen ChatMessage {
  @crdt(immutable)
  has id: String

  @crdt(lww)
  has author: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has reactions: Set<String>
}

docs {
  Chat message with multiple CRDT fields.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.statements.len(), 4);

        // Check first field: immutable id
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "id");
            assert_eq!(
                field.crdt_annotation.as_ref().unwrap().strategy,
                CrdtStrategy::Immutable
            );
        } else {
            panic!("Expected HasField statement");
        }

        // Check second field: lww author
        if let Statement::HasField(field) = &gene.statements[1] {
            assert_eq!(field.name, "author");
            assert_eq!(
                field.crdt_annotation.as_ref().unwrap().strategy,
                CrdtStrategy::Lww
            );
        } else {
            panic!("Expected HasField statement");
        }

        // Check third field: peritext content
        if let Statement::HasField(field) = &gene.statements[2] {
            assert_eq!(field.name, "content");
            assert_eq!(
                field.crdt_annotation.as_ref().unwrap().strategy,
                CrdtStrategy::Peritext
            );
        } else {
            panic!("Expected HasField statement");
        }

        // Check fourth field: or_set reactions
        if let Statement::HasField(field) = &gene.statements[3] {
            assert_eq!(field.name, "reactions");
            assert_eq!(
                field.crdt_annotation.as_ref().unwrap().strategy,
                CrdtStrategy::OrSet
            );
        } else {
            panic!("Expected HasField statement");
        }
    } else {
        panic!("Expected Gene declaration");
    }
}

// ============================================
// 4. Fields Without CRDT Annotations
// ============================================

#[test]
fn test_parse_field_without_crdt() {
    let input = r#"
gen User {
  has username: String
}

docs {
  User without CRDT annotation.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            assert_eq!(field.name, "username");
            assert!(field.crdt_annotation.is_none());
        } else {
            panic!("Expected HasField statement");
        }
    }
}

#[test]
fn test_parse_mixed_crdt_and_non_crdt_fields() {
    let input = r#"
gen Account {
  @crdt(immutable)
  has id: String

  has email: String

  @crdt(pn_counter)
  has balance: Int
}

docs {
  Account with mixed CRDT and non-CRDT fields.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Parse error: {:?}", result.err());

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.statements.len(), 3);

        // First field has CRDT
        if let Statement::HasField(field) = &gene.statements[0] {
            assert!(field.crdt_annotation.is_some());
        }

        // Second field has no CRDT
        if let Statement::HasField(field) = &gene.statements[1] {
            assert!(field.crdt_annotation.is_none());
        }

        // Third field has CRDT
        if let Statement::HasField(field) = &gene.statements[2] {
            assert!(field.crdt_annotation.is_some());
        }
    }
}

// ============================================
// 5. Error Cases
// ============================================

#[test]
fn test_parse_invalid_crdt_strategy() {
    let input = r#"
gen Message {
  @crdt(invalid_strategy)
  has id: String
}

docs {
  Message with invalid CRDT strategy.
}
"#;
    let result = parse(input);
    assert!(result.is_err(), "Should fail with invalid strategy");

    if let Err(ParseError::InvalidCrdtStrategy { strategy, .. }) = result {
        assert_eq!(strategy, "invalid_strategy");
    } else {
        panic!("Expected InvalidCrdtStrategy error");
    }
}

#[test]
fn test_parse_crdt_missing_strategy() {
    let input = r#"
gen Message {
  @crdt()
  has id: String
}

docs {
  Message with missing CRDT strategy.
}
"#;
    let result = parse(input);
    assert!(result.is_err(), "Should fail with missing strategy");
}

#[test]
fn test_parse_crdt_typo_in_keyword() {
    let input = r#"
gen Message {
  @crdt(immutble)
  has id: String
}

docs {
  Message with typo in strategy.
}
"#;
    let result = parse(input);
    assert!(result.is_err(), "Should fail with typo in strategy");
}

// ============================================
// 6. Edge Cases
// ============================================

#[test]
fn test_parse_crdt_with_whitespace() {
    let input = r#"
gen Message {
  @crdt( immutable )
  has id: String
}

docs {
  Message with whitespace in annotation.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Should parse with whitespace");

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Immutable);
        }
    }
}

#[test]
fn test_parse_crdt_with_newlines() {
    let input = r#"
gen Message {
  @crdt(
    peritext,
    formatting="full"
  )
  has content: String
}

docs {
  Message with newlines in annotation.
}
"#;
    let result = parse(input);
    assert!(result.is_ok(), "Should parse with newlines");

    if let Declaration::Gene(gene) = result.unwrap() {
        if let Statement::HasField(field) = &gene.statements[0] {
            let annotation = field.crdt_annotation.as_ref().unwrap();
            assert_eq!(annotation.strategy, CrdtStrategy::Peritext);
            assert_eq!(annotation.options.len(), 1);
        }
    }
}

// ============================================
// 7. RFC-001 Examples
// ============================================

#[test]
fn test_parse_rfc001_chat_message_example() {
    let input = r#"
gen message.chat {
  @crdt(immutable)
  has id: Uuid

  @crdt(immutable)
  has created_at: Timestamp

  @crdt(lww)
  has author: Identity

  @crdt(peritext, formatting="full", max_length=100000)
  has content: RichText

  @crdt(or_set)
  has reactions: Set<Reaction>

  @crdt(lww)
  has edited_at: Option<Timestamp>
}

docs {
  A collaborative chat message with:
  - Immutable identity (id, created_at, author)
  - Real-time collaborative rich text editing (Peritext CRDT)
  - Add-wins emoji reactions (OR-Set CRDT)
  - Last-write-wins edit timestamp (LWW CRDT)
}
"#;
    let result = parse(input);
    assert!(
        result.is_ok(),
        "RFC-001 example should parse: {:?}",
        result.err()
    );

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.name, "message.chat");
        assert_eq!(gene.statements.len(), 6);
    }
}

#[test]
fn test_parse_rfc001_mutual_credit_example() {
    let input = r#"
gen account.mutual_credit {
  @crdt(immutable)
  has id: Uuid

  @crdt(immutable)
  has owner: Identity

  @crdt(pn_counter, min_value=0)
  has confirmed_balance: Int

  @crdt(lww, min_value=0)
  has local_escrow: Int

  @crdt(pn_counter)
  has pending_credits: Int

  @crdt(or_set)
  has transaction_history: Set<TransactionRef>

  @crdt(lww)
  has reputation_tier: ReputationTier
}

docs {
  Mutual credit account operating under eventual consistency with escrow.
}
"#;
    let result = parse(input);
    assert!(
        result.is_ok(),
        "RFC-001 example should parse: {:?}",
        result.err()
    );

    if let Declaration::Gene(gene) = result.unwrap() {
        assert_eq!(gene.name, "account.mutual_credit");
        assert_eq!(gene.statements.len(), 7);
    }
}

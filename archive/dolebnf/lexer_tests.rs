//! Lexer unit tests for Metal DOL tokenization
//!
//! These tests verify that the lexer correctly tokenizes DOL source text
//! into the expected token stream.

use metadol::lexer::{Lexer, Token, TokenKind};

mod keyword_tests {
    use super::*;

    #[test]
    fn test_declaration_keywords() {
        let cases = [
            ("gene", TokenKind::Gene),
            ("trait", TokenKind::Trait),
            ("constraint", TokenKind::Constraint),
            ("system", TokenKind::System),
            ("evolves", TokenKind::Evolves),
            ("exegesis", TokenKind::Exegesis),
        ];

        for (input, expected_kind) in cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next_token();
            assert_eq!(
                token.kind, expected_kind,
                "Expected {:?} for input '{}', got {:?}",
                expected_kind, input, token.kind
            );
        }
    }

    #[test]
    fn test_predicate_keywords() {
        let cases = [
            ("has", TokenKind::Has),
            ("is", TokenKind::Is),
            ("derives", TokenKind::Derives),
            ("from", TokenKind::From),
            ("requires", TokenKind::Requires),
            ("uses", TokenKind::Uses),
            ("emits", TokenKind::Emits),
            ("matches", TokenKind::Matches),
            ("never", TokenKind::Never),
        ];

        for (input, expected_kind) in cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next_token();
            assert_eq!(
                token.kind, expected_kind,
                "Expected {:?} for input '{}', got {:?}",
                expected_kind, input, token.kind
            );
        }
    }

    #[test]
    fn test_evolution_keywords() {
        let cases = [
            ("adds", TokenKind::Adds),
            ("deprecates", TokenKind::Deprecates),
            ("removes", TokenKind::Removes),
            ("because", TokenKind::Because),
        ];

        for (input, expected_kind) in cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next_token();
            assert_eq!(
                token.kind, expected_kind,
                "Expected {:?} for input '{}', got {:?}",
                expected_kind, input, token.kind
            );
        }
    }

    #[test]
    fn test_test_keywords() {
        let cases = [
            ("test", TokenKind::Test),
            ("given", TokenKind::Given),
            ("when", TokenKind::When),
            ("then", TokenKind::Then),
            ("always", TokenKind::Always),
        ];

        for (input, expected_kind) in cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next_token();
            assert_eq!(
                token.kind, expected_kind,
                "Expected {:?} for input '{}', got {:?}",
                expected_kind, input, token.kind
            );
        }
    }
}

mod identifier_tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let mut lexer = Lexer::new("container");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "container");
    }

    #[test]
    fn test_qualified_identifier() {
        let mut lexer = Lexer::new("container.exists");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "container.exists");
    }

    #[test]
    fn test_deeply_qualified_identifier() {
        let mut lexer = Lexer::new("univrs.container.lifecycle.state");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "univrs.container.lifecycle.state");
    }

    #[test]
    fn test_identifier_with_underscore() {
        let mut lexer = Lexer::new("created_at");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "created_at");
    }

    #[test]
    fn test_identifier_with_numbers() {
        let mut lexer = Lexer::new("container2");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "container2");
    }

    #[test]
    fn test_identifier_vs_keyword() {
        // 'genesis' should be an identifier, not confused with 'gene'
        let mut lexer = Lexer::new("genesis");
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "genesis");
    }
}

mod version_tests {
    use super::*;

    #[test]
    fn test_simple_version() {
        let mut lexer = Lexer::new("@ 0.0.1");
        
        assert_eq!(lexer.next_token().kind, TokenKind::At);
        
        let version = lexer.next_token();
        assert_eq!(version.kind, TokenKind::Version);
        assert_eq!(version.lexeme, "0.0.1");
    }

    #[test]
    fn test_multi_digit_version() {
        let mut lexer = Lexer::new("@ 10.20.30");
        
        lexer.next_token(); // skip @
        let version = lexer.next_token();
        
        assert_eq!(version.kind, TokenKind::Version);
        assert_eq!(version.lexeme, "10.20.30");
    }

    #[test]
    fn test_version_constraint() {
        let mut lexer = Lexer::new(">= 1.0.0");
        
        assert_eq!(lexer.next_token().kind, TokenKind::GreaterEqual);
        
        let version = lexer.next_token();
        assert_eq!(version.kind, TokenKind::Version);
        assert_eq!(version.lexeme, "1.0.0");
    }

    #[test]
    fn test_version_lineage() {
        let mut lexer = Lexer::new("@ 0.0.2 > 0.0.1");
        
        assert_eq!(lexer.next_token().kind, TokenKind::At);
        assert_eq!(lexer.next_token().lexeme, "0.0.2");
        assert_eq!(lexer.next_token().kind, TokenKind::Greater);
        assert_eq!(lexer.next_token().lexeme, "0.0.1");
    }
}

mod delimiter_tests {
    use super::*;

    #[test]
    fn test_braces() {
        let mut lexer = Lexer::new("{ }");
        
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_operators() {
        let cases = [
            ("@", TokenKind::At),
            (">", TokenKind::Greater),
            (">=", TokenKind::GreaterEqual),
        ];

        for (input, expected_kind) in cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next_token();
            assert_eq!(
                token.kind, expected_kind,
                "Expected {:?} for input '{}', got {:?}",
                expected_kind, input, token.kind
            );
        }
    }
}

mod string_tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "hello world");
    }

    #[test]
    fn test_string_with_spaces() {
        let mut lexer = Lexer::new(r#""workload migration requires state preservation""#);
        let token = lexer.next_token();
        
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "workload migration requires state preservation");
    }
}

mod whitespace_and_comment_tests {
    use super::*;

    #[test]
    fn test_whitespace_handling() {
        let input = "gene   container   {   }";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_newline_handling() {
        let input = "gene\ncontainer\n{\n}";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_single_line_comment() {
        let input = "gene // this is a comment\ncontainer";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Gene);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
    }

    #[test]
    fn test_trailing_comment() {
        let input = "container has identity // property declaration";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Has);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Eof);
    }
}

mod complete_file_tests {
    use super::*;

    #[test]
    fn test_complete_gene() {
        let input = r#"
gene container.exists {
  container has identity
  container has state
  container has boundaries
}
"#;
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        
        // Filter out EOF
        let tokens: Vec<_> = tokens.into_iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        assert!(tokens.len() >= 10, "Expected at least 10 tokens, got {}", tokens.len());
        
        // Verify structure
        assert_eq!(tokens[0].kind, TokenKind::Gene);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
    }

    #[test]
    fn test_complete_evolution() {
        let input = r#"
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  because "workload migration"
}
"#;
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        
        let token_kinds: Vec<_> = tokens.iter()
            .map(|t| t.kind)
            .filter(|k| *k != TokenKind::Eof)
            .collect();
        
        assert!(token_kinds.contains(&TokenKind::Evolves));
        assert!(token_kinds.contains(&TokenKind::At));
        assert!(token_kinds.contains(&TokenKind::Greater));
        assert!(token_kinds.contains(&TokenKind::Adds));
        assert!(token_kinds.contains(&TokenKind::Because));
    }
}

mod span_tracking_tests {
    use super::*;

    #[test]
    fn test_span_positions() {
        let input = "gene container";
        let mut lexer = Lexer::new(input);
        
        let gene_token = lexer.next_token();
        assert_eq!(gene_token.span.start, 0);
        assert_eq!(gene_token.span.end, 4);
        assert_eq!(gene_token.span.line, 1);
        assert_eq!(gene_token.span.column, 1);
        
        let container_token = lexer.next_token();
        assert_eq!(container_token.span.start, 5);
        assert_eq!(container_token.span.end, 14);
        assert_eq!(container_token.span.line, 1);
        assert_eq!(container_token.span.column, 6);
    }

    #[test]
    fn test_multiline_span_tracking() {
        let input = "gene\ncontainer";
        let mut lexer = Lexer::new(input);
        
        let gene_token = lexer.next_token();
        assert_eq!(gene_token.span.line, 1);
        
        let container_token = lexer.next_token();
        assert_eq!(container_token.span.line, 2);
        assert_eq!(container_token.span.column, 1);
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_unknown_character() {
        let mut lexer = Lexer::new("container $ state");
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().kind, TokenKind::Error);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier);
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        let token = lexer.next_token();
        
        // Should return error token for unterminated string
        assert_eq!(token.kind, TokenKind::Error);
    }
}

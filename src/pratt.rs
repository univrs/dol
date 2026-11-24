//! Pratt parser for DOL expression precedence.
//!
//! This module implements the Pratt parsing algorithm for handling operator
//! precedence and associativity in DOL 2.0 expressions.
//!
//! # Binding Powers
//!
//! Operators are assigned binding powers that determine their precedence:
//! - Higher binding power = tighter binding
//! - For infix operators: (left_bp, right_bp)
//!   - left < right = right associative
//!   - left > right = left associative
//!   - left = right = non-associative
//!
//! # Precedence Table
//!
//! From lowest to highest:
//! 0. Implication `implies` (3, 2) - right associative
//! 1. Assignment `:=` (10, 9) - right associative
//! 2. Pipe `|>` (21, 20) - left associative
//! 3. Application `@` (31, 30) - left associative
//! 4. Compose `>>` (40, 41) - right associative
//! 5. Arrow `->` (50, 51) - right associative
//! 6. Logical Or `||` (61, 60) - left associative
//! 7. Logical And `&` (71, 70) - left associative
//! 8. Equality `==`, `!=` (80, 80) - non-associative
//! 9. Comparison `<`, `>`, `<=`, `>=` (90, 90) - non-associative
//! 10. Additive `+`, `-` (101, 100) - left associative
//! 11. Multiplicative `*`, `/`, `%` (111, 110) - left associative
//! 12. Power `^` (120, 121) - right associative
//! 13. Member access `.` (141, 140) - left associative

use crate::lexer::TokenKind;

/// Returns the binding power (left, right) for infix operators.
///
/// Higher binding power means the operator binds more tightly.
/// For associativity:
/// - left < right: right associative
/// - left > right: left associative
/// - left = right: non-associative
///
/// # Arguments
///
/// * `op` - The token kind of the operator
///
/// # Returns
///
/// `Some((left_bp, right_bp))` if the operator is a valid infix operator,
/// `None` otherwise.
pub fn infix_binding_power(op: &TokenKind) -> Option<(u8, u8)> {
    Some(match op {
        // Implies (loosest, right-assoc, lower than || but higher than pipes)
        TokenKind::Implies => (3, 2),

        // Assignment (loosest, right-assoc)
        TokenKind::Bind => (10, 9),

        // Pipe (left-assoc)
        TokenKind::Pipe => (21, 20),

        // Application (left-assoc)
        TokenKind::At => (31, 30),

        // Compose (right-assoc)
        TokenKind::Compose => (40, 41),

        // Arrow (right-assoc)
        TokenKind::Arrow => (50, 51),

        // Range (non-assoc)
        TokenKind::DotDot => (55, 55),

        // Logical Or (left-assoc)
        TokenKind::Or => (61, 60),

        // Logical And (left-assoc)
        TokenKind::And => (71, 70),

        // Equality (non-assoc)
        TokenKind::Eq | TokenKind::Ne => (80, 80),

        // Comparison (non-assoc)
        TokenKind::Lt | TokenKind::Le | TokenKind::Greater | TokenKind::GreaterEqual => (90, 90),

        // Additive (left-assoc)
        TokenKind::Plus | TokenKind::Minus => (101, 100),

        // Multiplicative (left-assoc)
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (111, 110),

        // Power (right-assoc)
        TokenKind::Caret => (120, 121),

        // Type cast (left-assoc, high precedence)
        TokenKind::As => (131, 130),

        // Member access (left-assoc, tightest)
        TokenKind::Dot => (141, 140),

        _ => return None,
    })
}

/// Returns the binding power for prefix operators.
///
/// Prefix operators have a single binding power that determines how
/// tightly they bind to their operand.
///
/// # Arguments
///
/// * `op` - The token kind of the operator
///
/// # Returns
///
/// `Some(bp)` if the operator is a valid prefix operator, `None` otherwise.
pub fn prefix_binding_power(op: &TokenKind) -> Option<u8> {
    match op {
        TokenKind::Minus => Some(130),   // Unary minus
        TokenKind::Bang => Some(130),    // Logical not / Eval
        TokenKind::Star => Some(130),    // Dereference
        TokenKind::Quote => Some(135),   // Quote
        TokenKind::Comma => Some(135),   // Unquote (splice)
        TokenKind::Reflect => Some(135), // Reflect
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_is_right_associative() {
        let (left, right) = infix_binding_power(&TokenKind::Bind).unwrap();
        assert!(left > right, "Assignment should be right associative");
    }

    #[test]
    fn test_pipe_is_left_associative() {
        let (left, right) = infix_binding_power(&TokenKind::Pipe).unwrap();
        assert!(
            left > right,
            "Pipe should be left associative (left > right)"
        );
    }

    #[test]
    fn test_equality_is_non_associative() {
        let (left, right) = infix_binding_power(&TokenKind::Eq).unwrap();
        assert_eq!(left, right, "Equality should be non-associative");
    }

    #[test]
    fn test_compose_is_right_associative() {
        let (left, right) = infix_binding_power(&TokenKind::Compose).unwrap();
        assert!(
            left < right,
            "Compose should be right associative (left < right)"
        );
    }

    #[test]
    fn test_precedence_order() {
        // Assignment binds looser than pipe
        assert!(
            infix_binding_power(&TokenKind::Bind).unwrap().0
                < infix_binding_power(&TokenKind::Pipe).unwrap().0
        );

        // Pipe binds looser than application
        assert!(
            infix_binding_power(&TokenKind::Pipe).unwrap().0
                < infix_binding_power(&TokenKind::At).unwrap().0
        );

        // Application binds looser than compose
        assert!(
            infix_binding_power(&TokenKind::At).unwrap().0
                < infix_binding_power(&TokenKind::Compose).unwrap().0
        );

        // Member access binds tightest
        assert!(
            infix_binding_power(&TokenKind::Dot).unwrap().0
                > infix_binding_power(&TokenKind::Caret).unwrap().0
        );
    }

    #[test]
    fn test_prefix_binding_power() {
        assert_eq!(prefix_binding_power(&TokenKind::Minus), Some(130));
        assert_eq!(prefix_binding_power(&TokenKind::Bang), Some(130));
        assert_eq!(prefix_binding_power(&TokenKind::Quote), Some(135));
        assert_eq!(prefix_binding_power(&TokenKind::Reflect), Some(135));
    }

    #[test]
    fn test_invalid_operators() {
        assert_eq!(infix_binding_power(&TokenKind::Gene), None);
        assert_eq!(prefix_binding_power(&TokenKind::Trait), None);
    }
}

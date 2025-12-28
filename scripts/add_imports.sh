cd ~/repos/univrs-dol/target/bootstrap/src

# Add imports to ast.rs
sed -i '4a use crate::token::Span;' ast.rs

# Add imports to lexer.rs  
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' lexer.rs
sed -i '5a use crate::token::{Token, TokenKind, Span};' lexer.rs

# Add imports to token.rs
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' token.rs

# Test again
cargo check 2>&1 | grep "^error\[" | wc -l

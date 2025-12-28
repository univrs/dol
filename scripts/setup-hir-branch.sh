#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# DOL HIR FEATURE BRANCH SETUP
# ═══════════════════════════════════════════════════════════════════════════════
#
# Creates feature/HIR-dolv3 branch with testing infrastructure.
#
# Usage:
#   ./setup-hir-branch.sh
#
# ═══════════════════════════════════════════════════════════════════════════════

set -e

PROJECT_ROOT="${PROJECT_ROOT:-$HOME/repos/univrs-dol}"
cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                    DOL HIR FEATURE BRANCH SETUP"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 1: Ensure on main and up to date
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 1: Syncing with main..."
git checkout main
git pull origin main
echo "  ✓ On main branch, up to date"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 2: Create feature branch
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 2: Creating feature/HIR-dolv3 branch..."
if git show-ref --verify --quiet refs/heads/feature/HIR-dolv3; then
    echo "  Branch already exists, checking out..."
    git checkout feature/HIR-dolv3
else
    git checkout -b feature/HIR-dolv3
    echo "  ✓ Created feature/HIR-dolv3"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 3: Create directory structure
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 3: Creating directory structure..."

mkdir -p src/hir
mkdir -p tests/hir
mkdir -p tests/fixtures/valid
mkdir -p tests/fixtures/invalid
mkdir -p tests/integration
mkdir -p tests/e2e
mkdir -p docs/hir

echo "  ✓ Created directories"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 4: Install pre-commit hook
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 4: Installing pre-commit hook..."

if [ -f scripts/pre-commit ]; then
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "  ✓ Installed pre-commit hook"
else
    echo "  ⚠ scripts/pre-commit not found, skipping"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 5: Create HIR module stubs
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 5: Creating HIR module stubs..."

# src/hir/mod.rs
cat > src/hir/mod.rs << 'EOF'
//! HIR - High-level Intermediate Representation
//!
//! Canonical forms for DOL compilation.
//! All surface syntax desugars to these 22 node types.

pub mod types;
pub mod desugar;
pub mod validate;

pub use types::*;
pub use desugar::desugar;
pub use validate::validate;
EOF

# src/hir/types.rs
cat > src/hir/types.rs << 'EOF'
//! HIR Type Definitions
//!
//! The 22 canonical HIR node types.

use crate::ast::Span;

/// Top-level HIR node
#[derive(Debug, Clone, PartialEq)]
pub enum HirNode {
    Type(HirType),
    Expr(HirExpr),
    Stmt(HirStmt),
}

/// Type definitions
#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    /// gene → struct
    Struct {
        name: String,
        fields: Vec<HirField>,
        span: Span,
    },
    /// enum variants
    Enum {
        name: String,
        variants: Vec<HirVariant>,
        span: Span,
    },
    /// trait → interface
    Interface {
        name: String,
        methods: Vec<HirMethod>,
        span: Span,
    },
}

/// Expression nodes
#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Literal {
        value: HirLiteral,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    Binary {
        op: HirBinOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
        span: Span,
    },
    Unary {
        op: HirUnaryOp,
        operand: Box<HirExpr>,
        span: Span,
    },
    Call {
        callee: Box<HirExpr>,
        args: Vec<HirExpr>,
        span: Span,
    },
    Field {
        object: Box<HirExpr>,
        field: String,
        span: Span,
    },
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
        span: Span,
    },
    Lambda {
        params: Vec<HirParam>,
        body: Box<HirExpr>,
        span: Span,
    },
    If {
        condition: Box<HirExpr>,
        then_branch: Box<HirExpr>,
        else_branch: Option<Box<HirExpr>>,
        span: Span,
    },
    Match {
        scrutinee: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
        span: Span,
    },
    Block {
        stmts: Vec<HirStmt>,
        expr: Option<Box<HirExpr>>,
        span: Span,
    },
}

/// Statement nodes
#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Binding {
        name: String,
        mutable: bool,
        ty: Option<HirTypeRef>,
        value: HirExpr,
        span: Span,
    },
    Assign {
        target: HirExpr,
        value: HirExpr,
        span: Span,
    },
    Return {
        value: Option<HirExpr>,
        span: Span,
    },
    Loop {
        kind: HirLoopKind,
        body: Vec<HirStmt>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Expr {
        expr: HirExpr,
        span: Span,
    },
}

// Supporting types
#[derive(Debug, Clone, PartialEq)]
pub struct HirField {
    pub name: String,
    pub ty: HirTypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirVariant {
    pub name: String,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirMethod {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_ty: HirTypeRef,
    pub body: Option<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub name: String,
    pub ty: HirTypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirPattern {
    Wildcard { span: Span },
    Ident { name: String, span: Span },
    Literal { value: HirLiteral, span: Span },
    Variant { name: String, fields: Vec<HirPattern>, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirBinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirUnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirLoopKind {
    Loop,
    While { condition: HirExpr },
    ForIn { var: String, iter: HirExpr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirTypeRef {
    pub name: String,
    pub params: Vec<HirTypeRef>,
    pub span: Span,
}

impl std::fmt::Display for HirBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HirBinOp::Add => write!(f, "+"),
            HirBinOp::Sub => write!(f, "-"),
            HirBinOp::Mul => write!(f, "*"),
            HirBinOp::Div => write!(f, "/"),
            HirBinOp::Mod => write!(f, "%"),
            HirBinOp::Eq => write!(f, "=="),
            HirBinOp::Ne => write!(f, "!="),
            HirBinOp::Lt => write!(f, "<"),
            HirBinOp::Le => write!(f, "<="),
            HirBinOp::Gt => write!(f, ">"),
            HirBinOp::Ge => write!(f, ">="),
            HirBinOp::And => write!(f, "&&"),
            HirBinOp::Or => write!(f, "||"),
        }
    }
}
EOF

# src/hir/desugar.rs
cat > src/hir/desugar.rs << 'EOF'
//! AST → HIR Desugaring
//!
//! Transforms DOL surface syntax to canonical HIR forms.

use crate::ast::{Declaration, Expr, Stmt};
use super::types::*;

/// Desugar AST to HIR
pub fn desugar(ast: &[Declaration]) -> Result<Vec<HirNode>, DesugarError> {
    // TODO: Implement desugaring rules
    todo!("Implement AST → HIR desugaring")
}

#[derive(Debug, Clone)]
pub enum DesugarError {
    UnsupportedConstruct(String),
    InvalidSyntax(String),
}

impl std::fmt::Display for DesugarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesugarError::UnsupportedConstruct(s) => write!(f, "unsupported: {}", s),
            DesugarError::InvalidSyntax(s) => write!(f, "invalid syntax: {}", s),
        }
    }
}

impl std::error::Error for DesugarError {}
EOF

# src/hir/validate.rs
cat > src/hir/validate.rs << 'EOF'
//! HIR Validation
//!
//! Type checking and semantic validation of HIR.

use super::types::*;

/// Validate HIR for type correctness
pub fn validate(hir: &[HirNode]) -> Result<(), ValidationError> {
    // TODO: Implement validation
    todo!("Implement HIR validation")
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    UndefinedVariable { name: String },
    TypeMismatch { expected: String, found: String },
    MissingReturn { function: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::UndefinedVariable { name } => {
                write!(f, "undefined variable: {}", name)
            }
            ValidationError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {}, found {}", expected, found)
            }
            ValidationError::MissingReturn { function } => {
                write!(f, "missing return in function: {}", function)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
EOF

echo "  ✓ Created HIR module stubs"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 6: Create test fixtures
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 6: Creating test fixtures..."

# Valid fixtures
cat > tests/fixtures/valid/gene_basic.dol << 'EOF'
gene Counter {
    has value: Int64
}
EOF

cat > tests/fixtures/valid/gene_with_methods.dol << 'EOF'
gene Counter {
    has value: Int64
    
    fun increment() -> Int64 {
        return self.value + 1
    }
    
    fun reset() {
        self.value = 0
    }
}
EOF

cat > tests/fixtures/valid/function_control_flow.dol << 'EOF'
fun fibonacci(n: Int64) -> Int64 {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}
EOF

cat > tests/fixtures/valid/pattern_matching.dol << 'EOF'
fun describe(n: Int64) -> String {
    match n {
        0 { return "zero" }
        1 { return "one" }
        _ { return "many" }
    }
}
EOF

cat > tests/fixtures/valid/pipe_operators.dol << 'EOF'
fun double(x: Int64) -> Int64 { return x * 2 }
fun increment(x: Int64) -> Int64 { return x + 1 }

fun process(x: Int64) -> Int64 {
    return x |> double |> increment
}
EOF

# Invalid fixtures
cat > tests/fixtures/invalid/syntax_error.dol << 'EOF'
gene Broken {
    has value Int64
}
EOF

cat > tests/fixtures/invalid/type_error.dol << 'EOF'
fun bad() -> Int64 {
    return "string"
}
EOF

cat > tests/fixtures/invalid/undefined_variable.dol << 'EOF'
fun bad() -> Int64 {
    return undefined_var
}
EOF

echo "  ✓ Created test fixtures"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 7: Update lib.rs to include HIR module
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 7: Updating lib.rs..."

if ! grep -q "pub mod hir;" src/lib.rs 2>/dev/null; then
    echo "pub mod hir;" >> src/lib.rs
    echo "  ✓ Added hir module to lib.rs"
else
    echo "  ✓ hir module already in lib.rs"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 8: Initial commit
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "Step 8: Creating initial commit..."

git add .
git commit -m "feat(hir): initialize HIR module structure

- Add HIR type definitions (22 canonical forms)
- Add desugar module stub
- Add validate module stub
- Add test fixtures (valid and invalid)
- Add pre-commit hook
- Add directory structure for tests

Part of Year 1 Q3: HIR + MLIR + MCP development"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 9: Summary
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                         SETUP COMPLETE"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "Branch: feature/HIR-dolv3"
echo ""
echo "Structure created:"
echo "  src/hir/"
echo "  ├── mod.rs"
echo "  ├── types.rs      ← 22 canonical HIR types"
echo "  ├── desugar.rs    ← AST → HIR transformation"
echo "  └── validate.rs   ← Type checking"
echo ""
echo "  tests/fixtures/"
echo "  ├── valid/        ← DOL files that should compile"
echo "  └── invalid/      ← DOL files that should fail"
echo ""
echo "Next steps:"
echo "  1. Implement desugaring rules in src/hir/desugar.rs"
echo "  2. Add tests for each desugar rule"
echo "  3. Implement validation in src/hir/validate.rs"
echo "  4. Run: cargo test hir::"
echo ""
echo "Push with: git push -u origin feature/HIR-dolv3"
echo ""

//! Built-in transformation passes.
//!
//! This module provides common optimization and transformation passes:
//! - Constant folding
//! - Dead code elimination
//! - Expression simplification

use crate::ast::{BinaryOp, Block, Declaration, Expr, Literal, Span, UnaryOp, Visibility};
use crate::transform::{Pass, PassResult};
use std::collections::{HashMap, HashSet};

/// Constant folding pass.
///
/// Evaluates constant expressions at compile time:
/// - Arithmetic on literals: 1 + 2 => 3
/// - Boolean logic: true && false => false
/// - String concatenation: "a" + "b" => "ab"
pub struct ConstantFolding;

impl ConstantFolding {
    /// Creates a new constant folding pass.
    pub fn new() -> Self {
        Self
    }

    /// Fold an expression, evaluating constant subexpressions.
    #[allow(dead_code)]
    #[allow(clippy::only_used_in_recursion)]
    pub fn fold_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Binary { left, op, right } => {
                let left = self.fold_expr(*left);
                let right = self.fold_expr(*right);

                // Try to evaluate constant binary operations
                if let (Expr::Literal(l), Expr::Literal(r)) = (&left, &right) {
                    if let Some(result) = Self::eval_binary(l, &op, r) {
                        return Expr::Literal(result);
                    }
                }

                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Expr::Unary { op, operand } => {
                let operand = self.fold_expr(*operand);

                // Try to evaluate constant unary operations
                if let Expr::Literal(lit) = &operand {
                    if let Some(result) = Self::eval_unary(&op, lit) {
                        return Expr::Literal(result);
                    }
                }

                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.fold_expr(*condition);

                // If condition is constant, select the appropriate branch
                if let Expr::Literal(Literal::Bool(b)) = &condition {
                    return if *b {
                        self.fold_expr(*then_branch)
                    } else if let Some(else_expr) = else_branch {
                        self.fold_expr(*else_expr)
                    } else {
                        // No else branch, return void-like
                        Expr::Block(Block {
                            statements: vec![],
                            final_expr: None,
                            span: Span::default(),
                        })
                    };
                }

                Expr::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(self.fold_expr(*then_branch)),
                    else_branch: else_branch.map(|e| Box::new(self.fold_expr(*e))),
                }
            }
            Expr::Call { callee, args } => Expr::Call {
                callee: Box::new(self.fold_expr(*callee)),
                args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
            },
            Expr::Lambda {
                params,
                return_type,
                body,
            } => Expr::Lambda {
                params,
                return_type,
                body: Box::new(self.fold_expr(*body)),
            },
            Expr::Block(Block {
                statements,
                final_expr,
                ..
            }) => Expr::Block(Block {
                statements,
                final_expr: final_expr.map(|e| Box::new(self.fold_expr(*e))),
                span: Span::default(),
            }),
            other => other,
        }
    }

    /// Evaluate a binary operation on literals.
    #[allow(dead_code)]
    fn eval_binary(left: &Literal, op: &BinaryOp, right: &Literal) -> Option<Literal> {
        match (left, right) {
            (Literal::Int(a), Literal::Int(b)) => match op {
                BinaryOp::Add => Some(Literal::Int(a.wrapping_add(*b))),
                BinaryOp::Sub => Some(Literal::Int(a.wrapping_sub(*b))),
                BinaryOp::Mul => Some(Literal::Int(a.wrapping_mul(*b))),
                BinaryOp::Div if *b != 0 => Some(Literal::Int(a / b)),
                BinaryOp::Mod if *b != 0 => Some(Literal::Int(a % b)),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                BinaryOp::Lt => Some(Literal::Bool(a < b)),
                BinaryOp::Le => Some(Literal::Bool(a <= b)),
                BinaryOp::Gt => Some(Literal::Bool(a > b)),
                BinaryOp::Ge => Some(Literal::Bool(a >= b)),
                _ => None,
            },
            (Literal::Float(a), Literal::Float(b)) => match op {
                BinaryOp::Add => Some(Literal::Float(a + b)),
                BinaryOp::Sub => Some(Literal::Float(a - b)),
                BinaryOp::Mul => Some(Literal::Float(a * b)),
                BinaryOp::Div if *b != 0.0 => Some(Literal::Float(a / b)),
                BinaryOp::Eq => Some(Literal::Bool((a - b).abs() < f64::EPSILON)),
                BinaryOp::Ne => Some(Literal::Bool((a - b).abs() >= f64::EPSILON)),
                BinaryOp::Lt => Some(Literal::Bool(a < b)),
                BinaryOp::Le => Some(Literal::Bool(a <= b)),
                BinaryOp::Gt => Some(Literal::Bool(a > b)),
                BinaryOp::Ge => Some(Literal::Bool(a >= b)),
                _ => None,
            },
            (Literal::Bool(a), Literal::Bool(b)) => match op {
                BinaryOp::And => Some(Literal::Bool(*a && *b)),
                BinaryOp::Or => Some(Literal::Bool(*a || *b)),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                _ => None,
            },
            (Literal::String(a), Literal::String(b)) => match op {
                BinaryOp::Add => Some(Literal::String(format!("{}{}", a, b))),
                BinaryOp::Eq => Some(Literal::Bool(a == b)),
                BinaryOp::Ne => Some(Literal::Bool(a != b)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Evaluate a unary operation on a literal.
    #[allow(dead_code)]
    fn eval_unary(op: &UnaryOp, operand: &Literal) -> Option<Literal> {
        match (op, operand) {
            (UnaryOp::Neg, Literal::Int(n)) => Some(Literal::Int(-n)),
            (UnaryOp::Neg, Literal::Float(f)) => Some(Literal::Float(-f)),
            (UnaryOp::Not, Literal::Bool(b)) => Some(Literal::Bool(!b)),
            _ => None,
        }
    }
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for ConstantFolding {
    fn name(&self) -> &str {
        "constant_folding"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // DOL 1.0 declarations (Gene, Trait, Constraint, System) don't contain
        // DOL 2.0 expressions directly - they use Statement predicates.
        // This pass is primarily for future DOL 2.0 expression contexts.
        Ok(decl)
    }
}

/// Dead code elimination pass.
///
/// Removes unreachable code and unused bindings within a single declaration.
/// For whole-program tree shaking, use [`TreeShaking`] instead.
pub struct DeadCodeElimination;

impl DeadCodeElimination {
    /// Creates a new dead code elimination pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for DeadCodeElimination {
    fn name(&self) -> &str {
        "dead_code_elimination"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // Single-declaration DCE is limited - most interesting cases
        // require whole-program analysis via TreeShaking
        Ok(decl)
    }
}

/// Whole-program tree shaking (dead code elimination).
///
/// Analyzes all declarations to build a dependency graph, then removes
/// declarations that are not reachable from root declarations (public items).
///
/// # Algorithm
///
/// 1. Build a dependency graph from all declarations
/// 2. Identify root declarations (public or entry points)
/// 3. Traverse from roots, marking all reachable declarations
/// 4. Remove unreachable declarations
///
/// # Example
///
/// ```rust
/// use metadol::transform::TreeShaking;
/// use metadol::ast::{Declaration, Gen, Trait, Statement, Span, Visibility};
///
/// // Create some declarations
/// let public_gen = Gen {
///     visibility: Visibility::Public,
///     name: "api.public".to_string(),
///     extends: None,
///     statements: vec![],
///     exegesis: "Public API".to_string(),
///     span: Span::default(),
/// };
///
/// let private_gen = Gen {
///     visibility: Visibility::Private,
///     name: "internal.unused".to_string(),
///     extends: None,
///     statements: vec![],
///     exegesis: "Unused internal".to_string(),
///     span: Span::default(),
/// };
///
/// let decls = vec![
///     Declaration::Gene(public_gen),
///     Declaration::Gene(private_gen),
/// ];
///
/// let mut shaker = TreeShaking::new();
/// let result = shaker.shake(decls);
///
/// // Only the public gen is retained
/// assert_eq!(result.len(), 1);
/// assert_eq!(result[0].name(), "api.public");
/// ```
#[derive(Debug, Default)]
pub struct TreeShaking {
    /// Dependency graph: declaration name -> dependencies
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse dependency graph: declaration name -> dependents
    dependents: HashMap<String, HashSet<String>>,
    /// Set of root declarations (entry points)
    roots: HashSet<String>,
    /// Set of reachable declarations
    reachable: HashSet<String>,
    /// Additional entry points specified by user
    extra_roots: HashSet<String>,
}

impl TreeShaking {
    /// Creates a new tree shaking instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an additional entry point.
    ///
    /// Use this to prevent specific declarations from being eliminated
    /// even if they appear unused (e.g., for reflection, serialization).
    pub fn add_root(&mut self, name: impl Into<String>) -> &mut Self {
        self.extra_roots.insert(name.into());
        self
    }

    /// Performs tree shaking on a collection of declarations.
    ///
    /// Returns a new vector containing only the reachable declarations.
    pub fn shake(&mut self, decls: Vec<Declaration>) -> Vec<Declaration> {
        // Reset state
        self.dependencies.clear();
        self.dependents.clear();
        self.roots.clear();
        self.reachable.clear();

        // Phase 1: Build dependency graph
        self.build_dependency_graph(&decls);

        // Phase 2: Identify roots (public declarations + extra roots)
        self.identify_roots(&decls);

        // Phase 3: Mark reachable declarations
        self.mark_reachable();

        // Phase 4: Filter to keep only reachable declarations
        decls
            .into_iter()
            .filter(|d| self.reachable.contains(d.name()))
            .collect()
    }

    /// Analyzes declarations without modifying them.
    ///
    /// Returns statistics about what would be eliminated.
    pub fn analyze(&mut self, decls: &[Declaration]) -> TreeShakingStats {
        // Reset state
        self.dependencies.clear();
        self.dependents.clear();
        self.roots.clear();
        self.reachable.clear();

        self.build_dependency_graph(decls);
        self.identify_roots(decls);
        self.mark_reachable();

        let total = decls.len();
        let retained = self.reachable.len();
        let eliminated = total - retained;

        let eliminated_names: Vec<String> = decls
            .iter()
            .filter(|d| !self.reachable.contains(d.name()))
            .map(|d| d.name().to_string())
            .collect();

        TreeShakingStats {
            total_declarations: total,
            retained_declarations: retained,
            eliminated_declarations: eliminated,
            eliminated_names,
            root_count: self.roots.len(),
        }
    }

    /// Builds the dependency graph from declarations.
    fn build_dependency_graph(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let name = decl.name().to_string();
            let mut deps = HashSet::new();

            // Collect explicit dependencies from 'uses' statements
            for dep in decl.collect_dependencies() {
                deps.insert(dep.clone());

                // Build reverse graph (dependents)
                self.dependents.entry(dep).or_default().insert(name.clone());
            }

            // Collect inheritance dependencies from 'extends'
            if let Declaration::Gene(gen) = decl {
                if let Some(ref parent) = gen.extends {
                    deps.insert(parent.clone());
                    self.dependents
                        .entry(parent.clone())
                        .or_default()
                        .insert(name.clone());
                }
            }

            self.dependencies.insert(name, deps);
        }
    }

    /// Identifies root declarations (entry points).
    fn identify_roots(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let name = decl.name();
            let visibility = decl.visibility();

            // Public declarations are always roots
            if matches!(visibility, Visibility::Public | Visibility::PubSpirit) {
                self.roots.insert(name.to_string());
            }
        }

        // Add extra roots specified by user
        for root in &self.extra_roots {
            self.roots.insert(root.clone());
        }
    }

    /// Marks all declarations reachable from roots.
    fn mark_reachable(&mut self) {
        let mut worklist: Vec<String> = self.roots.iter().cloned().collect();

        while let Some(name) = worklist.pop() {
            if self.reachable.contains(&name) {
                continue;
            }

            self.reachable.insert(name.clone());

            // Add all dependencies to worklist
            if let Some(deps) = self.dependencies.get(&name) {
                for dep in deps {
                    if !self.reachable.contains(dep) {
                        worklist.push(dep.clone());
                    }
                }
            }
        }
    }

    /// Returns the set of reachable declaration names.
    pub fn reachable_names(&self) -> &HashSet<String> {
        &self.reachable
    }

    /// Returns the dependency graph.
    pub fn dependency_graph(&self) -> &HashMap<String, HashSet<String>> {
        &self.dependencies
    }
}

/// Statistics from tree shaking analysis.
#[derive(Debug, Clone)]
pub struct TreeShakingStats {
    /// Total number of input declarations
    pub total_declarations: usize,
    /// Number of declarations retained
    pub retained_declarations: usize,
    /// Number of declarations eliminated
    pub eliminated_declarations: usize,
    /// Names of eliminated declarations
    pub eliminated_names: Vec<String>,
    /// Number of root declarations
    pub root_count: usize,
}

impl std::fmt::Display for TreeShakingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tree Shaking Results:")?;
        writeln!(f, "  Total declarations: {}", self.total_declarations)?;
        writeln!(f, "  Root declarations:  {}", self.root_count)?;
        writeln!(f, "  Retained:           {}", self.retained_declarations)?;
        writeln!(f, "  Eliminated:         {}", self.eliminated_declarations)?;
        if !self.eliminated_names.is_empty() {
            writeln!(f, "  Eliminated names:")?;
            for name in &self.eliminated_names {
                writeln!(f, "    - {}", name)?;
            }
        }
        Ok(())
    }
}

/// Expression simplification pass.
///
/// Applies algebraic simplifications:
/// - x + 0 => x
/// - x * 1 => x
/// - x * 0 => 0
/// - x - x => 0
/// - x / 1 => x
/// - !!x => x (double negation)
pub struct Simplify;

impl Simplify {
    /// Creates a new simplification pass.
    pub fn new() -> Self {
        Self
    }

    /// Simplify an expression using algebraic identities.
    #[allow(dead_code)]
    #[allow(clippy::only_used_in_recursion)]
    pub fn simplify_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Binary { left, op, right } => {
                let left = self.simplify_expr(*left);
                let right = self.simplify_expr(*right);

                // Apply simplification rules
                match (&left, &op, &right) {
                    // x + 0 => x
                    (x, BinaryOp::Add, Expr::Literal(Literal::Int(0))) => return x.clone(),
                    (Expr::Literal(Literal::Int(0)), BinaryOp::Add, x) => return x.clone(),

                    // x * 1 => x
                    (x, BinaryOp::Mul, Expr::Literal(Literal::Int(1))) => return x.clone(),
                    (Expr::Literal(Literal::Int(1)), BinaryOp::Mul, x) => return x.clone(),

                    // x * 0 => 0
                    (_, BinaryOp::Mul, Expr::Literal(Literal::Int(0))) => {
                        return Expr::Literal(Literal::Int(0))
                    }
                    (Expr::Literal(Literal::Int(0)), BinaryOp::Mul, _) => {
                        return Expr::Literal(Literal::Int(0))
                    }

                    // x - 0 => x
                    (x, BinaryOp::Sub, Expr::Literal(Literal::Int(0))) => return x.clone(),

                    // x / 1 => x
                    (x, BinaryOp::Div, Expr::Literal(Literal::Int(1))) => return x.clone(),

                    // true && x => x, false && x => false
                    (Expr::Literal(Literal::Bool(true)), BinaryOp::And, x) => return x.clone(),
                    (Expr::Literal(Literal::Bool(false)), BinaryOp::And, _) => {
                        return Expr::Literal(Literal::Bool(false))
                    }
                    (x, BinaryOp::And, Expr::Literal(Literal::Bool(true))) => return x.clone(),
                    (_, BinaryOp::And, Expr::Literal(Literal::Bool(false))) => {
                        return Expr::Literal(Literal::Bool(false))
                    }

                    // false || x => x, true || x => true
                    (Expr::Literal(Literal::Bool(false)), BinaryOp::Or, x) => return x.clone(),
                    (Expr::Literal(Literal::Bool(true)), BinaryOp::Or, _) => {
                        return Expr::Literal(Literal::Bool(true))
                    }
                    (x, BinaryOp::Or, Expr::Literal(Literal::Bool(false))) => return x.clone(),
                    (_, BinaryOp::Or, Expr::Literal(Literal::Bool(true))) => {
                        return Expr::Literal(Literal::Bool(true))
                    }

                    _ => {}
                }

                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Expr::Unary { op, operand } => {
                let operand = self.simplify_expr(*operand);

                // Double negation elimination: !!x => x
                if op == UnaryOp::Not {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        operand: inner,
                    } = operand
                    {
                        return *inner;
                    }
                }

                // Double minus elimination: --x => x
                if op == UnaryOp::Neg {
                    if let Expr::Unary {
                        op: UnaryOp::Neg,
                        operand: inner,
                    } = operand
                    {
                        return *inner;
                    }
                }

                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            other => other,
        }
    }
}

impl Default for Simplify {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for Simplify {
    fn name(&self) -> &str {
        "simplify"
    }

    fn run(&mut self, decl: Declaration) -> PassResult<Declaration> {
        // DOL 1.0 declarations don't contain DOL 2.0 expressions directly
        Ok(decl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_arithmetic() {
        let pass = ConstantFolding::new();

        // 1 + 2 => 3
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(3)));

        // 10 * 5 => 50
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(10))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(5))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(50)));
    }

    #[test]
    fn test_constant_folding_nested() {
        let pass = ConstantFolding::new();

        // (1 + 2) * 3 => 9
        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Int(1))),
                op: BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(2))),
            }),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(3))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(9)));
    }

    #[test]
    fn test_constant_folding_boolean() {
        let pass = ConstantFolding::new();

        // true && false => false
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Bool(false))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Bool(false)));
    }

    #[test]
    fn test_constant_folding_unary() {
        let pass = ConstantFolding::new();

        // -5 => -5
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::Int(5))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(-5)));

        // !true => false
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Bool(false)));
    }

    #[test]
    fn test_constant_folding_if() {
        let pass = ConstantFolding::new();

        // if true { 1 } else { 2 } => 1
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(true))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(1)));

        // if false { 1 } else { 2 } => 2
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(false))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        let result = pass.fold_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(2)));
    }

    #[test]
    fn test_simplify_identity() {
        let pass = Simplify::new();

        // x + 0 => x
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));

        // x * 1 => x
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));
    }

    #[test]
    fn test_simplify_zero() {
        let pass = Simplify::new();

        // x * 0 => 0
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Literal(Literal::Int(0)));
    }

    #[test]
    fn test_simplify_double_negation() {
        let pass = Simplify::new();

        // !!x => x
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Identifier("x".to_string())),
            }),
        };
        let result = pass.simplify_expr(expr);
        assert_eq!(result, Expr::Identifier("x".to_string()));
    }

    // ===== Tree Shaking Tests =====

    use crate::ast::{Gen, Statement, Trait};

    fn make_gen(name: &str, visibility: Visibility) -> Declaration {
        Declaration::Gene(Gen {
            visibility,
            name: name.to_string(),
            extends: None,
            statements: vec![],
            exegesis: format!("{} gen", name),
            span: Span::default(),
        })
    }

    fn make_gen_with_extends(name: &str, visibility: Visibility, extends: &str) -> Declaration {
        Declaration::Gene(Gen {
            visibility,
            name: name.to_string(),
            extends: Some(extends.to_string()),
            statements: vec![],
            exegesis: format!("{} gen", name),
            span: Span::default(),
        })
    }

    fn make_trait_with_uses(name: &str, visibility: Visibility, uses: &[&str]) -> Declaration {
        let statements = uses
            .iter()
            .map(|u| Statement::Uses {
                reference: u.to_string(),
                span: Span::default(),
            })
            .collect();

        Declaration::Trait(Trait {
            visibility,
            name: name.to_string(),
            statements,
            exegesis: format!("{} trait", name),
            span: Span::default(),
        })
    }

    #[test]
    fn test_tree_shaking_keeps_public_declarations() {
        let decls = vec![
            make_gen("public.api", Visibility::Public),
            make_gen("private.internal", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name(), "public.api");
    }

    #[test]
    fn test_tree_shaking_keeps_dependencies_of_public() {
        let decls = vec![
            make_trait_with_uses("public.api", Visibility::Public, &["private.dep"]),
            make_gen("private.dep", Visibility::Private),
            make_gen("private.unused", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|d| d.name()).collect();
        assert!(names.contains(&"public.api"));
        assert!(names.contains(&"private.dep"));
        assert!(!names.contains(&"private.unused"));
    }

    #[test]
    fn test_tree_shaking_follows_inheritance() {
        let decls = vec![
            make_gen_with_extends("public.child", Visibility::Public, "private.parent"),
            make_gen("private.parent", Visibility::Private),
            make_gen("private.unrelated", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|d| d.name()).collect();
        assert!(names.contains(&"public.child"));
        assert!(names.contains(&"private.parent"));
        assert!(!names.contains(&"private.unrelated"));
    }

    #[test]
    fn test_tree_shaking_transitive_dependencies() {
        let decls = vec![
            make_trait_with_uses("public.api", Visibility::Public, &["private.a"]),
            make_trait_with_uses("private.a", Visibility::Private, &["private.b"]),
            make_trait_with_uses("private.b", Visibility::Private, &["private.c"]),
            make_gen("private.c", Visibility::Private),
            make_gen("private.unused", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 4);
        let names: Vec<&str> = result.iter().map(|d| d.name()).collect();
        assert!(names.contains(&"public.api"));
        assert!(names.contains(&"private.a"));
        assert!(names.contains(&"private.b"));
        assert!(names.contains(&"private.c"));
        assert!(!names.contains(&"private.unused"));
    }

    #[test]
    fn test_tree_shaking_extra_roots() {
        let decls = vec![
            make_gen("public.api", Visibility::Public),
            make_gen("private.internal", Visibility::Private),
            make_gen("private.reflected", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        shaker.add_root("private.reflected");
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|d| d.name()).collect();
        assert!(names.contains(&"public.api"));
        assert!(names.contains(&"private.reflected"));
        assert!(!names.contains(&"private.internal"));
    }

    #[test]
    fn test_tree_shaking_pub_spirit_is_root() {
        let decls = vec![
            make_gen("spirit.internal", Visibility::PubSpirit),
            make_gen("private.internal", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name(), "spirit.internal");
    }

    #[test]
    fn test_tree_shaking_all_public_kept() {
        let decls = vec![
            make_gen("public.a", Visibility::Public),
            make_gen("public.b", Visibility::Public),
            make_gen("public.c", Visibility::Public),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_tree_shaking_analyze() {
        let decls = vec![
            make_trait_with_uses("public.api", Visibility::Public, &["private.dep"]),
            make_gen("private.dep", Visibility::Private),
            make_gen("private.unused1", Visibility::Private),
            make_gen("private.unused2", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let stats = shaker.analyze(&decls);

        assert_eq!(stats.total_declarations, 4);
        assert_eq!(stats.retained_declarations, 2);
        assert_eq!(stats.eliminated_declarations, 2);
        assert_eq!(stats.root_count, 1);
        assert!(stats
            .eliminated_names
            .contains(&"private.unused1".to_string()));
        assert!(stats
            .eliminated_names
            .contains(&"private.unused2".to_string()));
    }

    #[test]
    fn test_tree_shaking_empty_input() {
        let mut shaker = TreeShaking::new();
        let result = shaker.shake(vec![]);

        assert!(result.is_empty());
    }

    #[test]
    fn test_tree_shaking_no_public_declarations() {
        let decls = vec![
            make_gen("private.a", Visibility::Private),
            make_gen("private.b", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let result = shaker.shake(decls);

        // No roots means nothing is retained
        assert!(result.is_empty());
    }

    #[test]
    fn test_tree_shaking_stats_display() {
        let stats = TreeShakingStats {
            total_declarations: 10,
            retained_declarations: 6,
            eliminated_declarations: 4,
            eliminated_names: vec!["unused.a".to_string(), "unused.b".to_string()],
            root_count: 2,
        };

        let output = format!("{}", stats);
        assert!(output.contains("Total declarations: 10"));
        assert!(output.contains("Retained:           6"));
        assert!(output.contains("Eliminated:         4"));
        assert!(output.contains("unused.a"));
        assert!(output.contains("unused.b"));
    }

    #[test]
    fn test_tree_shaking_dependency_graph_accessor() {
        let decls = vec![
            make_trait_with_uses("public.api", Visibility::Public, &["private.dep"]),
            make_gen("private.dep", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let _ = shaker.shake(decls);

        let graph = shaker.dependency_graph();
        assert!(graph.contains_key("public.api"));
        assert!(graph["public.api"].contains("private.dep"));
    }

    #[test]
    fn test_tree_shaking_reachable_names_accessor() {
        let decls = vec![
            make_trait_with_uses("public.api", Visibility::Public, &["private.dep"]),
            make_gen("private.dep", Visibility::Private),
            make_gen("private.unused", Visibility::Private),
        ];

        let mut shaker = TreeShaking::new();
        let _ = shaker.shake(decls);

        let reachable = shaker.reachable_names();
        assert!(reachable.contains("public.api"));
        assert!(reachable.contains("private.dep"));
        assert!(!reachable.contains("private.unused"));
    }
}

//! Expression interpreter for DOL 2.0.
//!
//! This module implements the core evaluation logic for DOL expressions,
//! handling arithmetic, control flow, functions, and metaprogramming features.

use crate::ast::{BinaryOp, Expr, Literal, Pattern, Stmt, TypeExpr, UnaryOp};
use crate::eval::builtins;
use crate::eval::value::{Environment, EvalError, Value};

/// The expression interpreter.
///
/// Evaluates DOL expressions in a functional style, maintaining
/// an environment for variable bindings and supporting closures.
#[derive(Debug)]
pub struct Interpreter {
    /// Current evaluation environment
    env: Environment,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new interpreter with an empty environment.
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Register built-in functions
        env.bind("print", Value::Builtin("print".to_string()));
        env.bind("typeof", Value::Builtin("typeof".to_string()));
        env.bind("len", Value::Builtin("len".to_string()));
        env.bind("push", Value::Builtin("push".to_string()));
        env.bind("pop", Value::Builtin("pop".to_string()));
        env.bind("keys", Value::Builtin("keys".to_string()));
        env.bind("values", Value::Builtin("values".to_string()));

        Self { env }
    }

    /// Evaluates an expression in the current environment.
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        self.eval_in_env(expr, &mut self.env.clone())
    }

    /// Evaluates an expression in a specific environment.
    pub fn eval_in_env(&mut self, expr: &Expr, env: &mut Environment) -> Result<Value, EvalError> {
        match expr {
            // Literals - convert to values
            Expr::Literal(lit) => self.eval_literal(lit),

            // Identifiers - lookup in environment
            Expr::Identifier(name) => env
                .lookup(name)
                .cloned()
                .ok_or_else(|| EvalError::undefined_variable(name)),

            // Binary operations
            Expr::Binary { left, op, right } => self.eval_binary(left, op, right, env),

            // Unary operations
            Expr::Unary { op, operand } => self.eval_unary(op, operand, env),

            // Function calls
            Expr::Call { callee, args } => self.eval_call(callee, args, env),

            // Member access
            Expr::Member { object, field } => self.eval_member(object, field, env),

            // Lambda expressions
            Expr::Lambda {
                params,
                body,
                return_type: _,
            } => {
                // Capture current environment for closure
                let param_names: Vec<String> =
                    params.iter().map(|(name, _)| name.clone()).collect();
                Ok(Value::Function {
                    params: param_names,
                    body: body.clone(),
                    env: env.clone(),
                })
            }

            // If expressions
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_in_env(condition, env)?;
                if cond_value.is_truthy() {
                    self.eval_in_env(then_branch, env)
                } else if let Some(else_expr) = else_branch {
                    self.eval_in_env(else_expr, env)
                } else {
                    Ok(Value::Void)
                }
            }

            // Match expressions
            Expr::Match { scrutinee, arms } => {
                let scrutinee_value = self.eval_in_env(scrutinee, env)?;

                for arm in arms {
                    let mut arm_env = env.child();
                    if self.match_pattern(&arm.pattern, &scrutinee_value, &mut arm_env)? {
                        // Check guard if present
                        if let Some(guard) = &arm.guard {
                            let guard_value = self.eval_in_env(guard, &mut arm_env)?;
                            if !guard_value.is_truthy() {
                                continue;
                            }
                        }
                        // Evaluate arm body
                        return self.eval_in_env(&arm.body, &mut arm_env);
                    }
                }

                Err(EvalError::new("no match arm matched"))
            }

            // Block expressions
            Expr::Block {
                statements,
                final_expr,
            } => {
                let mut block_env = env.child();

                // Execute statements
                for stmt in statements {
                    self.eval_stmt(stmt, &mut block_env)?;
                }

                // Evaluate final expression or return Void
                if let Some(expr) = final_expr {
                    self.eval_in_env(expr, &mut block_env)
                } else {
                    Ok(Value::Void)
                }
            }

            // Quote - capture AST
            Expr::Quote(inner) => Ok(Value::Quoted(inner.clone())),

            // Eval - evaluate quoted expression
            Expr::Eval(inner) => {
                let value = self.eval_in_env(inner, env)?;
                match value {
                    Value::Quoted(expr) => self.eval_in_env(&expr, env),
                    _ => Err(EvalError::type_error("Quoted", value.type_name())),
                }
            }

            // Reflect - return type information
            Expr::Reflect(type_expr) => self.eval_reflect(type_expr),

            // Idiom brackets: [| f a b |] desugars to f <$> a <*> b
            // For evaluation, we treat it as function application over lifted values
            Expr::IdiomBracket { func, args } => {
                // Evaluate the function
                let func_val = self.eval_in_env(func, env)?;

                // Evaluate all arguments
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_in_env(arg, env)?);
                }

                // Apply function to arguments (simplified: direct application)
                // A full implementation would use Functor/Applicative type class instances
                match func_val {
                    Value::Function {
                        params,
                        body,
                        env: closure_env,
                    } => {
                        if params.len() != arg_vals.len() {
                            return Err(EvalError::arity_mismatch(params.len(), arg_vals.len()));
                        }
                        // Create new environment from closure
                        let mut call_env = closure_env;
                        for (param, val) in params.iter().zip(arg_vals) {
                            call_env.bind(param, val);
                        }
                        self.eval_in_env(&body, &mut call_env)
                    }
                    Value::Builtin(name) => builtins::call_builtin(&name, &arg_vals),
                    _ => Err(EvalError::new(format!(
                        "cannot apply idiom brackets to non-function: {}",
                        func_val.type_name()
                    ))),
                }
            }

            // Unquote - used inside quasi-quotes for splicing
            Expr::Unquote(inner) => {
                // Unquote only makes sense inside a quasi-quote context
                // For now, just evaluate the inner expression
                self.eval_in_env(inner, env)
            }

            // Quasi-quote - quote with splicing support
            Expr::QuasiQuote(inner) => {
                // For now, treat quasi-quote like regular quote
                // TODO: Implement proper quasi-quotation with unquote splicing
                Ok(Value::Quoted(inner.clone()))
            }

            // Forall - universal quantification (logic operator)
            Expr::Forall(forall_expr) => {
                // Forall expressions are logic predicates, evaluate the body
                // In a full implementation, this would verify the property for all values of the type
                self.eval_in_env(&forall_expr.body, env)
            }

            // Exists - existential quantification (logic operator)
            Expr::Exists(exists_expr) => {
                // Exists expressions are logic predicates, evaluate the body
                // In a full implementation, this would check if any value of the type satisfies the property
                self.eval_in_env(&exists_expr.body, env)
            }

            // Implies - logical implication
            Expr::Implies { left, right, .. } => {
                // P => Q is equivalent to !P || Q
                let left_val = self.eval_in_env(left, env)?;
                if !left_val.is_truthy() {
                    Ok(Value::Bool(true))
                } else {
                    self.eval_in_env(right, env)
                }
            }

            // Sex block - execute side-effecting code
            Expr::SexBlock {
                statements,
                final_expr,
            } => {
                let mut block_env = env.child();

                // Execute statements
                for stmt in statements {
                    self.eval_stmt(stmt, &mut block_env)?;
                }

                // Evaluate final expression or return Void
                if let Some(expr) = final_expr {
                    self.eval_in_env(expr, &mut block_env)
                } else {
                    Ok(Value::Void)
                }
            }

            // List literal - evaluate elements
            Expr::List(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_in_env(elem, env)?);
                }
                Ok(Value::Array(values))
            }

            // Tuple literal - evaluate elements (stored as array)
            Expr::Tuple(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_in_env(elem, env)?);
                }
                Ok(Value::Array(values))
            }

            // Type cast - evaluate expr, cast is a no-op in the interpreter
            Expr::Cast { expr, .. } => self.eval_in_env(expr, env),

            // Struct literal - evaluate field expressions and build record
            Expr::StructLiteral {
                type_name: _,
                fields,
            } => {
                let mut record = std::collections::HashMap::new();
                for (name, expr) in fields {
                    let value = self.eval_in_env(expr, env)?;
                    record.insert(name.clone(), value);
                }
                Ok(Value::Record(record))
            }

            // Try expression - evaluate the inner expression
            Expr::Try(inner) => self.eval_in_env(inner, env),

            // This - reference to current instance
            Expr::This => env
                .lookup("this")
                .cloned()
                .ok_or_else(|| EvalError::undefined_variable("this")),
        }
    }

    /// Evaluates a literal to a value.
    fn eval_literal(&self, lit: &Literal) -> Result<Value, EvalError> {
        Ok(match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::String(c.to_string()), // Char as single-char string
            Literal::Null => Value::Void,                     // Null maps to Void
        })
    }

    /// Evaluates a binary operation.
    fn eval_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        let left_val = self.eval_in_env(left, env)?;
        let right_val = self.eval_in_env(right, env)?;

        match op {
            // Arithmetic
            BinaryOp::Add => self.eval_add(&left_val, &right_val),
            BinaryOp::Sub => self.eval_sub(&left_val, &right_val),
            BinaryOp::Mul => self.eval_mul(&left_val, &right_val),
            BinaryOp::Div => self.eval_div(&left_val, &right_val),
            BinaryOp::Mod => self.eval_mod(&left_val, &right_val),
            BinaryOp::Pow => self.eval_pow(&left_val, &right_val),

            // Comparison
            BinaryOp::Eq => Ok(Value::Bool(left_val == right_val)),
            BinaryOp::Ne => Ok(Value::Bool(left_val != right_val)),
            BinaryOp::Lt => self.eval_lt(&left_val, &right_val),
            BinaryOp::Le => self.eval_le(&left_val, &right_val),
            BinaryOp::Gt => self.eval_gt(&left_val, &right_val),
            BinaryOp::Ge => self.eval_ge(&left_val, &right_val),

            // Logical
            BinaryOp::And => Ok(Value::Bool(left_val.is_truthy() && right_val.is_truthy())),
            BinaryOp::Or => Ok(Value::Bool(left_val.is_truthy() || right_val.is_truthy())),

            // Functional
            BinaryOp::Pipe => self.eval_pipe(&left_val, &right_val, env),
            BinaryOp::Compose => self.eval_compose(&left_val, &right_val),
            BinaryOp::Apply => self.eval_apply(&left_val, &right_val, env),
            BinaryOp::Bind => Ok(right_val),

            // Member access
            BinaryOp::Member => {
                // This shouldn't happen as Member has its own Expr variant
                Err(EvalError::new("member access should use Expr::Member"))
            }

            // Functor map <$>
            BinaryOp::Map => {
                // Map applies a function to a value inside a functor
                // For now, treat as function application (simplified)
                // Full functor support would require type class instances
                self.eval_apply(&left_val, &right_val, env)
            }

            // Applicative apply <*>
            BinaryOp::Ap => {
                // Ap applies a wrapped function to a wrapped value
                // For now, treat as function application (simplified)
                // Full applicative support would require type class instances
                self.eval_apply(&left_val, &right_val, env)
            }

            // Logical implication
            BinaryOp::Implies => {
                // P => Q is equivalent to !P || Q
                if !left_val.is_truthy() {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(right_val.is_truthy()))
                }
            }

            // Range operator
            BinaryOp::Range => {
                // For now, represent range as an array [start, end]
                Ok(Value::Array(vec![left_val, right_val]))
            }
        }
    }

    /// Evaluates a unary operation.
    fn eval_unary(
        &mut self,
        op: &UnaryOp,
        operand: &Expr,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        match op {
            UnaryOp::Neg => {
                let val = self.eval_in_env(operand, env)?;
                match val {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(EvalError::type_error("numeric", val.type_name())),
                }
            }
            UnaryOp::Not => {
                let val = self.eval_in_env(operand, env)?;
                match val {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    Value::Quoted(expr) => {
                        // ! on quoted expression evaluates it
                        self.eval_in_env(&expr, env)
                    }
                    _ => Ok(Value::Bool(!val.is_truthy())),
                }
            }
            UnaryOp::Quote => Ok(Value::Quoted(Box::new(operand.clone()))),
            UnaryOp::Reflect => Err(EvalError::new("reflect operator requires type expression")),
            UnaryOp::Deref => {
                // In DOL interpreter, dereference just passes through (no real pointers)
                self.eval_in_env(operand, env)
            }
        }
    }

    /// Evaluates a function call.
    fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        let func = self.eval_in_env(callee, env)?;
        let arg_values: Result<Vec<_>, _> = args.iter().map(|a| self.eval_in_env(a, env)).collect();
        let arg_values = arg_values?;

        match func {
            Value::Function {
                params,
                body,
                env: closure_env,
            } => {
                if params.len() != arg_values.len() {
                    return Err(EvalError::arity_mismatch(params.len(), arg_values.len()));
                }

                // Create new environment from closure
                let mut call_env = closure_env.child();
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    call_env.bind(param.clone(), arg.clone());
                }

                self.eval_in_env(&body, &mut call_env)
            }
            Value::Builtin(name) => builtins::call_builtin(&name, &arg_values),
            _ => Err(EvalError::type_error("function", func.type_name())),
        }
    }

    /// Evaluates member access.
    fn eval_member(
        &mut self,
        object: &Expr,
        field: &str,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        let obj_value = self.eval_in_env(object, env)?;

        match obj_value {
            Value::Record(fields) => fields
                .get(field)
                .cloned()
                .ok_or_else(|| EvalError::new(format!("field '{}' not found", field))),
            Value::TypeInfo { fields, .. } => {
                // Return field info if querying type metadata
                for (fname, ftype) in fields {
                    if fname == field {
                        return Ok(Value::String(ftype));
                    }
                }
                Err(EvalError::new(format!("field '{}' not found", field)))
            }
            _ => Err(EvalError::new(format!(
                "cannot access field '{}' on type {}",
                field,
                obj_value.type_name()
            ))),
        }
    }

    /// Evaluates type reflection.
    fn eval_reflect(&self, type_expr: &TypeExpr) -> Result<Value, EvalError> {
        let (name, kind, fields) = match type_expr {
            TypeExpr::Named(n) => (n.clone(), "primitive".to_string(), vec![]),
            TypeExpr::Generic { name, args } => {
                let arg_names: Vec<_> = args
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("arg{}", i), "type".to_string()))
                    .collect();
                (name.clone(), "generic".to_string(), arg_names)
            }
            TypeExpr::Function {
                params,
                return_type: _,
            } => (
                "Function".to_string(),
                "function".to_string(),
                params
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("param{}", i), "type".to_string()))
                    .collect(),
            ),
            TypeExpr::Tuple(types) => (
                "Tuple".to_string(),
                "tuple".to_string(),
                types
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("field{}", i), "type".to_string()))
                    .collect(),
            ),
            TypeExpr::Never => ("Never".to_string(), "never".to_string(), vec![]),
            TypeExpr::Enum { variants } => (
                "Enum".to_string(),
                "enum".to_string(),
                variants
                    .iter()
                    .map(|v| (v.name.clone(), "variant".to_string()))
                    .collect(),
            ),
        };

        Ok(Value::TypeInfo { name, kind, fields })
    }

    /// Matches a pattern against a value, binding variables as needed.
    fn match_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Value,
        env: &mut Environment,
    ) -> Result<bool, EvalError> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Identifier(name) => {
                env.bind(name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Literal(lit) => {
                let lit_value = self.eval_literal(lit)?;
                Ok(lit_value == *value)
            }
            Pattern::Tuple(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.match_pattern(pat, val, env)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Constructor { .. } => {
                // Constructor patterns not fully implemented yet
                Ok(false)
            }
            Pattern::Or(patterns) => {
                // Try each pattern alternative
                for pat in patterns {
                    // Clone the env to avoid binding side effects on failed matches
                    let mut test_env = env.clone();
                    if self.match_pattern(pat, value, &mut test_env)? {
                        // On success, apply the bindings to the real env
                        *env = test_env;
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Evaluates a statement.
    fn eval_stmt(&mut self, stmt: &Stmt, env: &mut Environment) -> Result<(), EvalError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.eval_in_env(value, env)?;
                env.bind(name.clone(), val);
                Ok(())
            }
            Stmt::Assign { target, value } => {
                let val = self.eval_in_env(value, env)?;
                if let Expr::Identifier(name) = target {
                    env.update(name, val)?;
                    Ok(())
                } else {
                    Err(EvalError::new("assignment target must be identifier"))
                }
            }
            Stmt::Expr(expr) => {
                self.eval_in_env(expr, env)?;
                Ok(())
            }
            Stmt::Return(_) => Err(EvalError::new("return outside function not yet supported")),
            Stmt::For { .. } => Err(EvalError::new("for loops not yet implemented")),
            Stmt::While { .. } => Err(EvalError::new("while loops not yet implemented")),
            Stmt::Loop { .. } => Err(EvalError::new("loops not yet implemented")),
            Stmt::Break => Err(EvalError::new("break outside loop")),
            Stmt::Continue => Err(EvalError::new("continue outside loop")),
        }
    }

    // Arithmetic helpers
    fn eval_add(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(EvalError::invalid_operation(
                "+",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_sub(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(EvalError::invalid_operation(
                "-",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_mul(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(EvalError::invalid_operation(
                "*",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_div(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvalError::division_by_zero())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvalError::division_by_zero())
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvalError::division_by_zero())
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvalError::division_by_zero())
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(EvalError::invalid_operation(
                "/",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_mod(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvalError::division_by_zero())
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(EvalError::invalid_operation(
                "%",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_pow(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b < 0 {
                    Ok(Value::Float((*a as f64).powf(*b as f64)))
                } else {
                    Ok(Value::Int(a.pow(*b as u32)))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powf(*b as f64))),
            _ => Err(EvalError::invalid_operation(
                "^",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    // Comparison helpers
    fn eval_lt(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
            _ => Err(EvalError::invalid_operation(
                "<",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_le(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            _ => Err(EvalError::invalid_operation(
                "<=",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_gt(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
            _ => Err(EvalError::invalid_operation(
                ">",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    fn eval_ge(&self, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            _ => Err(EvalError::invalid_operation(
                ">=",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    // Functional operators
    fn eval_pipe(
        &mut self,
        value: &Value,
        func: &Value,
        _env: &mut Environment,
    ) -> Result<Value, EvalError> {
        match func {
            Value::Function {
                params,
                body,
                env: closure_env,
            } => {
                if params.is_empty() {
                    return Err(EvalError::new(
                        "pipe target function must accept at least one argument",
                    ));
                }

                let mut call_env = closure_env.child();
                call_env.bind(params[0].clone(), value.clone());

                self.eval_in_env(body, &mut call_env)
            }
            Value::Builtin(name) => builtins::call_builtin(name, std::slice::from_ref(value)),
            _ => Err(EvalError::type_error("function", func.type_name())),
        }
    }

    fn eval_compose(&self, _left: &Value, _right: &Value) -> Result<Value, EvalError> {
        // Composition returns a new function
        Err(EvalError::new("function composition not yet implemented"))
    }

    fn eval_apply(
        &mut self,
        value: &Value,
        func: &Value,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        // Apply is similar to pipe
        self.eval_pipe(value, func, env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_literal() {
        let mut interp = Interpreter::new();

        let expr = Expr::Literal(Literal::Int(42));
        assert_eq!(interp.eval(&expr).unwrap(), Value::Int(42));

        let expr = Expr::Literal(Literal::Bool(true));
        assert_eq!(interp.eval(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut interp = Interpreter::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(5))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Int(3))),
        };
        assert_eq!(interp.eval(&expr).unwrap(), Value::Int(8));

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Int(10))),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Literal(Literal::Int(4))),
        };
        assert_eq!(interp.eval(&expr).unwrap(), Value::Int(6));
    }

    #[test]
    fn test_eval_if() {
        let mut interp = Interpreter::new();

        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(true))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        assert_eq!(interp.eval(&expr).unwrap(), Value::Int(1));

        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(false))),
            then_branch: Box::new(Expr::Literal(Literal::Int(1))),
            else_branch: Some(Box::new(Expr::Literal(Literal::Int(2)))),
        };
        assert_eq!(interp.eval(&expr).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_lambda() {
        let mut interp = Interpreter::new();

        let lambda = Expr::Lambda {
            params: vec![("x".to_string(), None)],
            body: Box::new(Expr::Identifier("x".to_string())),
            return_type: None,
        };

        let result = interp.eval(&lambda).unwrap();
        assert!(matches!(result, Value::Function { .. }));
    }

    #[test]
    fn test_eval_quote() {
        let mut interp = Interpreter::new();

        let quoted = Expr::Quote(Box::new(Expr::Literal(Literal::Int(42))));
        let result = interp.eval(&quoted).unwrap();
        assert!(matches!(result, Value::Quoted(_)));
    }

    #[test]
    fn test_eval_unquote() {
        let mut interp = Interpreter::new();

        let quoted = Expr::Quote(Box::new(Expr::Literal(Literal::Int(42))));
        let evaled = Expr::Eval(Box::new(quoted));

        assert_eq!(interp.eval(&evaled).unwrap(), Value::Int(42));
    }
}

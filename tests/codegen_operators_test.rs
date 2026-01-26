use metadol::ast::{BinaryOp, Expr, Literal};
use metadol::codegen::RustCodegen;

#[test]
fn test_gen_pipe_operator() {
    let gen = RustCodegen::new();
    // x |> f becomes f(x)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("x".to_string())),
        op: BinaryOp::Pipe,
        right: Box::new(Expr::Identifier("f".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "f(x)");
}

#[test]
fn test_gen_pipe_operator_with_literal() {
    let gen = RustCodegen::new();
    // 42 |> double becomes double(42)
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Int(42))),
        op: BinaryOp::Pipe,
        right: Box::new(Expr::Identifier("double".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "double(42)");
}

#[test]
fn test_gen_compose_operator() {
    let gen = RustCodegen::new();
    // f >> g becomes |__x| g(f(__x))
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Compose,
        right: Box::new(Expr::Identifier("g".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "|__x| g(f(__x))");
}

#[test]
fn test_gen_compose_chained() {
    let gen = RustCodegen::new();
    // f >> g >> h becomes |__x| h((|__x| g(f(__x)))(__x))
    let inner_compose = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Compose,
        right: Box::new(Expr::Identifier("g".to_string())),
    };
    let outer_compose = Expr::Binary {
        left: Box::new(inner_compose),
        op: BinaryOp::Compose,
        right: Box::new(Expr::Identifier("h".to_string())),
    };
    assert_eq!(
        gen.gen_expr(&outer_compose),
        "|__x| h((|__x| g(f(__x)))(__x))"
    );
}

#[test]
fn test_gen_apply_operator() {
    let gen = RustCodegen::new();
    // f @ x becomes f(x)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Apply,
        right: Box::new(Expr::Identifier("x".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "f(x)");
}

#[test]
fn test_gen_apply_with_literal() {
    let gen = RustCodegen::new();
    // square @ 5 becomes square(5)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("square".to_string())),
        op: BinaryOp::Apply,
        right: Box::new(Expr::Literal(Literal::Int(5))),
    };
    assert_eq!(gen.gen_expr(&expr), "square(5)");
}

#[test]
fn test_gen_implies_operator() {
    let gen = RustCodegen::new();
    // a => b becomes (!a || b)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("a".to_string())),
        op: BinaryOp::Implies,
        right: Box::new(Expr::Identifier("b".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "(!a || b)");
}

#[test]
fn test_gen_implies_with_expressions() {
    let gen = RustCodegen::new();
    // (x > 0) => (y > 0) becomes (!(x > 0) || (y > 0))
    let left = Expr::Binary {
        left: Box::new(Expr::Identifier("x".to_string())),
        op: BinaryOp::Gt,
        right: Box::new(Expr::Literal(Literal::Int(0))),
    };
    let right = Expr::Binary {
        left: Box::new(Expr::Identifier("y".to_string())),
        op: BinaryOp::Gt,
        right: Box::new(Expr::Literal(Literal::Int(0))),
    };
    let expr = Expr::Binary {
        left: Box::new(left),
        op: BinaryOp::Implies,
        right: Box::new(right),
    };
    assert_eq!(gen.gen_expr(&expr), "(!(x > 0) || (y > 0))");
}

#[test]
fn test_gen_bind_operator() {
    let gen = RustCodegen::new();
    // m := f becomes m.and_then(f)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("m".to_string())),
        op: BinaryOp::Bind,
        right: Box::new(Expr::Identifier("f".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "m.and_then(f)");
}

#[test]
fn test_gen_bind_with_lambda() {
    let gen = RustCodegen::new();
    // option_value := |x| Some(x + 1) becomes option_value.and_then(|x| { Some((x + 1)) })
    let lambda = Expr::Lambda {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Call {
            callee: Box::new(Expr::Identifier("Some".to_string())),
            args: vec![Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Literal(Literal::Int(1))),
            }],
        }),
    };
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("option_value".to_string())),
        op: BinaryOp::Bind,
        right: Box::new(lambda),
    };
    assert_eq!(
        gen.gen_expr(&expr),
        "option_value.and_then(|x| { Some((x + 1)) })"
    );
}

#[test]
fn test_gen_map_operator() {
    let gen = RustCodegen::new();
    // f <$> m becomes m.map(f)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Map,
        right: Box::new(Expr::Identifier("m".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "m.map(f)");
}

#[test]
fn test_gen_map_with_lambda() {
    let gen = RustCodegen::new();
    // (|x| x * 2) <$> vec becomes vec.map(|x| { (x * 2) })
    let lambda = Expr::Lambda {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::Int(2))),
        }),
    };
    let expr = Expr::Binary {
        left: Box::new(lambda),
        op: BinaryOp::Map,
        right: Box::new(Expr::Identifier("vec".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "vec.map(|x| { (x * 2) })");
}

#[test]
fn test_gen_ap_operator() {
    let gen = RustCodegen::new();
    // mf <*> mx becomes /* applicative apply */ mf.ap(mx)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("mf".to_string())),
        op: BinaryOp::Ap,
        right: Box::new(Expr::Identifier("mx".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "/* applicative apply */ mf.ap(mx)");
}

#[test]
fn test_gen_pow_operator() {
    let gen = RustCodegen::new();
    // x ^ y becomes x.pow(y as u32)
    let expr = Expr::Binary {
        left: Box::new(Expr::Identifier("x".to_string())),
        op: BinaryOp::Pow,
        right: Box::new(Expr::Identifier("y".to_string())),
    };
    assert_eq!(gen.gen_expr(&expr), "x.pow(y as u32)");
}

#[test]
fn test_gen_pow_with_literals() {
    let gen = RustCodegen::new();
    // 2 ^ 8 becomes 2.pow(8 as u32)
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Int(2))),
        op: BinaryOp::Pow,
        right: Box::new(Expr::Literal(Literal::Int(8))),
    };
    assert_eq!(gen.gen_expr(&expr), "2.pow(8 as u32)");
}

#[test]
fn test_gen_pow_in_expression() {
    let gen = RustCodegen::new();
    // (x + 1) ^ 2 becomes (x + 1).pow(2 as u32)
    let base = Expr::Binary {
        left: Box::new(Expr::Identifier("x".to_string())),
        op: BinaryOp::Add,
        right: Box::new(Expr::Literal(Literal::Int(1))),
    };
    let expr = Expr::Binary {
        left: Box::new(base),
        op: BinaryOp::Pow,
        right: Box::new(Expr::Literal(Literal::Int(2))),
    };
    assert_eq!(gen.gen_expr(&expr), "(x + 1).pow(2 as u32)");
}

#[test]
fn test_gen_pipe_and_compose_together() {
    let gen = RustCodegen::new();
    // x |> (f >> g) becomes (|__x| g(f(__x)))(x)
    let compose = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Compose,
        right: Box::new(Expr::Identifier("g".to_string())),
    };
    let pipe = Expr::Binary {
        left: Box::new(Expr::Identifier("x".to_string())),
        op: BinaryOp::Pipe,
        right: Box::new(compose),
    };
    assert_eq!(gen.gen_expr(&pipe), "(|__x| g(f(__x)))(x)");
}

#[test]
fn test_gen_map_bind_chain() {
    let gen = RustCodegen::new();
    // (f <$> m) := g becomes m.map(f).and_then(g)
    let map_expr = Expr::Binary {
        left: Box::new(Expr::Identifier("f".to_string())),
        op: BinaryOp::Map,
        right: Box::new(Expr::Identifier("m".to_string())),
    };
    let bind_expr = Expr::Binary {
        left: Box::new(map_expr),
        op: BinaryOp::Bind,
        right: Box::new(Expr::Identifier("g".to_string())),
    };
    assert_eq!(gen.gen_expr(&bind_expr), "m.map(f).and_then(g)");
}

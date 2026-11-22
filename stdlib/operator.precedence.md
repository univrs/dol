# Operator Precedence Specification

> **Design spec — not implemented.** There is no DOL standard library in dol 0.8.2: no stdlib `.dol` or `.rs` sources exist in `src/` or `stdlib/`, and the compiler loads none. This directory contains only Markdown specs. Code samples here may use syntax the 0.8.2 parser does not accept (see `docs/DOL-STATUS-2026-11.md` §5.4 and §7).
>
> The precedence table the parser actually uses is in `src/pratt.rs` (`infix_binding_power`, `prefix_binding_power`). Where this spec and that file disagree, the code is authoritative.

> Reference for DOL parser implementers defining operator binding strength and associativity.

## Precedence Table (Lowest to Highest)

| Level | Operator | Name | Associativity | Example |
|-------|----------|------|---------------|---------|
| 1 | `:=` | Assignment | Right | `x := y := 0` → `x := (y := 0)` |
| 2 | `\|>` | Pipe | Left | `a \|> b \|> c` → `(a \|> b) \|> c` |
| 3 | `@` | Application | Left | `f @ x @ y` → `(f @ x) @ y` |
| 4 | `>>` | Compose | Right | `f >> g >> h` → `f >> (g >> h)` |
| 5 | `->` | Arrow/Maps | Right | `A -> B -> C` → `A -> (B -> C)` |
| 6 | `\|` | Alternative | Left | `a \| b \| c` → `(a \| b) \| c` |
| 7 | `&` | Conjunction | Left | `a & b & c` → `(a & b) & c` |
| 8 | `==` `!=` | Equality | None | `a == b == c` → Error (no chaining) |
| 9 | `<` `>` `<=` `>=` | Comparison | None | `a < b < c` → Error |
| 10 | `+` `-` | Additive | Left | `a - b + c` → `(a - b) + c` |
| 11 | `*` `/` `%` | Multiplicative | Left | `a / b * c` → `(a / b) * c` |
| 12 | `^` | Power | Right | `a ^ b ^ c` → `a ^ (b ^ c)` |
| 13 | `!` `-` (unary) | Prefix | Right | `!!a` → `!(!a)` |
| 14 | `.` `[]` `()` | Postfix | Left | `a.b.c` → `(a.b).c` |

## Parse Tree Example

Expression: `a |> f >> g @ x := h`

```
        :=
       /  \
     |>    h
    /  \
   a    @
       / \
      >>  x
     /  \
    f    g
```

**Parsing steps:**
1. `:=` binds loosest → splits into LHS `a |> f >> g @ x` and RHS `h`
2. `|>` next → splits LHS into `a` and `f >> g @ x`
3. `@` next → splits into `f >> g` and `x`
4. `>>` tightest → composes `f` and `g`

**Evaluation order (left to right through tree):**
1. Compose `f` and `g` → `f >> g`
2. Apply composed function at `x` → `(f >> g) @ x`
3. Pipe `a` through result → `a |> ((f >> g) @ x)`
4. Assign to `h` → `h := a |> ((f >> g) @ x)`

## Operator Semantics

### Pipe (`|>`)

```dol
-- Left-to-right data flow
a |> f |> g
-- Equivalent to: g(f(a))
-- Read as: "a, then f, then g"
```

### Compose (`>>`)

```dol
-- Right-to-left function composition
f >> g >> h
-- Equivalent to: λx. h(g(f(x)))
-- Read as: "f, then g, then h" (applied to future input)
```

### Application (`@`)

```dol
-- Explicit function application (useful with composition)
f >> g @ x
-- Equivalent to: (f >> g)(x)
-- Read as: "compose f and g, apply to x"
```

### Assignment (`:=`)

```dol
-- Bind result to name
result := input |> transform
-- Read as: "result is defined as input piped through transform"
```

## Ambiguity Resolution

### Pipe vs Compose

```dol
-- These are different!
a |> f >> g      -- (a |> f) >> g  ← WRONG (type error: value >> function)
a |> (f >> g)    -- Explicit: pipe a through composed f>>g

-- Parser treats as:
a |> f >> g      -- a |> (f >> g)  ← Because >> binds tighter
```

### Application Binding

```dol
-- @ binds tighter than |> but looser than >>
a |> f @ x       -- a |> (f @ x)     ← apply f to x, then pipe a
f >> g @ x       -- (f >> g) @ x     ← compose, then apply to x
```

### Assignment Scope

```dol
-- := binds loosest, captures entire RHS
x := a |> f >> g @ y
-- Parses as: x := (a |> (f >> g) @ y)
-- NOT as: (x := a) |> ...
```

## Implementation Notes

### Pratt Parser Binding Powers

```rust
fn prefix_binding_power(op: &Token) -> u8 {
    match op {
        Token::Not | Token::Minus => 130,  // unary
        _ => 0,
    }
}

fn infix_binding_power(op: &Token) -> Option<(u8, u8)> {
    // Returns (left_bp, right_bp)
    // left < right = right associative
    // left > right = left associative
    Some(match op {
        Token::Assign     => (10, 9),   // := right assoc
        Token::Pipe       => (21, 20),  // |> left assoc
        Token::At         => (31, 30),  // @  left assoc
        Token::Compose    => (40, 41),  // >> right assoc
        Token::Arrow      => (50, 51),  // -> right assoc
        Token::Or         => (61, 60),  // |  left assoc
        Token::And        => (71, 70),  // &  left assoc
        Token::Eq | Token::Ne => (80, 80),  // == != non-assoc
        Token::Lt | Token::Gt | Token::Le | Token::Ge => (90, 90),
        Token::Plus | Token::Minus => (101, 100),
        Token::Star | Token::Slash | Token::Percent => (111, 110),
        Token::Caret      => (120, 121), // ^ right assoc
        Token::Dot        => (141, 140), // . left assoc
        _ => return None,
    })
}
```

### Parenthesization for Debugging

When implementing, add a `--show-parens` flag:

```
Input:  a |> f >> g @ x := h
Output: (((a |> ((f >> g) @ x)) := h))
```

## Test Cases

```dol
-- Basic precedence
assert_parse "a |> f >> g" as "a |> (f >> g)"
assert_parse "f >> g @ x" as "(f >> g) @ x"
assert_parse "x := a |> f" as "x := (a |> f)"

-- Complex expression
assert_parse "a |> f >> g @ x := h" as "((a |> ((f >> g) @ x)) := h)"

-- Associativity
assert_parse "a |> b |> c" as "(a |> b) |> c"
assert_parse "f >> g >> h" as "f >> (g >> h)"
assert_parse "x := y := z" as "x := (y := z)"

-- Error cases
assert_error "a < b < c"  -- non-associative comparison
assert_error "a == b == c" -- non-associative equality
```

## Visual Precedence Guide

```
LOOSEST (binds last)
    │
    │   :=          assignment
    │   |>          pipe (data flows left-to-right)
    │   @           application
    │   >>          compose (functions combine right-to-left)
    │   ->          arrow/function type
    │   |           alternative/or
    │   &           conjunction/and
    │   == !=       equality
    │   < > <= >=   comparison
    │   + -         addition
    │   * / %       multiplication
    │   ^           exponentiation
    │   ! - (unary) prefix operators
    │   . [] ()     postfix/access
    │
TIGHTEST (binds first)
```

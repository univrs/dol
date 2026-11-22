# Applicative Functor Specification

> **Design spec — not implemented.** There is no DOL standard library in dol 0.8.2: no stdlib `.dol` or `.rs` sources exist in `src/` or `stdlib/`, and the compiler loads none. This directory contains only Markdown specs. Code samples here may use syntax the 0.8.2 parser does not accept (see `docs/DOL-STATUS-2026-11.md` §5.4 and §7).

> Extends Functor with function application in a context.
> Uses extended DOL syntax with generics.

## Trait Definition

```dol
trait Applicative<F> {
  requires Functor<F>

  requires pure: Function<A, F<A>>
  requires apply: Function<(F<Function<A, B>>, F<A>), F<B>>
}
```

### Alternative Syntax (Operator Form)

```dol
trait Applicative<F> {
  requires Functor<F>

  -- Lift value into context
  function pure<A>(a: A) -> F<A>

  -- Apply wrapped function to wrapped value
  function (<*>)<A, B>(ff: F<Function<A, B>>, fa: F<A>) -> F<B>

  -- Lift binary function and apply (derived)
  function liftA2<A, B, C>(f: Function<(A, B), C>, fa: F<A>, fb: F<B>) -> F<C> {
    pure(f) <*> fa <*> fb
  }
}
```

## Implementations

### Optional

```dol
implement Applicative for Optional {
  function pure<A>(a: A) -> Optional<A> {
    Some(a)
  }

  function apply<A, B>(ff: Optional<Function<A, B>>, fa: Optional<A>) -> Optional<B> {
    match (ff, fa) {
      (Some(f), Some(a)) => Some(f(a))
      _ => None
    }
  }
}
```

**Examples:**
```dol
-- Applying wrapped function
let add_one: Optional<Function<Int, Int>> = Some(|x| x + 1)
let value: Optional<Int> = Some(5)
apply(add_one, value)  -- Some(6)

-- With None
apply(None, Some(5))   -- None
apply(Some(f), None)   -- None

-- Lifting and applying
let add: Function<(Int, Int), Int> = |a, b| a + b
liftA2(add, Some(3), Some(4))  -- Some(7)
liftA2(add, Some(3), None)     -- None
```

### List

```dol
implement Applicative for List {
  function pure<A>(a: A) -> List<A> {
    [a]
  }

  function apply<A, B>(ff: List<Function<A, B>>, fa: List<A>) -> List<B> {
    -- Cartesian product: apply each function to each value
    ff.flat_map(|f| fa.map(|a| f(a)))
  }
}
```

**Examples:**
```dol
-- Multiple functions, multiple values
let funcs = [|x| x + 1, |x| x * 2]
let values = [1, 2, 3]
apply(funcs, values)  -- [2, 3, 4, 2, 4, 6]

-- Combining lists
liftA2(|a, b| (a, b), [1, 2], ["a", "b"])
-- [(1, "a"), (1, "b"), (2, "a"), (2, "b")]
```

### Result

```dol
implement Applicative for Result<E> {
  function pure<A>(a: A) -> Result<E, A> {
    Ok(a)
  }

  function apply<A, B>(ff: Result<E, Function<A, B>>, fa: Result<E, A>) -> Result<E, B> {
    match (ff, fa) {
      (Ok(f), Ok(a)) => Ok(f(a))
      (Err(e), _) => Err(e)
      (_, Err(e)) => Err(e)
    }
  }
}
```

**Examples:**
```dol
-- Validating multiple fields
let validate_name: Result<Error, String> = Ok("Alice")
let validate_age: Result<Error, Int> = Ok(30)

let make_user = |name, age| User { name, age }
liftA2(make_user, validate_name, validate_age)  -- Ok(User { name: "Alice", age: 30 })

-- First error wins
let bad_name: Result<Error, String> = Err("Name required")
liftA2(make_user, bad_name, validate_age)  -- Err("Name required")
```

### Validation (Accumulating Errors)

```dol
implement Applicative for Validation<List<E>> {
  function pure<A>(a: A) -> Validation<List<E>, A> {
    Valid(a)
  }

  function apply<A, B>(
    ff: Validation<List<E>, Function<A, B>>,
    fa: Validation<List<E>, A>
  ) -> Validation<List<E>, B> {
    match (ff, fa) {
      (Valid(f), Valid(a)) => Valid(f(a))
      (Invalid(e1), Invalid(e2)) => Invalid(e1 ++ e2)  -- Accumulate!
      (Invalid(e), _) => Invalid(e)
      (_, Invalid(e)) => Invalid(e)
    }
  }
}
```

**Examples:**
```dol
-- Collect ALL validation errors
let bad_name = Invalid(["Name required"])
let bad_age = Invalid(["Age must be positive"])

liftA2(make_user, bad_name, bad_age)
-- Invalid(["Name required", "Age must be positive"])
```

### Function (Reader)

```dol
implement Applicative for Function<R, _> {
  function pure<A>(a: A) -> Function<R, A> {
    |_| a  -- Constant function
  }

  function apply<A, B>(
    ff: Function<R, Function<A, B>>,
    fa: Function<R, A>
  ) -> Function<R, B> {
    |r| ff(r)(fa(r))  -- Apply both to same environment
  }
}
```

### Gene (with Constraint Transfer)

```dol
implement Applicative for Gene {
  function pure<A>(a: A) -> Gene<A> {
    Gene {
      value: a,
      constraints: []  -- Minimal context
    }
  }

  function apply<A, B>(
    ff: Gene<Function<A, B>>,
    fa: Gene<A>
  ) -> Gene<B> {
    Gene {
      value: ff.value(fa.value),
      constraints: merge_constraints(ff.constraints, fa.constraints)
    }
  }
}
```

## Laws

Every Applicative implementation must satisfy:

### 1. Identity

```dol
apply(pure(id), v) == v

-- Example with Optional:
apply(Some(|x| x), Some(5)) == Some(5)  ✓
```

### 2. Composition

```dol
apply(apply(apply(pure(compose), u), v), w) == apply(u, apply(v, w))

-- Functions compose correctly within the context
```

### 3. Homomorphism

```dol
apply(pure(f), pure(x)) == pure(f(x))

-- Example with Optional:
apply(Some(|x| x + 1), Some(5)) == Some(6)
pure((|x| x + 1)(5)) == Some(6)  ✓
```

### 4. Interchange

```dol
apply(u, pure(y)) == apply(pure(|f| f(y)), u)

-- Applying to pure is same as mapping application
```

## Derived Operations

```dol
-- Sequence: evaluate left, then right, keep right
function (*>)<A, B>(fa: F<A>, fb: F<B>) -> F<B> {
  liftA2(|_, b| b, fa, fb)
}

-- Sequence: evaluate left, then right, keep left
function (<*)<A, B>(fa: F<A>, fb: F<B>) -> F<A> {
  liftA2(|a, _| a, fa, fb)
}

-- Lift ternary function
function liftA3<A, B, C, D>(
  f: Function<(A, B, C), D>,
  fa: F<A>, fb: F<B>, fc: F<C>
) -> F<D> {
  pure(f) <*> fa <*> fb <*> fc
}

-- When: conditional execution
function when(condition: Bool, action: F<Unit>) -> F<Unit> {
  if condition then action else pure(())
}

-- Unless: inverse conditional
function unless(condition: Bool, action: F<Unit>) -> F<Unit> {
  when(!condition, action)
}
```

## Relationship to Other Traits

```
         Functor
            │
            ▼
       Applicative
            │
            ▼
          Monad
```

```dol
-- Every Applicative is a Functor
-- map can be derived from pure and apply:
function map<A, B>(f: Function<A, B>, fa: F<A>) -> F<B> {
  apply(pure(f), fa)
}

-- Every Monad is an Applicative
-- apply can be derived from bind:
function apply<A, B>(ff: F<Function<A, B>>, fa: F<A>) -> F<B> {
  ff.bind(|f| fa.map(f))
}
```

## Use Cases

### 1. Parallel Validation

```dol
-- Validate form fields independently, collect all errors
let form_result = liftA4(
  make_form,
  validate_name(input.name),      -- May fail
  validate_email(input.email),    -- May fail independently
  validate_age(input.age),        -- May fail independently
  validate_password(input.pass)   -- May fail independently
)
-- Returns all errors, not just first one (with Validation)
```

### 2. Parallel Computation

```dol
-- Both computations can run in parallel
let result = liftA2(
  combine_results,
  fetch_user_async(user_id),    -- Async<User>
  fetch_orders_async(user_id)   -- Async<List<Order>>
)
-- Async<CombinedResult>
```

### 3. Configuration Composition

```dol
-- Build config from environment
let config: Reader<Env, Config> = liftA3(
  make_config,
  read_db_url,      -- Reader<Env, String>
  read_port,        -- Reader<Env, Int>
  read_log_level    -- Reader<Env, LogLevel>
)
```

### 4. Parser Combination

```dol
-- Parse structured data
let parse_point: Parser<Point> = liftA2(
  |x, y| Point { x, y },
  parse_int <* char(','),  -- Parse int, consume comma
  parse_int                 -- Parse second int
)
```

## Implementation Checklist

- [ ] Implement `pure` - lift value into minimal context
- [ ] Implement `apply` (or `<*>`) - apply wrapped function
- [ ] Verify identity law
- [ ] Verify composition law
- [ ] Verify homomorphism law
- [ ] Verify interchange law
- [ ] Derive `liftA2`, `liftA3` for convenience
- [ ] Derive `*>` and `<*` for sequencing

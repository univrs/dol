# Monad Specification

> **Design spec — not implemented.** There is no DOL standard library in dol 0.8.2: no stdlib `.dol` or `.rs` sources exist in `src/` or `stdlib/`, and the compiler loads none. This directory contains only Markdown specs. Code samples here may use syntax the 0.8.2 parser does not accept (see `docs/DOL-STATUS-2026-11.md` §5.4 and §7).

> Extends Applicative with sequential, context-dependent computation.
> Uses extended DOL syntax with generics.

## Trait Definition

```dol
trait Monad<M> {
  requires Applicative<M>

  requires bind: Function<(M<A>, Function<A, M<B>>), M<B>>

  -- Alternatively defined via join
  requires join: Function<M<M<A>>, M<A>>
}
```

### Alternative Syntax (Operator Form)

```dol
trait Monad<M> {
  requires Applicative<M>

  -- Bind: chain computations
  function (>>=)<A, B>(ma: M<A>, f: Function<A, M<B>>) -> M<B>

  -- Reverse bind (flip arguments)
  function (=<<)<A, B>(f: Function<A, M<B>>, ma: M<A>) -> M<B> {
    ma >>= f
  }

  -- Kleisli composition
  function (>=>)<A, B, C>(
    f: Function<A, M<B>>,
    g: Function<B, M<C>>
  ) -> Function<A, M<C>> {
    |a| f(a) >>= g
  }

  -- Join: flatten nested context
  function join<A>(mma: M<M<A>>) -> M<A> {
    mma >>= id
  }

  -- Derived from Applicative
  function pure<A>(a: A) -> M<A>
}
```

## Interdefinability

```dol
-- bind and join are interdefinable:

function bind<A, B>(ma: M<A>, f: Function<A, M<B>>) -> M<B> {
  join(map(ma, f))
}

function join<A>(mma: M<M<A>>) -> M<A> {
  bind(mma, id)
}

-- Minimal complete definition: pure + (bind OR join)
```

## Implementations

### Optional

```dol
implement Monad for Optional {
  function bind<A, B>(ma: Optional<A>, f: Function<A, Optional<B>>) -> Optional<B> {
    match ma {
      Some(a) => f(a)
      None => None
    }
  }

  function join<A>(mma: Optional<Optional<A>>) -> Optional<A> {
    match mma {
      Some(Some(a)) => Some(a)
      _ => None
    }
  }
}
```

**Examples:**
```dol
-- Chaining optional operations
let user: Optional<User> = find_user(id)
let address: Optional<Address> = user >>= |u| u.address
let city: Optional<String> = address >>= |a| a.city

-- Short-circuit on None
find_user(id) >>= get_address >>= get_city
-- If any step returns None, whole chain is None

-- Do notation (syntactic sugar)
do {
  user <- find_user(id)
  address <- user.address
  city <- address.city
  pure(city)
}
```

### List

```dol
implement Monad for List {
  function bind<A, B>(ma: List<A>, f: Function<A, List<B>>) -> List<B> {
    ma.flat_map(f)
  }

  function join<A>(mma: List<List<A>>) -> List<A> {
    mma.flatten()
  }
}
```

**Examples:**
```dol
-- Non-deterministic computation
let pairs = [1, 2, 3] >>= |x|
            [4, 5] >>= |y|
            pure((x, y))
-- [(1,4), (1,5), (2,4), (2,5), (3,4), (3,5)]

-- List comprehension equivalent
do {
  x <- [1, 2, 3]
  y <- [4, 5]
  pure((x, y))
}

-- Filtering with guard
do {
  x <- [1, 2, 3, 4, 5]
  guard(x % 2 == 0)  -- [] if false, [()] if true
  pure(x * x)
}
-- [4, 16]
```

### Result

```dol
implement Monad for Result<E> {
  function bind<A, B>(ma: Result<E, A>, f: Function<A, Result<E, B>>) -> Result<E, B> {
    match ma {
      Ok(a) => f(a)
      Err(e) => Err(e)
    }
  }

  function join<A>(mma: Result<E, Result<E, A>>) -> Result<E, A> {
    match mma {
      Ok(Ok(a)) => Ok(a)
      Ok(Err(e)) => Err(e)
      Err(e) => Err(e)
    }
  }
}
```

**Examples:**
```dol
-- Error propagation
let result = parse_int(input) >>= |n|
             validate_positive(n) >>= |n|
             divide(100, n)

-- Do notation with early return
do {
  n <- parse_int(input)
  validated <- validate_positive(n)
  result <- divide(100, validated)
  pure(result)
}

-- The ? operator is syntax sugar for bind + early return
fn process(input: String) -> Result<Error, Int> {
  let n = parse_int(input)?          -- Early return on Err
  let validated = validate_positive(n)?
  let result = divide(100, validated)?
  Ok(result)
}
```

### IO

```dol
implement Monad for IO {
  function bind<A, B>(ma: IO<A>, f: Function<A, IO<B>>) -> IO<B> {
    IO(|| {
      let a = ma.run()
      f(a).run()
    })
  }

  function pure<A>(a: A) -> IO<A> {
    IO(|| a)
  }
}
```

**Examples:**
```dol
-- Sequencing side effects
let program: IO<Unit> = do {
  name <- read_line()
  _ <- print("Hello, ")
  _ <- print_line(name)
  pure(())
}

-- Effects only happen when run
program.run()  -- Actually performs I/O
```

### State

```dol
implement Monad for State<S> {
  function bind<A, B>(ma: State<S, A>, f: Function<A, State<S, B>>) -> State<S, B> {
    State(|s| {
      let (a, s1) = ma.run(s)
      f(a).run(s1)
    })
  }

  function pure<A>(a: A) -> State<S, A> {
    State(|s| (a, s))
  }
}

-- State operations
function get<S>() -> State<S, S> {
  State(|s| (s, s))
}

function put<S>(s: S) -> State<S, Unit> {
  State(|_| ((), s))
}

function modify<S>(f: Function<S, S>) -> State<S, Unit> {
  State(|s| ((), f(s)))
}
```

**Examples:**
```dol
-- Stateful computation
let counter: State<Int, String> = do {
  n <- get()
  _ <- put(n + 1)
  m <- get()
  pure("Incremented from " ++ show(n) ++ " to " ++ show(m))
}

counter.run(0)  -- ("Incremented from 0 to 1", 1)
```

### Reader

```dol
implement Monad for Reader<R> {
  function bind<A, B>(ma: Reader<R, A>, f: Function<A, Reader<R, B>>) -> Reader<R, B> {
    Reader(|r| f(ma.run(r)).run(r))
  }

  function pure<A>(a: A) -> Reader<R, A> {
    Reader(|_| a)
  }
}

-- Reader operations
function ask<R>() -> Reader<R, R> {
  Reader(|r| r)
}

function asks<R, A>(f: Function<R, A>) -> Reader<R, A> {
  Reader(f)
}

function local<R, A>(f: Function<R, R>, ma: Reader<R, A>) -> Reader<R, A> {
  Reader(|r| ma.run(f(r)))
}
```

**Examples:**
```dol
-- Dependency injection
let app: Reader<Config, Response> = do {
  config <- ask()
  db <- asks(|c| c.database)
  user <- fetch_user(db, user_id)
  pure(render(user))
}

app.run(production_config)
```

### Writer

```dol
implement Monad for Writer<W> where W: Monoid {
  function bind<A, B>(ma: Writer<W, A>, f: Function<A, Writer<W, B>>) -> Writer<W, B> {
    let (a, w1) = ma.run()
    let (b, w2) = f(a).run()
    Writer((b, w1 <> w2))  -- Combine logs
  }

  function pure<A>(a: A) -> Writer<W, A> {
    Writer((a, W.empty))
  }
}

-- Writer operations
function tell<W>(w: W) -> Writer<W, Unit> {
  Writer(((), w))
}

function listen<W, A>(ma: Writer<W, A>) -> Writer<W, (A, W)> {
  let (a, w) = ma.run()
  Writer(((a, w), w))
}
```

**Examples:**
```dol
-- Logging computation
let computation: Writer<List<String>, Int> = do {
  _ <- tell(["Starting computation"])
  let x = 10
  _ <- tell(["x = " ++ show(x)])
  let y = x * 2
  _ <- tell(["y = " ++ show(y)])
  pure(y)
}

computation.run()
-- (20, ["Starting computation", "x = 10", "y = 20"])
```

### Parser

```dol
implement Monad for Parser {
  function bind<A, B>(pa: Parser<A>, f: Function<A, Parser<B>>) -> Parser<B> {
    Parser(|input| {
      match pa.parse(input) {
        Ok((a, rest)) => f(a).parse(rest)
        Err(e) => Err(e)
      }
    })
  }

  function pure<A>(a: A) -> Parser<A> {
    Parser(|input| Ok((a, input)))
  }
}
```

**Examples:**
```dol
-- Parse structured data
let parse_pair: Parser<(Int, Int)> = do {
  _ <- char('(')
  x <- integer
  _ <- char(',')
  y <- integer
  _ <- char(')')
  pure((x, y))
}

parse_pair.parse("(123,456)")  -- Ok(((123, 456), ""))
```

### Gene

```dol
implement Monad for Gene {
  function bind<A, B>(ga: Gene<A>, f: Function<A, Gene<B>>) -> Gene<B> {
    let gb = f(ga.value)
    Gene {
      value: gb.value,
      constraints: transfer_constraints(ga.constraints, gb.constraints)
    }
  }

  function pure<A>(a: A) -> Gene<A> {
    Gene { value: a, constraints: [] }
  }
}
```

## Laws

Every Monad implementation must satisfy:

### 1. Left Identity

```dol
pure(a) >>= f  ==  f(a)

-- Wrapping and immediately unwrapping has no effect
-- Example with Optional:
Some(5) >>= (|x| Some(x + 1))  ==  Some(6)
(|x| Some(x + 1))(5)           ==  Some(6)  ✓
```

### 2. Right Identity

```dol
m >>= pure  ==  m

-- Unwrapping and immediately rewrapping has no effect
-- Example with Optional:
Some(5) >>= Some  ==  Some(5)  ✓
```

### 3. Associativity

```dol
(m >>= f) >>= g  ==  m >>= (|x| f(x) >>= g)

-- Chaining order doesn't matter (given same sequencing)
-- Example:
(Some(5) >>= add_one) >>= double
  ==  Some(5) >>= (|x| add_one(x) >>= double)  ✓
```

### Kleisli Laws (Equivalent Formulation)

```dol
-- Left identity
pure >=> f  ==  f

-- Right identity
f >=> pure  ==  f

-- Associativity
(f >=> g) >=> h  ==  f >=> (g >=> h)
```

## Do Notation

Do notation is syntactic sugar for nested binds:

```dol
-- Desugaring rules:

do { x <- ma; rest }    ==  ma >>= |x| do { rest }
do { ma; rest }         ==  ma >>= |_| do { rest }
do { let x = e; rest }  ==  let x = e in do { rest }
do { e }                ==  e
```

**Example Desugaring:**

```dol
-- Do notation:
do {
  x <- foo()
  y <- bar(x)
  let z = x + y
  baz(z)
}

-- Desugars to:
foo() >>= |x|
bar(x) >>= |y|
let z = x + y in
baz(z)
```

## Common Derived Operations

```dol
-- Sequence, discarding first result
function (>>)<A, B>(ma: M<A>, mb: M<B>) -> M<B> {
  ma >>= |_| mb
}

-- Map with monadic function
function mapM<A, B>(f: Function<A, M<B>>, xs: List<A>) -> M<List<B>> {
  match xs {
    [] => pure([])
    [x, ...rest] => do {
      y <- f(x)
      ys <- mapM(f, rest)
      pure([y, ...ys])
    }
  }
}

-- Filter with monadic predicate
function filterM<A>(p: Function<A, M<Bool>>, xs: List<A>) -> M<List<A>> {
  match xs {
    [] => pure([])
    [x, ...rest] => do {
      keep <- p(x)
      ys <- filterM(p, rest)
      pure(if keep then [x, ...ys] else ys)
    }
  }
}

-- Fold with monadic combining function
function foldM<A, B>(f: Function<(B, A), M<B>>, init: B, xs: List<A>) -> M<B> {
  match xs {
    [] => pure(init)
    [x, ...rest] => do {
      acc <- f(init, x)
      foldM(f, acc, rest)
    }
  }
}

-- Execute list of monadic actions
function sequence<A>(mas: List<M<A>>) -> M<List<A>> {
  mapM(id, mas)
}

-- Execute and discard results
function sequence_<A>(mas: List<M<A>>) -> M<Unit> {
  mas.fold(pure(()), |acc, ma| acc >> ma)
}

-- Replicate action n times
function replicateM<A>(n: Int, ma: M<A>) -> M<List<A>> {
  sequence(List.replicate(n, ma))
}

-- Forever loop (for IO)
function forever<A>(ma: M<A>) -> M<Never> {
  ma >> forever(ma)
}

-- When with monadic condition
function whenM(mb: M<Bool>, ma: M<Unit>) -> M<Unit> {
  mb >>= |b| if b then ma else pure(())
}

-- Unless with monadic condition
function unlessM(mb: M<Bool>, ma: M<Unit>) -> M<Unit> {
  mb >>= |b| if b then pure(()) else ma
}

-- Join (flatten)
function join<A>(mma: M<M<A>>) -> M<A> {
  mma >>= id
}

-- Kleisli composition
function (>=>)<A, B, C>(
  f: Function<A, M<B>>,
  g: Function<B, M<C>>
) -> Function<A, M<C>> {
  |a| f(a) >>= g
}

-- Reverse Kleisli composition
function (<=<)<A, B, C>(
  g: Function<B, M<C>>,
  f: Function<A, M<B>>
) -> Function<A, M<C>> {
  f >=> g
}
```

## Monad Transformers (Preview)

```dol
-- Stack monads for combined effects
trait MonadTrans<T> {
  function lift<M, A>(ma: M<A>) -> T<M, A> where M: Monad
}

-- OptionT: Optional + any monad
newtype OptionT<M, A> = M<Optional<A>>

implement MonadTrans for OptionT {
  function lift<M, A>(ma: M<A>) -> OptionT<M, A> {
    OptionT(ma.map(Some))
  }
}

implement Monad for OptionT<M> where M: Monad {
  function bind<A, B>(
    ota: OptionT<M, A>,
    f: Function<A, OptionT<M, B>>
  ) -> OptionT<M, B> {
    OptionT(ota.run >>= |opt| match opt {
      Some(a) => f(a).run
      None => pure(None)
    })
  }
}

-- Example: IO + Optional
let program: OptionT<IO, User> = do {
  input <- lift(read_line())         -- IO effect
  user <- OptionT(find_user(input))  -- Optional effect
  pure(user)
}
```

## Relationship Diagram

```
                    Functor
                       │
                       │ map
                       ▼
                  Applicative
                       │
                       │ pure, <*>
                       ▼
                     Monad
                       │
                       │ bind (>>=)
          ┌────────────┼────────────┐
          ▼            ▼            ▼
     MonadFail    MonadPlus    MonadIO
          │            │            │
          │            │            │
     fail/empty    mzero/mplus   liftIO
```

## Use Cases

### 1. Sequential Validation with Dependencies

```dol
-- Each step depends on previous result
let validate_order: Result<Error, Order> = do {
  user <- validate_user(input.user_id)
  inventory <- check_inventory(input.product_id)
  price <- calculate_price(inventory, user.discount)
  order <- create_order(user, inventory, price)
  pure(order)
}
```

### 2. Recursive Parsing

```dol
-- Parse nested expressions
function parse_expr(): Parser<Expr> {
  parse_atom() >>= |left|
  (parse_binop() >>= |op|
   parse_expr() >>= |right|
   pure(BinOp(op, left, right)))
  <|> pure(left)
}
```

### 3. Stateful Traversal

```dol
-- Number nodes in a tree
function number_tree<A>(tree: Tree<A>) -> State<Int, Tree<(Int, A)>> {
  match tree {
    Leaf(a) => do {
      n <- get()
      _ <- put(n + 1)
      pure(Leaf((n, a)))
    }
    Node(left, right) => do {
      left' <- number_tree(left)
      right' <- number_tree(right)
      pure(Node(left', right'))
    }
  }
}
```

### 4. Effect Composition

```dol
-- Combine Reader (config) + State (counter) + IO (effects)
type App<A> = ReaderT<Config, StateT<Counter, IO, A>>

let program: App<Response> = do {
  config <- ask()
  count <- get()
  _ <- modify(|c| c + 1)
  _ <- lift(lift(log("Request #" ++ show(count))))
  response <- lift(lift(http_get(config.api_url)))
  pure(response)
}
```

## Implementation Checklist

- [ ] Implement `bind` (or `>>=`) - chain computations
- [ ] Implement `pure` (from Applicative)
- [ ] Verify left identity law
- [ ] Verify right identity law
- [ ] Verify associativity law
- [ ] Derive `join` from bind
- [ ] Derive `>>` for sequencing
- [ ] Derive `>=>` for Kleisli composition
- [ ] Implement do-notation desugaring in parser
- [ ] Add common utilities (mapM, filterM, foldM, sequence)

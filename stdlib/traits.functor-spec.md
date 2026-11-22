# Functor Specification

> **Design spec — not implemented.** There is no DOL standard library in dol 0.8.2: no stdlib `.dol` or `.rs` sources exist in `src/` or `stdlib/`, and the compiler loads none. This directory contains only Markdown specs. Code samples here may use syntax the 0.8.2 parser does not accept (see `docs/DOL-STATUS-2026-11.md` §5.4 and §7).

> The foundational abstraction for mapping over containers.
> Uses extended DOL syntax with generics.

## Trait Definition

```dol
trait Functor<F> {
  requires map: Function<(F<A>, Function<A, B>), F<B>>
}
```

### Alternative Syntax (Method Form)

```dol
trait Functor<F> {
  -- Apply function to value inside context
  function map<A, B>(self: F<A>, f: Function<A, B>) -> F<B>

  -- Infix operator (<$>)
  function (<$>)<A, B>(f: Function<A, B>, fa: F<A>) -> F<B> {
    map(fa, f)
  }
}
```

## Implementations

### Optional

```dol
implement Functor for Optional {
  function map<A, B>(self: Optional<A>, f: Function<A, B>) -> Optional<B> {
    match self {
      Some(a) => Some(f(a))
      None => None
    }
  }
}
```

**Examples:**
```dol
map(Some(5), |x| x + 1)     -- Some(6)
map(None, |x| x + 1)        -- None
map(Some("hi"), |s| s.len)  -- Some(2)
```

### List

```dol
implement Functor for List {
  function map<A, B>(self: List<A>, f: Function<A, B>) -> List<B> {
    self.fold_right([], |a, acc| [f(a)] ++ acc)
  }
}
```

**Examples:**
```dol
map([1, 2, 3], |x| x * 2)        -- [2, 4, 6]
map([], |x| x + 1)               -- []
map(["a", "bb"], |s| s.len)      -- [1, 2]
```

### Result

```dol
implement Functor for Result<E> {
  function map<A, B>(self: Result<E, A>, f: Function<A, B>) -> Result<E, B> {
    match self {
      Ok(a) => Ok(f(a))
      Err(e) => Err(e)
    }
  }
}
```

**Examples:**
```dol
map(Ok(5), |x| x * 2)              -- Ok(10)
map(Err("failed"), |x| x * 2)      -- Err("failed")
```

### Function (Reader)

```dol
implement Functor for Function<R, _> {
  function map<A, B>(self: Function<R, A>, f: Function<A, B>) -> Function<R, B> {
    |r| f(self(r))
  }
}
```

**Examples:**
```dol
let get_name: Function<Config, String> = |c| c.name
let get_name_len = map(get_name, |s| s.len)
-- get_name_len(config) returns length of config.name
```

### Pair (maps over second element)

```dol
implement Functor for Pair<A, _> {
  function map<B, C>(self: Pair<A, B>, f: Function<B, C>) -> Pair<A, C> {
    let (a, b) = self
    (a, f(b))
  }
}
```

**Examples:**
```dol
map(("key", 5), |x| x * 2)  -- ("key", 10)
```

### Gene

```dol
implement Functor for Gene {
  function map<A, B>(self: Gene<A>, f: Function<A, B>) -> Gene<B> {
    Gene {
      value: f(self.value),
      constraints: []  -- Note: constraints dropped (see map_preserving)
    }
  }
}
```

**Examples:**
```dol
let g: Gene<Int> = Gene { value: 5, constraints: [positive] }
map(g, |x| x * 2)  -- Gene { value: 10, constraints: [] }
```

### Tree

```dol
implement Functor for Tree {
  function map<A, B>(self: Tree<A>, f: Function<A, B>) -> Tree<B> {
    match self {
      Leaf(a) => Leaf(f(a))
      Node(left, a, right) => Node(
        map(left, f),
        f(a),
        map(right, f)
      )
    }
  }
}
```

### IO

```dol
implement Functor for IO {
  function map<A, B>(self: IO<A>, f: Function<A, B>) -> IO<B> {
    IO { run: || f(self.run()) }
  }
}
```

## Laws

Every Functor implementation must satisfy two laws:

### 1. Identity Law

Mapping the identity function has no effect.

```dol
map(fa, id) == fa

-- Proof for Optional:
map(Some(5), |x| x)
  == Some((|x| x)(5))
  == Some(5)  ✓

map(None, |x| x)
  == None  ✓
```

### 2. Composition Law

Mapping composed functions equals composing mapped functions.

```dol
map(fa, f >> g) == map(map(fa, f), g)

-- Proof for Optional:
let f = |x| x + 1
let g = |x| x * 2
let fa = Some(5)

-- Left side:
map(Some(5), f >> g)
  == map(Some(5), |x| (x + 1) * 2)
  == Some(12)

-- Right side:
map(map(Some(5), f), g)
  == map(Some(6), g)
  == Some(12)  ✓
```

## Derived Operations

```dol
-- Replace all values with constant
function (<$)<A, B>(a: A, fb: F<B>) -> F<A> {
  map(fb, |_| a)
}

-- Void: discard result
function void<A>(fa: F<A>) -> F<Unit> {
  () <$ fa
}

-- Flip arguments
function flip_map<A, B>(fa: F<A>, f: Function<A, B>) -> F<B> {
  map(fa, f)
}

-- Map with index (for indexed containers)
function map_with_index<A, B>(fa: F<A>, f: Function<(Int, A), B>) -> F<B>

-- Unzip mapped pairs
function unzip<A, B>(fab: F<Pair<A, B>>) -> Pair<F<A>, F<B>> {
  (map(fab, fst), map(fab, snd))
}
```

## Functor Composition

Functors compose: if F and G are functors, so is F[G[_]].

```dol
-- Composed functor
implement Functor for Compose<F, G> where F: Functor, G: Functor {
  function map<A, B>(self: F<G<A>>, f: Function<A, B>) -> F<G<B>> {
    F.map(self, |ga| G.map(ga, f))
  }
}
```

**Example:**
```dol
let nested: List<Optional<Int>> = [Some(1), None, Some(3)]
map(nested, |x| x * 2)  -- [Some(2), None, Some(6)]
```

## Contravariant Functor

For types that consume values rather than produce them:

```dol
trait Contravariant<F> {
  function contramap<A, B>(self: F<A>, f: Function<B, A>) -> F<B>
}

-- Laws:
-- contramap(fa, id) == fa
-- contramap(fa, f >> g) == contramap(contramap(fa, g), f)  -- Note: reversed!
```

**Example: Predicate**
```dol
implement Contravariant for Predicate {
  function contramap<A, B>(self: Predicate<A>, f: Function<B, A>) -> Predicate<B> {
    Predicate { test: |b| self.test(f(b)) }
  }
}

let is_even: Predicate<Int> = Predicate { test: |x| x % 2 == 0 }
let string_len_even = contramap(is_even, |s| s.len)
string_len_even.test("hi")    -- true (len 2 is even)
string_len_even.test("hello") -- false (len 5 is odd)
```

## Bifunctor

For types with two type parameters:

```dol
trait Bifunctor<F> {
  function bimap<A, B, C, D>(
    self: F<A, B>,
    f: Function<A, C>,
    g: Function<B, D>
  ) -> F<C, D>

  -- Derived: map over first
  function first<A, B, C>(self: F<A, B>, f: Function<A, C>) -> F<C, B> {
    bimap(self, f, id)
  }

  -- Derived: map over second
  function second<A, B, D>(self: F<A, B>, g: Function<B, D>) -> F<A, D> {
    bimap(self, id, g)
  }
}
```

**Example: Pair**
```dol
implement Bifunctor for Pair {
  function bimap<A, B, C, D>(
    self: Pair<A, B>,
    f: Function<A, C>,
    g: Function<B, D>
  ) -> Pair<C, D> {
    let (a, b) = self
    (f(a), g(b))
  }
}

bimap((1, "hi"), |x| x + 1, |s| s.len)  -- (2, 2)
```

## Profunctor

For types contravariant in first parameter, covariant in second:

```dol
trait Profunctor<P> {
  function dimap<A, B, C, D>(
    self: P<A, B>,
    f: Function<C, A>,  -- contravariant
    g: Function<B, D>   -- covariant
  ) -> P<C, D>
}

implement Profunctor for Function {
  function dimap<A, B, C, D>(
    self: Function<A, B>,
    f: Function<C, A>,
    g: Function<B, D>
  ) -> Function<C, D> {
    |c| g(self(f(c)))
  }
}
```

## Common Patterns

### 1. Mapping Over Nested Structures

```dol
-- Map at each level
let data: List<Optional<Int>> = [Some(1), None, Some(3)]

-- Map outer (list)
map(data, |opt| ...)

-- Map inner (optional) - needs nested map
map(data, |opt| map(opt, |x| x * 2))
-- [Some(2), None, Some(6)]
```

### 2. Transforming Data Pipelines

```dol
let pipeline = users
  |> map(|u| u.orders)      -- List<User> -> List<List<Order>>
  |> map(|orders| map(orders, |o| o.total))  -- -> List<List<Int>>
```

### 3. Functor as Container

Think of Functor as a container that allows transforming contents:

| Type | Container Metaphor |
|------|-------------------|
| Optional | Box that may be empty |
| List | Multi-compartment box |
| Result | Box or error message |
| Function | Deferred computation |
| IO | Side-effect wrapper |

## Relationship to Other Traits

```
                 Functor
                    │
        ┌───────────┼───────────┐
        │           │           │
   Contravariant  Bifunctor  Applicative
                    │           │
               Profunctor     Monad
```

```dol
-- Every Applicative is a Functor
-- map can be derived from pure and apply:
function map<A, B>(fa: F<A>, f: Function<A, B>) -> F<B> {
  apply(pure(f), fa)
}

-- Every Monad is a Functor
-- map can be derived from bind:
function map<A, B>(fa: F<A>, f: Function<A, B>) -> F<B> {
  bind(fa, |a| pure(f(a)))
}
```

## Implementation Checklist

- [ ] Implement `map` for your type
- [ ] Verify identity law: `map(fa, id) == fa`
- [ ] Verify composition law: `map(fa, f >> g) == map(map(fa, f), g)`
- [ ] Add `<$>` operator if using infix notation
- [ ] Derive `void`, `<$` for convenience
- [ ] Consider if Contravariant/Bifunctor also apply
- [ ] Document what happens to metadata (constraints, annotations)

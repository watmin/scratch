# wat-common flavor comparison — Haskell vs ML vs Clojure surfaces — sketch

User direction 2026-05-01, after the lab `wat-clojure-flavor`
draft + scratch 010 (Clojure-emits-wat) landed:

> you mentioned we could maybe support haskell-y for ml-y forms...
> i have basically no familiarity with either of these langs...
>
> what's a few simple demos you can think of that show me haskell
> or ml doing something and how we could mimick that in wat-common
> form?..

This is a design-exploration scratch — not an arc, not a DESIGN.
Captured so the per-flavor naming choices have a place to live
when communities actually start writing flavor packages.

## Why this matters

Per scratch 009 (substrate-fqdn-userspace-shorts), wat-rs
vendors only FQDN. Multiple community-flavored packages can
coexist; each one is an opt-in alias + macro layer over the
canonical substrate. This document tabulates what three of those
flavors look like in practice — Haskell, ML, Clojure — using
three small demos that exercise the load-bearing language
shapes.

## Demo 1 — Look up a key, fall back to default

### Haskell

```haskell
lookupOr :: String -> Map String Int -> Int -> Int
lookupOr key m default_ =
  case Map.lookup key m of
    Just v  -> v
    Nothing -> default_
```

What's distinctive: `Maybe a` (not `Option<T>`); constructors are
`Just` / `Nothing`; lowercase type vars (`a`).

### ML (Standard ML)

```sml
fun lookupOr key m default_ =
  case Map.find (m, key) of
    SOME v => v
  | NONE   => default_
```

What's distinctive: `'a option` (the `'a` is a type variable,
like Rust's `<T>`); constructors are `SOME` / `NONE` (uppercase);
lowercase concrete types (`int`).

### Wat-rs FQDN canonical

```scheme
(:wat::core::define
  (:my::lookup-or
    (key :wat::core::String)
    (m   :wat::core::HashMap<wat::core::String,wat::core::i64>)
    (default-v :wat::core::i64)
    -> :wat::core::i64)
  (:wat::core::match (:wat::core::get m key) -> :wat::core::i64
    ((Some v) v)
    (:None    default-v)))
```

### Wat-common-haskell-flavor

```scheme
(haskell/defn (lookup-or (key :String) (m :Map<String,Int>) (def :Int) -> :Int)
  (case (Map/lookup key m)
    ((Just v)  v)
    (Nothing   def)))
```

### Wat-common-ml-flavor

```scheme
(ml/fun (lookup_or (key :string) (m :map<string,int>) (default :int) -> :int)
  (case (Map.find m key)
    ((SOME v) v)
    (NONE     default)))
```

## Demo 2 — Safe division

### Haskell

```haskell
safeDivide :: Int -> Int -> Either String Int
safeDivide _ 0 = Left "division by zero"
safeDivide x y = Right (x `div` y)
```

What's distinctive: `Either e a` carries the error FIRST in the
type — `:Either<String,Int>` reads "either a String error or an
Int success." Constructors are `Left` (error) and `Right`
(success). This is the **opposite ordering** of Rust/wat's
`Result<T,E>`.

### ML

```sml
fun safeDivide _ 0 = Err "division by zero"
  | safeDivide x y = Ok (x div y)
```

What's distinctive: usually `('a, 'b) result` with `Ok`/`Err`,
ordering matches Rust. Some SML dialects use Left/Right; we'll
go with Ok/Err here.

### Wat-common-haskell-flavor (note Either flip — error first)

```scheme
(haskell/defn (safeDivide (x :Int) (y :Int) -> :Either<String,Int>)
  (if (== y 0) -> :Either<String,Int>
    (Left "division by zero")
    (Right (div x y))))
```

The package's macros translate `Either<E,T>` argument order to
the underlying wat `Result<T,E>` (success-first) at expansion;
constructors `Left` / `Right` map to substrate `Err` / `Ok`. The
flip is purely a surface naming choice.

### Wat-common-ml-flavor

```scheme
(ml/fun (safe_divide (x :int) (y :int) -> :result<int,string>)
  (if (= y 0) -> :result<int,string>
    (Err "division by zero")
    (Ok (div x y))))
```

## Demo 3 — Sum a list recursively

### Haskell

```haskell
sumList :: [Int] -> Int
sumList []     = 0
sumList (x:xs) = x + sumList xs
```

Lists have special syntax: `[Int]` is the type, `[]` is empty,
`(x:xs)` is cons (head + tail). Functions can have multiple
clauses keyed by pattern; the dispatcher matches top-down.

### ML

```sml
fun sumList []        = 0
  | sumList (x::xs)   = x + sumList xs
```

`int list` is the type, `nil` is empty, `(x::xs)` is cons. Same
multi-clause idea, slightly different syntax (`|` separates
clauses; `::` is cons).

### What both languages do

**Pattern-match-on-cons**: a list is either empty or `(head,
rest)`. The recursive case peels one element. wat doesn't have
first-class cons cells (Vec is wat's container; not a linked
list), so the wat-common wrappers lower this to `match` on
`first`/`rest` accessors. The wrapping is the package's job; the
substrate stays Vec-based.

### Wat-common-haskell-flavor (mimicking Haskell list-cons surface)

```scheme
(haskell/defn (sumList (xs :[Int]) -> :Int)
  (case xs
    ([]       0)
    ((x : xs') (+ x (sumList xs')))))
```

The `[]` and `(x : xs')` are macro-pattern forms — at expansion
they lower to `(:wat::core::match xs ...)` with
`(:wat::core::first xs)` and `(:wat::core::rest xs)` calls.
Multi-clause `defn` (with separate empty / cons cases) requires
either a more elaborate macro or a single-`case` body. The
Haskell flavor probably allows both spellings; the macro
collapses to one match.

### Wat-common-ml-flavor

```scheme
(ml/fun (sum_list (xs :int list) -> :int)
  (case xs
    (nil       0)
    ((x :: xs') (+ x (sum_list xs')))))
```

## Reading the demos — what the colon means

Every type annotation in the demos above keeps a leading `:` even
when "Clojure-flavored" or "ML-flavored." That's deliberate.

**The leading `:` on outer-position type annotations is a
permanent substrate requirement** — it's wat's lexer-level
distinction between keyword (global symbol table) and bare symbol
(local binding). Removing it would create ambiguity between "the
type i64" and "a binding named i64 the user introduced two lines
up." Flavor packages can SHORTEN keyword paths via aliases
(`:i64` for `:wat::core::i64`) but can NOT erase the keyword
marker.

So the cleanest a Clojure-flavored `defn` ever gets is something
like:

```scheme
(defn make-adder [(n :i64)] -> :fn(i64)->i64
  (fn [(m :i64)] -> :i64 (+ n m)))
```

Lowercase `:i64` works (post-arc-109 user-space alias for
`:wat::core::i64`). Inner `i64` inside `:fn(...)->i64` works (no
inner colons per arc 115; aliases resolve). `+` gets rewritten by
the macro layer. What stays: the leading `:` on every outer-type
annotation. Three of four colons gone vs raw FQDN; the fourth is
the wat way.

See scratch
`~/work/holon/scratch/2026/04/009-substrate-fqdn-userspace-shorts/NOTES.md`
§ "What's permanent vs what's temporary" for the canonical
permanent / temporary / user-space-reclamation framing.

## Substrate prerequisite for Clojure / Haskell / ML flavors

The Clojure demos above use **paren-wrapped param lists**
(`(defn (lookup-or (key :String) ...) body)`) — but real Clojure
uses **bracket-wrapped param lists** (`(defn lookup-or [key
default] body)`). Same for `{:k v}` map literals and `#{x y z}`
set literals. Haskell's `[a]` list type and ML's `(x::xs)`
cons-pattern have similar bracket needs.

The substrate's reader doesn't currently parse bracket forms. To
support those flavors at full surface fidelity, **one parser arc
adds reader-macro support for `[...]` / `{...}` / `#{...}`** —
substrate emits new AST variants (`WatAST::Bracket` /
`WatAST::Brace` / `WatAST::HashBrace`) without attaching semantic
meaning; user-space macros (per-flavor packages) decide what they
mean.

Until that arc lands, every flavor uses paren-wrapped surfaces
that LOOK like the language but with `(...)` instead of `[...]`.
Less fidelity, still ergonomic.

Canonical roadmap entry:
`~/work/holon/scratch/2026/04/012-wat-as-polyglot-lowering-target/NOTES.md`
§ "Gaps from syntax — reader-level extensions". One arc, multiple
flavors light up.

## The lookup table

Three different audiences, three different reading experiences,
one substrate. This is the core observation behind scratch 009 —
the substrate's vocabulary is canonical FQDN; community packages
layer different naming conventions on top.

| Concept | Haskell name | ML name | Clojure name (009) | wat FQDN |
|---|---|---|---|---|
| Optional value | `Maybe a` / `Just` / `Nothing` | `'a option` / `SOME` / `NONE` | `Maybe<T>` / `Just` / `Nothing` (or kept Option-flavored) | `:wat::core::Option<T>` / `Some` / `:None` |
| Fallible result | `Either e a` / `Left` / `Right` (error FIRST) | `('a,'b) result` / `Ok` / `Err` | `Either<E,T>` (error first) or `Result<T,E>` | `:wat::core::Result<T,E>` / `Ok` / `Err` |
| List type | `[a]` | `'a list` | `[T]` or `:Seq<T>` (Clojure-style) | `:wat::core::Vector<T>` (post-arc-109-1f) |
| Function def | `name :: Type ; name args = body` | `fun name args = body` | `(defn name [args] body)` | `(:wat::core::define (:name args -> :T) body)` |
| Lambda | `\x -> body` | `fn x => body` | `(fn [x] body)` | `(:wat::core::lambda ((x :T) -> :R) body)` |
| Pattern match | `case x of ...` | `case x of ...` | `(case x ...)` | `(:wat::core::match x -> :T ...)` |
| Map type | `Map k v` | `(k,v) Map.map` | `Map<K,V>` or `{}` literal | `:wat::core::HashMap<K,V>` |
| Map lookup | `Map.lookup k m` → `Maybe v` | `Map.find (m, k)` → `'v option` | `(get m k)` → nillable | `:wat::core::get` → `:Option<V>` |
| Sum type | `data Foo = A | B Int | C String` | `datatype foo = A | B of int | C of string` | `(defenum :Foo (A) (B :Int) (C :String))` (Clojure-flavored) | `(:wat::core::enum :Foo (A) (B :wat::core::i64) (C :wat::core::String))` |

## Variant-constructor naming — the substrate stays one shape

Each flavor SPELLS variant constructors differently at the
surface, but they all lower to the same underlying wat enum
machinery. The package's macro layer translates:

```
Just v       (haskell)  →  (Some v)        (substrate)
Nothing      (haskell)  →  :None           (substrate)
SOME v       (ml)       →  (Some v)        (substrate)
NONE         (ml)       →  :None           (substrate)
(Just v)     (clojure)  →  (Some v)        (substrate)
```

So a Haskell-flavored consumer reading a `.wat` file produced by
their flavor sees `Just` / `Nothing` everywhere; the substrate
sees `Some` / `None`. The macro layer is a renamer at compile
time.

## Open questions about the package family

1. **Haskell's currying.** Haskell functions are curried by
   default — `f x y` is `(f x) y`. wat doesn't curry; you write
   `(f x y)`. The Haskell flavor probably skips currying as a
   surface convenience (would require partial-application
   substrate support). Stays uncurried, just spells like
   Haskell.
2. **Haskell's lazy evaluation.** Haskell is lazy by default;
   wat is eager. The Haskell flavor doesn't fake laziness —
   eager semantics. Reading a Haskell-flavored wat program with
   Haskell-eyes might trip you on side-effect ordering. Documented
   as a known limitation.
3. **ML modules.** ML has a powerful module system (`structure`,
   `signature`, `functor`). wat doesn't have higher-kinded
   modules. The ML flavor probably uses `:my::module::*` keyword
   paths in place of ML structures. Functors retire; documented
   as a limitation.
4. **`do` notation (Haskell) / monadic syntax.** Probably
   skipped in v1 — these need substrate-level monad support
   that wat doesn't have. The Haskell flavor stays at the
   "looks like ADT-using Haskell" surface, not monadic Haskell.
5. **Pattern guards.** Both Haskell (`| cond = ...`) and ML
   (`case x of ... when cond => ...` in newer SML) support
   guarded patterns. wat's `match` has no guards (arc-098
   mention?). Either the flavor packages document the gap, or a
   future arc adds guard support to wat's match.
6. **Operator precedence + custom operators.** Haskell defines
   precedence with `infixl 6 +`. wat parses without precedence
   (everything is parenthesized). Both Haskell and ML's
   look-and-feel rely on infix; wat is purely prefix. The
   flavors don't fix this — they expose `(+ x y)` not `(x + y)`.
   Documented as an unavoidable substrate constraint.

## What the community decides

Each flavor is one wat package living under its OWN top-level
keyword namespace (NOT under `:wat::*` — that's substrate-
reserved). Multiple flavors coexist:

- `wat-common-haskell-flavor` — provides `:haskell::*` aliases
  and macros: `:haskell::Maybe<T>`, `:haskell::Either<E,T>`,
  `:haskell::List<T>`, `:haskell::defn`, etc.
- `wat-common-ml-flavor` — provides `:ml::*`: `:ml::option`,
  `:ml::list`, `:ml::result`, `:ml::fun`, etc.
- `wat-common-clojure-flavor` — provides `:clojure::*`:
  `:clojure::Map<K,V>`, `:clojure::Set<T>`, `:clojure::defn`,
  `:clojure::reduce`, etc.
- `wat-common-erlang-flavor` — provides `:erlang::*`:
  `:erlang::receive`, `:erlang::spawn`, `:erlang::!`, etc.

Different shops pick the package matching their team's
background. Substrate stays canonical (`:wat::*`); each flavor's
ergonomic surface lives under its own namespace.

**Cross-flavor calls work without FFI** — see scratch 012
§ "Cross-flavor calls — no FFI boundary" for the worked example.
Flavor markup is a compile-time thing; once macros expand, the
program is canonical wat AST and inter-flavor function calls are
just ordinary function calls.

## Cross-references

- `~/work/holon/scratch/2026/04/009-substrate-fqdn-userspace-shorts/NOTES.md`
  — the principle. Substrate vendors FQDN; ergonomic surface is
  user-space.
- `~/work/holon/scratch/2026/04/010-clojure-emits-wat/NOTES.md` —
  the orthogonal "different LANGUAGE entirely emits wat" path
  (Clojure source code on the JVM emits wat AST as a build
  artifact). Different from this scratch (which is "wat code that
  reads like X language").
- `holon-lab-trading/docs/drafts/wat-clojure-flavor.md` — the lab
  proof-of-concept of 009; first concrete instance of a flavor
  package.
- `wat-rs/docs/arc/2026/04/109-kill-std/` — the arc that puts the
  substrate in its FQDN-only position. Flavor packages can land
  cleanly once arc 109 closes.

## Status — append more here as the idea matures

- 2026-05-01: scratch captured during arc 109 slice 1f sweep.
  Three small demos in Haskell + ML + wat-common surfaces;
  per-concept lookup table; open questions about per-language
  semantic gaps named. No package yet; this is the "what would
  it look like" sketch for when communities start building
  flavor packages.

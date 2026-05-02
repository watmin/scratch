# wat-common-short-names — design notes (scratch)

**Status:** scratch notes; not yet a real package or doc. Captured
2026-05-01 during arc 109 slice 1e.

## The principle

**The substrate (`wat-rs`) vendors only FQDN.** Every substrate-
provided symbol — types, verbs, operators, kernel primitives — has
exactly one canonical spelling under `:wat::core::*`,
`:wat::kernel::*`, `:wat::io::*`, etc. Verbose, honest, unambiguous,
no shortcuts.

**Ergonomic short names are user-space.** Different communities want
different sugar. We don't ship one — we let the ecosystem grow them.
The first community-flavored package: `wat-common-short-names`,
Clojure-core flavored.

## Why this layering

1. **Substrate stays small + unopinionated about aesthetics.** No
   debate over `+` vs `add` vs `plus`; substrate has `:wat::core::i64::+`
   and walks away.
2. **Naming wars happen at the package level.** Less destructive.
3. **Multiple communities coexist.** Clojure-flavored shop loads
   `wat-common-short-names`. Haskell-flavored shop writes (or loads)
   `wat-common-haskell-style`. ML-flavored shop uses lowercase
   `option`/`result`. All on the same substrate; substrate doesn't
   pick a winner.
4. **Forward compatible.** If Clojure-core flavor proves dominant,
   the substrate doesn't have to change — the package just gets
   popular.
5. **Embedded / minimal users.** Don't load any common-names package;
   write FQDN at every site. The substrate doesn't force ergonomics
   on them.

## What's in `wat-common-short-names`

A wat package (or maybe `(load!)`-able file bundle) that provides
typealiases + macros over the substrate's FQDN surface. Pure
user-space; no Rust changes.

### Types — short aliases

```wat
(:wat::core::typealias :Maybe<T>          :wat::core::Option<T>)
(:wat::core::typealias :Either<E,T>       :wat::core::Result<T,E>)  ;; flipped per Haskell
;; ... or keep Result<T,E> ordering — community choice
(:wat::core::typealias :Map<K,V>          :wat::core::HashMap<K,V>)
(:wat::core::typealias :Set<T>            :wat::core::HashSet<T>)
(:wat::core::typealias :List<T>           :wat::core::Vector<T>)   ;; post-1f rename
(:wat::core::typealias :Bytes             :wat::core::Bytes)        ;; already short
```

### Operators — Clojure-core feel

```wat
(:wat::core::defmacro :+ ...)   ;; expands to :wat::core::i64::+ or polymorphic dispatch
(:wat::core::defmacro :- ...)
(:wat::core::defmacro :* ...)
(:wat::core::defmacro :/ ...)
(:wat::core::defmacro :=  ...)
(:wat::core::defmacro :<  ...)
(:wat::core::defmacro :>  ...)
(:wat::core::defmacro :<= ...)
(:wat::core::defmacro :>= ...)
```

Polymorphic-vs-typed: open question. Clojure has one `+` that does
the right thing. Wat-rs already has both `:wat::core::+` (polymorphic
across i64/f64) and `:wat::core::i64::+` (typed strict) per arc 050.
The macro layer can pick which to expose.

### Container ops — short verbs

```wat
(:wat::core::defmacro :assoc    ...)   ;; HashMap/assoc, polymorphic
(:wat::core::defmacro :dissoc   ...)
(:wat::core::defmacro :get      ...)
(:wat::core::defmacro :keys     ...)
(:wat::core::defmacro :vals     ...)
(:wat::core::defmacro :conj     ...)
(:wat::core::defmacro :empty?   ...)
(:wat::core::defmacro :length   ...)   ;; or :count Clojure-style
(:wat::core::defmacro :first    ...)
(:wat::core::defmacro :rest     ...)
(:wat::core::defmacro :nth      ...)
```

### Option/Result methods — short forms

```wat
(:wat::core::defmacro :try      ...)   ;; Result/try shortform
(:wat::core::defmacro :try?     ...)   ;; Option/try shortform
(:wat::core::defmacro :expect   ...)   ;; polymorphic Option/expect or Result/expect
(:wat::core::defmacro :unwrap   ...)   ;; Clojure-flavored unwrap (panic on None/Err)
```

### Variant constructors — short names

```wat
(:wat::core::defmacro :Some     ...)   ;; expands to wat::core::Some
(:wat::core::defmacro :None     ...)   ;; or stays as keyword
(:wat::core::defmacro :Ok       ...)
(:wat::core::defmacro :Err      ...)
```

After arc 109 § C ships and the FQDN forms are canonical, this
package brings back the bare-feeling constructors as a user choice.

## Loading

Two shapes possible:

**(a) Cargo dependency** — a separate crate that emits wat source
loaded at startup. User adds `wat-common-short-names = "..."` to
their Cargo.toml; the substrate's bundling machinery picks it up.

**(b) `(load!)` form** — pure wat package. User writes
`(:wat::load! "wat-common-short-names/all.wat")` at the top of their
program. No Rust dependency.

Likely (b) is correct because the package is pure typealiases +
macros — nothing Rust-side to bundle.

## Lifecycle dependencies

This package depends on:
- Arc 109 closing (so the FQDN canonical forms are stable).
- Arc 109 slice 1g (`:wat::core::Some` / `:None` / `:Ok` / `:Err`
  becoming canonical) before the constructor short-name macros land.
- Arc 109 § D' (`:wat::core::Option/expect` etc. becoming canonical)
  before the method-form short-name macros land.
- Arc 109 § D (`:wat::core::Vector` rename) before the `:List`
  alias.

So this package's first release waits until arc 109 substantially
closes. Stub design now; flesh out post-arc-109.

## Naming the package

Candidates:
- `wat-common-short-names` — descriptive, Clojure-leaning name
- `wat-clojure-core` — explicit about flavor
- `wat-prelude` — Haskell-flavored
- `wat-stdlib-shorts` — descriptive but verbose

Probably `wat-common` for the umbrella + `wat-common-clojure-core`
or similar for the specific flavor. Different communities can ship
their own under `wat-common-*`.

## Questions to settle when this becomes real

1. Macro vs typealias — types are alias-only, but operators have
   variadic forms (`(+ 1 2 3 4)`) that need macro expansion. Mixed
   package.
2. Polymorphic vs typed dispatch — which `+`? The polymorphic
   `:wat::core::+` or strict `:wat::core::i64::+`?
3. `=` vs `==` — Clojure uses `=`; many languages use `==`. Pick.
4. Result ordering — `Result<T,E>` (Rust) or `Either<E,T>` (Haskell)?
   Both are possible; the alias picks its own ordering.
5. How much does `wat-common` own vs delegate to per-flavor packages?
   Probably: minimum useful subset in `wat-common`; community
   variants in `wat-common-{clojure-core, haskell, ml, ...}`.

## Cross-references

- `wat-rs/docs/CONVENTIONS.md` — should grow a section "Substrate
  vendors FQDN; ergonomic surface is user-space" stating this
  principle as an architectural commitment.
- `wat-rs/docs/arc/2026/04/109-kill-std/` — the arc that puts the
  substrate in this position. After arc 109 closes, the substrate
  is ready for this layering.
- `feedback_no_new_types.md` (memory) — adjacent principle:
  substrate doesn't invent wrapper types. Same family of "substrate
  is honest; ergonomics is user-space."

## Status — append more here as the idea matures

- 2026-05-01: scratch notes captured during arc 109 slice 1e
  walker-shipped + sweep-running. Principle articulated; no package
  yet; waiting on arc 109 to close before fleshing out.

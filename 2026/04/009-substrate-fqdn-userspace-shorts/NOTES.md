# Substrate vendors FQDN; ergonomic short names are user-space — sketch

User direction 2026-05-01 during arc 109 slice 1e (mid-sweep, after
the `Queue*` → `Channel/Sender/Receiver` /gaze-finding had landed
in J-PIPELINE follow-ups):

> the position i want to be in... /all/ of our wat-provided forms
> are fqdn.. but we allow users to write common libs that implement
> the short names.. we could even do this... its not bundled in the
> wat-rs package - we, at a future time, could vend a
> "wat-common-short-names" package that makes these more
> "user-friendly" -- completely opt-in
>
> include operators like :+ :- :assoc and so on... we approach the
> clojure-core style forms as an opt-in

> i think do this in ~/work/holon/scratch/

This is a design-exploration scratch — not an arc, not a DESIGN.
Captured so the principle has a place to live until arc 109 closes
and the substrate is ready to anchor the layering.

## The principle

**The substrate (`wat-rs`) vendors only FQDN.** Every substrate-
provided symbol — types, verbs, operators, kernel primitives — has
exactly one canonical spelling under `:wat::core::*`,
`:wat::kernel::*`, `:wat::io::*`, etc. Verbose, honest, unambiguous,
no shortcuts.

**Ergonomic short names are user-space.** Different communities
want different sugar. Substrate doesn't ship one; the ecosystem
grows them.

The first community-flavored package: `wat-common-short-names`,
Clojure-core flavored.

## Why this layering

1. **Substrate stays small + unopinionated about aesthetics.** No
   substrate-level debate over `+` vs `add` vs `plus`; substrate
   has `:wat::core::i64::+` and walks away.
2. **Naming wars happen at the package level, not the substrate
   level.** Less destructive.
3. **Multiple communities coexist.** Clojure-flavored shop loads
   `wat-common-short-names`. Haskell-flavored shop writes (or
   loads) `wat-common-haskell-style`. ML-flavored shop uses
   lowercase `option`/`result`. All on the same substrate;
   substrate doesn't pick a winner.
4. **Forward compatible.** If Clojure-core flavor proves dominant,
   the substrate doesn't have to change — the package just gets
   popular.
5. **Embedded / minimal users.** Don't load any common-names
   package; write FQDN at every site. Substrate doesn't force
   ergonomics on them.

## Migration walker lifecycle (arc 109's enforcement)

- **During migration:** walkers like `BareLegacyContainerHead`
  flag bare-source spellings. Substrate-as-teacher pattern drives
  the consumer sweep.
- **After migration:** walkers retire. The typealiases
  (`:wat::core::Option<T>`, etc.) and the FQDN→bare canonicalization
  in `parse_type_inner` STAY. With walkers gone, both spellings
  type-check; users can re-introduce ANY name (including the old
  bare ones if they want) via their own typealiases or via opting
  into `wat-common-short-names`.

The migration is temporary discipline; the substrate's vocabulary
is permanent. After the dust settles: FQDN is what we *show* in
docs/examples (canonical we promote); whatever's in scope at the
call site is what the type-checker accepts.

## What's permanent vs what's temporary — the load-bearing distinction

This is the framing that keeps surfacing and keeps slipping. Stating
explicitly so it sticks:

### PERMANENT (substrate grammar, never goes away)

- **Leading `:` on type-annotation outer positions.** The colon is
  wat's "lift to global symbol table" marker. Lexer-level: `:foo`
  is a Keyword token; `foo` is a Symbol token. The substrate uses
  this distinction structurally: keywords name types / callables
  / struct paths; bare symbols name local bindings (let / lambda
  / match captures). This is load-bearing — it's how the
  substrate disambiguates "the type i64" from "a binding named
  i64 the user introduced two lines up."
- **`:wat::*` reserved for substrate.** Substrate owns this
  namespace. Users don't claim names here.
- **FQDN as canonical truth in docs / examples.** The substrate's
  documentation, error messages, and `--check-output` always show
  FQDN. The single source of canonical naming.
- **The "callable heads must be FQDN keywords" rule.** Slices
  1h+1i closed the last bare-symbol exceptions (`Some`/`:None`/
  `Ok`/`Err`). Universal post-arc-109; no carve-outs.

### TEMPORARY (retires post-arc-109)

- **Walker rejection of bare short forms.** `BareLegacyPrimitive`,
  `BareLegacyContainerHead`, `BareLegacyUnitType`, the variant-
  constructor poisons (slices 1h/1i). All retire once the
  consumer sweep is structurally complete.
- **Migration hint helpers** (`arc_109_*_migration_hint`
  functions in `collect_hints`). Same retirement.

### USER-SPACE RECLAMATION (post-arc-109)

Once the walkers retire, users can register short-form
typealiases that re-occupy the names the substrate retired
during migration:

```scheme
;; In wat-clojure-flavor / user prelude:
(:wat::core::typealias :i64       :wat::core::i64)
(:wat::core::typealias :f64       :wat::core::f64)
(:wat::core::typealias :Maybe<T>  :wat::core::Option<T>)
(:wat::core::typealias :Vec<T>    :wat::core::Vector<T>)
```

These are valid post-retirement. The substrate doesn't care —
its contract is "I own `:wat::*`; everything else is yours."

So lowercase `:i64` IS a valid user-space alias post-arc-109 —
same for `:Some`, `:Vec<T>`, etc. The bare names come back; users
get the ergonomic short forms via the alias layer.

### What flavor packages CAN and CAN'T do

| Can do | Can't do |
|---|---|
| Drop the `wat::core::` namespace prefix via aliases (`:i64` for `:wat::core::i64`) | Drop the leading `:` on outer-position type annotations |
| Rewrite bare-symbol heads in macro-controlled slots (`(+ n m)` → `(:wat::core::+ n m)` inside `defn` body) | Drop the `:` from arbitrary type annotations outside macro-controlled slots |
| Re-register names like `:Some` / `:i64` / `:Vec<T>` as aliases | Reclaim names under `:wat::*` |
| Add Clojure-style `defn` / `fn` / pattern-match macros | Make `i64` (no colon) work as a type annotation in arbitrary expression position |
| Provide ergonomic short keyword paths | Erase the keyword/symbol distinction at the lexer level |

**Bottom line:** the colon stays. The FQDN namespace shrinks
optionally via aliases. The substrate's grammar — keyword vs
symbol — is the spine; ergonomics is what hangs on the spine.

## What's in `wat-common-short-names`

Pure typealiases + macros over substrate FQDN. No Rust changes.
Likely shipped as a pure-wat `(load!)` package, not a Cargo
dependency.

### Types — short aliases

```wat
(:wat::core::typealias :Maybe<T>          :wat::core::Option<T>)
(:wat::core::typealias :Either<E,T>       :wat::core::Result<T,E>)  ;; flipped per Haskell
(:wat::core::typealias :Map<K,V>          :wat::core::HashMap<K,V>)
(:wat::core::typealias :Set<T>            :wat::core::HashSet<T>)
(:wat::core::typealias :List<T>           :wat::core::Vector<T>)    ;; post-arc-109-1f
(:wat::core::typealias :Bytes             :wat::core::Bytes)         ;; already short — pass through
```

### Operators — Clojure-flavored, keyword-headed

```wat
(:wat::core::defmacro :+ ...)   ;; → :wat::core::i64::+ or polymorphic dispatch
(:wat::core::defmacro :- ...)
(:wat::core::defmacro :* ...)
(:wat::core::defmacro :/ ...)
(:wat::core::defmacro :=  ...)
(:wat::core::defmacro :<  ...)
(:wat::core::defmacro :>  ...)
(:wat::core::defmacro :<= ...)
(:wat::core::defmacro :>= ...)
```

**Constraint:** wat's grammar forbids bare-symbol heads at call
sites. Every callable is a keyword (leading `:`). So the package
buys you `:+` over `:wat::core::i64::+` — short keyword vs long
keyword. It does NOT buy Clojure-style `(+ 1 2)` — that's a
`WatAST::Symbol` at head, which the substrate rejects.

```wat
(:+ 1 2 3)        ;; legal — keyword head, the package's offering
(+ 1 2 3)         ;; ILLEGAL — bare-symbol head, substrate rejects
```

The "Clojure-flavored" framing is about NAMING (short keyword
paths under conventional Clojure-core spellings), not about
bare-symbol invocation. Wat is keyword-first by design.

Polymorphic-vs-typed: open question. Wat-rs has both
`:wat::core::+` (polymorphic across i64/f64) and
`:wat::core::i64::+` (typed strict) per arc 050. The macro layer
picks which to expose.

### Container ops — short verbs

```wat
(:wat::core::defmacro :assoc    ...)
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
(:wat::core::defmacro :unwrap   ...)
```

### Variant constructors — short names

```wat
(:wat::core::defmacro :Some     ...)   ;; → :wat::core::Some
(:wat::core::defmacro :None     ...)   ;; or stays as keyword
(:wat::core::defmacro :Ok       ...)
(:wat::core::defmacro :Err      ...)
```

After arc 109 § C ships, this package brings back the bare-feeling
constructors as a user choice.

## Lifecycle dependencies

This package depends on:
- Arc 109 closing (so the FQDN canonical forms are stable).
- Arc 109 slice 1g (`:wat::core::Some` / `:None` / `:Ok` / `:Err`
  becoming canonical) before constructor short-name macros land.
- Arc 109 § D' (`:wat::core::Option/expect` etc.) before
  method-form short-names.
- Arc 109 § D (`Vector` rename) before `:List` alias.

So this package's first release waits until arc 109 substantially
closes. Stub design now; flesh out post-arc-109.

## Naming the package

Candidates:
- `wat-common-short-names` — descriptive, Clojure-leaning
- `wat-clojure-core` — explicit about flavor
- `wat-prelude` — Haskell-flavored
- `wat-stdlib-shorts` — descriptive but verbose

Probably `wat-common` for the umbrella + `wat-common-clojure-core`
or similar for the specific flavor. Different communities ship
their own under `wat-common-*`.

## Open questions when this becomes real

1. Macro vs typealias — types are alias-only, but operators have
   variadic forms (`(+ 1 2 3 4)`) that need macro expansion. Mixed
   package.
2. Polymorphic vs typed dispatch — which `+`?
3. `=` vs `==` — Clojure uses `=`; many languages use `==`. Pick.
4. Result ordering — `Result<T,E>` (Rust) or `Either<E,T>`
   (Haskell)? Both are possible; the alias picks its own ordering.
5. How much does `wat-common` own vs delegate to per-flavor packages?
   Probably: minimum useful subset in `wat-common`; community
   variants in `wat-common-{clojure-core, haskell, ml, ...}`.

## Where this becomes real (when arc 109 is closer to closing)

1. **`wat-rs/docs/CONVENTIONS.md`** — new section "Substrate
   vendors FQDN; ergonomic surface is user-space" stating this
   principle as an architectural commitment. References this
   scratch + the future package.
2. **Stub `wat-common` repo or directory** — the package itself.
3. **A wat-rs USER-GUIDE pointer** — once a flavor proves itself
   in practice, a one-paragraph note in the user guide
   acknowledging ecosystem packages exist (without endorsing one).

## Cross-references

- `wat-rs/docs/arc/2026/04/109-kill-std/` — the arc that puts the
  substrate in this position. The migration walkers (slices 1c,
  1d, 1e, 1f, 1g, ...) are temporary scaffolding; the FQDN
  canonical forms they protect are permanent. After arc 109
  closes, the substrate is ready for this layering.
- `wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — the discipline that
  drives migration. Step 4 ("retire the helper") applies to the
  walkers introduced for FQDN enforcement. Once retired, the
  substrate's surface is *still* FQDN-canonical (the typealiases
  + canonicalization stay), but user code can layer whatever they
  want on top.
- `wat-rs/docs/CONVENTIONS.md` — adjacent principle:
  feedback_no_new_types ("substrate doesn't invent wrapper
  types"). Same family: substrate is honest; ergonomics live in
  user-space.

## Status — append more here as the idea matures

- 2026-05-01: scratch notes captured during arc 109 slice 1e
  walker-shipped + sweep-running. Principle articulated; no
  package yet; waiting on arc 109 to substantially close before
  fleshing out.

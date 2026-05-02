# wat as a polyglot functional-language lowering target — sketch

User direction 2026-05-01, after the Haskell/ML demos (011) and
the Erlang demos walked through:

> so.. we can basically host all the major functional langs on
> wat?... that feels... /insane/....
>
> get your notes on disk - we just found a way to identify
> everything wat-rs is missing - we iterate towards as much
> compatability as possible

This is a design-exploration scratch — not an arc, not a DESIGN.
Captured because it ties scratch entries 009/010/011 together
into one meta-observation: **the substrate is positioned as a
universal lowering target for major functional languages, AND
the per-language gap analysis is a feature roadmap**.

## The meta-observation

Three substrate decisions made over arcs 057 → 109 turn out to
have one combined consequence:

1. **Substrate vendors FQDN; ergonomic surface is user-space.**
   (009)
2. **wat is a Lisp; data IS the AST.** Every language that
   compiles to s-expressions can target wat without leaving its
   native semantics behind. Arc 057's HolonAST + arc 092's
   wat-edn v4 wire format made this concrete.
3. **EDN is the wire.** Anything that emits EDN-shaped trees
   can drive wat-vm. (arc 092 + arc 103's `:wat::eval-edn!`)

Each one is small. Together they make wat **a polyglot lowering
target**. Not a niche substrate — a universal runtime that any
major functional language can compile to, with the language
community keeping their preferred surface naming via per-flavor
packages (009/010/011).

## What fits cleanly today

| Family | Members | Fit | Arc 109 dependency | Substrate gap |
|---|---|---|---|---|
| **Lisp** | Scheme, Racket, Clojure, ClojureScript, Janet, Common Lisp | ★★★ trivial | works post-1c/1d/1e/1f | none — wat IS Scheme-flavored |
| **ML** | SML, OCaml, F#, ReasonML | ★★★ clean | post-arc-109 | module system / functors |
| **Erlang/BEAM** | Erlang, Elixir, Gleam | ★★★ clean | post-arc-109; concurrency primitives already shipped | `link` / `monitor` / hot reload |
| **Haskell-eager-subset** | PureScript, Roc, Idris-without-deps | ★★ mostly | post-arc-109 | laziness, higher-kinded types, full type classes |
| **Functional Scala/Kotlin/F#** | Their FP subsets | ★★ mostly | post-arc-109 | class hierarchies, JVM interop |

## What has real gaps — the feature roadmap

This is the load-bearing realization. **Each gap a hosted-language
flavor can't bridge is a substrate feature wat-rs could plausibly
add.** The gap analysis IS the missing-features list. The substrate
gets steered by community demand: gaps that hurt many flavors get
filled first; gaps that hurt only edge cases don't.

### Gaps from Erlang flavor (concurrency / reliability)

| Gap | What Erlang gives | Substrate work to fill |
|---|---|---|
| **`link(Pid)` / monitor relationships** | Process A linked to B dies if B dies | Arc TBD: substrate adds `Thread/link` / `Process/link`; cascading panic via the existing chain |
| **Hot code reload** | Replace a running process's code | Arc TBD: freeze-time symbol table grows a "swap" verb; live programs see new bindings on next dispatch |
| **`process_info` introspection** | List mailbox depth, status | Arc TBD: substrate exposes per-Thread/Process introspection verbs |
| **Distributed Erlang wire** | Pid-aware network protocol | Arc TBD: spawn-remote (or similar); EDN over TCP with peer authentication |

### Gaps from Haskell flavor (type system + evaluation)

| Gap | What Haskell gives | Substrate work to fill |
|---|---|---|
| **Lazy evaluation** | Thunk-based; values computed on demand | Major arc: substrate lazy values + force semantics. Hard. |
| **Higher-kinded types** | `* -> *` abstraction over type constructors | Substantial: type system extension to first-class type constructors |
| **Type classes (full)** | `class Show a where show :: a -> String` + `instance Show Foo where ...` | Partial fit post-arc-109 § J slice 10d (typeclass dispatch). Full Haskell-style instances need more |
| **Monadic do-notation** | `do { x <- m; ... }` syntactic sugar | Macro-level (no substrate change); flavor package can implement |
| **Linear / affine types** | Resources tracked at type level (Linear Haskell) | wat's Channel single-owner discipline is loosely linear; formal linear types would need substrate work |

### Gaps from ML family (modules)

| Gap | What ML gives | Substrate work to fill |
|---|---|---|
| **Functors** (parameterized modules) | `module M = struct ... end`; functors are functions over modules | Substantial: substrate adds module-as-value; `:wat::core::module` form; namespace parameterization |
| **Module signatures** | Type-level interface declarations | Smaller: typeclass-shaped via arc 109 § J slice 10d |

### Gaps from broader functional languages

| Gap | What it gives | Hostable? |
|---|---|---|
| **Dependent types** (Coq, Agda, Lean, Idris) | Types depend on values | No — wat's type system is first-order. Major theoretical lift |
| **Continuations / first-class callcc** (Scheme R6RS) | `call/cc` saves the continuation | Substrate work; not hard but non-trivial |
| **Lazy streams as primitive** (Haskell, Idris) | Infinite data structures | Same as laziness above |
| **Effect systems** (Koka, Eff, OCaml 5) | Tracked effects in the type | Hard — needs substrate type-system extension + algebraic effect runtime |

### What the substrate does NOT give up — the grammar spine

Before listing gaps, name what's permanent. Flavor packages
operate WITHIN this spine:

- **Leading `:` on outer-position type annotations** — keyword vs
  symbol is wat's lexer-level distinction. Permanent. No flavor
  removes it.
- **`:wat::*` reserved namespace** — substrate owns it; users
  don't claim names there. **Flavor packages live under their
  OWN top-level namespaces** (`:clojure::*`, `:haskell::*`,
  `:erlang::*`, `:ml::*`, etc.) — NOT under `:wat::*`. The
  substrate's reserved-namespace contract is one-way.
- **FQDN canonical in docs / errors / `--check-output`** —
  substrate always shows the canonical form even when consumer
  source uses aliases.

These three rules are the SPINE. Every flavor package hangs off
this spine. Communities pick how to shape ergonomics; substrate
defines what's allowed at the lexer / type-system / namespace
layers.

See scratch 009 § "What's permanent vs what's temporary" for the
canonical framing of permanent vs temporary vs user-space-
reclamation distinctions.

### Cross-flavor calls — no FFI boundary

The flavor source markup is a **compile-time thing**. Once
macro expansion + type-check finishes, the program is canonical
wat AST. The substrate doesn't track "this came from
Haskell-flavor source" — it's just substrate-level forms now.

**Cross-flavor function calls are ordinary function calls.** No
FFI, no marshalling, no boundary. A Haskell-flavored pure helper
can be called from a Clojure-flavored business-logic function
running inside an Erlang-flavored actor — all three coexist and
call each other at the substrate-call layer.

Worked example — actor loop in Erlang flavor, worker in Clojure
flavor, helper in Haskell flavor:

```scheme
;; Haskell-flavored — pure pattern-match on Maybe:
(haskell/defn (lookupOr (key :String) (m :haskell::Map<String,Int>) (def :Int) -> :Int)
  (case (haskell::Map/lookup key m)
    ((Just v)  v)
    (Nothing   def)))

;; Clojure-flavored — worker body:
(clojure/defn (process-tick [(state :State) (tick :Tick)] -> :State)
  (let [(price :Int) (lookupOr "btc-usd" (:prices state) 0)
        (qty :Int)   (* (:qty tick) price)]
    (assoc state :total (+ (:total state) qty))))

;; Erlang-flavored — actor loop:
(erlang/defn (treasury-actor)
  (receive
    ((tick T)
      (let [(state2 :State) (process-tick @state T)]
        (set! state state2)
        (treasury-actor)))
    (stop  :ok)))
```

After all three flavors' macros expand, the program is canonical
wat:

```scheme
(:wat::core::define (:treasury::lookupOr ...) ...)
(:wat::core::define (:treasury::process-tick ...)
  (... (:treasury::lookupOr ...) ...))
(:wat::core::define (:treasury::treasury-actor ...)
  (... (:treasury::process-tick ...) ...))
```

The substrate sees ONE program. `process-tick` calling
`lookupOr` calling `Map/lookup` is three function calls — same as
any other function call. **No FFI, no marshalling.**

**This is the load-bearing polyglot win.** Other polyglot
runtimes have FFI boundaries between languages:

- JVM has JNI for native code
- .NET has P/Invoke
- Node has N-API
- WebAssembly modules have linear-memory ABIs

**wat doesn't need any of that** because flavor markup vanishes
at compile time. Every artifact is wat. Cross-flavor calls are
free.

**Practical implication for teams:** mix flavors per-module based
on what each module is best expressed in:

- Reliability/concurrency glue → Erlang flavor (actor model is
  native to wat-rs's substrate)
- Pure data transforms → Haskell flavor (ADTs + pattern match
  read clean)
- Domain logic → Clojure flavor (hashmap/set primitives +
  threading macros)
- All three call each other ordinarily.

The polyglot vision isn't "wat hosts other languages"; it's
"wat hosts the FUNCTIONAL IDEAS those languages each express
best, with the team choosing which surface fits which module."

### Cross-flavor naming — namespace collisions

Calls work freely; naming has constraints. Each flavor's
typealiases / macros need to live under their own top-level
keyword namespace to avoid colliding:

```scheme
;; Each flavor under its own namespace (NOT :wat::*):
(:wat::core::typealias :clojure::Map<K,V>  :wat::core::HashMap<K,V>)
(:wat::core::typealias :haskell::Map<K,V>  :wat::core::HashMap<K,V>)
(:wat::core::typealias :ml::map<K,V>       :wat::core::HashMap<K,V>)
```

Three different `Map` aliases coexist because they're under
different prefixes. User code that wants the bare-feeling
`:Map<K,V>` shortcut imports their primary flavor's prefix into
the local namespace via another alias:

```scheme
;; "I'm a Clojure shop; bring clojure::Map up to bare :Map":
(:wat::core::typealias :Map<K,V> :clojure::Map<K,V>)
```

When mixing flavors in one program, code uses the flavor-prefixed
forms (`:clojure::Map`, `:haskell::Map`) explicitly. No collision
because each flavor stays in its own namespace.

Similar to Clojure's `(require '[clojure.string :as str])`
pattern: prefix-named imports for explicit reach; aliasing for
common-case ergonomics.

### Gaps from syntax — reader-level extensions

| Feature | What it gives | Substrate work |
|---|---|---|
| **Bracket-form reader macros** (`[...]`, `{...}`, `#{...}`) | Vector / map / set literals; Clojure-style `defn` param lists; Haskell `[a]` list-type / `Map k v`; ML list-cons `(x::xs)` via reader sugar | Parser-level: extend the wat reader to recognize bracket forms and emit new AST variants (`WatAST::Bracket` / `WatAST::Brace` / `WatAST::HashBrace`). Substrate attaches **zero** semantic meaning — just emits the data. User-space macros (per-flavor packages) recognize and rewrite to canonical wat. **Lights up multiple flavors** in one pass: Clojure `(defn make-adder [n] body)` is a macro over the bracket-param form; Haskell `[a]` becomes possible; ML's `[]` / `(x::xs)` patterns surface. **Pattern**: substrate ships PURE syntax; semantic meaning is user-space. Same FQDN-substrate-vs-flavor-package split as scratch 009. **Cost: one arc, one parser pass; benefits multiple downstream flavors.** |

User direction (2026-05-01) prompting this entry:

> on the clojure-y forms of wat... we could deliver user-space
> macros who acccept "[...]" and "{...}" and "#{...}" as
> expressions that are legal right?...
>
> we could actually impl a macro who is basicaly clojure's real
> form?...
>
> (defn make-adder [n] -> :fn(:int64)->:int64 (fn [m] (+ n m))
> => some wat form?....

Worked example — `defn` macro expansion (post-arc):

```scheme
;; user writes (post-arc with reader extension + macro lib loaded):
(defn make-adder [(n :wat::core::i64)] -> :fn(wat::core::i64)->wat::core::i64
  (fn [(m :wat::core::i64)] -> :wat::core::i64 (+ n m)))

;; macro expands to canonical wat:
(:wat::core::define
  (:my::pkg::make-adder (n :wat::core::i64)
                        -> :fn(wat::core::i64)->wat::core::i64)
  (:wat::core::lambda ((m :wat::core::i64) -> :wat::core::i64)
    (:wat::core::+ n m)))
```

The `[...]` is the parser change (substrate); the `defn` macro is
user-space (flavor package). One reader extension ships → multiple
flavor surfaces become possible.

### Hard no — different paradigms

- **APL/J/K** (array languages) — different evaluation paradigm.
- **Prolog/miniKanren** (logic) — needs unification + backtracking runtime.
- **Smalltalk-style image** (Pharo, Squeak) — wat is freeze-time-loaded; no live image.

## The compatibility roadmap

Reading the gap tables top-to-bottom: **the substrate's
priority queue for new arcs is now community-driven**.

- A clojure shop doesn't need link/monitor — Clojure flavor is
  already complete.
- An Erlang shop needs link/monitor before adoption — that's an
  arc.
- A Haskell shop needs typeclass dispatch (10d, already queued)
  + maybe lazy eval (major) before adoption — those are arcs.
- An ML shop needs functors before module-heavy code maps — arc.

Each arc has a community asking for it. The substrate adds
features in the order the lowering targets need them. The
roadmap writes itself.

## Why this matters

1. **Cross-language interop becomes possible.** Haskell-flavored
   wat calls Erlang-flavored wat through wat-vm (they're all wat
   at the lowering level). This is unprecedented for typed
   functional languages.

2. **Communities can adopt without abandoning identity.** A
   Clojure shop adopts wat for fault-tolerance + types without
   becoming a Haskell shop. An Erlang shop adopts wat for typed
   compile-time safety without becoming a Scala shop. They each
   look at *their* surface; the substrate translates.

3. **The substrate becomes a research vehicle.** Want to add
   linear types? It lights up Linear Haskell's flavor. Want to
   add effect systems? Koka-flavor lights up. Each substrate
   addition creates a new lowering surface; each new flavor
   highlights a new gap.

4. **`wat` reads as the joke it deserves.** "wat is this" —
   wat is whatever-you-want. The naming was inevitable in
   hindsight.

## What this is NOT

- **Not "everything compiles to wat."** Imperative languages
  (Java, C++, Python) don't lower cleanly without substrate
  bytecode-translation work that's outside this trajectory.
- **Not a promise of feature parity.** Each flavor is honest
  about what doesn't translate (the per-flavor scratch notes
  document the gaps).
- **Not a replacement for the languages themselves.** A Haskell
  shop on wat is choosing wat *as a lowering target*; they keep
  GHC for libraries that don't translate, use wat for the parts
  that do. Hybrid is the realistic adoption shape.

## The vision statement (committed 2026-05-01)

User direction:

> i think we're going to make the trading lab a clojure-style
> app...
>
> this is a compelling path.. the substrate earns it name
>
> i think the vision statement... we build the trading lab as a
> strongly-typed-clojure -- this forces us to find gaps in the
> substrate - this is a forcing function for completeness...

**The trading lab is the forcing function for substrate
completeness.**

The lab is not "the first flavor consumer" in a passive sense.
It is the **active stress test**. Building a real production-style
trading system as **strongly-typed Clojure** on top of wat-rs
will surface every gap the substrate has. Each gap becomes a
candidate substrate arc; the lab's adoption journey IS the
substrate's roadmap.

**Strongly-typed Clojure** is the framing — not just "Clojure
ergonomics" but "Clojure ergonomics on a static-type substrate."
That's the unique offering. Clojure's expressive power; wat's
type discipline; running together as one program. The lab is
where this combination proves itself.

**Forcing function dynamic:**

```
lab module migrates → finds gap → substrate arc fills gap →
lab module ships → next module migrates → finds next gap → ...
```

Each iteration:
1. Migrate one lab module to strongly-typed-Clojure surface.
2. The migration trips into a substrate gap (missing reader
   form, missing typeclass, missing macro support, missing
   diagnostic).
3. The gap becomes an arc.
4. Substrate ships the arc; lab module migration continues.
5. Repeat until lab is fully migrated.

**By the time the lab is fully migrated, the substrate has
absorbed every gap a working strongly-typed Clojure system would
encounter.** That's substrate completeness, measured by usage,
not by spec.

This is why the lab is committed — it's not just a demo; it's
the requirements engine.

## Lifecycle — when does this become real

Same as the per-flavor entries (009/010/011): gates on arc 109
substantially closing.

- **Phase 0 — wait** (now). Substrate vocabulary still moving;
  flavor packages premature.
- **Phase 1 — Clojure flavor + lab migration (COMMITTED).**
  `wat-common-clojure-flavor` package scaffolds. Trading lab
  begins migration to strongly-typed-Clojure surface. Each
  migrated module surfaces substrate gaps; gaps become arcs.
  See `holon-lab-trading/docs/drafts/wat-clojure-flavor.md` for
  the lab-side phased plan. **This phase is the forcing function.**
- **Phase 2 — second flavor**. Likely Erlang (the highest-value
  community-fit because wat's concurrency model already IS
  BEAM's actor model). Adds the first set of substrate gap
  arcs (link/monitor/hot-reload). The Erlang shop adapting will
  surface its own gap list — a second forcing function for a
  different community's needs.
- **Phase 3 — third flavor + cross-flavor experiment**. Haskell
  or ML lands; Erlang-flavored process calls Haskell-flavored
  function via wat-vm; we have proof of cross-language interop.
- **Phase 4+ — community grows**. New flavors land as new
  communities show up. Each one identifies new substrate gaps.
  The substrate roadmap is community-driven, not architecture-
  speculative.

## Cross-references

### Sibling scratch entries (this is the meta-doc)

- `009-substrate-fqdn-userspace-shorts/NOTES.md` — the principle
  (substrate FQDN; ergonomic surface user-space). 012 is the
  meta-extension: not just *naming* user-space, but *whole
  language surfaces* user-space.
- `010-clojure-emits-wat/NOTES.md` — the orthogonal "different
  language entirely emits wat" path; complements per-flavor
  packages.
- `011-wat-common-flavor-comparison/NOTES.md` — the per-language
  demo comparison (Haskell vs ML vs Clojure surfaces).

### Substrate dependencies

- `wat-rs/docs/arc/2026/04/057-wat-holon-namespace/` — HolonAST
  schema; the substrate's foundation for AST as first-class data.
- `wat-rs/docs/arc/2026/04/092-wat-edn-v4/` (or wherever) — EDN
  v4 wire format; the universal serialization that makes
  cross-language emit possible.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/` — fork/exec/eval
  paths; the runtime ingest surface flavored compilers target.
- `wat-rs/docs/arc/2026/04/109-kill-std/` — FQDN substrate
  vocabulary; gates every flavor package.
- `wat-rs/docs/arc/2026/04/114-spawn-as-thread/` — Program<I,O>
  contract; the Erlang-actor-model alignment.
- `wat-rs/docs/arc/2026/04/113-cascading-runtime-errors/` —
  chain backtrace; the supervision-tree story.

## Status — append more here as the idea matures

- 2026-05-01: meta-observation captured during arc 109 slice 1f
  sweep, immediately after the Erlang demos walked through. The
  realization that "we found a way to identify everything wat-rs
  is missing" reframes the substrate's roadmap as
  community-driven: each flavor's gap list is a queue of
  candidate arcs. No commitment to any specific arc here; just
  the framing.

# Clojure-emits-wat — Clojure as a wat compile target — sketch

User direction 2026-05-01, after the lab `wat-clojure-flavor`
draft landed:

> i love it - great start
>
> this also begins to open another door... clojure can be used to
> generate wat... clojure devs can be given some lib we provide
> that allows a clojure dsl to export wat?...
>
> yes... another scratch... let's not forget this...

This is a design-exploration scratch — not an arc, not a DESIGN.
Captured so the framing has a place to live until the substrate
and ecosystem are ready.

## What this is — distinct from 009

| Doc | What it is | Author writes | Output |
|---|---|---|---|
| 009 / `wat-clojure-flavor` | Clojure-familiar NAMING for wat | `.wat` files (with short keyword names like `:fn`, `:defn`) | wat AST (native) |
| 010 / Clojure-emits-wat (THIS) | Clojure as a compile target for wat | `.clj` files on the JVM | `.wat` files or live wat-vm AST |

**009 is "wat that reads like Clojure."** The substrate runs the
output directly. Familiar names, native syntax tree.

**010 is "Clojure that produces wat."** A clojurist writes real
Clojure code (with their existing JVM toolchain, REPL, IDE, lein/
deps.edn, libraries). A Clojure macro library transforms it into
wat AST. The output gets piped to wat-vm or written as `.wat`
files.

These are complementary, not competitive. Some users want both:
flavor their wat to look Clojure-y; clojurists at JVM shops
generate wat from real Clojure source.

## Why this opens a real door

1. **Existing toolchain.** Clojure has Leiningen, deps.edn, CIDER,
   Calva, nREPL, paredit, structural editing, time-travel
   debugging. Clojurists carry decades of muscle memory. Writing
   Clojure that *targets* wat means they don't lose any of that.
2. **EDN bridge already exists.** Arc 092 minted wat-edn v4.
   Clojure code IS EDN. Reader macros / `read-string` produce
   data structures wat understands. The serialization gap is
   nearly zero.
3. **Macro infrastructure carries.** Clojure has `defmacro`,
   syntax-quote (`` ` ``), unquote (`~`, `~@`). These are
   exactly what you need to write a wat-emitting library. A
   `wat/defn` macro produces wat-AST data; user code reads as
   Clojure but expands to wat forms.
4. **Hindley-Milner over the Clojure surface.** Clojure is
   dynamically typed at runtime, but a Clojure-to-wat compiler
   can run inference (or accept explicit `^:type` metadata) and
   emit the FQDN type annotations wat requires. The compiler is
   the gate; the runtime is wat.
5. **Adoption story.** "Clojurists adopt wat without learning
   wat." A Clojure shop wanting fault-tolerant typed programs
   without giving up their stack writes Clojure-with-types, gets
   wat as the deployment artifact.

## Sketched API

A Clojure library — call it `wat-emit` for now.

```clojure
(ns my.app
  (:require [wat-emit.core :as wat]))

(wat/defn pnl
  [^:f64 entry ^:f64 exit ^:f64 qty]
  :-> :f64
  (* (- exit entry) qty))

(wat/defn portfolio-value
  [^{:tag :wat::core::Vector
     :element :wat::core::f64} prices]
  :-> :f64
  (reduce + 0.0 prices))
```

What the macros emit (after expansion):

```scheme
(:wat::core::define
  (:my::app::pnl
    (entry :wat::core::f64)
    (exit  :wat::core::f64)
    (qty   :wat::core::f64)
    -> :wat::core::f64)
  (:wat::core::*
    (:wat::core::- exit entry)
    qty))

(:wat::core::define
  (:my::app::portfolio-value
    (prices :wat::core::Vector<wat::core::f64>)
    -> :wat::core::f64)
  (:wat::core::foldl prices 0.0
    (:wat::core::lambda
      ((acc :wat::core::f64) (p :wat::core::f64) -> :wat::core::f64)
      (:wat::core::+ acc p))))
```

The wat AST is the deployable artifact. wat-vm reads it via the
EDN wire (arc 092 + arc 103a's `(:wat::eval-edn!)` etc).

## Architecture options

### (a) Clojure-side compiler emits .wat files

```
foo.clj  →  [wat-emit compiler]  →  foo.wat  →  wat-vm runs it
```

User writes `.clj`, runs `lein wat-compile foo.clj`, gets `foo.wat`.
That `.wat` file deploys like any wat program.

Best for: build-and-deploy workflows. CI compiles Clojure to wat
before shipping.

### (b) Clojure-side compiler emits in-memory wat AST + runs subprocess

```
foo.clj  →  [wat-emit]  →  EDN bytes  →  pipe → `wat ingest -`
```

User runs `lein run foo.clj` (or via REPL); wat-emit produces
EDN, pipes to a wat subprocess. Live runs from Clojure source.

Best for: REPL development. Clojurist iterates at the REPL;
each form re-emits and re-runs on wat.

### (c) Hybrid: emit during compile, run during dev

Build artifact = `.wat` file (option a). Dev loop = pipe to
subprocess (option b). Both available; user picks per workflow.

Probably (c) — same library, two output modes.

## What gates this work

Substantial dependency stack:

1. **Arc 109 closing** — FQDN canonical forms must be stable. The
   compiler emits FQDN annotations; if the substrate's vocabulary
   keeps shifting, the compiler chases.
2. **wat-edn v4 stable wire** — already minted (arc 092). Compiler
   emits EDN; wat-vm reads EDN. The bridge format must not move.
3. **Stable wat-vm subprocess interface** — already in place via
   arc 103a's `:wat::eval-edn!` / arc 104's `fork-program-ast`.
4. **A Clojure library** — `wat-emit` doesn't exist. New deliverable.
   Macros: `wat/defn`, `wat/let`, `wat/match`, `wat/struct`, etc.
   Type system: walk Clojure metadata, emit FQDN annotations.
5. **The lab `wat-clojure-flavor` proof** (009) shipping first —
   gives us evidence that "Clojure-named" wat is readable and
   useful. Compiles confidence to invest in 010.
6. **Optionally: the substrate's bracket-form reader-macro arc**
   (queued as a substrate gap in scratch 012). Without it, the
   emitter must lower Clojure's `[args]` / `{...}` / `#{...}` to
   wat's paren-only syntax (`(args)`, `(:wat::core::HashMap ...)`,
   etc.) — the round-tripped `.wat` file uses paren wrapping. With
   the bracket-reader arc, the emitter can produce wat that
   ROUND-TRIPS more naturally with the input Clojure shape and is
   readable when a Clojurist opens the artifact. Either path
   works; the bracket arc is an ergonomic-output improvement, not
   a hard prerequisite for Clojure-emits-wat. See scratch
   `012-wat-as-polyglot-lowering-target/NOTES.md` § "Gaps from
   syntax — reader-level extensions" for the canonical entry.

## Open questions

1. **Type inference vs annotation.** Clojure is dynamically typed.
   Options:
   - **All-explicit**: every binding/return annotated with
     `^:type`. Verbose but unambiguous. Closest to wat's discipline.
   - **All-inferred**: HM over the Clojure surface. Best ergonomics,
     hardest to implement; needs the Clojure compiler to walk types
     across `let`/`fn`/`recur` etc.
   - **Hybrid**: infer where possible (literal `1` → `:i64`, `1.0`
     → `:f64`, `"x"` → `:String`); require annotation at function
     boundaries (signatures stay explicit).
   Probably hybrid.
2. **Which Clojure subset compiles?** Not every Clojure form maps
   to wat:
   - `defn` → `:wat::core::define` ✓
   - `fn` → `:wat::core::lambda` ✓
   - `let` → `:wat::core::let*` ✓ (needs sequential, which wat has)
   - `if` → `:wat::core::if` ✓
   - `case` / `cond` → `:wat::core::match` / `:wat::core::cond` ✓
   - `loop`/`recur` → tail-recursive define ✓ (modulo arc 003 TCO)
   - `for` (list comprehension) → maps + filters ✓
   - `def` (top-level value binding) — wat has no top-level value
     bindings; either reject or rewrite as zero-arg defn
   - `defmulti`/`defmethod` — needs wat's typeclass dispatch (arc
     109 § J slice 10d)
   - `atom`/`ref`/`agent` — wat has no shared mutable state. REJECT.
     Channel-based instead.
   - `core.async/chan` → `:wat::kernel::Channel<T>` (post-rename)
   - Every JVM-only library — reject.
   The compiler defines the subset; users learn what's allowed.
3. **Runtime model.** Clojure code that compiles to wat doesn't
   run on the JVM at deploy time. Implications:
   - Clojure REPL during dev runs the *Clojure* (JVM) version,
     emits wat for testing — but the compiled wat runs differently
     (typed, no GIL, no JVM).
   - A "REPL test" that passes might fail in compiled wat (type
     errors, missing primitives).
   - Or: the compiler runs an in-Clojure wat-vm simulator.
     Heavyweight but accurate.
4. **Errors crossing back.** wat compile errors (type mismatch,
   etc.) need to surface back to Clojure source positions. The
   compiler tracks Clojure source locations through the macro
   expansion → wat AST pipeline → wat-vm error reports.
5. **Naming hygiene.** Clojure namespaces map to wat keyword
   paths: `my.app/pnl` → `:my::app::pnl`. Slash and dot
   conventions need consistent translation.
6. **What about `wat-clojure-flavor` macros?** If the lab ships
   `:defn` etc. as wat-side macros (009 path), Clojure-emitting
   compilers could emit either:
   - The FQDN forms (`:wat::core::define`) — most honest.
   - The `wat-clojure-flavor` short forms (`:defn`) — readable
     output if user opens the .wat file.
   Probably the FQDN form (deployment artifact stays canonical;
   readability of the .wat output is a secondary concern).

## What this is NOT

- Not a "Clojure on wat-vm" port. wat-vm doesn't run JVM
  bytecode. The compiler emits *wat code* that runs on wat-vm.
  Clojure source is just the input language.
- Not a replacement for `wat-clojure-flavor`. They serve
  different audiences:
  - `wat-clojure-flavor`: "I write wat; I want short familiar
    names." Stays as wat author.
  - Clojure-emits-wat: "I write Clojure; I want wat as my
    deployment target." Doesn't write wat by hand.
- Not a fork of Clojure the language. Stays compatible with
  Clojure semantics where possible; rejects what doesn't lower
  cleanly.

## When this becomes real

Phases (rough):

- **Phase 0 — wait** (now). Arc 109 mid-flight; lab proof of 009
  not yet shipped. Premature.
- **Phase 1 — proof of (009).** The lab adopts
  `wat-clojure-flavor`. Demonstrates that Clojure-flavored wat is
  readable + maintainable. Builds confidence.
- **Phase 2 — Clojure library scaffolding.** A separate repo
  (`wat-emit` or similar). Macros for `defn`/`fn`/`let`/`match`.
  Walk metadata, emit FQDN annotations. Hello-world: a Clojure
  function compiles to a `.wat` file that runs on wat-vm.
- **Phase 3 — REPL integration.** Pipe to wat subprocess; results
  read back into Clojure REPL. Iterative dev loop.
- **Phase 4 — broader subset.** `loop`/`recur`, `cond`, `case`,
  `for`, error handling.
- **Phase 5 — community.** Docs, examples, "writing wat in
  Clojure" tutorial. Adoption story for clojurists.

## Cross-references

- `~/work/holon/scratch/2026/04/009-substrate-fqdn-userspace-shorts/NOTES.md`
  — the broader principle (substrate FQDN; ergonomic surface
  user-space). 010 is the "different language entirely" case of
  the same principle: substrate is canonical; consumers can be
  ANY language that emits wat.
- `holon-lab-trading/docs/drafts/wat-clojure-flavor.md` — the
  lab proof of 009. Validates that Clojure-flavored *wat* works
  before we invest in Clojure-emits-wat.
- `wat-rs/docs/arc/2026/04/092-wat-edn-v4/` (or wherever) — the
  EDN bridge format that makes 010 mechanically simple.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/` — `:wat::eval-edn!`
  as the runtime ingest path.
- `wat-rs/docs/arc/2026/04/104-wat-cli-fork-isolation/` —
  `wat ingest` (or whatever shape lands) as the build-time
  pipeline target.

## Status — append more here as the idea matures

- 2026-05-01: scratch notes captured during arc 109 slice 1f
  walker-shipped + sweep-running, immediately after the lab
  `wat-clojure-flavor` draft landed. Distinguished from 009.
  Phased lifecycle named. No code, no library, no commitment;
  just a placeholder so the door doesn't get lost.

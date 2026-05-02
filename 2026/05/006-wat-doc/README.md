# wat-doc — the canonical wat documentation generator

User direction (2026-05-03):

> "first.. fmt -> lint -> cov [...] getting those basically gets
> us into parity with every major programming lang?.. [...]
> maybe a documentation thing is what's missing?"

> "we could just add doc strings as first class citizens in our
> forms.... that's probably the honest thing to do?...."

> "i totally agree with this... we'll update our core forms as
> we approach this... we'll have a linter flag when you didn't
> declare a doc string... this is an excellent arc to record"

This arc captures the design pre-implementation. wat-doc closes
the foundation-tier toolkit (fmt + lint + cov + DOC) — putting
wat at functional parity with every major language toolkit.

The load-bearing decision: **docstrings are first-class citizens
in the substrate forms.** Not comments-above (Rust's path) but
strings-in-form (Clojure's path). The honest move because wat
is a Lisp; morphology-over-position; the form carries its own
documentation as substrate-guaranteed structure, not tooling
convention.

---

## What wat-doc is

A documentation generator for wat code. Walks the AST; for each
public form (`:define` / `:lambda` / `:defmacro` / `:typealias` /
`:struct` / `:enum`), extracts the docstring + signature + cross-
references; emits HTML / Markdown / EDN / JSON.

Modeled on Clojure's codox (the cleanest Lisp-family doc tool)
with the wat-native shape: docstrings live INSIDE forms, not
as attached comments.

## Architecture in one paragraph

The substrate gets a small change: `define` / `lambda` / `defmacro`
accept an optional docstring (string literal) as the second
argument, between signature and body. Type checker recognizes
strings in that position as docstring metadata; no runtime
semantics impact. wat-doc walks the AST, extracts (form,
signature, docstring) triples for every public form, resolves
FQDN cross-references, emits documentation in four formats.
Same self-contained arc-013 crate pattern as wat-fmt / wat-lint /
wat-cov; CLI subcommand `wat doc`.

## The substrate change

**Three-arg form for `define` / `lambda` / `defmacro`** —
backwards-compatible (existing 2-arg forms still valid):

```scheme
;; Old shape (still valid):
(:wat::core::define (sig) body)

;; New shape (optional docstring between signature and body):
(:wat::core::define (sig) "Doc string." body)
```

Concrete examples:

```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Compute the thing. First sentence is the summary.

   Long-form explanation continues across lines per Rule 31."
  (:wat::core::* x x))
```

```scheme
(:wat::core::lambda
  ((x :T)
   -> :U)
  "Doc string for this anonymous function."
  body)
```

```scheme
(:wat::core::defmacro
  (:my-macro
    (x :AST<T>)
    -> :AST<U>)
  "Doc string. Explains what the macro expands to."
  `(template ,x))
```

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-doc/`.

```
wat-rs/crates/wat-doc/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-fmt (parser + comment access),
                       #   wat-edn (--json output)
  src/                 # Rust shim (extractor + formatters)
  wat/doc/             # wat-coded extraction rules + filters
  wat-tests/doc/       # wat-level tests
  tests/               # Rust harness + golden HTML/Markdown/EDN
```

Same arc-013 pattern as the other foundation crates. `wat-cli`
depends on `wat-doc`; users get docs generation for free.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture (substrate change + extractor + formatters), the four questions applied throughout, STYLE-RULES.md amendment cascade, wat-lint integration, output formats, cross-references, backwards-compat. |
| `SLICE-PLAN.md` | 4 slices: (1) substrate change + parser handling; (2) wat-doc crate with EDN + Markdown; (3) HTML output (codox-style); (4) cross-references + lint integration + polish. |

## The fourth corner of the foundation tier

```
wat-fmt    (003)    — format the code
wat-lint   (004)    — lint the code
wat-cov    (005)    — measure how much of the code is exercised
wat-doc    (006)    — document the code
```

Together: format, lint, measure, document. The discipline triad
becomes a quartet. Functional parity with Rust (rustfmt + clippy
+ tarpaulin + rustdoc), Ruby (rubocop + simplecov + yard),
Python (black + ruff + coverage.py + sphinx), Clojure (cljfmt +
clj-kondo + cloverage + codox).

## Conventions inherited

From the wat-fmt / wat-lint / wat-cov arcs:

- **The four questions as design compass** (Obvious / Simple /
  Honest / Good UX) — applied to every choice
- **Atomicity-and-signal principle** — docstrings are atomic
  (per Rule 27 + Rule 31); preserved verbatim
- **Arc-013 self-contained crate pattern** — bundled
  `wat_sources()` + `register()`
- **Developer-first output** — same EDN payload reads for
  humans and parses for machines; JSON via wat-edn; HTML for
  humans browsing
- **LLM out until the user delegates** — wat-doc produces
  documentation; spells (or future LLM tooling) can interpret
  / summarize / improve docs only when the user invokes them

## Cross-references

- **wat-fmt** at `scratch/2026/05/003-wat-fmt/` — provides the
  comment-preserving parser; STYLE-RULES.md needs a small
  amendment to handle the new 3-arg form
- **wat-lint** at `scratch/2026/05/004-wat-lint/` — adds the
  `documentation/missing-public-docstring` rule that flags
  forms without docstrings
- **wat-cov** at `scratch/2026/05/005-wat-cov/` — sibling
  foundation crate; same architectural pattern
- **arc 013** at `wat-rs/crates/wat-lru/src/lib.rs` — the
  external-crate-shipping contract wat-doc inherits
- **Clojure codox** — the model wat-doc draws from; the cleanest
  Lisp-family doc generator

## Status

- **Captured:** 2026-05-03
- **Architecture:** locked at the load-bearing level (docstrings
  as first-class; substrate change scope; arc-013 crate; four
  output formats; wat-lint integration)
- **Details to refine via chat (option C):** specific output
  format details (HTML theme, Markdown flavor, EDN schema),
  cross-reference resolution mechanics, code-example syntax in
  docstrings, multi-crate doc tree composition
- **Slice plan:** 4 slices sized; not opened
- **Bar to open as a real wat-rs arc:** the substrate docstring
  change requires an actual wat-rs arc (not just a wat-doc
  consumer arc); cited use is wat-doc itself; opens when the
  user signals start

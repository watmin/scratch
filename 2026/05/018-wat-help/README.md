# wat-help — runtime reflection: `(:wat::help :sym)` → formatted form-as-EDN

User direction (2026-05-03):

> *"we are working through reflection primitives now... i think we
> need a wat-help who provides the form (:wat::help :some-symbol)
> -> entire form-as-edn*
>
> *this ret val.. help should depend on wat-fmt and repl should
> depend on help*
>
> *when we run (:wat::help :some-symbol) we get a pretty formatted
> edn str based on wat-fmt's rules?.. the user is presented with a
> digestable form that's hopefully useful?..."*

---

## What wat-help is

The **runtime reflection** primitive for wat. Given a fully-qualified
symbol, returns a pretty-formatted EDN string showing the entire
form (definition + signature + docstring + source location + whatever
the substrate's reflection primitives expose).

```scheme
(:wat::help :sym)
;; => :String — formatted EDN representation of the symbol
```

Sample interaction in a wat-repl session:

```
wat> (:wat::help :wat::core::HashMap)
(:wat::core::define :wat::core::HashMap
  ;; A keyed collection mapping K to V.
  ;; Construction: (:wat::core::HashMap :(K,V)) — empty
  ;; Verbs: assoc, dissoc, get, contains?, keys, values,
  ;;        empty?, length (per arc 058)
  ...)

wat> (:wat::help :my-app::transfer-funds)
(:wat::core::define
  (:my-app::transfer-funds (from :AccountId)
                           (to :AccountId)
                           (amount :i64)
                           (memo :string)
                           -> :Result<:TransferReceipt, :TransferError>)
  ;; Transfer `amount` from one account to another.
  (:wat::core::let* (...)))
```

The user gets a digestible form. **Formatting matches wat-fmt's
rules** so the help output reads identically to source code that
just came through the formatter.

## The dep chain (per user direction)

```
LAYER N+1 — wat-cli (per-user compiled CLI; arcs 099/100/101
              of wat-rs — user's `my-wat-cli`)
              ↓ ships by default
LAYER N   — wat-pause (arc 005)
              ↓ uses
LAYER N-1 — wat-repl (arc 012)
              ↓ uses
LAYER N-2 — wat-help (THIS ARC)
              ↓ uses
LAYER N-3 — wat-fmt (arc 003)
              wat reflection primitives (from wat-rs proper / arc 109)
```

User direction (sequence):

> *"help should depend on wat-fmt and repl should depend on help."*

> *"further... wat-repl needs to be depended on by wat-cli .... when
> a user compiles their own 'my-wat-cli' they can have the repl be
> used with whatever symbols their project created.. so help /just
> works/ in their env"*

This arc captures both contracts.

The repl's `:help` / `:doc` / `:source` / `:type` special commands
route through wat-help (currently sketched in arc 012's DESIGN as
internal commands; this arc extracts them as a foundation crate
both repl and pause-attached sessions can consume).

**The "just works" property is load-bearing.** When a user runs
`my-wat-cli repl` (their compiled CLI binding their batteries +
wat-rs core), `(:wat::help :my-app::some-symbol)` works against
their symbols WITHOUT extra registration. The cli-default battery
list ships wat-repl + wat-help; help reflects the user's frozen
world; consistency comes for free.

## Why a separate crate (and not just inside wat-repl)

Three reasons aligned with the four questions:

1. **Honest layering.** Reflection-formatting is its own concern.
   Used by repl interactively; used by MCP tools (arc 006);
   used by tutoring/learning tools; used by IDE integration
   (autocomplete-with-doc-popup); used by pause sessions.
   Living in repl forces unrelated consumers to depend on the
   whole repl machinery.

2. **Simple shape.** `(:wat::help :sym) -> :String` is one
   function. Crate scope: tiny. Easy to test in isolation.
   Easy to evolve.

3. **Composability.** wat-help's output is a `:String` that any
   IO surface can present (terminal; HTML; markdown; JSON
   wrapping; etc.). Decoupling from the repl frontend means
   non-repl consumers (MCP tools; doc generators; agent
   queries) get the same surface.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-help/`
per the arc-013 pattern. Foundation-tier alongside wat-fmt /
wat-lint / wat-cov / wat-doc.

```
wat-rs/crates/wat-help/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-fmt (../wat-fmt)
  src/                 # Rust shim (calls reflection primitives;
                       #   delegates formatting to wat-fmt)
  wat/help/            # The :wat::help form; symbol-lookup
                       #   helpers; output composition
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: substrate reflection deps; output composition; per-symbol-kind handling (function/type/value); integration with wat-fmt; comparison to wat-doc. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arc 109's reflection primitives firm up.) |

## Conventions inherited

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: typed `:HelpError` for symbol-not-found,
  reflection-failure, etc. — never panics
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: depends on wat-fmt (canonical formatter)
  + wat-rs reflection primitives; no new external Rust deps

## Cross-references

- **arc 003 (wat-fmt)** — the formatter. wat-help delegates
  output formatting here; output reads identical to source
  through wat-fmt
- **arc 005 (wat-pause)** — uses wat-repl which uses wat-help
- **arc 006 (wat-doc)** — STATIC documentation generator; consumes
  similar reflection surfaces; complementary not redundant
  (different output forms; different runtime requirements)
- **arc 012 (wat-repl)** — depends on wat-help for `:help` /
  `:doc` / `:source` / `:type` commands; arc 012's DESIGN
  should be updated to reflect this dep
- **arc 006 wat-mcp** — MCP `tools/call` for help could expose
  wat-help to agents (Claude / any MCP-speaking client gets
  typed reflection over a running wat-vm)
- **arc 109 (wat-rs mass refactor)** — supplies the reflection
  primitives this crate consumes; wat-help ships when 109's
  reflection work has firmed up enough to expose stable APIs
- **DEPENDENCY-DOCTRINE.md** — wat-fmt as a chosen dep; standard
  composition shape

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-help` (gaze-approved; universal noun for
  this kind of tool — Python's `help()`, Ruby's `help`,
  Clojure's `(doc 'sym)` and `(source 'sym)`; matches user
  expectations from every other language)
- **Architecture:** sketched
- **Slice plan:** depends on arc 109 reflection primitives;
  conservative until those firm up
- **Bar to graduate to a real wat-rs arc:**
  1. Arc 109's reflection primitives (symbol lookup; definition
     serialization; source location) have firmed up enough to
     expose stable APIs
  2. arc 003 (wat-fmt) has shipped slice 1 (so the formatter
     dep is real)
  3. User signals "let's start"

## What this arc is NOT

- **Not a static documentation generator** — that's wat-doc
  (arc 006). wat-help is RUNTIME reflection over a frozen
  symbol table; wat-doc is BUILD-TIME generation from source.
  Both useful; different concerns.
- **Not a code search tool** — that's beyond reflection;
  sibling arc if needed (`wat-search`?)
- **Not autocomplete** — reflection PROVIDES the surface that
  autocomplete consumes; the autocomplete UX layer is
  application-tier (lives in IDE integrations, repl frontends)
- **Not an IDE protocol (LSP)** — wat-help could be a building
  block for a future LSP server, but the LSP shape itself is
  out of scope

## The honest framing

wat-help is a **small, sharply-scoped crate** that earns its
existence by being the canonical surface for "give me everything
you know about this symbol, formatted readably." Every interactive
consumer (repl; pause; mcp; tutor) lands on the same answer.

Three principles:
1. **Output IS the form.** No editorial synthesis. The EDN you
   see is what the substrate has, formatted via wat-fmt's rules.
2. **One function per consumer need.** `:help` for the canonical
   "tell me about this." Future variants (`:source`, `:type`,
   `:examples`) are separate small functions, not flags on `:help`.
3. **Reflection primitives stay in wat-rs proper.** wat-help is
   a USER of those primitives, not a re-implementation. When
   reflection evolves (arc 109 work), wat-help inherits.

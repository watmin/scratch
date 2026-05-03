# wat-http-route — SLICE-PLAN

Sketch only. Not sized for shipping. The bar to graduate this
arc into a real `wat-rs/docs/arc/...` arc is:

1. arc 009 (wat-http-serve) has shipped slice 1 (or close)
2. arc 008 (wat-kwargs) has shipped slice 1
3. User signals "let's start"

When that happens, this slice plan gets re-sized against the
substrate-as-it-then-is.

---

## Slice 1 — Static-path routing

**Goal:** an app with static (no-capture) paths dispatches
correctly.

**Done when:**
- `wat-rs/crates/wat-http-route/` exists with arc-013 layout
- `wat-http-route::define-app` macro accepts `(:get path :handler h)`
  forms and compiles to a wat-http-serve::Handler
- `:get`, `:post`, `:put`, `:patch`, `:delete`, `:head`, `:options`,
  `:any` method dispatch work
- Static paths only (no `:param` syntax yet)
- 404 default response on no match
- wat-tests covering positive match, negative match, method
  mismatch
- Integration test using arc 009's listener

**Out of scope for this slice:**
- Path captures (slice 2)
- Subroute mounting (slice 3)
- Per-route middleware (slice 4)

---

## Slice 2 — Path captures

**Goal:** routes match patterns with `:param` and `*wildcard`
syntax; captures bind to named parameters.

**Done when:**
- Path pattern syntax: `:name` for single-segment captures,
  `*name` for greedy wildcard
- Path matcher tokenizes patterns and matches incoming paths
  segment-by-segment
- `(:Request/param req "id")` accessor returns captured values
- Path normalization rejects path-traversal attempts
- wat-tests covering `:single`, `:multi`, `*wild`, no-match,
  partial-match
- Documentation: pattern syntax reference

---

## Slice 3 — Subroute composition (mounting)

**Goal:** apps can mount sub-apps under a path prefix.

**Done when:**
- `(:mount "/prefix" :app sub-app)` form
- Mount strips the path prefix before dispatching to the
  sub-app
- Nested mounting works (apps in apps in apps)
- 404 on prefix-match-but-no-sub-route propagates correctly
- wat-tests covering: simple mount, nested mount, prefix
  collision

---

## Slice 4 — Per-route middleware + custom error handlers

**Goal:** routes can declare their own middleware stack;
custom error handlers for 404/500 etc.

**Done when:**
- `:middleware` kwarg on route declarations
- Per-route middleware composes with app-level middleware
  (per-route runs innermost)
- `(:not-found :handler ...)` declaration
- `(:error-handler :for :ErrorVariant :handler ...)` declaration
- wat-tests covering each composition path

---

## Slice 5 — Production hardening

**Goal:** sufficient for a real production deployment.

**Done when:**
- Performance: routing decision <1µs for apps with 100 routes
  (microbenchmarks committed)
- Documentation: complete reference for all DSL forms
- One concrete deployed application using wat-http-route
- Cookbook of common patterns (REST resource, nested resource,
  auth-gated admin)

---

## Slices NOT planned

- **Static file serving** — sibling `wat-http-static` crate
  if needed
- **Template rendering** — sibling crate if needed
- **Sessions / cookies / auth** — application-layer middleware
  via wat-http-serve middleware mechanism
- **Live route reloading** — out of scope; routes are
  compile-time
- **OpenAPI / swagger generation** — sibling crate if needed
  (wat-http-route's route table is a structured value; a doc
  generator could read it)

---

## Honest accounting

This slice plan is **sketched, not sized**. The substrate work
required for slice 1 alone may surface unknowns about wat-vm
macro expansion (does `define-app` work as a wat macro? a
substrate-level form? a runtime function?) that reshape the
later slices.

The four-questions discipline applies to each slice
independently. Each slice should answer all four with
honest checkmarks before declaring the slice done.

The single biggest open question is: **how does `define-app`
actually compile?** Is it a wat macro that expands to a match
expression? A substrate-level form with its own evaluator
hook? A runtime function that builds a route table at app
startup? This decision shapes the slice 1 implementation
significantly. It gets answered when arc 008 (wat-kwargs)
firms up — that arc establishes the precedent for how DSL-style
forms in wat get implemented, and we'll inherit its pattern.

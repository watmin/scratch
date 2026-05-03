# wat-schema — SLICE-PLAN

Sketch only. Not sized for shipping. The bar to graduate this arc
into a real `wat-rs/docs/arc/...` arc:

1. arc 008 (wat-kwargs) has shipped slice 1 (the schema DSL leans
   heavily on kwargs)
2. arc 009 (wat-http-server) has shipped slice 1 (so we know what
   a schema-validated handler signature should look like)
3. User signals "let's start"

When that happens, this slice plan gets re-sized.

---

## Slice 1 — Refined types (the foundation)

**Goal:** declare and validate refined types for strings, numbers,
collections, common domain types.

**Done when:**
- `wat-rs/crates/wat-schema/` exists with arc-013 layout
- Refined types implemented (with regex via the `regex` crate):
  - String: `:min-length`, `:max-length`, `:pattern`, `:one-of`
  - Numeric: `:range`, `:min`, `:max`, `:exclusive-range`,
    `:precision`
  - Collection: `:min-length`, `:max-length`, `:unique?`
  - Domain sugar: `:email`, `:url-https`, `:uuid`, `:hex-string`,
    `:bearer-token`, `:iso-8601`, `:rfc3339`
- `(:wat::schema::validate-type :T value)` returns
  `:Result<:T, :Violation>`
- Structured `:Violation` type carrying field path + reason +
  expected vs actual
- wat-tests covering each refinement positive + negative paths
- Documentation: refinement reference

**Out of scope for this slice:**
- Composite shapes (slice 2)
- Rules (slice 3)
- Policy actions (slice 4)
- HTTP integration (slice 5)

---

## Slice 2 — Shapes (composition)

**Goal:** declare composite shapes; validate values against them;
collect all violations (not first-fail).

**Done when:**
- `(:wat::schema::define :ShapeName ...)` declarations work
- Field modifiers: `:optional`, `:default`, `:nullable`,
  `:one-of`, `:flexible`, `:strict`
- Shape composition: `:extend`, `:and`, `:or`
- `(:wat::schema::validate :ShapeName input)` returns
  `:Result<:ShapeName, :ValidationError>`
- `:ValidationError` collects all violations in one pass (not
  short-circuit)
- wat-tests covering shape composition; field modifiers; nested
  shapes; recursive shape references
- Documentation: shape declaration reference

**Out of scope:**
- Rules (slice 3)
- Policy (slice 4)

---

## Slice 3 — Rule engine (clara-esque)

**Goal:** cross-field invariants and conditional shape requirements.

**Done when:**
- `(:rule "name" :when pattern :then action)` declarations work
  inside schema definitions
- Rule actions: `:reject` (with optional message), `:require`,
  `:warn`, `:transform`
- Pattern combinators: `:and`, `:or`, `:not`, `:absent`, `:matches`,
  field-comparison ops (`:>`, `:<`, `:=`, etc.)
- Rules evaluated in declaration order; collect violations
- Cross-field rules can reference any validated field
- wat-tests covering: simple cross-field; conditional require;
  multi-shape disambiguation; transform-then-revalidate
- Documentation: rule DSL reference; comparison to clara's pattern
  shape

---

## Slice 4 — Policy actions

**Goal:** taxonomized actions on validation failure.

**Done when:**
- `(:wat::schema::policy :name ...)` declarations work
- Built-in actions implemented:
  - `:reject-with :status N`
  - `:log-to <pipeline>`
  - `:rate-limit (:by <key> :max <rate>)`
  - `:quarantine`
  - `:transform <fn>`
  - `:custom <wat-fn>`
- Actions compose (multiple actions per policy run in order)
- Different actions per violation type
  (`:on-refinement-violation` vs `:on-rule-violation` vs
  `:on-shape-violation`)
- wat-tests covering each action; composition; tiered policies
- Documentation: policy reference; common patterns cookbook

**Notes:**
- `:mirror-to` and `:alert` actions defer to slice 6 — they
  require integrations that may not be ready

---

## Slice 5 — HTTP integration (arcs 009/010)

**Goal:** wat-http-router routes can declare `:schema` and
`:on-violation`; wat-http-server validates before handler runs.

**Done when:**
- `:schema :ShapeName` kwarg on route declarations works
- `:on-violation :PolicyName` kwarg works
- Inbound request body deserialized via wat-edn (JSON or EDN per
  Content-Type); validated against schema; rejected per policy if
  bad
- Handler signature reflects validation:
  `:Handler<:Request<:ShapeName>>`
- Optional response validation: `:schema-out :ResponseShape`
- Round-trip integration test: arcs 009 + 010 + 013 working
  together; valid request flows through; invalid request gets
  policy-rejected
- Documentation: HTTP cookbook (common schema + policy patterns
  for REST APIs)

**Depends on:** arcs 009 + 010 shipping slice 1

---

## Slice 6 — RPC + config + IPC integration

**Goal:** wat-schema usable from arc 007 (RemoteProgram), config
loading, and arc 109 (program-with-mailbox).

**Done when:**
- arc 007 RemoteProgram declares `:input-schema` and
  `:output-schema`; validation runs at typed-RPC boundaries
- `(:wat::schema::validate-and-load :ConfigShape :path "...")`
  works for config files
- arc 109 program-with-mailbox declares `:message-schema`;
  invalid messages dropped per policy
- Cross-arc integration tests demonstrate each boundary
- Documentation: cross-boundary patterns

**Depends on:** arcs 007, 109 progressing

---

## Slice 7 — Production hardening

**Goal:** wat-schema is genuinely usable as the default at every
boundary in production wat applications.

**Done when:**
- Performance: validation overhead < 100µs for typical schemas
  (microbenchmarks committed)
- Memory: validation doesn't allocate unboundedly for streaming
  inputs
- `:mirror-to` and `:alert` policy actions implemented (slice 4
  deferred)
- Schema metrics emitted (validation latency; failure rate by
  type; rule-fire counts)
- Documentation: complete reference; security cookbook; comparison
  to WAFs; "the positive security argument" written for security
  review
- One concrete deployed application using wat-schema at every
  boundary

---

## Slices NOT planned

- **Async validation** — schemas are synchronous; semantic checks
  belong in handlers
- **Schema migration / versioning** — sibling crate
  (`wat-schema-migrate`?) if patterns surface
- **OpenAPI / JSON Schema export** — sibling crate
  (`wat-schema-export`?) if SDK generation needs surface
- **Code generation from schemas** — sibling crate if needed
- **GUI schema editor** — schemas are wat code; edit in your
  editor

---

## Honest accounting

This slice plan is **sketched, not sized**. The biggest unknown:
how does the rule engine integrate with the wat-vm's existing
match expression / pattern primitives? Lean toward expressing
rules as wat-vm-native pattern matching (so the substrate's
existing pattern facilities serve the rule engine; we don't
build a parallel pattern language).

The four-questions discipline applies to each slice independently.
Each slice should answer all four with honest checkmarks before
declaring the slice done.

The single biggest question: **can we structurally guarantee that
no validation can ever throw or panic?** If yes, the architecture
earns ✅✅✅ Honest (per the four questions). If no, ✅✅ stands.
This is worth design-time effort to lock down — total functions
all the way through is the "failure is data" position made
structural.

## Why this slice plan is conservative

The user's intent is to NOT need a WAF. That means wat-schema
needs to be production-ready before they can REPLACE WAF
deployment with schema enforcement. Slice 7's "one concrete
deployed application" is the proof point.

But: the substrate already does most of the work. The wat type
system is the heaviest lift; that's already in wat-rs. wat-schema
adds the thin refinement / shape / rule / policy layers on top.
**The actual implementation lift is small.** The slice plan
mostly captures the discipline of doing it right (per
four-questions) rather than the engineering effort of doing it
at all.

# wat-kwargs — SLICE PLAN

Four slices. Each ships independently. Per proposal 058's
discipline (ship only what's earned by cited use), each slice
has a real consumer that demands it.

The slice plan was reshaped 2026-05-03 after the auto-generation
direction landed (`:wat::kwargs::auto-kwargs` reads function
signatures and generates kwarg variants automatically). The
zero-boilerplate form is the headline; explicit-list form is
the lower-level primitive.

---

## Slice 1 — Substrate prerequisite + lower-level primitive

**Goal:** verify (or add) the macro-time signature lookup
primitive. Ship the explicit-list `define-with-kwargs` macro
as the lower-level building block.

**Deliverables:**

*wat-rs core check:*
- Verify `:wat::core::sig-of <fn-name>` (or equivalent)
  exists and returns the function's signature as a HolonAST
  the macro can inspect
- If missing, propose substrate addition in
  `wat-rs/docs/proposals/`; small primitive (look up the
  function's signature; return its AST)

*wat-kwargs crate* (`wat-rs/crates/wat-kwargs/`):
- Workspace member crate created per arc-013 contract
- Public API: arc-013 contract (`wat_sources()` + `register()`)
- The `:wat::kwargs::define-with-kwargs` macro (lower-level)
  - Takes: macro name, underlying function name, ordered kwarg
    list, docstring
  - Generates: a defmacro that accepts variadic kwargs and
    expands to a positional call
  - Handles missing required keyword → macro-time error
  - Handles unknown keyword → macro-time error
  - Handles duplicate keyword → macro-time error
- The first reference implementation: hand-written kwarg macro
  for one of RemoteProgram's constructors (probably mTLS) using
  `define-with-kwargs`

*Tests:*
- Macro expansion correctness (input kwargs → expected positional
  call)
- Diagnostic quality (missing/unknown/duplicate kwargs)
- Round-trip with the underlying typed function

**Consumer:** RemoteProgram (arc 007) — its mTLS constructor
gets the kwarg variant via `define-with-kwargs` as the
reference implementation.

**Out of scope:**
- Auto-generation `auto-kwargs` (slice 2 — needs slice 1's
  primitive in place first)
- wat-fmt convention (slice 3)
- wat-lint suggestion rule (slice 4)

**Estimated size:**
- ~50 LOC of wat-rs core changes if `sig-of` doesn't exist
- ~200-400 LOC of wat code in wat-kwargs (the
  `define-with-kwargs` macro + its parser + diagnostics)
- ~100-200 LOC of Rust shim (arc-013 contract)
- Reference impl: ~10-20 LOC of wat per use

---

## Slice 2 — `:wat::kwargs::auto-kwargs` (zero-boilerplate)

**Goal:** the headline macro. Reads a function's signature;
auto-generates the kwarg variant.

**Deliverables:**

*wat-kwargs crate:*
- The `:wat::kwargs::auto-kwargs` macro
  - Takes: function name + optional `:as` and `:doc` overrides
  - Looks up the function's signature via `:wat::core::sig-of`
  - Extracts parameter names + order
  - Auto-derives the macro name (`with-X` per convention; or
    `:as` override)
  - Auto-generates the macro's docstring (referencing the
    function's docstring; or `:doc` override)
  - Generates a `define-with-kwargs` call internally
- Convention documentation (the `with-X` naming) referenced
  from wat-doc's pattern catalog (when wat-doc ships)

*Tests:*
- Auto-generated macro produces same expansion as hand-written
  equivalent
- Override hooks work (`:as` renames; `:doc` replaces docstring)
- Auto-derivation handles function names with `/` separators
  (e.g., `Program/remote-mtls` → `Program/with-remote-mtls`)
- Re-expansion picks up signature changes (move a parameter;
  re-expand; new variant reflects the move)

**Consumer:** RemoteProgram's other constructors (Tier 1, 2, 3)
adopt `auto-kwargs` instead of hand-written `define-with-kwargs`
calls. Demonstrates the boilerplate reduction.

**Out of scope:**
- wat-fmt convention (slice 3)
- wat-lint suggestion rule (slice 4)

**Estimated size:**
- ~300-500 LOC of wat code in wat-kwargs (the auto-kwargs
  macro + its derivation logic)
- Tests: ~200-300 LOC

---

## Slice 3 — wat-fmt convention amendment

**Goal:** wat-fmt formats kwarg macro calls consistently with
HashMap construction (per Rule 22's no-alignment, single-space
discipline).

**Deliverables:**

*wat-fmt amendments:*
- New rule entry in STYLE-RULES.md: "Kwarg macro calls"
- Format pattern: keyword-on-line; value follows with single
  space; one keyword per line for multi-arg cases
- Single-line form for short macros (TBD threshold)

```scheme
;; Multi-arg (canonical):
(:wat::remote::Program/with-mtls
  :host        "api.example.com"
  :port        443
  :client-cert cert
  :client-key  key
  :ca-cert     ca-cert)

;; (TBD) Single-line for short:
(:my::with-fn :first 1 :second 2)
```

*Tests:*
- Golden files for kwarg macro calls in various sizes
- Round-trip with the lock-format-once-canonical discipline

**Consumer:** wat-fmt itself; downstream projects formatting
their own kwarg macro calls.

**Out of scope:**
- wat-lint suggestion rule (slice 4)

**Estimated size:**
- STYLE-RULES.md amendment (small)
- ~50-100 LOC of changes in wat-fmt's per-form emitters
- Tests: golden files

---

## Slice 4 — wat-lint suggestion rule

**Goal:** wat-lint flags functions that would benefit from a
kwarg macro and suggests `auto-kwargs`. Optional discipline;
default informational.

**Deliverables:**

*wat-lint integration:*
- New rule: `kwargs/long-positional-arg-list`
- Triggers: function has 4+ args AND 2+ args have the same type
- Default severity: L3-candidate (informational; not
  prescriptive)
- Hint: "consider `(:wat::kwargs::auto-kwargs <fn-name>)` for a
  kwarg variant"
- Rune: `kwargs(intentional-positional)` — function has a
  natural positional shape (e.g., point coordinates `(x, y, z)`)

*Tests:*
- Trigger cases: 5-arg with 3 same types → flag
- Non-trigger cases: 3-arg with distinct types → don't flag
- Rune suppression works

**Consumer:** wat-rs codebase + downstream projects wanting
the discipline.

**Out of scope:**
- Anything else.

**Estimated size:**
- ~100-200 LOC of wat (the lint rule)
- Tests: golden findings

---

## Sequencing

Linear 1 → 2 → 3 → 4. Slice 2 strictly depends on slice 1
(needs the primitive). Slice 3 + 4 can land in any order after
slice 2.

| Slice | Depends on | Blocks | Estimated time |
|---|---|---|---|
| 1 | wat-rs sig-of primitive | 2, 3, 4 | 1-2 weeks |
| 2 | 1 | 4 (lint hint references auto-kwargs) | 1 week |
| 3 | 1 | nothing | 3-5 days |
| 4 | 2 | nothing | 3-5 days |

Slice 1 is the bulk (substrate verification + the lower-level
primitive). Slice 2 is the headline (zero-boilerplate
auto-generation).

## What success looks like

- **RemoteProgram's four constructors** all use `auto-kwargs`
  by slice 2; each is one line of declaration on top of the
  underlying typed function
- **wat-fmt formats kwarg macro calls** consistently (slice 3)
- **wat-lint surfaces opportunity** without prescribing
  (slice 4)
- **Pattern is documentable + adoptable**: any consumer crate
  with a multi-arg API can drop in `(auto-kwargs <fn>)` for a
  kwarg variant
- **Cross-cutting**: this arc unlocks better ergonomics across
  the whole ecosystem, not just RemoteProgram

## What's deferred

- **Default values for omitted kwargs** — `[(host) (port 443)
  (client-cert) ...]` to default `:port` to 443 if omitted.
  Powerful but adds complexity. Maybe v2.
- **Strict vs lax validation modes** — per-call configuration
  of strict (error on unknown kwarg) vs lax (warn). Pick
  strict by default in v1; add lax later if real consumers
  want it.
- **Kwarg-spec reflection at runtime** — exposing "what kwargs
  does this macro take?" via a runtime API. Downstream tooling
  reads docstrings instead.
- **Substrate-level keyword arguments** — would obviate this
  pattern. Separate, larger arc; until then, this pattern is
  the answer.

# wat-kwargs — type-less keyword arguments via macros

User direction (2026-05-03):

> *"let's get a new arc just on type-less kwargs for function
> calls... i want this to be something remote program deps on..."*

This arc captures a CROSS-CUTTING design pattern: keyword
arguments at function call sites, implemented as a macro
layer over typed positional functions. Substrate-tier concern;
many consumer crates can adopt it. RemoteProgram (arc 007)
depends on it.

---

## What this is

**A pattern for making multi-arg function calls self-documenting
at the call site.** Two surfaces per API:

- **Layer 1** — the typed positional substrate function (where
  types live)
- **Layer 2** — a macro that accepts keyword-argument syntax
  and expands to the typed positional call

The user-facing surface is the macro (clean, kwarg-style, no
visible types). The substrate's type system enforces correctness
on the expansion. **Two surfaces; one truth.**

## Why this exists

Multi-arg functions with similarly-typed parameters are
positionally fragile. The mTLS constructor for RemoteProgram is
the case-in-point: 3 of its 5 args are `:Path`, and a positional
swap (`client-cert` ↔ `client-key`) wouldn't be caught by the
type checker.

Compare:

```scheme
;; Positional — error-prone for similar-typed args
(:wat::remote::Program/remote-mtls
  "api.example.com" 443 cert-path key-path ca-cert-path)

;; Kwarg macro — obvious, self-documenting
(:wat::remote::Program/with-mtls
  :host        "api.example.com"
  :port        443
  :client-cert cert-path
  :client-key  key-path
  :ca-cert     ca-cert-path)
```

The kwarg form prevents the swap by NAMING the slots. The macro
makes the syntax possible without substrate changes.

## "Type-less" means at the consumer site

Per user framing — "type-less kwargs":

- **At the consumer call site:** types are completely invisible.
  No annotations; just `:keyword value` pairs.
- **At the substrate function definition:** types REMAIN.
  Every typed function still declares its signature in full.
- **The macro absorbs the gap:** transforms the type-less
  consumer surface into the fully-typed substrate call.

The discipline is honest: types live in the function definition
where they belong; consumers don't repeat them; the macro is
the bridge.

## Where it lives

**Single self-contained crate (proposed):**
`wat-rs/crates/wat-kwargs/` — same arc-013 pattern as the
foundation-tier crates.

The crate is small in scope. It ships:
1. A documented PATTERN (this arc's content; rendered as
   per-crate guidance)
2. A helper macro `:wat::kwargs::define-with-kwargs` that
   reduces the boilerplate of writing kwarg macros (slice 2)
3. A wat-fmt convention for kwarg macro calls (slice 3)
4. A wat-lint rule that flags kwarg-eligible functions (slice 4)

Most of the value is the pattern documentation; the helper
macro is convenience.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | The pattern in detail; concrete examples; the four-questions analysis; cross-references to wat-fmt / wat-lint / wat-doc. |
| `SLICE-PLAN.md` | 4 slices: (1) pattern documentation + first reference impl; (2) `define-with-kwargs` helper macro; (3) wat-fmt convention; (4) wat-lint suggestion rule. |

## Conventions inherited

From the foundation-tier arcs:

- **The four questions as design compass** (Obvious / Simple /
  Honest / Good UX) — this pattern trades small "obvious" cost
  for big "good UX" win
- **Atomicity-and-signal principle** — types are atomic in the
  substrate; the macro doesn't violate type discipline
- **Arc-013 self-contained crate pattern** for any code that
  ships
- **LLM out** — this is mechanical pattern infrastructure; no
  LLM in the loop

## Cross-references

- **arc 007 (RemoteProgram)** — the first cited consumer; its
  four constructors use this pattern; this arc's slice 1
  reference impl IS RemoteProgram's mTLS constructor
- **wat-fmt (003)** — formatting convention for kwarg macro
  calls (slice 3)
- **wat-lint (004)** — suggestion rule for "this function
  signature would benefit from a kwarg macro" (slice 4)
- **wat-doc (006)** — both surfaces (macro + underlying
  function) need docstrings; the docstring discipline already
  established
- **058-031 (defmacro)** + **058-032 (typed-macros)** — the
  substrate macro infrastructure this pattern builds on
- **058-034 (stream-stdlib)** — the variadic `&` rest-param
  syntax used to capture the kwargs vec

## Status

- **Captured:** 2026-05-03
- **Architecture:** the pattern is articulated; helper macro
  shape sketched; integration points (wat-fmt, wat-lint,
  wat-doc) named
- **Slice plan:** 4 slices sized
- **Bar to graduate to a real wat-rs arc:**
  1. wat-fmt slice 1 has shipped
  2. RemoteProgram (007) has at least one constructor that
     wants this pattern
  3. User signals "let's start"
- **Dependency**: arc 007 (RemoteProgram) depends on this arc

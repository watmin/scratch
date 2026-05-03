# wat-define-nary — N-ary dispatch as a separate define form

User direction (2026-05-03):

> *"i don't know if rust lets us have many sigs for some name?.. but...
> in wat... we totally can?...*
>
> *i guess i'm asking - should we support N-ary or deliver a convention
> with our existing tools"*

After exploring three options (N-ary defn / variadic args / Vec+match)
and recognizing wat doesn't have `[...]` literal syntax, user direction:

> *"don't shim something into define.. make a new kind who allows N-ary
> exprs..."*

> *"get a new scratch for this... we'll entertian adding this as a
> post 109 closure thing..."*

> *"no new primitives in core until the mass refactor is done"*

---

## What wat-define-nary is

A **separate `define` form** that supports Clojure-style N-ary
dispatch — one function name with multiple bodies, one per arity.

Single return type at the head. Multiple arity bodies in the body.
Substrate-level dispatch by argument count at call site.

```scheme
(:wat::core::define-nary :my-merge-many -> :HashMap<K,V>
  ;; arity 0
  (()
    (:wat::core::HashMap :(K,V)))

  ;; arity 1
  (((m :HashMap<K,V>))
    m)

  ;; arity 2
  (((m1 :HashMap<K,V>) (m2 :HashMap<K,V>))
    (:my-merge m1 m2))

  ;; arity 3
  (((m1 :HashMap<K,V>) (m2 :HashMap<K,V>) (m3 :HashMap<K,V>))
    (:my-merge (:my-merge m1 m2) m3)))

;; Caller uses any declared arity:
(:my-merge-many)              ;; 0-ary; returns empty HashMap
(:my-merge-many m1)           ;; 1-ary; returns m1
(:my-merge-many m1 m2 m3)     ;; 3-ary; nested merge
```

## Why a NEW form, not extending `define`

User direction:

> *"don't shim something into define.. make a new kind who allows
> N-ary exprs..."*

The principle: `:wat::core::define` stays simple for the common case
(single arity). N-ary dispatch is opt-in via a separate form. The
type checker has a clear hook for the new form; the existing
checker for `define` doesn't grow another mode.

Per the four questions:
- **Simple** ✅✅ — neither form grows complexity to accommodate the
  other; each does one thing
- **Honest** ✅✅ — at the call site, you can tell whether the
  callee is N-ary by which form defined it; no hidden multi-arity
  in a function defined via `define`
- **Good UX** — common case stays simple; complex case is
  visibly opt-in via a different form name

## Why this is gated post-109

User direction:

> *"no new primitives in core until the mass refactor is done"*

Arc 109 is the wat-rs mass refactor — currently revealing every
bug it touches. Adding `define-nary` to wat-core during that work
would mean either:
1. Shipping a new primitive into a moving substrate (risky)
2. Holding 109 to integrate with the new form (slows the refactor)

Both are bad. **The refactor closes first; then post-109 work
considers adding `define-nary` to wat-core.** This arc captures
the design now so the work is sketched and ready when 109 closes,
but no implementation work begins until then.

## Where it would live (post-109)

`define-nary` is **substrate-level** — it's a new wat-core form, not
a crate-level pattern. Lives in wat-rs proper:

- Parser: recognizes the form syntax
- Type checker: validates each arity body against the declared
  return type; checks no two arities collide on arg count
- Evaluator: dispatches to the matching arity body at call site;
  errors clearly if no arity matches

This is NOT a scratch-arc-graduates-to-crate pattern (like 003-006).
It's a substrate addition. The slice plan reflects this — ALL slices
are wat-rs work, not crate work.

## Layering

```
LAYER — user wat code           uses (:wat::core::define-nary ...)
  ↓ uses
LAYER — wat-rs substrate        define-nary form (parser + type
                                  checker + evaluator)
  ↓
LAYER — Rust ecosystem          (no new external deps)
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: form syntax, dispatch semantics, type system implications, the convention-vs-primitive analysis, per-four-questions. |
| `SLICE-PLAN.md` | Slices for post-109 implementation. Conservative; gated. |

## Conventions inherited

- Four questions as design compass
- Failure engineering: dispatch errors are typed (no panic on
  arity mismatch — typed `:NoArityMatch` error)
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: no new external deps; pure substrate work
- **The "wat-rs verbose-by-design" principle** (per user direction):
  the new form uses fully explicit FQDN syntax;
  shortnames/ergonomics layer (`wat-common-shortnames` or similar)
  is OPT-IN and ships separately when the substrate stabilizes

## Cross-references

- **arc 008 (wat-kwargs)** — surfaced this discussion as a
  digression while exploring kwargs composability via merge-many.
  See arc 008's captured-beat for the closure note pointing here.
- **arc 109 (wat-rs mass refactor)** — the gate. This arc is
  POST-109 work; no implementation begins until 109 closes.
- **wat-rs/docs/CONVENTIONS.md** — substrate naming + verbosity
  conventions this arc respects
- **wat-rs/docs/USER-GUIDE.md** — `:wat::core::define`'s current
  shape; this arc adds a sibling form, doesn't modify `define`

## Status

- **Captured:** 2026-05-03
- **Naming:** `define-nary` (gaze-approved working name; "n-ary"
  is the canonical CS jargon for variable-arity functions; doesn't
  collide with type polymorphism; debatable later if a stronger
  name emerges)
- **Architecture:** sketched
- **Slice plan:** post-109; conservative
- **Substrate gate:** NO IMPLEMENTATION until arc 109 closes; per
  user direction "no new primitives in core until the mass
  refactor is done"
- **Bar to graduate:**
  1. Arc 109 has closed (the mass refactor finished; substrate
     stable)
  2. User signals "let's start"
  3. Concrete use case in user code that's painful enough today
     to justify the substrate addition (the kwargs work doesn't
     create that pressure — kwargs use hash-map composition,
     not N-ary dispatch)

## The honest framing

This arc captures a DESIGN ASK, not a commitment to ship. The
ask is real (N-ary defn would help certain functional patterns),
but:

1. The kwargs work that triggered the discussion DOESN'T need
   N-ary — kwargs dispatch on KEYS, not arity
2. The merge-many example is expressible today via `foldl` over
   a Vec arg (no N-ary needed)
3. Adding `define-nary` is genuine substrate work; should not
   happen during 109's mass refactor

The arc exists so we don't lose the design thread. When 109
closes and someone asks "should we add N-ary?", this scratch is
the starting point.

## What this arc is NOT

- Not a substrate change (yet) — design only
- Not a feature commitment — design ask captured for evaluation
- Not arc-008-blocking — kwargs ships without N-ary
- Not a vehicle for relitigating the bracket question (`[...]`
  literal sugar) — that's a separate scratch ASK if it surfaces
- Not a vehicle for variadic args — that's another separate
  ASK if it surfaces; orthogonal to N-ary

## A note on the bracket question

During the discussion that surfaced this arc, the related
question came up: **should wat add `[...]` literal sugar for
Vec?** wat is currently parens-only (Lisp-orthodox); EDN supports
`[...]` and `{...}` at the wire layer; the source/wire asymmetry
is itself a friction.

That question is ALSO post-109 territory. Same gate. If a future
arc opens for it, cross-reference here.

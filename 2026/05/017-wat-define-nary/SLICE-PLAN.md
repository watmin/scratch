# wat-define-nary — SLICE-PLAN

Conservative; post-109 gated; design-only until the substrate
stabilizes.

---

## Pre-implementation gate — arc 109 closure

**No implementation slices begin until arc 109 (the wat-rs mass
refactor) has closed.** Per user direction:

> *"no new primitives in core until the mass refactor is done"*

This is not arbitrary. Adding new primitives to a moving substrate
either:
- Ships into instability (risky; the new primitive might need
  rework as 109 reshapes things)
- Holds 109 to integrate (slows the refactor that's revealing
  every bug)

Both are bad. **Wait.** When 109 closes, evaluate:

1. Is the substrate stable? (No active refactor of evaluator,
   type checker, parser)
2. Has a concrete use case surfaced that's painful enough today
   to justify the substrate addition? (kwargs work — arc 008 —
   does NOT create that pressure)
3. Has the user signaled "let's start"?

If yes to all three, slice 1 begins.

---

## Slice 1 — Lower bound: arity-only dispatch (parser + type checker)

**Goal:** parse `(:wat::core::define-nary ...)` form; type-check
each arity body; reject invalid declarations.

**Done when:**
- Parser recognizes the new form
- Each arity is parsed as `((args) body)` pair
- Type checker validates each body against the declared
  `:ReturnType`
- Duplicate-arity check (no two arity bodies with same arg count)
- Error messages clear and actionable
- wat-tests cover: simple multi-arity definition; missing return
  type; duplicate arity; type-mismatched body

**Out of scope for this slice:**
- Evaluator dispatch (slice 2)
- Pattern-clause dispatch (slice 4 if pursued)
- Guards (slice 5 if pursued)

---

## Slice 2 — Lower bound: evaluator dispatch

**Goal:** at call site, look up the function; count caller args;
dispatch to matching arity body.

**Done when:**
- Symbol table stores arity table per `define-nary` function
- Evaluator counts caller args at call site
- Dispatch picks matching arity body; runs it
- `:NoArityMatch` typed error if no arity matches; declared
  arities listed in error
- Round-trip integration test: define a 3-arity function; call
  each arity; verify correct body fires
- Compile-time arity inference where the count is statically
  known (per arc 109's static-shape work)

---

## Slice 3 — Lower bound production hardening

**Goal:** lower bound is genuinely usable in production wat code.

**Done when:**
- Documentation: USER-GUIDE entry for `define-nary`; comparison to
  `define`; common-pattern cookbook
- Error messages reviewed for clarity
- Performance: dispatch overhead < 1µs (microbenchmark committed)
- One concrete use case in user code adopts the form (proves
  the value)

---

## Optional Slice 4 — Upper bound: pattern-clause dispatch

**Goal:** Erlang-style — clauses can have argument patterns,
dispatch picks first matching clause.

**Decision point:** does Slice 3 reveal real demand for
patterns beyond bare arity? If yes, proceed; if no, leave at
lower bound.

**Done when:**
- Each arity body's args parsed as patterns (literal, variable,
  structural — same vocabulary as `:wat::core::match`)
- Type checker validates patterns against arg types
- Evaluator tries patterns in declaration order; first match wins
- `:NoMatchingClause` typed error if no clause matches; clauses
  listed in error
- Exhaustiveness check (per arc 055 narrowing-pattern rules)
- wat-tests cover: literal patterns; variable patterns; structural
  patterns; pattern-failure error reporting

---

## Optional Slice 5 — Upper bound: guards

**Goal:** clauses can have `:when <expr>` guard expressions; body
fires only if guard is true.

**Decision point:** does Slice 4 reveal real demand for guards?

**Done when:**
- `:when` clause parsed in arity head
- Guard expression type-checked to return `:bool`
- Guard evaluated AFTER patterns match; AS PART OF clause selection
- `:NoMatchingGuard` typed error if all patterns matched but no
  guard true
- Documentation: guard semantics; comparison to Erlang

---

## Slices NOT planned

- **Modifying `:wat::core::define`** — explicit user direction
  ("don't shim something into define")
- **Variadic args** — orthogonal; separate ASK if it surfaces
- **Multiple return types per arity** — explicitly disallowed
  per user direction
- **Macro-only implementation** — would lose error-message
  quality and dispatch performance; substrate native is the
  right shape

---

## Honest accounting

This slice plan is **sketched, not sized for execution.** All
implementation work is post-109. Slice 1's start date is
unknown.

The biggest unknown: whether the upper bound (Erlang pattern +
guards) is worth implementing. The decision should be driven by
real wat code that's painful WITHOUT it, not by anticipated
elegance. Per the four-questions discipline: optimize for what's
there, not what might be there.

If the lower bound (Slices 1-3) ships and serves real use cases,
the upper bound (Slices 4-5) may never be needed. That's fine —
the design captures both bounds so the future conversation has
context, not because both bounds must ship.

Estimated effort:
- Lower bound (Slices 1-3): ~600 lines substrate Rust + ~50
  lines docs; ~5-7 days post-109
- Upper bound (Slices 4-5): ~1500-2000 additional lines; ~10-14
  days; only if needed

---

## The core principle

**Substrate primitives are forever.** Once `define-nary` ships,
existing wat code may depend on it; future-us cannot remove it
without breaking compatibility. Therefore:

1. Wait for substrate stability (post-109)
2. Wait for concrete demand (real use case in real code)
3. Ship the minimum (lower bound only at first)
4. Add upper-bound features only if real demand surfaces

The wait is the discipline. The arc captures the design so when
the time comes, we don't re-derive — but the implementation
patience is itself the design choice.

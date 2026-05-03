# wat-define-nary — DESIGN

A separate `define` form supporting multi-arity dispatch.
Substrate-tier; post-109 gated; design-only until the mass
refactor closes.

---

## The four questions are the design compass

- **Obvious?** Separate form name (`define-nary`) signals "this is
  the multi-arity case." Common case stays with `define`.
- **Simple?** Each form does one thing. `define` doesn't grow a
  mode for multi-arity; `define-nary` doesn't pretend to be the
  default.
- **Honest?** Single return type at the head of the form; arity
  bodies are visible as a list of (args body) pairs; dispatch
  happens at call site by arg count (and possibly pattern, see
  upper-bound design).
- **Good UX?** Caller writes `(my-fn a b c)` directly; substrate
  picks the matching arity body; arity mismatch is a typed error
  at compile time.

## The two design bounds

**LOWER BOUND — Clojure-style: arity-only dispatch.**
Each arity body is matched purely by argument count.

**UPPER BOUND — Erlang-style: arity + pattern + optional guards.**
Each arity body has typed arg patterns (literal values; structural
patterns; bound variables) plus an optional guard expression.

Both shapes capture the user's "function with multiple bodies"
intent. Erlang-style is strictly more expressive. The implementation
cut between them is a slice-time decision (post-109); **the design
captures both bounds so the future conversation has the full space.**

## Lower bound — Clojure-style arity-only

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
```

Form structure:
- `(:wat::core::define-nary :name -> :ReturnType` opens
- Body is a sequence of arity pairs: `((args) body)`
- Each pair declares one arity
- Closing `)` on the outermost form
- Single return type covers all arities

Type checking:
- Each body checked against `:ReturnType` independently
- No two arities may declare the same arg count (clear duplicate
  error)
- All arg types fully explicit (per wat-rs verbose-by-design)

Dispatch:
- Caller: `(:my-merge-many m1 m2 m3)`
- Substrate: counts caller args (3); looks up the 3-ary body;
  invokes
- Mismatch: typed `:NoArityMatch` error at compile time (or
  call-site arity inference if the count is known)

This is the MINIMUM. Most use cases satisfied: optional args via
default-arity wrappers; varying parameter sets per arity; clear
documentation per arity body.

## Upper bound — Erlang-style pattern + guards

```scheme
;; HYPOTHETICAL — extended design upper bound; NOT settled
(:wat::core::define-nary :factorial -> :i64
  ;; arity 1, literal pattern
  (((0))
    1)

  ;; arity 1, with guard
  (((n :i64)) :when (:wat::core::i64::> n 0)
    (:wat::core::i64::* n (:factorial (:wat::core::i64::- n 1))))

  ;; arity 1, fallback
  (((n :i64))
    (:wat::core::error :NegativeFactorial n)))
```

Erlang-style adds:
- **Literal patterns** — match an arg if it equals a literal
- **Structural patterns** — match an arg via the substrate's
  match patterns (same vocabulary as `:wat::core::match`)
- **Guards** — `:when <expr>` clause; body fires only if guard
  is true
- **Fallback ordering** — clauses tried in declaration order;
  first matching wins

This is significantly more expressive but adds substrate complexity:
- Type checker validates patterns against arg types
- Dispatch extends from "arg count check" to "pattern match +
  guard evaluation"
- Exhaustiveness checking gets more complex (similar to `match`
  exhaustiveness; per arc 055)
- Error messages need to name which clause failed and why

## Convention-vs-primitive analysis

Three options were considered before settling on this arc:

**Option A — N-ary defn (this arc).** Substrate work; cleanest
caller ergonomics; opt-in via separate form.

**Option B — Variadic args (`...args :T`).** Lighter substrate
work (rest-param syntax sugar); same caller ergonomics for
homogeneous variadic; doesn't help heterogeneous-arity dispatch.

**Option C — Vec arg + match.** Zero substrate work; works today;
caller wraps args in `(:wat::core::vec :T ...)` (verbose without
`[...]` literal sugar); function uses foldl/length/get internally.

For the immediate kwargs use case (arc 008): Option C is
sufficient — kwargs dispatch on KEYS, not arity. The merge-many
example expressible today via foldl with empty? check.

For the broader question "should wat support multi-arity": this
arc captures the design ASK. The substrate addition is real;
post-109 gated; user decides at that time.

## Per the four questions on each design bound

**Lower bound (arity-only):**
- Obvious  ✅✅ — separate form names what it does
- Simple   ✅   — single mechanism (count + dispatch); no pattern
                  matching; no guards
- Honest   ✅✅ — single return type; arities visible; mismatch
                  caught at compile time
- Good UX  ✅✅ — caller writes natural; substrate dispatches

**Upper bound (Erlang-full):**
- Obvious  ✅   — clauses with patterns are Erlang-familiar; less
                  obvious to non-Erlang users
- Simple   ❌   — pattern + guard checking adds substantial
                  substrate complexity
- Honest   ✅✅ — patterns visible; guards visible; declaration
                  order is the dispatch order
- Good UX  ✅✅ — most expressive option; covers any function-
                  shape pattern

The lower bound is the obvious starting point if implementation
ever begins. Upper-bound features could be added incrementally
later if patterns surface in real wat code.

## Substrate work required (post-109)

Implementation requires:
1. **Parser** — recognize `(:wat::core::define-nary :name -> :T
   body...)` form; parse each (args body) pair
2. **Type checker** — validate each body returns `:T`; check
   no-duplicate-arity rule; (upper bound) validate patterns and
   guards
3. **Symbol table** — register the function with its arity table
   (mapping arg-count → body); (upper bound) register patterns
   and guards alongside
4. **Evaluator** — at call site, look up the function; count
   args; dispatch to matching arity body; (upper bound) try
   patterns in order; check guards; pick first match
5. **Error reporting** — clear `:NoArityMatch` errors with
   declared arities listed; (upper bound) clear pattern-mismatch
   errors per clause

Estimated complexity:
- Lower bound: ~400-600 lines of substrate Rust + ~50 lines of
  user-guide docs (matches arc 008's earlier substrate estimates)
- Upper bound: ~1500-2500 lines of substrate Rust (pattern
  matching machinery overlaps with existing match; guard
  evaluation is incremental)

## Cross-references

- **arc 008 (wat-kwargs)** — the conversation that surfaced this
  arc; arc 008's captured-beat closes the loop pointing here
- **arc 109 (wat-rs mass refactor)** — the gate. No
  implementation until 109 closes. Substrate stability needed
  before adding primitives.
- **wat-rs/docs/CONVENTIONS.md** — verbose-by-design principle
  this arc respects
- **wat-rs/docs/USER-GUIDE.md** — `:wat::core::define`'s current
  shape; `:wat::core::match`'s pattern vocabulary (shared with
  this arc's upper-bound design)

## Open architectural questions

A. **Default arity body.** Erlang allows a "fallback clause" via
   bare-symbol patterns. Should `define-nary` allow a "default"
   arity (e.g., `((:default :args)) ...body...`)? Lean: NO for
   lower bound; the pattern-clause shape (upper bound) gives this
   for free via fallthrough.

B. **Recursion across arities.** Can the 2-ary body call the
   3-ary body? Yes — both are bound to the same name. Caller
   semantics: `(:my-fn a b c)` from inside the 2-ary body
   dispatches to 3-ary. No special syntax needed.

C. **Parametric type variables across arities.** Should `K` and
   `V` in `:HashMap<K,V>` be shared across all arity bodies, or
   independently quantified per arity? Lean: shared at the
   define-nary level (declared at the head; all arities use the
   same type variables).

D. **Variadic + N-ary interaction.** If wat someday adds variadic
   args (`...args`), how do they interact with N-ary? Lean:
   variadic is its own form (`define-variadic`?); doesn't
   compose with N-ary; user picks one shape per function.

E. **Macro vs evaluator-level dispatch.** Could `define-nary` be
   implemented as a macro that expands to a single `define` with
   internal arity-checking via match? Lean: NO — the substrate
   should know about the multi-arity shape natively for clean
   error reporting and dispatch performance. Macro-only
   implementation would lose those properties.

## What's NOT in scope

- **Variadic args (`...args :T`)** — separate ASK; orthogonal;
  could be its own future arc
- **Pattern-clause matching beyond arity** — captured as upper-
  bound design; implementation cut is post-109 decision
- **Multiple return types per arity** — explicitly disallowed
  per user direction ("the func may only have one ret val")
- **Square bracket literal sugar (`[...]`)** — separate question
  surfaced in the same conversation; not this arc's concern
- **Implementation work** — POST-109 gated per user direction
- **Modifying `:wat::core::define`** — explicitly NOT shimming
  N-ary into define; this is a separate form

## The honest framing

This arc is a DESIGN PLACEHOLDER, not an implementation plan.

What it captures:
- The design space (lower + upper bounds)
- The substrate work required
- The four-questions analysis on each bound
- The Erlang resonance the user named
- The cross-references for context

What it doesn't commit:
- Any implementation work (post-109 gated)
- A specific cut between lower and upper bound (decided when
  implementation begins)
- A timeline (depends on 109's closure + concrete pressure for
  the feature)

The arc exists so we don't lose the design thread. When 109
closes and someone asks "should we add N-ary defn?", this scratch
is the starting point.

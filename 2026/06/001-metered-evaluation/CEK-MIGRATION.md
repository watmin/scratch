# CEK Migration — scope, surface, and sequencing (home first, then CEK)

Companion to `DESIGN.md`. That doc is the *idea* (metered/pausable compute) and
resolves *that* the evaluator is a recursive tree-walker + tail-call trampoline.
This doc is the *engineering plan* for the third tier — the actual move to a CEK
machine — and the order it should happen in relative to the namespace refactor.

User framing (2026-06-12):

> *"i was planning on gutting the src/*.rs files into
> src/<namespace>/<bunch-of-scoped-files>.rs … should i do the large refactor
> to get things well homed first then go to CEK, or do CEK then mass refactor
> to better maintainable names?"*

**Answer: home first, then CEK. Strongly.** The reasoning, grounded, below.

Status: **plan capture — to mull.** The trigger is "after the syntax flip
(arc 251) settles the form set."

---

## How awful is the CEK move? Major, but bounded.

Grounded against `wat-rs/src/runtime.rs` (not guessed):

| Metric | Value |
|---|---|
| `runtime.rs` total lines | **30,426** (flat monolith) |
| `eval_*` handler fns | **289** |
| Recursive eval call sites | **~355** |
| `dispatch_keyword_head` form arms | hundreds (central dispatch) |
| `*_tail` family (partial-CEK seed) | 4 |

Every one of those ~355 recursion points is a spot where "recurse into the
subform" must become "**push a K-frame, set C to the subform, loop.**" That's a
from-the-studs rebuild of the evaluation core — top-tier-biggest-arc territory.

**But two of the three CEK registers already exist as data:**
- **C** = `WatAST` (already data). **E** = `Environment`
  (`src/value/environment.rs:90` — already a reified struct). **Only K is new.**
  You invent one register and thread it; you don't rebuild the state model.

**The blast radius is contained. CEK does NOT touch:**
- **The macro engine** (arc 249) — expansion is a separate, *earlier* phase;
  eval runs on already-expanded forms.
- **`:rust::` / native builtins** — *leaves*: they receive already-evaluated
  `Value`s and return a `Value`; they never recurse into eval. The whole
  Rust-interop surface is invisible to CEK. (This is why the ~355 recursion
  sites are *wat special forms*, not the builtin library.)
- **The kernel / threading model** — per-eval-context CEK; each thread runs its
  own step loop; a blocking `recv` stays an OS-thread block. No scheduler
  rewrite forced (cooperative-yield-on-block is a *later* option K enables, not
  a prerequisite).
- **Parser, type checker (`src/check`), stdlib** — untouched.

**The real complexity metric isn't 355 — it's the number of distinct K-frame
*shapes*: ~15–25.** eval-args-then-apply (one shape, reused for every call +
every holon op), if (cond→branch), let/do (sequence), match (scrutinee→arms),
and/or (short-circuit), quasiquote (eval-unquotes), the constructors
(fields→build), try (install handler frame). The 355 sites collapse into a
couple dozen frame variants — enumerable and bounded, which is what makes it
tractable rather than open-ended.

## Where it needs to be expressed (the surface area)

- A new **`K` type** (the continuation-frame enum) — for v1 a Rust enum; for the
  *dream*, K is `WatAST` / EDN so the continuation is content-addressable,
  signable, portable across hosts.
- A **`step(C, E, K) → Transition`** loop replacing `eval_inner`
  (`runtime.rs:3279`); each `eval_*` special-form handler re-expressed as "which
  frames this form pushes."
- `dispatch_keyword_head` → the dispatch *inside* the step loop.
- **`apply_function`'s trampoline (`runtime.rs:17335`) merges into the step
  loop** — a tail call becomes "replace C, don't push." The existing `*_tail`
  family + `EvalSignal::TailCall` is the seed: **the tail-position analysis CEK
  needs is already done.**
- `EvalBreak` / `EvalSignal` → `OutOfFuel` / `Yield` become "halt the step loop,
  return `(C, E, K)`."
- `try` → a handler frame in K (cleaner than the current Rust-stack unwind).

## The sequencing decision — home first, then CEK

The two changes live on different axes:
- **Homing** = *where* code lives (modules, names). Mechanical,
  behavior-preserving, low-risk, **wardable incrementally**.
- **CEK** = *how* eval works (control-flow semantics). Semantic,
  behavior-changing, **highest-risk rewrite in the language's history.**

The doctrine: **never compound the hardest semantic change with the worst code
organization.** CEK-first means executing a from-the-studs control-flow rewrite
*while navigating a 30k-line flat file*, finding all 355 sites by grep, holding
the dispatch in your head, with no warded seams to catch drift. The hardest
thing in the worst place. Home-first does the safe restructure to *create the
clean surface*, then lands the risky rewrite against it.

### Why "but I'd be homing code I'm about to rewrite" dissolves

CEK rewrites only the **control-flow core** — the ~15–25 K-frame shapes. Call it
a few thousand lines. The *other* ~25k lines of `runtime.rs` are **leaves**
(builtin impls, holon ops, value operations, type-interplay helpers) — things
CEK *calls* but never restructures. So:

- Homing moves **all 30k** lines into homes; CEK touches the **bodies** of maybe
  10% of it.
- Even for that 10%, the **home/file placement survives** CEK — only the
  function *bodies* change shape (recursive → frame-producer). You re-home
  nothing; you rewrite bodies in place.

Home-first "wastes" almost nothing. CEK-first wastes the entire homing context
*during the hardest rewrite*. The asymmetry is the whole decision.

### The refinement — home *with CEK in mind*

When you gut the eval core, don't scatter it — carve it into a structure that
anticipates the step machine, so CEK becomes a *localized arc*:

```
src/eval/
  dispatch.rs   ← dispatch_keyword_head (which frame for which head)
  forms/        ← if / let / do / match / and / or / quasiquote / try
                  — each becomes a frame producer
  apply.rs      ← apply_function's trampoline → becomes the step loop
  value.rs      ← the leaves CEK calls but never restructures
  cont.rs       ← (empty for now) the K type lands here when CEK starts
```

Then CEK is "rewrite `forms/` + `apply.rs`, add `cont.rs`" — against warded,
navigable, settled modules, with the 25k lines of leaves already home and
untouched.

### Full sequence

1. **Finish the syntax flip (arc 251)** — settles the form set, so you don't
   home/rewrite forms you're about to delete, and `dispatch_keyword_head`
   reaches its final shape.
2. **Gut the flat `src/*.rs` into homes** — warded as you go (`vigilatum` per
   home); eval carved into `src/eval/` as above.
3. **CEK** — against the clean, settled, navigable surface.

### The four questions, all pointing one way

- **Obvious?** ✅ safe-before-risky; clean surface before hard rewrite.
- **Simple?** ✅ keeps mechanical (homing) and semantic (CEK) *separate* instead
  of entangled.
- **Honest?** ✅✅ home-first keeps the per-home warding that catches drift;
  CEK-in-the-monolith hides it.
- **Good UX (future-you)?** ✅ CEK lands in a codebase you can actually read.

### Honest caveat

A few `forms/` files *will* get their bodies churned by CEK after you home them
— real, but small (the ~15–25 shapes), and the home survives. That churn is
vastly cheaper than doing the rewrite blind in the monolith.

## What CEK buys beyond billing

Worth remembering when weighing the arc — it's not just for metered-eval's dream
tier:
- **Cleaner error handling** — `try` as an explicit handler frame in K, not a
  Rust-stack unwind.
- **Cooperative scheduling** — a blocking op can *yield* the continuation
  instead of blocking an OS thread (optional, later).
- **Time-travel debugging** — a reified `(C, E, K)` is a snapshot you can step
  *backward* through.
- **Content-addressed, portable continuations** — the metered-eval dream:
  pause anywhere, hash/sign/send the continuation, resume on another host.

## Cross-references

- `DESIGN.md` (this dir) — the metered-eval idea + the three-tier resolution;
  CEK is the third (dream) tier. v1 does **not** need this arc.
- `wat-rs/src/runtime.rs` — `eval_inner` (`:3279`), `apply_function` (`:17335`),
  `EvalBreak` / `EvalSignal::TailCall` (the escape channel CEK extends).
- `wat-rs/src/value/environment.rs:90` — `Environment`, the E register (already
  reified).
- arc 251 (the syntax flip + the great migration / home-lifting) — the
  prerequisite that settles the form set and pays down the monolith.

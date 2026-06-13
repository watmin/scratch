# Metered Evaluation — gas for a Lisp, pay-to-continue compute on the wat network

A companion to `scratch/WAT-NETWORK.md`. That doc captured the network's
*identity / trust / transport* layer. This captures its **economic** layer:
evaluation as a metered, billable, pausable resource. The bat-shit idea, and
why the substrate is one of the few places it actually composes instead of
being bolted on.

User direction (2026-06-12, immediately after "IPC via unix domain sockets is
wrapping up — paving the road for programs on remote hosts with the same
surface area as threads and processes"):

> *"so - the bat shit idea - can we have programs run with a cost?.. like... in
> the world of agentic x402... can we have hosts who operate as remote compute
> and you're billed on evaluation.. like... down to statements... the program
> pauses if its burned its compute and you need to send more tokens to
> continue?.."*

Status: **idea capture — to mull.** Not a spec, not a roadmap. The author needs
to sit with it.

---

## The one-line version

Hosts on the wat network sell **evaluation**. You send a signed program with a
fuel budget and a payment; the host meters every form it evaluates against a
cost table; when the fuel runs out it **pauses**, hands you back a
content-addressed continuation and a quote, and you decide — from the partial
result — whether to pay to resume. Gas for a Lisp, with x402 as the payment
rail and the *continuation* as the receipt.

## Why this is a wat idea and not a generic "add metering" idea

The mechanism (gas / fuel) is well-precedented — EVM gas, Wasmtime fuel, eBPF
instruction limits. What's specific to wat:

1. **One eval chokepoint.** wat is a homoiconic tree-walker: a program is
   HolonAST (data), and every form passes through a single evaluator. You
   instrument *that*, not the program. "Bill down to statements" is the EASY
   part — it's where the interpreter already is.

2. **A typed, pluggable, per-form cost model — itself data.** Because forms are
   typed, cost can be a function of *what the form is*: `+` costs 1, a `bind`
   over a 10k-dim vector costs more, a `recv` costs for I/O, a remote call costs
   *the remote node's own quote*. EVM gas is a fixed opcode table; wat's cost
   table can be data, typed, and composable.

3. **The continuation is a wat value.** This is the load-bearing payoff. To
   pause mid-program and resume later, the evaluator must reify its
   continuation — and in a homoiconic substrate that continuation is *data*.
   Data is content-addressable (digest), signable, and **transferable**. So:
   - The pause **is** a content-addressed receipt. "Out of fuel → here's
     continuation-digest X; pay to resume X."
   - A paused program is **not trapped on the node that ran it.** Continuations
     are portable; you could resume on a different host.

## How it collapses into the existing wat-network primitives

Every piece this idea needs is already a load-bearing primitive in
`WAT-NETWORK.md`:

| This idea needs… | …is already this primitive |
|---|---|
| Authorize *and meter* a program | **signed eval** (same envelope, plus a fuel budget) |
| Identity for a program / a paused program | **digest** (content-addressed AST *and* content-addressed continuation) |
| Bill a who, not an IP | **mTLS membership** — the meter charges a cryptographic identity |
| Trustworthy bills | **deterministic eval** (same source → same canonical hash — already true for the total-pure macro engine) |
| Pay-to-continue rail | **x402** over the signed wat-network connection |

## The x402 dance, concretely

```
agent → node:  signed{ run: <digest>, input, fuel: 5000, pay: <x402 receipt> }
node:          evals, metering each form against the cost table
               ... fuel → 0 mid-program ...
node → agent:  402{ out-of-fuel,
                    continuation: <digest>,
                    consumed: 5000,
                    partial:  <any output produced so far>,
                    quote:    <price to resume> }
agent:         reads `partial`, DECIDES it's worth continuing, pays
agent → node:  signed{ resume: <continuation-digest>, fuel: 5000, pay: <receipt> }
node:          resumes from the checkpoint, keeps metering
               ... completes ...
node → agent:  200{ result, total-consumed: 8200, signed-receipt }
```

The `partial` field is the entire agentic UX: an agent budgets its own compute,
the program **halts**, the agent looks at what came back, and *then* decides to
top up. Pause → inspect → decide → continue. The human/agent is in the loop at
the granularity of their wallet.

## Two things that fall out for free

1. **Verifiable billing.** Eval is deterministic, so `(program, input) →
   fuel-consumed` is reproducible. The client can re-derive the bill; a third
   party can audit it. **The host cannot lie about what it charged** — the bill
   is a function of public data, not the host's word. This is the answer to
   "why would I trust the meter?": you don't have to.

2. **A memoization market.** Content-addressed program + deterministic eval ⇒
   `(digest, input) → result` is *also* content-addressable. A node can cache
   results and sell a **cache hit instead of a re-run.** Compute and its answers
   both get prices; a hot result might cost a lookup, not an evaluation.

## A new axis of flow control

`WAT-NETWORK.md` frames backpressure as structural rhythm — bounded channels +
blocking mean the system can't run *faster* than its slowest part. Metering adds
an orthogonal limiter: the system can't run *further* than its budget allows.
Same shape (a resource ceiling enforced by the substrate, not by convention),
new axis — **economic, not throughput.** "The program halts because the budget
is spent" is the dual of "the producer blocks because the consumer is full."

## The crux question — RESOLVED (2026-06-12): recursive core + tail-call trampoline

Resumability gates everything, so the gating question was: **is wat's evaluator
recursive Rust, or already a step / CEK machine?** Grounded against
`wat-rs/src/runtime.rs`, the answer is **both** — a recursive tree-walker
wrapped by an explicit tail-call trampoline.

- **The core is recursive.** `eval` (`runtime.rs:3478`) → `eval_inner` (`:3279`)
  → `eval_list` → the `eval_*` family, which call `eval_inner` on subforms
  recursively. For a non-tail position — `(+ (expensive) (other))` — the
  continuation lives on the **native Rust stack**; you can't reify it.
- **But function calls don't recurse — they trampoline.** `apply_function`
  (`:17335`) is a literal `loop {` (`:17366`) whose between-hop state is fully
  reified as owned locals:
  ```rust
  let mut cur_func = func;   // the continuation between function hops…
  let mut cur_args = args;   // …is plain data, not stack
  let mut cur_span = call_span;
  loop { ... }
  ```
  Tail positions emit `Err(EvalBreak::Signal(EvalSignal::TailCall { func, args, call_span }))`
  (`emit_tail_call`, `:3032`); the trampoline catches it (`:17449`), re-seats
  `cur_func`/`cur_args`, and loops. (This is how wat gets unbounded
  recursion/loops without growing the stack.)

So the binary resolves into **three tiers**:

| Granularity | Cost | Why |
|---|---|---|
| **Meter "down to statements"** | **weeks** | One chokepoint (`eval_inner`). A fuel counter, decremented per form, returns a new `EvalBreak::OutOfFuel` at zero. `EvalBreak` is already the escape channel. |
| **Pause/resume at function-call / loop-iteration boundary** | **small — no rewrite** | The trampoline is already a step machine whose between-hop continuation is `(cur_func, cur_args, cur_span, env)` — reified data. An `OutOfFuel` signal the trampoline catches *like* `TailCall`, but freezes and returns the state instead of looping. Same machinery, one more arm. |
| **Pause/resume mid-expression (continuation-as-data everywhere)** | **the arc** | The recursive core needs a CEK/SECD rewrite so the continuation is reified for *every* position, not just tail. The gorgeous content-addressed-portable-continuation version. |

**The pleasant surprise: the agentic UX needs the middle tier, not the arc.**
"Run with a budget, halt between function calls / loop iterations, hand back a
continuation, inspect the partial, pay to resume" — the trampoline already
reifies exactly the seam you'd pause on. The full CEK rewrite is only needed to
pause *inside* a single expensive form, or to make continuations
content-addressable and portable across hosts (the dream, not v1).

### What CEK is (for the future-mull)

A **CEK machine** (Felleisen & Friedman, 1986) runs a functional language as a
loop over three registers: **C**ontrol (the expression), **E**nvironment (the
bindings), **K**ontinuation (*"what to do with the result," as an explicit data
structure* — not the host call stack). K is the whole point: a recursive walker
leaves the continuation implicit on the Rust stack (un-holdable); CEK makes it a
**value**. At any step `(C, E, K)` fully describes where the program is — so
**pause = stop and keep the triple**, and because K is data, in a homoiconic
substrate it's wat AST / EDN → hashable (digest), signable, sendable, resumable
on another host. CEK is *why* "the continuation is a content-addressed receipt"
is achievable at all. wat's current trampoline is a **baby CEK for the
function-call layer only**; the rewrite extends explicit-continuation treatment
to every position. (SECD — Landin, 1964 — is the chunkier ancestor.)

And the lineage closes a loop: **Dan Friedman — Indiana, *The Little Schemer* —
co-authored CEK.** The machine that makes pausable, content-addressed,
pay-to-continue compute work was written by the person whose book taught this
way of thinking. The strange loop the realizations already clocked, cashed out
in the substrate.

## The other honest hard parts (four-questions, no hype)

- **Metering overhead** — a per-node fuel counter taxes every eval. Real,
  survivable (EVM/WASM live with it), and the cost-check compiles down.
- **Trust without determinism would be the soft spot** — but determinism is
  already there, so the bill is auditable by construction. Honest ✅.

## Where it sits in the bigger picture

This turns the wat network from a *trust + transport* substrate into a **compute
market**: nodes sell evaluation; programs and continuations and results are all
content-addressed (cacheable, transferable, priceable); identity and billing are
cryptographic. It's the lineage of EVM gas × Urbit-style sovereign nodes × a
typed Lisp × x402 — but none of those is all of these at once. "Metered,
signed, typed, *pausable* remote evaluation" hasn't existed in this exact shape.

And it sharpens the conj thesis (`Homoiconicity Was Always for Machines`) one
turn further: code-as-data is what makes compute **meterable, pausable,
billable, and verifiable** — because the program *and its continuation* are
values you can hash, sign, send, and price. **Homoiconicity was always for
machines that pay each other.**

## What this is NOT

- **Not a spec.** No fuel-unit accounting, no cost-table values, no x402
  message schema, no continuation-serialization format. Those are arcs of their
  own when their time comes.
- **Not committed.** It rides on resumable evaluation, which (resolved above) is
  a three-tier reality: metering is weeks, coarse function-boundary pause/resume
  is a natural extension of the existing trampoline, and only the
  content-addressed-portable-continuation dream needs the CEK arc. v1 doesn't
  need the arc.
- **Not exhaustive.** Open threads not covered: fuel pricing / unit economics,
  refunds on early completion, partial-result confidentiality, double-spend on
  resume tokens, who arbitrates a disputed bill, delegation (paying for a
  program that pays a third node), and metering of *I/O* vs *pure compute*.

## Cross-references

- `scratch/WAT-NETWORK.md` — the identity/trust/transport layer this is the
  economic dual of (signed eval, digest, mTLS, the four-tier model, RemoteProgram).
- The UDS-IPC work in flight (2026-06) — the transport rung that makes
  thread / process / remote one surface area, which is the layer metered eval
  rides on.
- `wat-rs/src/runtime.rs` — the evaluator (crux **resolved** above):
  `eval_inner` (`:3279`, the recursive core), `apply_function` (`:17335`, the
  tail-call trampoline = a baby CEK for the function-call layer), `EvalBreak` /
  `EvalSignal::TailCall` (the escape channel a fuel/pause signal would extend).
- The total-pure macro engine (arc 249) — the determinism precedent that makes
  verifiable billing real (same source → same canonical hash).
- x402 / agentic-payments — the external rail; the substrate provides the
  signed, typed, content-addressed half.

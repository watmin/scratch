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

## The honest hard part (four-questions, no hype)

- **Resumability is the real engineering, and it gates everything.** The crux
  question: **is wat's runtime evaluator recursive Rust today, or already a
  step / CEK machine?**
  - If it's a recursive tree-walker, it cannot pause mid-form, and the
    load-bearing arc is rewriting eval so the continuation is reified data
    (CEK/SECD-style). That's the gorgeous version — continuation-as-value — but
    it's not small.
  - A **coarse** version is much cheaper and still useful: checkpoint only at
    top-level-form or loop-iteration boundaries. You bill per statement and
    pause *between* statements, not inside one. Probably the right first cut.
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
- **Not committed.** It rides on a resumable evaluator that may or may not exist
  yet; the first question above decides whether this is "instrument the eval
  loop" (weeks) or "reify the continuation" (an arc).
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
- `wat-rs` runtime evaluator — **read it to answer the crux question**
  (recursive vs step/CEK), which decides the shape of the work.
- The total-pure macro engine (arc 249) — the determinism precedent that makes
  verifiable billing real (same source → same canonical hash).
- x402 / agentic-payments — the external rail; the substrate provides the
  signed, typed, content-addressed half.

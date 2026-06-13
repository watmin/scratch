# Prior art — is metered, pausable, code-travels-to-compute remote eval already a thing?

Companion to `DESIGN.md`. A cited survey (deep-research pass, 2026-06-12: 5 search
angles, 22 sources, 95 claims extracted, 25 adversarially verified at 2-of-3,
23 confirmed / 2 killed). Goal: ground the "has anyone built this?" question and
sharpen what's genuinely novel — for the plan and the conj talk.

**Bottom line (verified): no single system unifies the five ingredients** —
per-step metered eval · pause-on-budget-then-pay-to-resume · content-addressed
*portable* continuations · a cryptographic-identity compute market ·
code-travels-to-compute. Each existing system nails three or four; the keystone
that appears in **no** surveyed source is *the continuation captured at
exhaustion as a content-addressed, signable **receipt** a different node can be
**paid** to resume.* (Caveat: the survey is incomplete — see "Honest gaps.")

---

## The closest single ancestor — Telescript / General Magic (1990s)

Spookily close, and it shipped. Verified (3-0):
- **Budget-bearing mobile code.** Agents carry an *allowance* in **teleclicks**
  with a real teleclick→money exchange rate (the docs literally describe "a
  50-cent agent"), enforced by a **permit** (charges / age / extent).
- **Code travels to compute.** Telescript is *"remote programming"*: the network
  carries objects — *data AND procedures* — that the receiving engine executes.
  Not RPC; the code goes to the node.
- **True suspend / serialize / migrate / resume.** The `go` operation suspends
  execution, **saves the program counter**, serializes the agent to a "wireline
  encoding" / "bag of bits," transports it, and **resumes at the instruction
  after `go`** on the destination engine. A real, portable, resumable
  continuation — in 1994.

**The decisive gap (verified 3-0):** budget exhaustion is **destroy, not
pause-for-top-up.** Whitepaper: *"If the agent exceeds any of its quantitative
limits, the engine destroys the agent unceremoniously. No grace period is
extended."* Reference: *"the engine terminates the current sponsor … the current
process isn't restarted."* Telescript had *voluntary* dormancy as a lifecycle,
but **no pay-to-continue path** — and it predates content-addressing and
cryptographic signing entirely.
Sources: bitsavers Telescript Language Reference (Oct 1995, primary);
datarover.com whitepaper; Wooldridge KER95 survey; The Economist (1994).

## Finest metering, but ABORT not pause — EVM and Agoric

- **EVM gas** — per-opcode metering (cost summed over each executed
  instruction), but **abort/revert**: out-of-gas and `REVERT` (EIP-140) *"stop
  execution and roll back"* — **"no provision for pausing and resuming."**
  Execution is **replicated by every validator for consensus**, not sold as
  marketplace compute. And a talk-worthy nuance (peer-reviewed, "Broken Metre,"
  NDSS/EuroS&P 2020): *"very little correlation between execution cost and the
  utilised resources"* — **gas tracks storage, not CPU.** A homoiconic
  interpreter metering actual eval steps is arguably a *more honest* meter than
  gas. (Verified 3-0.) Sources: arxiv 1909.07220; EIP-140.
- **Agoric transform-metering** — per-**statement** (AST-level: wraps every
  function body in try/finally + a stack-meter check; prefixes every
  loop/catch/finally with a compute-meter decrement). But **meter-and-abort**:
  *"an exceeded meter … throws a RangeError all the way to the top of the
  evaluation"* — uncatchable, no resume. **Correction to my earlier from-memory
  take:** Agoric is the closest on *metering a real high-level language*, but it
  **aborts, it does not pause**; the broader "Agoric VM supports
  checkpoint/restart" claim was **refuted 0-3**. (Prod later moved to xsnap/XS
  with heap snapshots — a pause capability *may* exist but was not verified.)
  Source: `@agoric/transform-metering` (npm, primary; self-described "loose but
  deterministic best approximation").

## Closest on PAUSE, but the pause isn't portable — Wasmtime

Wasmtime is the mainstream engine nearest the pause ingredient (verified 3-0):
deterministic **fuel** + lower-overhead **epoch** interruption, and on
interruption it can **async-yield** (*"pauses … yields control back to the host,
and lets the host decide whether to resume"*) rather than only trap. Two real
limits:
- The yield is **same-store / same-stack cooperative timeslicing** — the paused
  state lives on the **host async stack**, *not* a serializable, portable
  continuation that can move to another node.
- The "refuel an exhausted store via `set_fuel` to pay-and-continue" path was
  **refuted (1-2)** — the public docs don't establish a built-in
  add-fuel-to-resume.

So: Wasmtime pauses, but not *into a portable, payable continuation.*
Sources: docs.wasmtime.dev (interrupting / deterministic-execution).

## The content-addressed, code-travels piece — Unison

The half the metering systems lack (verified 3-0). Each definition is
identified by **the hash of its AST** ("names are just separately stored
metadata that don't affect the hash"). Code travels by shipping the bytecode
tree: the recipient *"inspects the bytecode for any hashes it's missing … the
sender syncs them on the fly,"* then caches. And `Remote.transfer : Node ->
Remote Unit` — *"transfer control of remainder of computation to target node."*
**Gap:** Unison does **not** meter per step or charge/pause on a budget;
`transfer` is program-directed migration, not a metered fuel-pause. (The cited
`transfer` is from the ~2017 RFC; the runtime has since evolved — Unison Cloud,
`Remote.fork`.) Sources: unison-lang.org/docs/the-big-idea; distributed-
programming RFC (GitHub).

## Closest-on-each-ingredient (verified)

| Ingredient | Closest existing | Its gap vs the target |
|---|---|---|
| Per-step metering | EVM gas (per-opcode), Agoric (per-statement) | abort/revert, not pause |
| Deterministic metered **halt** | Wasmtime fuel | traps; no built-in resume |
| **Pause** then resume | Wasmtime async-yield | host-stack only; not portable/payable |
| Code as **addressable data** + travels | Unison (AST-hash), Telescript (RP) | Unison doesn't meter; Telescript pre-crypto |
| Serializable **migrate/resume** continuation | Telescript `go` / wireline | exhaustion = destroy, no pay-to-resume |
| Budget-bearing mobile code, monetary unit | Telescript teleclicks/permits | destroy-on-exhaust; no content-addressing |

## What's genuinely novel (the unification)

No verified source describes the combination: a **metered, fuel-paused
continuation captured as a content-addressed, signable receipt that a
possibly-different node can be paid to resume** — i.e. Telescript's
serialize-and-migrate + Unison's AST-hash identity/portability + Wasmtime-style
deterministic fuel-pause + EVM/x402-style cryptographic-identity payment +
metered eval of a **homoiconic typed** language, in one substrate. The pieces
all exist; the assembly does not.

**The cloud inversion, now grounded:**

| Cloud / serverless | This model |
|---|---|
| Rent **capacity**, billed by **time held** | Buy **evaluation**, billed by **work done** (per metered step) |
| **Deploy** your code to a box you own | **Code travels** to whatever idle node is free |
| Runs to completion (or you keep paying idle) | **Pausable** — and the paused continuation *is* the billable/payable artifact |

## Honest gaps (the novelty claim is PROVISIONAL)

The verification budget hit 25 claims and **did not** verify four branches the
research asked about, so "no unifying system exists" rests partly on
absence-of-evidence in an **incomplete** survey:
- **Decentralized compute markets** (Golem, Akash, iExec, io.net, Render,
  Gensyn, Bittensor) — billing units not verified (sources fetched for Akash
  bid-pricing + Gensyn litepaper suggest per-resource/per-time, not per-step,
  but unconfirmed).
- **WASM edge billing** (Cloudflare Workers CPU-time, Fastly Compute@Edge,
  Fermyon Spin) — not verified (Cloudflare pricing source fetched).
- **Serializable continuations** — *promising signal, not yet folded in:* a
  fetched primary source (Racket "two-state solution: native **and**
  serializable continuations") indicates a paused computation **can** be a
  serializable value in Racket's web-server, and GraalVM Espresso ships
  continuations. This is the closest precedent to "the continuation is a value
  you can store/send" and deserves a verified pass.
- **x402 / agentic M2M payments** — fast-moving (2024–2025), under-covered;
  a fetched primary source confirms **Coinbase x402 + Amazon Bedrock AgentCore
  Payments** is real (agentic payment rails shipping). Worth a dedicated pass.

**Refuted claims (kept for honesty):** (1) Wasmtime `set_fuel`
refuel-and-resume as a built-in pay-to-continue — refuted 1-2. (2) Agoric VM
checkpoint/restart pause-resume — refuted 0-3.

## For the conj talk

- The honest, strong line: *"Each piece of this has been built — Telescript
  carried a budget and migrated a live computation in 1994; Unison content-
  addresses code and ships it to a node; EVM meters per opcode; Wasmtime can
  pause. Nobody has assembled them, and the keystone — a paused computation that
  is itself a signed, content-addressed receipt you pay to resume — exists
  nowhere I can find."*
- The gas zinger: gas doesn't actually meter compute (it tracks storage); a
  homoiconic interpreter metering real eval steps is a *truer* meter.
- The lineage: it's Telescript's dream, finally on a substrate (homoiconic,
  typed, crypto-identextended, content-addressed) that can carry it.

## Cross-references

- `DESIGN.md` — the idea + three-tier resolution.
- `CEK-MIGRATION.md` — the engineering (the CEK arc that makes the portable
  continuation real).
- Sources (load-bearing): bitsavers Telescript Language Reference (Oct 1995);
  arxiv 1909.07220 ("Broken Metre"); EIP-140; `@agoric/transform-metering`;
  docs.wasmtime.dev (interrupting / deterministic); unison-lang.org +
  distributed-programming RFC; racket two-state-continuations blog;
  coinbase x402 + Bedrock AgentCore (agentic payments).

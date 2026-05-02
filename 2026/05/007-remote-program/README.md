# RemoteProgram — typed remote program calls in wat

User direction (2026-05-03):

> *"alright.. RemoteProgram... there's tiers to this...
>
> unix domain
> localhost http
> remote https
> remote mtls
>
> -- that's it - we don't allow clear text over the network
>
> i think we can model this right?..."*

> *"the first one... edn is the transport protocol..."*

This is the LAST item from random-notes.txt's four-item list
(linter / formatter / coverage / RemoteProgram). Application-
tier rather than foundation-tier — it sits on top of the
quartet (003-wat-fmt + 004-wat-lint + 005-wat-cov + 006-wat-doc)
as a real consumer crate.

Design is in-progress; this arc captures what's locked and
enumerates the open questions for one-at-a-time work.

---

## What RemoteProgram is

A typed remote-program abstraction. `RemoteProgram<I, O>` is a
typed remote function: `I → Result<O, RemoteError>`. The
transport is configuration; the type contract is the same
across tiers.

Four tiers, ordered by trust + scope:

| Tier | Transport | Auth | Network? | Encryption |
|---|---|---|---|---|
| 1 | Unix domain socket | Process boundary (file perms) | No | N/A (local) |
| 2 | Localhost HTTP | None / local trust | No (loopback) | N/A (local) |
| 3 | Remote HTTPS | Server cert verify | Yes | TLS |
| 4 | Remote mTLS | Mutual cert verify | Yes | TLS + client auth |

**Conspicuously absent: remote HTTP.** The user's invariant:
*"we don't allow clear text over the network."* This is enforced
at the type system level — the type `:RemoteProgram` has no
constructor for "HTTP over network." Honest by construction.

## What's locked

**Wire format (Q3):** EDN over the wire. Length-prefixed EDN
frames; round-trips through `wat::edn::write` / `wat::edn::read`.
Same shape as wat-edn already provides for other consumers.
JSON via wat-edn for heterogeneous-server scenarios is
implied but secondary.

User direction:

> *"the first one... edn is the transport protocol..."*

## What's open (worked one-at-a-time)

Seven design questions captured in DESIGN.md, ordered for
sequential work:

- **Q8** — Who is the first consumer? (grounds everything)
- **Q1** — Transport-as-config vs transport-as-type
- **Q2** — Connection model: persistent or per-call
- **Q4** — Server side scope (do we ship `serve` in v1?)
- **Q5** — Streaming: `:RemoteStream<I, O>` sibling vs included
- **Q6** — Error taxonomy: shape of `:RemoteError`
- **Q7** — Hermetic / sandbox interaction

Each question is a section in DESIGN.md ready to receive its
answer.

## Where it lives

**Single self-contained crate (proposed):**
`wat-rs/crates/wat-remote/` — same arc-013 pattern as wat-fmt /
wat-lint / wat-cov / wat-doc.

The crate name `wat-remote` is provisional — Tier 1 (Unix
domain) is technically local, so the name is slightly
misleading. Possible alternatives: `wat-rpc`, `wat-call`. To
be confirmed.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, status (in-progress design), one-at-a-time work plan |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | The four-tier model + locked decisions (Q3 EDN transport) + seven open questions ready to receive answers |
| `SLICE-PLAN.md` | (Will be created once the design firms up enough to slice) |

## Conventions inherited

From the foundation-tier arcs (003-006):

- **The four questions as design compass** (Obvious / Simple /
  Honest / Good UX) — applied to every choice
- **Arc-013 self-contained crate pattern**
- **Developer-first output** — EDN canonical; JSON via
  wat-edn for the wider ecosystem
- **The "honest by construction" principle** — invariants
  enforced by type system rather than convention (see the
  no-clear-text-over-network type-level enforcement above)

## Cross-references

- **wat-fmt** at `scratch/2026/05/003-wat-fmt/` — formatting
  applies to wat code defining RemoteProgram surfaces
- **wat-lint** at `scratch/2026/05/004-wat-lint/` — likely
  lint rules around remote-program declarations
- **wat-cov** at `scratch/2026/05/005-wat-cov/` — surface-
  attribution for remote calls (the call site counts; the
  remote execution doesn't)
- **wat-doc** at `scratch/2026/05/006-wat-doc/` — RemoteProgram
  surfaces are documented per the docstring convention
- **arc 058-035** (fork-substrate) — kernel pipe primitives
  potentially useful for Unix-domain-socket implementation
- **`:wat::kernel::Channel<T>`** — existing duplex IPC
  abstraction; RemoteProgram is the typed-program layer above

## Status

- **Captured:** 2026-05-03
- **Architecture:** in-progress design; four-tier model locked;
  EDN transport locked; seven open questions enumerated
- **Slice plan:** not yet sized (waiting for design to firm up)
- **Bar to graduate to a real wat-rs arc:** all seven open
  questions answered + slice plan sized + user signals start

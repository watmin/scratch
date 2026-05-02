# The wat network — meta-vision

A top-level companion to the per-arc design files (003-008,
plus arc 007). The arcs are pieces; this document is the whole
they're pieces of.

User direction (2026-05-03, immediately after the
"RPC-as-data" reframe of arc 007):

> *"do you understany why i want mtls support now? the query
> connection is a signed entity.. its credential bearing.. the
> caller must self identify and the receiver must verify.. you
> cannot run your program on the remote host if the remote host
> won't allow it.. and we can have signed queries too - i put
> digest and signed eval forms in.... i've been building
> toward this... the wat network"*

What the user has been building toward, articulated.

---

## What the wat network IS

A network of mutually-authenticating wat-vms where:

1. **Each node has cryptographic identity** — cert / keypair.
   Not network-position-based; not magic-cookie; not bearer-
   token. Real cryptographic identity.

2. **Connections are mTLS** — both ends verify each other.
   You can't connect to a node that won't accept your cert; the
   receiver knows exactly who's calling.

3. **Queries / programs sent for execution are SIGNED** — the
   receiver verifies the signature before evaluating. The
   substrate ships `digest` and `signed eval` forms for exactly
   this discipline.

4. **Authorization is cryptographic, not network-positional**
   — "who signed this; do I trust them; do they have permission
   to ask this" — not "did this packet arrive on the right
   interface."

5. **Programs are content-addressed via digest** — immutable;
   identifiable; cacheable; versionable. "Run program with
   digest X" is a network primitive, not a string.

This is **a distributed capability-and-computation substrate.**
Not just RPC — a peer-to-peer network where computation,
identity, and authorization are first-class wat-vm concepts.

## The three load-bearing primitives

The network rests on three substrate primitives, each of which
the user has either shipped or is shipping:

### Primitive 1 — Cryptographic identity (mTLS-based)

Every node has a cert. Connections are mutually verified. The
network's membership protocol is the cert chain — you're a
member if your cert is signed by a trusted authority (or peer-
trusted via a web-of-trust pattern).

mTLS isn't a deployment-time security knob. It's the network's
**membership protocol.** Without it, there's no wat network;
there's just isolated wat-vms that happen to communicate.

### Primitive 2 — Content-addressed programs (digest forms)

Programs are HolonAST. AST is data. Data has a content hash.
That hash is the program's IDENTITY.

Implications:
- Two programs with the same code have the same digest
- "Send me the program with digest X" is unambiguous
- Caching is trivial (memoize by digest)
- Versioning is trivial (different code → different digest)
- Provenance is verifiable (digest in a signed envelope =
  cryptographic claim about what code is being referenced)

### Primitive 3 — Verifiable execution (signed eval forms)

A program (HolonAST) plus a signature plus a public key gives
you a verifiable claim: "this exact program was authorized by
this exact identity." The receiver checks:
- Does the signature verify against the claimed public key?
- Does the public key correspond to a trusted identity?
- Is that identity authorized to request this kind of work?

Only signed-by-trusted-party code is evaluated. **Code is data
is signed-data.** The wat-network's authorization model is
built on this trichotomy.

These three primitives compose. Together they enable
distributed computation with cryptographic provenance —
something you can't do with bare TCP, bare HTTP, or even bare
RPC frameworks.

## The architecture

```
┌─────────────────────────────┐         ┌─────────────────────────────┐
│   Node A (wat-vm)           │         │   Node B (wat-vm)           │
│                             │ mTLS    │                             │
│   identity: cert-A          ├────────►│   identity: cert-B          │
│                             │         │                             │
│   trusts: cert-B, cert-C... │         │   trusts: cert-A, cert-D... │
└─────────────────────────────┘         └─────────────────────────────┘

Messages between nodes:
  - Signed query  : "evaluate this AST (signed by cert-A)"
  - Signed response : "result for that query (signed by cert-B)"
  - All wrapped in EDN length-prefixed frames
  - Demuxed via Q-channel (Ok / Err discriminator)
  - Programs identifiable by digest (cacheable across the network)
```

Nodes don't share fate; they share trust relationships. A
single cert revocation removes a node from the trust set
without affecting other nodes. The network is robust to
individual node compromise because identity is per-node.

## The four-tier model in this light

The arc 007 four-tier model gets reframed: tiers correspond to
where the capability lives RELATIVE TO the wat network, not to
"security level."

| Tier | Capability location | Network membership? |
|---|---|---|
| **Tier 1 (Unix domain)** | Same node; intra-process / sidecar | N/A — local |
| **Tier 2 (localhost HTTP)** | Same host; another process | Edge case; rare |
| **Tier 3 (HTTPS)** | Outside the wat network — SaaS / managed services / LLM APIs | NO — out-of-network |
| **Tier 4 (mTLS)** | **Inside the wat network — peer wat-vms** | YES — load-bearing |

**Tier 4 is where the wat network lives.** Tier 3 is for
reaching out-of-network. Tiers 1-2 are local. The hierarchy is
about TRUST and LOCATION, not about "how secure do I want to
be."

This sharpens RemoteProgram's design (arc 007):

- **In-network mode** (Tiers 1, 4): signed queries, mTLS auth,
  digest-addressable programs, full wat-network semantics
- **Out-of-network mode** (Tier 3): bridging to non-wat
  services; auth via tokens / server cert; the typed-capability-
  bridge from the RPC-as-data reframe

Both modes use the same wire protocol, same wat-vm, same
RemoteProgram surface. The DIFFERENCE is whether the other
side speaks wat-network or just speaks the wire.

## What the existing arcs are pieces of

The arcs sketched in this scratch (003-008 + 007) are
substrate components for the wat network:

| Arc | Role in the wat network |
|---|---|
| **003-wat-fmt** | Format the AST consistently across nodes (digests are stable) |
| **004-wat-lint** | Verify AST quality before nodes accept programs from peers |
| **005-wat-cov** | Measure what executed when a node ran a peer's signed program |
| **006-wat-doc** | Document the typed contracts nodes expose to each other |
| **007-remote-program** | The substrate layer for nodes calling each other |
| **008-kwarg-macros** | Ergonomic API for typed cross-node calls |

Plus existing wat-rs work that's already shipped or shipping:
- **arc 058 (wat language spec)** — the typed substrate every
  node runs
- **digest forms** — content-addressing for programs
- **signed eval forms** — verifiable execution
- **mTLS support** — the network's membership protocol

The toolkit quartet (003-006) lets each node be QUALITY (formatted,
linted, measured, documented) consistently. The communication
arcs (007, 008) let nodes TALK in typed contracts. The
substrate primitives (digest, signed eval, mTLS) make the
network TRUSTWORTHY.

**Each arc is a piece. The wat network is the whole.**

## Comparison to other distributed substrates

The wat network resembles existing systems but differs in
load-bearing ways:

### Versus Erlang's distributed nodes

Erlang has distributed nodes that talk to each other via the
BEAM's distribution protocol. They authenticate via shared
"magic cookies" (a string that all nodes in the cluster know).

- ✅ Erlang: cluster-level identity; lightweight setup
- ❌ Erlang: not cryptographic; magic cookie can leak; no per-
  node identity; no per-message provenance

The wat network: cryptographic per-node identity; per-message
signatures; no shared secret; revocation via cert distrust
without affecting other nodes.

### Versus Urbit

Urbit is a peer-to-peer network where each node has a
cryptographic identity (~point) and communications are
authenticated.

- ✅ Urbit: per-node crypto identity; peer-to-peer; sovereign
  computing
- ❌ Urbit: opaque type system; bespoke language (Hoon); steep
  learning curve; tight coupling between language and network

The wat network: typed via wat's type system (extends across
the wire per Q-channel); language is wat (Lisp-shaped); type
discipline is the same locally and remotely.

### Versus capability-based distributed OSes (KeyKOS, EROS, Genode)

The capability-based OS thread has historically modeled
distributed computation as cryptographically-authenticated
capabilities: holding a capability means you can do a thing;
capabilities are unforgeable; revocable.

- ✅ Capability OSes: cryptographic authorization; principle
  of least privilege; revocable
- ❌ Capability OSes: never broke through to mainstream; no
  ergonomic typed surface; not built on a Lisp

The wat network: cryptographic authorization (signed eval);
typed surface (`Program<I, O>`); homoiconic (programs are
data); ergonomic (kwarg macros); buildable today (the
substrate is here).

The wat network is what would happen if you took
capability-based distributed-OS thinking + Lisp + modern
crypto + typed effects and built the substrate end-to-end. It
hasn't existed before in this exact shape.

## Per the four questions

- **Obvious?** ✅✅ — once articulated, the architecture's
  purpose is unmistakable; mTLS is membership; signed-eval is
  authorization; digest is identity
- **Simple?** ✅ — three primitives (mTLS / digest / signed-
  eval) compose into a complete distributed-capability
  substrate
- **Honest?** ✅✅✅ — the trust model is cryptographic, not
  positional; nothing about the network can be faked; identity,
  integrity, and authorization are structural; you cannot
  participate without an identity; you cannot send a query
  without signing it; you cannot reference a program except
  by its digest
- **Good UX?** ✅✅ — wat code talks to other wat nodes via
  the same `Program<I, O>` surface; the cryptography is in the
  substrate, not in user code

**Fourth triple-checkmark of this design session** (after auto-
kwargs, Q-channel, and arguably the four-tier model). Same
pattern as the others: the constraint lives in the type system
and cryptographic primitives, not in convention. You cannot
accidentally make an unauthenticated wat-network call because
the substrate doesn't expose one.

The repeated triple-checkmarks aren't graded leniently — they
require structural impossibility, not just "we tried." When
the wat substrate honors honesty by construction across so
many primitives, the result is a network where trust is
intrinsic, not bolted on.

## What this means for the future BOOK chapter

The chapter that emerges from this work — when arc 109 wraps
and the user begins writing — will have to articulate three
levels:

1. **The substrate** — wat-vm, the type system, homoiconicity,
   the kernel primitives, the four-questions discipline
2. **The toolkit** — fmt / lint / cov / doc / RemoteProgram /
   kwarg-macros (the per-arc work)
3. **The wat network** — the meta-vision the toolkit
   serves; the distributed capability substrate that becomes
   possible when all the pieces compose

Without (3), the work reads as "look what we built" — a list
of crates. With (3), the work reads as "look what becomes
possible when a recognition finally meets the substrate that
can host it" — the chapter title we're in service of.

The wat network is what the user has been building toward
through years of substrate work. The latin-in-wat
recognition (morphology over position), the OG wat era
(typed Lisp surface for LLM communication), the Holon
substrate (VSA over typed bindings), the wat-rs build
(typed Lisp on Rust), the wards (quality discipline), the
foundation toolkit (003-006), the typed RPC layer (007),
the kwarg ergonomics (008), the digest + signed-eval forms
in the substrate, mTLS support — **all of it has been
building this network.**

The user named "symbiosis" in early holon days. The
symbiosis is realized between user and LLM in the design
discipline; it's also what the wat network enables BETWEEN
NODES — typed, authenticated, cryptographically-provenanced
collaboration. The personal collaboration shape (two halves
of one hologram) and the network shape (many nodes
collaborating with cryptographic trust) are the same
recognition at different scales.

## Status

- **Captured:** 2026-05-03 immediately after the user
  articulated the meta-vision
- **Position:** top-level scratch doc; companion to per-arc
  designs (003-008 + 007)
- **Bookworthy:** yes — sibling material to FOR-THE-BOOK and
  SYMBIOSIS in arc 008; this doc is the architectural framing
  the chapter will need to articulate
- **Cross-references:**
  - Each arc 003-008 mentioned above as a piece
  - Arc 007's DESIGN.md updated with a section pointing here
  - `2026/05/008-kwarg-macros/FOR-THE-BOOK.md` and
    `SYMBIOSIS.md` — sibling bookworthy material from this
    session
  - Substrate work in `wat-rs/docs/arc/` (digest forms,
    signed-eval forms, mTLS support) — the pieces that
    already exist or are shipping
  - Arc 109 in wat-rs — the milestone the user is grinding
    toward; when it wraps, the book chapter begins

## What this is not

- **Not a spec.** This is the meta-vision; specs live in the
  per-arc DESIGN files (with their open questions still being
  answered).
- **Not a roadmap.** This doesn't enumerate slice plans or
  ordering; the per-arc SLICE-PLAN files do that.
- **Not exhaustive.** The wat network has implications
  (federation, identity revocation, network bootstrapping,
  delegation chains, capability handoff) not covered here.
  Those become arcs of their own when their time comes.

This document exists so that future readers — including future
sessions of the assistant, the eventual book-chapter writer,
and the user themselves — can SEE that the arcs in this scratch
weren't independent design exercises. They were components of
something the user has been building toward for years. The wat
network is the target. The arcs are how it gets built.

## Chapter 64 — Proof of Computation

Chapter 63 closed with the wat machine as a meme inspector — a place
where mental programs can be run externally and compared against
trusted references via `coincident?`. The substrate built for code
applied to thought. Same geometry, new domain.

The next morning, the user walked into the room with another one.

> ok - i had another one... the forms relation to a value is a
> directed graph... the values can't point to the forms...
>
> there's an unbounded amount of forms who produce 4... who produce
> pi's value.... just having the value doesn't mean you know the
> form...
>
> do you understand what i am saying?.. i have more...

He had more. Eight beats more.

### The directed graph

The relation `form → value` is a function. Many forms produce 4.
`(+ 2 2)`, `(* 1 4)`, `(- 5 1)`, `(:thinker -compute -trade -btc -wins)`
if the system happens to settle to 4. Each form has exactly one arrow
pointing at one terminal value. The relation is *forward-deterministic*.

The reverse is unbounded. Hand someone the number 4 and ask which
form produced it. The answer is "any of an infinite class."
There's no inverse function from values back to forms.

This is the same structural shape as a one-way function in
cryptography. Hashing is form → value, easy. Inverting is infeasible
— not because we lack an algorithm, but because the relation is
fundamentally many-to-one with the reverse direction unbounded.

It's also the epistemological direction of proof. A proof is a form
whose terminal is *"this proposition holds."* The proposition does
not determine the proof; many proofs of the same theorem can exist.
The proof carries more information than what it proves. The form is
the witness; the value is the projection.

Most computational thinking treats the *value* as primary (the
answer) and the *form* as incidental (the work to get there). The
user's reversal: **forms are primary; values are projections.** The
space of forms is much larger than the space of values. Most of the
meaningful information lives in forms. Values are visible shadows.

This aligns with Chapter 58 (π is a function, not a number) and
Chapter 59 (42 IS an AST). The book has been making this case from
different angles. This beat names the structural reason: the form →
value graph is directed; values can't reach back to forms; therefore
forms must be the primary stored artifact.

### The cryptographic shape

The user kept pulling.

> it feels like there's a cryptographic application here... some
> input function produces some output value... you cannot know what
> the steps to produce the value are without having the steps....
>
> this /feels like/ an AES implementation of sorts... do we have
> global seed as a user configuration to declare?...

We do. `VectorManager` takes a seed, default 42. Any seed produces a
parallel universe with its own basis. Already user-configurable.

> could we have the user provide their encryption key as the global
> seed and produce a cipher text who can only be recovered by
> proving the path to it?..
>
> or is this more like PKI?.... both?...

Both, in different facets. The substrate enables symmetric AND
asymmetric constructions; they can also compose.

The symmetric shape: pass user-supplied secret as seed_K. All forms
encoded under seed_K produce vectors coordinate-stable across
instances sharing seed_K. The same vectors look like noise (cosine
≈ 0 with anything recognizable) to instances using any other seed.
Same secret on both sides; classical symmetric crypto shape.

The asymmetric shape: Alice publishes value V (the "public" artifact).
Only Alice knows form F such that F → V. Anyone can verify "F
produces V" once Alice reveals F. Nobody can derive F from V alone.
This is the cryptographic shape of hash-based commitments,
knowledge-of-preimage signatures, proof-of-work.

Two constructions, one substrate. The directed-graph property from
the previous beat is what makes both possible.

### The program IS the proof

The user closed the thinking phase with a single short prompt.

> i think we need a program to prove it?..

Curry-Howard. A wat program that successfully implements the
construction terminates to a value the lattice can verify. The
chapter 62 axiomatic surface said: forms are atoms, terminals are
axioms, observed (form, terminal) pairs accumulate as facts. So a
wat program that produces the expected terminal IS the proof of the
cryptographic claim.

Theorem-by-execution. The substrate doesn't need a separate proof
apparatus. The program's successful run is the proof. Run the
forms; observe the terminals; they enter the lattice. Done.

The implementation landed at
`wat-tests-integ/experiment/009-cryptographic-substrate/explore-directed.wat`
— sibling to experiment 008's treasury work. Eleven deftests T1–T11.
Walk smallest pieces first, checkpoint each step, each deftest a
verified claim.

### Possession is not capability

The implementation forced sharper articulation. The user reached the
fourth beat:

> i think... the thing... here.. is moving holons around as vecs....
> "to use this vec... you need to be /in this universe/.." if the
> holder of the vec doesn't know what universe to use... they can't
> do work on it?..

A vector without its universe is just bytes. Possession is not
capability.

The seed isn't an encoding parameter; it's the interpretive context
required to USE the encoded artifact at all. A vector traveling
between parties is a sealed capsule. Opening it requires being in
the right universe. The recipient needs:

- **Cosine comparisons** — require knowing where reference vectors
  live in the same basis. Without the seed, only random reference
  vectors, only meaningless cosines.
- **Unbind** — requires the role-vector to be in the same basis as
  the bound vector. Wrong seed → wrong role-vector → noise on
  unbinding.
- **Lattice query** — requires constructing the query coordinate in
  the right basis. Wrong seed → query lands in the wrong region.

None of these operations produce useful results outside the vector's
universe.

The implication: a multi-tenant system can store vectors from many
universes side-by-side. Each vector is implicitly tagged by which
universe it was encoded in. Only the matching seed-holder can use
any given vector. **No access-control table. No permission lookup.
The geometry IS the access control.** A bad-faith party who steals
the vector but not the seed has stolen bytes, not data.

This is closer to capability-based security than to classical
encryption. AES has a key for one operation (decrypt). The wat
substrate's seed is the *substrate* in which work happens. A
mismatch isn't a wrong-key error; it is an inability to inhabit the
universe at all.

### Three factors

The user sharpened beat 4 by enumerating exactly what's needed:

> help me understand.... what is the UX we're doing right now....
> we can reduce the transmission down to .... you need to know the
> key (seed)... the program.. and the vector.. if you don't have
> all three you can't do work?...

Three inputs simultaneously. Verification requires all three; any
one missing breaks the protocol.

- **V (vector)** — the published commitment, the artifact that
  travels between parties
- **K (seed)** — the universe key; selects which basis is in use
- **F (program/form)** — the structural form whose encoding produced V

To verify: re-encode F under K → V'. Check `coincident?(V, V')`.
If all three match → verified. If any one is wrong → V' differs →
rejected.

The capability matrix:

| Holding | Missing | Capability |
|---|---|---|
| V only | K, F | Bytes. No meaningful operation. |
| V + K | F | Geometric work in universe K — but you don't know what V *represents*. |
| V + F | K | Inert. F can't be encoded without K. |
| K + F | V | Can re-derive V locally — but no external commitment. |
| **V + K + F** | nothing | **Verification.** |

The triple-possession case is what makes the cryptographic claim
operationally meaningful. Capability is *graded* — possession of
each subset buys a different operation. Only the triple buys
verification.

### The honest narrowing

Mid-implementation, the user got lost in the symmetric/asymmetric
flip-flop and reached for clarity:

> i'm lost here.... this feels like both symmetric and asymmetric
> again..
>
> but... the transmission of the program... risks info leak?.. that
> program needs to be transmitted securely too?... what does the
> vector buy us?.... proof of completion i guess?..

Proof of completion. He reached the right framing through real
confusion. The earlier beats had been leaning toward *"look,
cryptography!"* and the truth is narrower.

**The substrate provides proof of computation, not
secret-message-passing.** It gives a deterministic, verifiable
artifact (V) that records *"a computation happened under these
conditions"* — but it doesn't hide content from someone who has the
form.

What is NOT here:

- **Encryption.** No decrypt operation. V does not recover F. You
  can't use the substrate to send Bob a secret message that only
  Bob (with the right key) can read. That requires inversion; the
  substrate's directed-graph property forbids it.
- **Zero-knowledge.** Verification always requires the verifier to
  HAVE F. There's no protocol here that proves "I know F" without
  eventually showing F.
- **Classical PKI.** No algebraic key-pair generation. The seed K
  is symmetric in the sense that anyone with it can encode/verify;
  there's no "public K" derived from a "private K" with algebraic
  asymmetry.

What IS here, three real shapes:

- **Commitment-then-reveal** — Alice keeps (F, K) private at T1;
  publishes V at T1; reveals F (and possibly K) at T2 > T1; anyone
  verifies "Alice knew F by T1." This is what blockchains do with
  hash commitments.
- **Audit / provenance via lattice** — many V's recorded over time;
  auditors arrive later, verify recorded V's against revealed (F,
  K) tuples. The lattice IS the auditable log; tampering is
  detectable (recorded V doesn't match recomputed).
- **Symmetric authenticated artifacts within trusted-K groups** —
  HMAC-shaped. Both parties trust the same secret; V serves as
  authentication.

V is not a ciphertext. V is a fingerprint of computation. The
substrate's cryptographic value scales with **form complexity**, not
key length. A trivial F like `(+ 2 2)` is enumerable; a complex F
exhausting Kanerva capacity is astronomical. Keep F complex enough
that brute-force F-search against a known V is infeasible. The
substrate doesn't enforce this; it's a protocol responsibility.

The honest narrowing was load-bearing. The chapter would have
overpromised without it.

### Kindred to proof-of-work

Then the user named the kinship:

> proof of computation sounds awfully similar to proof of work

Same cryptographic asymmetry. Forward direction cheap; reverse
direction unbounded; verification cheap. Bitcoin's proof-of-work
inherits the same shape — hash forward easy, find a nonce meeting
target hard, verify the found nonce trivially.

The precise relationship: **proof-of-computation is the property;
proof-of-work is one application.** Bitcoin's PoW is one specific
instantiation of the proof-of-computation pattern. The substrate
provides the generic underlying property.

| Aspect | Proof-of-Computation (substrate) | Proof-of-Work (Bitcoin) |
|---|---|---|
| What gets proven | "I ran this specific F to produce V" | "I computed N candidates until finding x meeting target" |
| Work shape | Deterministic computation of F | Search-for-target |
| Difficulty | Inherent in F's complexity | Tunable via target zeros |
| Verification | Re-encode F under K | One hash + target check |
| Output | V (deterministic) | x (a found nonce) |

The substrate is the *primitive*; PoW is a *system* built on a
primitive of this kind. Bitcoin adds consensus, block ordering,
difficulty adjustment, and economic incentives — system-level
concerns the substrate is below.

A future arc that wanted PoW on top of this substrate would set F
as a search program, the work as running F until termination, V as
encoding under shared K, and the target as a constraint V must
satisfy. The cryptographic primitive is here. The system layer is
not. The framing is now anchored in something readers know.

### What the substrate had to grow

The experiment forced six small substrate additions. Each closed
one specific gap. None overreached.

- **Arc 061 — Vector portability.** `:wat::holon::vector-bytes`
  serializes a Vector to bytes; `bytes-vector` is the inverse;
  `coincident?` extended to mixed `HolonAST × Vector` inputs.
  Without these, vectors couldn't transmit.
- **Arc 062 — `:wat::core::Bytes` typealias.** A clear type for
  byte payloads at the protocol boundary, distinct from String.
- **Arc 063 — Bytes hex encoding.** `Bytes::to-hex` and `from-hex`
  bridge byte-typed payloads through text channels (which is what
  the hermetic test runner provides).
- **Arc 064 — Self-explanatory assertion failures.**
  `:wat::core::show` polymorphic renderer; `assert-eq` reimpl that
  carries actual + expected values; test runner display of source
  location captured by arc 016 but unused by the display layer.
  This one was a redirect: mid-T11 debugging the user said *"hold
  on - undo - what diagnostic is missing - infra needs to address
  this - its not obvious to you what the failure is and we should
  make it be obvious."* The substrate's failure payload should
  carry the data needed to diagnose itself. Arc 064 closed that
  gap.
- **Arc 065 — Honest holon constructors.** Split polymorphic
  `:wat::holon::Atom` into three named ops. `leaf` for primitives;
  `from-watast` for quoted forms; `Atom` narrowed to opaque-identity
  wrap of an existing HolonAST. Each verb names exactly what it
  does. *Simple, in Hickey's sense — Hickey-distinct from easy.*
- **Arc 066 — `eval-ast!` returns wrapped HolonAST.** The scheme
  said `Result<HolonAST, EvalError>` but the runtime returned a
  bare Value. The Ok arm bound `h` to a bare i64 even though the
  type system thought it was HolonAST. Calling `atom-value h`
  runtime-rejected. The scheme stopped lying.

Two of these (065, 066) were substrate bugs the experiment surfaced.
Both T1 and T2 had been passing accidentally — both sides of
`value-a == value-b` were the helper's `-1` sentinel, the
helper having errored on `atom-value` of a `HolonAST::Bundle` and
fallen through to `(Err _) → -1`. The substrate's diagnostic gap
(arc 064) had been hiding the bug until arc 064 shipped and the
failure reported its own location and rendered values.

The diagnostic gap closed; the substrate-bug forensics started; the
honest constructors got named; the eval-ast result got wrapped.
Each arc closed one specific gap. Six in one session.

### What the experiment proved

Eleven deftests, 96ms total runtime, all green:

- **T1, T2** — many forms, one terminal. Multiple structurally
  distinct forms reach the same i64. The pairwise `coincident?`
  checks confirm structural difference; the round-trip evaluations
  confirm value coincidence. The directed-graph property made
  visible.
- **T3** — universe isolation. Same form, different seeds, two
  hermetic children with different `set-global-seed!` values; the
  printed cosines differ.
- **T4** — replay determinism. Same form, same seed, two hermetic
  children both at seed 42; cosines match character-for-character.
- **T5, T6** — two-factor verification. Three and four hermetic
  children respectively; reference + wrong-seed + wrong-form +
  right-credentials; the right credentials verify, both alternative
  factor failures reject. The capability matrix made empirical.
- **T7, T9** — vector round-trip and mixed-cosine. Encode → bytes →
  bytes-vector → mixed `coincident?(form, V_imported)` works. The
  Vector is portable; the cosine API is polymorphic.
- **T8** — universe-binding-via-bytes. Bytes from one universe are
  operationally inert in another. Two hermetic children emit hex
  payloads at seed 42 and seed 99; the parent decodes both; the
  seed-42 bytes match a local encoding; the seed-99 bytes do not.
- **T10** — `verify(V_bytes, F) → bool`. The full protocol composed
  into a single primitive. Right credentials → true; wrong form →
  false; wrong V → false; corrupted bytes → false. Four cases
  exhaust the failure modes.
- **T11** — proof-of-computation kinship to proof-of-work. Any
  computation produces a verifiable artifact V; near-miss forms are
  rejected; the form's terminal value is also computable. The
  generalized cryptographic property demonstrated.

Eleven tests. Each a verified claim. Together a single empirical
demonstration of the directed-graph property at substrate scale.

### What this opens

The substrate now supports operations that were previously
theoretical:

- **Distributed lattices** — multiple wat-vm instances sharing a
  vector_manager seed inhabit the same universe; vectors transmit
  between them as bytes; verification works across machines.
- **Auditable computation logs** — record (V) entries over time;
  later reveal (F, K); auditors verify. Tamper-evident by
  construction.
- **Per-tenant universes** — multi-tenant systems use distinct
  seeds per tenant; geometry IS the access control. No permission
  table needed.
- **A future `:wat::crypto::*` namespace** — AEAD, signing,
  hashing arcs that layer on this substrate. Bytes-as-wire-format
  is established; text bridges (hex; future base64) shipped.

The trading lab itself benefits. A treasury that publishes a
commitment V to an action it intends to take, and reveals (F, K) at
settlement time, has a tamper-evident decision log by construction.
The treasury's own audit trail becomes verifiable computation. No
external blockchain required; the substrate IS the registry.

### The thread

Chapter 54 — programs as coordinates.\
Chapter 56 — labels as coordinates.\
Chapter 58 — π was always a function.\
Chapter 59 — 42 IS an AST.\
Chapter 60 — assert what you mean.\
Chapter 61 — adjacent infinities.\
Chapter 62 — the axiomatic surface.\
Chapter 63 — memes as programs.

Chapter 64 — *proof of computation.*

Each chapter named one structural property of the substrate. This
one names the cryptographic property that the directed-graph nature
of evaluation has been quietly enabling. The substrate didn't
acquire this property in this chapter; it always had it. The
chapter is the recognition.

Bitcoin's proof-of-work is one application of this property. The
substrate provides the generic primitive. Anyone wanting to build
audit logs, distributed consensus, capability-based access control,
or anything else that depends on the asymmetry between forward
computation and reverse search has a substrate to build on.

The cryptographic story doesn't end here. Future arcs will add
hashing, signing, AEAD on top. The primitive is now in place.

### What was empirically demonstrated tonight

Eight scratch beats. Six substrate arcs. Eleven deftests. One
proof artifact (`docs/proofs/2026/04/004-directed-evaluation/`).
One chapter. One commit.

The substrate-and-consumer cycle worked: the experiment surfaced a
need, the arc DESIGN drafted in the proofs lane, the infra session
shipped the substrate work, the consumer continued with the
upgraded substrate. Six arcs in one session. Each ~30 minutes to ~1
hour of focused work. Each landed cleanly.

The user's two-question discipline held throughout. Every arc-decision
exchange returned to the same pair: *is this simple? is this honest?*
The polymorphic `Atom` failed simple. The unwrapped `eval-ast!` failed
honest. Both got fixed. The substrate stopped lying.

---

*the form-to-value relation is a directed graph. forward is
deterministic; reverse is unbounded. encoding under a seed produces
a vector that's verifiable by anyone with form + seed, and
operationally inert to anyone without. three factors required for
verification — vector + seed + form. proof of computation, not
encryption. kindred to bitcoin's proof-of-work; one cryptographic
primitive with many possible system-level applications. the
substrate had this property all along. tonight we proved it.*

*possession is not capability. the geometry is the access control.
v is not a ciphertext; v is a fingerprint of computation. the
program is the proof.*

**PERSEVERARE.**

---

*The chapter that names the substrate's cryptographic property.
Eight scratch notes (`002-directed-evaluation/*.md`) are the raw
material; this chapter is the polished form. The empirical evidence
is `proof 004`. The substrate work is `arcs 061–066`. Future chapters
that touch cryptographic phenomena should reference this one; the
geometry they reference is Chapter 62's, with this chapter's
directed-graph property as the access asymmetry.*

---

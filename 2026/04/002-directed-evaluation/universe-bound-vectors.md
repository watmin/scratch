# Vectors are universe-bound — possession ≠ capability

**Captured**: 2026-04-26
**Status**: raw — fourth beat of the directed-evaluation arc;
   sharpens the cryptographic framing from beat 2
**Trigger**: user articulated that to USE a vector, you need to be
   in the right universe — without that, the holder can't do work

---

## The user's articulation (their words, preserved)

> i think... the thing... here.. is moving holons around as vecs....
> "to use this vec... you need to be /in this universe/.." if the
> holder of the vec doesn't know what universe to use... they can't
> do work on it?..

The structural claim sharpened: **a vector without its universe is
just bytes.** Possession is not capability. The seed is not just an
encoding parameter — it is the interpretive context required to USE
the encoded artifact at all.

## The shift in framing

Up through beat 3, the arc had been:

- Beat 1: forms point at values; values can't point back
- Beat 2: cryptographic application — seed-as-key (AES-shaped) +
  form-as-preimage-knowledge (PKI-shaped)
- Beat 3: the program IS the proof

This beat sharpens by naming the OPERATIONAL CONSEQUENCE: the seed
isn't just a key for the encoding operation. It is a capability for
*using* the encoded artifact at all.

A vector traveling between parties is a sealed capsule. Opening it
requires being in the right universe. The recipient needs:

- Cosine comparisons → require knowing where other reference vectors
  live in the same basis. Without seed → only random reference
  vectors → meaningless cosines.
- Unbind → requires the role-vector to be in the same basis as the
  bound vector. Without seed → wrong role-vector → noise on
  unbinding.
- atom-value → requires the leaf-vector lookup to match how the
  atom was encoded. Without seed → wrong leaf-vectors → recovers
  random leaves (if anything).
- Lattice query "is this form here?" → requires constructing the
  query coordinate in the right basis. Without seed → query lands
  in the wrong region of the lattice → false negatives every time.

None of these operations produce useful results outside the
vector's universe.

## Connection to Chapter 61

Ch 61 already named this: *"Storage as coordinates; the seed as the
access path."* This beat is the operational consequence of that
storage model — what it means in practice for parties holding
vectors across a network or across processes.

## Tenancy is geometric

The implication: a multi-tenant system can store vectors from many
universes side-by-side. Each vector is implicitly tagged by which
universe it was encoded in. Only the matching seed-holder can use
any given vector.

No access-control table. No permission lookup. **The geometry IS
the access control.** A bad-faith party who steals the vector but
not the seed has stolen bytes, not data.

## Vectors are portable but not interpretable

The substrate's vectors:
- Serialize fine — they're just bytes
- Travel fine — over the wire, on disk, in queues
- Persist fine — write them anywhere
- BUT they only become *meaningful* when handled inside the universe
  they belong to

This is closer to capability-based security than to classical
encryption. AES has a key for one operation (decrypt). The wat
substrate's seed is the *substrate* in which work happens. A
mismatch isn't a wrong-key error; it is an inability to inhabit the
universe at all.

## The cryptographic claim sharpened

The two-factor model from beat 2 becomes more concrete:

- **Universe (seed)** = capability to inhabit the working environment
- **Form (preimage)** = knowledge required for specific verification
- **Either factor alone** is genuinely insufficient — not "harder to
  break" but *operationally inert*

Without the seed:
- The vector can't be compared to anything meaningful.
- The vector can't be combined with other vectors (Bind/Bundle
  produce nonsense).
- The vector can't be queried.
- It is a sealed object.

Without the form (but WITH the seed):
- You can compare the vector to other vectors.
- But you can't reproduce the vector, because you don't know what
  inputs would have produced it.
- This is the directed-graph property from beat 1: reverse-search
  is unbounded.

Together: the seed gets you into the universe; the form gets you
to the specific point. Either alone leaves you outside.

## What this opens for the experiment

T3 demonstrated that different seeds produce different cosines.
This beat suggests an additional T-step that demonstrates the
INVERSE: a vector encoded in seed_42 cannot be MEANINGFULLY USED in
seed_99's universe. Specifically:

- T-future: encode form F under seed_42 → vector V
- Try to compare V to a reference under seed_99 → cosine should be
  noise/meaningless
- Try to compare V to the same reference under seed_42 → cosine is
  meaningful

The "meaningless" half is harder to assert mechanically — you need
a reference whose cosine to F is KNOWN under seed_42 and SHOULD NOT
be that under seed_99. We have that; T3 already showed cosines
differ across seeds.

So this beat doesn't necessarily require a new T-step. It's a
sharpening of T3's interpretation: T3 didn't just show "different
cosines"; it showed "the vector encoded in one universe is
operationally useless in the other."

## Connection to the broader book

- Ch 61: storage as coordinates; seed as access path
- Ch 62: the axiomatic surface; lattice as cumulative knowledge
- This arc: forms-to-values is directed; vectors are universe-bound

The book has been building toward a substrate where:
- Forms are atoms in a lattice (Ch 54, 56, 62)
- Values are projections (this arc, beat 1)
- Vectors are the portable encoded form (this beat)
- The seed determines which universe the vector inhabits (Ch 61)
- Possession of a vector without the seed is possession of bytes,
  not data (this beat)

This sounds increasingly like a *spatial information system* with
geometric access control. Distributed by construction (per Ch 61's
adjacent universes); private by geometry (this beat); auditable by
form-revelation (beat 2's PKI-shaped construction).

## Open questions

User has paused after this articulation. Hold for next beat.

Speculation (DO NOT extrapolate without user):
- Does the vector itself reveal which universe it belongs to? Or is
  the universe-tag separate metadata? (Probably separate — the
  vector is just numbers; the universe is config.)
- Multi-universe co-existence on a single substrate: can a single
  process hold vectors from many universes simultaneously? (Likely
  yes — different EncoderRegistry instances per seed. Per-instance
  config.)
- Cross-universe operations: is there ANY meaningful work that can
  be done on vectors from different universes? (Probably no — the
  bases don't share a common reference. The universes really are
  parallel.)
- Implications for distributed lattices: a federation of lattices
  each in their own universe; cross-federation lookups would
  require per-universe encoding of the query.

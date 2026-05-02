# Proof-of-computation kin to proof-of-work — the cryptographic lineage

**Captured**: 2026-04-26
**Status**: raw — seventh beat of the directed-evaluation arc; the
   connection to Bitcoin's proof-of-work that anchors the chapter's
   framing in something readers already know
**Trigger**: user said "proof of computation sounds awfully similar
   to proof of work"

---

## The user's articulation (their words, preserved)

> proof of computation sounds awfully similar to proof of work

Short prompt, sharp connection. The user named the kinship between
what we built and Bitcoin's PoW.

## The shared asymmetry

Both proof-of-computation (this substrate) and proof-of-work
(Bitcoin) share the same cryptographic asymmetry:

- **Forward direction**: cheap. Compute F → V under K, OR compute
  hash(block || nonce).
- **Reverse direction**: expensive (unbounded search). Recover F
  from V alone, OR find a nonce meeting target hash.
- **Verification**: cheap. Re-encode F + K → check matches V, OR
  one hash and target check.

This asymmetry is the cryptographic property both inherit. It's the
foundation of the directed-graph claim from beat 1 of this arc.

## The precise relationship

**Proof-of-computation is the property; proof-of-work is one
application.**

Bitcoin's PoW is *one specific instantiation* of the proof-of-
computation pattern. The substrate we've built provides the *generic
underlying property*. The relationship:

| Aspect | Proof-of-Computation (substrate) | Proof-of-Work (Bitcoin) |
|---|---|---|
| What gets proven | "I ran this specific F to produce V" | "I computed N candidates until finding x meeting target" |
| Work shape | Deterministic computation of F | Search-for-target |
| Difficulty | Inherent in F's complexity | Tunable via target zeros |
| Layer | Cryptographic primitive | Cryptographic primitive + consensus system |
| Verification | Re-encode F under K | One hash + target check |
| Output | V (deterministic) | x (a found nonce satisfying target) |

The substrate is the *primitive*; PoW is a *system* built on top of
that kind of primitive. Bitcoin adds consensus, block ordering,
difficulty adjustment, and economic incentives. We have only the
cryptographic property — not the system.

## How to build PoW on this substrate

If someone wanted to build a PoW system using our substrate:

- F = a search program. *"Find x such that predicate(x) holds."*
- The "work" = running F until it terminates with a found x.
- V = encoding of F-with-x under shared K.
- Anchor (the "target") = some constraint that V must satisfy
  (cosine to a known reference > threshold, or similar).
- Verifier holds K and the candidate x; runs F under K with x;
  checks output matches V.

The substrate provides the cryptographic primitive. The PoW system
on top would handle:
- Block ordering (which V was first?)
- Consensus (which V wins ties?)
- Difficulty adjustment (how hard should F be?)
- Reward distribution (who got the V published?)

These are SYSTEM-LEVEL concerns. The substrate is below them.

## Implications for the book chapter

The chapter (Chapter 64, drafting) should:

1. **Lead with proof-of-computation** as the central claim
2. **Name proof-of-work as the familiar instance** that anchors the
   reader. PoW is widely understood; "this substrate generalizes
   PoW's cryptographic property" is a clear framing.
3. **Distinguish primitive from system**. We have the property; we
   haven't built consensus or block ordering or any of the
   superstructure.
4. **Show the consumption path**. Anyone building a PoW-shaped
   system could use this substrate. We've articulated the
   primitive; we haven't claimed the system.

## Why this anchors the framing

"Proof of computation" alone is somewhat abstract. Reader has to
build the model from primitives.

"Proof of computation, kindred to proof-of-work but more general"
gives the reader a known anchor (PoW) and shows ours is the property
PoW depends on. Reader's model snaps into place; the substrate's
generality becomes visible.

This is the same move the lattice arc (Ch 62) made with Hilbert /
Gödel / Turing — anchor the new in the recognized.

## Other proof-of-X primitives in the same family

- **Proof of Stake (PoS)**: prove you have locked capital. Different
  property (economic commitment, not computational).
- **Proof of History (PoH)**: prove sequential time passed. Uses
  cryptographic asymmetry similar to ours but in time domain.
- **Proof of Replication (PoR)**: prove you stored data. Uses the
  forward/reverse asymmetry for storage proofs.
- **Proof of Authority (PoA)**: prove identity. Different — relies
  on classical PKI, not on the cryptographic asymmetry.
- **Proof of Burn**: prove value destruction. Economic, not
  computational.

Of these, PoW and PoH are the closest kin to our proof-of-computation
substrate — they all use the same forward/reverse asymmetry.

## What this resolves from beat 6

Beat 6 ("proof of computation, not encryption") narrowed the
substrate's claims honestly. This beat (7) places the narrowed claim
in the lineage of cryptographic primitives readers already know.
Together they give the chapter a clean two-step:

- Beat 6: this is *proof of computation*, not encryption / not ZK /
  not classical PKI
- Beat 7: proof of computation is *the cryptographic property*
  underlying Bitcoin's PoW; we provide it as a generic primitive

The chapter can land on this without overclaiming. It's an honest,
strong position.

## Open questions

User has paused. Hold for next move.

Speculation (DO NOT extrapolate without user):
- Does this strengthen the claim that the substrate is foundational
  for blockchain-like systems? Probably yes, but with caveats —
  we have the primitive, not the system.
- Could a future arc add the system layer? E.g., a consensus
  mechanism that orders V's. That would be a major addition; for
  now the primitive is what's named.
- Does Chapter 64's title shift? "Proof of Computation" remains the
  cleanest claim; the PoW kinship is a body-of-chapter framing
  device, not a title element.

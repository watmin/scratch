# Three factors — the verification UX articulated

**Captured**: 2026-04-26
**Status**: raw — fifth beat of the directed-evaluation arc; sharpens
   beat 4 by enumerating exactly what's needed
**Trigger**: user asked "what is the UX we're doing right now... if
   you don't have all three you can't do work?"

---

## The user's articulation (their words, preserved)

> help me understand.... what is the UX we're doing right now.... i
> think we must entertain importing an existing vector into the
> system... that vector is coupled to some universe... if you don't
> know which universe you cannot do work.... we can reduce the
> transmission down to .... you need to know the key (seed)... the
> program.. and the vector.. if you don't have all three you can't
> do work?...

The framing the user is reaching for: **verification needs three
inputs simultaneously — vector V, seed K, program F. Any one missing
and the protocol breaks.**

## The three factors

For the verification operation specifically:

- **V (vector)** — the published commitment, the encoded artifact
  that travels between parties
- **K (seed)** — the universe key; selects which basis is in use
- **F (program/form)** — the structural form whose encoding produced V

To verify: re-encode F under K → V'. Check `coincident?(V, V')`.

If all three match → verified.
If any one is wrong → V' differs → rejected.

## The capability matrix

What you can DO depends on which subset of the three you possess:

| Holding | Missing | Capability |
|---|---|---|
| V only | K, F | Bytes. No meaningful operation. |
| V + K | F | Geometric work in universe K — cosines, comparisons — but you don't know what V *represents*. |
| V + F | K | Inert. F can't be encoded without K, so V can't be verified. |
| K + F | V | Can re-derive V locally — but no external commitment to verify against. |
| **V + K + F** | nothing | **Verification.** Encode F under K, compare to V via `coincident?`. |

The triple-possession case is what makes the cryptographic claim
operationally meaningful. The substrate makes this asymmetry
*geometric*: capability requires all three.

## Nuance — capability is a function of what you're trying to do

The three-factor claim is precise for VERIFICATION. Other operations
have different requirements:

- **Read as data within a known universe**: V + K. You can do
  geometric work without knowing the originating form. Use case: a
  party that's been given access to vectors in their universe but
  doesn't need provenance.
- **Re-derive a known commitment**: K + F. You can compute the
  commitment yourself, but you have nothing external to compare to.
  Use case: an originator computing their own commitments.
- **Verify a third party's commitment**: V + K + F. Cryptographically
  meaningful. The user's three-factor case.

The third row is what the substrate's directed-graph property earns
its keep on. T6 demonstrates exactly this protocol.

## How this sharpens beat 4

Beat 4 said *"possession ≠ capability."* This beat enumerates:

- Possession of V alone → no capability
- Possession of V + K → partial capability (geometry without semantics)
- Possession of V + K + F → full verification capability
- Possession of K + F → producer capability (can mint commitments)

Capability is a graded property tied to which factors are held.

The user's "if you don't have all three you can't do work" applies
to verification work specifically. This is the central cryptographic
operation; calling it "the work" is fair shorthand for the substrate's
core utility.

## Connection to existing artifacts

T6 in the experiment IS the three-factor demonstration:

- Reference child: produces V under (K=42, F=correct)
- Right-credentials child: V'_correct = encoding under (K=42, F=correct) → V_correct == V_ref
- Wrong-seed child: V'_wrong-seed = encoding under (K=99, F=correct) → V_wrong-seed != V_ref
- Wrong-form child: V'_wrong-form = encoding under (K=42, F=wrong) → V_wrong-form != V_ref

Three factors required; demonstrated by exhausting the failure modes.

The UX is now articulated as: *"to import and verify a vector, you
need its universe (seed) and the program that produced it. All three
or nothing."*

## Implications for the book chapter

When the chapter is drafted, this beat provides the chapter's
**operational synthesis**:

- Beat 1 establishes the directionality (forms→values, no reverse)
- Beat 2 names the cryptographic shapes (AES + PKI)
- Beat 3 names the proof model (theorem-by-execution)
- Beat 4 sharpens to "possession ≠ capability"
- Beat 5 (this beat) enumerates exactly: three factors, all required

The chapter can use the capability matrix as a clean structural
device — readers see what each subset of (V, K, F) buys you, and
the triple-possession case lands as the cryptographic primitive.

## Open questions / next beats

User has paused. Next move: write the book chapter + proof + commit
all together with the experiment.

Speculation (DO NOT extrapolate without user):
- The triple (V, K, F) starts to look like a *capability triple* in
  capability-based security terms — possession of the triple grants
  one specific operation (verification); subsets grant lesser ones.
- If the substrate added a way to PUBLISH triples atomically (e.g.,
  to the lattice) and let parties query them, you'd have a
  cryptographic registry: "here's V, K hint, F hint — verify if you
  can." This becomes a public-key infrastructure (PKI) substrate.
- The "F as program" angle is interesting because F can be ARBITRARILY
  COMPLEX — anything up to the Kanerva-capacity ceiling. Different
  programs encode different commitments. The PKI flavor: F is the
  identity-bearing thing.

# The program IS the proof — implementation as theorem

**Captured**: 2026-04-26
**Status**: raw — third beat of the directed-evaluation arc; the
   beat that pivots from thinking to implementation
**Trigger**: user said "i think we need a program to prove it?"
   after the cryptographic-application beat

---

## The user's articulation (their words, preserved)

> i think we need a program to prove it?..

Short prompt, big move. The user is closing the thinking phase and
opening the implementation phase. The cryptographic claims from
beats 1 and 2 are theoretical until a wat program demonstrates them.

## The Curry-Howard angle

A wat program that successfully implements the construction
terminates to a value the lattice can verify. Per Ch 62's axiomatic
surface, that termination IS the empirical proof: someone observed
F → V, the lattice records the (form, terminal) pair, the
construction is now demonstrably possible.

This is **theorem-by-execution**:
- Curry-Howard: programs are proofs of their type signatures
- Axiomatic surface: forms are atoms; terminals are axioms; once
  observed, they enter the lattice as facts
- Combined: a wat program that produces the expected terminal IS
  the proof of the cryptographic claim

The lattice doesn't need a separate proof apparatus. The program's
successful run is the proof.

## What a minimal demonstration program would prove

Three claims, three demonstrations:

### Claim 1: The directed-graph property

**Demonstration**: a set of forms F1, F2, F3 all terminate to the
same value V. Show:
- F1 → V (different surface, same terminal)
- F2 → V
- F3 → V
- Given V alone: no native query in the lattice that recovers any of
  the source forms

This is the empirical "many forms produce 4" from beat 1.

### Claim 2: Universe isolation (seed-as-key)

**Demonstration**: same form F evaluated under different seeds:
- F under seed_42 → V_42
- F under seed_99 → V_99
- coincident?(V_42, V_99) → false (they are coordinate-different)
- Replay under seed_42 produces V_42 deterministically (same form
  + same seed → same value)
- Replay under any other seed produces a different vector that's
  cosine ≈ 0 with V_42

This is the symmetric (AES-shaped) construction from beat 2.

### Claim 3: Two-factor verification

**Demonstration**: Alice publishes V (commitment); Verifier holds
(seed_K, candidate F'); verifier runs F' under seed_K → V'; verifier
asserts coincident?(V, V'). Cases:
- F' is the right form, seed_K is the right seed → V' coincides → verified
- F' is wrong, seed_K is right → V' different → rejected
- F' is right, seed_K is wrong → V' different → rejected (wrong universe)
- F' wrong AND seed_K wrong → rejected (everything fails)

This is the asymmetric (PKI-shaped) construction layered on top of
the symmetric one from beat 2.

## Where the implementation lands

Two options in the lab repo:

**A) Experiment** — `wat-tests-integ/experiment/NNN-cryptographic-substrate/`
- Sibling to experiment 008 (Treasury)
- Same file structure: `explore-X.wat` files with deftests
- Walk smallest pieces first; checkpoint each step
- Each deftest is a verified claim

**B) Proof** — `docs/proofs/2026/04/NNN-directed-evaluation/`
- Empirical write-up with measurements
- Cosine values, replay determinism, universe isolation tables
- More polished; usually follows successful experimentation

Recommended path: **experiment first, proof second**. The experiment
exercises the substrate directly and produces passing deftests
that demonstrate each claim. The proof, if wanted later, is the
measured write-up of what the experiment showed.

## Pattern reuse from experiment 008

The Treasury work this afternoon established a clean pattern:

1. Set up the namespace (`:exp::*` for short experiment names)
2. Define the smallest possible types (Request, State, etc.)
3. Build the smallest possible service skeleton
4. Walk forward in T1, T2, T3, ... — each adding one new claim
5. Commit + push at each meaningful checkpoint

For the cryptographic experiment:
- T1: `(spawn form-F)` produces value V; `(coincident? V V) = true`
- T2: F1, F2, F3 all produce V; pairwise coincident? checks pass
- T3: F under seed_K1 vs F under seed_K2 produce coordinate-different V's
- T4: Two-factor verification — only the right (seed, form) pair recovers
- T5+: Whatever further claims emerge during implementation

## Connection to prior arc work

- Beat 1 (forms-as-directed-graph) — the geometric foundation
- Beat 2 (cryptographic-application) — the cryptographic
  interpretation of the geometry
- This beat — the implementation directive

The directed-evaluation arc may close once the experiment lands
its first set of deftests. The proof step (a polished write-up of
what was demonstrated) would land in a subsequent artifact.

## Open questions

User hasn't directed scope yet — implicit in their "i think we need
a program to prove it" is "and you should propose what that program
looks like." The response is the proposal; user will direct from
there.

Speculation (DO NOT extrapolate without user):
- Whether to build the experiment now or schedule it (Treasury is
  also in flight; doing both in parallel is feasible since they're
  in different namespaces)
- Whether the experiment is a stepping stone toward a real
  cryptographic feature in the substrate (e.g., a `:wat::crypto::*`
  namespace) or just an empirical demonstration that stays in
  experiments
- Whether the proof artifact, when it eventually lands, becomes a
  book chapter (Chapter 64?) or stays as a docs/proofs entry

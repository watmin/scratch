# T7-T11 roadmap — post-arc-061 implementation plan

**Captured**: 2026-04-26
**Status**: roadmap for compaction recovery
**Trigger**: arc 061 (vector-portability) shipped at wat-rs c63297e;
   user said "go to T11 and we make a judgement call then"

---

## Substrate primitives now available (arc 052 + arc 061)

From arc 052 (already shipped pre-discussion):
- `:wat::holon::Vector` (first-class encoded vector type)
- `:wat::holon::encode` (HolonAST → Vector under current universe)
- `:wat::holon::cosine` polymorphic (HolonAST × HolonAST | Vector × Vector | mixed)

From arc 061 (just shipped, c63297e):
- `:wat::holon::vector-bytes` (Vector → Vec<u8>) — serialize
- `:wat::holon::bytes-vector` (Vec<u8> → Option<Vector>) — deserialize, fails on bad bytes
- `:wat::holon::coincident?` widened to polymorphic (HolonAST | Vector) in either position

Wire format: 4-byte dim header (u32 LE) + 2-bit-per-cell ternary
packing. At d=10000: 2504 bytes per vector.

---

## T7 — vector serialization + verification round-trip (in-process)

**Goal**: demonstrate the verification protocol works using the new
Vector primitives end-to-end within one process.

**Status**: planned

**Structure**:
- Alice's setup: form-correct as HolonAST atom; encode → V_alice
- Serialize: V_alice → bytes via vector-bytes
- Deserialize: bytes → V_imported via bytes-vector (Option, must Some)
- Round-trip assertion: coincident?(V_alice, V_imported) → true
- Verification with re-encoding: Bob encodes form-correct → V_bob
- Mixed-cosine assertion: coincident?(form-correct, V_imported) → true
  (uses arc 061's polymorphism — HolonAST × Vector)

**What it proves**: byte serialization round-trips losslessly; the
mixed-coincident? API verifies a HolonAST against an imported Vector
in the same universe.

**Failure modes**:
- bytes-vector returns :None → deserialize broken (would indicate
  arc 061 regression)
- Round-trip mismatch → encoding non-deterministic (unexpected)
- Mixed-coincident? false → polymorphism not working as documented

---

## T8 — universe-binding via cross-process byte handoff

**Goal**: empirically demonstrate beat 4 — a vector encoded in one
universe is operationally inert in another.

**Status**: planned

**Mechanism**: bytes ARE transmittable across hermetic boundary via
stdout, encoded as comma-separated decimal byte values on a single
line. Parent parses, reconstructs.

**Structure**:
- Helper (in test prelude): bytes-to-string and string-to-bytes
  conversions using:
  - `:wat::core::map` over the byte vector
  - `:wat::core::i64::to-string` per byte
  - `:wat::core::string::join ","` to combine
  - Reverse: `:wat::core::string::split ","` then `:wat::core::string::to-i64`
- Child A (seed 42): encode form → V_a → bytes_a → write as csv string
- Child B (seed 99): encode SAME form → V_b → bytes_b → write as csv string
- Parent reads both csv strings
- Parent (test runs under default seed 42) parses both:
  - bytes_a → V_a_reconstructed (via bytes-vector)
  - bytes_b → V_b_reconstructed (via bytes-vector)
- Parent computes coincident?(V_a_reconstructed, V_b_reconstructed)
- Assert: FALSE (different universes → different bytes → different
  reconstructed vectors → not coincident)

**What it proves**: bytes encoded in one universe, when reconstructed
elsewhere, do not coincide with bytes encoded under a different
universe — even if structurally the same form was used. The vector
is universe-bound; the bytes are universe-tagged by what they
encode.

**Failure modes**:
- byte transmission round-trip broken → check helper
- false-positive coincidence (vectors agree across universes) →
  astronomically unlikely; would indicate substrate bug

---

## T9 — mixed cosine as the verification primitive

**Goal**: demonstrate that mixed coincident?(HolonAST, Vector) is
the minimal verification API.

**Status**: planned

**Structure**:
- Alice (in current process): form-correct → V_alice
- Verifier scenarios:
  - Right form: coincident?(form-correct, V_alice) → true
  - Wrong form (structurally different): coincident?(form-wrong, V_alice) → false
  - Tampered V (deliberately corrupt bytes): bytes-vector returns
    Some(V'), coincident?(form-correct, V') → false
- Three assertions, three sub-cases

**What it proves**: a single coincident? call with (HolonAST,
Vector) is the complete verification API for proof-of-computation.
Different forms → false; tampered V → false. (Wrong universe is
covered by T8.)

**Failure modes**:
- Polymorphism not properly implemented → coincident? errors on
  mixed types
- Tampered bytes don't deserialize cleanly → tamper resistance is
  too aggressive (we want it to deserialize but verify-fail)

---

## T10 — explicit three-factor verification function

**Goal**: lift the verification protocol into a reusable wat
function: `verify(V_bytes, F) → :bool` (K is the current universe).

**Status**: planned

**Structure**:
- Helper define in prelude:
  ```scheme
  (:wat::core::define
    (:exp::verify (v-bytes :Vec<u8>) (form :wat::holon::HolonAST) -> :bool)
    (:wat::core::match (:wat::holon::bytes-vector v-bytes) -> :bool
      ((Some v) (:wat::holon::coincident? form v))
      (:None false)))
  ```
- Test cases:
  - Right (V, K, F): verify(bytes-of-V_correct, form-correct) → true
  - Wrong F: verify(bytes-of-V_correct, form-wrong) → false
  - Tampered V: verify(corrupted-bytes, form-correct) → false (or :None handled)
- Three assertions on the helper's return value

**What it proves**: the three-factor protocol composes into a single
predicate function. The function IS the verification primitive.

**Failure modes**:
- Helper takes wrong types / arity error
- Tampered bytes give Some(noisy-vector) AND coincident? somehow
  returns true (false positive — would indicate coincident?'s slack
  is too generous)

---

## T11 — PoW-shape demonstration (proof-of-work kinship)

**Goal**: show the substrate can host a proof-of-work-shaped
protocol. Proves beat 7's lineage claim concretely.

**Status**: planned

**Structure**:
- A "search" form F that takes input X and computes a predicate
- Specifically: F(X) = whether a structural property of X holds
  (e.g., "X = some target value")
- The "miner" (one process) tries candidate X values until one
  satisfies the predicate
- Once found, encode (F-with-found-X) → V; publish V
- Verifier with V and the X-candidate runs F(X) → V', checks
  coincident?(V, V')
- Quick verification, expensive search

**Implementation simplification for T11**:
The substrate doesn't have heavy-loop primitives well-suited to PoW
search. Two options:
- (a) Use a TINY search space (e.g., 4 candidates) and demonstrate
  the SHAPE without the actual computational effort
- (b) Skip the search and just show: the encoded result of a known
  computation can be verified by anyone with the same X

Going with (b) for cleanliness: T11 shows that COMPUTATIONAL WORK
(any deterministic computation, not specifically search) produces a
verifiable artifact. PoW is one application; this is the general
property.

**Concrete structure**:
- Worker computes: F = `(* (* 7 13) (* 11 17))` (= 17017, a non-trivial product)
- Encode (F applied to specific args, but here F has no free vars):
  encode form-of-F → V_worker
- Verifier with form-of-F: encode → V_verifier
- coincident?(V_worker, V_verifier) → true (same work, same K, same V)
- Plus: a "doubter" with a different-but-similar form computes their
  V → does NOT match V_worker
- Demonstrates: the work has a specific, verifiable result; cheap to
  verify, expensive (in principle) to forge

**What it proves**: any computation produces a verifiable proof
artifact via the substrate. Proof-of-work is one shape; this
generalizes.

**Failure modes**:
- F too trivial → demonstration unconvincing (mitigation: pick
  reasonably complex form)
- coincident? slack too generous → "doubter" form falsely matches
  (mitigation: pick clearly distinct form structures)

---

## Cross-cutting concerns

**Form-size budget**: Per Kanerva capacity, each form ≤100 statements.
All planned forms are well under (T11's largest is ~5 statements).

**Hermetic boundary**: T8 needs byte-as-csv string transmission.
Helper functions for byte-vec ↔ csv-string conversion are needed at
the test prelude.

**Test ordering**: T7 → T8 → T9 → T10 → T11. Each builds on the
previous. T7 establishes the primitives work; T8 establishes
cross-process transmission; T9 simplifies the verification API; T10
formalizes the protocol; T11 connects to PoW.

**Commit cadence**: per user pattern — commit + push at meaningful
checkpoints. Likely commit after T7-T8 lands cleanly, then again
after T11.

---

## After T11 — judgment call point

User said: "go to T11 and we make a judgement call then"

After T11 lands, options:
- Commit + push the experiment + arc 061 DESIGN + scratch arc 002
- Write the proof artifact at docs/proofs/2026/04/NNN-directed-evaluation/
- Draft Chapter 64 from the seven scratch beats + experiment 009 results
- Single commit per user's earlier preference: book + experiment + proof together

The judgment call is whether T11 adds enough or we need T12+ before
calling the experiment complete. T11 might be the natural endpoint —
PoW-shape closes the cryptographic-property arc.

---

## Recovery procedure if compaction hits mid-implementation

1. Read this roadmap (this file)
2. Read scratch/2026/04/002-directed-evaluation/INDEX.yaml for arc state
3. Check experiment 009 dir: `ls holon-lab-trading/wat-tests-integ/experiment/009-cryptographic-substrate/`
4. Run tests to see what passes:
   `cd holon-lab-trading && cargo test --release --features experiment-009 --test experiment_009 -- --nocapture`
5. The passing T-tests are done; the missing ones are what's left
6. Pick up from the next T in this roadmap

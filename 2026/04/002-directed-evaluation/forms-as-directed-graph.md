# Forms-to-values is a directed graph — values can't point back

**Captured**: 2026-04-26
**Status**: raw — first beat of the directed-evaluation arc
**Trigger**: user said "i had another one" after the axiomatic-surface
   arc closed; said they have more

---

## The user's articulation (their words, preserved)

> ok - i had another one... the forms relation to a value is a
> directed graph... the values can't point to the forms...
>
> there's an unbounded amount of forms who produce 4... who produce
> pi's value.... just having the value doesn't mean you know the
> form...
>
> do you understand what i am saying?.. i have more...

The core claim: **the form → value relation is a function (many-to-one,
deterministic); the value → form relation is unbounded (no inverse).**

## The geometry

Forms-to-values is a directed graph:
- Nodes: forms (left side), values (right side)
- Edges: each form has exactly ONE arrow pointing to its terminal value
- Multiplicity: many forms point to the same value (4 has unbounded
  forms feeding into it; π's ratio likewise)
- Reverse: no arrows from value nodes back to form nodes —
  that direction is undefined; values don't determine forms

The forward direction is *computation* — apply the form, get the
value. Cheap, deterministic, terminating (when the form terminates).

The reverse direction is *search* — given a value, which form
produced it? The answer is "any of unbounded many." There's no
inverse function.

## Why this matters

This is the same shape as a one-way function in cryptography. Hashing
is form → value, easy. Inverting (value → preimage form) is
infeasible — not because we lack the algorithm, but because the
relation is fundamentally many-to-one with the reverse direction
unbounded.

It's also the same shape as the epistemological direction of proof:
a proof is a form whose terminal is "this proposition holds." The
proposition doesn't determine the proof — many proofs of the same
theorem can exist. The proof carries more information than what it
proves.

## Implications for what we built (Ch 62 lattice)

1. **The lattice stores forms, not values.** Entries are keyed by
   form (surface form is the lookup key per Ch 62). A query "what
   forms produce 4?" isn't natively answerable; it requires
   enumerating all entries and filtering. The lattice is a function
   lookup, not a value-indexed search.

2. **Extensional equality is asymmetric in information.**
   `(= (+ 2 2) (* 1 4))` from Ch 62 holds: both terminate to 4.
   But the forms themselves carry more — structure, derivation,
   computational cost. The terminal is the value; the form is the
   witness.

3. **You can't deduce a program from its output.** Hand someone the
   number 4; they can't reconstruct `(+ 2 2)`. The output is the
   projection; the form is the source. Forms are PRIMARY artifacts;
   values are derived.

4. **The lattice's depth is the form depth, not the value depth.**
   When the lattice grows in DEPTH (theorems building on theorems
   from Ch 62), it's the form-graph deepening, not the value-graph.
   The value side is comparatively flat — values are rarely
   constructed from other values; forms are routinely constructed
   from other forms.

## The reframing

Most computational thinking treats the *value* as primary (the
answer) and the *form* as incidental (the work to get there).
Compilers optimize away forms that are equivalent under value. Cache
hits return values; nobody asks for the form back. The conventional
hierarchy is: values matter; forms are scaffolding.

The user's reversal: **forms are primary; values are projections.**
The space of forms is much larger than the space of values. Most of
the meaningful information lives in forms. Values are the visible
shadows.

This aligns with Ch 58 (π is a function, not a number) and Ch 59 (42
IS an AST). The book has been making this case from a different
angle. This beat names the structural reason: the form → value graph
is directed; values can't reach back to forms; therefore forms must
be the primary stored artifact.

## Connection to prior arcs

- Ch 54: programs as coordinates — programs (forms) get geometric
  identity in the substrate, separately from their evaluation
  results.
- Ch 58: π was always a function — function as form; the constant is
  what the function returns.
- Ch 59: 42 IS an AST — every "value" is itself a form; the
  distinction blurs at the substrate.
- Ch 62: the axiomatic surface — entries in the lattice are
  (form, terminal) pairs keyed on form.

This beat names what the directionality of the form-value graph
means for the substrate. Forms point at values; values can't point
back; the lattice is a function-indexed store, not a value-indexed
one.

## Open questions / next beats

User explicitly said "i have more". Hold for what comes next.

Speculation (DO NOT extrapolate without user):
- Cryptographic implications: the lattice is naturally one-way; this
  may be load-bearing for trust models in distributed lattices.
- Memetic implications: a meme's behavior (output) doesn't tell you
  the meme. Two distinct memes producing the same observable
  behavior are NOT necessarily the same meme. This complicates
  memetic forensics from Ch 63.
- Reverse search as work: building "what forms produce X?" is a
  search problem the lattice doesn't natively solve. May warrant a
  new substrate primitive (e.g., a value-keyed secondary index).
- Information-theoretic angle: form has more entropy than value;
  evaluation is information-lossy compression.

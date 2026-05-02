# Lattice as namespace — labels are tokens, not integers

**Captured**: 2026-04-26
**Status**: raw — third beat; follows lattice-bounded-infinities.md
**Trigger**: user generalized integer labels to arbitrary tokens

---

## The user's articulation (their words, preserved)

> now that we have the integer-like property - quantized.. discrete.. boundaries... they don't have to be integers... they are just labels... this space can implement a hash map...
>
> the atom of "foo" is tan to the atom of "bar"... if i had another atom.. (bundle "foo" "baz") and ask where this atom /is/ it will have a bias in the "foo" atom and essentially nothing (just tan) of the "bar" atom...
>
> yes?... "foo", "bar", "baz"... they /are/ integer like?...

The generalization: **the lattice doesn't care what the labels are.** Integers were one INSTANCE. Strings, UUIDs, symbols, any discrete tokens — all integer-like in the lattice sense. Each gets its own atom; the lattice structure is independent of token type.

## The signal-to-noise frame

User crystallized the discrimination geometry:
- **bias = present** — high cos to a known atom means that label is in the bundle
- **tan = absent** — near-zero residual means that label is NOT in the bundle

The threshold between bias and tan is the cell boundary from the previous beat. Crossing it flips your answer about presence.

This is the *signal* in the cosine query. Bias is signal; tan is noise. They are geometrically distinct because the lattice tangencies make them so.

## What the user's example implements

`bundle("foo", "baz")` is the SET form — superposition of unbound labels.
- Implements: set membership testing
- cos(bundle, X) tells you whether X is in the set

The HASH-MAP form adds binding:
- `bundle(bind("foo", 42), bind("baz", 99))` — superposition of key-value pairs
- Lookup: `unbind(bundle, "foo") ≈ 42` (plus noise from other entries)
- The bind step is what packs (key, value) into one vector that's recoverable from key alone
- Same lattice, just one extra rotation per entry

## Connection to the existing library (this is what it ALREADY does)

- `encoder.py` — role-filler binding for JSON. Each KEY gets an atom; each VALUE gets an atom; bind ties them together. The whole encoded structure is a bundle of these binds. This IS the hash-map form.
- `EngramLibrary` — patterns are atoms in the lattice; queries are cosine bias against the library; matches are "which atom (or atoms) does this query lean toward?"
- `cleanup` — moves a noisy bundle to its nearest CLEAN atom (snap to lattice point). This is "discrete-membership recovery" — removing the bias-toward-multiple noise to get back to a single label.

The user has BEEN doing this. They are now NAMING the geometry that makes it work.

## The bigger insight (under the hood)

The lattice with token-labels is a *namespace*. It's a way to embed any discrete vocabulary into a continuous space such that:

1. **Set operations** work via bundle + cosine threshold
2. **Map operations** work via bind + bundle + unbind
3. **Composition** is geometric (bundles are vectors, can be combined)
4. **Discrimination** has structure (bias vs tan, geometrically distinct)
5. **Queries are cheap** (cosine is one dot product)

This is functionally equivalent to a hash-map AT THE GEOMETRY LEVEL. No collision chains, no rebalancing — just lattice structure under cosine.

## Open questions / next beats

User asked "yes?... they /are/ integer like?..." — wants confirmation, has more.

Speculation (DO NOT extrapolate without user):
- The lattice structure suggests CAPACITY is geometric: more dimensions → more orthogonal labels → bigger namespace. The user has hinted at this with "raise dimension and the count explodes."
- Hash-map collisions in VSA aren't avoidable forever — at the Kanerva limit, atoms start to interfere (no longer pure tangency). The lattice DEGRADES gracefully (queries return both colliding labels with similar bias) rather than collide hard.
- The token-labels frame might be the bridge to symbolic computation in VSA. Symbols ARE labels; programs are bundles of bound (verb, args) pairs; execution is sequential unbinding.
- Or the user is heading toward: nested namespaces. A lattice in lattice. The interior of one cell is itself a 10k-dim space holding its own lattice of sub-labels. (This would be the holon recursion from the first beat.)

Hold for the user. They have more.

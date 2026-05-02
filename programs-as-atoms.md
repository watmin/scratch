# Programs as atoms — evaluation as lattice walk

**Captured**: 2026-04-26
**Status**: raw — fourth beat; follows lattice-as-hashmap.md
**Trigger**: user extended the labels-as-tokens insight to programs

---

## The user's articulation (their words, preserved)

> the atoms... they can be programs.. those programs... they have two terminal states... first... "did i terminate?" and "what value did i terminate to?"...
>
> we discussed this as the caching layer... i can ask those questions in sequence.... "have i seen this form terminate?" and if yes "what value is the form's terminal state?"...
>
> this means... we can do substitution in form evaluation... we can enounter a form duration evalutation... we can check if its been calculated and then check for its value... if we get a value (or terminal) miss we then must compute it...
>
> do you get this?... if there's some arbitrarily long form.. we can short cut parts of its interior as its already proven to have a value... we don't know if that value is useful.. but its proven to exist?... yes?..

The move: **labels are programs.** Each form gets its own atom. The atom encodes the form's identity (its structural coordinate); the terminal value is bound to that atom.

## The two-stage probe

Two queries from the lattice, in sequence:

1. **Presence query** (set membership): `cos(query, form-atom)` → bias means "yes, terminated"; tan means "haven't seen it."
2. **Value query** (hash-map lookup): `unbind(value-bundle, form-atom)` recovers the bound terminal value.

These are exactly the two lattice operations from beat 3. The user is composing them into a caching protocol.

## Evaluation becomes a lattice walk

At each sub-form encountered during reduction:
- Probe the lattice for presence.
- HIT → substitute the cached value, skip the computation.
- MISS → compute the form, then BIND the result back so the next encounter is a hit.

The cache grows organically. Every reduction contributes a new entry. No explicit memoization annotations needed — the geometry IS the memoization.

## The user's "useful vs exists" distinction

> we don't know if that value is useful.. but its proven to exist

This is the soundness boundary of the cache:

- **Existence**: the cache proves the form has a terminal value. This is a hard fact — observation accumulated from prior reduction.
- **Usefulness**: whether to USE the cached value depends on context-equivalence between cache-write and cache-read.

For PURE forms (deterministic in their explicit inputs): the cached value is always useful — substitution is safe.
For IMPURE forms (depend on hidden state, time, randomness): substitution only safe when context is invariant.

In a values-up substrate (which wat is), most forms are pure within their explicit inputs. The cache is sound for them by default.

## Halting as observation, not proof

This is a weak form of halting analysis:
- We can't PROVE that an arbitrary form terminates (that's the halting problem).
- We CAN observe that this exact form terminated last time we ran it.
- The lattice accumulates these observations as a growing knowledge base.

The cache is a memory of "what terminates" + "what it terminates to." Each entry is an empirical fact, not a theoretical guarantee.

## Connection to existing library work

The codebase already has hooks for this:
- `arc 057` (referenced in USER-GUIDE.md): "structural cache keys" — every wat form has a HolonAST with structural Hash + Eq, exactly so it can serve as a lattice atom.
- `:wat::lru::CacheService` — the runtime memoization service. Keys are typed; values are typed; the lookup is exactly the two-stage probe (presence via key match, then value retrieval).
- The comments in `wat/holon/*.wat` distinguish "Story 1: coordinate" (the atom) from "Story 2: value" (what it terminates to). The user is articulating WHY this distinction exists — it's the two-stage probe under the geometry.

The user has been BUILDING this. They are now naming why the geometry supports it.

## What this unlocks (foundational for what's next)

- **Universal memoization**: every form is potentially cached just by being an atom. No annotations needed.
- **Substitution-as-lookup**: form evaluation is a graph walk where each node is a lattice query.
- **Empirical termination**: the cache becomes a proof-by-observation that termination has happened for this form.
- **Composability**: caching one sub-form benefits all super-forms that contain it.
- **Discoverability**: the cache itself is queryable. "What forms do I know terminate?" is a lattice survey.

## Open questions / next beats

User asked "yes?" — wants confirmation, has more.

Speculation (DO NOT extrapolate without user):
- Cache invalidation: when does a cached entry become STALE? Probably when the form depends on something that's mutated. The geometry doesn't track this — it's the responsibility of the impurity-aware caller.
- Cache capacity: at the Kanerva limit, atoms start interfering. Cache lookups return noisy mixtures. The lattice DEGRADES gracefully — answers get less precise rather than wrong.
- Self-referential forms: a form that contains its own atom in its body. Does the cache handle this? Probably yes — the form-atom is computed structurally, not by evaluation.
- Distributed caches: multiple processes sharing the same lattice. Same vector_manager seed = same atoms = same cache entries. The user previously noted this enables consensus.
- Hot path: the user might next observe that the lattice walk itself has a cost — each query is a cosine sweep. Cleanup primitives (snap-to-nearest-atom) become important to keep the walk fast.

Hold for the user. They have more.

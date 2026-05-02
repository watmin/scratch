# Tangent Spheres — the geometry of holon coexistence

**Captured**: 2026-04-26
**Status**: raw — user's articulation + thinking-with extension
**Trigger**: user paused mid-Treasury work to think this clearly

---

## The user's articulation (their words, preserved)

> i need to think this at you... its hard for me to say.. but i can see it clearly... no... i can /think/ it clearly...
>
> the unbounded grid that we've been playing with.. the kanerva capacity... every 10k vec has 100 slots that can be assigned... those regions... those boxes... no.. those spheres.. they are unit spheres within a unit sphere.... (that's the holon part...)...
>
> these coexist... i've called them parallel before... but.. they are more than parallel... they are tangentially parallel.. they only points the 10k dim they share /are/ their edges..
>
> their contents... what's within their unit sphere... that's offlimits to others... i had a term.. i used it poorly...
>
> something like... a venn diagram who caches well... the venn diagram.. there's no overlap.. its just N circles who share some edge coordinate... they do not overlap.. they coexist with edges...

The corrections matter:
- "see clearly" → "think clearly" (precision about modality)
- "regions / boxes / spheres" (settling on spheres because of the 10k-dim curvature)
- "parallel" was wrong — too weak, implies side-by-side liftability
- "tangentially parallel" — closer, but "tangent" alone is already enough
- The Venn analogy: N circles, no overlap, sharing only edge coordinates

The user wanted a word and was groping for it. The word is **tangency**.

## The geometry (thinking-with extension)

Unit spheres tangent to unit spheres, all inside one larger unit sphere. They touch at edges. Their interiors stay sealed.

"Parallel" is wrong because parallel implies same-axis, side-by-side, liftable into each other. **Tangent says: geometry forbids merging.** Volume privacy, surface composition.

The Venn-without-overlap that the user wanted a word for IS tangency. N regions sharing boundary points, never volume.

## Why it caches

An interior is opaque to a query that doesn't carry the binding key. The edge IS the API. Nothing inside the sphere fires until someone arrives at the right tangent direction.

**Holon as closure**: opaque cell, surface-only interface, identity preserved in its tangencies.

This is also why cache invalidation only fires on edge-touching operations. The interiors are inert until a query with the matching tangent vector arrives.

## Kanerva capacity reframed

The Kanerva capacity (~100 slots per 10k-vec) isn't arithmetic.

It's the **kissing number** of 10k-dim under the cosine threshold the system tolerates.

- ~100 spheres can kiss in 10k-dim before they start to interfere measurably
- Lower the threshold → count rises
- Raise the dimension → count explodes

The "100 slots" emerge from geometry, not from a constant the user imposed. They are a *property of the space* under the chosen tolerance.

## What this unlocks (foundation for what's next)

- **Privacy is geometric**, not enforced by convention. You can't read another holon's interior without its binding key, because the interior literally lives in a tangent region you have no axis into.
- **Composition is edge-only.** Bundles sum boundary directions, not interior volumes.
- **Scaling is dimensional.** Need more holons? Raise dimension. Need stronger isolation? Drop threshold. The system scales by ADJUSTING THE GEOMETRY, not by adding cells.

## Open questions / next beats

User said: *"there's more to this... but i need you to think this thought with me.. this is the first step... for what i think next..."*

The "first step" frame implies further beats. Possibilities (speculation, not user-stated):
- How tangencies COMPOSE — when two holons share an edge, what does the edge ITSELF encode?
- Tangency-as-protocol: if edges are APIs, can holons negotiate which tangencies they expose?
- The outer sphere (the world holon) — does its edge connect to nothing, or to the next-larger world?
- Strange-loop implication: a holon's interior is itself a 10k-dim space that can hold tangent sub-holons. Recursion of the structure.
- Caching invariant: when can you safely re-derive a tangency vs when must you cache?

Hold for the user's next articulation. Don't extrapolate without them.

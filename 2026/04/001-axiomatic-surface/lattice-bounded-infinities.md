# Lattice — discrete labels, bounded infinities, cos/tan as cell geometry

**Captured**: 2026-04-26
**Status**: raw — second beat; follows tangent-spheres.md
**Trigger**: user continued the thread immediately after the first stash

---

## The user's articulation (their words, preserved)

> earlier.... we called this structure... a lattice... the spheres act as if they are point like... their surfaces.. are points-ish?... they are discrete places..
>
> like the integer number line?... 1 is an infinity away from 2 ... 1.5 is somewhere in this infinity.. not just somewhere... /it is/ the center of this infinity.... we can do measurements on these infinities....
>
> the idea of 1 itself.. its the bounded infinity between 0 and 1... the idea of 2 is the bounded infinity between 1 and 2.. do you get this?...
>
> we can assign a coordinates to these infinities... this is the (x y) and (x y z a b) argument from an earlier chapter....
>
> we can ask new questions... we can use trig to do work here... tan defines the bounaries... cos defines the bias.... yes?... if i have /something/ represented as a vector... i can ask...
>
> which known integer coordinates do you lean towards... if the something is 1.5 .... is leans equally to 1 and 2... it finds it self in the two infinitity.. everything greaer than 1 is in the 2 infinity.. but its closer to the 1 edge than the 2 edge.... 1.1 /is in/ 2 but its far closer to 1 than 2...
>
> do you think this with me?....

The key reframe: **the integer N stops being a point and becomes a NAME for a region.**

- The integer's POINT is just an edge between two regions.
- Cell-N owns the bounded infinity ending at N — `(N-1, N)`.
- 1.5 ∈ (1, 2) → "in cell-2", equidistant from both edges.
- 1.1 ∈ (1, 2) → "in cell-2" but leaning hard toward edge-1.

A vector has TWO answers when placed in this lattice:
- **Cell-membership**: which neighborhood does it inhabit? Discrete.
- **Position-within-cell**: how does it lean toward each edge? Continuous.

## The cos/tan reframe

The user named the trig directly:
- **cos = bias** — how strongly a query pulls toward each neighboring edge
- **tan = boundary** — perpendicular bisectors between atoms; crossing them flips cell-membership

In Euclidean / VSA terms:
- Cosine similarity between a query vector and an anchor atom IS the bias toward that atom
- The locus where two cosines are equal IS the cell boundary (Voronoi boundary)
- A point's cosines to its surrounding atoms ARE its barycentric coordinates within the cell

## Connection to existing library work

This IS what scalar encoding (`$log`, `$linear`, `$circular`) already does:
- Anchor atoms placed at known positions on a scale
- Continuous value encoded as a vector with cosine relationships to anchors
- Recovery = reading the cosines as cell-coords

The user has been DOING this. They're now NAMING the geometry that makes it work.

## Generalization (the bigger move)

The lattice isn't just for numbers. It's the general structure of:
- Any concept-space where things have neighbors
- The holon system itself (zoom in: tangent spheres; zoom out: lattice points)
- Discrete labels + continuous interiors at every scale

Tangent spheres ARE the lattice points when you zoom out. The interiors of those spheres ARE the bounded infinities between lattice points. Same geometry, different zoom.

## What this connects backward

The earlier (x, y) and (x, y, z, a, b) chapter — coordinate assignment to bounded infinities. The user references this explicitly. The lattice frame says: those coordinates aren't ARBITRARY. They're cell-local barycentric weights derived from cosines to surrounding lattice atoms.

## Open questions / next beats

User asked "do you think this with me?" — they want the thinking-with confirmation before the next beat lands.

Speculation (DO NOT extrapolate without user):
- The lattice cells COMPOSE — a cell in one dimension can be a lattice point in another. Recursion of the structure.
- Querying for "lean toward N" is a *projection*; querying for "which cell" is a *threshold*. These are different operations on the same geometry.
- The earlier chapter's coordinates may not have been (x, y) cartesian — they may have been cell-local lean coords. If so, every coordinate system in the library is implicitly a lattice query.
- "tan defines boundaries" is precise — tan(angle) crosses 1 at 45°, the exact half-way line between two unit vectors at 90° to each other. This may be load-bearing for choosing thresholds.

Hold for the user. They have more.

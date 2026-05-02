# Surface forms as the database key — content-addressable computation

**Captured**: 2026-04-26
**Status**: raw — fifth beat; follows programs-as-atoms.md
**Trigger**: user extended caching to a global database, surface
   forms as universal lookup keys

---

## The user's articulation (their words, preserved)

> these forms... they have a surface.. their expansion may result in a million forms being evaluated to hit a terminal state.. but their surface.. may just hold a few dozen forms...
>
> that surface form... with discrete values.. not variable names.... think... (fn [x] (* x x)) we applied to the input 2 becomes (fn [2] (* 2 2)) .. we can ask if this surface form has a terminal state and what its value is....
>
> we don't hvae to explore the forms expansion at all.... the fact it has a terminal state and a value... means we can do something like.. (let [x ((fn [x] (* x x)) 2)] (println! x)) and just do the form expansion without the arithmatic.. the surface of the form.. the x's being swapped for 2's.... that's the lookup key...
>
> it doesn't need to be a cache... the cache is for local optimization... a database could exist.. that answers... "is this form terminal?" and "what is the terminal value?" - both can be executed for the user querying....
>
> the database will return back (Some n) or :None ... yes?...

The two big moves:

1. **Surface form** as the lookup key. Variables substituted with concrete values. No free names, no symbolic abstraction. The surface is what makes the form a *deterministic identifier* — two invocations of the same (function, input) produce the same surface, regardless of where or when.

2. **Cache → database**. Cache is per-process. A *database* is shared. Anyone who has ever computed this surface contributes the entry; anyone querying gets the result. Computations become commodities.

## The surface vs interior distinction

- **Interior**: the recursive expansion of a form. Could be millions of sub-forms.
- **Surface**: the form with its inputs literally substituted, before any reduction.

The user's example: `(fn [x] (* x x))` applied to `2`. The interior eventually evaluates to `4`, but the SURFACE is `(fn [2] (* 2 2))` — the substitution alone, no arithmetic. That surface is the lookup key.

So:
- `(let [x ((fn [x] (* x x)) 2)] (println! x))` — outer form
- The inner `((fn [x] (* x x)) 2)` has surface `(fn [2] (* 2 2))`
- Lookup that surface in the lattice. Hit → substitute `4`. Skip the arithmetic entirely.
- Continue with `(let [x 4] (println! x))`.

The expensive interior happens once. After that, only the surface walk happens.

## Cache → database scope shift

Same geometry, bigger scope.

| Property | Cache | Database |
|---|---|---|
| Scope | one process | many processes, users, machines |
| Key derivation | local | deterministic via shared vector_manager seed |
| Population | own evaluations | anyone's evaluations |
| Reply shape | Option<T> | Option<T> |
| Geometry | lattice | same lattice |

The crucial enabler is the shared seed. The library's vector_manager.py was DESIGNED for this — "same seed → same vector everywhere, enabling distributed consensus." The user's been positioning for this for a long time.

## What the database is, structurally

It IS a service. Two questions:
- "Have I seen this form terminate?" (presence query)
- "What was its terminal value?" (value query)

Both return `Option<T>`. `Some(n)` when known; `:None` when not.

This is exactly the request/reply shape from explore-treasury.wat. From the canonical service-template.wat. From CacheService. The geometry is universal — every domain that wants to answer "do I know this?" + "what is it?" implements the same shape.

## Closing the loop

The entire system we've been building this session IS a service that answers two questions:
- "Have I seen this {form, paper, message, key, ...}?"
- "What value does it correspond to?"

Treasury opens/closes papers; CacheService stores/retrieves K→V; the global computation database stores form→terminal. Same skeleton. The discrete identifier varies by domain; the geometry is universal.

The strange-loop nature: the wat machine running these queries is itself a form whose surface can be a key. The runtime can store its OWN past evaluations and short-circuit them. The lattice eats its own tail.

## What this unlocks (the larger frame)

- **Computation as commodity**: once any node computes a surface, every node has it. Reuse is geometric, not just memorial.
- **Provenance**: the lattice IS the proof "this surface terminates to that value" — you can interrogate the database for empirical termination evidence.
- **Sound impurity**: bundle the function's hidden state into the surface, and impure functions become safely cacheable. The "purity" of cached lookup depends on what's on the surface, not on the function's intrinsic nature.
- **Universal addressability**: every computation has a coordinate. The coordinate doesn't need to be assigned by anyone — it's derived from the structural identity of the surface form.
- **Cooperative work**: machines (or agents) collaborate by populating the same lattice. Different agents pick different sub-forms; each contributes; the whole picture assembles itself.

## Open questions / next beats

User asked "yes?" — wants confirmation, has more.

Speculation (DO NOT extrapolate without user):
- The "useful but proven to exist" frame from the prior beat now applies BIGGER: the global database may have entries that aren't useful in MY context (different impurity assumptions, different state). Cache hits are evidence; usefulness is a local judgment.
- Partial-evaluation entries: if a form is partially evaluated but not yet terminal, can the lattice hold IT too? Surface = "this much has been done"; eventually a terminal entry will replace it.
- Negative entries: can the lattice store "I tried to evaluate this and it diverged"? Or only positive terminations? (Probably positive only — divergence is the absence of an entry, by construction.)
- Trust: the database is only as sound as its writers. A bad-faith writer can populate wrong values. The geometry doesn't protect against this. Cryptographic signing? Multi-party verification? The user might be approaching this.
- The strange-loop angle: the wat machine evaluating user code is ALSO a program whose surface can be a key. The substrate runs on the same lattice it builds.

Hold for the user. They have more.

# Cypher bridge — the Neptune-era context that shaped wat

The user, 2026-05-02, after the protocol-as-checksum + language-
form-gaps + wat-to-english thread settled:

> "i honestly don't remember how i was doing to string... i was
> doing a lot with neptune db at the time.. working with
> relational triples and whatever else... i found the cypher
> lang to be quite close to what i wanted but its expression
> forms.... are very bad... i wanted them in wat and i thought
> that would be the bridge"

This file captures that historical context. Wat exists in its
current shape partly because the user lived inside Cypher's
limitations for years and built the substrate that escaped them.
Important context for the eventual wat-english + wat-sift
implementation work — sharpens what those crates are measuring
against.

---

## The Neptune-Cypher era

Around 2024, the user worked extensively with AWS Neptune
(graph DB) using Cypher (originally Neo4j's query language;
also supported by Neptune via openCypher). Triples and graph
patterns were the day-to-day data shape.

Cypher's basic form:
```cypher
MATCH (a:Person {name: "Alice"})-[r:KNOWS]->(b:Person)
WHERE b.age > 30
RETURN b.name, r.since
```

What's expressed:
- **Pattern** — `(a:Person {name: "Alice"})-[r:KNOWS]->(b:Person)` —
  graph structural template with node labels, relationship
  types, property bindings, and named slots
- **Filter** — `WHERE b.age > 30` — predicate over the matched
  bindings
- **Projection** — `RETURN b.name, r.since` — what to extract

This is declarative graph traversal. Powerful in the structural-
pattern dimension; awkward everywhere else.

## What was good about Cypher

The pattern-matching primitive itself was excellent:

- **Declarative structure.** You name what you want to find by
  drawing it; the engine figures out how to find it.
- **Bidirectional / variable-length paths.** `(a)-[*1..3]->(b)`
  finds nodes 1-3 hops away from a. Path expressions compose.
- **Named slots.** Every node and edge in the pattern can be
  bound to a variable that subsequent clauses reference.
- **Property destructuring.** `{name: "Alice"}` filters at the
  pattern level, before WHERE.

These are real strengths. The user wanted to keep them.

## What was bad about Cypher

The expression forms surrounding the pattern were a mess:

- **String-glue.** Most non-trivial queries were assembled by
  string concatenation in the host language. No type safety; no
  syntactic protection from injection or malformed queries.
- **No real type system.** Properties are typed but expressions
  over them aren't checked beyond the runtime. Fail fast at
  execution, not at composition.
- **WHERE / WITH / RETURN are SQL-influenced and inconsistent.**
  Different surface forms for different operations; no
  algebraic uniformity.
- **No hygienic macros / homoiconicity.** You can't take a
  pattern and operate on it as data. You can't generate Cypher
  from Cypher reliably.
- **Function library is bolted-on.** Math, string ops,
  aggregations — all special-cased; no first-class user
  functions in the same expression form as the query.
- **Similarity is alien.** Cypher does exact-match graph
  patterns. Fuzzy match (cosine over embeddings, partial
  pattern matching) requires separate machinery, often
  external services.

The user lived with this for long enough to know what a better
expression layer would look like. wat was the answer.

## "Wat would be the bridge"

The user's plan around 2024:
- Keep Cypher's pattern matching (or something equivalent)
- Replace the WHERE / WITH / RETURN expression forms with wat
- wat would be the bridge between graph storage and clean
  algebraic queries

This is the bridge framing. Two endpoints (graph DB on one
side, clean expression language on the other), wat as the
spanning structure between them.

## What actually got built — the bridge absorbed both endpoints

Current wat-rs + Holon doesn't bridge between graph storage and
expressions. It collapses both endpoints into a single substrate.
The triple store IS the algebra IS the query language IS the
type system. There is no "other side."

Mapping the Cypher concepts to current wat:

| Cypher concept | Current wat realization |
|---|---|
| Triple `(s, p, o)` | `:wat::holon::Bind(role, value)` — role-marked edge |
| Subgraph pattern | `:wat::holon::Bundle` of Binds — the pattern IS the value |
| Node label | A type via `:wat::holon::Atom` typed leaf |
| Property | A Bind on a role axis |
| MATCH (structural pattern) | `:wat::holon::Hologram/find` with a Bundle as probe |
| WHERE (filter) | wat lambda passed as `filter` to Hologram/find |
| RETURN (projection) | wat function over the matched Bundles |
| Exact match | `:wat::holon::coincident?` predicate (cosine ≥ floor) |
| **Fuzzy match** | **Native** — cosine over the hypersphere IS the retrieval primitive |
| Variable-length paths | Bundle of Binds at varying Permute depths |
| Expression language | wat — Lisp, typed, hygienic, homoiconic |
| Storage | The substrate itself; `:wat::holon::HolonAST` IS the data |

The bridge they imagined is what got built. But it's stronger
than a bridge — it's a substrate where the storage and the
expression language ARE the same thing. No impedance mismatch
because there's no boundary.

## What's better than Cypher about the realized substrate

Five things wat-rs gives you that Cypher couldn't:

1. **Similarity is native.** Cosine retrieval is the primitive,
   not a bolted-on extension. Fuzzy match works out of the box.
   Partial patterns work — omitted Binds become unconstrained
   dimensions on the hypersphere.

2. **Patterns are first-class data.** A Bundle is a value. You
   can manipulate patterns with the same primitives you use to
   manipulate matched data. Cypher couldn't operate on its own
   queries this way.

3. **Hygienic, typed expression forms.** wat has Racket-style
   sets-of-scopes hygiene (058-031), parametric polymorphism
   (058-030 + 058-032 typed macros), and a type system that
   catches errors at parse time. Cypher's expression forms have
   none of this.

4. **Composability via the algebra.** Bind / Bundle / Permute /
   Blend / Reject / Subtract / Project / Amplify / Thermometer
   are six primitives that compose freely. Cypher's operators
   don't have this property.

5. **Learned patterns.** OnlineSubspace + Engram + EngramLibrary
   let the substrate LEARN what a pattern looks like from data,
   not just match against patterns the user wrote. Cypher
   couldn't do this at all.

## What's still missing — the explicit query surface

Current wat has the substrate but not yet the query surface. To
write the wat equivalent of:

```cypher
MATCH (a:Person {name: "Alice"})-[r:KNOWS]->(b:Person)
WHERE b.age > 30
RETURN b.name, r.since
```

You'd currently write something like:
```scheme
(:wat::holon::Hologram/find
  store
  (:wat::holon::Bundle
    (:wat::core::vec :wat::holon::HolonAST
      (:wat::holon::Bind :type    (:wat::holon::Atom :Person))
      (:wat::holon::Bind :name    (:wat::holon::Atom "Alice"))
      (:wat::holon::Bind :KNOWS   (:wat::holon::Bundle ...))))
  (:wat::core::lambda ((cos :wat::core::f64) -> :wat::core::bool)
    (:wat::core::> cos 0.7)))
```

Workable but verbose. The pattern is laid out by hand; the
filter is a lambda; the projection has to be done by walking
the result.

`wat-sift` is the consumer crate that surfaces this cleanly.
Per the memory-as-hologram arc (`scratch/2026/05/001-memory-as-hologram/sift.md`),
Sift is `stream → predicate → stream` — the wat-shape successor
to Cypher's MATCH-WHERE-RETURN composition. Once Sift ships, the
wat surface becomes:

```scheme
(:wat::sift::find store
  (:Person :name "Alice"
    (:KNOWS (:Person :age (> 30)))))
```

(or whatever shape the gaze pass settles on.)

## Sharpening what wat-sift is measuring against

This file's contribution to the eventual Sift implementation:
**the design brief is "wat-shaped Cypher with similarity
native, patterns as data, and hygienic expression forms."** Not
"a query DSL"; not "a graph language" — *the thing the user
needed when Neptune+Cypher fell short*.

Specific design implications:

- **Pattern surface should look like the data.** Cypher's
  ASCII-art `(a)-[r]->(b)` was nice but ad-hoc. The Sift surface
  should be Bundles-of-Binds with the same shape the substrate
  stores — uniformity beats ASCII art.
- **Filters should be wat lambdas.** Not a separate WHERE-
  clause language. The expression forms are the same forms used
  everywhere else in wat.
- **Projections should be wat functions.** Not a separate
  RETURN-clause language. Uniform composition.
- **Similarity threshold is a parameter, not a separate mode.**
  `:exact` corresponds to `coincident-floor`; loose matching to
  `presence-floor`; user-supplied thresholds work for everything
  in between. One operation; many calibrations.
- **Partial patterns are the default.** Cypher requires you to
  specify everything in the pattern. Sift should let you specify
  what you know and treat the rest as unconstrained dimensions
  on the hypersphere. The `?placeholder` shape in Cypher
  collapses into "just don't include the Bind."
- **Path / variable-length traversal piggybacks on Bundle depth
  + Permute.** Cypher's `[*1..3]` becomes a depth-bounded
  recursion in the Bundle structure. The substrate's encoding
  of nested Bundles via Permute already handles this.

## Connection to the rest of the arc

This context strengthens several other recognitions in this arc:

- **`protocol-as-checksum.md` — Verify shape.** Verification
  isn't just "did the LLM emit valid wat"; it's "did the LLM
  emit a wat-pattern that the substrate can match against
  stored data." Cypher's role here was always implicit — the
  query *was* the verification of "show me data shaped like X."
  The protocol architecture absorbs that role natively.

- **`wat-to-english.md` — Render via LLM call.** A wat-sift
  query rendered to English should read like a Cypher MATCH
  clause: *"Find all Person nodes named Alice who know a Person
  over 30."* Per the 2026-05-02 collapse, this rendering is an
  MCP call to the LLM ("render this EDN as English") — the LLM
  already has the engrams for structured-form-to-English
  rendering. No template walker; no per-form rendering rules.

- **`language-form-gaps.md` — Reference shape.** The reference
  / anaphora gap (Tier 1) IS solved by Hologram/find — which
  is the same primitive Sift uses. *"The previous one"* and
  *"the dog I mentioned earlier"* are queries against the
  conversation Hologram. Same machinery, different framing.

- **`english-surface-arc.md` — Surface dependency.** The
  wat-english consumer crate depends on wat-sift for the
  reference forms. Slice ordering: Sift first, then wat-english.

## Status

- **Captured:** 2026-05-02 in response to user's recall of the
  Neptune-Cypher era as the substrate's pre-history.
- **Design implication banked:** wat-sift is "wat-shaped Cypher
  with similarity native, patterns as data, hygienic expression
  forms" — that framing belongs at the head of the eventual
  wat-sift design.md / arc-opening.
- **Cross-references:**
  - `scratch/2026/05/001-memory-as-hologram/sift.md` — the
    naming and initial shape (gaze-named)
  - `language-form-gaps.md` — Tier 1 #7 (Reference / anaphora)
    depends on this surface
  - `wat-to-english.md` — render direction is an LLM call;
    Sift's output is rendered by the same MCP-mediated path
  - `english-surface-arc.md` — wat-english depends on Sift
- **Cross-history:** The user lived with Cypher's limitations
  for years; the substrate that escaped them is what this
  entire arc documents.

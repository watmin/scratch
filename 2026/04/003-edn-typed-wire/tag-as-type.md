# Tag as type — the pivot

## The user's question that reframed the design

> "can we not use edn's tags for types?.. go study edn and rust's
> edn lib"

This is the load-bearing turn of the arc. Everything before assumed dynamic-typed EDN with caller-side type narrowing; everything after assumed the wire format itself carries the type.

## Why the question collapses the earlier framing

EDN's tag system is **post-parse dispatch**. From the spec (edn-format/edn README):

> "A tag indicates the semantic interpretation of the following element... the reader will first read the next element (which may itself be or comprise other tagged elements), then pass the result to the corresponding handler for further interpretation, and the result of the handler will be the data value yielded by the tag + tagged element, i.e. reading a tag and tagged element yields one value."

So a tag is exactly: "here's a value; here's how to interpret it; the reader hands you the interpreted result." That IS a typed-read mechanism, native to EDN, with no need for a parallel sugar layer in wat.

The dynamic Value enum was solving "how does the caller specify the expected type at every read site?" The answer EDN already gives: **don't — let the data carry its own type via a tag.**

## The other constraints in the spec that shape the design

> "Tag symbols without a prefix are reserved by edn for built-ins
> defined using the tag system. User tags must contain a prefix
> component, which must be owned by the user (e.g. trademark or
> domain) or known unique in the communication context."

Built-in reserved tags:
- `#inst` — RFC-3339 instant in a string
- `#uuid` — canonical UUID string

Everything else is namespaced. So `wat/` is our tag namespace (with sub-namespaces beneath: `wat.core/`, `wat.holon/`, `wat.memory/`, `wat.scalar/`, ...). Application-level namespaces beneath the wat one too (`enterprise.observer/`, `my.app/`, etc.).

> "the next element (which may itself be or comprise other tagged
> elements)"

Nesting is supported. `#wat.holon/Atom #wat.holon/I64 42` is two tags, the inner one nested in the body of the outer. The reader processes them outside-in (parses the outer tag, then reads its body — which itself begins with another tag, recursively). This makes recursive types natural.

> "Implementations may report an error, call a designated
> unknown-element handler, or create a generic representation
> containing both tag and element."

Unknown tags can be preserved as `(tag, value)` for graceful interop. So a Clojure consumer that doesn't know about `wat.holon/Bind` can still read it — gets back a tagged-element pair, can ignore or pass through. Round-trip-safe interop with weaker partners.

## What this means concretely

The reader has three states:

1. **Untagged data** — bare `[1 2 3]`, `{...}`, `#{...}`, primitives. Parses to `:wat::edn::Value` (the dynamic enum from the earlier framing — still useful, just not primary). User pattern-matches to extract.

2. **Tagged data with a registered handler** — `#wat.core/Vec<i64> [1 2 3]`. Reader dispatches to the handler keyed on the tag symbol; handler returns a typed wat value (`:Vec<i64>` here). No caller annotation needed.

3. **Tagged data with no registered handler** — preserved as `:wat::edn::Value::TaggedElement(tag, body)`. Caller can dispatch manually or pass through.

The Value enum still exists. It's the fallback for paths (1) and (3). The typed path is (2), and tagged data drives that path automatically.

## What the reframing eliminates

- The `read-str-as<T>` typed-sugar function — no longer needed. The tag carries the type; the reader knows what to construct.
- The "caller annotates type at every site" cost — gone. The wire format itself has the discipline.
- The asymmetry between read and write — write emits a tag (since wat values know their types); read consumes a tag (since EDN tags carry type information). Symmetric round-trip.

## What the reframing doesn't eliminate

- The wat-side `:wat::edn::Value` enum — still the right answer for untagged or unknown-tagged EDN.
- The boundary transcoder — still needed (we discover in beat 3 that `::` doesn't lex inside edn-format symbols).
- The choice between sum-type variant styles (`#wat.core/Some` vs `#wat.core/Option<T> [:Some ...]`) — open in beat 6.

## Status of the pivot

This is the load-bearing decision the rest of the design hangs on. The user's question reframed a "how do we annotate types at read sites?" problem into a "the wire format already carries types" answer. Beats 3-6 work out the consequences.

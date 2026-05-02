# Dot namespace — Strategy A confirmed

## The decision

> User: "I like A - we need fqdn names .. the dot syntax is fine..."

Strategy A from beat 3 wins. Use `.` instead of `::` inside EDN tags and keywords; transcode at the wat ↔ EDN boundary.

## The transcoding rule

Mechanical, total, reversible.

**Wat → EDN (writing):**
- Wat keyword-path `:a::b::c::Name` becomes:
  - EDN keyword `:a.b.c.Name` (replace every `::` with `.`)
  - EDN tag `a.b.c/Name` (last `::` becomes `/`, preceding `::` become `.`)

**EDN → Wat (reading):**
- EDN keyword `:a.b.c.Name` becomes wat keyword-path `:a::b::c::Name` (replace every `.` with `::`)
- EDN tag `a.b.c/Name` becomes wat keyword-path `:a::b::c::Name` (replace `.` with `::`, then the single `/` with `::`)

The conversion is deterministic in both directions. No round-trip ambiguity.

## What this preserves

- **Wat source code is unchanged.** Keyword-paths still use `::`. Type system still operates on `:wat::holon::Atom`. The dot form NEVER appears in `.wat` files.
- **Type system is unchanged.** Same constructors, same dispatch, same paths.
- **EDN wire format is parser-clean.** No reliance on spec quirks edn-format doesn't honor. `cargo add edn-format` and we're done.
- **Round-trip identity.** Read → wat value → write produces byte-equivalent EDN.

## What this trades

- **Visual mirror is one transformation away.** Reading EDN, the developer's eye has to convert `.` to `::` mentally. Beat 5's examples are the surface this lives on; the dot form is readable but slightly less wat-native.
- **Wat keyword bodies in EDN look different.** `:wat.holon.Atom` instead of `:wat::holon::Atom`. Slight ugliness. But: the EDN form is only ever seen by EDN consumers (Clojure, debugging, fixtures); wat code itself never sees it.
- **Context-sensitivity at the boundary.** If we ever embed wat source-as-EDN (programs as transmitted data), the transcoder must NOT touch `::` inside string literals containing wat source. It only transcodes at the OUTERMOST EDN boundary. Implementation note for later.

## Multi-arg generic separator

User confirmed earlier:

> "ok.. assuming we go with _ for comma..."

Inside `<...>`, `,` is whitespace in EDN (would tokenize-break the type name), so we need a non-whitespace placeholder. Underscore wins:

- No prior meaning in wat type syntax (wat's underscore-prefix `_var` is for unused bindings, lexically distinct context)
- Visually quiet
- Symmetric with the dot rule — both are wire-format substitutions, both transcode mechanically

Final canonical shape:

```
#wat.core/Vec<i64>                  [1 2 3]
#wat.core/HashSet<i64>              #{1 2 3}
#wat.core/HashMap<String_i64>       {"a" 1 "b" 2}
#wat.core/Result<i64_String>        [:Ok 42]
#wat.core/HashMap<String_Vec<i64>>  {"a" #wat.core/Vec<i64> [1 2]}
```

## The compaction-amnesia pattern

This is exactly the pattern from the memory: **wire format and runtime form are different surfaces; the boundary is explicit; the transcoder is small and mechanical in both directions.**

- Wire (EDN): `wat.holon/Atom`, `String_i64`, `:wat.holon.Atom`
- Runtime (wat): `:wat::holon::Atom`, `:String,:i64`, `:wat::holon::Atom`

The runtime never sees the wire form. The wire never sees the runtime form. The transcoder has one job and never expands it.

## What "FQDN" means here

User said "we need fqdn names." That's the firm commitment to no-aliases-in-canonical-form. Every tag in the canonical wire format is fully qualified — `#wat.core/Vec<i64>`, never `#v/Vec<i64>` or `#Vec<i64>`. Two reasons surfaced:

1. **No registry-time configuration drift.** Two programs reading the same EDN must agree on what every tag means. FQDNs are content-addressed identifiers — the tag IS the name; the name IS the type.
2. **Clojure interop is more honest.** A Clojure consumer reading `wat.core/Vec<i64>` can decide what to do with it without consulting a wat-side alias config.

Reader-side aliases are an open question (see `open-questions.md`). But the writer always emits FQDN form. Asymmetric for ergonomics, symmetric for the wire.

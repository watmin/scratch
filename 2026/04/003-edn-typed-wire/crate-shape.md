# Crate shape — wat_edn as router, SymbolTable as authority

## The pivot (user's voice, verbatim)

After the gaze closed Q1, the conversation turned to where the dispatch lives. Initial framing: a `wat_edn` crate sibling to `wat_lru`, surfacing `edn-format` under `:rust::*` via `#[wat_dispatch]`, with bundled handlers for stdlib types and a `WatEdnEncode` trait for application types to register their own.

User: "i'd go against bundling in wat_edn.... wat_edn can't know what symbols exist in some user app... but wat knows... how can we have wat_edn suppport valid user expressions?..."

This reframes the entire layering question. The assistant had been thinking of `wat_edn` as a partial type registry; the user surfaced that it can't BE a type registry because it has no privileged view of the type universe. Only wat does.

## The reframing

`wat_edn` is not a registry. It's a **router**.

- **wat** owns the type universe via the SymbolTable. Every type — built-in (`Vec<T>`, `HashMap<K,V>`, `HolonAST`) or user-defined (`enterprise::observer::Engram`) — lives there with its full metadata: field names, types, variant constructors, generic parameters.
- **wat_edn** owns the protocol: parsing EDN, transcoding wire-form names to wat keyword-paths, walking type metadata to encode/decode bodies structurally.
- **At dispatch time, wat_edn DELEGATES to the SymbolTable.** No bundled handlers. No `register_handler!` calls. No application-side ceremony.

The dispatch loop:

```
"#enterprise.observer/Engram {...}"
  → edn-format parse → TaggedElement(Symbol("enterprise.observer/Engram"), body)
  → wat_edn transcodes prefix.name → wat path → :enterprise::observer::Engram
  → wat_edn looks up that path in the SymbolTable
  → Found a registered type? Walk its metadata, decode body structurally
  → Not found? Return Value::TaggedElement (dynamic fallback — caller decides)
```

`wat_edn` never knows what `Engram` is. It walks the metadata that the type's own SymbolTable entry already carries.

## Sketch

```rust
fn dispatch_tag(
    tag: &Symbol,
    body: Value,
    ctx: &EdnCtx,
    st: &SymbolTable,
) -> Result<WatValue, EdnError> {
    let wat_path = transcode_tag_to_path(tag);
    //   "wat.core/Vec<i64>"      -> ":wat::core::Vec<:i64>"
    //   "enterprise.observer/Engram" -> ":enterprise::observer::Engram"

    match st.lookup_type(&wat_path) {
        Some(TypeEntry::Struct(s))     => structural_decode_struct(s, body, ctx, st),
        Some(TypeEntry::Enum(e))       => structural_decode_enum(e, body, ctx, st),
        Some(TypeEntry::Parametric(p)) => structural_decode_parametric(p, body, ctx, st),
        Some(TypeEntry::CustomEdn(f))  => f(body, ctx, st),  // escape hatch
        None => Ok(WatValue::EdnValue(
            Value::TaggedElement(tag.clone(), Box::new(body))
        )),
    }
}
```

The `structural_decode_*` functions are wat_edn's. They never know what specific type they're decoding — they just walk metadata. The metadata IS the type definition, and the type definition lives in wat.

## Implications

1. **User apps get EDN read/write for free.** Declare a struct with `:wat::core::struct` → SymbolTable entry has fields and types → EDN read/write Just Works. Same for enums (variants map directly to per-variant tags, Style A from the gaze resolution).

2. **`wat_edn` stays tiny.** Parser shim + transcoder + dispatch loop + structural walker. Doesn't grow with the type universe. New application types don't add code to `wat_edn`.

3. **Built-in wat types are not special.** `Vec<T>`, `HashMap<K,V>`, `HashSet<T>`, `Option<T>`, `Result<T,E>`, the HolonAST variants — all live in the SymbolTable like everything else. wat_edn finds them through the same lookup. No `wat_edn_stdlib` companion crate, no special initialization step.

4. **Frozen-at-startup gives a stable type universe.** Per Model A (proposal 058), the SymbolTable is frozen before `:user::main` runs. wat_edn doing runtime EDN parsing sees a fixed type registry — no race, no dynamic redefinition surprise. Tags resolve deterministically against the same table for the lifetime of the process.

5. **`canonical_edn_holon` integration becomes natural.** The HolonAST type entry can carry a `CustomEdn` handler that delegates to holon-rs's existing `canonical_edn_holon` (already proven, byte-stable, used by signed-load digests). The structural default would also work, but registering the existing function is easier than verifying byte-equivalence. Same shape as Rust's `Serialize` derive vs manual impl.

6. **Unknown tags become dynamic fallback, not errors.** If a wat program reads EDN containing `#some.lib/UnknownType {...}` and that type isn't in the SymbolTable, the result is `Value::TaggedElement` — the caller can pattern-match it, pass it through, or error explicitly. Graceful interop, in line with the EDN spec's stated tolerance for unknown tags.

## Why this layering is the right shape

The principle the user surfaced: **dispatch mechanisms should not try to be authoritative about the universe they dispatch over.** HTTP routers don't define handlers; they route to them based on path. DNS resolvers don't own zones; they query authoritative servers. `wat_edn` should not own type definitions; it should query the SymbolTable.

This also tracks the project's existing architectural pattern. Capabilities (lru, channels, threads, file I/O) attach to the SymbolTable / encoding_ctx. They consult the table for what they need; they don't shadow it. `wat_edn` follows the same pattern — it's a capability that consults the SymbolTable's type-registry surface to do its job.

## Crate boundary, finalized

```
edn-format         (third-party)         parser, writer, Value::TaggedElement
                                              ↑
wat_edn            (capability crate)    Surfaces edn-format under :rust::*
                                          via #[wat_dispatch]
                                          Provides:
                                            - read-str / write-str
                                            - boundary transcoder (:: ↔ .)
                                            - dispatch loop (lookup + structural walk)
                                            - EdnEncode/EdnDecode trait (escape hatch)
                                          Consults SymbolTable. Holds NO handlers.
                                              ↑
wat                (language)            SymbolTable carries type metadata for
                                          every registered type — built-in and
                                          user-defined. wat_edn reads from this
                                          table at dispatch time. wat itself
                                          gains nothing new; the type system
                                          already had this metadata.
                                              ↑
application        (e.g. enterprise.*)   Declares its types via :wat::core::struct
                                          and :wat::core::enum. Gets EDN read/write
                                          for free. Can register CustomEdn
                                          handlers on specific types if the
                                          structural default is wrong.
```

## Status

Banked as design. No code written. Ready for implementation when a consumer surfaces — at which point the first slice would be: (1) wat_edn skeleton with `#[wat_dispatch]` shims, (2) transcoder, (3) dispatch loop with SymbolTable lookup, (4) structural walkers for struct/enum/parametric, (5) CustomEdn registration for HolonAST routing to canonical_edn_holon. Tests covering the realistic-blob example from `string-examples.md` would be the acceptance bar.

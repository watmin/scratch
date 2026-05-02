# String examples — the typed EDN wire format

> User: "ok.. assuming we go with _ for comma - can you generate a
> complex edn form that's fully typed - i want to see the various
> UX strings that could exist"

All examples in the **corrected** dot-namespace form (Strategy A from beat 4). All parse with `bowbahdoe/edn-format` as-is — no fork required. All round-trip to typed wat values via registered tag handlers. The transcoder converts `.` ↔ `::` at the boundary, so wat code never sees the dot form.

## Primitives — no tags needed (EDN handles them natively)

```edn
42                    ;; -> :i64
3.14                  ;; -> :f64
true                  ;; -> :bool
"hello"               ;; -> :String
:rsi                  ;; -> :Keyword (-> :rsi in wat)
:my.ns.thing          ;; -> :Keyword (-> :my::ns::thing in wat)
nil                   ;; -> :() / unit
```

EDN's primitive types map 1:1 to wat's. No wrapping required.

## Collections — the three core types from the original ask

```edn
#wat.core/Vec<i64>                 [1 2 3]
#wat.core/HashSet<i64>             #{1 2 3}
#wat.core/HashMap<String_i64>      {"a" 1 "b" 2}
```

The tag carries the type. The body is the natural EDN literal. Round-trip:
- Read `#wat.core/Vec<i64> [1 2 3]` → tag handler dispatches → `:Vec<i64>` value `[1 2 3]`.
- Write `:Vec<i64> [1 2 3]` → emit tag `#wat.core/Vec<i64>` + body `[1 2 3]`.

## Empty containers — tag is mandatory (no inference path)

```edn
#wat.core/Vec<i64>                              []
#wat.core/HashMap<String_f64>                   {}
#wat.core/HashSet<wat.holon.HolonAST>           #{}
```

Untagged `[]` would parse to `:wat::edn::Value::Vec<:wat::edn::Value>` (empty heterogeneous bag) — useless for typed extraction. Tagging makes the type explicit at the wire level.

## Multi-arg generics — `_` separator

```edn
#wat.core/HashMap<String_i64>          {"a" 1 "b" 2}
#wat.core/HashMap<i64_String>          {1 "one" 2 "two"}
#wat.core/Result<i64_String>           [:Ok 42]
#wat.core/Result<i64_String>           [:Err "boom"]
```

Why `_`: `,` is whitespace in EDN and would tokenize-break the type name; ` ` is whitespace too; `<` `>` `:` `.` `/` all have other meanings in the lex grammar; `_` is quiet and unambiguous.

## Nested generics — the cross-product gets rich

```edn
#wat.core/HashMap<String_Vec<i64>>
{"primes" #wat.core/Vec<i64> [2 3 5 7]
 "evens"  #wat.core/Vec<i64> [2 4 6 8]}

#wat.core/Vec<HashMap<String_f64>>
[#wat.core/HashMap<String_f64> {"open" 1.0 "close" 1.5}
 #wat.core/HashMap<String_f64> {"open" 1.5 "close" 1.7}]

#wat.core/HashMap<String_HashSet<i64>>
{"primes" #wat.core/HashSet<i64> #{2 3 5 7}
 "evens"  #wat.core/HashSet<i64> #{2 4 6 8}}
```

The outer tag declares the full shape; the inner tags declare each element's shape. Reader can verify: every element of a `Vec<HashMap<String_f64>>` should be tagged `#wat.core/HashMap<String_f64>`. Mismatch is a wire-format error.

## Option / Result — sum types, per-variant tagging (Style A, resolved 2026-04-26)

```edn
#wat.core/Some<i64>                42
#wat.core/None<i64>                nil
#wat.core/Ok<i64_String>           42
#wat.core/Err<i64_String>          "boom"
```

The tag IS the variant. The handler dispatched on a content-addressed identifier; no string-based discriminator inside the body, no protocol-level error paths in the handler. Symmetric with HolonAST's per-variant tagging. See `sum-style-resolution.md` for the gaze findings that closed this question.

## HolonAST — closed sum type, per-variant tags

The 11-variant union from holon-rs maps cleanly to per-variant tags (Style A applied):

```edn
#wat.holon/Symbol                  :rsi
#wat.holon/I64                     42
#wat.holon/F64                     3.14
#wat.holon/Bool                    true
#wat.holon/Keyword                 :momentum
#wat.holon/String                  "burst-pattern"

#wat.holon/Atom                    #wat.holon/I64 42

#wat.holon/Bind
  [#wat.holon/Atom #wat.holon/Symbol :role
   #wat.holon/Atom #wat.holon/Symbol :filler]

#wat.holon/Bundle
  [#wat.holon/Atom #wat.holon/Symbol :rsi-rising
   #wat.holon/Atom #wat.holon/Symbol :flow-positive
   #wat.holon/Atom #wat.holon/Symbol :momentum-up]

#wat.holon/Permute
  [#wat.holon/Atom #wat.holon/Symbol :x 3]

#wat.holon/Thermometer             [0.73 0.0 1.0]
```

Note nesting: `#wat.holon/Atom #wat.holon/I64 42` is the outer Atom tag wrapping the inner I64 tag wrapping the literal. Reader processes outside-in; each handler reads its body (which itself begins with another tag).

## Programs-as-holons — atomized AST inside another AST

```edn
#wat.holon/Atom<wat.holon.HolonAST>
  #wat.holon/Bundle
    [#wat.holon/Atom #wat.holon/Symbol :rsi-rising
     #wat.holon/Atom #wat.holon/Symbol :flow-positive]
```

The outer `Atom<wat.holon.HolonAST>` tag declares "this Atom wraps an AST"; the body itself is a tagged AST. This is the wire form of programs-as-holons (proposal 058's core principle): a program is an atomized HolonAST that can travel, hash, sign, and be evaluated.

## Mixed with EDN built-in tags — clean interop

```edn
{:created  #inst "2026-04-26T14:30:00Z"
 :id       #uuid "550e8400-e29b-41d4-a716-446655440000"
 :patterns #wat.core/HashSet<String> #{"burst" "trickle"}}
```

`#inst` and `#uuid` are EDN spec built-ins — the reader resolves them to instant/UUID values directly, no wat-side handler needed. They coexist with `wat.*` tags freely.

## Realistic blob — observer engram entry from the trading enterprise

```edn
#enterprise.observer/Engram
{:name      "rsi-divergence-buy"
 :asset     :BTC
 :timeframe #wat.scalar/Linear<f64> [300.0 0.0 86400.0]
 :horizon   #inst "2026-04-26T14:35:00Z"
 :tags      #wat.core/HashSet<String>
              #{"momentum" "oversold-recovery"}
 :priors    #wat.core/HashMap<String_f64>
              {"win-rate"   0.594
               "expectancy" 0.0023
               "sharpe"     1.41}
 :reasons   #wat.core/Vec<wat.holon.HolonAST>
              [#wat.holon/Atom #wat.holon/Symbol :rsi-rising
               #wat.holon/Bind
                 [#wat.holon/Atom #wat.holon/Symbol :flow
                  #wat.holon/Atom #wat.holon/Symbol :positive]
               #wat.holon/Bundle
                 [#wat.holon/Atom #wat.holon/Symbol :recent-low
                  #wat.holon/Atom #wat.holon/Symbol :volume-spike]]
 :stop      #wat.core/Some<f64> 64500.0
 :target    #wat.core/Ok<f64_String> 65800.0
 :uuid      #uuid "550e8400-e29b-41d4-a716-446655440000"}
```

This single blob exercises:
- An application-level tag (`enterprise.observer/Engram`) at the outermost layer
- A wat scalar form (`Linear<f64>` — a Thermometer-shaped scalar with min/max bounds)
- Built-in EDN tags (`#inst`, `#uuid`)
- Multi-arg generic with `_` separator (`HashMap<String_f64>`, `Ok<f64_String>`)
- Nested generics (`Vec<wat.holon.HolonAST>` — a vec of a sum type)
- Per-variant tagging (Style A) applied uniformly — HolonAST variants, `Some`/`Ok` for Option/Result. No body discriminators anywhere.

## Two things this surfaces

1. **The mechanical wat-↔-EDN namespace mapping is total.** Every wat keyword-path becomes an EDN tag/keyword by transcoding `::` → `.` (with the last `::` becoming `/` for tag prefixes). No special cases. No exceptions.

2. **Nested generics work without quoting hacks.** `Vec<HashMap<String_f64>>` is one atom (no whitespace breaks; all chars in legal set per `is_allowed_atom_character`). The reader recovers structure by parsing the type-name itself with a small grammar (angle brackets → generic args; underscore → comma).

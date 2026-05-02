# arc 003 — EDN typed wire

Notes from the conversation that designed this arc are in the
sibling `.md` files. The two-crate library scaffold sketched in
beat 9 (`consumer-bridge.md`) is realized below.

## Layout

```
003-edn-typed-wire/
├── INDEX.yaml                         beat-by-beat index, status, scope
├── README.md                          you are here
├── goal-and-constraints.md            beat 1 — problem framing
├── tag-as-type.md                     beat 2 — THE PIVOT (tags carry types)
├── edn-format-parser-findings.md      beat 3 — verification + lib limitation
├── dot-namespace-decision.md          beat 4 — Strategy A (`.` for `::`)
├── string-examples.md                 beat 5 — the surface
├── open-questions.md                  beat 6 — Q2/Q3 deferred
├── sum-style-resolution.md            beat 7 — Q1 closed via /gaze
├── crate-shape.md                     beat 8 — router not registry
├── consumer-bridge.md                 beat 9 — end-to-end proof
│
├── wat-edn/                           Rust + wat crate scaffold
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                     orchestration + #[wat_dispatch]
│   │   ├── transcode.rs               :: ↔ . boundary (with tests)
│   │   ├── value.rs                   Value + EdnError
│   │   ├── dispatch.rs                read/write + structural walkers
│   │   └── holon_ast.rs               CustomEdn for HolonAST
│   └── examples/
│       └── consumer.wat               example wat program (TradeSignal)
│
└── wat-edn-clj/                       Clojure library scaffold
    ├── deps.edn
    ├── src/wat_edn_clj/
    │   └── core.clj                   ~250 LOC: readers, writers, helpers
    ├── resources/
    │   └── data_readers.clj           Clojure manifest for code-as-data
    └── examples/
        └── dashboard.clj              example Clojure consumer
```

## What's complete vs stubbed

Both library scaffolds are SCRATCH-quality. They communicate the
shape — file layout, function signatures, key code paths — and
should COMPILE conceptually against wat-rs and Clojure 1.11+.
Verify against the actual wat-rs API surface before lifting into
real crates.

### wat-edn (Rust + wat)

- ✓ `Cargo.toml` with deps shape (edn-format, wat, chrono, uuid)
- ✓ Boundary transcoder with passing unit tests
- ✓ `Value` enum + `EdnError` enum
- ✓ `read_str` / `write_str` entrypoints with `#[wat_dispatch]`
- ✓ Structural decode for struct + parametric (Vec/HashMap/HashSet,
  Option/Result variants — Style A per beat 7)
- ⚠ Enum decode stubbed — needs wat-rs's actual `EnumEntry` layout
- ⚠ HolonAST custom handler is a placeholder — needs holon-rs's
  `canonical_edn_holon` decoder/encoder surfaces wired up
- ⚠ `TypeEntry` / `RustDepsBuilder` / `wat::main_macros` API
  shapes assumed from project memory; verify against wat-rs runtime
- ✓ Example wat consumer (TradeSignal emit + SizeAdjust read)

### wat-edn-clj (Clojure)

- ✓ `deps.edn`
- ✓ Default reader fn for the `wat.*` namespace (one fn handles
  collections, HolonAST variants, sums, unknown-passthrough)
- ✓ Variant wrappers (`some-of`/`none-of`/`ok-of`/`err-of`) +
  predicates (`some-variant?` etc.)
- ✓ `tag-as` helper for application types
- ✓ `register-types!` for one-line app-type wiring
- ✓ Read API (`read-str`/`read-stream`/`read-file`)
- ✓ Write API (`write-str`/`print-line!`/`append-file!`)
- ✓ `data_readers.clj` manifest for code-as-data paths
- ✓ Example dashboard

## First-slice acceptance bar

Round-trip the realistic blob (`enterprise.observer/Engram` from
`string-examples.md`, or `TradeSignal` from `consumer-bridge.md`)
through:

1. wat write → EDN file → Clojure read → display
2. Clojure write → EDN file → wat read → consume
3. Byte-equivalence on each leg

When this passes, the bridge is real.

# JSON I/O — the one substrate prerequisite

The MCP envelope is JSON-RPC. The substrate ships EDN read/write
(arc 086, `:wat::edn::read` / `:wat::edn::write`). For 006 to
land, the substrate needs an analogous JSON read/write surface:

- **Read** — parse a JSON-encoded string into a wat Value.
- **Write** — serialize a wat Value as a JSON-encoded string.

This is the only substrate dependency 006 has that 005-wat-pry
doesn't already cover.

## Two paths

### Path A — `wat-json` battery (the parallel to wat-sqlite)

A new workspace crate `crates/wat-json/` mirrors the shape of
existing batteries:

- `register()` — installs `:wat::json::read` and `:wat::json::write`
  Rust shims on the symbol table.
- `wat_sources()` — returns any wat-level helpers (probably
  empty; the primitives are sufficient).

Implementation: wraps a Rust JSON crate (likely `serde_json`,
already common in the workspace). Reads parse to wat Value
structures; writes serialize wat values (handles structs, enums,
parametric types via the SymbolTable's type metadata).

**Pros:**
- Fast (Rust JSON is fast).
- Battle-tested parsing edge cases (numerics, escapes, unicode).
- Composes cleanly with arc 100's battery composition.

**Cons:**
- New Rust crate dependency (serde_json, ~150KB compiled).
- Battery composition complexity — but this is what arc 100 was
  designed for; not really a cost.

### Path B — wat-level JSON parser/writer

Implement JSON in wat itself, in `wat/std/json.wat`. JSON's grammar
is small enough this is feasible:

- Parser: recursive descent over `:String` characters; produces
  wat Values.
- Writer: walks wat Values; emits JSON character sequences.

**Pros:**
- No new Rust dependency.
- The parser/writer is a wat program; demonstrates the substrate
  can host its own protocol implementations; pure dogfooding.
- Lives in the wat-rs source tree as one file.

**Cons:**
- Slower than Rust JSON. Likely ~10-50x slower for typical
  payloads.
- Edge cases in numeric parsing (unicode escapes, float
  precision) are easy to get subtly wrong; less battle-tested.
- The wat parser would be ~300-500 lines of careful character
  manipulation.

## Lean — Path A

Path A wins on practicality. JSON I/O performance matters for
MCP because the agent is making many small calls; per-call
parse latency adds up. Rust JSON is microseconds; wat-level JSON
would be milliseconds. Across a 1000-call debugging session,
that's 1ms vs 1s of cumulative parse time — the difference
between "fast" and "feels slow."

Path B has appeal as a proof-of-concept (the substrate hosts
its own protocol implementations), but the substrate has already
proven this pattern via the EDN read/write being substrate-native
in Rust + the wat-level usage feeling clean. Re-proving for
JSON is unnecessary.

The user's repeating discipline — *the substrate ships what real
callers demand* — applies cleanly here. MCP demands JSON; ship
JSON as a battery; future arcs that need JSON for non-MCP
reasons (HTTP APIs, gRPC bridges, whatever) inherit it.

## What `wat-json` ships

```
:wat::json::read  (input :String) -> :Result<:T, :wat::json::ParseError>
:wat::json::write (value :T)      -> :String
```

Polymorphic in the same way `:wat::edn::read` is — caller
annotates `T`; the parser walks the type metadata to populate
the right wat Value shape; type mismatch surfaces as a
ParseError variant.

Specific Value mappings:

| JSON | Wat Value |
|---|---|
| `null` | `:Option<T>::None` |
| `true` / `false` | `:Bool` |
| number (integer) | `:i64` |
| number (decimal) | `:f64` |
| string | `:String` |
| array | `:Vec<T>` |
| object | `:Struct` (if T is a registered struct) or `:HashMap<String, T>` |

For the MCP use case specifically, the agent's payloads always
have shape `{"msg": "<EDN string>"}` plus standard JSON-RPC
fields. The substrate parses the JSON envelope (~10 fields), then
parses the inner `msg` string as EDN (which it already knows
how to do). Two parse passes per call; both fast.

## Composition with EDN

The substrate's response to an MCP call fundamentally has the
shape:

```
1. Parse JSON-RPC envelope (wat-json::read)
2. Extract `msg` string
3. Parse `msg` as EDN (wat-edn::read)  — this is the agent's wat expression
4. Evaluate the expression (eval-edn! semantics)
5. Serialize the result as EDN (wat-edn::write) — this is the inner result string
6. Wrap in JSON-RPC response envelope (wat-json::write)
7. Send via IOWriter/println
```

Two parse passes (envelope + payload), two serialize passes
(payload + envelope). Each uses its own substrate primitive.
JSON for envelope; EDN for payload. Clean separation.

## Slicing wat-json — its own arc?

Whether `wat-json` ships as part of 006 or as a sibling arc
opens a question. The user has been disciplined about
"primitives that have callers ship as their own arcs." JSON I/O
has one immediate caller (006-wat-mcp); if other callers
surface (HTTP API for a battery, debug log dumps in JSON
format, etc.), wat-json's existence becomes inevitable.

**Lean: `wat-json` ships as its own arc, slice 1 of 006 depends
on it.** Same way `wat-sqlite` is its own crate even though
`wat-telemetry-sqlite` is its primary consumer. JSON support
is general enough to deserve its own crate.

The 006 BACKLOG would name this dependency: "Slice 1 unblocked
when arc XXX (wat-json) lands." When the time comes to open
the wat-rs arcs, we open `wat-json` first, then `wat-mcp`. Two
arcs; one direction; no circular dependency.

## What if wat-json is delayed?

If JSON support is genuinely far off, the slice 1 of 006 could
hard-code MCP envelope parsing in a small wat-level JSON parser
that handles the specific JSON-RPC shape (which is simpler than
arbitrary JSON — known fields, known nesting depth, no schema
evolution). This unblocks 006 without committing to a general
JSON solution.

But this is the wrong trade. Hard-coding envelope parsing means
every JSON consumer has to re-derive it. The right move is to
ship `wat-json` first, even if the work is a bit more upfront.

## Acceptance bar for the JSON dependency

Before opening 006 as a real wat-rs arc:

- `wat-json` battery exists.
- `:wat::json::read :String -> :HashMap<String, ?>` works.
- `:wat::json::write :HashMap<String, ?> -> :String` works.
- Polymorphic typed read works for at least the JSON-RPC
  envelope shape.
- Round-trip: read then write produces equivalent JSON
  (modulo whitespace).
- ~50 lines of wat-tests covering the corner cases (escapes,
  unicode, floats, deep nesting, arrays of objects).

That's the gate. Once it's clear, slice 1 of 006 can land.

## Why this dependency is actually a feature

Tracking JSON I/O as its own arc forces a clean shape: when
later arcs need JSON (HTTP servers, debug output, log
formatters, etc.), they inherit the same `wat-json` battery
006 used. The substrate accumulates protocol primitives as
discrete batteries; consumers compose them.

The discipline matches arc 103's pipe protocol: one shape, many
transports. JSON is one envelope; EDN is the substrate's
native; both are composable; consumers pick what fits.

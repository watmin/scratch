# The collapse — JSON is envelope, EDN is payload

The first sketch of MCP-as-tool-surface walked through:

1. Walk the SymbolTable; emit JSON Schema for each function's
   typed signature.
2. Publish each function as its own MCP tool with the schema.
3. Agent calls a tool with JSON params; the substrate transcodes
   JSON → wat values via the schema.
4. Substrate evaluates; transcodes wat values → JSON for the
   response.
5. Maintain the schema list as new defines / batteries land.

The user collapsed all of this with one sentence:

> "i think... the JSON rpc.. is just a thin wrapper... the input
> object would be something like '{\"msg\":\":some-edn-form\"}'"

The collapse: JSON-RPC carries the request/response envelope MCP
demands. The actual payload is **wat source as a string** inside
a single JSON field. The substrate doesn't translate JSON types
to wat types; the agent writes wat directly; the substrate
evaluates wat directly; results come back as wat rendered to EDN
inside a JSON string field.

## What goes away

- **JSON Schema generation.** No SymbolTable walk. No type
  expression encoding. Wat's parametric types, enum variants
  with payloads, `:Result<T,E>`, `:Option<T>`, struct shapes —
  none of these need awkward JSON Schema cousins.
- **Per-function tool registration.** No `tools/list` returning
  N items. Constant tool count regardless of how many wat
  functions the entry exposes.
- **Stale tool lists.** Adding a battery doesn't republish the
  tool surface. The same one tool's reach grows with the
  SymbolTable, transparent to the agent.
- **The mismatch at the type boundary.** Wat's type system stays
  native. The agent learns wat types; the substrate doesn't lie
  about them.
- **Translation latency.** No transcoding pass at every call.
  The substrate parses EDN (already shipped); evaluates;
  serializes EDN (already shipped). One round-trip.
- **Two systems of truth.** Without JSON Schema generation,
  there's no document the substrate publishes that could drift
  from the SymbolTable. The substrate IS the truth; the agent
  reads it via introspection calls.

## What stays

- **The MCP envelope.** JSON-RPC 2.0 over stdio. Request /
  response / notification framing. `initialize` handshake.
  Cancellation. Progress reporting. Standard MCP machinery the
  agent expects.
- **The substrate's evaluation discipline.** Constrained eval —
  no define/defmacro/struct/enum/typealias/load (FOUNDATION
  663). Capacity-mode `:error`. Type-checked function calls.
  All of arc 102's polymorphic eval semantics.
- **The freeze invariant.** Agent is a caller through the
  surface; cannot reach back. Same hologram property the human
  pry user sees.

## What the agent does instead of reading JSON Schema

The agent uses wat itself for discovery:

```
agent calls   wat-eval {"msg": "(:wat::pry::ls)"}
agent reads   "[:wat::core::*, :wat::std::*, :wat::sqlite::*, ...]"

agent calls   wat-eval {"msg": "(:wat::pry::ls :trading)"}
agent reads   "[:trading::types::Candle, :trading::compute-decision, ...]"

agent calls   wat-eval {"msg": "(:wat::pry::show :trading::compute-decision)"}
agent reads   "fn :trading::compute-decision (candle :Candle) -> :Action ..."
```

The substrate's pry-shipped introspection (slice 1 of 005) IS
the API documentation surface. The agent reads docs by calling
into the substrate. **The substrate's introspection is the
contract.** No second document; nothing to keep in sync.

This is the same discipline arc 103's HOLOGRAM.md named: the
binary is the surface; what the binary supports is queryable
through the binary itself; nothing outside the binary needs to
know what it supports. MCP tools/list traditionally announces
capabilities; wat-mcp's `tools/list` announces ONE capability
(`wat-eval`) and points the agent at the substrate's own
introspection for everything beyond that.

## The user's instinct in lineage

The user has been making this move across the project for
weeks. Each instance follows the same shape — collapse a
schema-shaped translation layer into a single payload-carrying
field that defers to the substrate's own type system:

- **Arc 003 (edn-typed-wire):** EDN tags carry wat types; no
  separate type registry on the wire. Tag IS the type.
- **Arc 086 (edn-roundtrip):** Substrate writes what substrate
  can read. No external schema; the EDN bytes are the contract.
- **Arc 103 (kernel-spawn):** EDN+newline IS the protocol; the
  OS pipe IS the bounded channel; no adapter layer.
- **Arc 104 (wat-cli-fork):** Cli is sovereignty + containment;
  doesn't translate user code, just hosts and forks.

Each move trims a translation layer. Each move surfaces what
the substrate ALREADY does cleanly. Today the same instinct
applies to MCP: JSON-RPC is the envelope MCP demands; the
substrate doesn't have to play schema games to be useful.

The collapse is the design. Everything else in 006-wat-mcp is
how to ship the consequence.

## Concrete example — one call, one composition

What the agent sends for a multi-step computation:

```json
{
  "method": "tools/call",
  "params": {
    "name": "wat-eval",
    "arguments": {
      "msg": "(:wat::core::let* (((c :wat::core::String) (:wat::io::read-file \"/tmp/demo.db\")) ((handle :wat::sqlite::ReadHandle) (:wat::sqlite::open-readonly c))) (:wat::std::stream::collect (:wat::telemetry::sqlite/stream-logs handle (:wat::core::vec :wat::telemetry::TimeConstraint (:wat::telemetry::since (:wat::time::hours-ago 1))))))"
    }
  }
}
```

That's ONE MCP call. It opens a sqlite database, streams
filtered logs from the last hour, collects the result. Returns
the collected events as an EDN-encoded value.

The per-function-tools alternative would be:
1. `tools/call wat-io-read-file path=...` → string
2. `tools/call wat-sqlite-open-readonly path=...` → handle
3. `tools/call wat-time-hours-ago n=1` → duration
4. `tools/call wat-telemetry-since ts=...` → constraint
5. `tools/call wat-telemetry-vec items=[...]` → vec
6. `tools/call wat-telemetry-sqlite-stream-logs handle=... cs=...` → stream
7. `tools/call wat-stream-collect s=...` → vec

Seven calls. Six intermediate results sitting in the agent's
context taking tokens. Each step's failure mode handled
separately. Each step's typed result transcoded twice (out → in
JSON).

One call vs seven. The composition is the substrate's; the
envelope is just transport. **The agent is using wat as its
Lisp; MCP is the dial-tone.**

## Why this matters past 006

The collapse generalizes. Any future "wat as a server" pattern
inherits the same shape:

- Wat as gRPC server? Same — one method `eval(string) -> string`;
  EDN inside the protobuf string field.
- Wat as HTTP API? Same — POST `/eval` with EDN in the body;
  EDN response.
- Wat as Lambda function? Same — invoke with EDN in `payload`;
  EDN response.
- Wat as Slack bot? Same — message text IS the EDN; bot replies
  with rendered EDN.

The substrate was already doing this for arc 103's pipe
protocol. The collapse names it explicitly: **wat speaks wat
over any envelope.** Adapter layers are unhelpful sugar.

The 006 scratch is the first instance of this principle applied
past the substrate's native pipe transport. The shape stays
applicable for every future "expose wat to consumer X" arc that
surfaces.

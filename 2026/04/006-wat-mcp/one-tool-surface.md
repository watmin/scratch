# One tool — `wat-eval` and discovery via wat itself

The MCP server publishes a single tool. That tool takes a string
parameter (an EDN-encoded wat expression) and returns a string
result (the EDN-encoded value the expression evaluated to). The
agent uses this one tool for everything — discovery, invocation,
inspection, composition.

## The tool definition

Wat-side schema (for the agent's `tools/list` response):

```scheme
(:wat::core::struct :wat::mcp::tool-wat-eval
  (name        :String)        ;; "wat-eval"
  (description :String)        ;; "Evaluate a wat expression. Input is an EDN-encoded form; output is the EDN-encoded result or error."
  (inputSchema :wat::edn::NoTag)) ;; JSON Schema with one required string property "msg"
```

The `inputSchema` is the standard MCP way to declare JSON-RPC
parameter shapes. For wat-eval, it's the simplest possible
schema:

```json
{
  "type": "object",
  "properties": {
    "msg": {
      "type": "string",
      "description": "An EDN-encoded wat expression to evaluate against the frozen world."
    }
  },
  "required": ["msg"]
}
```

That's the entire JSON Schema the substrate publishes. The
agent learns wat from the substrate's introspection; the
substrate doesn't translate between type systems.

## What the agent does on connect

1. **`initialize`** handshake — substrate responds with server
   info + capabilities (`tools` capability supported,
   `notifications/pause/break` supported, etc.).

2. **`tools/list`** — substrate responds with a one-element list
   containing the `wat-eval` tool definition above. The agent
   sees: "this server has one tool, `wat-eval`, which takes a
   string `msg` and returns a string."

3. **`tools/call`** — agent's first call typically discovers the
   surface:

```json
{
  "method": "tools/call",
  "params": {
    "name": "wat-eval",
    "arguments": {"msg": "(:wat::pause::ls)"}
  }
}
```

Substrate evaluates `(:wat::pause::ls)` (the introspection primitive
shipped by 005-wat-pause slice 1), serializes the result as EDN,
returns:

```json
{
  "content": [
    {"type": "text", "text": "[:wat::core::define :wat::core::let* :wat::core::lambda ... :user::main]"}
  ]
}
```

The agent now has the full list of callable symbols. From there
it can `wat-eval (:wat::pause::show :symbol)` to read source,
`wat-eval (:trading::compute-decision candle)` to invoke, or
build any composition in one expression.

## Why one tool

Three reasons the single-tool shape is right:

### 1. Composition is what makes wat valuable

Wat is a Lisp. The whole point of a Lisp is that expressions
compose — `(f (g (h x)))` is one form, not three function
invocations. Per-function MCP tools force the agent to make N
sequential tool calls for a composition, with intermediate
results sitting in context. That's a massive token cost on the
agent side AND it loses what makes wat expressive in the first
place.

One eval tool means one round-trip per composition. The agent
sends a let-bound multi-step computation; the substrate
evaluates it as one unit; one result comes back.

### 2. Tool surface stays stable as the SymbolTable grows

The `tools/list` response is fixed at `[wat-eval]`. Adding a
battery, loading a `.wat` file with new defines, defining a
struct mid-session — none of these change the tool surface
the agent sees. The agent's tool list never goes stale; never
needs reloading; never lies about what's available.

The agent's discovery via `(:wat::pause::ls)` always reflects
current state. Agent re-queries on demand if it suspects the
SymbolTable changed (it didn't; the freeze invariant prevents
runtime additions).

### 3. JSON Schema doesn't fit wat types cleanly

Wat's type system has shapes JSON Schema struggles with:

- **Parametric types** — `:Vec<:trading::Candle>` requires
  inline schema generation per generic instance. Possible but
  ugly.
- **Sum types with payloads** — `:Result<:Trade, :ParseError>`
  encodes as `oneOf` with discriminator hints. Each enum
  variant becomes a schema fragment. Verbose, error-prone.
- **Recursive types** — `(struct :Tree (left :Option<:Tree>)
  (right :Option<:Tree>) (val :i64))` requires
  `$ref`-resolution in the schema. JSON Schema supports it;
  most JSON Schema consumers don't fully.
- **Type expressions parameterized by integers** — wat's
  `:Therm<-100, 100>` (if it lands) has no JSON Schema cousin.

By keeping wat types behind the EDN-string boundary, the
substrate avoids all of these. Agents that want to reason about
types call `(:wat::pause::show :symbol)` which returns the wat
source verbatim; they get the type expression in its native
form.

## The optional second tool — `wat-eval-stream`

Slice 2 likely adds a second tool for break-mode interactions
where the eval can pause and the agent needs to make
intermediate calls before resuming. Two shapes considered:

**Shape A — single tool, both modes:**

`wat-eval` returns a special "paused" content type when a break
fires:

```json
{
  "content": [
    {"type": "text", "text": "(:break-info ...)"},
    {"type": "resource", "uri": "pause://session/abc123/continue"}
  ]
}
```

Agent makes follow-up calls referencing the session URI. Slightly
awkward; the resource URI shape is stretched.

**Shape B — separate `wat-eval-stream` tool:**

`wat-eval-stream` returns immediately with a pending-call ID. Agent
calls additional tools (`wat-pause-inspect`, `wat-pause-continue`)
referencing the ID. The substrate keeps the original eval
suspended.

This is closer to MCP's notification + cancellation patterns. It
also keeps `wat-eval` simple — the basic eval tool always returns
a value or an error, never a pending state. Break-mode is opt-in
via the streaming tool.

**Lean: Shape B.** Cleaner separation; matches MCP idiom; basic
`wat-eval` remains 90% of usage and stays simple.

See `break-as-notification.md` for the full break-mode
protocol.

## What discovery looks like in practice

A typical agent session against a wat-mcp server:

```
agent → wat-eval (:wat::pause::ls :trading)
agent ← [:trading::types::Candle, :trading::types::Direction,
         :trading::compute-decision, :trading::rsi, ...]

agent → wat-eval (:wat::pause::show :trading::compute-decision)
agent ← "fn :trading::compute-decision (candle :Candle) -> :Action
         (let* (((rsi :f64) (:trading::rsi candle))
                ((vol :f64) (:trading::vol candle))
                ...)
           (:trading::action regime rsi vol))"

agent → wat-eval (:wat::pause::show :trading::types::Candle)
agent ← "struct :trading::types::Candle
           open    :f64
           high    :f64
           low     :f64
           close   :f64
           volume  :f64"

agent → wat-eval (:trading::compute-decision
                   (:trading::types::Candle/new 50000.0 50500.0 49500.0 50250.0 1.5))
agent ← :Action::Buy
```

Five MCP calls. Agent learned the trading namespace's shape,
inspected the function's source, inspected the Candle struct,
constructed a Candle, called the function, got the result. Each
call was one wat expression in one MCP envelope.

## The agent learns wat once

The structural payoff: the agent doesn't need per-server
training. Once it learns wat (from documentation or by reading
the BOOK), every wat-mcp server is reachable through the same
discipline:

- `(:wat::pause::ls :prefix)` — list available symbols.
- `(:wat::pause::show :symbol)` — read source / signature.
- `(:wat::pause::completions :prefix)` — narrow the surface.
- `(<the symbol> args...)` — invoke.
- `(:wat::core::let* ...)` — compose.

Five idioms, applicable to every wat-mcp server the agent
connects to. No per-server tool documentation; no API
versioning at the MCP layer (versioning happens at the
substrate's wat language level, where it should).

This is the inverse of the typical MCP server design (each
server has its own bespoke tool surface the agent must learn
per-deployment). Wat-mcp shifts the per-server-learning to
"learn the program's domain through wat introspection," which
the agent does once per session via the same idioms.

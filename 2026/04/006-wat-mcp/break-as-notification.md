# Break as notification — agent-driven debugging

The killer use case the user named:

> "give the agent a way to run a program /and/ live debug it"

`(:wat::pause::break)` already exists as 005's substrate primitive
(slice 2). It captures Environment + CALL_STACK and runs an
inline pause loop reading from stdin. For a human, that loop reads
typed lines from a terminal. For an agent, the same shape
becomes: emit a JSON-RPC notification, suspend the call, expose
inspection tools, resume on agent command.

The substrate doesn't grow a new break primitive. The MCP-mode
break is the same `:wat::pause::break-with-stdio` primitive with a
flag indicating the loop should speak JSON-RPC to its stdin /
stdout instead of plain EDN. Or — cleaner — the MCP server
intercepts the break event before `:wat::pause::serve` enters its
inner loop, and runs the JSON-RPC variant of the loop instead.

## The protocol

### Step 1 — agent invokes a wat function via `wat-eval-stream`

```json
{
  "method": "tools/call",
  "params": {
    "name": "wat-eval-stream",
    "arguments": {
      "msg": "(:trading::compute-decision (:trading::types::Candle/new 50000.0 50500.0 49500.0 50250.0 1.5))"
    }
  }
}
```

The agent uses `wat-eval-stream` (not `wat-eval`) because it
expects this call may pause — the function being called has
`(:wat::pause::break)` in its body somewhere, and the agent wants
to handle the pause interactively.

### Step 2 — substrate evaluates until break fires

The substrate runs `:trading::compute-decision`. Let-bindings
fire for `rsi`, `vol`, `regime`. Then evaluation hits
`(:wat::pause::break)`.

The break primitive captures:
- The current Environment — `candle`, `rsi`, `vol`, `regime`
  plus parents.
- The CALL_STACK with each frame's location + Environment.
- A unique session ID for this paused call.

Substrate stores these in a process-global registry keyed by the
session ID. The original `wat-eval-stream` call DOES NOT return
yet.

### Step 3 — substrate emits a notification

JSON-RPC notification (no `id` field; one-way to client):

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/pause/break",
  "params": {
    "session": "pause-7f3a2c1e",
    "msg": "(:break-info :file \"trade.wat\" :line 42 :col 7 :function :trading::compute-decision :env-keys [:candle :rsi :vol :regime])"
  }
}
```

The `msg` field is EDN-encoded just like any other wat-mcp
payload. Agent parses the EDN; learns there's a paused session
at trade.wat:42:7 in compute-decision with four captured locals.

### Step 4 — agent inspects via more `wat-eval` calls

Each follow-up call references the session ID:

```json
{
  "method": "tools/call",
  "params": {
    "name": "wat-eval",
    "arguments": {
      "msg": "(:wat::pause::env)",
      "session": "pause-7f3a2c1e"
    }
  }
}
```

The substrate routes the eval to the captured Environment of
that session. Result comes back as an EDN-encoded value:

```json
{"content": [{"type": "text",
              "text": "{:candle <Candle> :rsi 0.6234 :vol 0.0083 :regime :Regime::Trending}"}]}
```

Subsequent calls walk frames, evaluate test expressions in the
captured scope, etc.:

```
agent → wat-eval session=pause-7f3a2c1e msg="rsi"
agent ← 0.6234

agent → wat-eval session=pause-7f3a2c1e msg="(:trading::action regime rsi vol)"
agent ← :Action::Buy

agent → wat-eval session=pause-7f3a2c1e msg="(:wat::pause::frames)"
agent ← [<frame trade.wat:42 compute-decision>, <frame trade.wat:120 :user::main>]

agent → wat-eval session=pause-7f3a2c1e msg="(:wat::pause::up)"
agent ← :() ;; eval scope now points at :user::main's frame

agent → wat-eval session=pause-7f3a2c1e msg="stdin"
agent ← <IOReader> ;; we're in main's scope; stdin is visible here
```

### Step 5 — agent resumes

```json
{
  "method": "tools/call",
  "params": {
    "name": "wat-eval",
    "arguments": {
      "msg": "(:wat::pause::continue)",
      "session": "pause-7f3a2c1e"
    }
  }
}
```

The substrate resumes the suspended `:trading::compute-decision`
call. The break primitive returns `:()`; the rest of the
function body evaluates against the same Environment (or a
modified one — see "early-return" below).

### Step 6 — original `wat-eval-stream` call returns

When the suspended function finishes evaluating, the substrate
serializes its return value as EDN and sends the response to
the original `wat-eval-stream` call:

```json
{
  "id": "<original-request-id>",
  "result": {
    "content": [{"type": "text", "text": ":Action::Buy"}]
  }
}
```

The agent has the function's actual return value. The break
session is closed; subsequent `wat-eval` calls with that session
ID return an error.

## What this gives the agent

**Live mid-program inspection without modifying the source.**
The wat program has `(:wat::pause::break)` in it; the agent
connects, calls a function that hits the break, freezes execution
at that point, inspects everything, makes hypothesis calls
(running other functions with the captured scope), then resumes
to see the actual result.

This is what `binding.pry` gives a Ruby developer at a terminal,
applied to an agent at a JSON-RPC connection. **The agent
becomes the pause frontend.** Same primitive; different consumer;
different envelope.

## Special operations during a break

Some pause operations are powerful enough to deserve naming
explicitly:

### `(:wat::pause::override-return <expression>)`

Force the suspended function to return a specific value
instead of resuming normally. Example:

```
agent → wat-eval session=pause-7f3a2c1e msg="(:wat::pause::override-return :Action::Sell)"
```

The substrate skips the rest of the function body and immediately
returns `:Action::Sell` from `:trading::compute-decision`.
Caller of compute-decision sees this as the function's actual
return.

This is what makes agent-driven debugging more powerful than a
human at pause — the agent can hypothesize **"what if this
function had returned Sell instead of Buy?"** and run downstream
code against that hypothetical without modifying source. Test
behavior under counterfactual returns; iterate; resume the
original or override different functions in subsequent breaks.

### `(:wat::pause::eval-in-frame <frame-idx> <expression>)`

Without walking, evaluate an expression in any captured frame's
scope by index:

```
agent → wat-eval session=pause-7f3a2c1e msg="(:wat::pause::eval-in-frame 0 (:trading::types::Candle/open candle))"
```

Frame 0 is the bottom of the stack (entry-point). Agent can
read locals from any depth in one call without walking.

### `(:wat::pause::set-binding! <symbol> <value>)`

**Forbidden by the freeze invariant** — the user explicitly
named "rust being frozen is a blessing, not a curse." Pause break
captures Environment immutably; subsequent calls in the
captured scope can READ the bindings but not REBIND them.

This means agent-driven debugging is "explore + override
returns" but not "rewrite locals and continue." Honest about
the substrate's discipline; agents can still test counterfactuals
via `override-return` and via composing fresh expressions
against the captured scope.

## Multi-session safety

The session ID is the discriminator. Multiple breaks can be
active simultaneously (e.g., if the program spawns threads that
each hit their own breaks). The substrate's session registry
maps `session-id → (Environment, CALL_STACK, suspended-call)`.
Agent's `wat-eval` calls always include the session ID.

If the agent calls `wat-eval` without a session ID, it evaluates
against the top-level frozen world (no captured break scope) —
this is the bare `wat-eval` we documented in
`one-tool-surface.md`. Both modes coexist.

## What the agent doesn't have to learn

The break protocol is conceptually simple:

1. Get a notification with a session ID.
2. Make `wat-eval` calls with that session ID until ready.
3. Send `(:wat::pause::continue)` (or `override-return`) to
   release.
4. Original call's response arrives.

Three rules. The agent doesn't need a dedicated debugger client
library; it just makes more MCP calls. The substrate handles
the suspended-call mechanics.

This composes with the agent's own reasoning loop — the agent
treats break notifications as "interesting events worth
investigating" and walks them via the same MCP machinery it
uses for normal calls. **The debugger IS the API.**

## Why this is structurally novel

Most debuggers couple the debugger client to the runtime via a
specific protocol (gdbserver, DAP, JDWP, Chrome DevTools, etc.).
Each is a custom protocol the client must implement.

Wat-mcp's debugger protocol is the same protocol the agent uses
for everything else — `tools/call wat-eval`. The "debugger" is
"call eval in the captured scope of a paused call." The protocol
needs no extension; the substrate's introspection (already
shipped via 005-wat-pause slice 1) covers what dedicated debug
protocols expose explicitly.

Three things compose to make this trivial:

1. **EDN+newline as the wire format** (arc 103) — already
   carries arbitrary wat values. Break notifications fit
   naturally.
2. **Frozen world** — the agent's inspection is honest;
   captured Environment doesn't drift.
3. **One-tool-surface** — the debugger doesn't need new tools;
   it's the same `wat-eval` with a session ID.

The substrate has been building toward this without anyone
naming "live agent debugging" as the goal. Tonight named it.

## Cost

At the substrate level, slice 2 of 006:

- A session registry — process-global `HashMap<String,
  PausedSession>` with each PausedSession holding the captured
  state. Maybe ~100 lines.
- Modified `wat-eval` dispatcher — when a `session` parameter
  is present, route to the captured Environment instead of the
  top-level world. ~50 lines.
- The notification emitter — when break fires inside an MCP
  eval, emit JSON-RPC notification, suspend the original call.
  ~80 lines.
- `:wat::pause::override-return` — substrate primitive to
  force-return from a paused frame. ~50 lines.
- `:wat::pause::eval-in-frame` — substrate primitive for
  cross-frame eval. ~40 lines.

~320 lines of substrate Rust. Plus ~150 lines of wat in
`wat/std/mcp.wat` for the JSON-RPC dispatch around break
events.

Compare to writing a custom debugger client + server: maybe
~5,000 lines for a basic implementation. The substrate's
existing arcs do the heavy lifting; 006 is the packaging.

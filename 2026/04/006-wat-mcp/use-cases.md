# Use cases — what wat-mcp unlocks

Five concrete shapes the substrate enables once 006 lands. Each
is something that's hard or impossible to build today and that
becomes natural with `wat --mcp` + the agent-driven break
protocol.

## 1. Agent-driven debugging

The killer use case the user named. A wat program contains
`(:wat::pry::break)` at suspect points. The agent connects via
MCP, calls the program's functions, hits the break, inspects
captured Environment, hypothesizes counterfactuals via
`override-return`, resumes, iterates.

Concrete shape:

```
agent (Claude in a debugging conversation)
  │
  ├─ wat-eval (:wat::pry::ls :trading)               ← discover
  ├─ wat-eval (:wat::pry::show :trading::compute-decision)  ← read source
  │
  ├─ wat-eval-stream (:trading::compute-decision <test-candle>)  ← invoke
  │   │
  │   ↓ (:wat::pry::break) fires inside compute-decision
  │   ← notification: paused at trade.wat:42:7
  │
  ├─ wat-eval session=X (:wat::pry::env)             ← inspect locals
  ├─ wat-eval session=X (:trading::action regime rsi vol)  ← test downstream
  ├─ wat-eval session=X (:wat::pry::override-return :Action::Sell)  ← counterfactual
  │
  └─ original wat-eval-stream returns :Action::Sell  ← see effect on caller
```

The agent goes from "this function makes bad decisions" to
"here's the line where the regime classification flips" via
break-mode inspection. No source modification; no debugger
client; no bespoke protocol. Same MCP envelope.

The trading lab's BOOK Chapter 71 framing — *cache is
consumption; every walker eats prior walkers* — extends to:
**the agent eats the program's locals.** The break primitive
exposes the substrate's runtime state to a different consumer.

## 2. Agent-driven library exploration

A wat library exposes its functions; the agent learns the
surface by experimenting.

Concrete: a user wants to understand the `wat-telemetry`
crate's surface without reading the docs. They run:

```
$ wat --mcp /path/to/telemetry-only.wat   ; an entry that just loads :wat::telemetry::*
```

The agent connects, calls `(:wat::pry::ls :wat::telemetry)`,
sees the namespace's shape, reads `(:wat::pry::show
:wat::telemetry::Event)` to learn the Event enum, constructs
test events, calls `(:wat::telemetry::log/info ...)` to test the
emit path, observes results. **The agent builds a mental model
of the library by interactive use.**

This is the inverse of "agent reads documentation": the agent
USES the library and the substrate's introspection IS the
documentation. Faster for unfamiliar territory; more honest
because the substrate can't lie about what it ships.

## 3. Multi-program orchestration

The agent connects to multiple wat programs simultaneously.
Each is its own hologram; the agent is the conductor.

Concrete shape: an agent debugging a distributed trading system
might run:

- `wat --mcp trader.wat` (port stdio A)
- `wat --mcp telemetry-server.wat` (port stdio B)
- `wat --mcp risk-monitor.wat` (port stdio C)

Three MCP connections. Three holograms. The agent reads the
trader's pending orders via A, queries the telemetry-server's
recent metrics via B, evaluates the risk-monitor's current
exposure via C, then composes a hypothesis ("the trader's
sizing would cause a margin call") by passing data between the
three programs via the agent's own context.

This is what BOOK Chapter 55's "two oracles" framing suggested
at the architectural level (cache + reckoner). With MCP, the
agent CONNECTS the oracles. **The agent is the bridge that the
substrate left abstract.**

Multi-program orchestration is hard today because each tool
needs a custom MCP server. Wat-mcp makes EVERY wat program an
MCP server for free. Composing N programs is just N
connections.

## 4. Self-hosted agent loops

A wat program calls Claude API (or any LLM API) and uses other
wat programs as the agent's tools.

Concrete: imagine a `wat/std/agent.wat` battery that ships
`:wat::agent::run-loop` taking a prompt and a list of MCP
endpoints. The function:

1. Spawns each MCP endpoint as a child wat process.
2. Calls Claude API with the prompt and the endpoints' tool
   surfaces.
3. Routes Claude's tool calls to the right child's MCP
   connection.
4. Feeds results back to Claude.
5. Returns the final assistant response.

This composes the substrate's own primitives — `spawn-program`
(arc 103), the EDN+newline protocol, Rust HTTP shims for the
Claude API call — into an agent runtime expressed in wat. The
trading lab's "many thinkers" pattern (BOOK Chapter 55's
bridge) becomes operational: **N wat programs, one orchestrator,
one prompt, one terminal answer.**

The recursion is structural. A wat program that calls Claude
that uses wat programs that may themselves call Claude. The
hologram model handles this naturally — each level is its own
hologram with its own freeze; the protocols stack.

## 5. Production interactive control planes

A long-running wat program (a service, a daemon, a trader) can
run with `--mcp --tcp 9999` (slice 5 of pry's plan; same
mechanism) and accept agent connections from a remote operator's
terminal.

Concrete: a deployed trader running on a server has bugs that
only manifest in production. The operator (or an LLM-driven
ops agent) connects to it via:

```
$ wat-pry --attach tcp://prod-trader:9999
```

The connection establishes; the agent makes `wat-eval` calls
against the live program's frozen world. It can read current
state, query telemetry, walk frames if break points are hit.
**Production debugging without restarting the program.**

The freeze invariant is what makes this safe. The connecting
agent CAN'T modify the program — no defines, no redefinitions,
no reloading. It can only inspect, call, and override-return on
break sessions. The production binary's behavior is bounded by
what was frozen at startup; the agent operates within that
boundary.

This is what BOOK Chapter 67 — *The Spell* — pointed at. The
spell is the move from local memoization to networked
proof-of-computation registry. With wat-mcp, the spell extends
to the agent layer: **one wat program; one agent; one secure
channel; full inspection without modification rights.**

## Why these are hard today

Each use case requires capabilities the wat substrate has but
agents can't currently access:

- **Live debugging** requires Environment capture +
  resume-after-inspection — wat has it; no protocol exposes it.
- **Library exploration** requires typed introspection of an
  arbitrary program's surface — wat has it via SymbolTable; no
  protocol exposes it.
- **Multi-program orchestration** requires uniform protocol
  across heterogeneous programs — wat has it via EDN+newline;
  every program would need a bespoke MCP server today.
- **Self-hosted agent loops** require recursive composition —
  wat has it via spawn-program / fork-program-ast; no agent
  framework consumes it.
- **Production control planes** require typed remote eval with
  bounded mutation — wat has it via constrained-eval; no
  remote-eval protocol exposes it under those bounds.

006-wat-mcp is the small layer that exposes all five. The
substrate's existing arcs do the work; MCP is the dial-tone.

## What this changes for the trading lab

The lab's BOOK chapters 49 / 51 / 54 / 55 / 65 / 67 keep
pointing at the same pattern: programs as data, coordinates on
a sphere, holograms nesting, the spell crossing transports. Each
chapter named a structural property of the substrate; each
unlocked future capability without naming what.

006-wat-mcp is the recognition that **the agent is the consumer
the substrate has been building for.** Every prior arc — pipe
protocol, hologram nesting, polymorphic eval, frozen world,
constrained mutation, FQDN explicit naming, struct/enum
introspection — composes into "the agent has wat as its Lisp
through one MCP tool." The lab benefits naturally; the user can
debug the trader, query the telemetry, orchestrate the
back-test, all through one agent connection.

The lab's BOOK doesn't name agents as a primary consumer
anywhere. After 006 ships, the agent is one more node in the
hologram cloud — speaking the same protocol every other node
speaks, bounded by the same freeze invariant every other
caller respects.

## What this DOESN'T do

Worth being explicit:

- **Doesn't make wat into an agent framework.** The substrate
  exposes wat to agents; the substrate doesn't replace agent
  frameworks like LangChain, AutoGPT, etc. Those exist at a
  different layer (orchestration, memory, planning); wat-mcp
  is the tool-surface those frameworks could compose with.
- **Doesn't introduce mutability for agents.** The freeze
  invariant holds for agents the same way it holds for humans.
  Agents can call functions, inspect state, override-return on
  breaks — but cannot define / redefine / load / mutate.
- **Doesn't replace MCP.** The substrate hosts MCP; it doesn't
  invent MCP or compete with it. Wat-mcp is one MCP server
  among many.
- **Doesn't auto-publish wat programs.** A binary built without
  `--mcp` does NOT expose an agent-callable surface. Gating
  is structural (see `gating.md`).

The shape is small and contained. The implications are large
because wat is large.

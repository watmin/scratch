# Integration contracts — where pry surfaces become MCP surfaces

The user's locked alignment: **MCP interfaces with pry; pry
exists for interactive exploration; MCP is how the agent does
it; pry directly is how operators do it.**

This document names the explicit handoff contracts at each
integration point. Each contract is a place where a pry
surface is consumed by an MCP surface. The contract specifies
the shape: what pry provides, what MCP wraps, what the
agreement is.

The principle: **MCP doesn't reimplement pry. MCP wraps it.**
Every pry primitive has exactly one implementation in the
substrate. That implementation is callable identically from
pry-mode (rustyline) and MCP-mode (JSON-RPC) consumers. The
only thing that varies is the wire format around the call.

## Contract 1 — The break primitive's two behaviors

### What pry provides

`:wat::pry::break-with-stdio (in :IOReader, out :IOWriter,
err :IOWriter) -> :()` — the substrate primitive that:

1. Captures the current `Environment` (Arc clone).
2. Captures `CALL_STACK` snapshot (each FrameInfo with its env).
3. Calls a `break-handler` function that owns the input/output
   loop.
4. Returns `:()` when the handler signals resume.

### What MCP wraps

The substrate's break-handler is **overridable at battery
registration**. Pry battery installs the default handler:
"read line from `in`; parse EDN; eval against captured env;
write result to `out`; recognize `:continue` / `:exit` /
etc.; loop."

MCP battery overrides the handler with: "register session in
process-global registry; emit JSON-RPC notification on stdout;
block on session's resume signal channel; on resume, return."

### The contract

| Property | Required of any handler |
|---|---|
| Captures the env-and-frames passed by `break-with-stdio` | Yes |
| Uses the captured env as eval context for any expressions | Yes |
| Returns when resume signal arrives (whatever its shape) | Yes |
| Honors constrained-eval (no define / redefine / load) | Yes |
| Doesn't leak captured env outside the handler's lifetime | Yes |

The handler IS the integration point. Pry mode and MCP mode
each provide a handler that satisfies the contract. The
substrate primitive doesn't know which mode is active; it
calls the registered handler and returns when the handler
returns.

### Why this is the right shape

The user's framing — "MCP interfaces with pry" — means MCP
wraps pry's primitives without changing them. The break
primitive's CAPTURE behavior is identical in both modes; only
the LOOP behavior (read from terminal vs read from JSON-RPC)
differs. Splitting capture from loop into the
primitive-plus-handler shape makes this concrete.

## Contract 2 — Inspection primitives are pry's, called via MCP's wat-eval

### What pry provides

The full inspection primitive set:
- `:wat::pry::ls (prefix :Option<String>) -> :Vec<:Symbol>`
- `:wat::pry::show (sym :Symbol) -> :Option<:String>`
- `:wat::pry::completions (prefix :String) -> :Vec<:String>`
- `:wat::pry::where () -> :Option<:Location>`
- `:wat::pry::frames () -> :Vec<:Frame>`
- `:wat::pry::up () -> :()`
- `:wat::pry::down () -> :()`
- `:wat::pry::eval-in-frame (frame-idx :i64, expr :String) -> :T`
- `:wat::pry::override-return (value :T) -> :()`
- `:wat::pry::continue () -> :()`
- `:wat::pry::last-error () -> :Option<:Failure>`

### What MCP wraps

**Nothing specific.** MCP's `wat-eval` tool just evaluates wat
expressions against the frozen world. Agent calls
`wat-eval (:wat::pry::ls :trading)`; substrate evaluates
`(:wat::pry::ls :trading)`; result comes back as EDN.

### The contract

| Property | Required |
|---|---|
| Pry primitives are callable via `eval-edn!` | Yes |
| Eval honors constrained-eval invariant (the substrate already enforces this) | Yes |
| Agent gets identical results to operator (same primitive; same eval engine) | Yes |
| Pry battery's `register()` makes these symbols visible to the eval-edn! lookup | Yes |

### Why this is the right shape

Pry primitives are just functions. The substrate's
`eval-edn!` already evaluates arbitrary expressions against
the frozen world. MCP's `wat-eval` is `eval-edn!` with a
JSON-RPC envelope. The agent's "inspection capability"
is identical to the operator's — same primitives, different
consumer.

This is the most important integration contract. Most of MCP
is "pass through to eval-edn!" with no special handling.
The agent gets full pry semantics by virtue of being a wat
caller.

## Contract 3 — Session resumption across the wire

### What pry provides

In pry mode, `:wat::pry::continue` is called from inside the
break-handler's loop. It returns from the handler; the break
primitive returns `:()`; execution resumes at the next form
after the break.

### What MCP wraps

In MCP mode, `:wat::pry::continue` is called by the agent via
a follow-up `wat-eval session=X` call. The substrate routes
the eval to the session's captured env; the eval evaluates
`(:wat::pry::continue)`; the substrate signals the suspended
thread's resume channel; the suspended thread wakes; the
break-handler returns; the original `wat-eval-stream` call
returns.

### The contract

| Property | pry mode | MCP mode |
|---|---|---|
| `:wat::pry::continue` symbol | Same | Same |
| Eval scope | The break-handler's loop | The captured session's env |
| Effect | Returns from the handler | Signals the suspended thread |
| Caller's experience | Inline loop ends | Original `wat-eval-stream` returns |

The agent doesn't see the resume mechanism — it just sees
the original tool call's response arrive after `:continue`
is sent. The substrate's session registry coordinates the
suspension and resumption; the wat-level surface stays
identical.

### Why this is the right shape

The agent uses `:wat::pry::continue` the same way the
operator does. The substrate translates the call into the
appropriate resumption semantics (inline-loop return vs
suspended-thread signal) without the agent or operator
needing to think about it. Same primitive; mode-dispatched
behavior.

## Contract 4 — Counterfactual (override-return)

### What pry provides

`:wat::pry::override-return (value :T) -> :()` — substrate
primitive that, when called inside a paused frame, sets the
return value AND triggers resume. The function unwinds with
the override value as its result.

### What MCP wraps

Same primitive; same mechanism. Agent calls
`wat-eval session=X (:wat::pry::override-return
:Action::Sell)`. The substrate routes to the captured frame;
sets the override; triggers resume; the suspended
`wat-eval-stream` returns `:Action::Sell` as the function's
result.

### The contract

| Property | Required |
|---|---|
| Override-return is a pry primitive (not MCP-specific) | Yes — pristine pry includes it |
| Operator can call it from rustyline at a break | Yes |
| Agent can call it via MCP at a break | Yes |
| Effect is identical regardless of consumer | Yes |
| Honors constrained-eval (the override value is just a value; no mutation) | Yes |

### Why this is the right shape

Counterfactual debugging is useful for both operators and
agents. Operators in a break might want to test "what if this
function returned X instead?"; agents want the same. The
primitive is content-agnostic — it doesn't care who called
it; it just sets the override and resumes.

This is the contract that closed the open question from the
006 scratch's "are these primitives MCP-only or pry-pristine?"
The user's answer: pry-pristine. The primitive lives in pry's
surface; MCP just exposes it through the JSON-RPC envelope.

## Contract 5 — The wat-eval tool as the universal gateway

### What pry provides

`:wat::eval-edn!` — the polymorphic eval primitive (arc 102).
Takes a string of EDN-encoded wat source, parses it,
evaluates against the frozen world (or a captured env if
inside a break), returns `:Result<:T, :EvalError>`.

### What MCP wraps

The `wat-eval` MCP tool extracts the `msg` field from
JSON-RPC params, calls `eval-edn!`, serializes the result as
EDN inside the response's `text` content field.

### The contract

| Property | Required |
|---|---|
| Agent can invoke any wat expression via wat-eval | Yes — eval-edn! supports all wat |
| Constrained-eval applies (no define / load / redefine) | Yes — eval-edn! enforces it |
| Errors return as JSON-RPC error responses for protocol-level issues; as EDN-encoded `:Result::Err` for wat-level issues | Yes |
| Agent learns the surface via wat-eval calls to introspection primitives | Yes |
| One tool covers all wat capabilities | Yes |

### Why this is the right shape

The user's central insight (the collapse): JSON-RPC is the
envelope; wat is the language; `wat-eval` is the dial-tone.
The agent gets full Lisp expressivity through a single tool
without per-function tool registration, JSON Schema
generation, or type transcoding ceremony.

This is the most important contract for MCP's "thin wrapper"
property. Everything MCP does that isn't initialization or
break coordination flows through `wat-eval` and `eval-edn!`.

## Contract 6 — The cli's mode dispatch

### What pry provides

The pry-cli wiring (Layer 3): `--pry` flag triggers
conditional battery registration + entry-point swap to
`:wat::pry::main`.

### What MCP wraps

The MCP cli wiring (Layer 8): `--mcp` flag triggers
conditional battery registration + entry-point swap to
`:wat::mcp::main`. When both `--pry --mcp` are set, both
batteries register; MCP wins entry-point lookup.

### The contract

| Property | Required |
|---|---|
| `--pry` without `--mcp`: pry battery loaded; entry-point `:wat::pry::main` | Yes |
| `--mcp` without `--pry`: MCP battery loaded; entry-point `:wat::mcp::main`; pry symbols NOT available | Yes |
| `--pry --mcp` together: BOTH batteries loaded; MCP wins entry-point; pry primitives available via MCP's wat-eval | Yes |
| Neither flag: neither battery; entry-point `:user::main` | Yes |

### Why this is the right shape

The cli is sovereignty (per arc 104's HOLOGRAM framing). It
decides which batteries are present and which entry-point
runs. The pry/MCP modes are two flags that adjust this; the
substrate doesn't grow any new mechanism beyond what arc 100
established.

The `--pry --mcp` combination is the developer's everyday
flag-set: agent connection plus pry primitives loaded for
break support. Pry battery registers first (installs default
handler); MCP battery registers second (overrides handler
with notification variant). Both surfaces coexist; agent has
full access to both.

## Contracts that DON'T need to exist

Worth being explicit about what's NOT a contract:

- **MCP doesn't have its own ls / show / completions /
  override-return / eval-in-frame.** Those are pry's. MCP
  invokes them via wat-eval; doesn't reimplement.
- **MCP doesn't have its own break primitive.** It overrides
  pry's break-handler; doesn't ship a parallel primitive.
- **Pry doesn't know about MCP.** The pry battery installs the
  default break-handler unconditionally; the MCP battery
  overrides if both are loaded. Pry doesn't check for MCP
  mode; doesn't have MCP-specific code paths.
- **The frozen world is the same in both modes.** No mode-
  dependent state in the substrate beyond the
  battery-registration-time handler installation. The
  freeze invariant holds identically.

## The integration map, condensed

```
       Operator (terminal)            Agent (MCP client)
            │                              │
            │ rustyline                    │ JSON-RPC over stdio
            ▼                              ▼
   ┌──────────────────┐           ┌──────────────────┐
   │  wat-cli --pry   │           │  wat-cli --mcp   │
   │  + pry battery   │           │  + MCP battery   │
   └──────────────────┘           │  + pry battery * │
            │                     └──────────────────┘
            │                              │
            ▼                              ▼
   :wat::pry::main                :wat::mcp::main
            │                              │
            │  serve loop                  │  JSON-RPC dispatch
            │  (read EDN line,             │  (read JSON line,
            │   parse, eval, write)        │   route by method)
            │                              │
            └──────────┬───────────────────┘
                       │
                       ▼
              :wat::eval-edn!  (the substrate eval engine)
                       │
                       │ evaluates expressions against
                       │ frozen world OR captured break env
                       │
            ┌──────────┴───────────────────┐
            │                              │
            ▼                              ▼
    pry primitives                  break-handler override
    :wat::pry::ls                   (MCP battery's notification
    :wat::pry::show                  + suspend mechanism, when
    :wat::pry::break-with-stdio      MCP loaded)
    :wat::pry::override-return
    :wat::pry::continue
    ... etc.

   * pry battery loaded only when --pry is also set;
     allows agent-side pry primitives via wat-eval
```

Two consumers; two cli modes; three batteries (pry, mcp,
wat-json); one substrate. Pry primitives are the surface;
both consumers reach them through the same `eval-edn!` engine.
MCP adds one new substrate mechanism (handler-override + session
registry) and a wat-level dispatcher (`wat/std/mcp.wat`); pry
provides everything else.

This is what surface integration looks like when MCP is a thin
wrapper over pry.

# Open questions

What's locked and what's not, as of 2026-04-29 conversation
close. Pattern matches 005's open-questions.md.

## Locked decisions (recap from README's table)

- **One MCP tool: `wat-eval`.** Single string parameter; EDN
  payload inside JSON envelope.
- **Discovery via wat introspection** — agent calls
  `(:wat::pause::ls)` and `(:wat::pause::show :sym)` through
  `wat-eval` to learn the surface.
- **Break-as-notification** — JSON-RPC notification when
  `(:wat::pause::break)` fires; suspended call resumes via
  `(:wat::pause::continue)`.
- **JSON I/O is the prerequisite.** Ships as `wat-json` battery
  before 006 slices.
- **Gating via `--mcp`** — same mechanism as `--pause`. MCP
  battery conditionally registered; freeze fails on `:wat::mcp::*`
  references without the flag.
- **006 depends on 005 slices 1+2.** No reimplementation;
  shared substrate primitives.
- **MCP server is a wat program** in `wat/std/mcp.wat`. ~200-400
  lines; minimal Rust glue.
- **Hologram preserved** — agent is a wat caller through JSON
  envelope; cannot define / redefine / load.

## Open — protocol-level

### 1. Single tool vs two tools (`wat-eval` + `wat-eval-stream`)?

**Lean:** two tools. `wat-eval` for fire-and-forget; never
pauses; always returns a value or error. `wat-eval-stream` for
calls that may pause via break; supports `session` parameter.

Cleaner separation; basic eval stays simple. Considered
alternative: single tool with mode flag in the params — works
but slightly muddier.

**Resolves when:** slice 1 implementation surfaces whether
break detection at the substrate layer is easier with one tool
or two.

### 2. JSON-RPC `id` field semantics for suspended calls

When `wat-eval-stream` invokes a function that hits break:
- The original request has `id: 5` (or whatever).
- The notification has no id (notifications don't have ids).
- Subsequent `wat-eval` follow-up calls have their own ids.
- Eventually the original request's response with id=5 lands.

Question: between the notification and the eventual response,
how does the agent know request 5 is "in progress" vs "lost"?

**Lean:** the substrate tracks pending request ids in its
session registry. Agent can query
`wat-eval (:wat::mcp::pending-requests)` if curious. Otherwise
the agent maintains its own state (most agent frameworks do).

**Resolves when:** slice 2 implementation. Test with a real
agent client to confirm the id-tracking story is honest.

### 3. Cancellation semantics

MCP supports request cancellation via `notifications/cancelled`.
What happens if the agent cancels a `wat-eval-stream` call
that's currently paused at a break?

**Options:**
- **Discard the session.** The substrate aborts the suspended
  eval; no return value sent; session deregistered.
- **Resume with a sentinel.** The substrate calls
  `(:wat::pause::override-return :wat::mcp::cancelled)` internally;
  the function unwinds; downstream code sees a sentinel value.
- **Refuse cancellation while paused.** The agent must explicitly
  resume; cancellation only applies to non-paused calls.

**Lean:** discard the session. Cancellation means "I no longer
care about this result"; the substrate aborts cleanly.

**Resolves when:** slice 2 + observed cancellation in real use.

### 4. Initialize handshake — what capabilities does wat-mcp advertise?

MCP's `initialize` exchange includes server capabilities. For
wat-mcp the minimum:

```json
{
  "capabilities": {
    "tools": {"listChanged": false},
    "experimental": {"watPry": {"version": "0.1.0"}}
  }
}
```

`tools.listChanged: false` because wat-mcp's tool list is
constant. The pause experimental capability indicates support for
the break protocol.

**Open:** what other capabilities does MCP convention expect?
`resources`? `prompts`? Not strictly needed; could ship later.

**Lean:** start minimal; add capabilities as use cases surface.

**Resolves when:** real MCP clients are tested; advertise what
they expect.

## Open — agent-experience

### 5. Should `wat-eval` errors be JSON-RPC error responses or successful responses with error content?

When the agent sends a malformed expression:

- **Option A:** JSON-RPC error response with code -32xxx and
  message describing the parse / type / runtime error.
- **Option B:** JSON-RPC success response with content
  containing an EDN-encoded `:Result::Err <ParseError>` value.

A is conventional MCP; B keeps everything inside the EDN
envelope.

**Lean:** A for protocol-level errors (malformed JSON, missing
fields, unknown tool); B for wat-level errors (parse, type,
runtime). Two layers of error: protocol + payload.

**Resolves when:** slice 1 implementation. Both shapes work;
pick one and document.

### 6. Should the substrate render large results lazily?

If the agent calls `wat-eval (:wat::std::stream::collect ...)`
and the result is a 100,000-element vector, the substrate has
to serialize all 100,000 to EDN, wrap in JSON, return as one
message. That's a multi-megabyte response.

**Options:**
- **Cap response size.** Truncate with a marker; agent knows
  the result was abbreviated.
- **Stream large results.** Multiple JSON-RPC messages with
  pagination tokens.
- **No special handling.** The agent decides whether to call a
  function that returns a huge result.

**Lean:** no special handling for slice 1; revisit if real
agents start hitting MCP message size limits.

**Resolves when:** real agent debugging surfaces a need.

### 7. Pretty-printing for MCP results?

EDN written compact is valid but agent token counts grow with
verbosity. Pretty-printed EDN with indentation is more
readable but uses more tokens.

**Lean:** ship compact for slice 1. Add a pretty option later
if agents prefer it (likely they do — readable EDN is easier
for the agent to reason about).

**Resolves when:** slice 1; revisit slice 2 based on agent
behavior.

## Open — security / production

### 8. TLS / auth for TCP attach (slice 4)?

Slice 4 lets agents attach via TCP. Production deployments
need auth. Three layers:

- **Transport:** TLS via rustls.
- **Authentication:** API token, mTLS client cert, or OS-level
  (only allow connections from local users).
- **Authorization:** even with auth, what can an agent DO?
  Restrict to read-only? Restrict to specific symbol prefixes?

**Lean for slice 4 v1:** localhost-only by default; require an
explicit `--bind 0.0.0.0` for external; use OS user auth on
localhost; defer TLS + finer-grained auth for production
deployments that demand it.

**Resolves when:** a production deployment surfaces. The
substrate's existing batteries don't have auth conventions yet;
the slice 4 work would establish one.

### 9. Sandboxed evaluation for untrusted agents?

If a wat-mcp server accepts connections from untrusted agents,
those agents can call any function in the SymbolTable —
including ones that touch the filesystem, the database, the
network. The freeze invariant prevents schema-level damage;
runtime damage is unbounded.

**Options:**
- **No protection.** Document that wat-mcp servers should only
  accept trusted connections; auth lives at the transport
  layer.
- **Capability filtering.** Allow listing of permitted
  namespaces; agent's `wat-eval` rejected if it calls outside
  the allowlist.
- **Read-only mode.** A `--readonly` flag that blocks any
  primitive with side effects.

**Lean:** no protection for slice 1; the connection itself is
trusted (localhost by default). Capability filtering as a
future arc when production needs it.

**Resolves when:** untrusted-agent use cases surface (likely
later than slice 4).

### 10. Audit logging — should agent calls be logged?

Production wat-mcp servers may need to log every agent
interaction for compliance / debugging. The substrate's
existing telemetry battery (`wat-telemetry`) could be
configured as an MCP-call sink.

**Lean:** make it opt-in via a battery — `wat-mcp-audit` (or
similar) that wraps `:wat::mcp::dispatch` and logs every
incoming request + outgoing response. Optional; off by default.

**Resolves when:** a production deployment demands it.

## Open — UX / discoverability

### 11. Should `wat-eval` accept multi-line input?

The MCP envelope is JSON; the JSON string can contain any
characters including newlines. So technically yes. But the
EDN payload is one expression, not a sequence. If the agent
sends multiple top-level forms separated by newlines, what
happens?

**Lean:** treat the payload as exactly one expression. If the
agent wants to evaluate multiple forms, wrap them in `(do form1
form2)` (which wat doesn't have today — would be `let* with
ignored bindings`) or use sequencing:
`(:wat::core::let* (((_ :()) form1)) form2)`.

Slightly awkward; the alternative (parse multiple top-level
forms) is more code in the substrate.

**Resolves when:** slice 1; pick a behavior; document.

### 12. Tool description content?

The `wat-eval` tool's `description` field shows up in the
agent's tool list. What should it say?

**Lean:** something like:

> Evaluate a wat expression against the frozen world. Input is
> an EDN-encoded form in the `msg` field. Output is the
> EDN-encoded result, or a wat error wrapped as `:Result::Err`.
> Use `(:wat::pause::ls)` to list visible symbols and
> `(:wat::pause::show :sym)` to read source. The freeze invariant
> applies — define / redefine / load forms are forbidden.

Short, points the agent at introspection, names the constraints.

**Resolves when:** slice 1 implementation; final wording per
testing with real agent clients.

### 13. Should the agent receive notifications about non-break events?

E.g., the substrate's stderr output, capacity-mode warnings,
log messages. MCP notifications are one-way client-bound;
piping diagnostic output as notifications is feasible.

**Options:**
- Pipe ALL stderr as `notifications/log` events.
- Pipe nothing; let agents read stderr through some other
  mechanism if curious.
- Configurable via initialize handshake.

**Lean:** pipe nothing for slice 1; add as a slice 4+ feature
if real agents need diagnostic visibility.

**Resolves when:** later.

## What's NOT a question

Anything in the README's "What's locked" table — those are
decisions, not open questions. Anything in `slice-plan.md`'s
"What works at the end" sections — those are committed
acceptance bars.

The 13 questions above are the load-bearing unknowns. Most
have leans documented; resolution is "first slice that surfaces
the question." None block opening the wat-rs arc once 005's
prerequisites are met.

The biggest unknown is #8 (TLS/auth for TCP attach), which is
real production work and may need its own architectural arc
beyond just 006. The substrate doesn't have a security-conventions
doc today; slice 4's work would establish one. Hence: 4 is
deferred until a real production deployment demands it.

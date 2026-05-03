# Slice plan — four slices, ordered by dependency on 005

Each slice closes a discrete capability. The first two land
the basic agent-callable surface. Slice 3 adds counterfactual-
debugging primitives. Slice 4 (deferred) adds remote attach.

Total estimated effort: ~7-9 days for slices 1+2 (the core
agent experience). Slices 3+4 add another 4-6 days when
demanded.

## Prerequisite — wat-json arc

Before any 006 slice can ship, the JSON I/O substrate
(`crates/wat-json/`) must exist. See `json-prerequisite.md`.

Estimated effort: ~3-4 days for a `wat-json` battery that
wraps `serde_json` with type-aware read/write.

**Acceptance bar:**
- `:wat::json::read :T :String -> :Result<:T, :ParseError>` works
- `:wat::json::write :T -> :String` works
- Round-trip tests for primitives + structs + parametric types
- ~50 lines of wat-tests covering edge cases

Open the wat-json arc first; close it; then open 006 slices.

---

## Slice 1 — Basic MCP eval

**Depends on:** 005 slice 1 (`:wat::pause::ls`, `:wat::pause::show`,
`:wat::pause::completions`) + wat-json arc.

**What ships:**

- `crates/wat-mcp/` — new workspace crate. Rust shims for the
  small set of MCP-specific primitives (initialize handshake,
  notification emitter — most other things live in wat).
- Substrate primitive: minimal — most of the work happens in
  wat-level code reading/writing JSON.
- `wat/std/mcp.wat` — the MCP server itself:
  - `:wat::mcp::main` (cli entry-point function)
  - `:wat::mcp::serve` (the read-eval-print loop with JSON-RPC
    framing)
  - `:wat::mcp::dispatch` (route by JSON-RPC method)
  - Method handlers: `initialize`, `tools/list`, `tools/call`,
    cancellation
  - `wat-eval` tool implementation (extract `msg`, parse EDN,
    eval, serialize result)
- wat-cli additions:
  - `--mcp` argv flag.
  - Conditional mcp-battery registration (gating; mirrors
    `--pause`).
  - Entry-point lookup swap to `:wat::mcp::main` when in mcp
    mode.

**What works at the end:**

- `wat --mcp <entry.wat>` opens an MCP-callable session.
- Agent connects via stdio JSON-RPC.
- `tools/list` returns the single `wat-eval` tool.
- `tools/call wat-eval msg=<edn>` evaluates against the frozen
  world; returns EDN result.
- Eval errors return as JSON-RPC errors.
- Discovery via `wat-eval (:wat::pause::ls ...)` and similar.
- All non-break wat operations work — the agent has full access
  to the program's symbol table.

**What doesn't work yet:**

- `(:wat::pause::break)` notification — slice 2.
- Counterfactual returns (`override-return`) — slice 3.
- Remote TCP attach — slice 4.

**Estimated effort:** 3-4 days (assuming 005-1 + wat-json
arc are done).

**Acceptance bar:**
- Run `wat --mcp /tmp/test.wat` where test.wat declares a
  function `(:foo (n :i64) -> :i64)`.
- Agent (or test harness simulating one) sends
  `tools/call wat-eval msg="(:foo 5)"`.
- Substrate returns `5` (or whatever the function computes).
- Agent sends `wat-eval (:wat::pause::ls)`.
- Substrate returns the symbol list including `:foo`.
- Agent sends `wat-eval (:wat::pause::show :foo)`.
- Substrate returns the function's source.

---

## Slice 2 — Break-as-notification

**Depends on:** 005 slice 2 (`:wat::pause::break-with-stdio` +
FrameInfo::env) + slice 1 of 006.

**What ships:**

- Substrate primitive: session registry — process-global
  `HashMap<SessionId, PausedSession>` storing captured Environment,
  CALL_STACK, suspended call ID.
- Substrate primitive: MCP-aware variant of break — when
  `:wat::pause::break-with-stdio` is invoked under MCP mode, it
  emits a JSON-RPC notification, suspends the original eval,
  registers a session.
- Substrate primitive: `:wat::pause::continue` — closes a session;
  resumes the suspended eval.
- `wat-eval-stream` tool — second MCP tool that supports
  pause-and-resume semantics. Includes `session` parameter for
  follow-up calls during a break.
- `wat-eval` (slice 1's tool) gains optional `session` parameter
  to route eval calls to a paused session's captured Environment.
- `:wat::mcp::dispatch` recognizes notifications/pause/break and
  emits them when paused sessions need agent input.

**What works at the end:**

- A wat program with `(:wat::pause::break)` inside a function body.
- Agent calls `wat-eval-stream` to invoke the function.
- When break fires, agent receives a notification with session
  ID, file/line/col, env summary.
- Agent makes follow-up `wat-eval` calls with the session ID to
  inspect the captured Environment.
- Agent sends `wat-eval session=X (:wat::pause::continue)` to
  resume.
- Original `wat-eval-stream` call returns with the function's
  actual return value.
- Multi-frame inspection works via `:wat::pause::up` /
  `:wat::pause::down` (reusing 005's primitives).

**What doesn't work yet:**

- Override-return — slice 3.
- One-shot frame addressing — slice 3.

**Estimated effort:** 3-4 days.

**Acceptance bar:**
- A wat program with `(:wat::pause::break)` in a function body;
  function called via `wat-eval-stream`.
- Agent receives a notification with session id.
- Agent calls `wat-eval session=X (:wat::pause::env)` and sees
  the locals.
- Agent calls `wat-eval session=X (some-arbitrary-expression)`
  and sees the result evaluated against the captured scope.
- Agent calls `wat-eval session=X (:wat::pause::continue)`.
- Original `wat-eval-stream` call returns; agent sees the
  function's return value.
- All transcripts captured in integration tests.

---

## Slice 3 — Counterfactual-debugging primitives

**Depends on:** 006 slice 2.

**What ships:**

- Substrate primitive: `:wat::pause::override-return :T -> :()`
  — force-return from the current paused frame with a specific
  value. The original suspended call ends; `wat-eval-stream`
  returns the override value as the function's "actual" result.
- Substrate primitive: `:wat::pause::eval-in-frame :i64 :String -> :T`
  — evaluate an expression against a specific frame's
  Environment, addressed by index (0 = bottom of stack).
  Doesn't change the active frame; just one-shot eval at a
  different scope.
- `wat/std/mcp.wat` extensions: surface these in the agent's
  pause-command vocabulary.

**What works at the end:**

- Agent in a break can hypothesize: "what if compute-decision
  returned :Action::Sell here?" — sends
  `wat-eval session=X (:wat::pause::override-return :Action::Sell)`.
  The function unwinds with that value; downstream code receives
  it; agent sees the cascade.
- Agent can read state from any frame without walking:
  `wat-eval session=X (:wat::pause::eval-in-frame 0 candle)`.
- Combined with slice 2's tools, agent has the full pause
  inspection surface plus counterfactual control.

**What doesn't work yet:**

- Same as slice 2's "doesn't work yet" minus override / eval-in-
  frame.

**Estimated effort:** 2-3 days.

**Acceptance bar:**
- Agent in a break sends override-return; original
  wat-eval-stream returns the override.
- Agent sends eval-in-frame with frame=0 and gets the
  bottom-frame's locals; sends with frame=N and gets that
  frame's locals; both work without changing the "current"
  frame.

---

## Slice 4 (deferred) — TCP attach for remote MCP

**Depends on:** 006 slice 3, plus possibly 005 slice 5 (TCP
attach for human pause; same machinery).

**What ships:**

- `wat --mcp --serve-tcp 9999` — backend binds to TCP, accepts
  connections, runs MCP server over the socket.
- Authentication / authorization — TLS, token, IP whitelist,
  whatever's appropriate. Open question (see `open-questions.md`).
- Multi-client behavior — does one TCP server accept multiple
  agents? Lean: yes, each agent gets its own session-scope; the
  frozen world is shared.

**What works at the end:**

- Agent on machine A connects to wat-mcp running on machine B.
- All slice 1-3 capabilities work over TCP.
- Production deployments can opt into agent control planes
  via the flag.

**Estimated effort:** 4-6 days (substantially due to security
concerns).

**Acceptance bar:** Agent on a different machine connects via
TCP; `wat-eval` calls work; break-mode works; production
deployment of a wat program with `--mcp --serve-tcp` is safe
(TLS, auth, etc., as decided).

---

## Total committed effort

- wat-json prerequisite arc: 3-4 days
- 006 slice 1 (basic eval): 3-4 days
- 006 slice 2 (break-as-notification): 3-4 days
- 006 slice 3 (counterfactuals): 2-3 days

Subtotal: ~11-15 days for the full agent-driven debugging
experience over local stdio.

Slice 4 (TCP attach): 4-6 days when production deployment
demands.

## Critical path with 005

The full critical path from "no pause, no mcp" to "feature-
complete pause + mcp":

1. 005 slice 1 (bare pause mode + introspection): 3-4 days
2. 005 slice 2 (break primitive + FrameInfo::env): 3-4 days
3. wat-json arc (JSON I/O): 3-4 days
4. 006 slice 1 (basic MCP eval): 3-4 days
5. 006 slice 2 (break-as-notification): 3-4 days

Subtotal: ~15-20 days for both pause experiences (human and
agent) + their core debugging features.

005 slice 3 (rustyline) and 006 slice 3 (counterfactuals) and
remote-attach for both can ship in parallel afterward, on
demand.

## What this isn't

Same disclaimer that applied to 005:

This isn't a substrate redesign. The substrate has shipped
nearly everything pause+MCP need across arcs 097-104 (and earlier
arcs for type-checking, eval, etc.). 005 + 006 are packaging
layers exposing existing capabilities through new envelopes.

The user's recognition was load-bearing: "the substrate has
been collecting this capability arc by arc without anyone
naming it." 005 and 006 are the naming.

## Migration path

When the user is ready to open the wat-rs arcs, the order:

1. Open arc XXX-wat-pause-1 (005-1's substrate work).
2. Open arc XXX+1-wat-pause-break (005-2's break primitive).
3. Open arc XXX+2-wat-json (the prerequisite).
4. Open arc XXX+3-wat-mcp-1 (006-1's basic eval).
5. Open arc XXX+4-wat-mcp-break (006-2's break-as-notification).
6. (Optional later) Open arc XXX+N for slice 3 of each.

Six arcs over ~3 weeks. Each sealed independently with INSCRIPTION;
each builds on the previous; the BOOK gets a chapter or two
documenting the wat-as-agent-Lisp recognition.

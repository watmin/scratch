# Relation to 005-wat-pause

006 builds on 005. The two scratches are siblings — same era,
shared substrate primitives, complementary surfaces. This file
names the dependency precisely so future readers walking these
scratches see the ordering.

## What 005 ships that 006 reuses

The key 005 primitives 006 depends on:

| 005 primitive | What 006 uses it for |
|---|---|
| `:wat::pause::break-with-stdio` | The break primitive itself. 006 wraps it in an MCP-aware variant; doesn't reimplement. |
| Environment capture (Arc clone in `eval_form`) | Same mechanism. 006's break-as-notification needs the captured Environment to expose to the agent across MCP calls. |
| `FrameInfo::env` extension | 005 slice 2 adds Environment to FrameInfo for `:up`/`:down` walking. 006 reuses for `:wat::pause::eval-in-frame`. |
| `:wat::pause::ls` | The agent calls this through `wat-eval` to discover the SymbolTable. |
| `:wat::pause::show` | The agent calls this through `wat-eval` to read source. |
| `:wat::pause::completions` | Could be exposed for agent-side completion if it helps; not strictly needed since agent has full string composition. |
| `:wat::pause::where` / `:wat::pause::frames` | Used during break-mode for agent-side stack inspection. |
| `:wat::pause::last-error` | Same — agent reads via `wat-eval`. |
| `wat/std/pause.wat` (the loop) | Pattern reference for `wat/std/mcp.wat`'s implementation. |

Roughly: **005 builds the substrate's introspection +
break-and-resume mechanics. 006 wraps them in JSON-RPC.**

Without 005's slice 2, there's no `(:wat::pause::break)` for an
agent to hit. Without 005's slice 1, there's no `:wat::pause::ls`
for the agent to call for discovery. 006 doesn't reinvent any
of these; it just wires them to a different envelope.

## What 006 adds that 005 doesn't have

| 006 addition | Why 005 doesn't need it |
|---|---|
| `wat-json` battery (or a JSON-RPC envelope parser/writer) | 005 doesn't speak JSON; rustyline is the human-facing wire. |
| Session registry for paused MCP eval calls | 005's break drops into stdin-reading inline; no session tracking needed. |
| `:wat::pause::override-return` | Useful for agents doing counterfactual reasoning; humans rarely want this (humans usually want `:continue` and let the function complete). |
| `:wat::pause::eval-in-frame` (with explicit frame index) | Humans use `:up` / `:down` to walk; agents may prefer one-shot frame addressing without state. |
| `:wat::mcp::dispatch` and the JSON-RPC method routing | Wat code in `wat/std/mcp.wat`; matches `:wat::pause::dispatch` shape. |
| `:wat::mcp::main` (cli entry-point for MCP mode) | Symmetric to `:wat::pause::main`. |

Each addition is small. None replaces 005's work; all extend it.

## Slicing dependencies

| 006 slice | Depends on |
|---|---|
| 006-1 (basic MCP eval) | 005-1 (`:wat::pause::ls` + `:wat::pause::show`) + wat-json arc |
| 006-2 (break-as-notification) | 005-2 (`:wat::pause::break-with-stdio` + FrameInfo::env) |
| 006-3 (override-return + eval-in-frame primitives) | 006-2 |
| 006-4 (TCP attach for remote MCP) | Either 005-5 (TCP attach for human pause) or 006-3 |

So the natural order:

1. 005 slice 1 (bare pause mode + introspection)
2. 005 slice 2 (break primitive + FrameInfo::env)
3. wat-json arc (JSON I/O dependency)
4. 006 slice 1 (basic MCP eval)
5. 006 slice 2 (break-as-notification)
6. 005 slice 3 (rustyline frontend) — independent of 006
7. 006 slice 3 (override-return / eval-in-frame)
8. 005 slice 4 (stepping) and 006 slice 4 (TCP attach) — both
   deferred; ship when consumers demand

005's slice 3 (rustyline) and 006's slice 1+2 are independent
and can ship in either order. Slices 4+ are demand-driven.

## Shared `wat/std/` patterns

Both 005 and 006 ship wat-level loop programs that follow the
same shape:

```scheme
(:wat::core::define
  (:wat::TYPE::main (in :IOReader) (out :IOWriter) (err :IOWriter) -> :())
  (:wat::TYPE::serve in out err))

(:wat::core::define
  (:wat::TYPE::serve (in :IOReader) (out :IOWriter) (err :IOWriter) -> :())
  ;; tail-recursive read-eval-print
  ...)

(:wat::core::define
  (:wat::TYPE::dispatch (request :Request) -> :Response)
  ;; pattern-match the request shape; route to handlers
  ...)
```

Where `TYPE` is `pause` for 005 and `mcp` for 006. Same shape;
different wire format inside the main loop. Future wat-level
servers (HTTP? gRPC?) would follow the same template.

This is what BOOK Chapter 67 / arc 103's pipe-protocol memory
entry named: **the protocol is the substrate's; transports are
interchangeable.** 005 picks plain EDN+newline as the
transport (with rustyline frontend). 006 picks JSON-RPC over
stdio. Both reuse the same underlying pause primitives.

## Should 006 fold into 005 as another slice?

Considered. Folding would:

- Pros: one scratch, one arc, one sealed INSCRIPTION at the end.
  Slightly less scaffolding overhead.
- Cons: 005's scope is "pause-shape REPL for humans"; 006's
  scope is "wat programs as MCP servers for agents." Different
  consumer; different envelope; different deployment shape
  (rustyline frontend vs JSON-RPC stdio). Folding muddles both.

**Lean: keep them separate.** The user's discipline has been
"one scratch per architectural concern." Pause is the concern;
MCP is the concern. They share machinery; they don't share
purpose.

The two scratches reference each other; the two future arcs
will reference each other; readers walking back through history
see two distinct moves with a documented dependency.

## What about a shared `wat-introspect` battery?

Considered: extract the introspection accessors (`:wat::pause::ls`
/ `:wat::pause::show` / `:wat::pause::completions`) into a separate
battery that both wat-pause and wat-mcp depend on.

This would:
- Pros: clear that introspection is general-purpose; the battery
  could be loaded by users who want neither pause nor MCP but
  do want runtime symbol lookup.
- Cons: more workspace crates; the introspection is tightly
  coupled to pause semantics (Environment capture for break).
  Splitting them might be premature.

**Lean: defer.** Ship wat-pause with introspection inside it. If
a non-pause caller surfaces (e.g., a battery that wants to do
runtime symbol lookup for debug logging), revisit and extract
then.

## The shared mechanism the user keeps making

A thread runs through the substrate's design — every arc that
adds a "second mode of an existing thing" follows this shape:

- Arc 099 — wat-cli extracted from substrate (one binary becomes a separable component)
- Arc 100 — wat-cli vended as library (the component's API exposed to consumers)
- Arc 101 — wat-cli single-purpose (third mode pruned in favor of cargo composition)
- Arc 103 — `spawn-program` (in-process sibling of `fork-with-forms`)
- Arc 104 — wat-cli always-fork (containment becomes structural)
- 005-wat-pause — pause-mode entry-point + read-eval-print over stdio
- 006-wat-mcp — agent-mode entry-point + JSON-RPC over stdio

Each is a thin overlay on substrate that already exists. None
introduces architecture that was missing; each names a packaging
that the substrate already implies.

The relation between 005 and 006 is the latest instance: same
substrate, two consumers, one shared discipline (the freeze
invariant gates both). Walking the scratches in order makes
the pattern legible for future readers.

## Implementation flow

When the user is ready to migrate to real wat-rs arcs, the
ordering would be:

1. Open arc XXX-wat-pause (slice 1 ships).
2. Open arc XXX+1-wat-pause-break (slice 2 ships).
3. Open arc XXX+2-wat-json (the substrate prerequisite).
4. Open arc XXX+3-wat-mcp (slice 1 ships).
5. Open arc XXX+4-wat-mcp-break (slice 2 ships).

Five arcs over what's likely two-three weeks of work. Each
sealed independently. Each composes with the others without
forcing simultaneous landing.

The user has been disciplined about this kind of sequencing
across the project — small arcs with clear borders, ordered by
dependency. 005+006 fits the pattern.

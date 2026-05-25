# Scratch 006 — `wat-mcp` — wat programs as MCP servers

**Started:** 2026-04-29.
**Status:** structural shape locked in conversation. Depends on
005-wat-pause slices 1+2 for substrate primitives. Ready to migrate
to a wat-rs arc when those land.

> **Note (2026-04-29 alignment update):** scratch 007
> (dependency resolution) supersedes one detail in this scratch.
> `:wat::pause::override-return` and `:wat::pause::eval-in-frame` are
> **pause primitives**, not MCP-specific. Same substrate primitive
> serves both consumers (operator at rustyline; agent via MCP).
> See `scratch/2026/04/007-dependency-resolution/DEPENDENCIES.md`
> for the locked layered tree and
> `INTEGRATION-CONTRACTS.md` for surface handoffs. The rest of
> 006's framing stays valid.

**Sibling materials:**
- `scratch/2026/04/005-wat-pause/` — the pause primitives this builds
  on. The break primitive, environment capture, and introspection
  accessors are 005's substrate work; 006 wraps them in JSON-RPC.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md` — the
  framing that says programs ARE the surface between worlds. MCP
  is the protocol the surface speaks when the consumer is an
  agent.
- `wat-rs/docs/arc/2026/04/086-edn-roundtrip-and-natural/` — the
  EDN read/write substrate. JSON I/O is the analog needed for MCP
  envelope handling.
- `wat-rs/docs/arc/2026/04/103-kernel-spawn/INSCRIPTION.md` — the
  EDN+newline pipe protocol. MCP-over-stdio is the same shape;
  JSON envelope around the same EDN payload.
- `BOOK.md` Chapter 67 — *The Spell.* The protocol crosses every
  transport boundary. Adding "agent over JSON-RPC" to the list
  is purely an envelope choice; the substrate stays unchanged.
- `BOOK.md` Chapter 65 — *The Hologram of a Form.* The agent is
  another caller through the surface. Sees through; cannot reach
  back. The freeze invariant gates what the agent can do; the
  pause primitives gate what the agent can inspect.

---

## What this scratch captures

Mid-conversation 2026-04-29, after 005-wat-pry's structural shape
landed, the user said:

> "i just had a wild idea.... you talked about a :wat::pry::serve
> ... what if... we could have a program-as-an-mcp.... give the
> agent a way to run a program /and/ live debug it?..."

[Quote preserved verbatim. The arc was originally named `wat-pry` at
this point in time. Renamed to `wat-pause` on 2026-05-03 — see
INDEX.yaml captured-beats. The rest of this doc uses the new naming.]

The first sketch of MCP-as-tool-surface walked through generating
JSON Schema per wat function, transcoding params, etc. The user
collapsed that with one sentence:

> "i think... the JSON rpc.. is just a thin wrapper... the input
> object would be something like '{\"msg\":\":some-edn-form\"}'"

That's the recognition. JSON-RPC is the envelope MCP demands;
the actual payload is wat source as a string in the `msg` field.
The substrate doesn't translate types. The agent talks wat
directly. The MCP layer is purely transport.

This collapse eliminates the JSON Schema generation, the
per-function tool registration, and the transcoding ceremony at
the type boundary. **One MCP tool: `wat-eval`. One string
parameter. The agent uses wat itself for discovery via
`(:wat::pause::ls)` / `(:wat::pause::show :sym)`.**

The substrate has been set up for this since arc 103 named the
EDN+newline protocol. JSON-RPC is just one more transport for
the same payload shape — same way the spell from Chapter 67
crosses every transport boundary because the wire format is the
contract, not the framing.

---

## What's locked

| Decision | Resolution |
|---|---|
| MCP server's tool surface | **One tool: `wat-eval (msg :String) -> :String`.** Possibly a second `wat-eval-stream` for break-mode interaction. |
| Tool params | EDN-encoded wat expression inside JSON `msg` field. JSON is envelope; EDN is payload. |
| Type discovery | Agent calls `wat-eval (:wat::pause::ls :prefix)` and `wat-eval (:wat::pause::show :sym)` to walk the SymbolTable. No JSON Schema generation; introspection is wat-shaped. |
| Break-as-notification | When `(:wat::pause::break)` fires during MCP eval, substrate sends JSON-RPC notification, leaves original `wat-eval` call suspended. Agent inspects via additional `wat-eval` calls. `(:wat::pause::continue)` resumes; original call returns. |
| Substrate prerequisite | JSON read/write. Either a `wat-json` battery (mirrors `wat-sqlite`) or wat-level parser/writer. JSON is simpler than EDN; wat-level feasible. |
| Gating | `--mcp` flag, mirroring `--pause`'s mechanism. MCP battery registered conditionally; freeze fails on `:wat::mcp::*` references without the flag. |
| Dependency on 005 | Slices 1+2 of wat-pause must ship first. The break primitive, Environment capture, introspection accessors are 005's substrate work. 006 reuses them; doesn't reimplement. |
| MCP server implementation | Wat code in `wat/std/mcp.wat`. ~200-400 lines + the JSON I/O dep. Substrate doesn't grow Rust; the wat-level surface adds. |
| Hologram preserved | Agent is a wat caller through a JSON envelope. Same constrained-eval rules. Cannot define / redefine / load. The freeze invariant gates the agent the same way it gates a human pause user. |
| `:wat::mcp::main` | Cli's mcp-mode entry-point. Replaces `:user::main` when `--mcp` set, mirroring `--pause`. |

## Open

See:
- `the-collapse.md` — the user's central insight; what changed when "JSON is just envelope, EDN is payload" landed.
- `one-tool-surface.md` — `wat-eval` as the single MCP tool; discovery via wat introspection.
- `break-as-notification.md` — agent-driven debugging; suspended-call + notification protocol.
- `json-prerequisite.md` — the one substrate dependency; battery vs wat-level.
- `gating.md` — `--mcp` flag; mcp battery conditionally registered; same mechanism as `--pause`.
- `use-cases.md` — what this unlocks: agent debugging, library exploration, multi-program orchestration, self-hosting.
- `relation-to-005.md` — dependency on pause primitives; shared machinery; ordering.
- `slice-plan.md` — build order; what ships first.
- `open-questions.md` — unresolved items.

## Why this is bigger than it looks

The wat substrate becomes one of the most natural MCP hosts
possible the moment this lands. **Every `.wat` file ever written
becomes agent-callable for free.** No glue code per program; no
per-tool registration; no JSON Schema annotations; no type
mapping. The trading lab, the DDoS lab, the telemetry
interrogator, every battery — all immediately exposable as agent
surfaces by passing `--mcp`.

And the agent gets full Lisp expressivity through one tool. The
agent can compose `(:wat::core::let* ((x (:foo 1)) (y (:bar x)))
(:baz x y))` as a single eval call. Compare to per-function
MCP tools, where the same composition would require sequential
tool calls with intermediate results in the agent's context. The
substrate's compositional core is preserved on the wire.

The freeze invariant + pause's introspection + the EDN+newline
protocol have been three separate properties of the substrate.
Tonight they compose into a fourth property — **wat as the
agent's Lisp** — that none of them implied alone.

---

## Extension — 2026-05-25 (the daemon revision)

Three weeks and the 109→170→236 arcs later, the design was revisited cold and
**re-derived the same spine** (one tool, speak-wat, EDN-in/out, `{"msg":...}`,
REPL, MCP-as-dial-tone — a self-convergence). What grew: from *eval + pause*
to a **persistent daemon that does real work** (spawns threads/processes,
touches the world), enabled by the spawn/service/stdio primitives that landed
in between. The new layer (all dated 2026-05-25):

- **`the-daemon-revision`** — the convergence + the persistent-daemon model +
  the key collapse: *spawn-program ≡ stdio = universe-residency with Claude as
  a tier* (wat-mcp is `spawn-program` where the parent is Claude; the only new
  surface is the JSON envelope).
- **`transport-and-posture`** — stdio-only by construction: the loopback-
  exploit class *ceases to exist* (no listener to attack); payload gated by
  `def-restricted` + selective-hermetic; remote (UDS/TLS/mTLS) is a separate
  future surface ("wat remote programs").
- **`purpose-think-in-functions`** — the purpose: let the LLM express a
  concept in wat and have it evaluated (writing programs + structural data
  analysis). The one-tool collapse = *notation is the barrier* applied to the
  agent interface (schemas are the "go learn Rust" of MCP; speak-wat is the
  refusal). The coherence-gate frontier.
- **`substrate-surface`** — audit of what's *shipped* to build a v1 from today
  + the honest constraints (manual service pattern not `defservice`;
  `HologramCache` LRU not `EngramHologram`; no network primitive; bridge /
  telemetry renames).

Net: a v1 is buildable today on shipped primitives, over stdio, with a thin
JSON shim. Two named frontiers remain — durable-recall (`EngramHologram`) and
the coherence gate (truth engine).

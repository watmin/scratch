# Scratch 007 — Dependency resolution — pry pristine, MCP wraps it

**Started:** 2026-04-29.
**Status:** dependency tree locked. The user gave the four-question
discipline as the gate, named the order (pry pristine first, then
MCP), and required surface integration. This scratch captures the
layer-by-layer dependency tree.

**Sibling materials:**
- `scratch/2026/04/005-wat-pry/` — the pry slice plan + primitives
  + open questions. This scratch's Layers 1-4 reference 005's
  surfaces.
- `scratch/2026/04/006-wat-mcp/` — the MCP slice plan + collapse
  recognition + use cases. This scratch's Layers 5-8 reference
  006's surfaces, with one correction baked in (override-return
  / eval-in-frame are pry primitives, not MCP-specific).

---

## What this scratch captures

User's locked alignment 2026-04-29:

> "we do not take short cuts - we need pry to be pristine first,
> then build mcp to interface with it"

> "we follow several questions without fault... is this obvious?
> is this simple? is this honest? is this a good ux? if those
> are satisfied - we proceed"

> "this forces us to resolve our entire dep tree with composable
> functions to deliver simplicity on all surfaces..."

> "i want alignments now... the surfaces must integrate"

> "mcp interfaces with pry - pry exists to satisfy interactive
> exploration. mcp is how the agent does it, pry directly is
> how operators will use it"

The four-question gate is the discipline. The pry-pristine-first
ordering is the constraint. The surface integration is the goal.

This scratch documents the dependency tree such that:

- Each layer's surfaces are explicit.
- Each layer's dependencies on prior layers are explicit.
- The four-question gate validates each layer before the next
  opens.
- The integration contracts between pry and MCP are explicit.
- Concrete impl details are deferred per the user's directive.

## What's locked

| Decision | Resolution |
|---|---|
| Order | Pry pristine (Layers 1-4) → wat-json (Layer 5) → MCP (Layers 6-8) |
| Validation gates | Four questions per layer; four questions in composition before next layer opens |
| Inspection primitives | All belong to `:wat::pry::*` namespace. MCP doesn't reimplement. |
| `override-return` / `eval-in-frame` | **Pry primitives**, not MCP-specific. Operators get them at the rustyline prompt; agents get them through `wat-eval`. (Corrects the original 006 scratch's framing.) |
| Break primitive's two behaviors | One substrate primitive captures env + frames; battery-registered handler does the loop. Pry installs inline-loop handler; MCP overrides with notification handler. |
| MCP's only new substrate mechanism | Session registry + handler-override. Everything else reuses pry primitives via wat-eval. |
| `wat-eval` tool | The agent's universal gateway. EDN payload inside JSON-RPC envelope. Discovery via wat introspection. |
| Naming refactor | Doesn't block. Use today's names; migration path will reconcile. |
| Deps | Not a constraint. rustyline, serde_json, whatever — get them when needed. |
| Rustyline | **Part of pristine pry** (Layer 4). The operator's UX isn't pristine without line editing, history, completion. |

## Files

- `DEPENDENCIES.md` — the layered dependency tree. Eight layers
  with surfaces, dependencies, and the four-question gate at
  each. Two validation gates (pristine pry, MCP) named with
  end-to-end scenarios.
- `INTEGRATION-CONTRACTS.md` — six explicit contracts naming
  where pry surfaces become MCP surfaces. The principle: MCP
  doesn't reimplement pry; MCP wraps it. Each contract spells
  out the agreement at one integration point.
- `INDEX.yaml` — beat-by-beat capture matching scratch
  convention.
- `README.md` (this file) — top-level overview.

## What this scratch does NOT do

- **Specify implementation.** That's deferred per the user's
  "we'll work out concrete impl details when we get there"
  directive.
- **Replace 005's or 006's slice plans.** Those are the
  operational work breakdowns. This scratch is the
  architectural surface alignment underneath.
- **Estimate effort.** Effort lives in 005 / 006. This scratch
  is dependency-shape, not work-volume.

## What corrections this scratch makes to 005/006

One correction is baked in:

**Correction:** The 006 scratch placed `:wat::pry::override-return`
and `:wat::pry::eval-in-frame` as MCP-specific primitives. The
user's locked alignment says they're pry primitives — the same
substrate primitive serves both consumers (operator at rustyline,
agent via MCP). The 006 scratch's slice 3 ("Counterfactual-
debugging primitives") becomes part of pristine pry's substrate
work; MCP slice 3 becomes empty (or those slices fold into
MCP slice 1 since the primitives already exist by then).

The scratch dirs themselves stay as historical record (they
captured the conversation flow); this scratch supersedes the
specific assignment of those primitives. When the wat-rs arcs
open, they reference this scratch's layering as the authoritative
dependency tree.

## When to use this scratch

- **Before opening the wat-pry arc:** read `DEPENDENCIES.md`
  Layers 0-4. Validate Layer 0 substrate state. Open arcs in
  the order Layers 1-4 prescribe.
- **After pristine pry seals:** validate the four questions
  end-to-end via DEPENDENCIES.md's scenarios. Iterate any
  layer that fails before opening Layer 5.
- **Before opening the wat-mcp arc:** read `DEPENDENCIES.md`
  Layers 5-8 + `INTEGRATION-CONTRACTS.md`. The contracts spell
  out where MCP touches pry; honor them.
- **Whenever surface alignment is in question:** the contracts
  document is authoritative. If a contract is wrong, fix the
  contract here before fixing implementation downstream.

## The principle that holds across all eight layers

> The substrate ships pry primitives once. Operators reach
> them through rustyline. Agents reach them through MCP's
> wat-eval. Same primitives; two consumers; one substrate.
> The four questions hold at every layer; nothing proceeds
> past a failing gate.

This is the alignment the user asked for. Concrete impl
follows.

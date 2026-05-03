# wat-help — SLICE-PLAN

Sketch only. Not sized for shipping. Bar to graduate:

1. Arc 109's reflection primitives (symbol lookup; AST
   serialization; source location; etc.) have firmed up enough
   to expose stable APIs
2. Arc 003 (wat-fmt) has shipped slice 1
3. User signals "let's start"

---

## Slice 1 — Basic symbol lookup + formatted output

**Goal:** `(:wat::help :sym)` looks up a symbol; returns a
formatted EDN string of its definition.

**Done when:**
- `wat-rs/crates/wat-help/` exists with arc-013 layout
- `(:wat::help :sym)` works for: top-level functions, types
  (struct/enum/newtype), constants
- Output formatted via wat-fmt; reads identical to source code
- `:HelpError` taxonomy populated; symbol-not-found returns
  cleanly; never panics
- wat-tests cover: function lookup; type lookup; constant lookup;
  not-found error
- Integration test: spin up a wat-vm with known batteries; call
  help; verify formatted output

**Out of scope:**
- Docstring extraction (slice 2)
- Variant accessors (slice 3)
- Repl integration (slice 4)
- MCP exposure (slice 5)

---

## Slice 2 — Docstring extraction + presentation

**Goal:** docstrings (where present) appear above the form in
help output.

**Done when:**
- Substrate reflection's `:wat::reflect::doc :sym` is consumed
- Docstrings rendered as `;;` comments at the top of the
  formatted output
- Multi-line docstrings format correctly (each line as its own
  `;;`)
- Symbols WITHOUT docstrings show the form without comment
  prefix (no fabricated content)
- wat-tests cover: with-docstring; without-docstring;
  multi-line; per-symbol-kind variation

---

## Slice 3 — Variant accessors

**Goal:** convenience variants `:wat::help/source`,
`:wat::help/type`, `:wat::help/doc`, `:wat::help/list`.

**Done when:**
- Each variant returns a focused subset of the symbol's
  reflection
- `:wat::help/source` — body only (no signature/doc)
- `:wat::help/type` — type signature only
- `:wat::help/doc` — docstring only
- `:wat::help/list` — symbol names in a namespace
- All variants return `:Result<:String, :HelpError>`
- wat-tests cover each variant

---

## Slice 4 — wat-repl integration

**Goal:** wat-repl's `:help`, `:doc`, `:source`, `:type`, `:ls`
special commands route through wat-help.

**Done when:**
- arc 012 (wat-repl) takes wat-help as a dep
- `:help <sym>` calls `(:wat::help :sym)` and prints
- `:doc <sym>` calls `(:wat::help/doc :sym)` and prints
- `:source <sym>` calls `(:wat::help/source :sym)` and prints
- `:type <sym>` calls `(:wat::help/type :sym)` and prints
- `:ls <ns>` calls `(:wat::help/list :ns)` and prints
- arc 012's earlier "internal commands" approach replaced; the
  commands now delegate to wat-help
- Integration test: end-to-end repl session; commands work

---

## Slice 5 — wat-cli "just works" verification

**Goal:** user's compiled `my-wat-cli` (per arc 099/100/101's
wat-cli architecture) ships wat-repl by default; help works on
user batteries without any extra setup.

**Done when:**
- wat-cli's default battery list includes wat-repl + wat-help
- `my-wat-cli repl` opens a REPL bound to the user's frozen
  world (their batteries + wat-rs core)
- `(:wat::help :my-app::some-symbol)` works against user's
  symbols — no extra registration needed
- Integration test: build a sample `my-wat-cli` with a custom
  battery; verify help reflects the custom symbols
- Documentation: deployment recipe / cookbook for "shipping a
  wat-cli with help"

**This is the LOAD-BEARING UX SLICE.** If help doesn't "just
work" in user-built CLIs, the abstraction has failed. The
user's direction:

> *"wat-repl needs to be depended on by wat-cli .... when a user
> compiles their own 'my-wat-cli' they can have the repl be used
> with whatever symbols their project created.. so help /just
> works/ in their env"*

---

## Slice 6 — wat-mcp tool exposure

**Goal:** wat-mcp can expose wat-help as an MCP tool so agents
get typed reflection over running wat-vms.

**Done when:**
- arc 006 (wat-mcp) registers a `wat-help` tool by default when
  wat-help is in the battery list
- MCP `tools/call` for `wat-help` accepts `{ symbol: "..." }`,
  returns `{ formatted: "..." }`
- Errors propagate as MCP tool-error responses (not exceptions)
- Integration test: MCP client sends a help query; agent gets
  formatted EDN back
- Cross-reference: enables Claude-as-measurer (arc 005's
  captured-beat vision) — agents need to UNDERSTAND the program
  they're inspecting

---

## Slice 7 — Production hardening

**Goal:** wat-help is genuinely usable in production wat
applications.

**Done when:**
- Performance: lookup + format < 5ms for typical symbols
  (microbenchmark committed)
- Memory: bounded; no leak on long-lived REPL sessions
- Documentation: complete reference; comparison to wat-doc;
  cookbook for common queries
- One concrete deployed wat-vm uses wat-help via REPL and via
  MCP
- Cross-references work: help output for fn A includes
  reference to fn B if B is called from A's body (optional
  sub-slice)

---

## Slices NOT planned

- **Static doc generation** — that's wat-doc (arc 006)
- **Code search** — separate concern; sibling arc
- **Autocomplete UX** — application-tier (IDE integrations,
  repl frontend); wat-help is the data layer
- **LSP server** — could be built ON wat-help + wat-fmt; not
  this arc's concern
- **Modifying source files** — wat-help is read-only

---

## Honest accounting

This slice plan is **sketched, not sized**. The biggest unknown:
the exact shape of arc 109's reflection primitives. wat-help
consumes them; if their API differs from what's sketched in
DESIGN.md, slice 1 sizing may shift.

The biggest design question: should the substrate's reflection
primitives expose a CANONICAL "give me everything about this
symbol" function, or several smaller functions that wat-help
composes? Lean: smaller functions composed by wat-help — keeps
the substrate's reflection surface minimal; lets wat-help own
the composition.

The wat-cli "just works" property (slice 5) is the load-bearing
UX win. Slices 1-4 are foundation; slice 5 is where the user
actually feels the value. Without slice 5, wat-help is a
library; with slice 5, it's an interactive surface every
wat-cli user gets for free.

The four-questions discipline applies to each slice
independently. Each slice should answer all four with honest
checkmarks before declaring the slice done.

# wat-repl — interactive evaluator (foundation under wat-pause)

User direction (2026-05-03):

> *"i think we rename and split... break the repl part into its own
> and the breakpoint/pry into its own.... ruby's irb doesn't do what
> pry does but pry builds upon irb...."*

> *"but whatever month item numbers.. they are just a disambiguator..
> the next one holds the split.... they are not an impl order.. just
> a thing to visually separate...."*

The arc number (012) is later than wat-pause (005) because the
recognition that wat-repl was the foundation came AFTER wat-pause's
design was sketched. **Arc number reflects discovery order, not dep
order.** wat-repl is conceptually the foundation; wat-pause builds
on top.

---

## What wat-repl is

The wat **interactive evaluator**. The Lisp counterpart to Ruby's
`irb`, Python's `python` REPL, Node's `node` REPL.

Read input → evaluate in some context → print result + type → loop.

That's it. No suspension; no breakpoint; no context capture from a
running program. Just an interactive evaluator with line editing,
history, completion, and a small set of meta-commands (`:type`,
`:source`, `:doc`, `:reload`, `:quit`).

A standalone tool: `wat repl` drops you into it.
An embeddable function: `(:wat::repl::start :context ctx)` opens
a REPL in the given context.

## Why it's a separate arc from wat-pause (005)

The same recognition Ruby made: **irb is the foundation; pry builds
on irb.** They are two separate concerns:

- **wat-repl (this arc):** read-eval-print loop. Generic. Doesn't
  know anything about paused programs.
- **wat-pause (arc 005):** suspension protocol + context capture +
  step controls. Knows everything about paused programs. **Uses
  wat-repl as the interactive shell after the pause captures
  context.**

If we conflated them in one crate, freestanding REPL use cases
would have to drag in pause infrastructure they don't need:

- Quick wat experimentation at the prompt
- Tutorials and learning environments
- One-shot script eval (`wat repl -e "(+ 1 2)"`)
- Editor integration (drive a wat-repl from emacs/vim/vscode)
- The wat-mcp tool "give me a freestanding wat-repl" use case

All these want a REPL without needing `--pause` gating, suspension
infrastructure, or context-capture machinery.

## Layering

```
LAYER — wat-mcp           MCP tools exposing :wat::repl::start
                           and (:wat::pause::break) to Claude
  ↓ uses both
LAYER — wat-pause          Suspension + context capture +
                           step controls (arc 005)
  ↓ uses
LAYER — wat-repl           Interactive evaluator (THIS ARC)
                           CLI: `wat repl`
                           Embeddable: (:wat::repl::start :context ctx)
  ↓ uses
LAYER — wat-rs / wat-vm    The runtime
```

Three layers, each with one concern. Each can be used standalone OR
composed.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-repl/` per the
arc-013 pattern. Same shape as wat-fmt / wat-lint / wat-cov /
wat-doc / wat-pause / wat-http-{server,router,client}.

```
wat-rs/crates/wat-repl/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   rustyline (line editing + history),
                       #   wat-edn (printing values)
  src/                 # Rust shim (rustyline frontend; eval dispatch)
  wat/repl/            # The REPL loop in wat;
                       # special commands (:type, :source, :doc, etc.)
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
                       # (driving the REPL programmatically)
```

## The interface

### CLI

```
wat repl                       # start REPL with batteries-only context
wat repl my-program.wat        # load my-program.wat, then start REPL
                               # with its symbols visible
```

### Embedded form

```scheme
(:wat::repl::start
  :context current-context     ; required; the eval context
  :prompt "wat> "              ; optional; prompt string
  :history-file "~/.wat_history") ; optional; history persistence
;; => :Result<:Unit, :ReplError>
```

A wat-pause break invocation calls into this with the captured
paused-program context — that's the integration point.

### Special commands (in-REPL)

| Command | Effect |
|---|---|
| `:help` | List commands |
| `:type <expr>` | Show the type signature of the expression |
| `:source <fqdn>` | Show source of a function |
| `:doc <fqdn>` | Show docstring of a function |
| `:reload` | Re-source files from disk (rebuild frozen world) |
| `:quit` | Exit |

These are GENERIC REPL commands. Pause adds its OWN commands
(`:where`, `:up`, `:down`, `:continue`, etc.) when its session is
running on top of wat-repl.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: rustyline integration, eval dispatch, special commands, context binding, integration with wat-pause. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once wat-pause's architecture firms up.) |

## Conventions inherited

From the foundation-tier arcs and the recent application-tier arcs:

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: errors are typed; no exceptions
- Type contract enforces what convention would otherwise hope for
- Dependency doctrine: rustyline is the canonical Rust line-editing
  crate; couple to it deliberately

## Cross-references

- **arc 005 (wat-pause)** — the consumer. wat-pause attaches a
  wat-repl to a captured paused-program context. The relationship
  is: wat-pause owns the suspension; wat-repl owns the
  interaction.
- **wat-mcp** (planned arc) — exposes both wat-repl (freestanding)
  and wat-pause (attached) as MCP tools to Claude / agents
- **DEPENDENCY-DOCTRINE.md** — rustyline as the dep choice
- **arc 008 (wat-kwargs)** — the `:context` / `:prompt` /
  `:history-file` kwargs follow the kwarg pattern

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-repl` — universal noun for this kind of tool;
  matches the established toolkit naming rhythm (wat-fmt, wat-lint,
  wat-cov, wat-doc, wat-pause); no collision
- **Architecture:** sketched; design firms up via chat iteration
- **Slice plan:** not yet sized
- **Bar to graduate to a real wat-rs arc:**
  1. wat-pause (arc 005) has shipped slice 1 (which can ship a
     pause-internal mini-REPL first; wat-repl extracts when both
     uses are firm)
  2. OR wat-repl ships standalone for editor/tutorial use cases
     and wat-pause's slice 1 builds on it
  3. User signals "let's start"

## Why this arc opened later than its foundation suggests

Honest accounting: wat-pause (arc 005) was designed before we
recognized wat-repl as a proper foundation. The original 005 design
had "the pry loop is itself a wat program shipped in
wat/std/pry.wat" — a standalone REPL that happened to live inside
the pry crate. The split into wat-repl + wat-pause makes that
internal-REPL into a proper foundation crate.

This is the same pattern as **arc 010 (wat-http-router) recognizing
arc 009 (wat-http-server) as its foundation** — except in our case,
the recognition came in the OPPOSITE direction: pry was designed
first; the REPL underneath was discovered later.

Numbers reflect when arcs OPENED. Layering is in the design.
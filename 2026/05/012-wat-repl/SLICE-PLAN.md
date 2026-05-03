# wat-repl — SLICE-PLAN

Sketch only. Not sized for shipping. The bar to graduate this arc
into a real `wat-rs/docs/arc/...` arc:

1. wat-pause (arc 005) has shipped slice 1 (so the integration
   pattern between pause's context capture and repl's attachment
   is firm), OR
2. wat-repl ships standalone first for tutorial / editor / mcp
   use cases, with wat-pause adapting to use it
3. User signals "let's start"

When that happens, this slice plan gets re-sized.

---

## Slice 1 — Bare REPL with rustyline

**Goal:** `wat repl` opens an interactive session against the
batteries-only context. Standard line editing works. History
persists.

**Done when:**
- `wat-rs/crates/wat-repl/` exists with arc-013 layout
- `wat repl` CLI works: prompts, reads, evaluates, prints, loops
- rustyline integrated for line editing; vi/emacs key bindings
- History persistence to `~/.wat_history` by default
- Multi-line input via paren balance tracking
- Special commands: `:help`, `:quit` / `:exit`
- Eval errors print and the loop continues
- Infrastructure errors (terminal failure; history file unwritable)
  surface as typed `ReplError`
- wat-tests covering: simple eval; multi-line input; history
  load/save; quit cleanly

**Out of scope for this slice:**
- Tab completion (slice 2)
- `:type` / `:source` / `:doc` commands (slice 3)
- Embedded form `(:wat::repl::start :context ctx)` (slice 4)
- wat-pause integration (slice 5)

---

## Slice 2 — Tab completion

**Goal:** TAB completes wat FQDNs from the SymbolTable.

**Done when:**
- `:wat::repl::completions :prefix str -> :vec<str>` wat function
  defined; walks SymbolTable for prefix matches
- rustyline completer wired to call this on every TAB
- Multi-segment FQDN completion works (`:wat::core::ar` → completes
  to `:wat::core::array`, `:wat::core::arc`, etc.)
- Locally-bound symbols completable (when context has lets)
- Documentation: completion behavior reference

---

## Slice 3 — Special commands

**Goal:** `:type`, `:source`, `:doc`, `:reload` work.

**Done when:**
- `:type <expr>` prints type signature without side effects
- `:source <fqdn>` prints function body source
- `:doc <fqdn>` prints docstring (per wat-doc, arc 006)
- `:reload` re-sources entry files; batteries stay frozen
- Each command is a wat function in `wat/repl/commands/`
- wat-tests cover each command's positive and negative paths
- Plugin extensibility verified: add a custom command via a
  battery; verify it's callable from the prompt

---

## Slice 4 — Embedded form + context binding

**Goal:** `(:wat::repl::start :context ctx)` works embeddably.

**Done when:**
- The embedded form takes a context parameter
- Eval happens in the passed context, not a default global
- A wat program can spawn a sub-REPL bound to a captured local
  scope (manually, without going through wat-pause yet)
- :stdin / :stdout / :stderr kwargs work for testing
  programmatically driving the REPL
- wat-tests cover: standalone eval in custom context; sub-REPL
  in a let scope sees the let bindings

---

## Slice 5 — wat-pause integration

**Goal:** wat-pause's `(:wat::pause::break)` opens a wat-repl
attached to the captured context.

**Done when:**
- wat-pause's slice 2 (or whichever slice ships break) imports
  wat-repl as a dep
- `(:wat::pause::break)` calls `(:wat::repl::start :context
  captured-env)`
- The `:continue` command (defined by wat-pause, not wat-repl)
  signals the loop to exit cleanly via a sentinel value
- After `:continue`, control returns to the program at the next
  form
- Round-trip integration test: pause a wat program; attach;
  evaluate against captured locals; modify a value; continue;
  verify the modification took effect

**Depends on:** wat-pause arc shipping enough for the integration.

---

## Slice 6 — Production hardening + polish

**Goal:** wat-repl is genuinely usable as the everyday wat REPL.

**Done when:**
- Color output via terminal detection (`is_terminal` check); flag
  override (`--color always` / `never`)
- Pretty-printing of nested structures (uses wat-edn's printer
  with indent)
- Type-aware printing (numbers; strings; vectors; structs; ASTs
  rendered distinctly)
- Performance: prompt-to-eval latency under 50ms for cold
  start; under 5ms steady-state
- Documentation: complete reference for all commands; key
  bindings; configuration

---

## Slices NOT planned

- **TCP attach / remote sessions** — sibling crate
  (`wat-repl-remote`?) if needed; out of scope for local
- **Notebook-style cell evaluation** — different shape; sibling
  arc if a real consumer asks
- **Syntax highlighting in input line** — slice-N enhancement;
  not core
- **Full IDE integration** — editors talk to wat-repl via a
  protocol; not wat-repl's job to provide editor integration

---

## Honest accounting

This slice plan is **sketched, not sized**. The biggest unknown:
how much of wat-pause's existing "internal pry loop" code maps
1:1 to wat-repl. The original wat-pause design (arc 005, before
the rename + split) had `wat/std/pry.wat` as a wat-implemented
loop. Some of that code becomes wat-repl; some stays
pause-specific.

Lean: extract aggressively into wat-repl. Anything that doesn't
require pause-specific knowledge (reading input; eval; printing;
generic commands) belongs in wat-repl. Pause keeps only the
suspension protocol, context capture, step controls, and pause-
specific commands.

The four-questions discipline applies to each slice independently.
Each slice should answer all four with honest checkmarks before
declaring the slice done.
# wat-repl — DESIGN

The wat interactive evaluator. Built on rustyline. The foundation
that wat-pause builds on.

---

## The four questions are the design compass

- **Obvious?** A wat program calling `(:wat::repl::start :context
  ctx)` opens a REPL bound to that context. Nothing hidden.
- **Simple?** One entry point. One responsibility (read-eval-print
  loop). Generic enough to be useful standalone; flexible enough
  to be embedded.
- **Honest?** Errors land as typed values; no exceptions. The
  REPL state is visible; nothing magical happens behind the
  user's back.
- **Good UX?** Familiar shape — anyone who has used `irb` /
  `python` / `node` knows what to do. Line editing, history,
  completion match standard expectations.

## Architecture

```
LAYER 4 — wat REPL loop      Read input → wat AST → eval → print
                             (wat code in wat/repl/loop.wat)
  ↓ uses
LAYER 3 — wat-vm eval        Evaluation in a captured context
                             (uses wat-rs's eval primitives)
  ↓
LAYER 2 — Rust shim          rustyline frontend; line editing;
                             history; tab completion; multiline;
                             color output (via wat-edn print)
  ↓
LAYER 1 — Rust ecosystem     rustyline (line editing); same
                             family as ripgrep, sccache, etc.
```

The layers compose cleanly. Layer 1 handles terminal interaction.
Layer 2 wraps Layer 1 in a wat-friendly shim. Layer 3 dispatches
to the wat-vm for evaluation. Layer 4 is the actual loop in wat.

## The wat-side interface

### Entry points

```scheme
;; Embedded: open a REPL with a given context
(:wat::repl::start
  :context current-context        ; required
  :prompt "wat> "                 ; optional; default "wat> "
  :history-file "~/.wat_history"  ; optional; default no persistence
  :stdin  std-stdin               ; optional; default ambient stdin
  :stdout std-stdout              ; optional; default ambient stdout
  :stderr std-stderr)             ; optional; default ambient stderr
;; => :Result<:Unit, :ReplError>

;; Single-shot eval (useful for `wat repl -e` or scripting)
(:wat::repl::eval-one
  :input "(+ 1 2)"
  :context current-context)
;; => :Result<:HolonAST, :EvalError>
```

The kwarg pattern (per arc 008) makes optional parameters
discoverable.

### Context type

The `:context` parameter is the eval context. For freestanding REPL
use, this is the batteries-only context (per the existing
wat-pause design — battery composition is established). For
embedded use (wat-pause attaching), it's the captured Environment
from the paused program.

The `:context` type is whatever wat-rs's eval substrate uses today
for "the namespace + bindings of an evaluation." This is the same
type wat-pause's `(:wat::pause::break)` captures.

### Error model

```scheme
(:wat::core::enum :wat::repl::ReplError
  ((InputError      (cause :HolonAST)))     ; line read failed
  ((HistoryError    (cause :HolonAST)))     ; history file I/O
  ((TerminalError   (cause :HolonAST)))     ; terminal capability
                                            ; (color; resize; etc.)
  ((InternalError   (cause :HolonAST))))    ; rustyline internal

;; Eval errors come from the wat-vm; not REPL's responsibility
;; to taxonomize. We surface them as values to the user.
```

The REPL doesn't taxonomize EVAL errors — those are the wat-vm's
domain. Eval errors are printed but the loop continues. Only
infrastructure errors (terminal failure; history file unwritable;
input stream closed) constitute a `ReplError` that bubbles out.

This matches the failure-engineering position: errors are typed;
each layer owns its taxonomy; the REPL doesn't pretend to
understand eval semantics.

## Special commands

| Command | What it does |
|---|---|
| `:help` | Print available commands |
| `:type <expr>` | Print the type signature of `<expr>` without evaluating its side effects |
| `:source <fqdn>` | Print the source of a function/value |
| `:doc <fqdn>` | Print the docstring of a function (per wat-doc, arc 006) |
| `:reload` | Re-source files from disk; rebuild the frozen world |
| `:quit` / `:exit` / Ctrl-D | Leave the REPL |

These are GENERIC. Pause adds its own commands (`:where`, `:up`,
`:down`, `:continue`, `:step`, `:next`, `:finish`) when its
session runs on top — pause-specific commands live in
wat-pause's crate, not here.

The command dispatcher is itself a wat function inside
`wat/repl/loop.wat`. Adding a command (whether by us or by a
battery extending the REPL) is just defining a wat function with
a matching name. **Plugin extensibility falls out of "the REPL
loop is a wat program" — no Rust additions needed.**

## How the loop works

```
loop {
  prompt    = print "wat> "
  raw-line  = rustyline.readline()       (Layer 2)

  if raw-line is a special command:
    dispatch to command handler         (Layer 4 wat fn)

  else if raw-line is an incomplete form:
    accumulate; show continuation prompt (Layer 4 reader peek)

  else:
    parsed-ast = wat-parse(raw-line)     (Layer 3 / wat-rs)
    result     = eval(parsed-ast, ctx)   (Layer 3 / wat-rs)
    formatted  = wat-edn::print(result)  (Layer 2 / wat-edn)
    print formatted
    push to history                       (Layer 2 / rustyline)
}
```

Multi-line forms are detected by tracking paren balance in the
reader. When unmatched, the prompt changes to a continuation
indicator (`...> `). When the form completes, eval fires.

History persistence: rustyline handles writing on REPL exit (or
on Ctrl-C). History file path is per-user (`$HOME/.wat_history`
default).

## Line editing — what rustyline gives us

Per dependency doctrine: don't reinvent line editing. rustyline
is the canonical Rust crate for this. Used by ripgrep, sccache,
nushell (originally), countless others. Hardened.

What we get:
- **vi / emacs key bindings** — toggleable via config
- **Multi-line input** — known from paren balance
- **History** — persistent across sessions
- **Tab completion** — pluggable; we provide a wat-FQDN completer
- **Search-back history** — Ctrl-R
- **Line editing** — Ctrl-A, Ctrl-E, etc.
- **Resize handling** — terminal-aware

What we add:
- **Wat-FQDN tab completer** — walks the SymbolTable for prefix
  matches (per the existing wat-pause design; this generalizes
  for both freestanding REPL and pause sessions)
- **Wat-syntax-aware paren matching** — knows about `:`-prefixed
  symbols, `()`/`[]`/`{}` types
- **Optional color output** — typed values get type-colored
  printing via wat-edn

## Tab completion

The completer is a Rust callback rustyline calls on every TAB. It
asks the wat-vm: "what symbols begin with this prefix?"

Implementation: `:wat::repl::completions :prefix str -> :vec<str>`
(per the existing wat-pause design — same primitive). The
completer is a thin Rust shim around this wat function.

For pause sessions, the completer ALSO sees locally-bound symbols
in the captured Environment (so `let` bindings inside the paused
function are completable). For freestanding REPL, only top-level
symbols complete.

## Integration with wat-pause

When `(:wat::pause::break)` fires:

1. wat-pause captures the current Environment + CALL_STACK
2. wat-pause calls `(:wat::repl::start :context captured-env)`
3. wat-repl runs the loop with the captured context as eval scope
4. User types `:continue` (a pause-added command, not a generic
   REPL command)
5. The pause-command handler returns a sentinel value
6. wat-repl's loop receives the sentinel and exits cleanly
7. wat-pause's break primitive returns
8. Original program resumes execution

The integration point is the `:context` parameter and the
sentinel-value protocol for `:continue`. Clean separation:
wat-repl knows nothing about pause-specific concepts (frames,
step controls); wat-pause knows nothing about REPL internals
(line editing, history).

## Per the four questions

- **Obvious?** ✅ — `wat repl` does what every developer
  expects; embedded `(:wat::repl::start :context ctx)` is one
  function with named parameters
- **Simple?** ✅ — one responsibility (REPL); one entry point;
  generic commands; pause-specific commands belong to pause
- **Honest?** ✅✅ — typed errors; eval errors print and continue
  (the loop is supposed to be resilient); infrastructure errors
  bubble; the loop is wat code so the dispatcher is inspectable
- **Good UX?** ✅ — familiar shape from every-language REPLs;
  rustyline gives standard line editing; tab completion is
  FQDN-aware; multi-line forms work transparently

Strong shape. Honest is ✅✅ because the typed error model is real
but not triple because rustyline has its own error semantics we
adopt without re-articulating.

## Cross-references

- **arc 005 (wat-pause)** — the consumer. wat-pause attaches a
  wat-repl to a captured context. Originally designed without
  recognizing wat-repl as a separate foundation; this arc is
  the extraction.
- **arc 008 (wat-kwargs)** — the `:context` / `:prompt` etc.
  kwargs follow the kwarg pattern
- **arc 006 (wat-doc)** — the `:doc <fqdn>` command pulls
  docstrings from the wat-doc surface
- **DEPENDENCY-DOCTRINE.md** — rustyline as a chosen dep;
  canonical Rust line-editing crate
- **wat-mcp** (planned arc) — exposes both freestanding REPL
  and pause-attached REPL as MCP tools to Claude

## Open architectural questions

A. **Multi-line input syntax.** Track paren balance only, or
   support explicit `\` line continuation? Lean: paren balance
   only — wat is parens-all-the-way-down; explicit continuation
   would be redundant.

B. **History across sessions.** Per-user only, or per-context
   (different histories for different paused contexts)? Lean:
   per-user for freestanding; pause-attached sessions use an
   ephemeral history (the paused context is short-lived).

C. **Color output.** Always-color, never-color, terminal-detect?
   Lean: terminal-detect (standard `is_terminal` check); user can
   force via `--color always` / `--color never` per
   convention.

D. **Reader prompt customization.** Hardcoded `wat> ` /
   `...> `, or themable? Lean: hardcoded for slice 1; theme
   support if real demand surfaces.

E. **`:reload` semantics.** Reload only the current entry's
   sources, or all batteries too? Lean: only entry sources;
   batteries are frozen at startup per arc 100.

## What's NOT in scope

- **Suspension / breakpoint primitives** — that's wat-pause
  (arc 005)
- **Step controls (`:next`, `:step`, `:finish`)** — pause-specific
  commands; wat-pause adds them when its session runs on top
- **Frame walking (`:where`, `:up`, `:down`, `:frames`)** —
  pause-specific commands
- **TCP attach / remote sessions** — out of scope for the local
  REPL; if needed, a sibling crate (`wat-repl-remote`?) ships later
- **Notebook-style cell evaluation** — different shape; sibling
  arc if a real consumer asks
- **Syntax highlighting in the input line** — slice-N enhancement;
  rustyline supports it but adds complexity
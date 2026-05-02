# Slice plan — five slices to feature-complete pry

Each slice closes a discrete capability. The order matters
because each slice's UX depends on the prior slice's substrate
work.

The first three slices give you the pry experience as the user
described it — bare session + `binding.pry` mid-program break +
clean terminal UX. ~10 days of work, mostly substrate, no
architectural moves.

Slices 4 and 5 are the debugger and the networked-attach
extensions. Both are deferred — ship when a real consumer
demands them.

---

## Slice 1 — Bare pry mode (`wat --pry`)

**What ships:**

- `crates/wat-pry/` — new workspace crate. Rust shims for the
  pry battery's primitives (most of which live in wat; this crate
  is the registration glue + the few primitives that need
  substrate-level Rust).
- Substrate primitives:
  - `:wat::pry::completions` (SymbolTable prefix walk)
  - `:wat::pry::ls`
  - `:wat::pry::show`
- `wat/std/pry.wat` — shipped via `wat_pry::wat_sources()`:
  - `:wat::pry::main` (the entry-point function the cli invokes)
  - `:wat::pry::serve` (the read-eval-print loop)
  - Command dispatchers (`:ls`, `:show`, `:reload`, `:exit`)
- `wat-cli` additions:
  - `--pry` argv flag.
  - Conditional pry-battery registration (gating; see
    `gating.md`).
  - Entry-point lookup swap: `:user::main` → `:wat::pry::main`
    when in pry mode.
  - Bare-session mode: `wat --pry` (no entry argument) freezes
    with batteries-only and invokes pry::main.

**What works at the end:**

- `wat --pry` opens a bare session against batteries.
- `wat --pry trade.wat` opens a session with the entry's defines
  visible.
- `:ls`, `:ls :prefix`, `:show :symbol`, `:exit` work.
- Tab completion (no rustyline yet — completion exists at the
  substrate primitive level; the cli's parent reads stdin via
  basic line-buffered I/O).
- Any wat expression typed at the prompt evaluates and prints.
- `(:user::main stdin stdout stderr)` is callable from the
  prompt to run the entry program.

**What doesn't work yet:**

- `(:wat::pry::break)` — primitive isn't shipped (slice 2).
- Line editing, history, multi-line input, fancy completion —
  those require rustyline (slice 3).
- Stepping — slice 4.

**Estimated effort:** 3-4 days.

**Acceptance bar:** `wat --pry` opens a session; `(:wat::std::math::sqrt 2.0)`
prints `1.414...`; `:ls :wat::std::math` lists the math
primitives; `:exit` closes the session cleanly.

---

## Slice 2 — `(:wat::pry::break)` and friends

**What ships:**

- Substrate primitives:
  - `:wat::pry::break-with-stdio` (the binding.pry primitive).
    Captures Environment + CALL_STACK; calls
    `:wat::pry::serve`; resumes on `:continue`.
  - `:wat::pry::where` (current break point's Span).
  - `:wat::pry::frames` (CALL_STACK snapshot).
  - `:wat::pry::last-error` (last panic via the cli's panic_hook).
- Substrate addition: `FrameInfo::env` field; `FrameGuard`
  captures Environment at frame entry. ~30 lines in `runtime.rs`.
- `wat/std/pry.wat` extensions:
  - `(:wat::pry::break)` macro expanding to
    `(break-with-stdio stdin stdout stderr)`.
  - Break-mode command dispatchers: `:continue`, `:where`,
    `:env`, `:frames`, `:up`, `:down`, `:wtf`.
  - Break-mode prompt rendering (`wat-pry (broken @ file:line:col)
    function-name>`).

**What works at the end:**

- `(:wat::pry::break)` in any function (with stdin/stdout/stderr
  in scope) drops to the break loop when execution hits it.
- All bare-mode commands work in break-mode too.
- `:env` lists captured locals with names + types + values.
- `:up` / `:down` walk the frame stack; eval scope follows.
- `:continue` resumes execution.
- `:wtf` shows the last error if execution panicked.

**What doesn't work yet:**

- `:next` / `:step` / `:finish` — slice 4.
- Multi-frame Environment introspection beyond the captured
  CALL_STACK depth.

**Estimated effort:** 3-4 days.

**Acceptance bar:** A wat program with `(:wat::pry::break)` in
the middle of a function, run via `wat --pry program.wat`
followed by `(:user::main ...)` at the prompt, drops the user
into a break with the function's locals visible. Typing
`:continue` resumes the function and returns its result.

---

## Slice 3 — Rustyline frontend in wat-cli (or wat-pry)

**What ships:**

- `rustyline` as a wat-cli (or wat-pry, see `packaging.md`)
  dependency.
- The cli's parent process reads from rustyline instead of
  bare stdin. Sends each line through the child's stdin pipe
  as it's typed.
- Multi-line input: when the lexer reports `UnclosedBracket`,
  the frontend shows a continuation prompt and accumulates
  until balanced.
- Tab completion: rustyline's `Helper` trait calls
  `:wat::pry::completions` via the EDN-over-pipe protocol
  (round-trip per TAB; ~milliseconds at d=10k).
- History persistence: `~/.wat_pry_history` written on each
  line, read at startup.
- Color / styled prompts using ANSI codes.

**What works at the end:**

- Full pry terminal UX. Up-arrow recalls history. TAB
  completes paths. `(` followed by Enter shows continuation
  prompt. Colors distinguish prompt / output / errors.

**What doesn't work yet:**

- Same as slice 2's "doesn't work yet."

**Estimated effort:** 2-3 days.

**Acceptance bar:** A user opens `wat-pry`, types
`:trading::ty<TAB>`, sees completions populate, picks one,
types `(<ENTER>` and gets a continuation prompt, completes the
expression, hits `<ENTER>` and sees the result. Up-arrow recalls
the previous command.

---

## Slice 4 (deferred) — Stepping

**What ships:**

- `wat/std/pry.wat` extensions:
  - `:next` — eval-step one form forward.
  - `:step` — eval-step into a sub-form.
  - `:finish` — eval-step until the current frame returns.
- Pry holds the current evaluation state between steps; the
  user can inspect after each step.

Built on `:wat::eval-step!` (arc 068, already shipped). The
substrate doesn't grow; only the pry loop's command set does.

**What works at the end:**

- Pry is a debugger, not just a probe. Walk through evaluation
  one form at a time; inspect at each pause.

**Estimated effort:** 2-3 days.

**Acceptance bar:** From a break point, `:next` advances one
form; `:env` shows updated locals (a let* binding might be
visible after stepping past its definition); `:finish` jumps to
the function's return value.

---

## Slice 5 (deferred) — TCP attach for remote sessions

**What ships:**

- `wat --pry --serve-tcp 9999` — backend binds to TCP, accepts
  one connection, runs the pry loop over the socket. Same EDN+
  newline protocol; different transport.
- `wat-pry --attach tcp://host:port` — frontend connects to a
  running backend via TCP. Same rustyline UX; different
  transport for the protocol.
- Authentication / authorization concerns (TLS? token?
  IP whitelist?). Open question; minimum viable might be "only
  bind to 127.0.0.1 by default; require a flag for external
  bind."

**What works at the end:**

- Open a pry session against a running production wat program
  on a remote machine. Inspect state, run queries, `:reload`
  if the source has changed on disk.
- The "spell" from BOOK Chapter 67 reaches the developer-tools
  layer — same protocol, different transport, multi-tenant by
  design.

**What doesn't work yet:**

- Multi-client attach. Unless explicitly designed in, the
  backend accepts one connection at a time.

**Estimated effort:** 4-5 days (more if security gets
complicated).

**Acceptance bar:** Run `wat --pry --serve-tcp 9999` on machine
A; connect from machine B with `wat-pry --attach
tcp://A:9999`; pry session works identically to local mode.

---

## Total effort estimate

- Slice 1: 3-4 days
- Slice 2: 3-4 days
- Slice 3: 2-3 days

Subtotal (full pry experience): ~9-11 days.

- Slice 4: 2-3 days (when demanded)
- Slice 5: 4-5 days (when demanded)

The first three slices are committed work. Four and five ship
when consumers ask for them.

## What this isn't

This isn't an architecture. The substrate has shipped most of
what pry needs through unrelated arcs over the past months. Pry
is a packaging layer that exposes existing capabilities through
a developer-friendly UX.

The user's recognition was load-bearing: *the substrate has been
collecting this capability arc by arc without anyone naming it.*
The slice plan is "name it, register a new battery, ship a wat
loop." That's all.

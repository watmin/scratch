# Dependency resolution — pry pristine, then MCP wraps it

The integration map. Built from the user's locked alignment
2026-04-29:

> "we do not take short cuts - we need pry to be pristine first,
> then build mcp to interface with it"

> "mcp interfaces with pry - pry exists to satisfy interactive
> exploration. mcp is how the agent does it, pry directly is how
> operators will use it"

This document describes the layered dependency tree: what
surfaces exist at each layer, what each layer depends on, and
the four-question validation gate between layers. Concrete
implementation details are deferred; surface integration is the
goal.

## The four-question gate

Each layer must satisfy four questions before the next layer
opens:

1. **Is this obvious?** A reader walking in cold can name what
   the layer does without consulting hidden context.
2. **Is this simple?** The concepts at this layer are
   unentangled — Hickey-simple. Each piece does one thing.
3. **Is this honest?** The layer's surface doesn't lie about
   what it provides. No latent capabilities; no missing
   protections.
4. **Is this a good UX?** The consumer of this layer (whether
   substrate code, the cli, or the human/agent) gets a clean
   interface that composes naturally.

If any question fails at any layer, work stops and we iterate
that layer until all four pass. We do not proceed to the next
layer with unresolved questions at a prior one.

## Locked alignments

These are the architectural commitments that don't move:

- **Pry is pristine first.** All pry primitives ship complete
  before MCP starts. Pry's substrate surface is the gate.
- **MCP interfaces with pry.** MCP doesn't reimplement
  introspection or break semantics — it wraps the pry
  primitives in a JSON-RPC envelope.
- **All inspection primitives belong to pry.** This includes
  `override-return`, `eval-in-frame`, frame walking, source
  reading — everything pry slice 2 of 005 names, plus the
  pieces 006 originally placed in MCP. The 006 scratch's
  earlier framing (those primitives as MCP-specific) was wrong
  — they're pry's.
- **Operators use pry directly. Agents use pry through MCP.**
  Same primitives; two consumers; one substrate.
- **Naming refactor doesn't block.** Today's substrate names
  are the references. The user has a migration path; whatever
  refactor lands later, post-pry will reconcile.
- **Deps aren't a constraint.** rustyline, serde_json, whatever
  — get them when needed.
- **Rustyline IS pristine.** The pry experience for operators
  isn't pristine without line editing, history, multi-line
  input, tab completion. The four questions don't pass without
  it.

## The layered tree

```
LAYER 0 — substrate state today
  ↓
LAYER 1 — pry substrate primitives (Rust)
  ↓
LAYER 2 — pry wat-level code
  ↓
LAYER 3 — pry cli integration
  ↓
LAYER 4 — pry frontend (rustyline)
  ↓
─────── PRISTINE PRY GATE (four questions) ───────
  ↓
LAYER 5 — wat-json (MCP prerequisite)
  ↓
LAYER 6 — MCP substrate additions (Rust)
  ↓
LAYER 7 — MCP wat-level code
  ↓
LAYER 8 — MCP cli integration
  ↓
─────── MCP GATE (four questions) ───────
  ↓
DONE
```

Each layer's surfaces and dependencies follow.

---

## Layer 0 — Substrate state (must hold before pry starts)

These are properties of today's substrate that pry depends on.
**Verify before slice 1 of pry opens.** If any are missing or
shaped differently than assumed, that's a prerequisite arc
before pry starts.

### Layer-0 surfaces

| Surface | Required shape |
|---|---|
| `Environment` | Runtime value; cheaply cloneable (Arc-shaped expected); passed as eval context to `eval_form` |
| `FrameInfo` | Carries Span (file/line/col); FrameGuard pushes/pops on entry/exit |
| `CALL_STACK` | Thread-local Vec<FrameInfo>; readable from the runtime |
| `SymbolTable` | Iterable; defines store enough metadata to reconstruct source |
| `eval_form` | Takes `&Environment`; honors constrained-eval invariants |
| `:wat::eval-edn!` | Polymorphic return per arc 102 |
| `:wat::edn::read` / `:wat::edn::write` | Working per arc 086 |
| Battery composition | `wat_cli::run(&[Battery])` per arc 100 |
| Conditional battery loading | Battery list assembled per cli flags before freeze |
| `:wat::core::define`'s body cache | Function source AST stored in symbol table for compilation |

### Layer-0 verification questions

Before opening Layer 1, the substrate-side verification:

- **`Environment` shape:** Is it Arc-shaped today? If not, slice
  1's "capture env cheaply" assumption changes. Verify before
  break primitive design.
- **`FrameInfo` field set:** Does it carry only Span, or already
  carry env? If only Span, slice 1 needs to extend it. Verify.
- **`SymbolTable`'s define body:** Is the parsed AST kept around
  per define? `:wat::pry::show` depends on this. Verify.

These verification questions are surfaced explicitly so the
user can check the substrate state before slice 1 starts.
Layer 0's surfaces are assumptions; the user confirms they hold
or names what's missing.

---

## Layer 1 — Pry substrate primitives (Rust)

The substrate-level Rust additions pry needs. These are the
load-bearing primitives that everything in layers 2-4 composes
over.

### Layer-1 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `FrameInfo::env` extension | Each frame carries its lexical scope at frame entry | Layer 0 (FrameGuard, Environment) |
| `:wat::pry::break-with-stdio` | Captures Environment + CALL_STACK; runs inline loop reading stdin | FrameInfo::env, Environment::clone |
| `:wat::pry::ls` | Walks SymbolTable; returns matching paths | SymbolTable iteration |
| `:wat::pry::show` | Reconstructs source for a symbol from cached AST | SymbolTable's body cache |
| `:wat::pry::completions` | Walks SymbolTable for prefix matches | SymbolTable iteration |
| `:wat::pry::where` | Returns current break point's Span | CALL_STACK |
| `:wat::pry::frames` | Snapshot of CALL_STACK with each frame's env | CALL_STACK + FrameInfo::env |
| `:wat::pry::up` / `:wat::pry::down` | Walk frames; switch eval context to a different captured frame | break-with-stdio |
| `:wat::pry::eval-in-frame` | Evaluate an expression in any captured frame's env | break-with-stdio + FrameInfo::env |
| `:wat::pry::override-return` | Force-return from a paused frame with a specific value | break-with-stdio |
| `:wat::pry::continue` | Resume from break (returns from break-with-stdio with `:()`) | break-with-stdio |
| `:wat::pry::last-error` | Read the cli's panic-hook output (arc 016) | Existing panic hook |

### Layer-1 dependencies

`FrameInfo::env` is the FOUNDATION primitive in this layer.
Every break-related primitive depends on it. **It must land
first.**

After FrameInfo::env, the four primitives can land in any order:

- `break-with-stdio` (the load-bearing one)
- `ls` / `show` / `completions` (independent of break)
- `where` / `frames` (depend on break for capture; sat alongside)
- `up` / `down` / `eval-in-frame` / `override-return` / `continue`
  (all consume break's captured state)

A reasonable internal ordering:

1. FrameInfo::env extension (substrate touch)
2. ls / show / completions (independent SymbolTable readers; can ship in parallel)
3. break-with-stdio (the central primitive)
4. where / frames / up / down / eval-in-frame / override-return / continue (all depend on break)

### Layer-1 four-question gate

Before Layer 2 opens:

- **Obvious?** Each primitive's name says what it does. `:wat::pry::ls` lists; `:wat::pry::break-with-stdio` breaks given stdio. Pass.
- **Simple?** Each primitive does ONE thing. The break primitive captures + runs a loop; the readers read; the resume signals. Pass.
- **Honest?** No primitive lies. `override-return` admits it overrides. `continue` admits it resumes. The freeze invariant holds throughout — no primitive lets the user mutate the frozen world. Pass.
- **Good UX?** From wat code: `(:wat::pry::ls)` returns a Vec of symbols. `(:wat::pry::show :sym)` returns a String. `(:wat::pry::break)` (via macro, see Layer 2) drops you in. Pass.

If any of these fail, iterate Layer 1 before Layer 2 opens.

---

## Layer 2 — Pry wat-level code

The wat code that composes Layer 1's primitives into the human-
facing pry experience. Lives in `wat/std/pry.wat`, shipped via
the pry battery's `wat_sources()`.

### Layer-2 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `:wat::pry::break` (macro) | Sugar over `break-with-stdio` using lexically-scoped stdio names | break-with-stdio |
| `:wat::pry::serve` (loop) | Tail-recursive read-eval-print: read line, parse EDN, eval, write EDN, recurse | EDN read/write, eval-edn!, IOReader/IOWriter |
| `:wat::pry::dispatch` (command router) | Recognizes `:ls`, `:show`, `:continue`, etc. and dispatches | ls / show / continue / etc. |
| `:wat::pry::main` (cli entry-point) | Banner + invokes serve | serve |

### Layer-2 dependencies

`serve` is the central piece — both pry mode (rustyline-driven)
and break mode (called from break-with-stdio) use the same
serve loop. The break primitive's inline loop IS serve called
with the captured env as eval context.

`break` macro depends on the substrate primitive
`break-with-stdio` from Layer 1.

`dispatch` recognizes pry-specific commands and routes; it's
called inside serve before falling through to expression eval.

`main` is a thin wrapper — for pristine pry it's just `(serve
stdin stdout stderr)`. The wat-level main exists so the cli's
entry-point swap (Layer 3) has a wat symbol to invoke.

### Layer-2 four-question gate

- **Obvious?** A reader of `wat/std/pry.wat` sees serve as a
  read-eval-print loop, dispatch as a command router, break as
  a macro, main as the entry point. Names match shapes. Pass.
- **Simple?** Each function does one thing. serve doesn't dispatch;
  dispatch doesn't loop; main doesn't print. Composition is the
  pattern. Pass.
- **Honest?** No wat function pretends to do more than it does.
  Pass.
- **Good UX?** A user dropping `(:wat::pry::break)` into their
  code gets a working break point with no further setup; the
  cli's `--pry` flag drives the bare-mode session through the
  same serve. Pass.

---

## Layer 3 — Pry cli integration

The wat-cli's argv handling and entry-point lookup. This is
where the cli "becomes pry-aware."

### Layer-3 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `--pry` argv flag | Triggers conditional pry-battery registration + entry-point swap | wat-cli's argv parser |
| Pry battery (`crates/wat-pry/`) | Rust crate; `register()` installs Layer 1 primitives; `wat_sources()` returns `wat/std/pry.wat` | Layer 1 + Layer 2 |
| Entry-point lookup swap | `:user::main` → `:wat::pry::main` when `--pry` set | Battery composition + freeze pipeline |
| Bare mode (no entry argument) | `wat --pry` with no entry: freeze with batteries-only, invoke pry main | Conditional battery loading + freeze with empty user source |

### Layer-3 dependencies

The pry battery crate is the packaging — Rust-side `register()`
+ `wat_sources()` + crate `Cargo.toml`. Once the crate exists,
wat-cli adds it to the battery list when `--pry` is set.

The entry-point lookup swap is a single conditional in
wat-cli's `run()` — if pry mode, look up `:wat::pry::main`
instead of `:user::main` after freeze.

Bare mode (no entry argument) requires the cli to handle the
"no source given" case — instead of exiting with a usage
error, build the frozen world from batteries-only and invoke
pry main.

### Layer-3 four-question gate

- **Obvious?** `wat --pry` opens pry; `wat --pry trade.wat` opens
  pry with the entry's defines visible; `wat trade.wat`
  unchanged. Three commands; clear semantics. Pass.
- **Simple?** Adding pry mode adds one flag, one battery, one
  entry-point conditional. The substrate doesn't grow new
  mechanisms — it composes existing battery + freeze machinery.
  Pass.
- **Honest?** Without `--pry`, the cli has no pry surface — pry
  battery isn't registered; pry symbols don't exist; freeze
  fails on any `:wat::pry::*` reference. Production deployments
  without `--pry` host no pry internals. Pass.
- **Good UX?** Single flag. Composes with `wat <entry.wat>`'s
  argv shape. Bare mode and entry-loaded mode are the same
  command differing only in whether an entry is given. Pass.

---

## Layer 4 — Pry frontend (rustyline)

The terminal UX layer. Line editing, history, multi-line input,
tab completion. Lives in `crates/wat-pry/src/frontend.rs` (or
similar) within the pry battery's Rust code, OR in a separate
`crates/wat-pry-frontend/` crate.

### Layer-4 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `rustyline` crate dep | Terminal control (raw mode, history, completion) | Layer 3's pry battery (or its sibling crate) |
| Line editing | Standard rustyline: arrow keys, history navigation, edit-in-place | rustyline |
| History persistence | `~/.wat_pry_history` written per command, read at startup (XDG fallback) | rustyline |
| Tab completion via `:wat::pry::completions` | Round-trip to substrate per TAB; populate rustyline's completer | Layer 1's `completions` primitive |
| Multi-line input | Lexer's `UnclosedBracket` signal triggers continuation prompt; accumulate until balanced | Lexer state exposure |
| Prompt rendering | `wat-pry>` for bare; `wat-pry (broken @ file:line:col)>` in break mode | Substrate-level mode flag |
| Color / styled output | ANSI codes; `--no-color` flag for non-tty | rustyline |

### Layer-4 dependencies

rustyline goes in as a Rust dep on the pry battery crate (or
a sibling). The frontend reads from the user's terminal and
sends each line through the cli's child stdio pipe (Layer 3's
fork-and-proxy from arc 104).

Tab completion makes a round-trip: parent reads TAB; sends a
completion request to the child via the stdin pipe; child runs
`:wat::pry::completions`; sends result back via stdout pipe;
parent populates rustyline's suggestion list. Sub-millisecond
at typical d.

Multi-line input requires the lexer to expose its
"unclosed-bracket" state. The cli's parent buffers the user's
input until the lexer reports a balanced expression; only then
does it send the complete expression to the child for eval.

### Layer-4 four-question gate

- **Obvious?** Operator types at the prompt; arrow keys recall
  history; TAB completes paths; ENTER on incomplete expressions
  shows continuation. Standard terminal UX. Pass.
- **Simple?** rustyline is one crate; integration is at the
  cli's parent process; substrate untouched. Pass.
- **Honest?** The frontend doesn't pretend to add capabilities.
  Tab completion calls the substrate; multi-line input uses
  the lexer's state; everything's a thin wrapper over Layer 1
  primitives. Pass.
- **Good UX?** Yes. This IS the UX. Without it, pry feels like
  a dialect of `cat | sh` — line-buffered, no editing, no
  history. Pristine pry needs Layer 4. Pass.

---

## ─── PRISTINE PRY GATE ───

Before Layer 5 opens, the full pry experience (Layers 1-4)
must satisfy all four questions end-to-end. Not just per layer
— in composition.

### End-to-end validation scenarios

1. **Bare session.** `wat --pry`. Operator types
   `(:wat::pry::ls :wat::std)`; sees substrate symbols. Calls
   `(:wat::pry::show :wat::std::math::sqrt)`; reads source.
   Calls `(:wat::std::math::sqrt 2.0)`; sees result. Types
   `:exit`; cli closes cleanly.

2. **Entry-loaded session.** `wat --pry trade.wat`. Operator
   sees user-defined symbols. Calls `(:trading::compute-decision
   <test-candle>)`; sees result. Reloads via `:reload`; world
   rebuilds.

3. **Mid-program break.** Operator drops `(:wat::pry::break)` in
   `compute-decision`. Runs program. Hits break. Inspects
   locals via `(:wat::pry::env)`. Walks frames via `:up`.
   Tests counterfactual via `(:wat::pry::override-return
   :Action::Sell)`. Original call returns the override; sees
   downstream behavior.

4. **Tab completion.** Types `:trading::ty<TAB>`; rustyline
   shows completions; picks one; expression completes.

5. **Multi-line.** Types `(:wat::core::let* (((x :i64) 5))<ENTER>`.
   Continuation prompt appears. Types `  x)<ENTER>`. Result is
   `5`.

6. **Error display.** Types a malformed expression. Error
   prints with file:line:col + the rendered offending form.
   Pry continues.

If any of these fail any of the four questions, **return to
the failing layer and iterate.** Do not proceed to Layer 5
until pristine pry is sealed.

---

## Layer 5 — wat-json (MCP prerequisite)

The JSON read/write substrate. New crate `crates/wat-json/`
mirroring `wat-edn`'s shape (arc 086 / arc 003 sketch).

### Layer-5 surfaces

| Surface | Purpose |
|---|---|
| `:wat::json::read` (polymorphic typed) | Parse a JSON-encoded string into a wat Value of the caller-annotated type |
| `:wat::json::write` (polymorphic) | Serialize a wat Value as JSON-encoded string |
| Type mappings | `null ↔ :Option::None`; numerics; strings; arrays → Vec; objects → Struct/HashMap |
| Error variants | `ParseError`, `TypeMismatch`, `MalformedNumber`, etc. |

### Layer-5 dependencies

Wraps a Rust JSON crate (likely `serde_json`). Mirrors arc
086's edn pattern — type-aware parsing driven by SymbolTable
metadata; type-aware serialization driven by Value variant
tags.

### Layer-5 four-question gate

- **Obvious?** Same shape as wat-edn. Pass.
- **Simple?** One battery; two primitives; clear type mappings.
  Pass.
- **Honest?** Parse errors surface explicitly; type mismatches
  return Result::Err; nothing latent. Pass.
- **Good UX?** Round-trip works; structs/enums/parametrics
  serialize cleanly; matches the wat-edn experience. Pass.

---

## Layer 6 — MCP substrate additions (Rust)

The thin Rust additions MCP needs beyond pry. **Note: most
inspection primitives already exist from pry's Layer 1.** MCP
adds only what's specific to the JSON-RPC wire + suspended-
call coordination.

### Layer-6 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| Session registry | Process-global `HashMap<SessionId, PausedSession>` storing captured Environment + frames + suspended call ID + resume signal | Pry's break-with-stdio capture mechanism |
| Break-handler override | Substrate-level mechanism that lets MCP override pry's "drop to inline loop" behavior with "emit notification + suspend" | Pry's break-with-stdio |
| MCP session resume signal | Crossbeam channel pattern: suspended thread waits on a "resume" signal; agent's `:wat::pry::continue` triggers it | Session registry |

### Layer-6 dependencies

This is the load-bearing engineering for MCP. The pry break
primitive captures Environment + frames; pry's default behavior
is to drop into an inline loop reading stdin. **MCP's job is
to override that "drop into inline loop" behavior.**

The override is installed at battery registration. When `--mcp`
is set, the MCP battery's `register()` overrides the
break-handler — instead of running the inline loop, the
break-handler emits a JSON-RPC notification, registers the
session, and blocks on a crossbeam "resume" signal. When
`:wat::pry::continue` is invoked through MCP (via a follow-up
`wat-eval` call with the session ID), the substrate signals
the suspended thread to resume.

This is the only genuinely new mechanism MCP introduces. All
other MCP behavior (eval, ls, show, etc.) is just calling pry
primitives over the JSON-RPC wire.

### Layer-6 four-question gate

- **Obvious?** When break fires under MCP, an event goes out and
  the thread waits. Operator-mode break fires drops you in
  inline. Same primitive; mode-dependent handler. Pass.
- **Simple?** One handler-override mechanism. Substrate doesn't
  grow new break primitives; it grows ONE behavior toggle. Pass.
- **Honest?** Without MCP loaded, the override doesn't exist.
  With MCP loaded, the override is named in the battery's
  `register()` and visible to anyone reading the substrate
  state. Pass.
- **Good UX?** Pry programs are unchanged — same `(:wat::pry::break)`
  call, behavior depends on cli mode. The agent doesn't see
  break implementation details; they see notifications and
  resumes. Pass.

---

## Layer 7 — MCP wat-level code

The wat code that drives the JSON-RPC protocol. Lives in
`wat/std/mcp.wat`, shipped via the MCP battery's `wat_sources()`.

### Layer-7 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `:wat::mcp::main` | Cli's MCP-mode entry-point. Calls serve. | serve |
| `:wat::mcp::serve` | Tail-recursive JSON-RPC dispatch loop: read line, parse JSON, route by method, write response, recurse | wat-json, IOReader/IOWriter |
| `:wat::mcp::dispatch` | Routes JSON-RPC method (initialize, tools/list, tools/call, cancellation, etc.) | Layer 1 primitives via eval-edn |
| `wat-eval` tool implementation | Extract msg from JSON-RPC params, parse EDN, eval, serialize result | eval-edn!, EDN read/write |
| `wat-eval-stream` tool implementation | Same but uses MCP-mode break-handler; supports session parameter | Session registry, eval-edn! |
| `initialize` handler | Returns server capabilities | (none) |
| `tools/list` handler | Returns the 1-2 tool definitions | (none) |
| `notifications/pry/break` emission | When break-handler fires, emit notification with session ID + EDN-encoded break info | Session registry |

### Layer-7 dependencies

`serve` is the JSON-RPC sibling of pry's serve. Same shape;
different framing (JSON-RPC instead of bare EDN+newline).

`dispatch` routes by method. Most methods invoke pry
primitives via `eval-edn!` against the frozen world. The
`tools/call wat-eval` method is the most common; it extracts
the EDN payload, calls `eval-edn!`, returns the EDN result.

The `tools/call wat-eval-stream` method is the variant that
supports break — it invokes eval-edn! the same way but with
MCP-mode break-handler active. If the eval hits a break, the
notification fires and the suspended thread blocks until
resume.

### Layer-7 four-question gate

- **Obvious?** Same shape as pry's serve, with JSON-RPC instead
  of EDN as the wire format. Pass.
- **Simple?** Dispatch routes by method; each handler is a
  small function; the substrate's existing primitives do the
  work. Pass.
- **Honest?** Tool definitions don't lie about what's available.
  `wat-eval` says "evaluate any wat expression"; agent uses
  introspection to discover the actual surface. Pass.
- **Good UX?** Agent has one tool to learn. All of wat's
  expressivity is reachable through it. Pass.

---

## Layer 8 — MCP cli integration

The wat-cli's MCP-mode wiring. Mirrors Layer 3 (pry cli) with
appropriate substitutions.

### Layer-8 surfaces

| Surface | Purpose | Depends on |
|---|---|---|
| `--mcp` argv flag | Triggers conditional MCP-battery registration + entry-point swap | wat-cli's argv parser |
| MCP battery (`crates/wat-mcp/`) | Rust crate; `register()` installs Layer 6 + handler override; `wat_sources()` returns `wat/std/mcp.wat` | Layer 5 (wat-json), Layer 6, Layer 7 |
| Entry-point lookup swap | `:user::main` → `:wat::mcp::main` when `--mcp` set; `:wat::pry::main` when `--pry` set; MCP wins if both | Battery composition + freeze pipeline |
| `--pry --mcp` combination | Both batteries load; MCP main runs; pry primitives reachable through MCP | Layer 3 + Layer 8 batteries |

### Layer-8 dependencies

Same mechanism as Layer 3: argv flag → conditional battery
registration → entry-point swap.

The `--pry --mcp` combination is the standard developer
flag-set: agent connection plus pry primitives loaded for
break support. Pry battery registers first (installs the
default break-handler); MCP battery registers second
(overrides break-handler with the notification-emitting
variant).

### Layer-8 four-question gate

- **Obvious?** Single flag opens MCP mode; combined with `--pry`
  enables break support. Same shape as pry's flag mechanism.
  Pass.
- **Simple?** No new mechanism beyond what pry's cli integration
  established. Pass.
- **Honest?** Without `--mcp`, the MCP battery isn't loaded;
  MCP symbols don't exist; freeze fails on `:wat::mcp::*`
  references. Production-safe. Pass.
- **Good UX?** Three modes (none, --pry, --mcp), four flag
  combinations (none, --pry, --mcp, --pry --mcp), each with
  clear semantics. Pass.

---

## ─── MCP GATE ───

Before declaring MCP shipped, the integration must pass the
four questions end-to-end across pry + MCP combined.

### End-to-end validation scenarios

1. **Bare MCP session.** `wat --mcp`. Agent connects;
   `tools/list` shows wat-eval; agent calls
   `wat-eval (:wat::std::math::sqrt 2.0)`; receives `1.414...`.

2. **Entry-loaded MCP session.** `wat --mcp trade.wat`. Agent
   discovers symbols via `wat-eval (:wat::pry::ls :trading)`,
   reads source via `wat-eval (:wat::pry::show :sym)`, calls
   functions, receives results.

3. **Break under MCP.** `wat --pry --mcp trade.wat`. Agent calls
   `wat-eval-stream (:trading::compute-decision <candle>)`. Hits
   `(:wat::pry::break)` inside the function. Notification
   arrives. Agent calls `wat-eval session=X (:wat::pry::env)`;
   sees locals. Agent calls `wat-eval session=X
   (:wat::pry::continue)`; suspended call returns; agent
   receives the function's result.

4. **Counterfactual under MCP.** Agent in a break calls
   `wat-eval session=X (:wat::pry::override-return :Action::Sell)`.
   Original `wat-eval-stream` returns `:Action::Sell`. Same
   primitive used by pry mode for counterfactual debugging;
   agent gets identical capability through MCP.

5. **Mixed modes.** Operator opens `wat --pry --mcp trade.wat`
   from a terminal; the agent connects via MCP from a different
   process. Both have full pry surface access; both can hit
   breaks; the substrate manages session isolation.

If any of these fail any of the four questions, return to the
failing layer.

---

## What's NOT in this document

- **Slice-by-slice work breakdown.** That's in 005's and 006's
  slice-plan.md files. This document is layer-by-layer surface
  alignment.
- **Concrete impl details.** Per the user's directive: "we'll
  work out concrete impl details when we get there." This is
  surface alignment.
- **Effort estimates.** That's in 005's and 006's slice plans.
- **Open questions.** Those are in 005's and 006's
  open-questions.md files. The dependency tree itself is
  locked enough to commit; remaining questions are within
  layers, not between them.

## Summary

Eight layers. Two validation gates. Surface integration:

**Layers 1-4: pristine pry** (substrate primitives → wat code →
cli integration → frontend). Four questions per layer; four
questions in composition. Pristine when all pass.

**Layers 5-8: MCP wraps pry** (wat-json prerequisite → MCP
substrate handler-override → wat code → cli integration). MCP
adds one substrate mechanism (the break-handler override + session
registry); everything else reuses pry primitives via the EDN-in-
JSON-envelope pattern.

**The integration contract:** MCP is the JSON-RPC envelope around
pry. Same primitives, two consumers, one substrate. The four
questions hold at every layer. We don't proceed past a failing
gate.

This is the dependency tree that makes pry pristine and MCP a
clean wrapper. Every layer's surface is aligned with its
dependencies. Concrete impl details follow the surface
alignment; we don't choose impl until the surfaces lock.

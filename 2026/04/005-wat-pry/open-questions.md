# Open questions

What's locked and what's not, as of 2026-04-29 conversation
close. Every question that came up explicitly during the
conversation gets a row here, with the resolution if reached
or a tentative lean if not.

## Locked decisions (recap from README's table)

- **Vocabulary:** the word is *pry*, not REPL. Flag is `--pry`.
  Namespace `:wat::pry::*`. Entry-point `:wat::pry::main`.
- **Two modes, one freeze pipeline:** bare (`wat --pry`) vs
  entry-loaded (`wat --pry <entry>`). Same code path; differ
  only in whether an entry source is loaded.
- **Gating:** pry battery registered only when `--pry` set.
  Without the flag, freeze fails on any pry reference.
- **No `cd`:** wat is FQDN everywhere. Tab completion is the
  navigator.
- **The break primitive captures Environment + CALL_STACK; runs
  the pry loop with the captured scope; resumes on `:continue`.**
- **The pry loop is wat code, not Rust.** Lives in
  `wat/std/pry.wat`. Extensible by batteries.
- **`:user::main` callable from the prompt.** It's just a
  symbol in the frozen world.

## Open — substrate-level

### 1. `:wat::pry::main` — wat-shipped or Rust shim?

**Lean:** wat-shipped (in `wat/std/pry.wat`).

The cli's pry-mode entry-point. Could be either:

- **Rust shim:** `crates/wat-pry/src/lib.rs::pry_main` does the
  banner-print + signal-handling + delegation to `serve`.
- **Wat-level:** `wat/std/pry.wat::wat::pry::main` does the same
  in wat code, calling `:wat::io::IOWriter/println` for the
  banner and `:wat::pry::serve` for the loop.

Wat-level is more honest (pry is "just a wat program"). Lean
toward wat-level unless there's a concrete reason Rust-level is
needed (e.g., terminal control codes that need a Rust crate).

**Resolves when:** slice 1 implementation surfaces whether wat
can do the banner cleanly. Most likely yes.

### 2. `FrameInfo::env` — sized or shared?

The substrate addition: each `FrameInfo` records its
Environment at frame entry. Question: does the FrameInfo own
the Environment (sized, more memory) or hold an Arc clone
(shared, possible cycle)?

**Lean:** Arc clone. The Environment is already Arc-shaped;
cloning an Arc is constant-time; no cycles because parent
pointers go up only.

**Resolves when:** slice 2's runtime addition lands; pick the
shape, run benchmarks if it matters.

### 3. Pretty-printing — substrate or wat-level?

`:wat::edn::write` produces compact EDN today. Pretty-printing
would indent maps, vectors, nested structures. Two paths:

- **Substrate:** `:wat::edn::write-pretty` as a separate primitive,
  or `(write value :pretty true)` as an option.
- **Wat-level:** A wat function `:wat::pry::pretty-print` that
  walks an EDN string and re-formats. Slower but no Rust
  changes.

**Lean:** start wat-level for pry's needs (slice 1); promote to
substrate if a real consumer wants pretty-printing for telemetry
output or other use cases.

**Resolves when:** slice 3 (rustyline frontend) — pretty-printing
becomes user-visible there.

### 4. Doc-strings — `///` syntax addition?

Pry's `show-doc` reads doc-strings attached to functions /
classes. Wat doesn't have doc-string syntax today. To support
it:

- Parser change: recognize `///`-prefixed lines preceding a
  define as the function's doc.
- SymbolTable change: each function entry gains an
  `Option<String>` for doc.
- `:wat::pry::show` extension: print the doc above the
  signature.

**Lean:** defer. Slice 1's `:show` prints type signature +
source body; that covers 90% of the help use case. Doc-strings
are a polish addition, separate arc.

**Resolves when:** users start asking "where do I put the
explanation of this function?" The cue will surface naturally.

### 5. Multi-client TCP attach (slice 5) — defer the protocol?

Slice 5 lets one frontend attach to one backend over TCP. What
about multi-client — multiple frontends on the same backend?

- Naive: backend serializes; one client at a time; second
  client gets "session in use" error.
- Multi-tenant: backend spawns a child pry session per
  connection (each is its own forked child of the backend);
  state is shared (same FrozenWorld) but interaction is
  isolated. Each connection gets its own EDN+newline protocol
  stream.

**Lean:** defer to slice 5+. The first version handles single-
client; multi-client is a separate arc when consumers demand it.

**Resolves when:** there's a real multi-developer use case. For
now, the "attach to a running production wat program" usage is
single-developer.

### 6. Pry sessions and concurrent threads in user code

If the user's program spawns multiple threads (via
`:wat::kernel::spawn`) and one of them hits
`(:wat::pry::break)`, what happens?

- Naive: the breaking thread reads from the program's stdin /
  writes to stdout, blocking other threads' stdio. Other
  threads continue executing in the background.
- Disciplined: pry break acquires a process-wide "pry lock";
  other threads block on stdio access until the break ends.

**Lean:** naive. The substrate doesn't have a global stdio
lock; concurrent stdio access is the user's problem to manage.
Pry break just reads/writes the same way any other code does.
If the user wants single-threaded inspection, they design their
program that way.

**Resolves when:** a real multi-threaded program demands more
disciplined handling.

## Open — UX

### 7. Continuation prompt for multi-line input — what triggers it?

Slice 3 (rustyline) shows a continuation prompt when the user's
input is incomplete. What's the signal?

- **Lexer-based:** call the lexer on each line; if it returns
  `UnclosedBracket`, show continuation. Cheap; depends on
  exposing the lexer's incremental state.
- **Bracket-counting:** count `(` and `)` and `[` and `]`; if
  imbalanced, show continuation. Simple; ignores strings (a
  `(` inside a string would mis-count).
- **Try-parse:** try to parse the buffer; if parse fails with
  EOF, show continuation; if other failures, show parse error.
  Slowest but most correct.

**Lean:** try-parse. Slow at the per-keystroke level isn't an
issue (only fires on Enter, not on every keystroke). Most
correct shape.

**Resolves when:** slice 3 implements rustyline integration.

### 8. Prompt color / theming

The base prompt is `wat-pry>`. In break-mode it becomes
`wat-pry (broken @ file:line:col) function-name>`. Should
colors highlight (red break header, green prompt, etc.)?

**Lean:** yes for slice 3 polish. Use ANSI codes; provide a
`--no-color` flag for non-tty contexts.

**Resolves when:** slice 3.

### 9. History file location — `$HOME/.wat_pry_history` vs `$XDG_STATE_HOME/wat-pry/history`

XDG basedir spec is the right answer; `$HOME/.wat_pry_history`
is the convenient one. Both are common.

**Lean:** XDG, with `$HOME/.wat_pry_history` as fallback if
XDG vars aren't set.

**Resolves when:** slice 3.

### 10. `:reload` semantics — full kill or in-place reload?

When the user types `:reload` in pry mode, two interpretations:

- **Full kill:** the cli's parent kills the child, re-forks
  with the (re-read) entry source. New frozen world; previous
  session's state is gone.
- **In-place rebuild:** the child receives a signal, re-runs
  freeze in-place, replaces its symbol table. Previous
  session's state (e.g., open sqlite handles) might survive or
  not.

**Lean:** full kill. Honest; predictable; matches "freeze is
once" discipline. In-place rebuild would require a "freeze
twice" capability the substrate doesn't have.

**Resolves when:** slice 1 implements `:reload`.

### 11. Should bare-pry have a way to load source after start?

`wat --pry` opens with no entry. Should the user be able to
type `:load /path/to/file.wat` at the prompt and have its
defines added?

**Lean:** no. That would violate the freeze invariant — at the
prompt, you can't add new defines. The `:load` would have to
trigger a `:reload`-like full-rebuild with the new file as part
of the corpus.

If we DO want this, the cleanest shape is "bare-pry remembers a
list of loaded files; `:load` adds to the list and triggers
`:reload`." Future addition; not slice 1.

**Resolves when:** a user asks for it.

## Open — gating edge cases

### 12. Programs that conditionally include pry forms

User code that says "only call `(:wat::pry::break)` if some
debug flag is set" — is that supported?

The strict gating answer: no. Pry forms are illegal at freeze if
`--pry` isn't set; conditional code paths don't help because
freeze rejects the symbol on resolve.

If the user wants conditional debugging behavior, they ship
their own `(:my::debug-break)` that checks a flag and either
calls `(:wat::pry::break)` or no-ops. But the conditional form
itself can only freeze in `--pry` mode.

**Lean:** keep the strict gate. Document the workaround.

**Resolves:** locked. The trade is that pry is a developer-time
tool, not a runtime feature.

### 13. Pry forms in third-party batteries' wat sources

A battery's `wat_sources()` could in principle include pry
forms. If the battery is registered in non-pry mode, freeze
fails on the pry references.

**Lean:** batteries are responsible for not shipping pry forms
in their core sources. If a battery wants pry-aware features,
it ships them in a separate wat file that's only loaded when
the pry battery is also loaded (the user opts in by linking
both).

**Resolves:** convention only. Document in the battery-author
guide.

### 14. Test code that uses pry — how does it register?

`wat::test!` macros run programs in sandboxed in-process freezes.
If a test wants to exercise pry primitives:

```rust
#[test]
fn pry_break_round_trip() {
    let mut deps = wat::rust_deps::with_wat_rs_defaults();
    wat_pry::register(&mut deps);
    // ... test that exercises (:wat::pry::break) ...
}
```

The test opts in explicitly. The pry battery is just another
battery; tests register it the same way the cli does.

**Lean:** documented in the test-harness section of pry's
USER-GUIDE entry.

**Resolves:** convention. No special test-mode at the substrate.

## What's NOT a question

Anything in the README's "What's locked" table — those are
decisions, not open questions. Anything in `slice-plan.md`'s
"What works at the end" sections — those are committed
acceptance bars, not unknowns.

The 14 questions above are the load-bearing unknowns. Most have
leans documented; resolution is "first slice that surfaces the
question." None block opening the wat-rs arc.

# `(:wat::pause::break)` — the binding.pry shape

User's framing 2026-04-29:

> "binding.pry is /the most/ powerful thing i've ever seen in a
> program..."

> "i want the binding.pry experience.... being able to measure my
> environment and continue..."

This is the load-bearing primitive. Everything else in the pause
surface is convenience around the break form. The freeze invariant
is what makes this work cleanly — when you `:continue`, the
program you continue into is *exactly* the program you inspected.

## The shape at the call site

```scheme
(:wat::core::define
  (:trading::compute-decision (candle :Candle) -> :Action)
  (:wat::core::let*
    (((rsi    :f64)    (:trading::rsi candle))
     ((vol    :f64)    (:trading::vol candle))
     ((regime :Regime) (:trading::classify rsi vol)))

    (:wat::pause::break)   ;; ← stop here

    (:trading::action regime rsi vol)))
```

When execution hits the form, the substrate:

1. **Captures the current Environment** — every let*-bound name
   plus every function arg in scope. `candle`, `rsi`, `vol`,
   `regime`, plus parents. The Environment is already a runtime
   value (Arc-shaped, parent-pointer-chained); capturing is a
   single Arc clone.
2. **Captures the CALL_STACK** — arc 016 already populates a
   thread-local Vec<FrameInfo> on every function entry/exit. Pause
   reads it directly.
3. **Hands control to the pause loop** — reads from the program's
   stdin (which, under `wat --pause`, is the user's terminal via
   the cli's stdio proxy), evaluates each line against the
   captured Environment, writes results to stdout.
4. **Resumes on `:continue`** — the loop returns from
   `pause::break` with `:()`. The function continues at the next
   form. Execution is exactly where it was; the rest of the body
   evaluates against the same bindings the inspection saw.

## The wat-honest shape — explicit stdio + macro sugar

The substrate primitive takes stdio handles explicitly:

```scheme
(:wat::pause::break-with-stdio
  (in  :wat::io::IOReader)
  (out :wat::io::IOWriter)
  (err :wat::io::IOWriter)
  -> :())
```

This is the substrate-honest form. Wat's discipline is "pass what
you need" — no thread-locals capturing implicit context.

The pause-shaped UX comes from a defmacro shipped in
`wat/std/pause.wat`:

```scheme
(:wat::core::defmacro
  (:wat::pause::break) -> :wat::WatAST
  `(:wat::pause::break-with-stdio stdin stdout stderr))
```

Now `(:wat::pause::break)` works wherever `stdin`/`stdout`/`stderr`
are visible — which is `:user::main`'s body and any function that
threads them as args. For functions without them in scope, the
caller passes them explicitly through the call chain (or uses the
verbose form directly).

This matches Ruby's `binding.pry`: takes no args, looks up the
ambient stdio. The wat version makes the ambient lookup
syntactic (the macro grabs lexically-scoped names) rather than
semantic (no thread-local). Honest about what the symbols are.

## What the user sees inside a break

```
$ wat --pause trade.wat
wat-pause> (:user::main stdin stdout stderr)
[program executes, hits the break]
wat-pause (broken @ trade.wat:42:7) compute-decision>

  candle  =  #trading.types.Candle {:open 50000.0 :high 50500.0 :low 49500.0 :close 50250.0 :volume 1.5}
  rsi     =  0.6234
  vol     =  0.0083
  regime  =  :Regime::Trending

wat-pause (broken @ trade.wat:42:7) compute-decision>
```

The header shows file:line:col + current function name. The
locals readout is what `:env` returns by default at break-time;
configurable via a setting.

User can now type any expression:

```
wat-pause (broken)> rsi
0.6234

wat-pause (broken)> (:trading::action regime rsi vol)
:Action::Buy

wat-pause (broken)> (:wat::pause::frames)
[FrameInfo @ trade.wat:42 compute-decision,
 FrameInfo @ trade.wat:120 :user::main,
 FrameInfo @ wat-cli runtime entry]

wat-pause (broken)> :up
[walked to :user::main frame]
wat-pause (broken @ trade.wat:120) :user::main>

wat-pause (broken)> :down
[walked back]
wat-pause (broken @ trade.wat:42:7) compute-decision>

wat-pause (broken)> :continue
[returns from break; function body continues]
:Action::Buy
wat-pause>
```

`:continue` resumes execution. The function returns whatever the
rest of the body produces. Pause prompt reappears at the top level.

## The capture mechanism — what the substrate must do

`eval_form` for `:wat::pause::break-with-stdio` in
`src/runtime.rs` does:

```rust
fn eval_pause_break_with_stdio(
    args: &[Value],
    env: &Environment,
    world: &FrozenWorld,
) -> Result<Value, RuntimeError> {
    let stdin  = expect_io_reader(&args[0])?;
    let stdout = expect_io_writer(&args[1])?;
    let stderr = expect_io_writer(&args[2])?;

    // Capture the current environment + frames
    let captured_env = env.clone();           // Arc clone, cheap
    let captured_frames = CALL_STACK.with(|s| s.borrow().clone());

    // Run the inline pause loop
    pause_loop(world, &captured_env, &captured_frames, stdin, stdout, stderr)?;

    Ok(Value::unit())
}
```

`pause_loop` reads lines, parses, evals against `captured_env`,
writes results, recognizes `:continue` / `:exit` / `:up` / etc.
The loop returns when the user types `:continue`; then
`eval_pause_break_with_stdio` returns `Value::unit()` and the
calling form's evaluation continues normally.

Critically: `captured_env` is the SAME Environment the calling
form's evaluator already had. We don't construct a new evaluator;
we just run a sub-evaluator in the same scope. Names resolve
exactly as they would in the calling code. Function calls
dispatch the same way. The only difference is that the input
comes from the user's terminal instead of the source AST.

## Why the freeze invariant is load-bearing

Ruby's `binding.pry` captures a binding (lexical scope) and runs
a pause against it. But Ruby is mutable — while the user inspects,
another thread can mutate locals via shared references; methods
the user reads via `show-source` can be redefined out from under
inspection; constants can change.

Wat's `(:wat::pause::break)` captures an Environment in a frozen
world. While the user inspects:

- The Environment is an Arc-cloned snapshot. The original keeps
  evolving in the calling thread (it isn't, because pause is
  single-threaded inline, but even in principle).
- Function definitions are immutable in the frozen world.
  `:show :trading::compute-decision` shows the same source
  before, during, and after the break.
- Type signatures are immutable. The set of callable symbols is
  what it was at freeze-time.
- No thread can redefine a function or reassign a constant —
  the substrate doesn't have those operations at runtime.

When the user types `:continue`, execution resumes against
exactly the program the user just inspected. No drift. No
"oh wait, did the function I just looked at get redefined?"
The freeze invariant turns pause into a static-time inspection
running over runtime values.

This is the user's point: **rust being frozen is a blessing,
not a curse**. Ruby has to fight its own mutability to make pause
reliable; wat inherits the substrate's stability for free.

## Multi-frame inspection — `:up` and `:down`

The CALL_STACK is a Vec<FrameInfo>; index 0 is the entry frame,
the last index is the current frame. `:up` walks toward index 0;
`:down` walks back toward the current.

Each frame has its own Environment (the let-bindings + args at
that frame's call site). `:up` switches the pause loop's eval
context to the parent frame's Environment. The user can inspect
caller's locals, evaluate expressions in the caller's scope,
walk further up through parents.

The CALL_STACK already carries this — arc 016 populated frames
with file:line:col but not Environment. To make `:up`/`:down`
work, the runtime needs to also record each frame's Environment
in `FrameInfo`. That's a small addition: `FrameInfo` gains an
`env: Environment` field; FrameGuard captures it at frame entry.
Cheap (Arc clone); no semantic change to existing failure-trace
output (which doesn't read env).

When the user walks back to the break-point frame and types
`:continue`, execution resumes from the break form. Walking up
during inspection doesn't move the program counter; it only
moves the pause loop's eval scope.

## What break does NOT do

- **Modify the program.** No define / defmacro / struct / enum
  / typealias / load. Same constrained-eval rule that applies
  to all wat eval sites. The freeze invariant holds at the
  prompt.
- **Write to the captured Environment.** You can compute new
  values; you can pass them as arguments to functions; you
  cannot rebind `rsi` in the captured scope. (Wat doesn't have
  rebinding syntax; let* shadows by introducing a new binding,
  which would only be visible inside the let*'s body.)
- **Step into a function from the prompt.** Stepping is slice 4
  via arc 068's `eval-step!`. The break primitive itself is just
  capture-and-run-loop; stepping is composed on top.

## Cost

The break primitive is small at the substrate:

- One new dispatch arm in `runtime.rs::eval_form` matching
  `:wat::pause::break-with-stdio`.
- `Environment::clone` and `CALL_STACK.borrow().clone()` —
  existing infrastructure.
- An inline loop that reads, parses, evals, writes — uses
  existing `:wat::edn::read` and `:wat::edn::write` and
  `:wat::eval-edn!`.
- Recognition of pause-command keywords (`:continue`, `:up`,
  etc.) at the wat level, in `wat/std/pause.wat`.

Estimated ~300 lines of Rust + ~150 lines of wat. The pause loop
(`:wat::pause::serve`) is shared between bare-mode and break-mode;
break-mode just calls it with the captured Environment as
context.

The big architectural moves were already made (arcs 099-104).
The break primitive composes existing pieces.

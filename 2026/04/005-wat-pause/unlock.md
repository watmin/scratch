# The unlock — what arcs 097–104 made true

Pause needs three things from its host:

1. **A read-eval-print loop primitive** — read source, parse, evaluate,
   render result, loop.
2. **A frozen evaluation context** — somewhere to evaluate expressions
   against. The host's "what symbols exist."
3. **A wire format** — bytes-in, bytes-out for transporting expressions
   and results between user and runtime.

Pre-arc-097 the substrate had pieces of (1) — `:wat::eval-edn!` and
`:wat::eval-ast!` exist, so single-shot evaluation works. The
`startup_from_source` machinery already produces (2) — every
`wat <entry.wat>` invocation builds a `FrozenWorld`. What was
missing was (3) at the right granularity, and a clean way to keep
the user's program's universe (batteries + their wat sources) but
swap the entry-point.

Arcs 097-104 closed both gaps.

## The chain

**Arc 099 — `crates/wat-cli/`** — The cli moves out of the substrate
crate. wat-rs becomes library-only; wat-cli is the canonical
batteries-included consumer. The cli is now a single,
inspectable, compositionally-built artifact.

**Arc 100 — wat-cli public API** — `wat_cli::run(&[Battery])`
exposed. Downstream consumers (and us) can build alternative
front-ends that share wat-cli's machinery — argv parse, freeze
pipeline, signal handlers, exit codes. **`wat-pause` is one such
alternative front-end.**

**Arc 101 — kill `wat test`** — single-purpose CLI. The cli is now
"parse argv, run one entry program." Adding a second mode
(`--pause`) is symmetric and doesn't fight a third (`test`). The
clarity of the single shape makes the second shape honest.

**Arc 102 — `:wat::eval-ast!` polymorphic return** — schema flipped
from `Result<:HolonAST, EvalError>` to `Result<:T, EvalError>`.
Caller annotates the type they expect. Same trust-the-caller
discipline as `:wat::edn::read`. **Without this, every pause result
would have to be unwrapped from a HolonAST envelope before
display; with it, the result IS whatever the expression produced.**

**Arc 103 — `:wat::kernel::spawn-program` + HOLOGRAM.md** — the
load-bearing arc. Three things landed:
- The EDN+newline pipe protocol got named as the universal
  cross-surface contract.
- The hologram framing made explicit what was already true: the
  wat binary is a one-way projection surface; programs see
  through but cannot reach back.
- The dispatcher demo (`echo '#demo/Job {...}' | wat dispatch.wat`)
  proved hologram-aware RPC works in operational form.

**Arc 104 (open) — wat-cli always forks the entry program** — when
this lands, **the cli is no longer a co-resident with user code**.
The user's program runs in its own COW-isolated address space.
This matters for `(:wat::pause::break)` specifically — when the
break fires inside the user's program, the program's own stdin/
stdout/stderr are the channel back to the user's terminal.
Containment is structural, not aspirational.

## What this means concretely

Pause is no longer a substrate addition. It's a **second shape of a
thing that already exists** — a wat program the cli runs. The
substrate just needs to ship the loop (`:wat::pause::serve`), the
break primitive (`:wat::pause::break-with-stdio`), and a few
introspection primitives (`ls`, `show`, `completions`). All of
these are tiny — and gated behind `--pause`, so no production
build accidentally hosts an interactive interpreter.

The pieces that would have been hard to build five arcs ago land
trivially today:

- **Read source from terminal** — `IOReader/read-line` already
  exists; rustyline wraps the parent's `Stdin` for line-editing/
  history.
- **Parse the line** — `:wat::edn::read` does it.
- **Evaluate** — `:wat::eval-edn!` does it (now polymorphic via
  arc 102).
- **Render result** — `:wat::edn::write` does it.
- **Loop** — tail-recursive wat function (arc 003 TCO).
- **Communicate result back to terminal** — `IOWriter/println`
  through the cli's stdio proxy.
- **Capture environment for break** — `Environment` is already a
  runtime value; `eval_form` already takes one as context; the
  CALL_STACK thread-local is already populated for arc 016's
  failure traces.
- **Frontend ↔ backend split** — the EDN+newline protocol is the
  same one every other transport uses (shell→wat, wat→wat
  in-thread, wat→wat cross-process, future wat↔external,
  future wat↔remote).

## The honest thing about freeze

The substrate's freeze invariant is what makes pause's interrogation
static. You're looking at exactly the program that's running. When
you `:continue`, you continue into exactly the program you
inspected. No drift between inspection and resumption.

Ruby's pry fights this — methods can be redefined out from under
inspection, constants reassigned mid-session, threads mutate locals
asynchronously. Wat's pause inherits the substrate's stability.

The user's framing: **rust being frozen is a blessing, not a
curse.** What gives up is "redefine a method live" (which the
hologram forbids structurally, not by convention). What you get
back is "the inspection is honest" — the program you're looking at
is the program you're going to run.

The trade is the right one. The book has been arguing for the
hologram model for chapters; pause is what falls out when you ask
"what does interrogation look like INSIDE the hologram?"

The answer: the substrate already had everything. Tonight named it.

## The gating constraint

Pause symbols are not free. They expose program internals
(Environment capture, source listing, frame stack walking) that
production binaries should not host. The user's discipline:

> "all pause forms are illegal if --pause isn't passed... we ship with
> pause but the user must enable it... if we see (:wat::pause::break)
> and we're not in pause mode - panic"

The substrate ships the pause battery as part of `wat-cli`'s bundled
extensions, but **the cli only registers it when `--pause` is set**.
Without the flag, the `:wat::pause::*` namespace doesn't exist;
freeze fails with `UnknownFunction(":wat::pause::break")` if any
pause form is in the source — at the earliest possible point. No
runtime-conditional latent activation; no production build that
silently hosts the interpreter; the gate is structural at freeze.

See `gating.md` for the full mechanism.

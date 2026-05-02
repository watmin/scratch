# Two modes — bare vs entry-loaded — one freeze pipeline

User's framing 2026-04-29:

> "if we ship a wat-pry... it ignores the :user::main and runs
> :wat::pry::main instead.. the user's entrypoint file is still
> sourced.. all of their forms are loaded but their main isn't
> evaluated... is that what you were getting at?..."

That's the shape. The cli always honors the entry's source (if
given); the only thing that varies is which top-level function
gets invoked at the end of freeze.

## Three invocations, one branch

```
wat <entry.wat>          → freeze entry → invoke :user::main
wat --pry <entry.wat>    → freeze entry → invoke :wat::pry::main
wat --pry                → freeze with no entry → invoke :wat::pry::main
```

Same freeze pipeline runs in all three cases. The pipeline:

1. Argv parse, decide pry mode (`--pry` flag) and entry path
   (if given).
2. Build battery list — pry battery prepended if `--pry` set.
3. Run `startup_from_source`:
   - Config setters from entry source fire (if entry given).
   - All baked battery sources load (`wat/std/*.wat`,
     `wat/std/pry.wat` if pry-mode).
   - Entry source's defines + struct decls + load chain resolve.
   - Type checker runs. Symbol table seals.
4. Look up the entry-point symbol — `:user::main` for normal,
   `:wat::pry::main` for pry-mode.
5. Invoke it with the program's stdin/stdout/stderr handles.

The only branch is at step 4. Everything before is the same code
path with different inputs.

## What "the entry's source loads" means concretely

For `wat --pry trade.wat`:

- `(:wat::config::set-capacity-mode! :error)` at trade.wat top-level
  → fires during config pass. Pry session inherits this config.
- `(:wat::config::set-dim-router! ...)` → fires. Pry session uses
  the user's chosen tier list.
- `(:wat::core::define (:trading::compute-decision ...) ...)` →
  registered in the symbol table. Available at the prompt.
- `(:wat::core::struct :trading::Candle ...)` → registered. Pry
  can construct Candles, accessor functions auto-generated.
- `(:wat::load-file! "./decision.wat")` → loaded. Recursive
  load chains all resolve. Every define visible at any depth is
  reachable from the prompt.
- `(:wat::core::define (:user::main ...) ...)` → registered like
  any other function. NOT invoked, but present in the symbol
  table — the user can call it from the prompt.

The user's program is fully present. Only the entry-point
selection differs.

## What "no entry" means concretely

For `wat --pry`:

- No source file argument; entry source is `None`.
- No config setters fire from entry; the substrate's defaults
  (arc 043) apply — capacity-mode `:error`, the default tier
  list `[256 4096 10000 100000]`, the default sigma functions.
- No user defines. The frozen world has only what the batteries
  contributed.
- `:wat::pry::main` is invoked.

Bare pry is a calculator-shaped session against the substrate's
shipped libraries. Useful for testing primitives, exploring
substrate behavior, building up forms before committing them to a
file.

## The user's `:user::main` is callable from the prompt

In entry-loaded mode, `:user::main` is just another symbol in the
frozen world. The user can invoke it manually:

```
wat-pry> (:user::main stdin stdout stderr)
```

This runs the entry program from inside the pry session, using
the same stdio handles pry is using. Output appears in the user's
terminal; stdin reads come from the user's terminal. When main
returns, the pry prompt reappears.

This is the pry "step into the entry point" workflow, falling out
for free because main is just a symbol — there's no special
"entry point" status at the substrate level. The cli's choice of
which symbol to invoke at startup is just a naming decision; from
inside the frozen world, all symbols are equal.

A common workflow:

```
$ wat --pry trade.wat
wat-pry> (:trading::types::Candle/new 50000.0 50500.0 49500.0 50250.0 1.5)
#trading.types.Candle {:open 50000.0 :high 50500.0 :low 49500.0 :close 50250.0 :volume 1.5}

wat-pry> (:trading::rsi candle)        ;; experiment with a function
0.6234

wat-pry> (:user::main stdin stdout stderr)   ;; run the actual program
[program runs to completion]

wat-pry> (:wat::pry::last-error)       ;; inspect what happened
:None

wat-pry> :exit
```

## The same source file runs both ways

`trade.wat` is a single file. It declares `:user::main` with the
standard three-stdio signature. Whether main is invoked is the
cli's choice:

- `wat trade.wat` runs the trader.
- `wat --pry trade.wat` opens a pry session against the trader's
  symbols.

No conditional in the source. No `#[cfg]`-style mode flags in
wat. The file declares its capabilities; the cli decides what to
do with them. Same way `cargo run` and `cargo test` use the same
crate.

## What about programs without `:user::main`?

A library `.wat` file that only declares functions and doesn't
declare a main can be loaded into a pry session via the entry
mechanism without issue:

```
;; lib.wat
(:wat::core::define (:my::add (a :i64) (b :i64) -> :i64)
  (:wat::core::i64::+ a b))
;; (no :user::main)
```

```
$ wat --pry lib.wat
wat-pry> (:my::add 2 3)
5
```

The cli looks up `:wat::pry::main` (not `:user::main`); it doesn't
care whether the entry declared `:user::main`. So pry-mode works
on library files. Useful for "open this file's symbols and let me
play with them."

For normal `wat lib.wat` (no `--pry`), the cli would fail to find
`:user::main` and exit with the usual `EXIT_MAIN_SIGNATURE` (3).
The same file behaves differently in the two modes; the
difference is honest.

## Why this composes cleanly

The cli's only role is orchestration. It doesn't know what `pry`
is structurally — it just knows "in pry mode, look up
`:wat::pry::main` instead of `:user::main`." The substrate's
freeze pipeline is unchanged. The sandboxing, the
symbol-resolution, the type-checking, the config preamble — all
identical.

This is why the gating story (see `gating.md`) is also clean:
*the only thing pry mode does at the cli level is load the pry
battery and call a different entry-point.* Everything else is a
consequence of those two choices, applied through unmodified
substrate machinery.

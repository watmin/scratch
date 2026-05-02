# Gating — `--pry` as the registration gate

User's constraint, verbatim:

> "i think a requirement.. all pry forms are illegal if --pry isn't
> passed... we ship with pry but the user must enable it... if we
> see (:wat::pry::break) and we're not in pry mode - panic"

This is load-bearing. Pry exposes program internals — Environment
capture, source listing, frame walking — that production binaries
should not host. Three things must all be true:

1. The pry battery ships with `wat-cli`. Users don't install
   anything separately.
2. Without `--pry`, the `:wat::pry::*` namespace doesn't exist at
   all. Freeze rejects any source that references it.
3. With `--pry`, the namespace is registered, the loop is loaded,
   and `(:wat::pry::break)` fires the interactive break.

## Implementation — the pry battery is conditionally registered

Arc 100's `wat_cli::run(&[Battery])` already gives us the
mechanism. The cli's argv parser checks for `--pry`:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let pry_mode = args.iter().any(|a| a == "--pry");

    let mut batteries: Vec<Battery> = vec![
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register,    wat_sqlite::wat_sources),
        (wat_lru::register,       wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (wat_telemetry_sqlite::register, wat_telemetry_sqlite::wat_sources),
    ];

    if pry_mode {
        batteries.push((wat_pry::register, wat_pry::wat_sources));
    }

    wat_cli::run_with_args(&args, &batteries)
}
```

`wat_pry` is a Rust crate that ships in the workspace alongside
`wat_telemetry` etc. Its `register` function adds the
`:wat::pry::*` Rust shims (`break-with-stdio`, `serve`,
`completions`, `ls`, `show`, `where`, `frames`, `last-error`).
Its `wat_sources()` returns `wat/std/pry.wat` (the loop
implementation, the command dispatcher, the `break` macro).

When `--pry` is absent, `wat_pry` is not in the battery list. The
freeze pipeline's resolve pass fails any AST node referencing
`:wat::pry::*` with `UnknownFunction(":wat::pry::break")`.

## What the user sees

**Normal run with no pry forms in source:**

```
$ wat trade.wat
[normal program execution]
```

**Normal run with pry forms in source, no flag:**

```
$ wat trade.wat
error: UnknownFunction(":wat::pry::break") at trade.wat:42:7
       (:wat::pry::break) is gated behind --pry; pass --pry to enable.
```

**Pry-mode run:**

```
$ wat --pry trade.wat
[freeze normally — pry battery loaded — :wat::pry::main runs the loop]
wat-pry>
```

**Pry-mode run with break in user code:**

```
$ wat --pry trade.wat
[freeze; :wat::pry::main runs; user types (:user::main stdin stdout stderr)]
wat-pry> (:user::main stdin stdout stderr)
[program runs; hits (:wat::pry::break) inside compute-decision]
wat-pry (broken @ trade.wat:42:7)>
```

## Why freeze-time rejection over runtime panic

User said "panic." Two interpretations:

**Path A — runtime panic.** All pry primitives registered always;
each impl checks a process-global flag and panics if not set.
Pros: programs with unreachable pry calls don't fail. Cons: a
program with reachable pry calls runs successfully until the
moment the break fires, then crashes — ugly developer experience;
production binaries technically hosting the panic-guarded
primitives.

**Path B — freeze-time rejection.** Pry primitives registered only
when `--pry` is set. Without the flag, the namespace doesn't
exist; freeze fails on any reference. Pros: earliest possible
failure point; production binaries genuinely don't host pry
internals; the cli's discipline is structural, not procedural.
Cons: even unreachable pry calls fail freeze.

**Path B is the wat-honest answer.** The substrate has consistently
chosen "fail at freeze, not at runtime" wherever the choice
exists — type-check vs runtime-type-error, capacity-mode :error
vs :silent, signed-load verification before invoke. Gating pry
the same way fits the discipline.

Path B's "even unreachable pry calls fail freeze" is a feature,
not a bug: it prevents shipping a binary with latent break points
that would activate if someone passed `--pry` to a deployed
container by mistake.

## What about programs that conditionally use pry?

You can't write code that says "if pry mode, drop a break;
otherwise, no-op." Because the pry symbol literally doesn't exist
outside pry mode.

This is fine. Pry is a developer-time tool, not a runtime feature.
If you want a debug-mode in your program, ship a
`(:user::config/debug?)` flag your own code consults. If you want
to run with pry sometimes and not other times, use the same
source file — `wat trade.wat` for normal, `wat --pry trade.wat`
for inspection — and don't put `(:wat::pry::break)` in code paths
that run in non-pry mode.

The discipline that emerges: pry forms appear in code paths you
only run with `--pry`. The most common shape is dropping a break
into a function for one debug session, then deleting it. The
substrate doesn't fight this; it just refuses to host a binary
that pretends to be pry-free while carrying pry calls.

## What about `wat --pry` (no entry, bare session)?

Same gate. The pry battery is loaded; the substrate's `:wat::pry::*`
symbols register; `:wat::pry::main` is invoked instead of any
user main. Bare sessions are just pry mode with no user source —
the gate is identical to entry-loaded mode.

## What about test code?

`wat::test!` macros run programs in a sandboxed in-process freeze.
If a test wants to exercise pry primitives, it has to register
the pry battery in its test harness explicitly:

```rust
#[test]
fn pry_break_round_trip() {
    let mut deps = wat::rust_deps::with_wat_rs_defaults();
    wat_pry::register(&mut deps);

    // ... test that exercises (:wat::pry::break) ...
}
```

The pry battery is just another battery; tests opt in the same
way the cli does. No special test mode.

## Summary

| Mode | Pry battery loaded? | `:wat::pry::*` symbols exist? |
|---|---|---|
| `wat <entry>` | No | No — freeze fails on any pry reference |
| `wat --pry <entry>` | Yes | Yes |
| `wat --pry` (no entry) | Yes | Yes; `:wat::pry::main` invoked |
| `wat::test!` (default) | No | No |
| `wat::test!` with explicit register | Yes | Yes |

The gate is at registration. The substrate's existing
"unknown-symbol-fails-at-freeze" discipline does the rest. No new
runtime check, no new error variant, no new diagnostic to maintain.
The cli decides which batteries to load; everything downstream
follows.

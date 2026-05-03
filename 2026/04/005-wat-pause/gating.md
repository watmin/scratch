# Gating — `--pause` as the registration gate

User's constraint, verbatim:

> "i think a requirement.. all pry forms are illegal if --pry isn't
> passed... we ship with pry but the user must enable it... if we
> see (:wat::pry::break) and we're not in pry mode - panic"

[Quote preserved as spoken on 2026-04-29; the arc was originally
named `wat-pry` and the flag was `--pry`. Renamed to `wat-pause` /
`--pause` on 2026-05-03 — see INDEX.yaml captured-beats. The rest
of this document uses the new naming.]

This is load-bearing. Pause exposes program internals — Environment
capture, source listing, frame walking — that production binaries
should not host. Three things must all be true:

1. The pause battery ships with `wat-cli`. Users don't install
   anything separately.
2. Without `--pause`, the `:wat::pause::*` namespace doesn't exist at
   all. Freeze rejects any source that references it.
3. With `--pause`, the namespace is registered, the loop is loaded,
   and `(:wat::pause::break)` fires the interactive break.

## Implementation — the pause battery is conditionally registered

Arc 100's `wat_cli::run(&[Battery])` already gives us the
mechanism. The cli's argv parser checks for `--pause`:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let pause_mode = args.iter().any(|a| a == "--pause");

    let mut batteries: Vec<Battery> = vec![
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register,    wat_sqlite::wat_sources),
        (wat_lru::register,       wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (wat_telemetry_sqlite::register, wat_telemetry_sqlite::wat_sources),
    ];

    if pause_mode {
        batteries.push((wat_pause::register, wat_pause::wat_sources));
    }

    wat_cli::run_with_args(&args, &batteries)
}
```

`wat_pause` is a Rust crate that ships in the workspace alongside
`wat_telemetry` etc. Its `register` function adds the
`:wat::pause::*` Rust shims (`break-with-stdio`, `serve`,
`completions`, `ls`, `show`, `where`, `frames`, `last-error`).
Its `wat_sources()` returns `wat/std/pause.wat` (the loop
implementation, the command dispatcher, the `break` macro).

When `--pause` is absent, `wat_pause` is not in the battery list. The
freeze pipeline's resolve pass fails any AST node referencing
`:wat::pause::*` with `UnknownFunction(":wat::pause::break")`.

## What the user sees

**Normal run with no pause forms in source:**

```
$ wat trade.wat
[normal program execution]
```

**Normal run with pause forms in source, no flag:**

```
$ wat trade.wat
error: UnknownFunction(":wat::pause::break") at trade.wat:42:7
       (:wat::pause::break) is gated behind --pause; pass --pause to enable.
```

**Pause-mode run:**

```
$ wat --pause trade.wat
[freeze normally — pause battery loaded — :wat::pause::main runs the loop]
wat-pause>
```

**Pause-mode run with break in user code:**

```
$ wat --pause trade.wat
[freeze; :wat::pause::main runs; user types (:user::main stdin stdout stderr)]
wat-pause> (:user::main stdin stdout stderr)
[program runs; hits (:wat::pause::break) inside compute-decision]
wat-pause (broken @ trade.wat:42:7)>
```

## Why freeze-time rejection over runtime panic

User said "panic." Two interpretations:

**Path A — runtime panic.** All pause primitives registered always;
each impl checks a process-global flag and panics if not set.
Pros: programs with unreachable pause calls don't fail. Cons: a
program with reachable pause calls runs successfully until the
moment the break fires, then crashes — ugly developer experience;
production binaries technically hosting the panic-guarded
primitives.

**Path B — freeze-time rejection.** Pause primitives registered only
when `--pause` is set. Without the flag, the namespace doesn't
exist; freeze fails on any reference. Pros: earliest possible
failure point; production binaries genuinely don't host pause
internals; the cli's discipline is structural, not procedural.
Cons: even unreachable pause calls fail freeze.

**Path B is the wat-honest answer.** The substrate has consistently
chosen "fail at freeze, not at runtime" wherever the choice
exists — type-check vs runtime-type-error, capacity-mode :error
vs :silent, signed-load verification before invoke. Gating pause
the same way fits the discipline.

Path B's "even unreachable pause calls fail freeze" is a feature,
not a bug: it prevents shipping a binary with latent break points
that would activate if someone passed `--pause` to a deployed
container by mistake.

## What about programs that conditionally use pause?

You can't write code that says "if pause mode, drop a break;
otherwise, no-op." Because the pause symbol literally doesn't exist
outside pause mode.

This is fine. Pause is a developer-time tool, not a runtime feature.
If you want a debug-mode in your program, ship a
`(:user::config/debug?)` flag your own code consults. If you want
to run with pause sometimes and not other times, use the same
source file — `wat trade.wat` for normal, `wat --pause trade.wat`
for inspection — and don't put `(:wat::pause::break)` in code paths
that run in non-pause mode.

The discipline that emerges: pause forms appear in code paths you
only run with `--pause`. The most common shape is dropping a break
into a function for one debug session, then deleting it. The
substrate doesn't fight this; it just refuses to host a binary
that pretends to be pause-free while carrying pause calls.

## What about `wat --pause` (no entry, bare session)?

Same gate. The pause battery is loaded; the substrate's `:wat::pause::*`
symbols register; `:wat::pause::main` is invoked instead of any
user main. Bare sessions are just pause mode with no user source —
the gate is identical to entry-loaded mode.

## What about test code?

`wat::test!` macros run programs in a sandboxed in-process freeze.
If a test wants to exercise pause primitives, it has to register
the pause battery in its test harness explicitly:

```rust
#[test]
fn pause_break_round_trip() {
    let mut deps = wat::rust_deps::with_wat_rs_defaults();
    wat_pause::register(&mut deps);

    // ... test that exercises (:wat::pause::break) ...
}
```

The pause battery is just another battery; tests opt in the same
way the cli does. No special test mode.

## Summary

| Mode | Pause battery loaded? | `:wat::pause::*` symbols exist? |
|---|---|---|
| `wat <entry>` | No | No — freeze fails on any pause reference |
| `wat --pause <entry>` | Yes | Yes |
| `wat --pause` (no entry) | Yes | Yes; `:wat::pause::main` invoked |
| `wat::test!` (default) | No | No |
| `wat::test!` with explicit register | Yes | Yes |

The gate is at registration. The substrate's existing
"unknown-symbol-fails-at-freeze" discipline does the rest. No new
runtime check, no new error variant, no new diagnostic to maintain.
The cli decides which batteries to load; everything downstream
follows.

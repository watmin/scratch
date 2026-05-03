# Packaging — `--pause` flag vs separate `wat-pause` binary

Two equivalent shapes the same substrate can ship. The choice
between them is packaging, not architecture.

## Shape A — `wat-cli --pause`

The bundled `wat` binary gains a `--pause` flag. Argv parser
notices the flag, prepends the pause battery to the battery list,
swaps the entry-point lookup from `:user::main` to
`:wat::pause::main`. Otherwise unchanged.

**Pros:**
- One binary to install, one binary to maintain.
- Discoverability: `wat --help` shows pause alongside other modes.
- Same `wat-cli` machinery (signal handling, fork containment,
  battery composition).

**Cons:**
- The cli grows a frontend concern (rustyline integration in the
  parent process). Substrate-shape is a kernel; rustyline is a
  user-facing tool. Crossing the boundary in the same crate is
  honest but not minimal.
- Anyone who wants a different terminal-frontend (different
  history file, different completion strategy, no rustyline at
  all) has to fork wat-cli or add flags.

## Shape B — `crates/wat-pause/` as its own binary

`wat-pause` is a separate workspace crate. Its `main()` looks like:

```rust
fn main() -> ExitCode {
    wat_cli::run_with_args_and_extra_batteries(
        &std::env::args().collect::<Vec<_>>(),
        &[
            (wat_pause::register, wat_pause::wat_sources),
        ],
        Some(wat_pause::main_entry_point),  // override entry-point lookup
    )
}
```

Same battery composition pattern arc 100 established. The
`wat-pause` binary is "wat-cli plus the pause battery plus the
rustyline frontend." Different binary, same substrate.

**Pros:**
- Clean crate boundaries. `wat-cli` stays minimal; rustyline
  lives in `wat-pause` only. Production deployments of `wat-cli`
  don't link readline.
- Custom-pause consumers compose the same way arc 100 documented:
  `wat_cli::run(&[Battery])` + a custom main loop. They build
  `my-app-pause` with their own batteries + readline + commands.
- Clear discoverability — different binary name signals
  different purpose.

**Cons:**
- Two binaries to install, two binaries to keep in sync.
- More moving parts at the build layer.

## The decision — both, eventually

**Slice 1-2 (initial):** Ship as Shape A — `wat-cli --pause`. The
cli already exists; adding a flag + battery + entry-point
override is small. The rustyline integration adds ~170 lines to
wat-cli; not enough to justify a separate crate yet.

**Slice 3 (when frontend grows):** Extract Shape B — split
`wat-pause` into its own crate. Move rustyline and the prompt
rendering out of wat-cli; lift them into `crates/wat-pause/`.
wat-cli reverts to its single-purpose shape. Both binaries ship
from the workspace.

The split happens when the frontend has enough features to
deserve its own crate — likely when slice 4 (stepping) and slice
5 (TCP attach) land, since each adds frontend complexity.

## Why both forms work

The substrate doesn't change between Shape A and Shape B. The pause
battery is the same; the gating mechanism is the same; the
entry-point swap is the same. Only the wrapper shifts.

This is exactly the property arc 100 was designed to enable:
**downstream consumers can build their own batteries-included
binaries** by composing `wat_cli::run(&[Battery])` + custom
batteries. `wat-pause` is an internal example of that pattern; we
ship it ourselves to demonstrate the move.

## What `wat-pause` would NOT do that `wat-cli --pause` can

If a user ships their own custom binary (e.g.,
`my-trading-pause` that links wat-pause plus their custom
batteries), they get pause-shape UX with their batteries' symbols
visible. This is the "downstream consumers build custom CLIs"
pattern from arc 100, applied to pause.

`wat-cli --pause` won't fit this shape — it's a single binary with
a fixed battery list. Custom binaries have to use the
`wat_cli::run(&[Battery])` API directly, optionally importing
the pause battery's `register` + `wat_sources`.

## What changes for the user (the downstream consumer)

Today (post-arc-100):

```rust
// my custom CLI
fn main() -> ExitCode {
    wat_cli::run(&[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (my_battery::register,    my_battery::wat_sources),
    ])
}
```

Post-pause (Shape B exists):

```rust
// my custom CLI with pause mode
fn main() -> ExitCode {
    let pause_mode = std::env::args().any(|a| a == "--pause");

    let mut batteries = vec![
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (my_battery::register,    my_battery::wat_sources),
    ];

    if pause_mode {
        batteries.push((wat_pause::register, wat_pause::wat_sources));
    }

    wat_cli::run(&batteries)
}
```

Same pattern. Pause is just another opt-in battery. The custom
binary inherits the pause surface for free when `--pause` is passed.

## The `wat-pause` binary's role in the workspace

For the canonical `wat-pause` binary specifically, the value-add
over "wat-cli --pause" is:

1. **Rustyline frontend** — line editing, history, completion
   integration. Substrate provides `:wat::pause::completions`;
   the frontend consumes it.
2. **History file** — `~/.wat_pause_history` persistent across
   sessions.
3. **Prompt rendering** — `wat-pause>` vs `wat-pause (broken @
   file:line:col)>` formatting. Color codes (eventually).
4. **TCP attach mode** (slice 5) — `wat-pause --attach
   tcp://host:port` opens a connection to a running wat program
   instead of forking one.

These are all frontend concerns. The substrate stays library-
shape; `wat-pause` is the rustyline-bearing wrapper.

## Composition lineage

This shape is structurally identical to:

- **`cargo` and `cargo-edit` / `cargo-watch` etc.** Cargo is the
  base; subcommands compose via the `cargo-FOO` binary
  convention.
- **`git` and `git-lfs` / `git-secrets` etc.** Same shape.
- **Linux's `ip` and `ip-link` / `ip-addr`.** Same.

Each is a base tool with extension binaries that share the base's
machinery. The wat workspace settling into this pattern is
honest — wat-cli is the kernel; wat-pause is the developer-tools
extension; future binaries (wat-test? wat-fmt? wat-doc?) follow
the same shape.

The substrate doesn't enforce this; the workspace conventions do.
The pattern is recognizable enough that downstream consumers
adopt it naturally.

## Summary

| Property | Shape A (--pause flag) | Shape B (wat-pause binary) |
|---|---|---|
| Substrate code | identical | identical |
| Pause battery code | identical | identical |
| Gating mechanism | identical | identical |
| Frontend (rustyline) | in wat-cli | in wat-pause |
| Production wat-cli links readline? | yes | no |
| Custom downstream binaries can build their own pause CLI? | only by using wat_cli::run directly | yes, naturally |
| Number of installed binaries | 1 | 2 |
| Discoverability | `--help` flag | binary name |

**Recommendation:** Ship Shape A first (slice 1-2). Migrate to
Shape B when the frontend grows enough features to deserve its
own crate (around slice 3-4). The substrate doesn't change either
way.

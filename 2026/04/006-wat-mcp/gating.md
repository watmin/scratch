# Gating — `--mcp` mirrors `--pry`

Same discipline as 005-wat-pry's gating: pry primitives are
illegal without `--pry`; mcp primitives are illegal without
`--mcp`. Pry battery and MCP battery are independently loadable.

## The mechanism

The wat-cli's argv parser checks for both flags:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let pry_mode = args.iter().any(|a| a == "--pry");
    let mcp_mode = args.iter().any(|a| a == "--mcp");

    let mut batteries: Vec<Battery> = vec![
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register,    wat_sqlite::wat_sources),
        (wat_json::register,      wat_json::wat_sources),  // always available — useful beyond MCP
        (wat_lru::register,       wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (wat_telemetry_sqlite::register, wat_telemetry_sqlite::wat_sources),
    ];

    if pry_mode {
        batteries.push((wat_pry::register, wat_pry::wat_sources));
    }
    if mcp_mode {
        batteries.push((wat_mcp::register, wat_mcp::wat_sources));
    }

    wat_cli::run_with_args(&args, &batteries, choose_entry_point(pry_mode, mcp_mode))
}

fn choose_entry_point(pry: bool, mcp: bool) -> EntryPointName {
    match (pry, mcp) {
        (false, false) => ":user::main",
        (false, true)  => ":wat::mcp::main",
        (true,  false) => ":wat::pry::main",
        (true,  true)  => ":wat::mcp::main",  // MCP wins; agent gets pry through it
    }
}
```

Without `--mcp`, the `:wat::mcp::*` namespace doesn't exist;
freeze fails on any reference. Same Path B (freeze-time
rejection) the pry gating chose.

## What `--pry --mcp` together does

A program invoked with both flags loads BOTH batteries. The MCP
battery's `:wat::mcp::main` is the cli's entry point (MCP
"wins" when both are set, because the agent on the other end is
the consumer of choice). But `:wat::pry::*` symbols are also
registered, so:

- The agent can call pry introspection through MCP:
  `wat-eval (:wat::pry::ls)`.
- The user's code can use `(:wat::pry::break)` and have it fire
  via the MCP notification protocol.
- The pry battery's `wat/std/pry.wat` is loaded, but its
  `:wat::pry::main` (the human-facing terminal loop) is not
  invoked.

So `--pry --mcp` means "agent-mode with pry primitives enabled."
This is the most common combination for live debugging — the
agent connects via MCP; the program has `(:wat::pry::break)`
forms in it; both batteries' surfaces are reachable.

`--mcp` alone is "agent-mode without break support." Useful when
the program doesn't have break forms; saves the (small) pry
battery overhead.

`--pry` alone is "human-mode; rustyline frontend." No MCP
plumbing; just the terminal.

`--pry --mcp` is the developer's everyday flag combination for
debugging. The cli prints both flag names in startup banner so
the user knows what mode they're in.

## Why two flags, not one combined

Considered: a single `--introspect` that enables both pry and
mcp.

Rejected because the use cases are distinct:

- A production program with `--mcp` and no `--pry` exposes its
  surface to an agent (e.g., a long-running query server) but
  refuses `(:wat::pry::break)` calls. Production-safe; agent-
  callable.
- A development session with `--pry` and no `--mcp` is a
  human-in-terminal flow with no MCP machinery loaded.
- `--pry --mcp` is the live-debug case.

Three modes, three flag combinations. One unified flag would
muddy the production-vs-development distinction.

## Production deployment safety

The user's framing for pry gating applies equally to MCP:

> "we ship with [it] but the user must enable it... if we see
> [an mcp form] and we're not in [mcp] mode - panic"

A binary that ships without `--mcp` loaded does NOT host an
agent-callable surface. The `:wat::mcp::main` symbol doesn't
exist; the JSON-RPC server doesn't run; no MCP tooling is
reachable. The substrate is safe to deploy without worrying
about an agent stumbling into the wrong production system.

To enable MCP for a production deployment, an operator passes
`--mcp` explicitly. This mirrors how Linux services typically
expose debug interfaces — opt-in, with operational signaling
(a flag, an environment variable, a config setting).

## The MCP battery's contract

`crates/wat-mcp/`:

- `register(builder: &mut RustDepsBuilder)` — installs the few
  Rust shims for break-as-notification machinery (the session
  registry, `:wat::pry::override-return`, `:wat::pry::eval-in-frame`).
- `wat_sources() -> &'static [WatSource]` — returns
  `wat/std/mcp.wat` containing `:wat::mcp::main`,
  `:wat::mcp::dispatch`, and the JSON-RPC framing helpers.

Same shape every shipped battery uses. Composes cleanly with
arc 100's `wat_cli::run(&[Battery])` API.

## Custom MCP binaries

Downstream consumers can build their own MCP-flavored CLIs by
composing the mcp battery with their own batteries:

```rust
fn main() -> ExitCode {
    let mcp_mode = std::env::args().any(|a| a == "--mcp");

    let mut batteries = vec![
        (my_battery::register,    my_battery::wat_sources),
        (wat_telemetry::register, wat_telemetry::wat_sources),
    ];

    if mcp_mode {
        batteries.push((wat_mcp::register, wat_mcp::wat_sources));
    }

    wat_cli::run(&batteries)
}
```

Now `my-cli --mcp my-app.wat` exposes the consumer's program +
their custom batteries' shims as an agent-callable surface.
Same pattern arc 100 documented; MCP is just another battery.

## What about test code?

Tests that exercise MCP primitives register the mcp battery
explicitly, same pattern as pry tests:

```rust
#[test]
fn mcp_eval_round_trip() {
    let mut deps = wat::rust_deps::with_wat_rs_defaults();
    wat_json::register(&mut deps);
    wat_mcp::register(&mut deps);
    // exercise an mcp-mode wat program in-process ...
}
```

The mcp battery requires the json battery as a dep (since the
mcp loop calls `:wat::json::read`/`:wat::json::write`); tests
register both.

## Combined gate matrix

| Flags | Mode | Entry point | Available namespaces |
|---|---|---|---|
| (none) | Normal | `:user::main` | `:wat::*` (default), batteries' shims |
| `--pry` | Human pry | `:wat::pry::main` | + `:wat::pry::*` |
| `--mcp` | Agent pry | `:wat::mcp::main` | + `:wat::mcp::*`, + `:wat::json::*` (always available) |
| `--pry --mcp` | Agent + break | `:wat::mcp::main` | + `:wat::pry::*`, + `:wat::mcp::*` |

Four combinations; clear semantics for each. Non-flagged builds
have neither pry nor mcp loaded — production-safe by default,
opt-in by flag.

## Summary

The gating story for 006 is: copy 005's mechanism, change the
namespace and flag name, ship as its own battery. No new
mechanism; no new doctrine; just another instance of the
"battery as gate" pattern arc 100 made possible.

The user's discipline holds at every layer:

> "we ship with [it] but the user must enable it"

Wat-rs ships with three new batteries (wat-pry, wat-mcp,
wat-json). Two of them gate behind explicit flags. One
(wat-json) is always available because it's general-purpose
infrastructure. Each opts in via the cli's `--pry` / `--mcp`
flags or via downstream consumers' own battery composition.

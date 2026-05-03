# wat-cli-options — argv parsing DSL + the :user::main argv contract + user: subcommand convention

User direction (2026-05-03):

> *"i think we need to ship something like... wat-cli-options.. and
> we update the :user::main func to accept argv who is of a mandatory
> spec...*
>
> *wat file.wat some list of whatever arguments after the file*
>
> *argv is /always/ :*
> *$0 = the wat binary*
> *$1 = the wat file*
> *$N = whatever whitespace deliminted string values appaer after
> the file"*

> *"think of ruby's thor here...*
>
> *wat my-file.wat --something some-value --another-thing another
> --a-bool-flag*
>
> *my-file.wat could implement an argparse or optparse or whatever
> the 'most correct name' is..."*

> *"and i think we can go one step further...*
>
> *we could make wat-cli proper extensible... we reserve the right
> to claim any subcommand but users can shim in their own with a
> convention prefix of user:<somename>*
>
> *wat user:cmd .... whatever they want ..."*

> *"because wat is static, users /must/ compile their own cli if
> they want their symbols found - that's the agreement.. they can
> export their own binary with wat forms bound in the binary.. but
> they can and should make their own wat to get stuff like repl to
> work for them..."*

---

## What this arc captures

Three intertwined parts of making wat-cli a production-grade
extensible CLI surface:

### 1. wat-cli-options crate

A declarative argv parser. The wat equivalent of Ruby Thor /
Python argparse / Rust clap. Lets a wat program declare its
options + flags + positional args once, then parses the
caller's argv against that declaration.

```scheme
(:wat::cli::options::define :MyAppArgs
  ;; named options
  (--something :string :default "default-value")
  (--another-thing :string :required true)
  (--verbose :bool :default false :short "-v")
  ;; positional args
  (file :string :required true))
```

### 2. The `:user::main` argv contract (substrate)

Every user-defined `:user::main` function MUST accept an `argv`
of `:Vec<String>`. Argv layout is mandatory:

```
$0 = the wat binary path (e.g., /usr/local/bin/wat)
$1 = the wat source file (e.g., my-file.wat)
$N = subsequent whitespace-delimited args
```

```scheme
(:wat::core::define
  (:user::main (argv :Vec<String>) -> :ExitCode)
  (:wat::core::let*
    ((parsed (:wat::cli::options::parse :MyAppArgs argv)))
    (:wat::core::match parsed
      ((:Ok args)  ...do-the-work-with args...)
      ((:Err err)  ...print-usage-and-exit...))))
```

This contract is a **substrate-level change** in wat-rs proper —
every program has the same entry-point shape. wat-cli passes the
full argv (its own argv0 + the file + everything after) to the
program's main.

### 3. The `user:` subcommand convention (wat-cli)

wat-cli is extensible via subcommands. **wat reserves bare
subcommand names** (`wat repl`, `wat fmt`, `wat help`, etc.).
**Users add their own subcommands with the `user:` prefix**:

```
wat repl                        # wat-reserved
wat fmt my-file.wat             # wat-reserved
wat help                        # wat-reserved

wat user:my-cmd                 # USER-defined subcommand
wat user:my-cmd --some-flag value
wat user:deploy --env prod
wat user:lint-strict src/
```

The `user:` prefix prevents collision with future wat-reserved
names. If wat ever adds a `wat deploy` subcommand, the user's
`wat user:deploy` keeps working unchanged.

## The user agreement (per user direction)

> *"because wat is static, users /must/ compile their own cli if
> they want their symbols found - that's the agreement"*

The wat-cli architecture (per arcs 099/100/101 of wat-rs proper)
is built on the static-compile-per-user model. Each user produces
their own `my-wat-cli` binary with their batteries baked in:

- **wat-rs default batteries**: telemetry, sqlite, lru, etc.
- **wat-tooling batteries**: wat-repl, wat-help, wat-fmt
- **User batteries**: user's own crates with their `:user::*`
  symbols + their `:user:` subcommands

When the user runs `my-wat-cli user:deploy --env prod my-file.wat`,
the dispatcher:
1. Recognizes `user:deploy` as a user-extended subcommand
2. Looks up the user's registered handler
3. Invokes it with the full argv

The user's handler can use `wat-cli-options` to parse the argv,
or handle it manually — their choice.

## Layering

```
LAYER N+1 — User's compiled my-wat-cli
              (wat-cli + wat-rs core + tooling + user batteries)
                ↓ runs
LAYER N   — wat-cli (per arcs 099/100/101 of wat-rs proper)
              (subcommand dispatcher; reserves bare names;
               recognizes user: prefix)
                ↓ invokes
LAYER N-1 — :user::main (argv :Vec<String>) -> :ExitCode
              (the user's program; argv contract enforced)
                ↓ uses
LAYER N-2 — wat-cli-options (THIS ARC)
              (argv parsing DSL; declarative; typed result)
                ↓ uses
LAYER N-3 — wat substrate (string parsing; type checking;
              error model)
```

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-cli-options/`
per the arc-013 pattern. Substrate-tier alongside wat-kwargs and
wat-schema.

```
wat-rs/crates/wat-cli-options/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-schema (../wat-schema; for
                       #   typed-arg refinement)
  src/                 # Rust shim (argv tokenization; typed
                       #   conversion; error formatting)
  wat/cli/options/     # The DSL: define, parse, validate,
                       #   help-text generation
  wat-tests/           # wat-level tests
  tests/               # Rust harness + integration tests
```

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture; conventions; status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: the DSL; argv contract; subcommand convention; usage-text generation; error model; comparison to Thor / argparse / clap. |
| `SLICE-PLAN.md` | Slices for shipping. (Sized once arc 109's substrate work firms up.) |

## Conventions inherited

- Four questions as design compass
- Arc-013 self-contained crate pattern
- Failure engineering: parse failures are typed
  `:OptionsParseError`; usage text always available; never panics
- Type contract: parsed args are typed structs (per wat-schema
  refinement; arc 013)
- Dependency doctrine: depends on wat-schema for typed args; no
  new external Rust deps

## Cross-references

- **arc 003 (wat-fmt)** — usage-text formatting could use wat-fmt
  for consistency
- **arc 005 (wat-pause)** — `--pause` flag is one of wat's
  reserved CLI options; wat-cli-options supplies the parsing
- **arc 006 (wat-mcp)** — `--mcp` flag similarly
- **arc 012 (wat-repl)** — wat-cli ships wat-repl as default
  battery; `wat repl` is a reserved subcommand
- **arc 013 (wat-schema)** — typed argument refinement
- **arc 018 (wat-help)** — `wat help <subcommand>` could expose
  per-subcommand help-text from wat-cli-options' declarations
- **wat-rs arc 099/100/101** — wat-cli architecture; this arc's
  `:user::main` contract + `user:` subcommand convention are
  substrate updates to wat-cli proper
- **arc 109 (wat-rs mass refactor)** — supplies the substrate
  changes (argv contract; subcommand dispatcher updates) this
  arc depends on

## Status

- **Captured:** 2026-05-03
- **Naming:** `wat-cli-options` (gaze-approved per user proposal;
  "options" is the universal noun for "command-line arguments
  and flags"; matches Python argparse/Ruby Thor/Rust clap
  vocabulary; doesn't conflict with `wat-cli` proper)
- **Architecture:** sketched
- **Slice plan:** depends on arc 109 substrate updates; sized
  conservatively
- **Bar to graduate to a real wat-rs arc:**
  1. Arc 109's substrate work has shipped the `:user::main`
     argv contract change (every program receives `:Vec<String>`
     argv)
  2. Arc 109's substrate work has shipped the wat-cli subcommand
     dispatcher updates (recognizes `user:` prefix; routes to
     user-registered handlers)
  3. arc 013 (wat-schema) has shipped slice 1 (typed-arg
     refinement available)
  4. User signals "let's start"

## What this arc is NOT

- **Not the wat-cli architecture itself** — that's wat-rs arcs
  099/100/101 (already in flight; substrate-level work)
- **Not the `:user::main` contract change** — that's a substrate
  update flagged here as a prerequisite; lives in wat-rs proper
- **Not the subcommand dispatcher** — that's a wat-cli internal
  change; flagged here as a prerequisite
- **Not autocomplete for shells** — sibling crate if needed
  (`wat-cli-completions`?)
- **Not config-file loading** — separate concern; argv is the
  surface this crate parses
- **Not sub-sub-commands** — single-level subcommands only at v1;
  nested commands deferred until real demand

## The four-questions on the trio

The whole package (wat-cli-options + argv contract + user:
convention) earns these checkmarks:

- **Obvious** ✅✅ — universal pattern (Thor/argparse/clap);
  reserved-vs-user namespace prevents collision
- **Simple** ✅ — one DSL; one entry-point contract; one
  convention; each piece does one thing
- **Honest** ✅✅ — argv contract is structural (every program
  has same signature); user: prefix makes extension visible;
  parser produces typed errors not silent defaults
- **Good UX** ✅✅ — declarative DSL reads as documentation;
  `wat user:my-cmd` is unambiguous; usage text auto-generated

Strong shape. Could be ✅✅✅ Honest if every parsed argv
structurally matches the declaration AND the substrate enforces
the `:user::main` argv contract at compile time (so a program
without the right entry-point signature fails freeze, not
runtime). Both achievable; worth confirming in slice planning.

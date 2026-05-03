# wat-cli-options — DESIGN

Substrate-tier crate. Three intertwined parts: the DSL crate
itself, the `:user::main` argv contract, the `user:` subcommand
convention.

---

## The four questions are the design compass

- **Obvious?** Declarative DSL reads as documentation;
  `wat user:my-cmd` is unambiguous; argv layout is fixed and
  visible
- **Simple?** One DSL form; one parse call; one entry-point
  contract; one subcommand-prefix convention
- **Honest?** Argv contract is structural (every program same
  signature); user: prefix makes extension visible at the
  command line; parser produces typed `:Result` not silent
  defaults
- **Good UX?** Familiar shape (Thor/argparse/clap); usage
  text auto-generated; consistent across every wat-built CLI

## Part 1 — The DSL (the wat-cli-options crate)

### Declaration form

```scheme
(:wat::cli::options::define :MyAppArgs
  ;; named options (--foo or -f short form)
  (--something :string :default "default-value"
                       :doc "Set the something parameter")
  (--another-thing :string :required true
                           :doc "Required: the another thing")
  (--count :i64 :default 1 :short "-c"
                :doc "Number of times to do it"
                :validator (:i64 :range 1 100))
  (--verbose :bool :default false :short "-v"
                   :doc "Enable verbose output")
  (--config :path :default "~/.config/my-app.edn"
                  :doc "Path to config file")

  ;; positional args (consumed in order after options)
  (file :string :required true
                :doc "The file to process")
  (output-dir :path :default "."
                    :doc "Where to write output"))
```

Each option/positional carries:
- **Type** — `:string`, `:i64`, `:bool`, `:path`, etc.
- **Default** — `:default <value>`; absent ⇒ required (or
  explicit `:required true`)
- **Short form** — `:short "-X"` for single-char alias
- **Doc** — `:doc "..."` shown in usage text
- **Validator** — wat-schema refined-type constraint
  (per arc 013); rejected at parse time if violated

### Parse form

```scheme
(:wat::cli::options::parse :MyAppArgs argv)
;; => :Result<:MyAppArgs, :OptionsParseError>
```

Returns a typed struct (the `:MyAppArgs` shape) on success;
typed error on failure. The error includes:
- What went wrong (missing required; type mismatch; validator
  failed; unknown option)
- Where (which arg position; which option name)
- Suggested usage text (auto-generated from the declaration)

### Usage-text generation

```scheme
(:wat::cli::options::usage-text :MyAppArgs)
;; => :String
```

Generates the standard "USAGE: ..." text from the declaration.
Used internally on parse failure; can be invoked manually for
`--help` flag handling.

```
USAGE: my-app [OPTIONS] FILE [OUTPUT-DIR]

ARGUMENTS:
  FILE         The file to process
  OUTPUT-DIR   Where to write output (default: .)

OPTIONS:
  --something STRING       Set the something parameter
                           (default: default-value)
  --another-thing STRING   Required: the another thing
  --count, -c I64          Number of times to do it
                           (default: 1; range: 1..100)
  --verbose, -v            Enable verbose output
  --config PATH            Path to config file
                           (default: ~/.config/my-app.edn)
```

## Part 2 — The :user::main argv contract (substrate)

### The contract

```scheme
(:wat::core::define
  (:user::main (argv :Vec<String>) -> :ExitCode)
  ...body...)
```

**Mandatory signature.** Every user-defined `:user::main` MUST
accept `argv :Vec<String>` and return `:ExitCode`. Argv layout
is mandatory:

| Position | Contents |
|---|---|
| `argv[0]` | Path to the wat binary (e.g., `/usr/local/bin/wat` or `/usr/local/bin/my-wat-cli`) |
| `argv[1]` | Path to the wat source file (when invoked as `wat my-file.wat ...`) |
| `argv[2..N]` | Subsequent whitespace-delimited args |

For subcommand invocations (`wat user:my-cmd ...`), the layout
shifts:

| Position | Contents |
|---|---|
| `argv[0]` | Wat binary path |
| `argv[1]` | The subcommand (e.g., `user:my-cmd` or `repl`) |
| `argv[2..N]` | Subsequent args |

The substrate enforces this contract:
- Programs without the right `:user::main` signature fail
  freeze (compile-time error)
- The wat-cli passes the full argv (its argv0 + everything) to
  the program's main
- No silent argv reshaping; what the binary received is what
  the program sees

### Why this matters

Today's wat programs may have ad-hoc `:user::main` signatures
(per the substrate's pre-contract state). The contract makes
every wat program callable the same way; cli tools can pass
argv generically; users build muscle memory around one shape.

This is a SUBSTRATE CHANGE in wat-rs proper. Captured here as
a prerequisite; actual implementation lives in arc 109's
mass-refactor work or post-109.

### `:ExitCode` type

```scheme
(:wat::core::enum :wat::core::ExitCode
  ((Success))
  ((Failure (code :i64))))

;; Accessor: integer code (0 for Success; 1+ for Failure)
(:wat::core::ExitCode/to-i64 ec) -> :i64
```

Standard exit-code semantics. The wat-cli wrapper translates
to OS process exit code on shutdown.

## Part 3 — The user: subcommand convention (wat-cli)

### The convention

wat-cli is extensible via subcommands. wat reserves bare
subcommand names; users prefix theirs with `user:`:

| Subcommand | Reserved by |
|---|---|
| `wat repl` | wat (arc 012) |
| `wat fmt` | wat (arc 003) |
| `wat lint` | wat (arc 004) |
| `wat help` | wat (arc 018) |
| `wat eval` | wat (substrate) |
| `wat <future>` | wat (reserved namespace) |
| `wat user:my-cmd` | USER (arbitrary) |
| `wat user:deploy` | USER |
| `wat user:lint-strict` | USER |

The `user:` prefix prevents collision. If wat ever adds
`wat deploy`, the user's `wat user:deploy` keeps working — the
namespaces never overlap.

### Dispatch

```
wat user:my-cmd --foo bar baz
       │
       ▼
wat-cli subcommand dispatcher recognizes "user:" prefix
       │
       ▼
Looks up registered handler for "user:my-cmd" in the user's
batteries
       │
       ▼
Invokes (:user::my-cmd (argv :Vec<String>) -> :ExitCode)
with the full argv
       │
       ▼
Handler can call (:wat::cli::options::parse :MyCmdArgs argv)
to parse its options
```

### Registration

In the user's `my-wat-cli` Cargo.toml-equivalent build:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let batteries: Vec<Battery> = vec![
        // wat-rs default batteries
        ...
        // tooling
        (wat_repl::register, wat_repl::wat_sources),
        (wat_help::register, wat_help::wat_sources),
        (wat_cli_options::register, wat_cli_options::wat_sources),
        // user batteries (registers user: subcommands)
        (my_app::register, my_app::wat_sources),
    ];
    wat_cli::run_with_args(&args, &batteries)
}
```

Within the user's battery (wat-side):

```scheme
;; my_app's wat sources
(:wat::cli::register-user-subcommand
  :name "user:deploy"
  :handler :user::deploy::main
  :doc "Deploy the application to the configured environment.")

(:wat::core::define
  (:user::deploy::main (argv :Vec<String>) -> :ExitCode)
  ...)
```

The wat-cli dispatcher discovers user: subcommands at freeze
time via the registered handlers.

## Per-symbol-kind handling — usage-text composition

The usage text auto-generates from the declaration:

```
USAGE: my-app [OPTIONS] FILE [OUTPUT-DIR]
```

Width-aware formatting: long option names wrap; descriptions
column-align. Multi-line `:doc` strings render as wrapped
paragraphs.

For wat-cli's own subcommand-aware usage:

```
USAGE: my-wat-cli <SUBCOMMAND> [args]

WAT SUBCOMMANDS:
  repl    Open an interactive REPL bound to this binary's
          frozen world.
  fmt     Format wat source files.
  help    Show help for a subcommand or symbol.
  eval    Evaluate a single wat form.

USER SUBCOMMANDS:
  user:deploy   Deploy the application.
  user:lint-strict   Run extra-strict linting on user batteries.
```

## Comparison to Thor / argparse / clap

| | Ruby Thor | Python argparse | Rust clap | wat-cli-options |
|---|---|---|---|---|
| Declarative | Yes (DSL methods) | Yes (add_argument) | Yes (derive macros) | Yes (`:options::define`) |
| Subcommands | Yes | Yes (subparsers) | Yes | Yes (via wat-cli dispatcher; user: prefix) |
| Auto help | Yes | Yes | Yes | Yes (`:options::usage-text`) |
| Type coercion | Limited | Yes | Yes (rich) | Yes (via wat-schema refinement) |
| Subcommand registration | Class-based | Function-based | Struct-based | Battery-based |
| Static binary distribution | Limited | N/A (interpreter) | Native | Native (via wat-cli compile) |

wat-cli-options sits at the convergence: declarative like Thor;
typed like clap; extensible per-binary like Rust crate-built CLIs.

## Per the four questions

- **Obvious?** ✅✅ — universal Thor/argparse/clap shape; `user:`
  prefix is unambiguous; argv contract is fixed and visible
- **Simple?** ✅ — one DSL; one parse call; one entry-point
  contract; one subcommand prefix; each piece does one thing
- **Honest?** ✅✅ — typed errors; substrate-enforced argv
  contract; user: prefix forces extension visibility; usage
  text auto-generated from same declaration
- **Good UX?** ✅✅ — familiar across language ecosystems;
  consistent across every wat-built CLI; user agreement
  (compile-your-own-cli) makes the model explicit

Strong shape. **Could be ✅✅✅ Honest** if substrate enforces
the `:user::main` argv signature at compile time (program with
wrong signature fails freeze, not runtime). Achievable; flagged
as substrate prereq.

## Cross-references

- **arc 003 (wat-fmt)** — usage-text rendering can use wat-fmt
- **arc 005 (wat-pause)** — `--pause` is a wat-reserved option
- **arc 006 (wat-mcp)** — `--mcp` similarly
- **arc 012 (wat-repl)** — `wat repl` is a wat-reserved
  subcommand
- **arc 013 (wat-schema)** — typed-arg refinement; validator
  primitives
- **arc 018 (wat-help)** — `wat help <subcommand>` exposes
  per-subcommand usage from wat-cli-options
- **wat-rs arc 099/100/101** — wat-cli architecture; this
  arc's substrate prereqs (argv contract + dispatcher) update
  wat-cli proper
- **arc 109 (wat-rs mass refactor)** — the substrate work that
  unlocks this arc's prerequisites
- **DEPENDENCY-DOCTRINE.md** — wat-schema as a chosen dep;
  no new external deps

## Open architectural questions

A. **Sub-sub-commands.** `wat user:deploy aws --region us-east-1`?
   Single-level subcommands only at v1; nested deferred.
   Lean: keep flat; if real demand surfaces, extend later.

B. **Config-file integration.** Should `wat-cli-options` natively
   support layered config (env vars → config file → argv)? Lean:
   NO at v1; argv-only is the surface. Layering belongs in
   sibling crate (`wat-cli-config`?) if real demand surfaces.

C. **Shell completions.** Bash/zsh/fish completion scripts
   generated from declarations? Sibling crate
   (`wat-cli-completions`?) post-v1.

D. **Mutually exclusive options.** `--quiet` + `--verbose`
   should be exclusive. Lean: support `:exclusive-with [other]`
   modifier in slice 2.

E. **Required-with / required-unless.** Conditional required
   semantics. Lean: support in slice 2 via `:required-when
   <expr>` modifier.

F. **Positional vs option ambiguity.** What if a positional
   value starts with `--`? Lean: standard `--` separator
   convention (`wat my-file -- --not-an-option`).

G. **--version / --help auto-handling.** Standard flags every
   CLI has. Lean: auto-add at parse time unless explicitly
   declared by the user.

## What's NOT in scope

- **Implementing the substrate prereqs** — `:user::main` argv
  contract change + `user:` subcommand dispatcher live in
  wat-rs proper; this arc flags them as required
- **Sub-sub-commands** — flat namespace for v1
- **Config-file loading** — sibling crate if needed
- **Shell completions** — sibling crate if needed
- **TUI / interactive prompting** — beyond CLI-options scope;
  separate concern
- **Argument parsing for non-Latin scripts** — Unicode-aware
  string handling is the substrate's job; this crate just
  consumes :String values

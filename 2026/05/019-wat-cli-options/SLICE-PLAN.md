# wat-cli-options — SLICE-PLAN

Sketch only. Not sized for shipping. Bar to graduate:

1. Arc 109's substrate work has shipped the `:user::main` argv
   contract (every program receives `:Vec<String>` argv at the
   mandatory layout; signature enforced at freeze)
2. Arc 109's substrate work has shipped the wat-cli subcommand
   dispatcher updates (recognizes `user:` prefix; routes to
   user-registered handlers)
3. arc 013 (wat-schema) has shipped slice 1 (typed-arg
   refinement available)
4. User signals "let's start"

---

## Slice 0 — Substrate prerequisites (wat-rs proper)

**This is NOT a wat-cli-options slice; it's the prereq tracker.**

Captured here so the dependency is visible:

- `:user::main` argv contract enforcement (substrate change)
- `user:` subcommand dispatcher in wat-cli (per-arc-099/100/101
  architecture)
- Battery-side registration for `user:` subcommands (e.g.,
  `:wat::cli::register-user-subcommand`)
- `:wat::core::ExitCode` enum (if not already present)

These are wat-rs proper concerns. The wat-cli-options crate
slices below assume these have shipped.

---

## Slice 1 — DSL define + parse (basic)

**Goal:** users can declare options and parse argv against the
declaration; basic types work end-to-end.

**Done when:**
- `wat-rs/crates/wat-cli-options/` exists with arc-013 layout
- `(:wat::cli::options::define :Name ...)` macro works
- Option types: `:string`, `:i64`, `:bool`, `:path`
- Positional args parsed in declaration order after options
- `(:wat::cli::options::parse :Name argv)` returns
  `:Result<:Name, :OptionsParseError>`
- `:OptionsParseError` taxonomy: `:MissingRequired`,
  `:TypeMismatch`, `:UnknownOption`, `:DuplicateOption`,
  `:InvalidValue`
- wat-tests cover: positive parse; missing required; type
  mismatch; unknown option; default values applied

**Out of scope:**
- Usage-text generation (slice 2)
- Validators via wat-schema (slice 3)
- Short-form flags (slice 2)
- `--version` / `--help` auto-handling (slice 4)

---

## Slice 2 — Usage-text + short forms

**Goal:** auto-generated usage text; short-form aliases.

**Done when:**
- `(:wat::cli::options::usage-text :Name)` generates standard
  USAGE block from declaration
- Width-aware formatting; description column-align
- Multi-line `:doc` strings render as wrapped paragraphs
- `:short "-X"` modifier supported; short forms parse
- `--option=value` form supported (alongside `--option value`)
- `--` separator convention (everything after is positional)
- wat-tests cover: usage text matches expected layout; short
  forms work; -- separator handles edge cases

---

## Slice 3 — wat-schema validator integration

**Goal:** options can carry wat-schema refined-type validators;
parse-time validation rejects invalid values.

**Done when:**
- `:validator (:i64 :range 1 100)` works
- `:validator (:string :min-length 1 :max-length 64)` works
- `:validator (:path :exists? true)` works (if path-existence
  refinement available; otherwise defer)
- Validator failures surface in `:OptionsParseError` with
  clear message
- Usage text includes constraint info
  (e.g., "range: 1..100")
- wat-tests cover each validator type; positive + negative

---

## Slice 4 — Auto-handle --help and --version

**Goal:** every options-enabled program supports `--help` and
`--version` without explicit declaration.

**Done when:**
- `--help` / `-h` flag auto-added unless explicitly declared
- `--version` / `-V` flag auto-added unless explicitly declared
- `--help` prints usage text + exits 0
- `--version` prints declared version (from declaration metadata)
  + exits 0
- Both work as the FIRST argument or anywhere in argv
- wat-tests cover both flags; explicit-declaration override

---

## Slice 5 — Subcommand registration + dispatch integration

**Goal:** user batteries register `user:` subcommands; wat-cli
dispatcher routes to them.

**Done when:**
- `(:wat::cli::register-user-subcommand :name "user:my-cmd"
   :handler :user::my-cmd::main :doc "...")` works in user wat
   batteries
- wat-cli dispatcher discovers registered subcommands at freeze
- `wat user:my-cmd ...args...` invokes the registered handler
  with full argv
- `wat help` (per arc 018) shows registered user: subcommands
- Round-trip test: build a sample `my-wat-cli` with a user:
  subcommand; verify dispatch end-to-end

**Depends on:** Slice 0 substrate work shipped.

---

## Slice 6 — Production hardening

**Goal:** wat-cli-options is genuinely usable for production
wat applications.

**Done when:**
- Performance: parse <1ms for typical declarations (50 options;
  10 positional)
- Memory: bounded; no leak on long-running CLI processes
- Documentation: complete reference; cookbook patterns
  (REST CLIs; data pipelines; deployment tools)
- One concrete deployed user-built CLI uses wat-cli-options +
  user: subcommands end-to-end
- Integration with wat-help: `wat help user:my-cmd` shows the
  subcommand's usage text via wat-cli-options

---

## Slices NOT planned

- **Mutually exclusive options** — useful; defer to slice 2.5
  if real demand surfaces
- **Conditional required (required-when)** — same; defer
- **Sub-sub-commands** — flat namespace for v1
- **Config-file loading** — sibling crate if needed
  (`wat-cli-config`?)
- **Shell completions** — sibling crate if needed
  (`wat-cli-completions`?)
- **TUI / interactive prompting** — beyond CLI-options scope
- **Argument parsing for env vars** — sibling crate if needed
  (env vars are a different concern from argv)

---

## Honest accounting

This slice plan is **sketched, not sized**. Slice 0 is
substrate prerequisites that this arc cannot ship without. Until
arc 109 makes those changes, slice 1 cannot begin.

The biggest design unknown: how does the wat-cli dispatcher
discover user-registered subcommands at freeze time? Two options:
(a) wat-side registration via a substrate-provided form
    (`:wat::cli::register-user-subcommand`)
(b) Rust-side battery API (each battery exposes its
    subcommand list)

Lean: (a) for consistency — registration belongs in wat code,
discovered at freeze, surfaced via reflection (which wat-help can
also use).

The four-questions discipline applies to each slice independently.
Each slice should answer all four with honest checkmarks before
declaring the slice done.

The load-bearing properties:
- Argv contract structural enforcement (slice 0; ✅✅✅ Honest)
- Auto-generated usage text matches declaration (slice 2; ✅✅
  Honest)
- user: prefix prevents collision (slice 5; ✅✅ Honest)
- Type-checked options via wat-schema (slice 3; ✅✅ Honest)

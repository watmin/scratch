# wat-fmt — the canonical wat formatter

The user, 2026-05-02, picking from random-notes.txt's four-item
unburdening list (linter / formatter / test coverage /
RemoteProgram<I,O>):

> "the thing i'm most keen on is the formatter... i have very
> specific taste but i want to get the idea how to implement a
> formatter.. our code is edn.. so we can do work on it... i'll
> correct whatever style guideslines we encounter... you start
> opinioned.. i'll course correct.... wat-fmt would be fitting?..
> some create that we can ship that glues into wat-cli? `wat
> --format some-file.wat` ya?"

This scratch arc captures the design pre-implementation. The
formatter is foundation-level infra alongside the wards —
quality gate that ships with wat-rs proper.

---

## What wat-fmt is

A canonical formatter for wat code. Reads source; emits
canonically-formatted source. One opinionated style; no
configuration.

The wat-fmt advantage over most formatters: **wat code is
HolonAST**. The AST is the source of truth; formatting is a
total function over it. No regex, no whitespace heuristics, no
operator-precedence guesswork. Walk the tree; emit text per
per-node rules. The hard problem is comment preservation;
everything else falls out.

## Architecture in one paragraph

**wat-fmt is wat code, not Rust** (decision flipped 2026-05-02
per RuboCop-style language alignment — see DESIGN.md). The
pipeline: source bytes → wat-rs tokenizer (with comment-token
preservation) → wat-rs parser (attaches comments to adjacent
AST nodes) → AST → wat-coded formatter (the wat-vm walks the
tree, applies per-node format rules in wat, emits text).
Round-trip stable (`format(format(x)) == format(x)`);
semantically preserving (`parse(format(x)) == parse(x)`).

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-fmt/`.

Both the Rust shim AND the wat code live inside the crate.
The wat code is embedded into the Rust binary via `include_str!`
(per arc 013's two-part contract); the crate is one shippable
Cargo unit with no runtime filesystem dependency.

```
wat-rs/crates/wat-fmt/
  Cargo.toml           # depends on wat (path = "../..") + wat-macros
  src/                 # Rust shim (wat_sources + register + format API)
  wat/fmt/             # the actual formatter (per-rule wat files)
  wat-tests/fmt/       # wat-level tests
  tests/               # Rust harness + golden files + property tests
```

The pattern mirrors the existing wat-shipping crates (wat-lru,
wat-holon-lru, wat-sqlite, wat-telemetry, wat-telemetry-sqlite)
exactly — same arc-013 two-part contract, same crate-name-prefix
in the wat path, same wat-tests/ split.

wat-cli depends on wat-fmt; users who want the CLI get the
formatter for free as a transitive dep. wat-lint (when it
ships) also depends on wat-fmt — gets both the Rust API and
the wat-coded `:wat::fmt::*` primitives via the embedded wat
code, registered into the wat-vm alongside its own lint rules.

## CLI surface

```
wat fmt path/to/file.wat              # format in place
wat fmt --check path/to/file.wat      # exit 0 if formatted, 1 if not, 2 if parse error
wat fmt --diff path/to/file.wat       # show diff vs current
wat fmt --stdin                       # read stdin, write stdout
wat fmt path/to/dir/                  # recurse
```

Subcommand pattern (not `wat --format`) per user direction; opens
the path for `wat check`, `wat lint`, etc. as siblings.

## Reading order

| File | What it contains |
|---|---|
| `README.md` | This file. Top-level orientation. |
| `INDEX.yaml` | Beat-by-beat capture, conventions, status. |
| `DESIGN.md` | Architecture, crate layout, CLI integration, public API, error model, configuration philosophy. |
| `STYLE-RULES.md` | The numbered style rules. Confirmed (1-12) + draft (special forms, FQDN handling, type annotations). **User will mark up directly.** |
| `SLICE-PLAN.md` | Five slices from "core formatter no comments" through "linter foundation." |

## Style rules — at a glance

12 rules confirmed by the user's first pass; ~10 draft rules
covering special forms (let, define, lambda, cond, if, match)
that the user will iterate on ("i am /very/ opinionated...
we'll work on special forms later").

Quick highlights of confirmed rules:
- Two-space indent
- Closing parens stack (Lisp convention)
- 120-column line length (will probably go longer "with reasons")
- One blank line between top-level forms
- Comments preserved verbatim
- No trailing whitespace; trailing newline at EOF
- FQDN names never wrap

See STYLE-RULES.md for the full numbered list.

## Status

- **Captured:** 2026-05-02
- **Architecture:** locked (parse → AST → emit; comment-preserving
  parser; Rust crate at `wat-rs/crates/wat-fmt/`; subcommand CLI)
- **Confirmed style:** 12 rules around indentation, whitespace,
  comments, top-level form spacing
- **Draft style:** special forms, FQDN handling, type annotations
  — to refine with user
- **Slice plan:** 5 slices sized; not opened
- **Bar to open as a real wat-rs arc:** user's mark-up of
  STYLE-RULES.md + signal that foundations are at a good stopping
  point + "let's start"

## Related work

- **wat-lint** (next from random-notes.txt) — uses wat-fmt's
  parser as shared infrastructure; lint rules operate on the AST
  too. Slice 5 of the wat-fmt plan establishes the foundation.
- **The wards** (`/sever`, `/reap`, `/scry`, `/gaze`, `/forge`,
  `/temper`, `/assay`, `/ignorant`) — same tier of foundation
  work; could land before/alongside/after wat-fmt depending on
  user priority.
- **`wat check`** — user noted this CLI promotion path
  (presumably the wards' /check skill becomes a CLI subcommand
  alongside `wat fmt`). The subcommand pattern wat-fmt uses opens
  this door.

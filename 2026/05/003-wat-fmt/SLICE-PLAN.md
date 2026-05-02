# wat-fmt — SLICE PLAN

Five slices. Each ships independently. Per proposal 058's
discipline (ship only what's earned by cited use), each slice
has a real consumer that demands it.

---

## Slice 1 — Core formatter, no comments

**Goal:** parse → AST → emit, handling the substrate's common
forms. No comment preservation in this slice.

**Deliverables:**
- `wat-rs/crates/wat-fmt/` workspace member crate created
- Public API: `wat_fmt::format(input: &str) -> Result<String, FormatError>`
- AST emitters for: Atom, Bind, Bundle, lambda, let, let*,
  if, cond, define, defmacro, try, match, vec, HashMap, HashSet
- Special-form rules per STYLE-RULES.md §4 (whatever's settled
  by start-of-implementation)
- Indent / line-length logic per STYLE-RULES.md §1, §2
- Tests:
  - Golden files for each substrate form (input.wat + expected.wat)
  - Round-trip stability: `format(format(x)) == format(x)` (idempotent)
  - Semantic preservation: `parse(format(x)) == parse(x)`

**Consumer:** wat-rs's own codebase. Run wat-fmt on the entire
`wat/` tree; verify output is structurally identical to input
(modulo whitespace). The substrate eats its own dog food.

**Out of scope:**
- Comment preservation (slice 2)
- CLI integration (slice 3)
- Recursive directory traversal
- File IO (only string-to-string)

**Estimated size:** ~1500-2500 LOC of Rust, mostly emitter
functions per AST node type + tests.

---

## Slice 2 — Comment preservation

**Goal:** comments in the input survive to the output, in the
right relative position per STYLE-RULES.md §3.

**Deliverables:**
- Comment-preserving tokenizer variant in
  `wat-rs/crates/wat-fmt/src/tokenizer.rs` (or extend wat-rs's
  tokenizer with a `keep_comments: bool` mode)
- AST extension: `Comment` type; comments attached to adjacent
  AST nodes during parsing
- Emitter handles leading / trailing-inline / section-break
  comment positions per Rules 7-10 in STYLE-RULES.md
- Tests:
  - Every comment in input appears in output
  - Comment positions preserved (leading stays leading; trailing
    inline stays trailing inline; section-break stays section-break)
  - Round-trip: `format(format(x)) == format(x)` still holds with
    comments in the input

**Consumer:** wat-rs's own codebase, again. But this time the
codebase has comments; wat-fmt must preserve them. The wards
(/sever, /reap, etc.) are good test corpora because they have
substantive comments.

**Out of scope:**
- `;; @format-off` / `;; @format-on` directives (open question
  D in DESIGN.md; decision deferred)

**Estimated size:** ~500-1000 LOC.

---

## Slice 3 — CLI subcommand integration

**Goal:** `wat fmt PATH` works at the command line, with
`--check`, `--diff`, `--write`, `--stdin` modes per DESIGN.md's
CLI section.

**Deliverables:**
- `wat-cli` adds `fmt` subcommand
- Argument parsing (clap or whatever wat-cli uses)
- Modes: format-in-place (default), `--check`, `--diff`,
  `--stdin`, recursive directory walk
- Exit code contract per DESIGN.md (0 / 1 / 2 / 3)
- Tests:
  - CLI integration tests (spawn `wat fmt ...`, check output,
    check exit code)
  - Recursive directory mode skips non-`.wat` files

**Consumer:** the user (typing `wat fmt foo.wat` at a terminal),
plus pre-commit hook for the wat-rs repo, plus CI.

**Out of scope:**
- Editor plugins (slice 4 establishes the contract for them; the
  plugins themselves are downstream)

**Estimated size:** ~300-600 LOC.

---

## Slice 4 — Editor contract (no plugins ship)

**Goal:** establish the `--check --stdin` contract that any
future editor plugin can rely on.

**Deliverables:**
- Document the contract in `wat-rs/crates/wat-fmt/README.md`:
  - Editor calls `wat fmt --check --stdin` with file content on
    stdin
  - Exit 0 = formatted (no action needed)
  - Exit 1 = needs format (editor displays warning or
    auto-formats by re-piping through `wat fmt --stdin`)
  - Exit 2 = parse error (editor displays parse diagnostic)
- A reference shell script demonstrating the integration
  pattern (like `examples/editor-integration.sh`)
- Tests:
  - Stdin pipe modes work; exit codes match contract

**Consumer:** establishes the surface so future plugin authors
(VSCode extension, Emacs mode, Neovim plugin) have a stable
target. No plugins ship from wat-fmt itself.

**Out of scope:**
- Actual editor plugins
- LSP integration (separate arc; would be wat-lsp or similar)

**Estimated size:** ~100-200 LOC + docs.

---

## Slice 5 — Linter foundation (deferred)

**Goal:** wat-fmt's parser becomes shared infrastructure for
wat-lint (the next item from random-notes.txt).

**Deliverables:**
- The comment-preserving tokenizer + AST-with-comments graduates
  from `wat-fmt` to `wat-rs` proper, becoming a shared parser
  mode
- `wat-fmt` depends on the moved infrastructure
- The wat-lint crate (next arc) can also depend on it

**Consumer:** wat-lint, the next arc. wat-lint can't open until
wat-fmt's parser infrastructure is stable.

**Status:** explicitly deferred. This slice doesn't open until
either:
- wat-lint is being designed (and needs the shared infra), OR
- A second consumer of the comment-preserving parser surfaces

**Estimated size:** small — mostly a re-org / re-export, plus
updating one Cargo.toml dependency.

---

## Sequencing

Linear. 1 → 2 → 3 → 4. Slice 5 deferred until wat-lint demands.

| Slice | Depends on | Blocks | Estimated time |
|---|---|---|---|
| 1 | nothing (foundations only) | 2, 3 | 1-2 weeks |
| 2 | 1 | 3 (CLI is more useful with comments) | 1 week |
| 3 | 1, 2 | nothing critical | 3-5 days |
| 4 | 3 | future editor plugins | 1-2 days |
| 5 | wat-lint demanding it | wat-lint | 1-2 days when triggered |

Slice 1 is the bulk. Slices 2-4 are smaller increments on top.

## What success looks like

- wat-rs's own codebase passes `wat fmt --check` cleanly (after
  one initial reformat to bring everything to canonical style).
- Pre-commit hook in wat-rs runs `wat fmt --check`; CI runs
  `wat fmt --check --recursive .`.
- Style discussions move from chat to STYLE-RULES.md edits;
  rule changes are diffs to the formatter, not arguments in PRs.
- `wat fmt` is fast enough that running it on save in editors
  feels instant (sub-100ms for typical files).
- The formatter's output never causes a code review comment about
  whitespace — that argument is just gone.

## What's deferred

- **Configurability.** Per DESIGN.md "Configuration philosophy":
  start with NO config. Add when cited use earns it.
- **`;; @format-off` directives.** Per DESIGN.md Open Question B.
- **Multi-language support** (e.g., wat-english consumer crate
  formatted differently). If wat-english ships, its forms get
  formatted by wat-fmt's same pipeline; no separate formatter.
- **Refactoring features.** wat-fmt formats; wat-lint flags;
  refactoring is a third tool (or never, if wat-lint with `--fix`
  covers the use cases).

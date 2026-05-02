# wat-lint — SLICE PLAN

Five slices. Each ships independently. Per proposal 058's
discipline (ship only what's earned by cited use), each slice
has a real consumer that demands it.

---

## Slice 1 — Runner + complectens Phase-1 rules

**Goal:** wat-lint as a self-contained crate that runs the five
complectens rules over wat code and emits structured EDN
findings. No CLI, no rune suppression, no JSON yet — just
Rust API + wat code.

**Deliverables (all inside `wat-rs/crates/wat-lint/`):**

*Rust shim* (`src/`):
- Workspace member crate created
- Public Rust API:
  - `lint(input: &str, rules: &[&str]) -> Result<Vec<Finding>, LintError>`
  - `lint_file(path: &Path, rules: &[&str]) -> ...`
- `lib.rs` parses input, loads embedded wat code, invokes
  `:wat::lint::run` via the wat-vm, returns findings
- `findings.rs` — typed `Finding` struct + EDN deserialization
- `invoke.rs` — wat-vm invocation helper
- arc-013 contract: `wat_sources()` + `register()`

*wat code* (`wat/lint/`):
- `runner.wat` — top-level entry: `(:wat::lint::run target rules)`
- `complectens/deftest-body-length.wat` — Rule 1.1
- `complectens/let-star-binding-count.wat` — Rule 1.2
- `complectens/forward-reference.wat` — Rule 1.3
- `complectens/stepping-stone-multi-file.wat` — Rule 1.4
- `complectens/helper-without-deftest.wat` — Rule 1.5
- `primitives/ast-walk.wat` — shared walker
- `primitives/finding.wat` — finding constructor

*Tests:*
- Golden findings for each rule, derived from arc 130's
  `complected-2026-05-02/` calibration set
- Round-trip: `lint(format(x)) == lint(x)` (formatting doesn't
  change findings)
- Rust integration tests against a small wat fixture corpus

**Consumer:** the complectens spell — its Phase-1 mechanical
pass becomes `wat lint --rule complectens/* <file> --edn`. The
spell's SKILL.md gets updated to reference the wat-lint command
instead of the bootstrap `.claude/skills/<spell>/<spell>.wat`.

**Out of scope:**
- Rune suppression (slice 2)
- JSON output (slice 3)
- CLI integration (slice 3)
- Custom-rule discovery (slice 4)
- Layout-aware rules (slice 5)

**Estimated size:**
- ~300-500 LOC of Rust (shim + finding plumbing)
- ~600-1000 LOC of wat (5 rules + runner + primitives)

---

## Slice 2 — Rune suppression + EDN output

**Goal:** support runes per the holon-lab-trading discipline.
Findings get a `:rune` field; suppressed findings ARE in the
output (with the rune attached) but don't trigger nonzero exit.
Pretty-printed EDN output.

**Deliverables:**

*Rust shim:*
- `runes.rs` — pre-parse text inspection for `;; rune:<spell>(<category>) — <reason>`
- Rune metadata attached to AST nodes before rule evaluation
- Rune category vocabulary validation (per Open Question A in
  DESIGN.md — enforce; unknown category → meta-finding)

*wat code:*
- Each rule updated to consume rune metadata; if a matching
  rune exists at the finding's site, set the `:rune` field
  and skip the L1/L2 trigger

*EDN output formatter* (`output.rs`):
- Pretty-printed EDN by default (no flag needed)
- Top-level `:findings` array + `:summary` object
- Each finding includes `:rune` field (nil if not suppressed)

**Consumer:** the user — anyone running `lint(file)` from
their Rust code gets EDN they can read top-down OR pipe to a
parser. wat-rs's own codebase becomes the first user (run lint
on existing wat-tests/; add runes where the discipline allows
exceptions).

**Out of scope:**
- JSON output (slice 3)
- CLI subcommand (slice 3)

**Estimated size:** ~200-400 LOC of Rust + minor wat updates
(rune-awareness in each rule).

---

## Slice 3 — JSON output + CLI integration

**Goal:** `wat lint PATH` works at the command line; `--json`
emits JSON via wat-edn; `--check` exit-code contract for
CI / pre-commit.

**Deliverables:**

*Rust shim:*
- `--json` flag: pipes EDN through `wat_edn::edn_to_json` (per
  user direction: *"wat-edn provides an edn->json interface"*)
- `--check` flag: exit nonzero if any L1+L2 findings exist
  (suppressed-by-rune don't trigger; they're conscious)
- `--rule <name>` and `--spell <name>` filters
- `--severity <L1|L2|L3>` filter
- `--stdin` mode for piped use

*wat-cli integration:*
- Add `lint` subcommand to wat-cli alongside `fmt`
- Same dep registration shape: `wat_cli::run(&[(register, wat_sources), ...])`
- Exit code contract:
  - `0` — no findings (or all findings runed)
  - `1` — L1+L2 findings present (in `--check` mode)
  - `2` — parse error
  - `3` — IO error

**Consumer:** wat-cli users; pre-commit hooks; CI pipelines.

**Out of scope:**
- Custom-rule discovery (slice 4)
- Editor integration plugins (slice 5+)

**Estimated size:** ~200-400 LOC of Rust + CLI plumbing.

---

## Slice 4 — Custom-rule discovery (project-local + crate-distributed)

**Goal:** users can define their own lint rules and have them
loaded by the runner — both as `.wat` files in a project-local
directory and as separate crates that depend on `wat-lint`.

**Deliverables:**

*Rust shim:*
- `wat-lint.toml` config file parsing (per DESIGN.md custom-rules
  section)
- Project-local rule discovery: `wat-lint-rules/*.wat` (path
  configurable via `[discovery]` section)
- Crate-distributed rule discovery: rules registered via the
  arc-013 `wat_sources()` contract by any crate the user lists
  in their Cargo.toml + `wat-lint.toml`'s `[require]` section
  (RuboCop-equivalent)
- Severity overrides: `[severity]` section in the config
- Rule-specific config: per-rule `[rules.<name>]` table for
  thresholds etc.

*wat code:*
- Runner extended to load rules from both paths
- Rule-loading-order conflict resolution (project-local wins;
  warning emitted)

**Consumer:** the first user who writes a custom rule. Could
be ad-hoc (drop a `.wat` file in their project) or a
distributable crate (e.g., a hypothetical `acme-lints` crate).

**Out of scope:**
- Layout-aware rules (slice 5)
- Editor LSP integration (separate arc)

**Estimated size:** ~300-500 LOC of Rust (config parsing,
discovery, registration) + wat-lint.toml schema docs.

---

## Slice 5 — Layout-aware rules using wat-fmt

**Goal:** the lint rules that depend on wat-fmt's analyses
land. wat-lint adds `wat-fmt` as a dependency (already in
Cargo.toml from slice 1, just for rules that haven't shipped
yet).

**Deliverables:**

*wat code:*
- `layout/over-line-limit.wat` — flag lines that would be over
  120 cols after `wat fmt`
- `layout/post-format-drift.wat` — flag forms whose layout
  would change significantly under `wat fmt` (canonicalize-then-diff)
- `layout/oversized-symbol.wat` — flag symbols that push a
  line past 120 cols even after format (the "make a type alias"
  signal — per wat-fmt Rule 23 / 13c)

*Each rule calls* `:wat::fmt::*` *primitives* exposed by the
wat-fmt crate's wat code (e.g., `:wat::fmt::would-wrap?`,
`:wat::fmt::indent-of`).

**Consumer:** users who want to enforce layout discipline in
CI. The wat-rs codebase becomes the first user once a real
"this codebase has consistent layout" guarantee matters.

**Status:** explicitly deferred. This slice doesn't open until:
- wat-fmt slice 1 has shipped (so the layout-query primitives
  exist)
- A real consumer demands layout-aware lints (most lint use
  cases are structural, not layout)

**Estimated size:** ~200-400 LOC of wat (3 layout rules) +
minor Rust plumbing (wat-fmt is already a dep from slice 1).

---

## Sequencing

Linear 1 → 2 → 3. Slice 4 can land in parallel with slice 5
once slice 3 is done. Slice 5 explicitly deferred until
wat-fmt slice 1 lands.

| Slice | Depends on | Blocks | Estimated time |
|---|---|---|---|
| 1 | nothing (foundations only) | 2, 3 | 1-2 weeks |
| 2 | 1 | 3 (CLI is more useful with rune support) | 3-5 days |
| 3 | 1, 2 | nothing critical | 3-5 days |
| 4 | 3 | nothing | 3-5 days |
| 5 | wat-fmt slice 1; ideally 3 | nothing | 2-3 days |

Slice 1 is the bulk. Slices 2-5 are smaller increments.

## What success looks like

- **complectens spell can drop its bootstrap `.claude/skills/<spell>/<spell>.wat`** and reference `wat lint --spell complectens` instead. The spell's SKILL.md gets simpler.
- wat-rs's own wat-tests/ tree passes `wat lint --check` (after
  one initial sweep + adding runes where appropriate).
- Pre-commit hook in wat-rs runs `wat lint --check`; CI runs
  `wat lint --check --recursive .`.
- `wat lint --edn` is fast enough to run on save in editors
  (sub-200ms target on 10k-line files).
- Spells (perspicere, vocare, complectens) all reference
  wat-lint commands rather than maintaining their own
  bootstrap wat scripts.
- Future user-defined lint rules are easy to add via either
  project-local files OR distributable crates; both paths
  feel symmetric.

## What's deferred

- **Editor integration / LSP.** Separate arc; would be
  `wat-lsp` or similar. wat-lint provides the underlying
  engine; LSP is a shell over it.
- **Auto-fix mode.** wat-lint flags; wat-fmt formats; refactoring
  is a third tool (or never). No `--fix` flag in v1.
- **Lint discovery via tags / categories.** RuboCop has cop
  groups (`Style/`, `Lint/`, `Performance/`); wat-lint v1 uses
  spell-prefixed rule names (`complectens/...`). Tag-based
  selection can add later if a real consumer wants it.
- **Lint rule documentation generation.** Each rule has a doc
  comment; future tooling could emit a unified rule reference
  doc. v1 ships rules with their inline docs; reference doc
  is a v2 nice-to-have.
- **Cross-language lints.** wat-lint lints wat. Rust uses
  clippy. Python uses ruff. No unified meta-linter.

# wat-doc — SLICE PLAN

Four slices. Each ships independently. Per proposal 058's
discipline (ship only what's earned by cited use), each slice
has a real consumer that demands it.

---

## Slice 1 — Substrate change + parser handling

**Goal:** the substrate accepts the 3-arg form for `define` /
`lambda` / `defmacro`. Type checker recognizes optional
docstring slot. Backwards-compat preserved (2-arg forms still
work). STYLE-RULES.md amended.

**Deliverables:**

*wat-rs core changes:*
- `define` / `lambda` / `defmacro` arity expanded to accept
  optional docstring at position [1]
- Type checker: validates `:String` at the optional position;
  rejects non-String with clear diagnostic
- Runtime evaluator: ignores docstring (metadata only); body
  evaluation unchanged
- Substrate proposal arc opened in `wat-rs/docs/proposals/`
  for the substrate change (058's discipline requires this)

*wat-fmt amendments:*
- Rule 14 / 14b / 14c updated in STYLE-RULES.md
- wat-fmt's per-form emitters add docstring rendering
- Multi-line docstring follows Rule 31 (atomic)
- Tests: golden files for each rule with + without docstring

*Tests:*
- Round-trip: existing 2-arg forms parse + format identically
- New 3-arg forms parse, type-check, format correctly
- Docstring as non-String → clear type error

**Consumer:** wat-doc itself (slice 2 needs the substrate
change to extract docstrings). Plus any wat code that wants
to start adding docstrings.

**Out of scope:**
- wat-doc crate (slice 2)
- wat-lint missing-docstring rule (slice 4)
- HTML output (slice 3)

**Estimated size:**
- ~100-200 LOC of Rust changes in wat-rs (3-arg parsing +
  type check + diagnostics)
- ~50-100 LOC of changes in wat-fmt's emitters per form
- STYLE-RULES.md amendments (small)

---

## Slice 2 — wat-doc crate with extractor + EDN + Markdown

**Goal:** `wat-rs/crates/wat-doc/` exists; walks AST; emits
EDN doc tree + Markdown. The first usable docs.

**Deliverables:**

*wat-doc crate:*
- Workspace member crate created per arc-013 contract
- Public Rust API:
  - `document(input: &str) -> Result<DocTree, DocError>`
  - `document_crate(path: &Path) -> ...`
- `DocTree` struct: per-crate, per-module, per-form
- `src/extract.rs` — AST walker; pulls (form, signature,
  docstring) triples
- `src/formatters/edn.rs` — EDN emission
- `src/formatters/markdown.rs` — Markdown emission

*wat code* (`wat/doc/`):
- Extraction filters (which forms count as public)
- Default include/exclude patterns

*Tests:*
- Golden EDN outputs for fixture corpus
- Golden Markdown outputs for the same corpus
- Round-trip: extracted docs round-trip through EDN write/read

**Consumer:** the wat-rs codebase itself. Generate Markdown
docs for wat-fmt, wat-lint, wat-cov; embed in their READMEs
or commit alongside.

**Out of scope:**
- HTML site (slice 3)
- Cross-references between crates (slice 3)
- wat-lint integration (slice 4)
- JSON output (slice 4)
- Fallback to leading comments (slice 4 polish)

**Estimated size:**
- ~400-700 LOC of Rust (extractor + EDN + Markdown formatters)
- ~100-200 LOC of wat (filters + extraction rules)

---

## Slice 3 — HTML output (codox-style) + cross-references

**Goal:** browseable static HTML docs with working cross-
references; the polish layer that makes docs feel professional.

**Deliverables:**

*wat-doc crate:*
- `src/cross_ref.rs` — FQDN parsing in docstrings; resolution
  against the doc tree; broken-reference detection
- `src/formatters/html.rs` — HTML site emission:
  - Index page with crate navigation
  - Per-module pages
  - Per-form sections (signature + docstring + cross-references)
  - Searchable (client-side; no JS framework)
- HTML templates (askama or tera)
- Minimal CSS (single file; no asset bundling complexity)

*Cross-references in Markdown* (added retroactively to slice 2's
Markdown formatter):
- FQDN mentions render as Markdown links
- Same resolution mechanism as HTML

*Tests:*
- Golden HTML outputs (snapshot match)
- Cross-reference resolution correctness (FQDN → link)
- Broken-reference detection emits warnings

**Consumer:** anyone wanting browseable wat docs. The wat-rs
project's public docs become the first user.

**Out of scope:**
- wat-lint integration (slice 4)
- JSON output (slice 4)
- Project-wide multi-crate composition (deferred to v2)

**Estimated size:**
- ~400-600 LOC of Rust (HTML formatter + cross-ref resolver)
- ~200-400 LOC of HTML/CSS templates
- ~100-200 LOC of wat (cross-ref rules in wat layer if needed)

---

## Slice 4 — wat-lint integration + JSON output + polish

**Goal:** docstrings become enforceable; the docs ecosystem
expands to the JSON consumer surface; small polish items
surfaced from slices 1-3.

**Deliverables:**

*wat-lint integration:*
- New rule: `documentation/missing-public-docstring`
- Lives in `wat-rs/crates/wat-lint/wat/lint/documentation/`
- Severity tunable per project; default L2-candidate
- Rune categories: `documentation(self-evident)`,
  `documentation(transitional)`
- New rule: `documentation/broken-cross-ref` (FQDN mentions
  in docstrings that don't resolve)

*wat-doc crate:*
- `src/formatters/json.rs` — JSON emission via wat-edn
- `--check` CLI mode: exits nonzero on missing docstrings (uses
  wat-lint under the hood)
- Transitional fallback: read leading `;;` comments when
  docstring slot is empty (eases migration)

*Polish items* (driven by real-world feedback from slices 1-3):
- Edge cases in cross-reference resolution
- HTML theme refinements
- Markdown flavor specifics (CommonMark vs GFM)
- Multi-line docstring rendering edge cases

*Tests:*
- wat-lint rule tests with rune-suppression scenarios
- JSON output round-trips through wat-edn correctly
- Fallback-to-comments preserves existing wat-rs codebase docs
  during migration

**Consumer:** wat-rs project's CI (enforce docstrings on public
forms); ecosystems wanting JSON-formatted docs; users
migrating from comments-above to first-class docstrings.

**Out of scope:**
- Project-wide multi-crate doc composition (deferred to v2)
- LLM-generated docstring suggestions (downstream user-invoked
  tooling; not wat-doc's responsibility per LLM-out discipline)
- Documentation versioning (project / crate concern, not
  wat-doc's)

**Estimated size:**
- ~200-400 LOC of Rust (JSON + lint integration + fallback)
- ~50-100 LOC of wat (lint rule)
- ~100-200 LOC of polish items across all formatters

---

## Sequencing

Linear 1 → 2 → 3 → 4. Slice 1 is the substrate change; it
must land first because everything else depends on the doc
slot existing.

| Slice | Depends on | Blocks | Estimated time |
|---|---|---|---|
| 1 | wat-rs core; wat-fmt slice 1 | 2, 3, 4 | 1 week |
| 2 | 1 | 3, 4 | 1-2 weeks |
| 3 | 2 | 4 | 1-2 weeks |
| 4 | 2, 3; wat-lint slice 1 | nothing | 4-7 days |

Slice 1 is small in LOC but has process overhead (substrate
proposal arc, type-checker validation, backwards-compat
testing). Slices 2-4 are larger in LOC but pure consumer-crate
work.

## What success looks like

- **Substrate change lands cleanly** without breaking existing
  wat-rs code (backwards-compat preserved; existing 2-arg forms
  unchanged)
- **wat-rs's own crates use the docstring slot** for public
  forms (after slice 2 + slice 4 lint pressure)
- **Public wat-rs docs** generated via wat-doc; the docs site
  is the first real consumer of slice 3's HTML output
- **CI enforces docstrings** via `wat doc --check` (slice 4)
- **Migration from comments-above is gradual** — fallback
  reads existing leading `;;` comments during transition;
  long-term, the docstring slot is canonical
- **wat-doc is the fourth foundation crate** alongside wat-fmt,
  wat-lint, wat-cov. The four-corner toolkit is complete.

## What's deferred

- **Project-wide multi-crate doc composition** (cargo doc
  --workspace equivalent) — useful but not needed for v1; ship
  per-crate first, add composition when a real consumer
  surfaces the need.
- **Documentation versioning** (per-tag doc trees) — a project /
  release-engineering concern, not wat-doc's.
- **LLM-generated docstring suggestions** — downstream
  user-invoked tooling per LLM-out discipline. wat-doc emits
  structured data; LLM tools consume it.
- **IDE integration** (hover docs, go-to-doc) — IDE plugins
  consume wat-doc's EDN output; plugin development is
  downstream.
- **Code-example execution** (doctest-style — execute examples
  in docstrings, verify output) — interesting future capability;
  out of scope for the doc generator (would be wat-doctest as
  its own arc).

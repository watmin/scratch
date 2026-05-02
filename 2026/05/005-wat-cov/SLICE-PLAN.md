# wat-cov — SLICE PLAN

Five slices. Each ships independently. Per proposal 058's
discipline (ship only what's earned by cited use), each slice
has a real consumer that demands it.

---

## Slice 1 — VM hooks + counter accumulation + EDN output

**Goal:** wat-cov as a self-contained crate that, when active,
tracks line/branch/function coverage during wat-vm execution
and emits structured EDN at end of run. No CLI yet; no formats
beyond EDN; no thresholds; no baseline.

**Deliverables:**

*wat-rs core changes (minimal; gated):*
- Coverage hooks added to `eval` in `wat-rs/src/runtime.rs`
- New `CoverageState` global (RwLock around HashMap<NodeId,
  HitCount>)
- On/off switch via `wat::coverage::enable()` /
  `wat::coverage::disable()`
- Compile-time elision via `#[cfg(feature = "coverage")]` for
  zero-cost when coverage isn't needed at all

*wat-cov crate* (`wat-rs/crates/wat-cov/`):
- Workspace member crate created
- Public Rust API:
  - `run_with_coverage<F: FnOnce()>(test_fn: F) -> Result<CoverageReport, CoverageError>`
  - `to_edn(report: &CoverageReport) -> String`
- `CoverageReport` struct with three dimensions (line/branch/
  function) per file
- `src/instrument.rs` — registers/manages CoverageState
- `src/accumulate.rs` — counter aggregation
- `src/formatters/edn.rs` — EDN emission

*wat code* (`wat/cov/`):
- Minimal — just enough to expose Rust-side coverage state to
  wat tooling (the runner.wat for wat-coded threshold checks
  comes in slice 4)

*Tests:*
- Golden EDN outputs for a small fixture corpus (function with
  uncalled branch; nested cond; uncalled lambda; etc.)
- Property: turning coverage on then off twice yields the same
  hit counts for the same input

**Consumer:** the wat-rs codebase itself. Run `wat-cov` over
the existing wat-tests/ tree; see what's covered. The first
real run is the validation.

**Out of scope:**
- CLI integration (slice 2)
- Cobertura output (slice 2)
- Hermetic test transport (slice 3)
- HTML output (slice 4)
- Thresholds / baseline / refuse_coverage_drop (slice 4)
- wat-as-config (slice 5)

**Estimated size:**
- ~150-300 LOC of Rust changes in wat-rs/src (hooks + state)
- ~500-800 LOC of Rust in wat-cov crate (instrument + accumulate
  + EDN formatter + tests)

---

## Slice 2 — Cobertura XML formatter + CLI integration

**Goal:** `wat cov` subcommand works at the command line; emits
Cobertura XML for CI ingestion. The first slice users can
actually invoke.

**Deliverables:**

*wat-cov crate:*
- `src/formatters/cobertura.rs` — XML emission per Cobertura
  schema
- CLI argument parsing
- `report` subcommand with `--cobertura` and `--edn` flags
- `run` subcommand that wraps `run_with_coverage` over a
  test invocation

*wat-cli integration:*
- Add `cov` subcommand to wat-cli alongside `fmt` and `lint`
- Same dep registration shape:
  `wat_cli::run(&[..., (wat_cov::register, wat_cov::wat_sources)])`
- Exit code contract:
  - `0` — success
  - `2` — parse / runtime error
  - `3` — IO error

**Consumer:** wat-rs codebase + any CI pipeline. Cobertura XML
ingests into Jenkins/GitLab/GitHub Actions natively.

**Out of scope:**
- Hermetic transport (slice 3)
- JSON output (slice 3)
- HTML output (slice 4)
- Thresholds (slice 4)

**Estimated size:**
- ~300-500 LOC of Rust (Cobertura formatter + CLI)
- Cobertura schema is well-documented; emission is mechanical

---

## Slice 3 — Hermetic transport + result merging + JSON

**Goal:** coverage data from hermetic test children gets merged
into the parent's report. JSON output via wat-edn for
ecosystem ingestion.

**Deliverables:**

*wat-cov crate:*
- `src/transport.rs` — per-child file transport in
  `/tmp/wat-cov-<run-id>/child-<pid>.edn`
- Child write at exit (signal handler ensures even on crash
  partial data is preserved)
- Parent merge logic at end-of-run
- `merge` subcommand for explicit merging:
  `wat cov merge run1.edn run2.edn`
- `src/formatters/json.rs` — JSON emission via `wat_edn::edn_to_json`

*wat-rs core changes:*
- Hermetic test runner integration (the `run-sandboxed-hermetic-ast`
  pathway needs to know about the per-run transport directory)

*Tests:*
- Hermetic test fixture that spawns N children; each writes
  partial coverage; parent merges; result equals what a
  single-process run would have produced
- Crash-resilience test: kill a child mid-run; verify parent
  reads what's there + warns

**Consumer:** wat-rs codebase's hermetic test discipline. Once
this slice ships, the existing `:wat::test::deftest-hermetic`
tests start contributing coverage data to the unified report.

**Out of scope:**
- HTML output (slice 4)
- Thresholds (slice 4)

**Estimated size:**
- ~300-500 LOC of Rust (transport + merge logic + JSON formatter)
- ~50-100 LOC of changes in wat-rs core (hermetic runner
  integration)

---

## Slice 4 — HTML formatter + thresholds + baseline / refuse_coverage_drop

**Goal:** the killer features. HTML reports for humans;
threshold gates for CI; baseline-drop prevention for PR gating.

**Deliverables:**

*wat-cov crate:*
- `src/formatters/html.rs` — HTML emission with file table +
  color-coded source listings (askama or tera templating)
  - File table sortable by coverage %
  - Per-file source view with green/red/yellow line coloring
- `src/thresholds.rs` — overall + per-file + per-dimension
  threshold checks
- `src/baseline.rs` — read/write `.wat-cov-baseline.edn`;
  compare current vs baseline; report regressions
- CLI extensions:
  - `wat cov check --minimum 80` — exit nonzero if below
  - `wat cov check --baseline` — exit nonzero if regressed
  - `wat cov update-baseline` — rewrite baseline file
  - `wat cov report --html` — emit HTML

*wat code* (`wat/cov/`):
- `runner.wat` — wat-coded threshold check rules (this is where
  the wat layer earns its keep; threshold logic is small but
  user-customizable)
- `filters.wat` — wat-coded filter logic (include/exclude
  patterns)

*Tests:*
- HTML output golden tests (snapshot HTML against expected)
- Threshold check tests (above / at / below threshold cases)
- Baseline comparison tests (no change / improvement /
  regression)

**Consumer:** the user (CI pipeline) — once thresholds + baseline
are in, coverage becomes a guardrail not a metric.

**Out of scope:**
- wat-as-config (slice 5)

**Estimated size:**
- ~400-600 LOC of Rust (HTML templating + thresholds + baseline)
- ~100-200 LOC of HTML/CSS templates
- ~100-200 LOC of wat (threshold/filter rules)

---

## Slice 5 — wat-as-config option + polish

**Goal:** support `wat-cov.wat` as an alternative to
`wat-cov.edn` for users who want programmatic config. Plus
any polish surfaced by real-world usage from slices 1-4.

**Deliverables:**

*wat-cov crate:*
- `src/config.rs` — config loading: try `wat-cov.edn` first;
  fall back to `wat-cov.wat`; CLI `--config <path>` overrides
- wat-as-config evaluator: load `wat-cov.wat` via wat-vm; expect
  it to evaluate to a config map matching the EDN schema
- Documentation of both formats; same fields; different surfaces

*Polish items* (driven by real-world feedback from slices 1-4):
- Performance tuning if needed
- Edge cases that surfaced in real test corpora
- Documentation improvements

**Consumer:** users who need programmatic config (rare; most
projects fine with EDN).

**Status:** explicitly deferred. Probably opens 6-12 months
after slice 4 lands, IF a real consumer surfaces a programmatic
config need. Until then, EDN is enough.

**Estimated size:**
- ~100-200 LOC of Rust (config loader + wat-vm integration)
- ~50 LOC of wat (config schema example)

---

## Sequencing

Linear 1 → 2 → 3 → 4. Slice 5 explicitly deferred.

| Slice | Depends on | Blocks | Estimated time |
|---|---|---|---|
| 1 | wat-rs core hooks | 2, 3, 4 | 1-2 weeks |
| 2 | 1 | 3, 4 (CLI is more useful with all formatters) | 4-7 days |
| 3 | 1, 2 | 4 (HTML is more useful with hermetic merge) | 1-2 weeks |
| 4 | 1, 2, 3 | nothing critical | 1-2 weeks |
| 5 | 4 (and real-world feedback) | nothing | 3-5 days when triggered |

Slice 1 is the bulk because it requires wat-rs core changes
(coverage hooks in eval). Slices 2-4 are smaller increments
on top.

## What success looks like

- **wat-rs codebase becomes the first user**, after slice 1
  ships. Run `wat cov run` against existing wat-tests/; see
  what's covered. The first run reveals real gaps.
- **CI integration via Cobertura** (slice 2) — every PR shows
  coverage in the CI dashboard.
- **Hermetic tests contribute to coverage** (slice 3) — the
  `:wat::test::deftest-hermetic` tests become first-class
  coverage citizens.
- **PR coverage gating via refuse_coverage_drop** (slice 4) —
  coverage stops being a vanity metric, becomes a guardrail.
- **HTML reports for humans browsing** (slice 4) — the polish
  layer; SimpleCov-style.
- **wat-cov is the third foundation crate** alongside wat-fmt
  and wat-lint. The triad: format the code, lint the code,
  measure how much of the code is exercised.

## What's deferred

- **Code complexity metrics** (cyclomatic complexity, nesting
  depth) — these are linter territory; route through wat-lint.
- **Mutation testing** — different discipline; separate
  arc/crate if it ever ships.
- **Coverage trend graphs** (week-over-week) — downstream
  consumer territory; wat-cov produces the data, dashboards
  consume it.
- **IDE integration** (highlighted gutter for uncovered lines)
  — IDE plugins consume wat-cov's EDN/JSON output; plugin
  development is downstream.
- **Sampling coverage** (count every Nth eval) — performance
  optimization; ship only if real-world latency demands it.
- **Production tracing** — wat-telemetry's domain, not
  wat-cov's.

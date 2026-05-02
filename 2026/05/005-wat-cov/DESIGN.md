# wat-cov — DESIGN

Architecture, coverage dimensions, attribution rules, hermetic
transport protocol, configuration, formatters, baseline-drop
prevention. The four questions applied throughout.

---

## The four questions are the design compass

Per the established discipline (carried from wat-fmt and
wat-lint arcs):

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; coverage gaps
  are surfaced, not smoothed.
- **Good UX?** A user can do the right thing without ceremony.

Obvious + Simple + Honest must hold before Good UX is even
considered.

## The three coverage dimensions

Each maps cleanly to a HolonAST property:

### Line coverage

Was this source line evaluated? Tracked per `(file, line)`.

The most basic and most ubiquitous metric. CI dashboards default
to line coverage. Easy to interpret: red lines never ran.

### Branch coverage

For each `:wat::core::if` / `:wat::core::cond` / `:wat::core::match`,
was each arm taken? Tracked per `(file, line, arm-index)`.

The conditional family from wat-fmt Rule 16 is exactly the
universe of branches we instrument:
- `:if` — two branches (then, else)
- `:cond` — N branches (one per clause + `:else`)
- `:match` — N branches (one per pattern arm)

### Function coverage

Was this `:wat::core::define` / `:wat::core::lambda` /
`:wat::core::defmacro` ever called? Tracked per FQDN.

Captures uncalled functions ("dead code by definition") — high
signal, low noise. SimpleCov calls this method coverage; for
wat we call it function coverage since that's the substrate's
abstraction.

## Q1 (locked) — instrumentation lives in the wat-vm

User direction:

> *"A -- i think this must be in the vm not the lang?..."*

Confirmed. **Coverage is a wat-vm concern, not a language
concern.** The language doesn't know about coverage; the vm does.

**Implementation:** every `eval` call in the wat-vm optionally
increments a counter at the AST node it's evaluating. Controlled
by a global on/off switch (cheap when off; precise when on).
Like Ruby's `Coverage` stdlib — runtime-tracked, not source-
transformed.

Counter storage: a `HashMap<NodeId, HitCount>` where `NodeId`
identifies the AST node by `(file, span)`. The vm looks up the
counter and increments; if coverage is off, the lookup is
skipped entirely (single boolean check).

**Why this beats the AST-rewriting alternative:**
- Source code is never modified at load time
- Coverage and execution are the same operation; impossible to
  drift
- The vm is the single source of truth for "what got evaluated"
- Turning coverage off is free at runtime (one boolean check
  per eval, dropped entirely under feature flag)

Per the four questions:
- **Obvious?** ✅ — vm tracks; tools read; no AST mutation
  surprises
- **Simple?** ✅ — one mechanism (counter increment in eval);
  one switch (on/off)
- **Honest?** ✅ — what got executed IS what got counted; no
  proxy mechanisms
- **Good UX?** ✅ — turn coverage on, run tests, get data; no
  source-modification ceremony

## Q2 (locked) — surface-evaluation attribution

User direction:

> *"the answer is... is this form evaluated?.... and if yes...
> how much of it was evaluated?.... -- that limits the concern
> to the surface?.. the use of a macro's result is different
> from the invocation of the macro?...."*

The attribution rule: **count the form the user wrote.** Don't
count the form the substrate constructed.

### Macro coverage attribution

For a macro invocation `(my-macro x y)` that expands to `(:wat::core::* x y)`:

- The **invocation site** `(my-macro x y)` is what counts. If
  the line containing `(my-macro x y)` was evaluated, that line
  is covered.
- The **expansion** `(:wat::core::* x y)` does NOT get separate
  coverage attribution. It's the macro author's internal; the
  user wrote the invocation, not the expansion.
- The **fact that the expansion was evaluated** is what
  determines coverage of the invocation. If `(my-macro x y)`
  expanded but the expansion wasn't evaluated (e.g., dead
  branch in the expansion), the invocation's line is NOT
  covered.

This means: macro authors don't see internal expansion
coverage; macro users see whether their invocation evaluated.
Clean separation of concerns.

### Quasiquote attribution

For `` `(:foo ,x ,y)`` — the template `(:foo ,x ,y)` is data
construction, not "evaluated user code." Only the unquoted
parts (`,x` and `,y`) are user code being evaluated.

- The **template structure** doesn't get coverage attribution
  (it's data, not control flow)
- The **unquoted forms** (`,x`, `,y`, `,@xs`) DO get coverage
  attribution as normal user code

### Substrate call attribution

`:wat::core::*` and `:wat::holon::*` evaluations are below the
user-code surface. They're invoked BY user code; they're not
themselves user code.

- **Default exclusion:** the `wat-rs/wat/` tree (including
  substrate stdlib) is excluded from coverage by default
- The `:wat::core::*` etc. evaluations are attributed to the
  user's call site; they don't get separate counts
- Users can opt back in via `wat-cov.edn` filters if they're
  developing the substrate itself

Per the four questions:
- **Obvious?** ✅ — coverage matches what the user wrote;
  expansions and substrate are below the line
- **Simple?** ✅ — one rule: count the surface; below-surface
  doesn't count
- **Honest?** ✅ — macro authors aren't blamed for users'
  uncovered invocations; users aren't blamed for substrate
  internals
- **Good UX?** ✅ — coverage report matches the user's mental
  model of "did my code run?"

## Q3 (locked) — hermetic transport protocol

User direction:

> *"oooo good find on hermetic... yea.. we need a transport
> protocol forr this?..."*

wat-rs has hermetic test execution (per arc 058-035, fork-
substrate). Each hermetic child is a forked process; coverage
data needs to get from child back to parent.

**Transport: per-child file in a known temp directory, parent
merges at end.**

```
/tmp/wat-cov-<run-id>/
  child-<pid>.edn        # each child writes its counters here
                         # at exit
  child-<pid>.edn        #
  ...
```

At test-run end:
1. Parent waits for all children to exit
2. Parent reads all `child-*.edn` files
3. Parent merges (sum counters per `NodeId`)
4. Parent emits unified report
5. Parent cleans up the temp directory

**Why per-child file, not pipe:**
- Pipe requires parent-active connection; if parent crashes
  mid-run, all coverage data is lost
- Per-child file is robust to crashes; the parent can recover
  partial coverage from whatever children wrote before the
  crash
- Files are easier to debug (you can `cat` them); pipes are
  ephemeral
- File-based transport is what most coverage tools use
  (gcov, JaCoCo, etc.); the discipline is well-understood

**Failure mode handling:**
- If a child crashes mid-run, its file may be partial (or
  empty); parent reads what's there + warns
- If the parent crashes, the temp directory + child files
  remain; user can rerun `wat cov merge /tmp/wat-cov-<run-id>/*.edn`
  to recover

**File format:** the same EDN shape as the final report,
scoped to one child:

```edn
{:cov
 {:run-id "abc123"
  :child-pid 12345
  :test-name ":test::cache-round-trip"
  :counters
  {("wat-tests/lru/CacheService.wat" 87) 5
   ("wat-tests/lru/CacheService.wat" 88) 5
   ...}}}
```

Per the four questions:
- **Obvious?** ✅ — files in a temp dir; one per child; named
  by pid
- **Simple?** ✅ — write at child exit; read at parent merge;
  no IPC complexity
- **Honest?** ✅ — partial data preserved on crash; transport
  failure modes are visible
- **Good UX?** ✅ — user can poke at the temp files; rerun
  merge if parent crashed

## Q4 (locked) — config: EDN canonical, wat-as-config future

User direction:

> *"i prefer edn for all the things.. if this is a wat express
> as config i'd love it..."*

**v1: EDN as canonical config format.** Static, statically
inspectable, the right amount of expressive power.

```edn
;; wat-cov.edn

{:thresholds
 {:overall {:line 90 :branch 80 :function 95}
  :per-file {:line 70}}

 :groups
 [{:name "src"   :pattern "src/**.wat"}
  {:name "tests" :pattern "wat-tests/**.wat"}]

 :filters
 {:exclude ["wat-tests/**"
            ":wat::core::*"
            ":wat::holon::*"]}

 :baseline
 {:enabled true
  :file ".wat-cov-baseline.edn"
  :refuse-drop true}

 :output
 {:formatters [:edn :cobertura]
  :paths {:edn "coverage.edn"
          :cobertura "coverage.xml"}}}
```

**v2 (future): wat-as-config option.** For users who want
programmatic config (dynamic thresholds based on file path
patterns, computed from build-time data, etc.):

```scheme
;; wat-cov.wat
(:wat::cov::config
  :thresholds (:HashMap :Symbol :i64
    :line 90 :branch 80 :function 95)
  :filters (:vec :String "wat-tests/**" ":wat::core::*"))
```

Loaded via `:wat::core::eval`. Same fields as the EDN version;
just expressed in wat. Defer to v2 unless a user surfaces a
concrete need.

The runner default search order:
1. `wat-cov.edn` in project root (canonical)
2. `wat-cov.wat` in project root (v2; programmatic)
3. `--config <path>` CLI flag override

Per the four questions:
- **Obvious?** ✅ — `wat-cov.edn` in the project root; reads
  top-down
- **Simple?** ✅ — one canonical format; one optional power
  format
- **Honest?** ✅ — config file is the source of truth; no
  hidden defaults outside it
- **Good UX?** ✅ — drop in the EDN; it works; reach for wat
  if you need programmatic config

## Q5 (locked) — refuse_coverage_drop

User direction: *"yea - totally have this"*

The killer SimpleCov feature for PR gating. Compares the
current run's coverage to a baseline; fails if THIS run is
worse.

**Mechanism:**
1. Baseline file `.wat-cov-baseline.edn` lives at project root
2. `wat cov check --baseline` compares current run to baseline
3. If any dimension regressed (line/branch/function), exit
   nonzero with diff
4. `wat cov update-baseline` rewrites the baseline file (after
   the user has accepted improvements OR the CI has merged)

**Configurable behavior** in `wat-cov.edn`:
```edn
:baseline
{:enabled true
 :file ".wat-cov-baseline.edn"
 :refuse-drop true              ;; fail on any regression
 :allow-drop-pct 1.0            ;; allow ≤1.0% drop (rare)
 :update-on-improvement false}  ;; don't auto-bump baseline
```

**Why this matters:** turns coverage from a vanity metric into
a guardrail. PRs that decrease coverage get caught; the bar
only goes up.

Per the four questions:
- **Obvious?** ✅ — baseline file is human-readable EDN
- **Simple?** ✅ — `cov check --baseline` does one job
- **Honest?** ✅ — diff is shown; you see exactly what
  regressed
- **Good UX?** ✅ — pre-commit / CI hook catches drops before
  merge

## Q6 (locked) — HTML as another renderer

User direction:

> *"could we do HTML 'trivially'?... if yes.. then yes.. its
> just another output renderer like json, edn, xml?...."*

Yes — HTML is "another renderer" architecturally. Implementation
cost is moderate (template engine + colored source listings)
but the pattern is right: **same EDN coverage data, different
output formatter.**

**Slice 4 deliverable.** Simple HTML output for v1:
- File table (per-file coverage % for each dimension)
- Per-file source listings with color-coded lines (green =
  covered, red = uncovered, yellow = partial branch)
- Sortable by coverage %
- No fancy SPA / search / tabs (those are downstream consumer
  territory)

Implementation: probably `askama` or `tera` templating in the
Rust shim. ~300-500 LOC of Rust + CSS + minimal HTML templates.

For more sophisticated HTML output (interactive dashboards,
trend graphs, etc.), users can take the EDN data and pipe it
through their own templating. wat-cov ships the canonical
"good enough" HTML; downstream takes over.

Per the four questions:
- **Obvious?** ✅ — files + lines + colors
- **Simple?** ✅ — one templating pass; no SPA framework
- **Honest?** ✅ — colors represent literal counter state
- **Good UX?** ✅ — open the HTML in a browser; navigate
  by file; see what's red

## All four formatters together

| Formatter | Audience | When |
|---|---|---|
| EDN | wat-aware tooling, scripts, future wat-cov consumers | default |
| Cobertura XML | CI ingestion (Jenkins, GitLab, GitHub Actions) | CI runs |
| JSON | non-wat tooling (most-of-the-world CI / dashboards) | --json flag |
| HTML | humans browsing coverage | --html flag |

All four are derived from the same EDN coverage data. Adding a
new formatter is "another renderer in `src/formatters/`."

## CLI integration

```
wat cov run <test-pattern>             # run tests with coverage on
wat cov report                         # emit report (default: edn)
wat cov report --cobertura             # emit Cobertura XML
wat cov report --json                  # emit JSON via wat-edn
wat cov report --html                  # emit HTML
wat cov check --minimum 80             # exit nonzero if below threshold
wat cov check --baseline               # compare to baseline; exit on regression
wat cov merge run1.edn run2.edn        # combine multiple runs
wat cov update-baseline                # rewrite .wat-cov-baseline.edn
```

Same wat-cli subcommand pattern as wat-fmt and wat-lint. Exit
code contract:
- `0` — coverage met (or no thresholds checked)
- `1` — threshold not met (in `--check` mode)
- `2` — parse error or runtime error
- `3` — IO error (can't write report etc.)

## Public API

### Rust API

```rust
// Run coverage over a test invocation
pub fn run_with_coverage<F: FnOnce()>(
    test_fn: F,
) -> Result<CoverageReport, CoverageError>;

// Read existing coverage data
pub fn report_from_edn(edn: &str) -> Result<CoverageReport, CoverageError>;

// Merge multiple reports
pub fn merge(reports: &[CoverageReport]) -> CoverageReport;

// Threshold checks
pub fn check_thresholds(
    report: &CoverageReport,
    config: &Config,
) -> Result<(), ThresholdViolations>;

// Baseline comparison
pub fn check_baseline(
    report: &CoverageReport,
    baseline: &CoverageReport,
    config: &Config,
) -> Result<(), BaselineViolations>;

// Format
pub fn to_cobertura(report: &CoverageReport) -> String;
pub fn to_edn(report: &CoverageReport) -> String;
pub fn to_json(report: &CoverageReport) -> String;
pub fn to_html(report: &CoverageReport) -> String;
```

### wat API

Per arc-013 contract: `wat_sources()` + `register()`. The wat
code exports:

```scheme
;; Threshold checking in wat
(:wat::cov::check-thresholds
  (report :wat::cov::Report)
  (config :wat::cov::Config)
  -> :wat::core::Result<:wat::core::unit, :wat::cov::Violations>)

;; Filter logic in wat (which files to include / exclude)
(:wat::cov::apply-filters
  (report :wat::cov::Report)
  (filters :wat::cov::Filters)
  -> :wat::cov::Report)
```

Most of wat-cov's logic is Rust (instrumentation hooks, counter
math, format emission); the wat layer is thin (filters,
threshold rules, custom formatters if any).

## Performance

Coverage tracking adds overhead to every `eval` call in the
vm. Two modes:

- **Off (default):** zero overhead. The vm checks a global
  boolean once per eval; if off, no counter operations happen.
  Compile-time elision via `#[cfg(feature = "coverage")]` if
  the user doesn't need coverage at all.
- **On:** roughly 2-5x slowdown on eval-heavy workloads
  (rough estimate; needs benchmarking). Acceptable for test
  runs; not acceptable for production.

The `wat cov run` command turns coverage on; everything else
runs normally.

If performance becomes a real concern, the natural optimization:
- Sampling coverage (count every Nth eval) for very long runs
- Counter compression (RLE per file) for memory savings on
  large test suites

## What goes into wat-rs proper vs wat-cov

**`wat-rs/src/`** (Rust runtime + parser):
- The standard `eval` machinery (already there)
- Coverage hooks (NEW — `eval` checks a global flag and
  optionally increments counters)
- The `CoverageState` struct (NEW — global coverage tracker)

**`wat-rs/crates/wat-cov/`** (self-contained crate):
- `src/instrument.rs` — registers/manages the global
  `CoverageState`; provides on/off switch
- `src/accumulate.rs` — counter merging, threshold checks
- `src/formatters/` — Cobertura, EDN, JSON, HTML
- `src/baseline.rs` — refuse_coverage_drop logic
- `wat/cov/` — wat-coded filter / threshold rules

The minimal wat-rs change: add coverage hooks to `eval`. Should
be a small, gated addition; shouldn't affect non-coverage
performance.

## Open architectural questions

Three flagged for slice-time decisions; all guided by the four
questions:

A. **Counter storage allocation strategy.** `HashMap<NodeId,
   HitCount>` works but allocates per-node. Two alternatives:
   (a) flat `Vec<u64>` indexed by node id (faster; needs
   pre-pass to assign ids); (b) sparse map with bumpalloc.
   Decide during slice 1 implementation; profile.

B. **Per-thread counter merging.** Multiple threads in the
   same process incrementing counters → need atomics OR
   thread-local + merge-at-exit. Probably thread-local +
   merge for performance. Decide during slice 1 with the
   actual concurrency story.

C. **Branch coverage for elided arms.** A `:cond` with `:else`
   that's never taken — does the `:else` arm count as a
   "covered branch" since it's the default fallthrough? Or
   does it count as "not covered" because it never evaluated?
   Lean: treats `:else` like any other arm — must be evaluated
   to count as covered. (This matches how most coverage tools
   handle default branches.)

## What's NOT in scope

- **Code complexity metrics** (cyclomatic complexity etc.) —
  these are linter territory, not coverage.
- **Mutation testing** — separate tool; very different
  discipline.
- **Production tracing** — coverage is for tests, not for
  observability. Use `wat-telemetry` for production traces.
- **IDE integration** — wat-cov produces data; IDE plugins
  consume it. Plugin development is downstream.

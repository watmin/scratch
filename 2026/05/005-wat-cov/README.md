# wat-cov — the canonical wat code coverage tool

User direction (2026-05-03):

> "i want a code coverage thing who can measure branch coverage..
> uncalled funcs.. all the things a proper lang should have for
> this domain of concern.. i think Cobertura is the format to
> target (and some edn native one for ourselves?...)"

> "ruby has an excellent example... simple-cov ... that's my
> favorite of all the langs i've seen - just like rubocop i
> want to model simple-cov"

This arc captures the design pre-implementation. wat-cov is the
third foundation-tier crate in the ecosystem (after wat-fmt and
wat-lint), modeled on SimpleCov's discipline.

---

## What wat-cov is

Code coverage for wat code, three dimensions:

1. **Line coverage** — was this source line evaluated?
2. **Branch coverage** — for each `:if` / `:cond` / `:match`,
   was each arm taken?
3. **Function coverage** — was this `:define` / `:lambda` /
   `:defmacro` ever called?

Modeled on SimpleCov (Ruby) — auto-start at test load,
runtime-tracked counters, multiple formatters, threshold gates,
result merging from parallel test runs, baseline-drop
prevention.

## Architecture in one paragraph

Coverage is a **wat-vm concern, not a language concern.** Per
the user's direction, instrumentation hooks live inside the
wat-vm: every `eval` call optionally increments a per-AST-node
counter; controlled by a global on/off switch. Cheap when off;
precise when on. At test-run end, counters get serialized to
EDN per-process; hermetic test children write their counters to
a transport, parent merges, formatters render the unified
report (Cobertura XML for CI, EDN for wat-aware tooling, JSON
via wat-edn, HTML for humans).

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-cov/`.

Same arc-013 pattern as wat-fmt and wat-lint — Rust shim +
embedded wat code + own tests. wat-cli depends on wat-cov;
users who pull in the CLI get coverage for free as a transitive
dep.

```
wat-rs/crates/wat-cov/
  Cargo.toml           # depends on wat (../..), wat-macros,
                       #   wat-edn (--json + EDN parsing)
  src/                 # Rust shim + wat-vm hooks + formatters
  wat/cov/             # wat-coded coverage analysis
                       #   (filters, threshold rules, etc.)
  wat-tests/cov/       # wat-level tests
  tests/               # Rust harness + golden coverage outputs
```

## The two-phase architecture (consistent with wat-lint)

```
wat-rs        (parser, type checker, vm + coverage hooks)  — mechanical
  ↑
wat-cov       (counter accumulation, formatters,           — mechanical
               threshold checks, baseline comparison)
─────────────────────────────────────────────────────────────────
[user reads coverage report; decides what to do]           — human
─────────────────────────────────────────────────────────────────
[downstream LLM analysis, if user delegates]               — opt-in
```

Same line as wat-lint: wat-cov stops at structured findings.
The user (or an opt-in LLM) decides what to do with them.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture: vm-level instrumentation, coverage dimensions, macro/quasiquote/substrate attribution rules, hermetic transport protocol, EDN config (with wat-as-config future option), formatters (Cobertura/EDN/JSON/HTML), refuse_coverage_drop, the four questions applied. |
| `SLICE-PLAN.md` | 5 slices, sized; slice 1 = vm hooks + counter accumulation + EDN output. |

## Conventions inherited

From the wat-fmt + wat-lint arcs:

- **The four questions as design compass** (Obvious / Simple /
  Honest / Good UX) — applied to every architectural choice
- **Atomicity-and-signal principle** — coverage data is
  preserved verbatim; gaps are visible (uncovered lines,
  branches, functions); we don't statistically smooth
- **Arc-013 self-contained crate pattern** — bundled
  `wat_sources()` + `register()`
- **Developer-first output** — same EDN payload reads for
  humans and parses for machines; JSON via wat-edn for
  ecosystem ingestion; Cobertura XML for CI

## Cross-references

- **wat-fmt** at `scratch/2026/05/003-wat-fmt/` — sibling
  formatter; established the wat-coded-with-Rust-shim pattern
- **wat-lint** at `scratch/2026/05/004-wat-lint/` — sibling
  linter; established the developer-first EDN / `--json` /
  CLI integration patterns + LLM-out-until-end discipline
- **arc 013** at `wat-rs/crates/wat-lru/src/lib.rs` — the
  external-crate-shipping contract wat-cov inherits
- **arc 058-035** (fork-substrate) — kernel pipe primitives
  potentially useful for hermetic coverage transport
- **SimpleCov** (Ruby) — the canonical coverage tool wat-cov
  models on; auto-start, dimension-rich, formatter-pluggable,
  result-mergeable, baseline-drop-preventing

## Status

- **Captured:** 2026-05-03
- **Architecture:** locked (vm hooks; surface-evaluation
  attribution rule; hermetic transport via per-child file/pipe;
  EDN config; Cobertura+EDN+JSON+HTML formatters;
  refuse_coverage_drop)
- **Slice plan:** 5 slices sized; not opened
- **Bar to open as a real wat-rs arc:** wat-fmt slice 1 lands
  (so the established crate-shipping pattern is real); user
  signals "let's start"

# wat-lint — the canonical wat linter

User direction (2026-05-02): *"the thing i'm most keen on is the
formatter... [later, after wat-fmt locked] some wat-lint crate
we can model next who depends on the existence of wat-fmt?"*

This arc captures the design pre-implementation. wat-lint is
the mechanical companion to the spells — it produces structured
findings; spells (perspicere, vocare, complectens, ...) consume
them OPTIONALLY for LLM-judgment; the user is always in the
loop to decide whether to delegate.

---

## What wat-lint is

A canonical, mechanical linter for wat code. Reads source;
emits structured findings. Same self-contained crate pattern
as wat-fmt (per arc 013); same atomicity-and-signal principle
inherited from wat-fmt.

**wat-lint never calls an LLM.** It produces structured
findings; the user decides whether to read them directly,
script over them, or delegate to a spell for LLM judgment.

## Architecture in one paragraph

Source bytes → wat-rs parser → AST → wat-coded lint rules walk
the AST and emit structured findings → runner aggregates →
output formatter (EDN by default, JSON via wat-edn). Each lint
rule is a wat function; rules ship as crates (per arc 013) or
as project-local `.wat` files (per RuboCop's pattern). Output
is **developer-first** — same EDN payload reads as text for a
human and parses as data for a script.

## Where it lives

**Single self-contained crate:** `wat-rs/crates/wat-lint/`.

```
wat-rs/crates/wat-lint/
  Cargo.toml           # depends on wat (path = "../..") +
                       #   wat-fmt (path = "../wat-fmt") for
                       #   layout-aware rules + wat-edn for
                       #   --json output
  src/                 # Rust shim (runner + wat-vm bridge +
                       #   public API)
  wat/lint/            # the actual lint rules in wat
  wat-tests/lint/      # wat-level tests
  tests/               # Rust harness + golden findings
```

Same arc-013 contract as wat-fmt: `wat_sources()` exports the
embedded `.wat` files; `register()` for any Rust shim dispatch.

## The two-phase architecture

```
wat-rs        (parser, type checker, vm)         — mechanical
  ↑
wat-fmt       (format rules in wat)              — mechanical
  ↑
wat-lint      (lint rules in wat → structured    — mechanical
               findings)
─────────────────────────────────────────────────────────────
[user reads findings; decides what to do]        — human
─────────────────────────────────────────────────────────────
spells        (.claude/skills/<spell>/SKILL.md)  — opt-in LLM
              [user delegates findings to spell    judgment
               when they want interpretation]
```

The line between mechanical and interpretive is firm. wat-lint
stops at structured findings. Spells start where the user
hands off.

## Reading order

| File | What it captures |
|---|---|
| `INDEX.yaml` | Beat-by-beat capture, conventions, status |
| `README.md` | This file. Top-level orientation. |
| `DESIGN.md` | Architecture, crate layout, runner shape, output format, rune suppression pattern, custom-rule paths. |
| `LINT-RULES.md` | Initial concrete rules (the five complectens Phase-1 rules, plus placeholders for perspicere and vocare). |
| `SLICE-PLAN.md` | 5 slices, sized; slice 1 is the runner + complectens rules. |

## Conventions inherited from wat-fmt

The wat-lint design inherits these from the wat-fmt arc:

- **The four questions are the design compass.** *Obvious?
  Simple? Honest? Good UX?* — applied to every lint rule, every
  output format choice, every API decision.
- **Atomicity-and-signal principle.** wat-lint surfaces what
  the formatter exposes; suppression is conscious (rune) not
  silent.
- **Arc-013 self-contained crate pattern.** Same shape as
  wat-fmt; bundled `wat_sources()` + `register()`.
- **Developer-first output.** Same payload reads for both
  humans and machines; EDN by default; JSON via wat-edn for
  the JSON-everywhere ecosystem.

## Cross-references

- **wat-fmt** at `scratch/2026/05/003-wat-fmt/` — the formatter
  arc; wat-lint's sibling and (for layout-aware rules) optional
  dependency.
- **complectens spell** at
  `wat-rs/.claude/skills/complectens/SKILL.md` — first user of
  wat-lint; specifies the initial five Phase-1 rules.
- **arc 130** at
  `wat-rs/docs/arc/2026/05/130-cache-services-pair-by-index/`
  — origin of the complectens discipline; calibration set for
  the rules.
- **holon-lab-trading spells** at
  `holon-lab-trading/.claude/skills/{sever,reap,ignorant}/` —
  where the `rune:<spell>(<category>) — <reason>`
  suppression pattern was born; wat-lint inherits it verbatim.
- **wat-edn** at `wat-rs/crates/wat-edn/` — the EDN<->JSON
  bridge; wat-lint depends on it for `--json` output.
- **arc 013 / external-crate contract** at
  `wat-rs/crates/wat-lru/src/lib.rs` — reference implementation
  of the `wat_sources` + `register` pattern.

## Status

- **Captured:** 2026-05-02
- **Architecture:** locked (two-phase mechanical+spell;
  arc-013 crate; rune suppression; EDN/JSON output)
- **First concrete rules:** the five complectens Phase-1 rules
  (per `LINT-RULES.md`)
- **Slice plan:** 5 slices sized; not opened
- **Bar to open as a real wat-rs arc:** wat-fmt slice 1 lands
  (so wat-lint can depend on a real Rust crate); user signals
  "let's start"

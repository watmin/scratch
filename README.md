# scratch

Durable thinking artifacts. Things the user has thought clearly
but not yet polished. Notes survive Claude conversation
compaction because they live on disk outside the conversation
history.

The user's scratch repo. Not a production codebase; not a repo
of record. Just durable.

## Top-level meta-vision

Three companion documents at scratch root articulate the
work's meta-frame. Read them after orienting to the per-arc
work; they tie the arcs together.

**`FUNCTIONS-ARE-REALITY.md`** — the WHY. The cosmological
recognition that drives the language and substrate choices.
Functions are the most primitive unit of reality (π is a
function; the wave function is at the bottom; Einstein's
equations at the top; DNA / memes / metabolism / WiFi / LLM
inference all functions). Once you see this, Lisp becomes the
only honest expressive language. The wat substrate is the
user's expression of this recognition.

**`FAILURE-ENGINEERING.md`** — the DISCIPLINE. The art of
removing failure from systems. Failure isn't recovered from;
it is read. Stop immediately when failure surfaces; eliminate
the CLASS, not the symptom. The five Honest ✅✅✅ wins across
the recent arcs are failure engineering applied at the
architectural layer; the wards are failure engineering
applied at code-review time; the four questions are failure
engineering applied at design time. User-coined term;
articulated in `wat-rs/docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`.

**`WAT-NETWORK.md`** — the WHAT. The architectural target the
per-arc work is building toward: a network of mutually-
authenticating wat-vms with cryptographic identity (mTLS),
content-addressed programs (digest forms), and verifiable
execution (signed eval forms). Each arc in `2026/05/003-008`
is a piece of this larger architecture; the WAT-NETWORK doc
is the whole.

Plus the per-arc bookworthy material in `2026/05/008-kwarg-macros/`
(`FOR-THE-BOOK.md`, `SYMBIOSIS.md`) — the HOW. The
collaboration shape; the four-questions discipline; the
triple-checkmark moments; the work-as-it-happens.

WHY (recognition) → DISCIPLINE → WHAT (architecture) → HOW
(collaboration). Source material for a future BOOK chapter.

## A note for readers

This scratch references **private companion repositories**
that aren't included here:

- `wat-rs/` — the wat substrate (Lisp on Rust runtime,
  type system, evaluator, kernel). Many arcs reference its
  `docs/arc/` tree (e.g. `wat-rs/docs/arc/2026/04/058-...`)
  and its `crates/` tree (e.g. `wat-rs/crates/wat-lru/`).
  These paths point at private context.
- `holon-lab-trading/` — the trading-lab application that
  exercises the substrate. Some arcs reference its
  `docs/proposals/` and `docs/drafts/` trees.
- A `~/.claude/projects/.../memory/` layer with `feedback_*.md`,
  `project_*.md`, `user_*.md`, and `reference_*.md` entries
  that capture cross-conversation discipline. These are
  per-user; the references are honest historical artifacts.

The scratch is **the design half** of the work. The
implementation half lives in the private repos. Reading
scratch alone gives you the WHY and the WHAT; the HOW lives
in code you can't see from here.

This is intentional. Some arcs may eventually graduate into
public substrate work; until then, the scratch is the public
trace of how the design surfaces.

## Organization

```
YYYY/MM/NNN-slug/         Arc directories — multi-file thoughts.
                          Each arc has its own INDEX.yaml and
                          README.md inside.

*.md at root              Orphan notes — pre-arc thinking that
                          hasn't cohered into a multi-file shape
                          yet. Migrate into an arc dir once it
                          coheres.
```

When a root-level note grows companions and becomes a multi-beat
thread, move it into a new arc directory and seed
`YYYY/MM/NNN-slug/INDEX.yaml` to catalog the beats.

## Current arcs

```
2026/04/001-axiomatic-surface/                   closed (BOOK chapters 62 + 63)
2026/04/002-directed-evaluation/
2026/04/003-edn-typed-wire/
2026/04/004-inverse-hologram/
2026/04/005-wat-pry/
2026/04/006-wat-mcp/
2026/04/007-dependency-resolution/
2026/04/008-reactor-for-program-supertype/
2026/04/009-substrate-fqdn-userspace-shorts/
2026/04/010-clojure-emits-wat/
2026/04/011-wat-common-flavor-comparison/
2026/04/012-wat-as-polyglot-lowering-target/
2026/04/013-wat-common-erlang-flavor/
2026/05/001-memory-as-hologram/
2026/05/002-og-wat-lineage/
2026/05/003-wat-fmt/                             foundation tier — formatter
2026/05/004-wat-lint/                            foundation tier — linter
2026/05/005-wat-cov/                             foundation tier — coverage
2026/05/006-wat-doc/                             foundation tier — documentation
2026/05/007-remote-program/                      app tier — typed remote calls (in-progress design)
2026/05/008-kwarg-macros/                        substrate-tier pattern; remote-program dep
2026/05/009-wat-http-serve/                      app tier — Rack analog; tokio+hyper shim + minimal handler interface
2026/05/010-wat-http-route/                      app tier — Sinatra analog; routing DSL on top of arc 009
2026/05/011-wat-http-client/                     app tier — HTTP client (the other end of arc 009); reqwest-based
```

The `2026/05/003-006` arcs together form the **foundation
toolkit quartet** — a sketched-toward-parity design for
formatter, linter, coverage, documentation. None of them have
shipped as code yet; the arcs are the design.

Each arc dir has its own `INDEX.yaml` listing the beats inside it
and a `README.md` orienting a reader to what's there. Open the
arc dir to start reading.

## Conventions

- **Filenames:** kebab-case `.md`, descriptive of the topic.
- **Voice capture:** in each file, capture the user's words,
  corrections, and repetitions. Attribute thinking-with
  extensions to the conversation, so future-us knows what was
  original insight vs collaborative articulation.
- **Quote the user verbatim** at decision points; don't
  paraphrase the load-bearing lines.
- **Status field** in each note's INDEX entry: `captured-raw` /
  `closed` / `open` / `migrated`.

## Orphan notes currently at root

- `wat-common-short-names.md` — design notes from wat-rs arc 109
  slice 1e (2026-05-01); pre-arc thinking. Could migrate into a
  proper arc dir if it grows into multi-file work.
- `random-notes.txt` — user-maintained text notes. Not authored
  by Claude tools. Do not modify without user direction.

## Migration log

- **2026-05-02** — 10 files belonging to the 001-axiomatic-surface
  arc moved from scratch root into
  `2026/04/001-axiomatic-surface/` (where the arc's `INDEX.yaml`
  had already been authored). Old root `INDEX.yaml` (which
  catalogued those 8 raw beats from the older "everything at
  root" organization) was dropped — the 001 arc's `INDEX.yaml`
  is more recent and more complete.

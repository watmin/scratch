# scratch

Durable thinking artifacts. Things the user has thought clearly
but not yet polished. Notes survive Claude conversation
compaction because they live on disk outside the conversation
history.

This directory is the user's private scratch repo
(`git@github.com:watmin/scratch.git`). Not a production
codebase; not a repo of record. Just durable.

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
```

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

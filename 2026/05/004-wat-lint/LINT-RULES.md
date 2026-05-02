# wat-lint — LINT RULES

The first concrete rules wat-lint v1 ships. Mechanical only —
each rule emits structured findings; spells (perspicere, vocare,
complectens) consume optionally for LLM judgment.

The five Phase-1 rules are derived directly from the complectens
spell at `wat-rs/.claude/skills/complectens/SKILL.md` (which
itself was triggered by arc 130's failed sweep — the canonical
calibration set lives at
`wat-rs/docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/`).

Per the user direction, rules deferred for perspicere and vocare
go in their own sub-sections as placeholders. They formalize
when each spell's mechanical phase has been written down.

---

## §1 — complectens family ✅ (concrete; from arc 130)

Arc 130 origin per complectens SKILL.md:

> *"docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md
> — the canonical doc that named the discipline.
> docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/
> — the failed sweep preserved as the calibration set."*

Five mechanical rules. Each one is a candidate-detector; the
spell's Phase-2 judgment renders verdicts.

### Rule 1.1 — `complectens/deftest-body-length`

**What it counts:** lines in a `(:wat::test::deftest ...)` body
(or an alias from a `make-deftest` factory).

**Thresholds** (per complectens SKILL.md):
- `> 30 lines` → L3 candidate (suspect)
- `> 50 lines` → L1 candidate (likely lie)
- `> 100 lines` → L1 candidate (definite lie)

**Rune categories:**
- `complectens(monolithic)` — scenario is irreducibly long;
  decomposing would lose an integration property
- `complectens(narrative)` — body is long because it documents
  a journey-narrative test (rare; usually replaceable by named
  helpers)

**Finding shape:**
```edn
{:rule "complectens/deftest-body-length"
 :severity :L1-candidate
 :file "wat-tests/lru/CacheService.wat"
 :line 145
 :context {:deftest-name ":test-cache-round-trip"
           :body-line-count 87
           :threshold-crossed :L1-likely}
 :message "deftest body is 87 lines (>50 likely L1; >100 definite L1)"
 :hint "extract layered helpers; each gets its own deftest"}
```

### Rule 1.2 — `complectens/let-star-binding-count`

**What it counts:** entries in a top-level `(:wat::core::let* ((...) ...) body)`
form within a deftest body.

**Threshold:** `> 10 entries` → L1 candidate.

**Why sharper than line count:** binding count is exactly the
"how many anonymous units of work am I claiming to test?"
metric.

**Rune categories:**
- `complectens(monolithic)` — same as Rule 1.1; the binding
  count reflects the scenario's irreducible complexity
- `complectens(setup-cascade)` — binding count is high because
  the scenario requires extensive deterministic setup that has
  been intentionally laid out for top-down readability

**Finding shape:** as Rule 1.1, with `:context :binding-count`
instead of `:body-line-count`.

### Rule 1.3 — `complectens/forward-reference`

**What it scans:** for each `:wat::core::define` of a helper, grep
for references to helpers / aliases NOT yet defined above it
in the same file.

**Threshold:** any forward reference → L1 candidate.

**Rune categories:**
- `complectens(forward-reference)` — known forward-ref needed
  because of macro auto-recursion (e.g., `make-deftest`
  referencing the alias it just registered) or substrate
  primitive that legitimately uses forward declaration
- `complectens(circular-test-helpers)` — two helpers
  legitimately call each other; co-recursive test discipline

**Finding shape:**
```edn
{:rule "complectens/forward-reference"
 :severity :L1-candidate
 :file "wat-tests/foo.wat"
 :line 45
 :context {:helper-defined ":test::layer-1-helper"
           :forward-references-to [":test::layer-2-helper"]
           :referenced-at-line 200}
 :message "helper :test::layer-1-helper references :test::layer-2-helper which is defined later (line 200)"
 :hint "test files MUST read top-down; refactor so dependencies flow upward only"}
```

### Rule 1.4 — `complectens/stepping-stone-multi-file`

**What it scans:** `find` for groups of `step-*.wat` /
`proof_*.wat` files in the same directory.

**Threshold:** any multi-file stepping-stone family → L2
candidate.

**Rune categories:**
- `complectens(stepping-stone)` — multi-file split is
  intentional because the scenarios are genuinely independent
  but related (rare; usually a refactor target)
- `complectens(historical)` — files are kept as historical
  record (e.g., proof_004's preserved exploratory phases);
  use this rune sparingly

**Finding shape:**
```edn
{:rule "complectens/stepping-stone-multi-file"
 :severity :L2-candidate
 :file "wat-tests/pipes/"
 :line 1
 :context {:file-family ["step-A.wat" "step-B.wat" "step-C.wat"
                         "step-D.wat" "step-E.wat"]
           :family-pattern "step-*.wat"}
 :message "stepping-stone family of 5 files; complectens discipline says ONE file"
 :hint "consolidate into one file with named, layered helpers + per-layer deftests"}
```

### Rule 1.5 — `complectens/helper-without-deftest`

**What it scans:** for each `:wat::core::define` in a
`make-deftest` prelude or at file top-level, search for a
sibling `(:deftest ...)` referencing it.

**Threshold:** any helper without a sibling deftest → L2
candidate.

**Rune categories:**
- `complectens(helper-orphan)` — helper has no deftest by
  design (e.g., a thin wrapper used in exactly one place; the
  parent deftest exercises it transitively)
- `complectens(scaffolding)` — helper is part of in-progress
  layered build; the deftest is coming in the next slice

**Finding shape:**
```edn
{:rule "complectens/helper-without-deftest"
 :severity :L2-candidate
 :file "wat-tests/lru/CacheService.wat"
 :line 67
 :context {:helper-name ":test::lru-spawn-then-put"
           :helper-defined-line 67
           :siblings-found ["test-cache-round-trip"]
           :no-direct-test true}
 :message "helper :test::lru-spawn-then-put has no direct sibling deftest"
 :hint "extract a per-layer deftest that exercises this helper alone"}
```

---

## §2 — perspicere family ❓ DRAFT (placeholder)

Per the perspicere spell at
`wat-rs/.claude/skills/perspicere/SKILL.md`, this spell sees
through deeply-nested type expressions and suggests typealiases
that name the noun the depth was hiding.

**Mechanical rules expected** (will formalize when the spell's
Phase-1 mechanics are written down):

- `perspicere/type-depth` — flag type expressions over N levels
  deep (`:Foo<:Bar<:Baz<:Quux<:Wibble>>>>`); L1 candidate
  threshold TBD
- `perspicere/typealias-eligible-recurrence` — flag identical
  multi-level type expressions that appear N+ times in a file
  (likely should be aliased)
- `perspicere/typealias-shadow` — flag typealiases that hide
  semantically distinct noun-types behind a shared name

**Rune categories** (TBD per spell): `perspicere(structural)`
for unavoidably-deep types; `perspicere(intentional-recurrence)`
for repeats that are conscious.

**Status:** placeholder. Open when perspicere's mechanical
phase formalizes (probably triggered by a real lint pass on
existing wat code).

---

## §3 — vocare family ❓ DRAFT (placeholder)

Per the vocare spell at
`wat-rs/.claude/skills/vocare/SKILL.md`, this spell calls the
test to its caller — checks whether the test exercises the
caller's interface vs reaching into the implementation.

**Mechanical rules expected:**

- `vocare/private-call-from-test` — flag tests that call
  internal-only functions; L1 candidate
- `vocare/missing-public-coverage` — flag public API surface
  without corresponding test coverage at the public boundary;
  L2 candidate
- `vocare/test-bypasses-interface` — flag tests that construct
  internal state directly (e.g., struct literals) instead of
  going through public constructors

**Rune categories** (TBD): `vocare(integration)` for tests that
deliberately reach past the interface for integration validation;
`vocare(legacy)` for tests pre-dating an interface refactor.

**Status:** placeholder. Open when vocare's mechanical phase
formalizes.

---

## §4 — Layout-aware rules 🔧 DEFERRED (slice 5)

The lint rules that depend on wat-fmt's analyses. Examples:

- `layout/over-line-limit` — flag lines that would be over
  120 cols after `wat fmt`; L2 candidate
- `layout/post-format-drift` — flag forms whose layout would
  change significantly under `wat fmt` (canonicalize-then-diff)
- `layout/oversized-symbol` — flag symbols that push a line
  past 120 cols even after format (the "make a type alias"
  signal — already established in wat-fmt's Rule 23 / 13c)

**Rune categories:** TBD; likely `layout(unavoidable)` for the
oversized-symbol case where no alias makes sense.

**Status:** deferred to slice 5. wat-fmt slice 1 must land
first (so the layout-query API exists).

---

## §5 — User-extensible rules 🔧 OPEN

Custom rules from third-party crates or project-local
`wat-lint-rules/` directories follow the same shape as
built-in rules. Each rule:

1. Lives in its own `.wat` file
2. Exports a function with the canonical signature (TBD per
   Q2)
3. Documents its rune categories
4. Has at least one golden test (input + expected findings)

**Naming convention:** `<crate-or-org>/<rule-name>`. E.g.,
`my-org/no-shouting-deftest` for a project-local rule;
`acme-lints/no-todo-comments` for a third-party crate's rule.

**Status:** OPEN. The mechanism is locked (per DESIGN.md
custom-rule paths section); specific user-defined rules are
out of scope for wat-lint v1.

---

## How rules become Phase-2 verdicts

For reference (the spell's job, not wat-lint's):

The mechanical pass emits **candidates** (severity = L1/L2/L3
candidate). Spells render **verdicts** (Level 1 / Level 2 /
Level 3) by applying the four questions to each candidate:

- **Obvious?** When this candidate's site fails, will the
  failure narrow to a specific named layer?
- **Simple?** Is the surface form (deftest / helper /
  define) doing one thing or many?
- **Honest?** Do the names match what's there?
- **Good UX?** Can a fresh reader trace top-down?

A 30-line deftest body might be inherently complex (e.g., a long
match expression on a complex enum) and NOT be a Level 1 lie.
The line count is a candidate flag; the verdict requires reading
the actual code. That's why wat-lint stops at candidates and
spells render verdicts.

## Status

- **Captured:** 2026-05-02
- **Phase-1 rules locked:** the five complectens rules above
  are concrete (derived from the spell's specification + arc
  130's calibration set)
- **Phase-2 spell delegation:** the mechanical/judgment split
  is locked per DESIGN.md
- **Other spell families:** placeholders for perspicere + vocare;
  formalize when those spells' mechanical phases are written
- **Layout-aware rules:** deferred to slice 5 (wat-fmt
  dependent)
- **User-extensible rules:** mechanism locked; rule content
  out of scope for wat-lint v1

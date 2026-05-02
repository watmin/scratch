# wat-lint — DESIGN

Architecture, crate layout, runner shape, output format, rune
suppression, custom-rule discovery, the four questions applied
throughout.

Rule signature shape deferred per Q2; figure out during
implementation.

---

## The four questions are the design compass

User direction:

> *"is this obvious?... is this simple?... is this honest?... is
> this a good ux?... we need these to guide us..."*

Per the established discipline (memory: `feedback_four_questions.md`),
applied in order:

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape per
  concept; rules layer instead of branching.
- **Honest?** What's named matches what's there; no hidden
  state; no surprise side-effects.
- **Good UX?** A user/developer can do the right thing without
  ceremony.

Obvious + Simple + Honest must hold before Good UX is even
considered. This guides every decision in this arc.

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

User direction:

> *"i want to keep the llm out of the process until the very
> end... the user can choose to interpret the lint outputs or
> delegate to an llm to get a judgement call.. but before
> that... its all structural analysis..."*

**The line between mechanical and interpretive is firm.**

- wat-lint stops at structured findings. It never invokes an
  LLM. It never renders verdicts. It produces evidence.
- The user reads the EDN/JSON output and decides what to do.
  They might fix the code. They might add a rune. They might
  shrug.
- Optional next step: the user invokes a spell
  (`/complectens`, etc.) and delegates the findings to LLM
  judgment. The spell renders verdicts (Level 1 / 2 / 3) by
  applying the four questions to each finding's surrounding
  context.

The user is always in the loop to decide whether to delegate.

Per the four questions:
- **Obvious?** ✅ — running `wat lint` is mechanical only;
  surprise model invocations don't happen
- **Simple?** ✅ — one tool, one job; spells are separate
- **Honest?** ✅ — the architecture says what it does; no
  hidden interpretation
- **Good UX?** ✅ — quick CLI + scriptable output without
  network/model dependencies

## Crate layout — single self-contained per arc-013

User direction:

> *"i think this means we need wat-fmt to be in wat.. not
> rust?... so wat-lint can use the capability wat-fmt has to
> observe conditions being present?"*

> *"i don't know that i agree wat-fmt should be wat-rs ..... it
> should be completely isolated in wat-rs/crates/wat-fmt..."*

Same shape applies to wat-lint. Self-contained crate; arc-013
contract; lint rules are wat code; runner is a Rust shim.

```
wat-rs/crates/wat-lint/
  Cargo.toml                       # depends on:
                                   #   wat (path = "../..")
                                   #   wat-macros (path = "../wat-macros")
                                   #   wat-fmt (path = "../wat-fmt")
                                   #     -- for layout-aware rules
                                   #   wat-edn (path = "../wat-edn")
                                   #     -- for --json output
  src/
    lib.rs                         # public Rust API:
                                   #   lint(input: &str, rules: &[&str])
                                   #     -> Result<Vec<Finding>, LintError>
                                   #   lint_file(path: &Path, ...) -> ...
                                   # PLUS the arc-013 contract:
                                   #   wat_sources() -> &'static [WatSource]
                                   #   register(&mut RustDepsBuilder)
    invoke.rs                      # wat-vm invocation helper
    findings.rs                    # EDN finding deserialization
                                   #   helpers; typed Finding struct
                                   #   for Rust callers
    output.rs                      # EDN pretty-printer; JSON via
                                   #   wat-edn for --json mode
    runes.rs                       # rune detection in source
                                   #   (Rust because it's pre-parse
                                   #   text inspection)
  wat/
    lint/
      runner.wat                   # top-level entry:
                                   #   (:wat::lint::run target rules)
                                   #     -> :Vector<:Finding>
      complectens/                 # arc 130's spell, mechanical rules
        deftest-body-length.wat
        let-star-binding-count.wat
        forward-reference.wat
        stepping-stone-multi-file.wat
        helper-without-deftest.wat
      perspicere/                  # type-depth spell (when its
        ...                        #   mechanical rules formalize)
      vocare/                      # test-interface spell (when its
        ...                        #   mechanical rules formalize)
    primitives/
      ast-walk.wat                 # shared AST walking helpers
      finding.wat                  # structured finding constructor
      edn-emit.wat                 # EDN output formatter
  wat-tests/
    lint/
      complectens/                 # wat-level tests of each rule
  tests/
    test.rs                        # Rust harness running wat-tests
    golden/                        # input.wat + expected-findings.edn
                                   #   (one per Phase-1 rule;
                                   #    derived from arc 130's
                                   #    calibration set)
    properties.rs                  # round-trip + suppression-fidelity
                                   #   property tests
```

The pattern mirrors wat-fmt and the existing wat-shipping crates
(wat-lru, wat-holon-lru, wat-sqlite, wat-telemetry,
wat-telemetry-sqlite) exactly.

## Public API

### Rust API

For wat-cli and library consumers:

```rust
pub fn lint(
    input: &str,
    rules: &[&str],          // empty slice = all enabled rules
) -> Result<Vec<Finding>, LintError>;

pub fn lint_file(
    path: &Path,
    rules: &[&str],
) -> Result<Vec<Finding>, LintError>;

pub struct Finding {
    pub rule: String,        // e.g. "complectens/deftest-body-length"
    pub severity: Severity,  // CANDIDATE — not VERDICT
    pub file: String,
    pub line: u32,
    pub context: HashMap<String, Value>,  // rule-specific structured data
    pub message: String,     // human-readable
    pub hint: Option<String>,
    pub rune: Option<Rune>,  // if a rune suppressed this finding,
                             // the rune is reported (not omitted)
}

pub enum Severity {
    L1Candidate,             // threshold significantly exceeded
    L2Candidate,             // threshold mildly exceeded
    L3Candidate,             // informational; possibly intentional
}

pub struct Rune {
    pub spell: String,       // e.g. "complectens"
    pub category: String,    // e.g. "monolithic"
    pub justification: String,
    pub line: u32,
}

pub enum LintError {
    Parse(ParseError),       // input wasn't valid wat
    UnknownRule(String),
    // Lint errors are NOT possible — rules are total over a
    // parsed AST. Once parsed, lint always succeeds.
}
```

### wat API

Per the arc-013 contract, the wat code exports:

```scheme
(:wat::lint::run
  (target :wat::core::String)         ; raw source
  (rules :wat::core::Vector<:String>) ; empty = all enabled
  -> :wat::core::Vector<:wat::lint::Finding>)
```

Each individual rule exports:

```scheme
(:wat::lint::rules::<spell>::<rule-name>
  (input :wat::lint::Input)           ; carries both String + AST
                                      ; (signature TBD per Q2)
  -> :wat::core::Vector<:wat::lint::Finding>)
```

## Output format — developer-first

User direction:

> *"wat-lint ouput is developer-first ... humans or machines"*

> *"we should support --json since the whole planet uses json
> (for now... we'll get them to migrate to edn soon enough...)
> -- wat-edn provides an edn->json interface"*

The output is structured EDN by default. Same payload reads as
text for a human and parses as data for a script.

**Default output — pretty-printed EDN** (no flag needed):

```edn
{:findings
 [{:rule "complectens/deftest-body-length"
   :severity :L1-candidate
   :file "wat-tests/lru/CacheService.wat"
   :line 145
   :context {:deftest-name ":test-cache-round-trip"
             :body-line-count 87}
   :message "deftest body is 87 lines (>50 likely L1; >100 definite L1)"
   :hint "extract layered helpers; each gets its own deftest"
   :rune nil}
  {:rule "complectens/let-star-binding-count"
   :severity :L1-candidate
   :file "wat-tests/lru/CacheService.wat"
   :line 145
   :context {:deftest-name ":test-cache-round-trip"
             :binding-count 23}
   :message "let* has 23 entries (>10 indicates L1)"
   :rune {:spell "complectens"
          :category "monolithic"
          :justification "this scenario is irreducibly long"
          :line 144}}]
 :summary {:total 2 :L1 2 :L2 0 :L3 0 :suppressed-by-runes 1}}
```

Per the four questions:
- **Obvious?** ✅ — keys tell you what each field is
- **Simple?** ✅ — one schema; one shape; reads top-down
- **Honest?** ✅ — runes reported, not silently dropped
- **Good UX?** ✅ — pretty-printed EDN reads like sectioned text;
  parsers handle it without ceremony

**`--json` flag** — pipe the EDN through `wat-edn`'s edn→json
converter; emit JSON to stdout. Same data; different surface.

**`--check` flag** — exit nonzero if any L1+L2 findings exist
(suppressed-by-rune findings DON'T trigger nonzero — they're
conscious by definition). For CI / pre-commit hooks.

## Rune suppression — verbatim from holon-lab-trading

User direction:

> *"we need to have linter marks... i /really/ like how we do
> this with our spells.... study the holon-lab-trading spells
> for the idea... wat users can rune something as a declaration
> of 'i know this violates, i don't care, here's my justification'"*

Studied `sever`, `reap`, `ignorant` in
`holon-lab-trading/.claude/skills/`. The rune pattern is
established discipline; wat-lint inherits it verbatim, just
translated to wat's comment syntax.

### The rune syntax

```scheme
;; rune:complectens(monolithic) — this scenario tests fork-with-pipes
;;                                end-to-end; cannot decompose without
;;                                losing the integration property
(:wat::test::deftest :test-fork-with-pipes-end-to-end
  body...)
```

**Three required elements:**
1. `rune:<spell>(<category>)` — names which spell and which
   subtype the rune declares
2. `—` em-dash separator between rune marker and justification
3. Justification — REQUIRED. The site author must articulate
   why this rune is conscious.

**Three properties of the discipline** (verbatim from sever's
SKILL.md):
- *"Skip findings annotated with `rune:<spell>(category)` in a
  comment at the site."*
- *"The annotation must include a reason after the dash."*
- *"Report the rune so the human knows it exists, but don't
  flag it as a finding."*

> *"Runes suppress bad thoughts without denying their presence.
> A rune tells the ward: the datamancer has been here. This is
> conscious."*

### Where the rune attaches

The rune comment goes on the line BEFORE the offending site.
The matching is "next form after this comment block." If the
form spans multiple lines, the rune covers the whole form.

For wat-lint: rune detection is pre-parse text inspection (the
parser strips comments, but runes need to survive to the rule
evaluation phase). Implementation lives in `src/runes.rs`
(Rust); the rune metadata gets attached to the AST node before
rule evaluation.

### Rune categories per rule

Each lint rule documents its category vocabulary. Per the
complectens spell:

- `complectens(monolithic)` — long deftest body, can't decompose
- `complectens(forward-reference)` — known forward-ref needed
- `complectens(stepping-stone)` — multi-file split intentional
- `complectens(helper-orphan)` — helper has no deftest by design

The runner validates that a rune's category is a known category
for its spell; unknown categories trigger their own finding
("malformed rune; unknown category").

### Per the four questions

- **Obvious?** ✅ — `rune:<spell>(<category>) — <reason>` is
  unmistakable; no ambiguity about what's being suppressed
- **Simple?** ✅ — one rune syntax; one rule (skip + report);
  one comment-attached location
- **Honest?** ✅ — required justification forces consciousness;
  output reports the rune so suppression is visible (per
  `:rune` field in EDN output)
- **Good UX?** ✅ — comment-attached at the site means future
  readers see the rune next to the code; no out-of-band ignore
  file to consult

## Custom rules — RuboCop-modeled paths

User direction (Q1):

> *"how does rubocop do this?.... that's the model..."*

RuboCop has three integration paths:

1. **In-project rules** — drop `lib/rubocop/cop/custom/<rule>.rb`
   files; reference via `.rubocop.yml`'s `require:` array
2. **Gem-distributed rules** — create a Ruby gem subclassing
   `RuboCop::Cop::Base`; users `gem install` and add to
   `.rubocop.yml`'s `require:`
3. **Configuration** — single `.rubocop.yml` per project;
   declares enabled cops, severity overrides, paths, rule-specific
   config

For wat-lint, both Cargo and project-local paths converge to
"wat code in the wat-vm symbol table tagged as a lint rule":

### Crate-distributed rules

Custom-rule crate depends on `wat-lint` and follows the
arc-013 contract:

```rust
// my-custom-lints/src/lib.rs
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        wat::WatSource {
            path: "my-custom-lints/lint/no-magic-numbers.wat",
            source: include_str!("../wat/lint/no-magic-numbers.wat"),
        },
    ];
    FILES
}

pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    // no Rust types to surface; no-op
}
```

The user's project pulls in `my-custom-lints` via Cargo;
wat-lint discovers the registered rules in the wat-vm symbol
table.

### Project-local rules

Drop `.wat` files in a project-relative directory the runner
discovers:

```
<project>/
  Cargo.toml
  src/
  wat-lint-rules/             # discovered by wat-lint runner
    no-shouting-deftest.wat
    require-attribution.wat
```

Same wat function shape as crate-distributed rules; the runner
loads them at startup alongside the crate-registered rules.

### Configuration — `wat-lint.toml`

Single config file per project. Enables/disables rules, sets
severity overrides, configures rule-specific thresholds:

```toml
# wat-lint.toml

[rules]
# everything enabled by default; opt out individually
"complectens/deftest-body-length" = { enabled = true, l1-threshold = 50 }
"complectens/let-star-binding-count" = { enabled = true, l1-threshold = 10 }
"complectens/stepping-stone-multi-file" = { enabled = false }  # we like our
                                                               # stepping stones

# severity overrides
[severity]
"my-custom-lints/no-magic-numbers" = "L2"

# project-local rule discovery
[discovery]
project-rule-dirs = ["wat-lint-rules"]
```

Per the four questions:
- **Obvious?** ✅ — config file shows what's loaded and how
  severities are configured
- **Simple?** ✅ — one config; one mechanism; both crate and
  local rules go through the same registration
- **Honest?** ✅ — looking at config tells you what to expect
- **Good UX?** ✅ — drop a file → it works; depend on a crate →
  it works; same wat-lint.toml controls both

## Runner shape

```scheme
(:wat::lint::run
  (target :wat::core::String)         ; the file's text
  (rules :wat::core::Vector<:String>) ; rule names; empty = all enabled
  -> :wat::core::Vector<:wat::lint::Finding>)
```

Implementation:

1. Parse target with comment-preserving parser
2. Detect runes via pre-parse text inspection; attach rune
   metadata to corresponding AST nodes
3. For each rule in the rule set:
   - Invoke `(:wat::lint::rules::<rule-name> input)`
   - Collect returned findings
   - For each finding, check if a matching rune exists at the
     same site; if so, set `rune:` field and skip flagging
4. Aggregate findings; sort by file + line
5. Return

## Rule signature — DEFERRED per Q2

User direction:

> *"Q2 - uh... i don't know.. we can figure out what's more
> ergonomic... no.... the questions..."*

Per the four questions:
- **Obvious?** Reading the signature should tell you what the
  rule sees and what it produces.
- **Simple?** One canonical shape per rule; no rule-specific
  input/output negotiation.
- **Honest?** If a rule needs both raw String AND parsed AST,
  the input type carries both — no surprise side-loading.
- **Good UX?** Authoring a new rule should feel like writing
  any other wat function.

Sketched candidate (subject to revision during implementation):

```scheme
(:wat::core::define
  (:wat::lint::rules::<spell>::<rule>
    (input :wat::lint::Input)         ; carries text + AST + path
    (config :wat::lint::Config)       ; rule-specific config
                                      ;   (thresholds etc.)
    -> :wat::core::Vector<:wat::lint::Finding>)
  body)
```

The `Input` carries both raw text (for line counting / regex-ish
work) AND parsed AST (for structural rules). The rule picks
what it needs.

Lock during implementation; iterate via the four questions.

## Performance

Same considerations as wat-fmt: wat-vm interpretation overhead
is acceptable for v1. Lint runs are rare (on save, on commit,
on PR). Format the entire file at once; no incremental linting.

Target: ≤ 200ms for a 10k-line file on save. Above that,
optimize. The natural optimization paths:
- Skip files unchanged since last lint (mtime check)
- Memoize per-rule results keyed on AST hash
- Compile hot rules to Rust at build time (v2 only)

## What goes into wat-rs proper vs wat-lint vs the wat code

**`wat-rs/src/`** (Rust runtime + parser):
- Standard tokenizer + comment-preserving variant (graduated
  from wat-fmt when wat-lint ships, since both crates need it)
- HolonAST definition; type checker; wat-vm evaluator

**`wat-rs/crates/wat-lint/`** (self-contained crate):
- `src/` — Rust shim (runner, rune detection, output formatting,
  EDN/JSON conversion via wat-edn)
- `wat/lint/` — the actual lint rules in wat code
- `tests/` + `wat-tests/` — golden findings + property tests

**`wat-rs/crates/wat-fmt/`** (sibling — sometimes a dependency):
- For layout-aware lint rules, wat-lint depends on wat-fmt;
  rules call `:wat::fmt::*` to ask "would this wrap?",
  "what's the indent here?", etc.
- Most lint rules don't need wat-fmt (structural inspection
  doesn't depend on layout)

## Open architectural questions

Three flagged for slice-time decisions; all guided by the four
questions:

A. **Rune category vocabulary validation.** Should the runner
   enforce that a rune's `<category>` matches a known category
   for the spell, or treat unknown categories as warnings?
   Proposal: enforce; unknown category triggers a meta-finding
   ("malformed rune"). Per **Honest** — silent acceptance hides
   typos.

B. **Rule loading order.** When project-local rules and
   crate-distributed rules have name collisions, who wins?
   Proposal: project-local wins (consistent with most config
   systems); the runner emits a warning. Per **Obvious** — local
   override is the expected default.

C. **Multi-file lints.** Most rules are file-local. Some
   (helper-without-deftest, stepping-stone-multi-file) need to
   look across files. Proposal: rules can be marked
   `:cross-file` in their declaration; the runner loads the
   file-set into one batch before invoking. Per **Simple** —
   one mechanism for both file-local and cross-file rules.

## What's NOT in scope

- **Auto-fix mode.** wat-lint flags; wat-fmt formats; refactoring
  is a third tool (or never, if wat-lint findings + manual
  refactor cover the use cases). No `--fix` flag in v1.
- **LLM-mediated judgment.** Spells handle this separately.
  wat-lint is mechanical only.
- **Cross-language lints.** wat-lint lints wat code only. If
  Rust code needs linting, that's clippy's job.
- **Performance lints.** Profile-guided optimization hints,
  hot-path detection, etc. are downstream of a profiler, not
  the linter.

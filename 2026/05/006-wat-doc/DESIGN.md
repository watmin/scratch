# wat-doc — DESIGN

Architecture: substrate change for first-class docstrings,
extractor crate, four output formats, cross-references, wat-lint
integration. The four questions applied throughout.

Specific output-format details (HTML theme, Markdown flavor,
EDN schema specifics, code-example syntax) are TO BE REFINED
via chat (per user's "C after A" sequencing). This document
locks the load-bearing architecture; details follow.

---

## The convention — comments vs docstrings

User direction (2026-05-03):

> *"i think a convention... for wat-doc....
>
> ;; comments are for the developer's inner monolouge... 'this
> is what i want thinking when i made this...'
>
> docstrings are communicating instructions on how to use the
> thing... not detialing how the thing is made... yea?... that's
> the partiiton line... we can impose this as a convention?..
> not a linter/format thing.. (though we should have a rune for
> 'i'm not writing a doc string' and make it annoying to
> encourage good doc strings...)"*

The partition:

| Channel | Audience | Purpose |
|---|---|---|
| `;;` comments | The author (and future-author) | **Inner monologue** — "this is what I was thinking when I made this" |
| docstrings | The consumer | **Instructions** — "this is how to use this thing" |

Two audiences, two channels, complementary. A well-documented
form may have BOTH — a `;;` block above explaining design
tradeoffs and rejected alternatives (author's notes), plus a
docstring inside explaining the public contract (user's
instructions).

**The discipline this convention encodes:**

- Comments are first-person internal ("I went with X because Y";
  "rejected approach Z because W"; "TODO: revisit when N")
- Docstrings are second-person external ("call this with X to
  get Y"; "raises Err on empty input"; "see also `:other-fn`")
- Neither replaces the other; they serve different readers
- A form may have just comments (internal-only thinking that
  needs no public doc), just a docstring (clean public API
  with no tricky implementation), or both (rich public API
  with non-obvious implementation)

**Implications:**

1. **wat-doc extracts ONLY docstrings.** No fallback to leading
   `;;` comments. Comments are NEVER promoted to documentation
   by default. The earlier-proposed fallback (transition
   easement) is REMOVED from this design — per the convention,
   it would falsely promote inner monologue into user-facing
   docs.

2. **wat-fmt Rule 8 reframes** (small amendment to STYLE-RULES.md):
   the format rule is unchanged ("block comments above the form;
   no blank line between"), but the SEMANTIC shifts —
   comments express the AUTHOR'S INTENT, not the form's
   documentation. Format behavior stays; framing sharpens.

3. **wat-lint missing-docstring rule is L1 default** —
   annoyingly visible. Per the user direction: "make it
   annoying to encourage good doc strings." See the wat-lint
   integration section below for the full disposition.

4. **The migration story changes.** Existing wat code without
   docstrings doesn't get its comments auto-promoted to docs.
   It gets the lint warning (annoyingly). User either writes
   a docstring OR a rune. Comments stay where they are,
   serving their original audience (the author).

**Why this matters for the four questions:**

- **Obvious?** ✅ — two channels, two audiences; the difference
  is named explicitly, not inferred
- **Simple?** ✅ — one rule (comments are inner; docstrings are
  outer); no negotiated overlap
- **Honest?** ✅✅ — the author's notes don't pretend to be
  user docs; the user docs don't pretend to be implementation
  notes. Each says what it is.
- **Good UX?** ✅ — readers know where to look: scan the
  docstring for usage; scan the comments for context

This is a CONVENTION, not enforced syntax. The substrate
doesn't differentiate comments from docstrings semantically
(both are valid wat). The convention is enforced by:
- Tooling (wat-doc only reads docstrings; wat-lint flags
  missing docstrings)
- The community (linting + reviewing pressure)
- The discipline articulated here (this section is the
  authoritative source of the partition)

## The four questions are the design compass

Per the established discipline (carried from wat-fmt / wat-lint /
wat-cov arcs):

- **Obvious?** Reading the artifact tells you what it does.
- **Simple?** No speculative complexity; one canonical shape
  per concept.
- **Honest?** What's named matches what's there; docs cannot
  drift from forms because they live INSIDE the forms.
- **Good UX?** A user can do the right thing without ceremony.

Obvious + Simple + Honest must hold before Good UX is even
considered.

## The load-bearing decision — docstrings are first-class

User direction (verbatim):

> *"we could just add doc strings as first class citizens in our
> forms.... that's probably the honest thing to do?...."*

Through the four questions:

**Honest?** ✅✅ — load-bearing argument. A comment-above-form
is a TOOLING CONVENTION; a docstring-in-form is a SUBSTRATE
GUARANTEE. The doc IS part of the form's structure — it cannot
drift from the form because it lives inside the form. The
substrate type-checks it. Two parsers (with/without comment
preservation) see the same thing. Honest by construction.

**Simple?** ✅ slight edge — adds one tiny substrate concept
("string in position N is the docstring"), but eliminates the
entire comment-attachment machinery wat-doc would need from
comments-above. Net: simpler tooling.

**Obvious?** ✅ slight edge — the form's shape literally tells
you "this slot is for documentation." Comments-above requires
knowing the convention.

**Good UX?** wat is a Lisp; its paradigm family is Clojure /
Scheme / CL, not Rust / C. The host-language tiebreaker (which
normally points to Rust) does NOT apply here — wat's
grammatical lineage (per latin-in-wat: morphology over position)
says the form should carry its own meaning. Docstring-IN-form
is morphological. Docstring-as-comment is positional.

**Decision:** docstring as first-class.

## The substrate change

**Five forms get an optional docstring slot:** `define`,
`lambda`, `defmacro`, `typealias`, `struct`, `enum`.

Backwards-compatible (existing forms without docstrings still
valid):

```scheme
;; Old shape (still valid; backwards-compat):
(:wat::core::define (sig) body)

;; New shape (optional docstring between signature and body):
(:wat::core::define (sig) "Doc string." body)
```

**Function-shaped forms** (`define`, `lambda`, `defmacro`) —
docstring goes between signature and body, per the canonical
shape:

```scheme
(:wat::core::define
  (:my-fn (x :T) -> :U)
  "Compute the thing."
  body)
```

**Type-shaped forms** (`typealias`, `struct`, `enum`) —
docstring goes after the type's structural body. These forms
don't have a "body" position separate from their structure;
the docstring follows the structural definition:

```scheme
;; typealias with docstring (justifies the alias's existence)
(:wat::core::typealias :MyMap
  :HashMap<:Symbol, :i64>
  "A map keyed on symbol; values are signed counters.
   Used in the metrics layer; never construct directly —
   call :wat::metrics::make-bucket instead.")

;; struct with docstring (explains the type's domain)
(:wat::core::struct :Position
  ((file :i64)
   (rank :i64))
  "A position on a chess board. file ∈ [0, 7]; rank ∈ [0, 7].
   Origin (0, 0) is a1; (7, 7) is h8.")

;; enum with docstring (explains the variant set's purpose)
(:wat::core::enum :ParseResult<:T>
  ((Ok (value :T))
   (Err (error :ParseError)))
  "Parser output. Ok carries the parsed value; Err carries
   diagnostics with location information.")
```

User direction (2026-05-03) on type-shaped forms:

> *"i think typealiases can have a docstring... maybe not lint
> compain on it.. but its there if the user wants?... we'll
> make use of it to justify the type alias's existence...
> [...] same for struct and enum... that's a good idea..."*

**The asymmetry:** function-shaped forms (define / lambda /
defmacro) get LINT-FLAGGED for missing docstrings (high signal —
functions DO things; document what they do). Type-shaped forms
(typealias / struct / enum) get the docstring SLOT but no lint
pressure (lower signal — types NAME things; the name often
suffices).

Function-shaped forms: lint complains. Type-shaped forms: lint
doesn't complain (but the user CAN document if they want; often
the right move for type aliases that exist to "justify their
existence").

### Concrete examples

**`define` with docstring:**
```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Compute the thing. First sentence is the summary.

   Long-form explanation continues across lines per Rule 31."
  (:wat::core::* x x))
```

**`lambda` with docstring:**
```scheme
(:wat::core::lambda
  ((x :T)
   -> :U)
  "Doc string for this anonymous function."
  body)
```

**`defmacro` with docstring:**
```scheme
(:wat::core::defmacro
  (:my-macro
    (x :AST<T>)
    -> :AST<U>)
  "Doc string. Explains what the macro expands to."
  `(template ,x))
```

### Type-checker handling

The type checker's recognition rule:

```
A `define` / `lambda` / `defmacro` form has either:
- 2 args: (sig body)
- 3 args: (sig docstring body)
  where docstring is :wat::core::String

If 3 args and the middle is NOT a String, type error.
If 3 args and the middle IS a String, the docstring slot is
recognized; the body is args[2].
```

No runtime semantics impact. The docstring is metadata; the
runtime evaluator ignores it. Only tooling (wat-doc, wat-lint)
reads the docstring slot.

### Why between signature and body, not inside the signature

Three reasons:

1. **Signature stays purely about types.** Cleaner separation
   between "what the function takes/returns" and "what it
   does."
2. **Lambda symmetry.** Lambda has no name in the signature;
   placing the docstring outside the signature works
   identically for define/lambda/defmacro. Inside the signature,
   lambda would have an awkward "first arg vs first docstring"
   ambiguity.
3. **The docstring documents the WHOLE form, not just the
   signature.** It belongs between sig and body — adjacent to
   both — rather than nested inside one or the other.

### Why optional, not required

Backwards compatibility: existing wat code without docstrings
keeps working. Adoption is gradual; users add docstrings where
they help; wat-lint can later require docstrings on public
forms (see lint integration below).

## STYLE-RULES.md amendment cascade

Three rules in the wat-fmt arc need amendments:

### Rule 14 (`:define`) — add docstring slot

After the signature, before the body, the docstring (if
present) goes on its own line indented 2:

```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Docstring on its own line indented 2."
  body)
```

Multi-line docstrings follow Rule 31 (atomic; preserved
verbatim):

```scheme
(:wat::core::define
  (:my-fn
    (x :T)
    -> :U)
  "Docstring summary.

   Long-form explanation continues here. The continuation
   lines are part of the string content per Rule 31."
  body)
```

### Rule 14b (`:lambda`) — same pattern

```scheme
(:wat::core::lambda
  ((x :T)
   -> :U)
  "Lambda docstring."
  body)
```

### Rule 14c (`:defmacro`) — same pattern

```scheme
(:wat::core::defmacro
  (:my-macro (x :AST<T>) -> :AST<U>)
  "Macro docstring."
  `(template ,x))
```

These amendments are SMALL. The rest of each rule (signature
shape, body indent, vertical principle) is unchanged.

## wat-doc crate architecture

Same self-contained pattern as the other foundation crates:

```
wat-rs/crates/wat-doc/
  Cargo.toml                       # depends on:
                                   #   wat (path = "../..")
                                   #   wat-macros (path = "../wat-macros")
                                   #   wat-fmt (path = "../wat-fmt")
                                   #     -- shared parser + comment access
                                   #   wat-edn (path = "../wat-edn")
                                   #     -- for --json output
  src/
    lib.rs                         # public Rust API:
                                   #   document(input: &str) -> Result<DocTree, DocError>
                                   #   document_crate(...) -> ...
                                   # PLUS arc-013 contract:
                                   #   wat_sources() / register()
    extract.rs                     # AST walker; pulls (form,
                                   #   signature, docstring) triples
    cross_ref.rs                   # FQDN resolution; link generation
    formatters/
      edn.rs                       # EDN schema for the doc tree
      markdown.rs                  # Markdown emission
      html.rs                      # HTML site (codox-style)
      json.rs                      # via wat-edn
    invoke.rs                      # wat-vm bridge
  wat/
    doc/
      extract.wat                  # wat-coded extraction filters
                                   #   (which forms count as public)
      cross-ref.wat                # wat-coded cross-ref rules
      filters.wat                  # include/exclude patterns
  wat-tests/
    doc/                           # wat-level tests
  tests/
    test.rs                        # Rust harness
    golden/                        # input.wat + expected outputs
                                   #   (one per output format)
```

## Output formats

Four formats. Same EDN data model under the hood; different
renderers.

### EDN (canonical)

Same developer-first principle: structured data; humans can
read; machines can parse.

```edn
{:doc-tree
 {:crate "wat-fmt"
  :modules
  [{:fqdn ":wat::fmt::format"
    :kind :function
    :signature "(format (ast :HolonAST) -> :String)"
    :docstring "Format the given AST per the canonical wat-fmt rules."
    :file "wat/fmt/format.wat"
    :line 12
    :cross-refs [":wat::holon::HolonAST" ":wat::core::String"]}
   ...]}}
```

### Markdown

Embeddable in README files; per-form sections; cross-references
as Markdown links. Suitable for GitHub-rendered project docs.

### HTML

Browseable static site (codox-style):
- Index page with crate navigation
- Per-module pages
- Per-form sections with signature + docstring + cross-references
- Searchable (client-side; no server)
- Cross-references are working links

Templating: askama or tera (Rust templating). Asset bundling:
minimal (single CSS file, no JS framework).

### JSON

Via wat-edn. Same data; ecosystem-friendly format. For
non-wat tooling (e.g., a JS-based docs viewer).

## Macros and docstring attribution

User direction (2026-05-03):

> *"doc strings on the macro, not the results of the macro...
> we measure the surface forms.. if they inject new things..
> so be it.. the user didn't express them.. the macro did and
> the macro has a context string for it..."*

**The rule:** docstrings live on macros (`:defmacro` definitions);
expansions DON'T get separate docstrings. This is the surface-
attribution rule from wat-cov (Q2) applied to documentation:
count what the user wrote; the expansion is below the user-
visible surface.

**Concrete implications:**

1. **Macro definition** carries the docstring:
   ```scheme
   (:wat::core::defmacro
     (:my-macro (x :AST<T>) -> :AST<U>)
     "Doc string. Explains what the macro expands to and how
      to use it. The user reads THIS to understand the macro."
     `(template ,x))
   ```

2. **Macro invocation** (`(my-macro x)`) — wat-doc treats
   this as a call to `:my-macro`; the macro's docstring is
   what describes the call's behavior. No separate
   documentation needed at the invocation site beyond what
   the macro's docstring already says.

3. **Macro expansion** — the forms produced by the expansion
   do NOT get separate documentation. If the macro injects
   new defines / lambdas / variables, those are
   "below-surface" — the user didn't write them; the macro
   did. They inherit no docstring; they're not flagged by
   the missing-docstring lint (because they weren't written
   by the user).

4. **If the macro injects a `define`** (e.g., a `defstruct`
   macro that expands to a `define` for the struct's
   constructor), the injected `define` doesn't need a
   docstring. The macro's docstring is the canonical source
   of truth for what's being defined and why.

**Why this works:** the user reads the macro's docstring to
understand the macro. The expansion is implementation; treating
expanded forms as needing their own docstrings would force
macro authors to either:
- Inject docstring values into every expanded form (verbose
  and redundant)
- Get lint-flagged for every expansion (noisy and wrong)

The surface-attribution rule sidesteps both. One docstring per
macro; expansion is below the surface; lint scopes only over
user-written forms.

**Per the four questions:**
- **Obvious?** ✅ — one docstring per macro; expansion is
  internal
- **Simple?** ✅ — same surface rule as wat-cov; one principle
  applied across the toolkit
- **Honest?** ✅ — macro author writes the docstring once;
  it describes the user-visible contract; expansion is the
  author's implementation
- **Good UX?** ✅ — macro consumers see the macro's
  documentation; macro authors aren't penalized for what
  their expansions inject

## Cross-reference resolution

When a docstring mentions an FQDN like `:wat::holon::Bind`, the
formatter renders it as a link to that form's documentation
page.

**Mechanism:**
1. Parse the docstring text; identify FQDN-shaped tokens
2. Look up each FQDN in the doc tree
3. If found, render as link in HTML / Markdown
4. If not found, render as plain text + warn (potentially a
   wat-lint rule: `documentation/broken-cross-ref`)

**FQDN detection:** a token starting with `:` and matching
`:[a-zA-Z_][a-zA-Z0-9_]*(::[a-zA-Z_][a-zA-Z0-9_]*)*` is an
FQDN candidate. Walk the regex; check against the doc tree.

**Code-example syntax in docstrings:** TBD (refine via C). Two
candidates:
- Markdown-style fenced code blocks: ` ```scheme ... ``` `
- Indented code blocks (4-space indent)

The Markdown style is more familiar; lean toward it.

## CLI integration

Same wat-cli subcommand pattern:

```
wat doc <crate-or-path>            # generate docs (default: edn)
wat doc <path> --markdown          # emit Markdown
wat doc <path> --html              # emit HTML site
wat doc <path> --json              # emit JSON via wat-edn
wat doc <path> --output ./docs/    # output directory
wat doc --check <path>             # exit nonzero on missing
                                   # docstrings (uses wat-lint)
```

Exit code contract:
- `0` — success
- `1` — missing docstrings detected (in `--check` mode)
- `2` — parse / runtime error
- `3` — IO error

## wat-lint integration

New lint rule:

### `documentation/missing-docstring`

**Scope:** `:wat::core::define`, `:wat::core::lambda`,
`:wat::core::defmacro` only. Type-shaped forms (`:typealias`,
`:struct`, `:enum`) are EXCLUDED from required-docstring scope
per the user's asymmetry direction (they CAN have docstrings
but lint doesn't pressure for them).

**No public/private distinction.** Per user direction
(2026-05-03):

> *"wat doesn't have public or private -- the user's need to
> be dilligent - i'm not going to police them"*

The lint applies to ALL function-shaped forms. The user runes
the ones they don't want documented (internal helpers, thin
wrappers, etc.). The discipline is the user's responsibility;
the lint just makes the gap loud.

(Note: the rule was originally proposed as
`documentation/missing-public-docstring`; renamed to
`documentation/missing-docstring` to reflect the scope —
there's no "public" filter to apply.)

**What it counts:** for each `:define` / `:lambda` / `:defmacro`,
is a docstring present?

**Default severity: L1-candidate.** Annoyingly visible per the
user direction: "make it annoying to encourage good doc
strings." Configurable to L2 or L3 via `wat-lint.edn` if a
project wants softer enforcement (e.g., during wholesale
migration). Default is loud.

**The discipline:** every public form is EITHER documented OR
runed. Both are conscious choices; both make intent visible.
The lint catches forms that are NEITHER (the silent "I forgot"
case the convention wants to surface).

**Rune categories:**
- `documentation(self-evident)` — name + signature is
  sufficient documentation; no docstring needed
  (e.g., `:wat::core::first` — name says it all)
- `documentation(transitional)` — code is in active migration;
  docstring will come in a follow-up
- `documentation(internal)` — form is internal-only; not
  user-facing API; lint scope shouldn't include it
  (could also handle via filter config instead of rune)
- `documentation(generated)` — form was generated by
  macro/tooling expansion; docstring N/A

**Finding shape:**
```edn
{:rule "documentation/missing-docstring"
 :severity :L1-candidate
 :file "wat/foo.wat"
 :line 12
 :context {:form-name ":wat::foo::bar"
           :form-kind :define}
 :message ":wat::foo::bar has no docstring"
 :hint "add a docstring as the second arg to define, OR
        suppress with `;; rune:documentation(<category>) — <reason>`"}
```

**The "annoying" disposition in practice:**
- L1-candidate by default → CI fails on `wat lint --check`
- Findings appear in standard reports (no opt-out by default)
- Hint includes BOTH paths: "write a docstring" OR "write a rune"
- Both paths are equally valid; the discipline is "be conscious"
  not "always document"

### `documentation/broken-cross-ref` (future)

Flags FQDN mentions in docstrings that don't resolve to a form
in the doc tree. Helps catch typos and stale references.

## Backwards compatibility

The substrate change is OPT-IN:

- Existing 2-arg forms (`(define sig body)`) continue to work
  unchanged
- Code without docstrings doesn't break; just generates docs
  with no description (or wat-lint flags as missing,
  annoyingly)
- Migration is gradual: add docstrings where they help;
  existing comments-above stay as their original audience —
  the AUTHOR (per the partition convention)

**No transitional fallback to comments.** Per the
comments-vs-docstrings partition (top of this document),
comments are NEVER promoted to documentation. The earlier-
proposed fallback (read leading `;;` comments as docs when no
docstring exists) was REMOVED — it would falsely promote
inner monologue into user-facing docs, violating the
partition.

The migration story is honest:
1. Existing wat code without docstrings → wat-lint flags
   (L1-candidate, annoyingly visible)
2. User decides per form: write a docstring OR write a rune
   (`;; rune:documentation(<category>) — <reason>`)
3. Comments above stay where they are; they continue to serve
   the author (their original audience)
4. Long-term, the docstring slot is the canonical home for
   user-facing docs; comments are the canonical home for
   author-facing notes

## Per-crate vs project-wide doc trees

**Per-crate** (default): each crate's documentation is a
self-contained tree. Cross-references within the crate work;
cross-references to other crates link to those crates' docs
if they exist; otherwise plain text.

**Project-wide composition** (future): when a project depends
on multiple crates, wat-doc can compose them into one unified
doc site. Like Rust's `cargo doc --workspace`. Defer to v2 or
slice 4 polish.

## Performance

Documentation generation is rare (build-time, not runtime).
Performance is not a concern for v1. Format the entire doc tree
at once; no incremental docs.

If future demand surfaces (large multi-crate workspaces with
slow doc builds), the natural optimization:
- Skip crates whose source hasn't changed since last doc build
- Cache the EDN doc tree per crate; only re-render formatters

## What goes into wat-rs proper vs wat-doc

**`wat-rs/src/`** (Rust runtime + parser + type checker):
- Add 3-arg form recognition to define / lambda / defmacro
- Type checker: optional String at position [1]; reject
  non-String at that position
- No runtime semantics changes (docstring is metadata)

**`wat-rs/crates/wat-doc/`** (self-contained crate):
- All extraction logic (Rust shim + wat code)
- All format emitters
- Cross-reference resolution
- CLI integration

**`wat-rs/crates/wat-fmt/`** (sibling — small amendment):
- Rule 14 / 14b / 14c amended in STYLE-RULES.md
- Per-form emitters in slice 1 add docstring rendering
  (one if-check per affected form)

**`wat-rs/crates/wat-lint/`** (sibling — adds new rule):
- `documentation/missing-public-docstring` rule
- `documentation/broken-cross-ref` rule (future)

## Open architectural questions

Three flagged for slice-time decisions or chat refinement (per
"C after A"):

A. **Code-example syntax in docstrings.** Markdown fenced code
   blocks (` ```scheme `) vs indented (4-space) blocks. Lean
   Markdown for familiarity; confirm via C.

B. **HTML site theme + navigation.** Codox provides a
   functional default; wat-doc could match that or design our
   own. Defer to slice 3 implementation.

C. **Multi-crate doc composition.** Per-crate (default) is
   slice 1-3; project-wide composition is slice 4 polish or
   v2.

## What's NOT in scope

- **Substrate-level documentation requirements.** wat-lint
  flags missing docstrings; the substrate doesn't require them.
  Enforcement is configurable per project.
- **Tutorial / guide content.** wat-doc generates API reference;
  long-form tutorials are written by humans in Markdown / HTML
  separately.
- **LLM-generated documentation.** Per the LLM-out discipline,
  wat-doc emits structured data; LLM-generated improvements
  are downstream tooling the user invokes explicitly.
- **Cross-language docs.** wat-doc documents wat. Rust
  documentation comes from rustdoc.
